//! Persistence for `virtaxd`: the per-user SQLite registry and the per-VM `machine.toml` folder.

mod error;
pub mod folder_lock;
pub mod manifest_io;
pub mod registry;

pub use error::StoreError;
pub use folder_lock::FolderLock;
pub use manifest_io::{ReadOutcome, read_manifest, write_manifest};
pub use registry::{
    EventRecord, MachineRow, OperationRecord, PendingChange, PendingEdit, Registry,
};

use std::path::PathBuf;

/// Per-user data directory: `$VIRTAX_DATA_DIR`, else the platform local-data dir
/// (`~/.local/share/virtax`, `~/Library/Application Support/Virtax`, `%LOCALAPPDATA%\Virtax`).
pub fn default_data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("VIRTAX_DATA_DIR") {
        return PathBuf::from(dir);
    }
    let base = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    base.join(if cfg!(target_os = "linux") {
        "virtax"
    } else {
        "Virtax"
    })
}

/// The current UTC time as RFC 3339 with milliseconds, e.g. `2026-11-24T03:11:07.673Z`. The registry stores
/// timestamps in this fixed-width format, so they also sort and compare correctly as strings.
pub fn now_rfc3339() -> String {
    rfc3339(jiff::Timestamp::now())
}

pub(crate) fn rfc3339(t: jiff::Timestamp) -> String {
    t.strftime("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamps_are_fixed_width_utc_with_milliseconds() {
        assert_eq!(
            rfc3339(jiff::Timestamp::UNIX_EPOCH),
            "1970-01-01T00:00:00.000Z"
        );
        let t = jiff::Timestamp::from_millisecond(1_795_478_400_007).unwrap();
        assert_eq!(rfc3339(t), "2026-11-24T00:00:00.007Z");
        assert_eq!(now_rfc3339().len(), 24);
    }
}
