//! End to end: a real `virtaxd` (MockEngine) on a private socket, driven by the real `virtaxctl` binary.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde_json::Value;

/// `virtaxd` lives in another package. Build it once, then reuse it across tests and restarts.
fn virtaxd_bin() -> &'static Path {
    static BIN: OnceLock<PathBuf> = OnceLock::new();
    BIN.get_or_init(|| {
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let status = Command::new(cargo)
            .args([
                "build",
                "--locked",
                "--quiet",
                "--package",
                "virtaxd",
                "--package",
                "virtax-vmm",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()
            .expect("run cargo build");
        assert!(status.success(), "building virtaxd failed");
        Path::new(env!("CARGO_BIN_EXE_virtaxctl"))
            .with_file_name(format!("virtaxd{}", std::env::consts::EXE_SUFFIX))
    })
}

struct Env {
    _data: tempfile::TempDir,
    run: tempfile::TempDir,
    data_dir: PathBuf,
    socket: PathBuf,
    daemon: Option<Child>,
    mock: bool,
}

impl Env {
    fn new() -> Self {
        let data = tempfile::tempdir().unwrap();
        // Unix socket paths are limited to ~104 bytes; keep this one short.
        let run = tempfile::Builder::new()
            .prefix("vtxe2e")
            .tempdir_in("/tmp")
            .unwrap();
        let mut env = Self {
            data_dir: data.path().to_path_buf(),
            socket: run.path().join("d.sock"),
            _data: data,
            run,
            daemon: None,
            mock: true,
        };
        env.start_daemon();
        env
    }

    /// A daemon with only the real engines (virtax-vmm on KVM, found next to virtaxd).
    fn without_mock() -> Self {
        let mut env = Self::new();
        env.stop_daemon();
        env.mock = false;
        env.start_daemon();
        env
    }

    fn daemon_cmd(&self) -> Command {
        let mut cmd = Command::new(virtaxd_bin());
        if self.mock {
            cmd.arg("--mock-engine").args(["--mock-latency-ms", "40"]);
        }
        cmd.arg("--socket")
            .arg(&self.socket)
            .arg("--data-dir")
            .arg(&self.data_dir)
            .env("RUST_LOG", "warn")
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        cmd
    }

    fn start_daemon(&mut self) {
        let log_path = self.run.path().join("virtaxd.stderr");
        let log = std::fs::File::create(&log_path).expect("open virtaxd log");
        let child = self
            .daemon_cmd()
            .stderr(log)
            .spawn()
            .expect("spawn virtaxd");
        self.daemon = Some(child);
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let status = self
                .daemon
                .as_mut()
                .unwrap()
                .try_wait()
                .expect("poll virtaxd");
            assert!(
                status.is_none(),
                "virtaxd exited during startup ({status:?}):\n{}",
                std::fs::read_to_string(&log_path).unwrap_or_default()
            );
            if self.socket.exists() && self.virtaxctl(&["version"]).status.success() {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "virtaxd did not answer the version RPC at {}:\n{}",
                self.socket.display(),
                std::fs::read_to_string(&log_path).unwrap_or_default()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn stop_daemon(&mut self) {
        if let Some(mut child) = self.daemon.take() {
            if matches!(child.try_wait(), Ok(Some(_))) {
                return;
            }
            let term = Command::new("kill").arg(child.id().to_string()).output();
            if term.is_ok_and(|out| out.status.success()) {
                let deadline = Instant::now() + Duration::from_secs(6);
                while Instant::now() < deadline {
                    match child.try_wait() {
                        Ok(Some(_)) => return,
                        Ok(None) => std::thread::sleep(Duration::from_millis(20)),
                        Err(_) => break,
                    }
                }
            }
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn virtaxctl(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_virtaxctl"))
            .arg("--socket")
            .arg(&self.socket)
            .args(args)
            .output()
            .expect("run virtaxctl")
    }

    /// Run with `--json`; return (exit code, parsed stdout). Stdout must be pure JSON.
    fn json(&self, args: &[&str]) -> (i32, Value) {
        let mut all = vec!["--json"];
        all.extend_from_slice(args);
        let out = self.virtaxctl(&all);
        let stdout = String::from_utf8(out.stdout).unwrap();
        let value = serde_json::from_str(stdout.trim())
            .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {stdout:?}"));
        (out.status.code().unwrap(), value)
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        self.stop_daemon();
    }
}

#[test]
fn machine_lifecycle_through_cli() {
    let mut env = Env::new();

    let (code, v) = env.json(&[
        "vm",
        "create",
        "--name",
        "web",
        "--os-type",
        "Ubuntu_64",
        "--cpus",
        "2",
        "--memory",
        "1024",
        "--group",
        "/lab",
    ]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["config"]["memory_mib"], 1024);
    let etag = v["etag"].as_str().unwrap().to_string();

    let (code, v) = env.json(&["vm", "create", "--name", "web"]);
    assert_eq!(
        (code, v["error"]["code"].as_str()),
        (4, Some("VTX-CFG-031"))
    );

    let (code, v) = env.json(&["vm", "list"]);
    assert_eq!(code, 0);
    assert_eq!(v.as_array().unwrap().len(), 1);
    assert_eq!(v[0]["state"], "powered_off");

    let (code, v) = env.json(&["vm", "start", "web"]);
    assert_eq!((code, v["state"].as_str()), (0, Some("succeeded")), "{v}");
    let (_, v) = env.json(&["vm", "list", "--state", "running"]);
    assert_eq!(
        v.as_array().unwrap().len(),
        1,
        "read model reflects the running state"
    );

    let (code, v) = env.json(&["vm", "start", "web"]);
    assert_eq!(
        (code, v["changed"].as_bool()),
        (0, Some(false)),
        "start is idempotent without --strict"
    );
    let (code, _) = env.json(&["vm", "start", "web", "--strict"]);
    assert_eq!(code, 5);

    let (code, v) = env.json(&["vm", "set", "web", "--memory", "2048", "--dry-run"]);
    assert_eq!(code, 0);
    assert_eq!(v["field_results"][0]["apply_mode"], "needs_power_off");
    let (code, v) = env.json(&["vm", "set", "web", "--memory", "2048"]);
    assert_eq!((code, v["error"]["code"].as_str()), (5, Some("VTX-RT-002")));

    let (code, _) = env.json(&["vm", "stop", "web", "--force"]);
    assert_eq!(code, 0);
    let (code, v) = env.json(&["vm", "set", "web", "--memory", "2048", "--etag", &etag]);
    assert_eq!(code, 0, "{v}");
    let new_etag = v["machine"]["etag"].as_str().unwrap().to_string();
    assert_ne!(new_etag, etag);

    // A stale etag loses: exit 4 and the current etag is reported.
    let (code, v) = env.json(&["vm", "set", "web", "--cpus", "4", "--etag", &etag]);
    assert_eq!(code, 4);
    assert_eq!(v["error"]["etag"].as_str(), Some(new_etag.as_str()));

    let (code, v) = env.json(&["ops", "list"]);
    assert_eq!(code, 0);
    assert_eq!(v.as_array().unwrap().len(), 2);

    // State survives a daemon restart through the registry and manifest.
    env.stop_daemon();
    env.start_daemon();
    let (_, v) = env.json(&["vm", "show", "web"]);
    assert_eq!(v["config"]["memory_mib"], 2048);
    assert_eq!(v["etag"].as_str(), Some(new_etag.as_str()));

    let (code, _) = env.json(&["vm", "delete", "web", "--delete-files"]);
    assert_eq!(code, 2, "--delete-files needs --yes");
    let (code, _) = env.json(&["vm", "delete", "web", "--delete-files", "--yes"]);
    assert_eq!(code, 0);
    let (code, v) = env.json(&["vm", "show", "web"]);
    assert_eq!(
        (code, v["error"]["code"].as_str()),
        (3, Some("VTX-API-404"))
    );
}

#[test]
fn second_daemon_is_refused() {
    let env = Env::new();
    let out = env.daemon_cmd().output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("VTX-API-409"));
}

#[test]
fn truncated_manifest_is_recovered_at_startup() {
    let mut env = Env::new();
    let (_, v) = env.json(&["vm", "create", "--name", "db", "--cpus", "2"]);
    let folder = PathBuf::from(v["folder"].as_str().unwrap());
    let (code, _) = env.json(&["vm", "set", "db", "--cpus", "6"]);
    assert_eq!(code, 0);
    env.stop_daemon();

    let manifest = folder.join("machine.toml");
    let text = std::fs::read_to_string(&manifest).unwrap();
    std::fs::write(&manifest, &text[..text.len() / 3]).unwrap();

    env.start_daemon();
    let (_, v) = env.json(&["vm", "list"]);
    assert_eq!(v[0]["name"], "db", "the machine is not dropped");
    assert_eq!(v[0]["accessible"], true);
    assert_eq!(v[0]["error_code"], "VTX-CFG-012");
    assert_eq!(v[0]["cpu_count"], 2, "restored from machine.toml.prev");
    assert!(std::fs::read_dir(&folder).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".corrupt-")
    }));
}

#[test]
fn validate_reports_line_and_column_without_daemon() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("machine.toml");
    std::fs::write(
        &path,
        "schema = \"virtax.machine/1\"\nid = \"not-a-uuid\"\n",
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_virtaxctl"))
        .args(["--json", "vm", "validate"])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let v: Value = serde_json::from_slice(&out.stdout).expect("stdout is a single JSON document");
    assert_eq!(v["valid"], false);
    assert!(v["issues"][0]["line"].is_number());
}

#[test]
fn events_watch_survives_daemon_restart_without_loss_or_duplicates() {
    use std::io::{BufRead, BufReader};
    use std::sync::mpsc;

    let mut env = Env::new();
    struct KillOnDrop(Child);
    impl Drop for KillOnDrop {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    let mut watcher = Command::new(env!("CARGO_BIN_EXE_virtaxctl"))
        .arg("--socket")
        .arg(&env.socket)
        .args(["--json", "events", "watch", "--since", "0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let (tx, rx) = mpsc::channel::<Value>();
    let stdout = watcher.stdout.take().unwrap();
    let watcher = KillOnDrop(watcher);
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            if tx
                .send(serde_json::from_str(&line).expect("each line is one JSON event"))
                .is_err()
            {
                break;
            }
        }
    });
    let wait_for = |kind: &str, events: &mut Vec<Value>| {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let e = rx
                .recv_timeout(left)
                .unwrap_or_else(|_| panic!("no {kind} event; got {events:?}"));
            let done = e["kind"] == kind;
            events.push(e);
            if done {
                return;
            }
        }
    };

    let mut events = Vec::new();
    assert_eq!(env.json(&["vm", "create", "--name", "ev"]).0, 0);
    assert_eq!(env.json(&["vm", "start", "ev"]).0, 0);
    wait_for("operation.finished", &mut events);

    env.stop_daemon();
    std::thread::sleep(Duration::from_millis(300)); // the watcher is now reconnecting
    env.start_daemon();
    assert_eq!(
        env.json(&["vm", "set", "ev", "--os-type", "Debian_64"]).0,
        0
    );
    assert_eq!(env.json(&["vm", "delete", "ev"]).0, 0);
    wait_for("machine.deleted", &mut events);
    drop(watcher);

    let seqs: Vec<u64> = events.iter().map(|e| e["seq"].as_u64().unwrap()).collect();
    assert_eq!(
        seqs,
        (1..=seqs.len() as u64).collect::<Vec<_>>(),
        "gap-free and duplicate-free: {seqs:?}"
    );
    let kinds: Vec<&str> = events.iter().map(|e| e["kind"].as_str().unwrap()).collect();
    for k in [
        "machine.created",
        "operation.updated",
        "machine.state_changed",
        "operation.finished",
        "machine.updated",
        "machine.deleted",
    ] {
        assert!(kinds.contains(&k), "missing {k} in {kinds:?}");
    }
    let updated = events
        .iter()
        .find(|e| e["kind"] == "machine.updated")
        .unwrap();
    assert_eq!(updated["payload"]["summary"]["os_type"], "Debian_64");
    assert_eq!(updated["payload"]["changed"][0], "os_type");
}

