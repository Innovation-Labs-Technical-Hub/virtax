//! Proves that another OS user cannot use a user's `virtaxd` (FR-CP01-002), at both layers:
//! the socket's filesystem permissions, and the daemon's own peer-credential check behind them.
//!
//! Linux only, and it needs passwordless `sudo` to create and remove a throwaway user (CI runners
//! have it). The daemon itself runs as the invoking user.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const OTHER_USER: &str = "virtax-other";

struct Cleanup {
    daemon: Option<Child>,
    dir: PathBuf,
    user_created: bool,
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        if let Some(mut d) = self.daemon.take() {
            let _ = d.kill();
            let _ = d.wait();
        }
        // Files under the socket directory may have been chmod'ed by root; remove as root.
        let _ = Command::new("sudo")
            .arg("rm")
            .arg("-rf")
            .arg(&self.dir)
            .status();
        if self.user_created {
            let _ = Command::new("sudo").args(["userdel", OTHER_USER]).status();
        }
    }
}

fn run(cmd: &mut Command) -> Result<std::process::Output, String> {
    cmd.output().map_err(|e| format!("{cmd:?}: {e}"))
}

fn sudo(args: &[&str]) -> Result<(), String> {
    let out = run(Command::new("sudo").arg("-n").args(args))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "sudo {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// `virtaxctl --json version` as the other user; returns (exit code, stdout).
fn as_other_user(ctl: &Path, socket: &Path) -> Result<(i32, String), String> {
    let out = run(Command::new("sudo")
        .args(["-n", "-u", OTHER_USER])
        .arg(ctl)
        .arg("--socket")
        .arg(socket)
        .args(["--json", "version"]))?;
    Ok((
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    ))
}

pub fn run_test(root: &Path) -> Result<(), String> {
    if !cfg!(target_os = "linux") {
        return Err("multiuser-test runs on Linux only".into());
    }
    sudo(&["true"]).map_err(|_| {
        "multiuser-test needs passwordless sudo (it creates a throwaway user)".to_string()
    })?;

    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo)
        .args([
            "build",
            "--locked",
            "--quiet",
            "-p",
            "virtaxd",
            "-p",
            "virtaxctl",
        ])
        .current_dir(root)
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("building virtaxd and virtaxctl failed".into());
    }
    let bin = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"))
        .join("debug");

    // A short, world-traversable directory: the other user must be able to reach the copied CLI,
    // and the socket path must stay under the Unix limit.
    let dir = PathBuf::from(format!("/tmp/virtax-mu-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut cleanup = Cleanup {
        daemon: None,
        dir: dir.clone(),
        user_created: false,
    };
    let ctl = dir.join("virtaxctl");
    std::fs::copy(bin.join("virtaxctl"), &ctl).map_err(|e| e.to_string())?;
    run(Command::new("chmod").arg("0755").arg(&dir).arg(&ctl))?;

    if run(Command::new("id").arg(OTHER_USER))?.status.success() {
        return Err(format!("user {OTHER_USER} already exists; remove it first"));
    }
    sudo(&[
        "useradd",
        "--system",
        "--no-create-home",
        "--shell",
        "/usr/sbin/nologin",
        OTHER_USER,
    ])?;
    cleanup.user_created = true;

    let socket = dir.join("run").join("d.sock");
    let daemon = Command::new(bin.join("virtaxd"))
        .arg("--mock-engine")
        .arg("--socket")
        .arg(&socket)
        .arg("--data-dir")
        .arg(dir.join("data"))
        .env("RUST_LOG", "warn")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("start virtaxd: {e}"))?;
    cleanup.daemon = Some(daemon);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !socket.exists() {
        if Instant::now() > deadline {
            return Err("virtaxd did not create its socket".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // 1. Default permissions (socket 0600 in a 0700 directory): the other user cannot connect at all.
    let (code, _) = as_other_user(&ctl, &socket)?;
    if code == 0 {
        return Err(
            "another user reached the daemon through the default socket permissions".into(),
        );
    }
    eprintln!("multiuser: default permissions refuse the other user (virtaxctl exit {code})");

    // 2. With the filesystem opened up by root, the daemon's peer check must still refuse.
    sudo(&["chmod", "0755", &dir.join("run").to_string_lossy()])?;
    sudo(&["chmod", "0666", &socket.to_string_lossy()])?;
    let (code, stdout) = as_other_user(&ctl, &socket)?;
    if code == 0 || !stdout.contains("VTX-API-403") {
        return Err(format!(
            "peer check did not refuse the other user (exit {code}): {stdout}"
        ));
    }
    eprintln!("multiuser: peer-credential check refuses the other user with VTX-API-403");

    // The owner still gets through.
    let owner = run(Command::new(&ctl)
        .arg("--socket")
        .arg(&socket)
        .arg("version"))?;
    if !owner.status.success() {
        return Err(format!(
            "the daemon's own user was refused: {}",
            String::from_utf8_lossy(&owner.stderr)
        ));
    }
    eprintln!("multiuser: ok");
    Ok(())
}
