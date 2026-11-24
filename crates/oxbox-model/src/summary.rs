//! `MachineSummary`: the read-model DTO cached as `machines.summary_json` and returned by `ListMachines`.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{EngineKind, MachineManifest, MachineState};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MachineSummary {
    pub id: Uuid,
    pub name: String,
    pub state: MachineState,
    pub os_type: String,
    pub groups: Vec<String>,
    pub tags: Vec<String>,
    pub memory_mib: u64,
    pub cpu_count: u32,
    pub engine: EngineKind,
    pub snapshot_count: u32,
    pub current_snapshot: Option<Uuid>,
    /// False when the manifest could not be read; `error_code` then holds the OXE code.
    pub accessible: bool,
    pub error_code: Option<String>,
}

impl MachineSummary {
    pub fn from_manifest(m: &MachineManifest, state: MachineState) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            state,
            os_type: m.os_type.clone(),
            groups: m.groups.clone(),
            tags: m.tags.clone(),
            memory_mib: m.memory.size_mib,
            cpu_count: m.cpu.count,
            engine: m.engine,
            snapshot_count: 0,
            current_snapshot: None,
            accessible: true,
            error_code: None,
        }
    }
}
