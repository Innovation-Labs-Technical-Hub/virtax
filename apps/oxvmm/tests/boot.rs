//! Boots the test guest from `cargo xtask guest-kernel` on KVM. Run with `cargo xtask kvm-test`.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn guest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.cache/guest")
}

#[test]
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_boots_to_userspace_and_exits_on_guest_reboot() {
    let dir = tempfile::tempdir().unwrap();
    let serial = dir.path().join("serial.log");
    let mut child = Command::new(env!("CARGO_BIN_EXE_oxvmm"))
        .arg("--kernel")
        .arg(guest_dir().join("vmlinuz"))
        .arg("--initrd")
        .arg(guest_dir().join("initramfs.cpio"))
        .args([
            "--memory-mib",
            "512",
            "--cmdline",
            "oxvmm.test=1",
            "--serial",
        ])
        .arg(&serial)
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn oxvmm");

    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("guest did not finish within 60 s");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let stderr = std::io::read_to_string(child.stderr.take().unwrap()).unwrap();
    let log = std::fs::read_to_string(&serial).unwrap();
    assert!(status.success(), "oxvmm failed: {status}\n{stderr}\n{log}");
    assert!(stderr.contains("oxvmm: guest running"), "{stderr}");
    assert!(
        log.contains("OXVMM-GUEST-OK"),
        "userspace marker missing:\n{log}"
    );
    assert!(
        log.contains("oxvmm.test=1"),
        "extra cmdline not passed:\n{log}"
    );
}

#[test]
fn rejects_more_than_one_vcpu_before_touching_kvm() {
    let out = Command::new(env!("CARGO_BIN_EXE_oxvmm"))
        .args(["--kernel", "/nonexistent", "--cpus", "2"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("single vCPU"));
}
