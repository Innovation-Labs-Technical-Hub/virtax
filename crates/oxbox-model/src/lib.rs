//! OxBox domain model: the portable `machine.toml` manifest (`oxbox.machine/1`), the collapsed VM
//! lifecycle, the `MachineSummary` read-model DTO and the bilingual OXE error catalogue.

pub mod errors;
pub mod manifest;
pub mod state;
pub mod summary;

pub use errors::{OxeCode, OxeError};
pub use manifest::{
    Arch, Attachment, Bus, Chipset, Clipboard, Cpu, DeviceKind, Display, EngineKind, Firmware,
    Integration, Issue, MachineManifest, Memory, NetAttachment, Nic, NicModel, Platform, SCHEMA_ID,
    SharedFolder, Storage, StorageController, Tpm, json_schema, parse_manifest,
};
pub use state::{Action, MachineState};
pub use summary::MachineSummary;
