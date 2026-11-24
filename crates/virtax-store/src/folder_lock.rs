//! Exclusive advisory lock on `<vm folder>/.virtax.lock`, recording who holds it.
//! The OS releases the lock when the holder exits, so a killed daemon never strands a VM.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use virtax_model::{VtxCode, VtxError};

use crate::StoreError;

pub const LOCK_FILE: &str = ".virtax.lock";

#[derive(Debug)]
pub struct FolderLock {
    _file: File,
    path: PathBuf,
}

impl FolderLock {
    /// Take the lock on `folder` (created if missing). Fails with `VTX-CFG-030` naming the holder.
    pub fn acquire(folder: &Path) -> Result<Self, StoreError> {
        std::fs::create_dir_all(folder)?;
        Self::acquire_file(&folder.join(LOCK_FILE))
    }

    /// Take an exclusive lock on an arbitrary lock file (also used for the daemon single-instance lock).
    pub fn acquire_file(path: &Path) -> Result<Self, StoreError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        match file.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                let mut holder = String::new();
                let _ = file.read_to_string(&mut holder);
                return Err(VtxError::with(
                    VtxCode::CFG_VM_LOCKED,
                    format!("{} held by {}", path.display(), holder.trim()),
                )
                .into());
            }
            Err(std::fs::TryLockError::Error(e)) => return Err(e.into()),
        }
        let info = serde_json::json!({
            "host": hostname(),
            "user": std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_default(),
            "pid": std::process::id(),
            "since": crate::now_rfc3339(),
        });
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(info.to_string().as_bytes())?;
        file.sync_data()?;
        Ok(Self {
            _file: file,
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .ok()
        .or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .ok()
                .map(|s| s.trim().to_string())
        })
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_holder_is_refused_until_release() {
        let dir = tempfile::tempdir().unwrap();
        let first = FolderLock::acquire(dir.path()).unwrap();
        match FolderLock::acquire(dir.path()) {
            Err(StoreError::Vtx(e)) => {
                assert_eq!(e.code, VtxCode::CFG_VM_LOCKED);
                assert!(e.detail.unwrap().contains(&std::process::id().to_string()));
            }
            other => panic!("expected lock conflict, got {other:?}"),
        }
        drop(first);
        FolderLock::acquire(dir.path()).unwrap();
    }
}
