//! Boots the test guest from `cargo xtask guest-kernel` on KVM. Run with `cargo xtask kvm-test`.

use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

fn guest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.cache/guest")
}

/// A running `virtax-vmm` with the test guest. Killed on drop, so a failing test leaves no VM behind.
struct Guest {
    child: Child,
    _dir: tempfile::TempDir,
    serial: PathBuf,
}

impl Guest {
    fn start(cpus: u32, cmdline: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let serial = dir.path().join("serial.log");
        let child = Command::new(env!("CARGO_BIN_EXE_virtax-vmm"))
            .arg("--kernel")
            .arg(guest_dir().join("vmlinuz"))
            .arg("--initrd")
            .arg(guest_dir().join("initramfs.cpio"))
            .args([
                "--memory-mib",
                "512",
                "--cpus",
                &cpus.to_string(),
                "--cmdline",
                cmdline,
                "--serial",
            ])
            .arg(&serial)
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn virtax-vmm");
        Self {
            child,
            _dir: dir,
            serial,
        }
    }

    fn log(&self) -> String {
        std::fs::read_to_string(&self.serial).unwrap_or_default()
    }

    /// Wait until the serial log contains `marker` at least `count` times.
    fn wait_for(&mut self, marker: &str, count: usize) {
        let deadline = Instant::now() + Duration::from_secs(60);
        while self.log().matches(marker).count() < count {
            if let Some(status) = self.child.try_wait().unwrap() {
                panic!(
                    "virtax-vmm exited ({status}) before {marker} x{count}:\n{}",
                    self.log()
                );
            }
            assert!(
                Instant::now() < deadline,
                "no {marker} x{count} within 60 s:\n{}",
                self.log()
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Wait for the process to exit; returns its status and stderr.
    fn wait_exit(&mut self) -> (ExitStatus, String) {
        let deadline = Instant::now() + Duration::from_secs(60);
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "virtax-vmm still running after 60 s:\n{}",
                self.log()
            );
            std::thread::sleep(Duration::from_millis(50));
        };
        let stderr = std::io::read_to_string(self.child.stderr.take().unwrap()).unwrap();
        (status, stderr)
    }

    fn signal(&self, signal: &str) {
        let ok = Command::new("kill")
            .args([signal, &self.child.id().to_string()])
            .status()
            .unwrap();
        assert!(ok.success(), "kill {signal} failed");
    }
}

impl Drop for Guest {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn host_utc_date() -> String {
    let out = Command::new("date")
        .args(["-u", "+%Y-%m-%d"])
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

#[test]
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_boots_to_userspace_with_the_host_date_and_powers_off() {
    let mut guest = Guest::start(1, "virtax.test=1");
    let (status, stderr) = guest.wait_exit();
    let log = guest.log();
    assert!(
        status.success(),
        "virtax-vmm failed: {status}\n{stderr}\n{log}"
    );
    assert!(stderr.contains("virtax-vmm: guest running"), "{stderr}");
    assert!(stderr.contains("guest powered off"), "{stderr}");
    assert!(
        log.contains("VIRTAX-GUEST-OK"),
        "userspace marker missing:\n{log}"
    );
    assert!(
        log.contains("virtax.test=1"),
        "extra cmdline not passed:\n{log}"
    );
    let date = format!("VIRTAX-GUEST-DATE {}", host_utc_date());
    assert!(
        log.contains(&date),
        "guest clock is not the host's date ({date}):\n{log}"
    );
}

#[test]
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_boots_four_vcpus_and_powers_off_through_acpi() {
    let mut guest = Guest::start(4, "");
    let (status, stderr) = guest.wait_exit();
    let log = guest.log();
    assert!(
        status.success(),
        "virtax-vmm failed: {status}\n{stderr}\n{log}"
    );
    assert!(
        log.contains("VIRTAX-GUEST-CPUS 4"),
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
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_sigterm_presses_the_power_button_and_the_guest_shuts_down() {
    let mut guest = Guest::start(2, "virtax.hold");
    guest.wait_for("VIRTAX-GUEST-ACPID", 1);
    guest.signal("-TERM");
    let (status, stderr) = guest.wait_exit();
    let log = guest.log();
    assert!(
        status.success(),
        "virtax-vmm failed: {status}\n{stderr}\n{log}"
    );
    assert!(stderr.contains("power button pressed"), "{stderr}");
    assert!(
        log.contains("VIRTAX-GUEST-POWER-BUTTON"),
        "guest never saw the power button:\n{log}"
    );
    assert!(
        log.contains("reboot: Power down"),
        "guest did not power down:\n{log}"
    );
}

#[test]
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_guest_reboot_restarts_the_vm_in_the_same_process() {
    let mut guest = Guest::start(1, "virtax.reboot");
    guest.wait_for("VIRTAX-GUEST-OK", 2);
    assert!(
        guest.child.try_wait().unwrap().is_none(),
        "the process must survive the reboot"
    );
    guest.signal("-KILL");
    let (_, stderr) = guest.wait_exit();
    assert!(stderr.contains("restarting the VM"), "{stderr}");
    assert!(
        stderr.matches("virtax-vmm: guest running").count() >= 2,
        "{stderr}"
    );
}

#[test]
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_no_reboot_exits_when_the_guest_reboots() {
    let dir = tempfile::tempdir().unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_virtax-vmm"))
        .arg("--kernel")
        .arg(guest_dir().join("vmlinuz"))
        .arg("--initrd")
        .arg(guest_dir().join("initramfs.cpio"))
        .args([
            "--memory-mib",
            "512",
            "--cmdline",
            "virtax.reboot",
            "--no-reboot",
            "--serial",
        ])
        .arg(dir.path().join("serial.log"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(stderr.contains("(--no-reboot)"), "{stderr}");
}

#[test]
// Elsewhere the binary only reports that the host is unsupported.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn rejects_an_invalid_vcpu_count_before_touching_kvm() {
    let out = Command::new(env!("CARGO_BIN_EXE_virtax-vmm"))
        .args(["--kernel", "/nonexistent", "--cpus", "0"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("vCPUs are supported"));
}
