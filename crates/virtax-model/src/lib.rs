//! Virtax domain model: the portable `machine.toml` manifest (`virtax.machine/1`), the collapsed VM
//! lifecycle, the `MachineSummary` read-model DTO and the bilingual VTX error catalogue.

pub mod errors;
pub mod manifest;
pub mod state;
pub mod summary;

pub use errors::{VtxCode, VtxError};
pub use manifest::{
    Arch, Attachment, Boot, Bus, Chipset, Clipboard, Cpu, DeviceKind, Display, EngineKind,
    Firmware, Integration, Issue, MachineManifest, Memory, NetAttachment, Nic, NicModel, Platform,
    SCHEMA_ID, SharedFolder, Storage, StorageController, Tpm, json_schema, parse_manifest,
};
pub use state::{Action, MachineState};
pub use summary::MachineSummary;
