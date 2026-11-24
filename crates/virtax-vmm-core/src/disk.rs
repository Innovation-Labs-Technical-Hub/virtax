//! Disk images: open a raw image, refuse every other format by its header, and lock it so that no
//! two VMs write the same file.
//!
//! The lock is an OS file lock (`flock`) on the open file: exclusive for read-write images, shared for
//! read-only ones. It lasts as long as the file stays open, and the OS releases it when the process
//! ends for any reason, `SIGKILL` included.

use std::fs::{File, OpenOptions, TryLockError};
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

pub const SECTOR_SIZE: u64 = 512;

/// A disk as given on the command line: an image file, attached read-write unless `read_only`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskConfig {
    pub path: PathBuf,
    pub read_only: bool,
}

/// An open, locked raw image.
#[derive(Debug)]
pub struct DiskImage {
    pub file: File,
    pub path: PathBuf,
    pub read_only: bool,
    /// Size in bytes, rounded down to whole sectors.
    pub size: u64,
}

/// The image formats virtax-vmm recognises but cannot attach yet.
pub fn detect_format(file: &File) -> std::io::Result<Option<&'static str>> {
    let mut head = [0u8; 512];
    let n = file.read_at(&mut head, 0)?;
    let head = &head[..n];
    if head.starts_with(b"QFI\xfb") {
        return Ok(Some("qcow2"));
    }
    if head.len() >= 68 && head[64..68] == 0xbeda_107f_u32.to_le_bytes() {
        return Ok(Some("VDI"));
    }
    if head.starts_with(b"KDMV") || head.starts_with(b"# Disk DescriptorFile") {
        return Ok(Some("VMDK"));
    }
    if head.starts_with(b"vhdxfile") {
        return Ok(Some("VHDX"));
    }
    // A VHD keeps its footer in the last 512 bytes (dynamic VHDs also copy it to the start).
    if head.starts_with(b"conectix") {
        return Ok(Some("VHD"));
    }
    let len = file.metadata()?.len();
    if len >= 512 {
        let mut footer = [0u8; 8];
        file.read_exact_at(&mut footer, len - 512)?;
        if &footer == b"conectix" {
            return Ok(Some("VHD"));
        }
    }
    Ok(None)
}

impl DiskImage {
    /// Open and lock the image `config` names; the error says what to do about it.
    pub fn open(config: &DiskConfig) -> Result<Self, String> {
        let path = &config.path;
        let file = OpenOptions::new()
            .read(true)
            .write(!config.read_only)
            .open(path)
            .map_err(|e| format!("cannot open disk image {}: {e}", path.display()))?;
        if !file.metadata().map(|m| m.is_file()).unwrap_or(false) {
            return Err(format!(
                "disk image {} is not a regular file",
                path.display()
            ));
        }
        match detect_format(&file) {
            Ok(None) => {}
            Ok(Some(format)) => {
                return Err(format!(
                    "disk image {} is in {format} format; virtax-vmm attaches raw images only \
                     (convert it with `qemu-img convert -O raw`)",
                    path.display()
                ));
            }
            Err(e) => return Err(format!("cannot read disk image {}: {e}", path.display())),
        }
        lock(&file, config.read_only, path)?;
        let size = file
            .metadata()
            .map_err(|e| format!("disk image {}: {e}", path.display()))?
            .len()
            / SECTOR_SIZE
            * SECTOR_SIZE;
        Ok(Self {
            file,
            path: path.clone(),
            read_only: config.read_only,
            size,
        })
    }

    /// Read `len` bytes at `offset` (used by tests and diagnostics; the block device reads straight
    /// into guest memory).
    pub fn read_vec(&mut self, offset: u64, len: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(&mut buf)?;
        Ok(buf)
    }
}

fn lock(file: &File, shared: bool, path: &Path) -> Result<(), String> {
    let result = if shared {
        file.try_lock_shared()
    } else {
        file.try_lock()
    };
    match result {
        Ok(()) => Ok(()),
        Err(TryLockError::WouldBlock) => Err(format!(
            "disk image {} is in use by another VM",
            path.display()
        )),
        Err(TryLockError::Error(e)) => {
            Err(format!("cannot lock disk image {}: {e}", path.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    fn rw(path: &Path) -> DiskConfig {
        DiskConfig {
            path: path.to_path_buf(),
            read_only: false,
        }
    }

    #[test]
    fn refuses_other_formats_by_their_headers() {
        let dir = tempfile::tempdir().unwrap();
        let mut vdi = vec![0u8; 1024];
        vdi[64..68].copy_from_slice(&0xbeda_107f_u32.to_le_bytes());
        let mut vhd = vec![0u8; 4096];
        vhd[4096 - 512..4096 - 504].copy_from_slice(b"conectix");
        let cases: [(&str, Vec<u8>); 6] = [
            ("qcow2", [&b"QFI\xfb"[..], &[0; 508]].concat()),
            ("VDI", vdi),
            ("VMDK", [&b"KDMV"[..], &[0; 508]].concat()),
            ("VMDK", b"# Disk DescriptorFile\nversion=1\n".to_vec()),
            ("VHDX", [&b"vhdxfile"[..], &[0; 504]].concat()),
            ("VHD", vhd),
        ];
        for (i, (format, bytes)) in cases.iter().enumerate() {
            let path = image(dir.path(), &format!("{i}.img"), bytes);
            let err = DiskImage::open(&rw(&path)).unwrap_err();
            assert!(
                err.contains(&format!("in {format} format")) && err.contains(&format!("{i}.img")),
                "{err}"
            );
        }
        let raw = image(dir.path(), "raw.img", &vec![0x5a; 4096 + 100]);
        let disk = DiskImage::open(&rw(&raw)).unwrap();
        assert_eq!(disk.size, 4096, "rounded down to whole sectors");
    }

    #[test]
    fn missing_image_is_named() {
        let err = DiskImage::open(&rw(Path::new("/nonexistent/disk.img"))).unwrap_err();
        assert!(err.contains("/nonexistent/disk.img"), "{err}");
    }

    #[test]
    fn a_read_write_image_is_locked_until_released() {
        let dir = tempfile::tempdir().unwrap();
        let path = image(dir.path(), "disk.img", &[0; 4096]);
        let first = DiskImage::open(&rw(&path)).unwrap();
        let err = DiskImage::open(&rw(&path)).unwrap_err();
        assert!(err.contains("in use by another VM"), "{err}");
        let ro = DiskConfig {
            path: path.clone(),
            read_only: true,
        };
        assert!(
            DiskImage::open(&ro).is_err(),
            "a reader cannot share a written image"
        );
        drop(first);
        let readers = (DiskImage::open(&ro).unwrap(), DiskImage::open(&ro).unwrap());
        assert!(
            DiskImage::open(&rw(&path)).is_err(),
            "no writer while it is shared"
        );
        drop(readers);
        DiskImage::open(&rw(&path)).unwrap();
    }
}
