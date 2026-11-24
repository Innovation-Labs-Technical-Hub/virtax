//! Test guest for the KVM tests: a pinned Ubuntu kernel and a busybox initramfs, in `.cache/guest/`.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Pinned kernel package from the Ubuntu archive. The SHA-256 comes from the signed archive index.
const KERNEL_DEB_URL: &str = "https://archive.ubuntu.com/ubuntu/pool/main/l/linux/linux-image-unsigned-7.0.0-42-generic_7.0.0-42.42_amd64.deb";
const KERNEL_DEB_SHA256: &str = "b5102b9aec5267dbc99cbf4a78fc81b039d4149995b25206706489b0934532b0";
const KERNEL_PATH_IN_DEB: &str = "./boot/vmlinuz-7.0.0-42-generic";
/// A statically linked busybox (Debian/Ubuntu package `busybox-static`).
const BUSYBOX: &str = "/usr/bin/busybox";

/// The test guest's `/init`: prints markers the KVM tests look for on the serial console, then
/// reboots (which ends the VM) unless the kernel command line contains `oxvmm.hold`.
const INIT_SCRIPT: &str = "#!/bin/busybox sh
/bin/busybox mount -t proc proc /proc
/bin/busybox echo \"OXVMM-GUEST-OK $(/bin/busybox uname -r)\"
/bin/busybox echo \"OXVMM-GUEST-CMDLINE $(/bin/busybox cat /proc/cmdline)\"
if /bin/busybox grep -q oxvmm.hold /proc/cmdline; then /bin/busybox sleep 3600; fi
/bin/busybox reboot -f
";

pub fn dir(root: &Path) -> PathBuf {
    root.join(".cache").join("guest")
}

fn run(cmd: &mut Command) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("{cmd:?}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmd:?} failed with {status}"))
    }
}

fn sha256(path: &Path) -> Result<String, String> {
    let out = Command::new("sha256sum")
        .arg(path)
        .output()
        .map_err(|e| format!("sha256sum: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.split_whitespace()
        .next()
        .map(str::to_string)
        .ok_or_else(|| "sha256sum printed nothing".into())
}

pub fn fetch(root: &Path) -> Result<(), String> {
    let dir = dir(root);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let kernel = dir.join("vmlinuz");
    if kernel.exists() {
        eprintln!("kernel: {} (cached)", kernel.display());
    } else {
        let deb = dir.join("kernel.deb");
        if !deb.exists() || sha256(&deb)? != KERNEL_DEB_SHA256 {
            run(Command::new("curl")
                .args([
                    "--fail",
                    "--location",
                    "--silent",
                    "--show-error",
                    "--output",
                ])
                .arg(&deb)
                .arg(KERNEL_DEB_URL))?;
        }
        let got = sha256(&deb)?;
        if got != KERNEL_DEB_SHA256 {
            let _ = std::fs::remove_file(&deb);
            return Err(format!(
                "kernel package checksum mismatch: expected {KERNEL_DEB_SHA256}, got {got}"
            ));
        }
        // dpkg-deb streams the data tarball; tar writes the single kernel file to stdout.
        let tarball = Command::new("dpkg-deb")
            .arg("--fsys-tarfile")
            .arg(&deb)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("dpkg-deb: {e} (Debian/Ubuntu hosts only)"))?;
        let out = std::fs::File::create(&kernel).map_err(|e| e.to_string())?;
        let status = Command::new("tar")
            .args(["-xO", KERNEL_PATH_IN_DEB])
            .stdin(tarball.stdout.expect("piped"))
            .stdout(out)
            .status()
            .map_err(|e| format!("tar: {e}"))?;
        if !status.success() || std::fs::metadata(&kernel).map(|m| m.len()).unwrap_or(0) == 0 {
            let _ = std::fs::remove_file(&kernel);
            return Err("could not extract the kernel from the package".into());
        }
        let _ = std::fs::remove_file(&deb);
        eprintln!("kernel: {}", kernel.display());
    }

    let busybox =
        std::fs::read(BUSYBOX).map_err(|e| format!("{BUSYBOX}: {e} (install busybox-static)"))?;
    let mut cpio = Newc::default();
    for d in ["bin", "dev", "proc", "sys"] {
        cpio.entry(d, 0o040_755, 0, &[]);
    }
    cpio.entry("dev/console", 0o020_600, (5 << 8) | 1, &[]);
    cpio.entry("bin/busybox", 0o100_755, 0, &busybox);
    cpio.entry("init", 0o100_755, 0, INIT_SCRIPT.as_bytes());
    let initrd = dir.join("initramfs.cpio");
    std::fs::write(&initrd, cpio.finish()).map_err(|e| e.to_string())?;
    eprintln!("initramfs: {}", initrd.display());
    Ok(())
}

/// Minimal writer for the cpio "newc" format the kernel unpacks into its initramfs.
#[derive(Default)]
struct Newc {
    out: Vec<u8>,
    ino: u32,
}

impl Newc {
    /// `rdev` is `(major << 8) | minor` for device nodes.
    fn entry(&mut self, name: &str, mode: u32, rdev: u32, data: &[u8]) {
        self.ino += 1;
        let fields = [
            self.ino,
            mode,
            0,
            0,
            1,
            0,
            data.len() as u32,
            0,
            0,
            rdev >> 8,
            rdev & 0xff,
            name.len() as u32 + 1,
            0,
        ];
        self.out.extend_from_slice(b"070701");
        for f in fields {
            self.out.extend_from_slice(format!("{f:08x}").as_bytes());
        }
        self.out.extend_from_slice(name.as_bytes());
        self.out.push(0);
        self.pad();
        self.out.extend_from_slice(data);
        self.pad();
    }

    fn pad(&mut self) {
        while !self.out.len().is_multiple_of(4) {
            self.out.push(0);
        }
    }

    fn finish(mut self) -> Vec<u8> {
        self.entry("TRAILER!!!", 0, 0, &[]);
        while !self.out.len().is_multiple_of(512) {
            self.out.push(0);
        }
        self.out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newc_layout() {
        let mut c = Newc::default();
        c.entry("init", 0o100_755, 0, b"abc");
        let out = c.finish();
        assert_eq!(&out[..6], b"070701");
        // header (110) + "init\0" (5) = 115 -> padded to 116, then data "abc" padded to 4.
        assert_eq!(&out[110..115], b"init\0");
        assert_eq!(&out[116..119], b"abc");
        assert_eq!(out.len() % 512, 0);
        assert!(out.windows(10).any(|w| w == b"TRAILER!!!"));
    }
}
