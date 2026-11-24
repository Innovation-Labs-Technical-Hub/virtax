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
        Self::start_with(cpus, cmdline, &[])
    }

    /// Start with extra `virtax-vmm` arguments, such as `--disk`.
    fn start_with(cpus: u32, cmdline: &str, extra: &[String]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let serial = dir.path().join("serial.log");
        let child = Command::new(env!("CARGO_BIN_EXE_virtax-vmm"))
            .args(extra)
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

#[test]
fn an_invalid_disk_argument_exits_1_with_a_clear_message() {
    let out = Command::new(env!("CARGO_BIN_EXE_virtax-vmm"))
        .args(["--kernel", "/nonexistent", "--disk", ",ro"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("missing image path"), "{stderr}");
    let help = Command::new(env!("CARGO_BIN_EXE_virtax-vmm"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--disk <PATH[,ro|,rw]>"));
}

/// A raw image of `mib` MiB filled with zeros, in `dir`.
fn raw_image(dir: &std::path::Path, name: &str, mib: u64) -> PathBuf {
    let path = dir.join(name);
    std::fs::File::create(&path)
        .unwrap()
        .set_len(mib << 20)
        .unwrap();
    path
}

fn disk_arg(path: &std::path::Path, read_only: bool) -> Vec<String> {
    let mut value = path.display().to_string();
    if read_only {
        value.push_str(",ro");
    }
    vec!["--disk".into(), value]
}

fn starts_with_pattern(path: &std::path::Path, disk: &str) -> bool {
    let pattern = format!("VIRTAX-DISK-PATTERN-{disk}");
    std::fs::read(path).unwrap().starts_with(pattern.as_bytes())
}

#[test]
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_guest_enumerates_pci_and_finds_only_the_host_bridge() {
    let mut guest = Guest::start(1, "");
    let (status, stderr) = guest.wait_exit();
    let log = guest.log();
    assert!(status.success(), "{status}\n{stderr}\n{log}");
    assert!(
        log.contains("VIRTAX-GUEST-PCI 0000:00:00.0 \n")
            || log.contains("VIRTAX-GUEST-PCI 0000:00:00.0 \r\n"),
        "the bus should hold only the host bridge:\n{log}"
    );
    assert!(log.contains("[1b36:0008] type 00 class 0x060000"), "{log}");
    assert!(log.contains("reboot: Power down"), "{log}");
}

#[test]
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_flushed_disk_writes_reach_the_image_even_if_the_vmm_is_killed() {
    let dir = tempfile::tempdir().unwrap();
    let image = raw_image(dir.path(), "disk.img", 64);
    let mut guest = Guest::start_with(2, "virtax.disktest virtax.hold", &disk_arg(&image, false));
    guest.wait_for("VIRTAX-DISK-DONE", 1);
    let log = guest.log();
    assert!(
        log.contains("VIRTAX-DISK vda sectors=131072 ro=0"),
        "64 MiB read-write disk:\n{log}"
    );
    assert!(log.contains("VIRTAX-DISK-WROTE vda"), "{log}");
    assert!(
        log.contains("[1af4:1042]"),
        "modern virtio-blk on PCI:\n{log}"
    );
    guest.signal("-KILL");
    guest.wait_exit();
    assert!(
        starts_with_pattern(&image, "vda"),
        "pattern missing from the image"
    );
}

#[test]
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_read_only_disk_refuses_guest_writes() {
    let dir = tempfile::tempdir().unwrap();
    let image = raw_image(dir.path(), "ro.img", 8);
    let mut guest = Guest::start_with(1, "virtax.disktest", &disk_arg(&image, true));
    let (status, stderr) = guest.wait_exit();
    let log = guest.log();
    assert!(status.success(), "{status}\n{stderr}\n{log}");
    assert!(log.contains("VIRTAX-DISK vda sectors=16384 ro=1"), "{log}");
    assert!(log.contains("VIRTAX-DISK-WRITE-FAILED vda"), "{log}");
    assert!(
        std::fs::read(&image).unwrap().iter().all(|b| *b == 0),
        "the read-only image changed"
    );
}

#[test]
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_two_disks_appear_in_the_order_given() {
    let dir = tempfile::tempdir().unwrap();
    let first = raw_image(dir.path(), "first.img", 16);
    let second = raw_image(dir.path(), "second.img", 8);
    let args = [disk_arg(&first, false), disk_arg(&second, false)].concat();
    let mut guest = Guest::start_with(2, "virtax.disktest", &args);
    let (status, stderr) = guest.wait_exit();
    let log = guest.log();
    assert!(status.success(), "{status}\n{stderr}\n{log}");
    assert!(log.contains("VIRTAX-DISK vda sectors=32768"), "{log}");
    assert!(log.contains("VIRTAX-DISK vdb sectors=16384"), "{log}");
    assert!(starts_with_pattern(&first, "vda"), "first image is not vda");
    assert!(
        starts_with_pattern(&second, "vdb"),
        "second image is not vdb"
    );
}

#[test]
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_qcow2_image_is_refused_before_the_guest_runs() {
    let dir = tempfile::tempdir().unwrap();
    let image = dir.path().join("disk.qcow2");
    std::fs::write(&image, [&b"QFI\xfb"[..], &[0u8; 1020]].concat()).unwrap();
    let mut guest = Guest::start_with(1, "", &disk_arg(&image, false));
    let (status, stderr) = guest.wait_exit();
    assert_eq!(status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("disk.qcow2") && stderr.contains("qcow2 format"),
        "{stderr}"
    );
    assert!(!stderr.contains("guest running"), "{stderr}");
    assert!(guest.log().is_empty(), "the guest ran");
}

#[test]
#[ignore = "needs /dev/kvm and `cargo xtask guest-kernel`; run with `cargo xtask kvm-test`"]
fn kvm_an_image_in_use_is_refused_until_its_vm_stops() {
    let dir = tempfile::tempdir().unwrap();
    let image = raw_image(dir.path(), "shared.img", 8);
    let mut first = Guest::start_with(1, "virtax.hold", &disk_arg(&image, false));
    first.wait_for("VIRTAX-GUEST-OK", 1);

    let mut second = Guest::start_with(1, "", &disk_arg(&image, false));
    let (status, stderr) = second.wait_exit();
    assert_eq!(status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("in use by another VM"), "{stderr}");
    assert!(second.log().is_empty(), "the second guest ran");

    // Killing the first VM releases the lock.
    first.signal("-KILL");
    first.wait_exit();
    let mut third = Guest::start_with(1, "", &disk_arg(&image, false));
    let (status, stderr) = third.wait_exit();
    assert!(status.success(), "{status}\n{stderr}");
}
