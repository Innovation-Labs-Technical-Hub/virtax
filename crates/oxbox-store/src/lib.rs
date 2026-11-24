//! Persistence for `oxboxd`: the per-user SQLite registry and the per-VM `machine.toml` folder.

mod error;
pub mod folder_lock;
pub mod manifest_io;
pub mod registry;

pub use error::StoreError;
pub use folder_lock::FolderLock;
pub use manifest_io::{ReadOutcome, read_manifest, write_manifest};
pub use registry::{EventRecord, MachineRow, OperationRecord, Registry};

use std::path::PathBuf;

/// Per-user data directory: `$OXBOX_DATA_DIR`, else the platform local-data dir
/// (`~/.local/share/oxbox`, `~/Library/Application Support/OxBox`, `%LOCALAPPDATA%\OxBox`).
pub fn default_data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("OXBOX_DATA_DIR") {
        return PathBuf::from(dir);
    }
    let base = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    base.join(if cfg!(target_os = "linux") {
        "oxbox"
    } else {
        "OxBox"
    })
}

pub(crate) fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
