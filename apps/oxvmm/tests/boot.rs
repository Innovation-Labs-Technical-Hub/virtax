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
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_boots_four_vcpus_and_powers_off_through_acpi() {
    let dir = tempfile::tempdir().unwrap();
    let serial = dir.path().join("serial.log");
    let out = Command::new(env!("CARGO_BIN_EXE_oxvmm"))
        .arg("--kernel")
        .arg(guest_dir().join("vmlinuz"))
        .arg("--initrd")
        .arg(guest_dir().join("initramfs.cpio"))
        .args([
            "--memory-mib",
            "512",
            "--cpus",
            "4",
            "--cmdline",
            "oxvmm.poweroff",
            "--serial",
        ])
        .arg(&serial)
        .output()
        .expect("run oxvmm");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let log = std::fs::read_to_string(&serial).unwrap();
    assert!(
        out.status.success(),
        "oxvmm failed: {}\n{stderr}\n{log}",
        out.status
    );
    assert!(stderr.contains("guest powered off"), "{stderr}");
    assert!(
        log.contains("OXVMM-GUEST-CPUS 4"),
        "guest did not see 4 CPUs:\n{log}"
    );
    assert!(
        log.contains("reboot: Power down"),
        "guest did not power down through ACPI:\n{log}"
    );
    // ACPI says there is no PS/2 controller, so Linux skips it instead of failing a probe.
    assert!(
        log.contains("No PS/2 controller found") && !log.contains("Can't read CTR"),
        "the kernel probed for a PS/2 controller:\n{log}"
    );
}

#[test]
fn rejects_an_invalid_vcpu_count_before_touching_kvm() {
    let out = Command::new(env!("CARGO_BIN_EXE_oxvmm"))
        .args(["--kernel", "/nonexistent", "--cpus", "0"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("vCPUs are supported"));
}
