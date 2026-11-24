//! Crash-safe `machine.toml` I/O.
//!
//! Write protocol: serialize + validate → write `machine.toml.tmp` and fsync → copy the current file to
//! `machine.toml.prev` (via a fsynced temp + rename) → rename tmp over `machine.toml` → fsync the folder.
//! At every step a valid `machine.toml` (old or new) exists, and `.prev` holds the previous version.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use virtax_model::{MachineManifest, VtxCode, VtxError, parse_manifest};

use crate::StoreError;

pub const MANIFEST: &str = "machine.toml";
const TMP: &str = "machine.toml.tmp";
const PREV: &str = "machine.toml.prev";
const PREV_TMP: &str = "machine.toml.prev.tmp";

#[derive(Debug)]
pub enum ReadOutcome {
    Ok(MachineManifest),
    /// `machine.toml` was unreadable and was restored from `machine.toml.prev`; the damaged file
    /// was kept at `corrupt_copy` (absent if `machine.toml` was missing altogether).
    Recovered {
        manifest: MachineManifest,
        corrupt_copy: Option<PathBuf>,
    },
}

impl ReadOutcome {
    pub fn manifest(&self) -> &MachineManifest {
        match self {
            Self::Ok(m) | Self::Recovered { manifest: m, .. } => m,
        }
    }

    pub fn into_manifest(self) -> MachineManifest {
        match self {
            Self::Ok(m) | Self::Recovered { manifest: m, .. } => m,
        }
    }
}

fn sync_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        File::open(dir)?.sync_all()
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
        Ok(())
    }
}

fn write_synced(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut f = File::create(path)?;
    f.write_all(bytes)?;
    f.sync_all()
}

/// Atomically replace `<folder>/machine.toml` with `manifest`. The caller must hold the folder lock.
pub fn write_manifest(folder: &Path, manifest: &MachineManifest) -> Result<(), StoreError> {
    let issues = manifest.validate();
    if !issues.is_empty() {
        let detail = issues
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        return Err(VtxError::with(VtxCode::CFG_INVALID_MANIFEST, detail).into());
    }
    fs::create_dir_all(folder)?;
    let text = manifest.to_toml();
    let tmp = folder.join(TMP);
    write_synced(&tmp, text.as_bytes())?;

    let current = folder.join(MANIFEST);
    if current.exists() {
        let prev_tmp = folder.join(PREV_TMP);
        write_synced(&prev_tmp, &fs::read(&current)?)?;
        fs::rename(&prev_tmp, folder.join(PREV))?;
    }
    fs::rename(&tmp, &current)?;
    sync_dir(folder)?;
    Ok(())
}

fn try_read(path: &Path) -> Result<MachineManifest, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse_manifest(&text).map_err(|issues| {
        issues
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ")
    })
}

/// Read `<folder>/machine.toml`, restoring it from `.prev` when it is missing or damaged.
pub fn read_manifest(folder: &Path) -> Result<ReadOutcome, StoreError> {
    let current = folder.join(MANIFEST);
    let primary_err = match try_read(&current) {
        Ok(m) => return Ok(ReadOutcome::Ok(m)),
        Err(e) => e,
    };
    let prev = folder.join(PREV);
    let manifest = try_read(&prev).map_err(|prev_err| {
        VtxError::with(
            VtxCode::CFG_MANIFEST_UNREADABLE,
            format!("{}: {primary_err}; {PREV}: {prev_err}", current.display()),
        )
    })?;

    let corrupt_copy = if current.exists() {
        let ts = chrono::Utc::now().format("%Y%m%dT%H%M%S%.3fZ");
        let dest = folder.join(format!("{MANIFEST}.corrupt-{ts}"));
        fs::rename(&current, &dest)?;
        Some(dest)
    } else {
        None
    };
    let tmp = folder.join(TMP);
    write_synced(&tmp, &fs::read(&prev)?)?;
    fs::rename(&tmp, &current)?;
    sync_dir(folder)?;
    Ok(ReadOutcome::Recovered {
        manifest,
        corrupt_copy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use virtax_model::EngineKind;

    #[test]
    fn write_keeps_previous_version() {
        let dir = tempfile::tempdir().unwrap();
        let mut m = MachineManifest::new("a", "Linux_64", EngineKind::Mock);
        write_manifest(dir.path(), &m).unwrap();
        m.cpu.count = 8;
        write_manifest(dir.path(), &m).unwrap();
        assert_eq!(read_manifest(dir.path()).unwrap().manifest().cpu.count, 8);
        let prev = try_read(&dir.path().join(PREV)).unwrap();
        assert_eq!(prev.cpu.count, 2);
        assert!(!dir.path().join(TMP).exists());
    }

    #[test]
    fn invalid_manifest_is_not_written() {
        let dir = tempfile::tempdir().unwrap();
        let mut m = MachineManifest::new("a", "Linux_64", EngineKind::Mock);
        m.cpu.count = 0;
        assert!(write_manifest(dir.path(), &m).is_err());
        assert!(!dir.path().join(MANIFEST).exists());
    }

    #[test]
    fn truncated_manifest_is_restored_from_prev() {
        let dir = tempfile::tempdir().unwrap();
        let mut m = MachineManifest::new("a", "Linux_64", EngineKind::Mock);
        write_manifest(dir.path(), &m).unwrap();
        m.memory.size_mib = 4096;
        write_manifest(dir.path(), &m).unwrap();
        let text = fs::read_to_string(dir.path().join(MANIFEST)).unwrap();
        fs::write(dir.path().join(MANIFEST), &text[..text.len() / 2]).unwrap();

        match read_manifest(dir.path()).unwrap() {
            ReadOutcome::Recovered {
                manifest,
                corrupt_copy,
            } => {
                assert_eq!(manifest.memory.size_mib, 2048);
                assert!(corrupt_copy.unwrap().exists());
            }
            other => panic!("expected recovery, got {other:?}"),
        }
        assert!(matches!(
            read_manifest(dir.path()).unwrap(),
            ReadOutcome::Ok(_)
        ));
    }

    #[test]
    fn crash_between_steps_leaves_a_valid_manifest() {
        // Simulate a crash after the tmp file was written but before the final rename.
        let dir = tempfile::tempdir().unwrap();
        let m = MachineManifest::new("a", "Linux_64", EngineKind::Mock);
        write_manifest(dir.path(), &m).unwrap();
        fs::write(dir.path().join(TMP), "garbage").unwrap();
        fs::write(dir.path().join(PREV_TMP), "garbage").unwrap();
        assert!(matches!(
            read_manifest(dir.path()).unwrap(),
            ReadOutcome::Ok(_)
        ));
    }

    #[test]
    fn unreadable_without_prev_reports_vtx_code() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(MANIFEST), "not toml [").unwrap();
        match read_manifest(dir.path()) {
            Err(StoreError::Vtx(e)) => assert_eq!(e.code, VtxCode::CFG_MANIFEST_UNREADABLE),
            other => panic!("unexpected {other:?}"),
        }
    }
}