#[test]
fn events_watch_with_expired_cursor_asks_for_resync() {
    let env = Env::new();
    let (code, v) = env.json(&["events", "watch", "--since", "999"]);
    assert_eq!(
        (code, v["error"]["code"].as_str()),
        (5, Some("VTX-API-410"))
    );
}

#[test]
fn changes_queued_at_shutdown_apply_after_stop() {
    let env = Env::new();
    assert_eq!(
        env.json(&["vm", "create", "--name", "q", "--memory", "1024"])
            .0,
        0
    );
    assert_eq!(env.json(&["vm", "start", "q"]).0, 0);

    let (code, v) = env.json(&[
        "vm",
        "set",
        "q",
        "--memory",
        "4096",
        "--cpus",
        "4",
        "--tag",
        "db",
        "--at-shutdown",
    ]);
    assert_eq!(code, 0, "{v}");
    let queued: Vec<(String, bool)> = v["field_results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["path"].as_str().unwrap().to_string(),
                r["queued"].as_bool().unwrap(),
            )
        })
        .collect();
    assert!(
        queued.contains(&("memory_mib".into(), true)) && queued.contains(&("tags".into(), false)),
        "{queued:?}"
    );

    let (_, v) = env.json(&["vm", "pending", "list", "q"]);
    assert_eq!(v.as_array().unwrap().len(), 2);
    let (code, v) = env.json(&["vm", "pending", "cancel", "q", "cpu_count"]);
    assert_eq!(code, 0);
    assert_eq!(v.as_array().unwrap().len(), 1);
    let (code, _) = env.json(&["vm", "pending", "cancel", "q", "cpu_count"]);
    assert_eq!(code, 3, "already cancelled");
    let (_, v) = env.json(&["vm", "list"]);
    assert_eq!(v[0]["pending_change_count"], 1);

    assert_eq!(env.json(&["vm", "stop", "q", "--force"]).0, 0);
    let deadline = Instant::now() + Duration::from_secs(2);
    let v = loop {
        let (_, v) = env.json(&["vm", "show", "q"]);
        if v["pending_changes"].as_array().unwrap().is_empty() || Instant::now() > deadline {
            break v;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    assert!(
        v["pending_changes"].as_array().unwrap().is_empty(),
        "applied within 2 s: {v}"
    );
    assert_eq!(v["config"]["memory_mib"], 4096);
    assert_eq!(
        v["config"]["cpu_count"], 2,
        "the cancelled change was not applied"
    );
    assert_eq!(v["config"]["tags"][0], "db");
}

fn guest_file(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.cache/guest")
        .join(name);
    path.canonicalize()
        .unwrap_or_else(|_| panic!("{} missing; run `cargo xtask guest-kernel`", path.display()))
        .to_string_lossy()
        .into_owned()
}

fn wait_for_state(env: &Env, vm: &str, state: &str, within: Duration) -> Value {
    let deadline = Instant::now() + within;
    loop {
        let (_, v) = env.json(&["vm", "show", vm]);
        if v["summary"]["state"] == state {
            return v;
        }
        assert!(Instant::now() < deadline, "{vm} never reached {state}: {v}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_guest_boots_through_virtaxctl_and_powers_off() {
    let env = Env::without_mock();
    let (kernel, initrd) = (guest_file("vmlinuz"), guest_file("initramfs.cpio"));
    let (code, v) = env.json(&[
        "vm",
        "create",
        "--name",
        "g",
        "--engine",
        "virtax-vmm",
        "--cpus",
        "2",
        "--memory",
        "512",
        "--kernel",
        &kernel,
        "--initrd",
        &initrd,
    ]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["summary"]["engine"], "virtax-vmm");
    let serial = Path::new(v["folder"].as_str().unwrap()).join("logs/serial.log");

    // The test guest powers off right after printing its markers, which ends the VM by itself.
    let (code, v) = env.json(&["vm", "start", "g"]);
    assert_eq!((code, v["state"].as_str()), (0, Some("succeeded")), "{v}");
    wait_for_state(&env, "g", "powered_off", Duration::from_secs(60));
    let log = std::fs::read_to_string(&serial).unwrap();
    assert!(
        log.contains("VIRTAX-GUEST-OK"),
        "no userspace marker in {}:\n{log}",
        serial.display()
    );

    // Keep the guest up; `vm stop` presses the ACPI power button and the guest shuts down.
    assert_eq!(
        env.json(&["vm", "set", "g", "--cmdline", "virtax.hold"]).0,
        0
    );
    let wait_for_boot = |n: usize| {
        let deadline = Instant::now() + Duration::from_secs(60);
        // acpid must be running before the button is pressed.
        while std::fs::read_to_string(&serial)
            .unwrap()
            .matches("VIRTAX-GUEST-ACPID")
            .count()
            < n
        {
            assert!(
                Instant::now() < deadline,
                "boot {n} never reached userspace"
            );
            std::thread::sleep(Duration::from_millis(100));
        }
    };
    assert_eq!(env.json(&["vm", "start", "g"]).0, 0);
    wait_for_boot(2);
    wait_for_state(&env, "g", "running", Duration::from_secs(1));
    let (code, v) = env.json(&["vm", "stop", "g"]);
    assert_eq!((code, v["state"].as_str()), (0, Some("succeeded")), "{v}");
    wait_for_state(&env, "g", "powered_off", Duration::from_secs(5));
    assert!(
        std::fs::read_to_string(&serial)
            .unwrap()
            .contains("VIRTAX-GUEST-POWER-BUTTON")
    );

    // `--force` powers off without asking the guest.
    assert_eq!(env.json(&["vm", "start", "g"]).0, 0);
    wait_for_boot(3);
    let (code, v) = env.json(&["vm", "stop", "g", "--force"]);
    assert_eq!(code, 0, "{v}");
    wait_for_state(&env, "g", "powered_off", Duration::from_secs(5));
    assert_eq!(
        std::fs::read_to_string(&serial)
            .unwrap()
            .matches("VIRTAX-GUEST-POWER-BUTTON")
            .count(),
        1
    );
}

#[test]
fn vmm_machines_explain_what_they_need_before_starting() {
    // Runs everywhere: the configuration is rejected before any hypervisor is touched.
    let env = Env::new();
    let (code, v) = env.json(&["version"]);
    assert_eq!(code, 0);
    if !v["engines"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e == "virtax-vmm")
    {
        eprintln!("virtax-vmm engine not available on this host; skipping");
        return;
    }
    assert_eq!(
        env.json(&[
            "vm",
            "create",
            "--name",
            "nokernel",
            "--engine",
            "virtax-vmm"
        ])
        .0,
        0
    );
    let (code, v) = env.json(&["vm", "start", "nokernel"]);
    assert_eq!(
        (code, v["error"]["code"].as_str()),
        (10, Some("VTX-ENG-011")),
        "{v}"
    );
    let (_, v) = env.json(&["vm", "show", "nokernel"]);
    assert_eq!(v["summary"]["state"], "powered_off");
}
