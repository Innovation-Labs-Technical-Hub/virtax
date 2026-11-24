//! Daemon core: machines (registry + manifests), per-VM serialization, engines and operations.
//! Transport-agnostic so it can be tested without gRPC.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::StreamExt;
use oxbox_api::v1 as pb;
use oxbox_engine::{ApplyMode, Engine, EngineEvent, OpHandle, StartOpts, StopMode};
use oxbox_model::{
    Action, EngineKind, Firmware, MachineManifest, MachineState, MachineSummary, OxeCode, OxeError,
};
use oxbox_store::{
    FolderLock, OperationRecord, ReadOutcome, Registry, StoreError, read_manifest, write_manifest,
};
use tokio::sync::{broadcast, watch};
use tokio_util::sync::CancellationToken;
use tonic::Code;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::events::{BROADCAST_BUFFER, operation_payload, summary_payload};

/// API field-mask path → manifest field path.
pub const MASK_PATHS: &[(&str, &str)] = &[
    ("display_name", "name"),
    ("os_type", "os_type"),
    ("groups", "groups"),
    ("tags", "tags"),
    ("cpu_count", "cpu.count"),
    ("memory_mib", "memory.size_mib"),
    ("vram_mib", "display.vram_mib"),
    ("monitors", "display.monitors"),
    ("firmware", "platform.firmware"),
    ("secure_boot", "platform.secure_boot"),
];

const FINISHED_OP_RETENTION: Duration = Duration::from_secs(300);
/// Progress events per operation are journaled at most this often (state changes always are).
const PROGRESS_EVENT_INTERVAL: Duration = Duration::from_millis(250);

struct LiveOp {
    tx: watch::Sender<pb::Operation>,
    cancel: CancellationToken,
    last_progress_event: Option<Instant>,
}

pub struct Daemon {
    data_dir: PathBuf,
    pub(crate) registry: Mutex<Registry>,
    pub(crate) events: broadcast::Sender<pb::Event>,
    pub(crate) appended: AtomicU64,
    /// Cancelled when the daemon stops; ends every open watch stream so shutdown is not blocked.
    pub shutdown: CancellationToken,
    engines: HashMap<EngineKind, Arc<dyn Engine>>,
    default_engine: Option<EngineKind>,
    folder_locks: Mutex<HashMap<Uuid, FolderLock>>,
    vm_locks: Mutex<HashMap<Uuid, Arc<tokio::sync::Mutex<()>>>>,
    ops: Mutex<HashMap<String, LiveOp>>,
}

pub struct UpdateOutcome {
    pub machine: pb::Machine,
    pub field_results: Vec<pb::FieldResult>,
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn api_apply_mode(m: ApplyMode) -> pb::ApplyMode {
    match m {
        ApplyMode::Live => pb::ApplyMode::Live,
        ApplyMode::NextBoot => pb::ApplyMode::NextBoot,
        ApplyMode::NeedsPowerOff => pb::ApplyMode::NeedsPowerOff,
    }
}

pub fn summary_to_pb(s: &MachineSummary) -> pb::MachineSummary {
    pb::MachineSummary {
        id: s.id.to_string(),
        display_name: s.name.clone(),
        state: s.state.as_str().into(),
        os_type: s.os_type.clone(),
        groups: s.groups.clone(),
        tags: s.tags.clone(),
        memory_mib: s.memory_mib,
        cpu_count: s.cpu_count,
        engine: s.engine.as_str().into(),
        snapshot_count: s.snapshot_count,
        accessible: s.accessible,
        error_code: s.error_code.clone().unwrap_or_default(),
    }
}

fn config_from_manifest(m: &MachineManifest) -> pb::MachineConfig {
    pb::MachineConfig {
        display_name: m.name.clone(),
        os_type: m.os_type.clone(),
        groups: m.groups.clone(),
        tags: m.tags.clone(),
        cpu_count: m.cpu.count,
        memory_mib: m.memory.size_mib,
        vram_mib: m.display.vram_mib,
        monitors: m.display.monitors.into(),
        firmware: m.platform.firmware.as_str().into(),
        secure_boot: m.platform.secure_boot,
    }
}

/// Copy one API field into the manifest.
fn apply_field(m: &mut MachineManifest, c: &pb::MachineConfig, api_path: &str) -> ApiResult<()> {
    match api_path {
        "display_name" => m.name = c.display_name.trim().to_string(),
        "os_type" => m.os_type = c.os_type.clone(),
        "groups" => m.groups = c.groups.clone(),
        "tags" => m.tags = c.tags.clone(),
        "cpu_count" => m.cpu.count = c.cpu_count,
        "memory_mib" => m.memory.size_mib = c.memory_mib,
        "vram_mib" => m.display.vram_mib = c.vram_mib,
        "monitors" => {
            m.display.monitors = u8::try_from(c.monitors)
                .map_err(|_| ApiError::bad_request("monitors out of range"))?
        }
        "firmware" => {
            m.platform.firmware = Firmware::parse(&c.firmware).ok_or_else(|| {
                ApiError::bad_request(format!(
                    "firmware must be bios or efi, got `{}`",
                    c.firmware
                ))
            })?
        }
        "secure_boot" => m.platform.secure_boot = c.secure_boot,
        other => {
            return Err(ApiError::bad_request(format!(
                "unknown update_mask path `{other}`"
            )));
        }
    }
    Ok(())
}

fn manifest_path_of(api_path: &str) -> Option<&'static str> {
    MASK_PATHS
        .iter()
        .find(|(a, _)| *a == api_path)
        .map(|(_, m)| *m)
}

fn invalid_manifest(issues: &[oxbox_model::Issue]) -> ApiError {
    let detail = issues
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ");
    OxeError::with(OxeCode::CFG_INVALID_MANIFEST, detail).into()
}

fn op_to_record(op: &pb::Operation) -> OperationRecord {
    OperationRecord {
        id: op.name.clone(),
        kind: op.kind.clone(),
        target: Some(op.target.clone()),
        state: op.state.clone(),
        percent: op.percent,
        step: Some(op.step.clone()).filter(|s| !s.is_empty()),
        started_at: Some(op.started_at.clone()),
        finished_at: Some(op.finished_at.clone()).filter(|s| !s.is_empty()),
        error_json: op.error.as_ref().map(|e| {
            serde_json::json!({"code": e.code, "message": e.message, "message_vi": e.message_vi, "hint": e.hint, "detail": e.detail})
                .to_string()
        }),
        result: None,
    }
}

fn op_from_record(r: OperationRecord) -> pb::Operation {
    let error = r
        .error_json
        .and_then(|j| serde_json::from_str::<serde_json::Value>(&j).ok())
        .map(|v| {
            let s = |k: &str| {
                v.get(k)
                    .and_then(|x| x.as_str())
                    .unwrap_or_default()
                    .to_string()
            };
            pb::OperationError {
                code: s("code"),
                message: s("message"),
                message_vi: s("message_vi"),
                hint: s("hint"),
                detail: s("detail"),
            }
        });
    let done = matches!(r.state.as_str(), "succeeded" | "failed" | "cancelled");
    pb::Operation {
        name: r.id,
        kind: r.kind,
        target: r.target.unwrap_or_default(),
        state: r.state,
        percent: r.percent,
        step: r.step.unwrap_or_default(),
        done,
        cancellable: false,
        error,
        started_at: r.started_at.unwrap_or_default(),
        finished_at: r.finished_at.unwrap_or_default(),
    }
}

fn op_error(e: &ApiError) -> pb::OperationError {
    pb::OperationError {
        code: e.code.code.into(),
        message: e.code.en.into(),
        message_vi: e.code.vi.into(),
        hint: e.code.hint_en.into(),
        detail: e.detail.clone().unwrap_or_default(),
    }
}

impl Daemon {
    pub fn new(
        data_dir: PathBuf,
        registry: Registry,
        engines: Vec<Arc<dyn Engine>>,
        default_engine: Option<EngineKind>,
    ) -> Arc<Self> {
        let engines: HashMap<_, _> = engines.into_iter().map(|e| (e.kind(), e)).collect();
        let default_engine = default_engine.or_else(|| engines.keys().next().copied());
        Arc::new(Self {
            data_dir,
            registry: Mutex::new(registry),
            events: broadcast::channel(BROADCAST_BUFFER).0,
            appended: AtomicU64::new(0),
            shutdown: CancellationToken::new(),
            engines,
            default_engine,
            folder_locks: Mutex::new(HashMap::new()),
            vm_locks: Mutex::new(HashMap::new()),
            ops: Mutex::new(HashMap::new()),
        })
    }

    pub fn engine_kinds(&self) -> Vec<String> {
        let mut v: Vec<_> = self
            .engines
            .keys()
            .map(|k| k.as_str().to_string())
            .collect();
        v.sort();
        v
    }

    pub fn registry_schema(&self) -> u32 {
        self.registry.lock().unwrap().schema_version().unwrap_or(0)
    }

    fn engine(&self, kind: EngineKind) -> ApiResult<Arc<dyn Engine>> {
        self.engines.get(&kind).cloned().ok_or_else(|| {
            OxeError::with(
                OxeCode::ENG_NOT_AVAILABLE,
                format!("engine `{}` is not enabled", kind.as_str()),
            )
            .into()
        })
    }

    fn vm_lock(&self, id: Uuid) -> Arc<tokio::sync::Mutex<()>> {
        self.vm_locks.lock().unwrap().entry(id).or_default().clone()
    }

    /// Accept `machines/{uuid}`, `machines/{name}`, a bare UUID or a bare name.
    pub fn resolve(&self, name: &str) -> ApiResult<oxbox_store::MachineRow> {
        let key = name.strip_prefix("machines/").unwrap_or(name);
        if key.is_empty() {
            return Err(ApiError::bad_request("machine name is required"));
        }
        let reg = self.registry.lock().unwrap();
        let row = match Uuid::parse_str(key) {
            Ok(id) => reg.get_machine(id)?,
            Err(_) => None,
        };
        let row = match row {
            Some(r) => Some(r),
            None => reg.find_by_name(key)?,
        };
        row.ok_or_else(|| ApiError::not_found(format!("machines/{key}")))
    }

    async fn current_state(&self, row: &oxbox_store::MachineRow) -> MachineState {
        match self.engines.get(&row.engine) {
            Some(e) => e.state(row.id).await.unwrap_or(MachineState::PoweredOff),
            None => row
                .summary
                .as_ref()
                .map_or(MachineState::PoweredOff, |s| s.state),
        }
    }

    fn load_manifest(row: &oxbox_store::MachineRow) -> ApiResult<MachineManifest> {
        Ok(read_manifest(&row.folder)?.into_manifest())
    }

    fn machine_pb(
        row: &oxbox_store::MachineRow,
        m: &MachineManifest,
        summary: &MachineSummary,
    ) -> pb::Machine {
        pb::Machine {
            name: format!("machines/{}", row.id),
            etag: row.config_rev.to_string(),
            summary: Some(summary_to_pb(summary)),
            config: Some(config_from_manifest(m)),
            manifest_toml: m.to_toml(),
            folder: row.folder.to_string_lossy().into_owned(),
        }
    }

    /// Reconcile the registry with the manifests on disk and register machines with their engines.
    pub async fn reconcile(&self) -> ApiResult<()> {
        let failed = serde_json::json!({"code": OxeCode::ENG_FAILED.code, "message": "daemon restarted while the operation was running"});
        self.registry
            .lock()
            .unwrap()
            .fail_orphaned_operations(&failed.to_string())?;
        let rows = self.registry.lock().unwrap().list_machines()?;
        for row in rows {
            let mut summary = row.summary.clone().unwrap_or_else(|| {
                let mut m = MachineManifest::new(&row.name, "Other", row.engine);
                m.id = row.id;
                MachineSummary::from_manifest(&m, MachineState::PoweredOff)
            });
            match self.reconcile_one(&row).await {
                Ok((manifest, notice)) => {
                    let state = self.current_state(&row).await;
                    let fresh = MachineSummary {
                        error_code: notice.map(|c| c.code.to_string()),
                        ..MachineSummary::from_manifest(&manifest, state)
                    };
                    let changed_outside = fresh.name != summary.name
                        || fresh.groups != summary.groups
                        || fresh.tags != summary.tags
                        || fresh.cpu_count != summary.cpu_count
                        || fresh.memory_mib != summary.memory_mib
                        || fresh.os_type != summary.os_type;
                    if changed_outside {
                        tracing::info!(vm = %row.id, "manifest changed outside oxboxd; re-imported");
                        self.registry.lock().unwrap().update_machine(
                            &manifest,
                            row.config_rev,
                            &fresh,
                        )?;
                    } else {
                        self.registry.lock().unwrap().set_summary(row.id, &fresh)?;
                    }
                    if changed_outside || fresh.error_code.is_some() {
                        self.emit(
                            &format!("machines/{}", row.id),
                            "machine.updated",
                            summary_payload(&fresh),
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(vm = %row.id, error = %e.code, detail = ?e.detail, "machine needs attention");
                    summary.accessible = false;
                    summary.error_code = Some(e.code.code.to_string());
                    self.registry
                        .lock()
                        .unwrap()
                        .set_summary(row.id, &summary)?;
                    self.emit(
                        &format!("machines/{}", row.id),
                        "machine.updated",
                        summary_payload(&summary),
                    );
                }
            }
        }
        Ok(())
    }

    async fn reconcile_one(
        &self,
        row: &oxbox_store::MachineRow,
    ) -> ApiResult<(MachineManifest, Option<OxeCode>)> {
        let lock = FolderLock::acquire(&row.folder)?;
        let (manifest, notice) = match read_manifest(&row.folder)? {
            ReadOutcome::Ok(m) => (m, None),
            ReadOutcome::Recovered {
                manifest,
                corrupt_copy,
            } => {
                tracing::warn!(vm = %row.id, corrupt = ?corrupt_copy, "manifest restored from machine.toml.prev");
                (manifest, Some(OxeCode::CFG_MANIFEST_RECOVERED))
            }
        };
        if manifest.id != row.id {
            return Err(OxeError::with(
                OxeCode::CFG_INVALID_MANIFEST,
                format!("manifest id {} does not match registry", manifest.id),
            )
            .into());
        }
        self.engine(manifest.engine)?.register(&manifest).await?;
        self.folder_locks.lock().unwrap().insert(row.id, lock);
        Ok((manifest, notice))
    }

    /// Keep `summary_json` warm from engine events.
    pub fn spawn_event_pumps(self: &Arc<Self>) {
        for engine in self.engines.values() {
            let mut events = engine.events();
            let me = Arc::downgrade(self);
            tokio::spawn(async move {
                while let Some(EngineEvent::StateChanged { id, state }) = events.next().await {
                    let Some(me) = me.upgrade() else { break };
                    me.record_state(id, state);
                }
            });
        }
    }

    fn record_state(&self, id: Uuid, state: MachineState) {
        let changed = {
            let mut reg = self.registry.lock().unwrap();
            match reg.get_machine(id) {
                Ok(Some(row)) => match row.summary {
                    Some(mut s) if s.state != state => {
                        s.state = state;
                        match reg.set_summary(id, &s) {
                            Ok(()) => true,
                            Err(e) => {
                                tracing::warn!(vm = %id, error = %e, "failed to update read model");
                                false
                            }
                        }
                    }
                    _ => false,
                },
                _ => false,
            }
        };
        if changed {
            self.emit(
                &format!("machines/{id}"),
                "machine.state_changed",
                serde_json::json!({ "state": state }),
            );
        }
    }

    pub fn list(&self, state: &str, group: &str) -> ApiResult<Vec<pb::MachineSummary>> {
        let rows = self.registry.lock().unwrap().list_machines()?;
        Ok(rows
            .into_iter()
            .filter_map(|r| r.summary)
            .filter(|s| state.is_empty() || s.state.as_str() == state)
            .filter(|s| group.is_empty() || s.groups.iter().any(|g| g == group))
            .map(|s| summary_to_pb(&s))
            .collect())
    }

    pub async fn get(&self, name: &str) -> ApiResult<pb::Machine> {
        let row = self.resolve(name)?;
        let manifest = Self::load_manifest(&row)?;
        let mut summary = row
            .summary
            .clone()
            .unwrap_or_else(|| MachineSummary::from_manifest(&manifest, MachineState::PoweredOff));
        summary.state = self.current_state(&row).await;
        Ok(Self::machine_pb(&row, &manifest, &summary))
    }

    pub async fn create(&self, req: pb::CreateMachineRequest) -> ApiResult<pb::Machine> {
        let c = req.config.unwrap_or_default();
        let kind = if req.engine.is_empty() {
            self.default_engine.ok_or_else(|| {
                ApiError::from(OxeError::with(
                    OxeCode::ENG_NOT_AVAILABLE,
                    "no engine is enabled",
                ))
            })?
        } else {
            EngineKind::parse(&req.engine)
                .ok_or_else(|| ApiError::bad_request(format!("unknown engine `{}`", req.engine)))?
        };
        let engine = self.engine(kind)?;
        let os_type = if c.os_type.is_empty() {
            "Other".to_string()
        } else {
            c.os_type.clone()
        };
        let mut m = MachineManifest::new(c.display_name.trim(), os_type, kind);
        m.groups = c.groups.clone();
        m.tags = c.tags.clone();
        if c.cpu_count != 0 {
            m.cpu.count = c.cpu_count;
        }
        if c.memory_mib != 0 {
            m.memory.size_mib = c.memory_mib;
        }
        if c.vram_mib != 0 {
            m.display.vram_mib = c.vram_mib;
        }
        if c.monitors != 0 {
            apply_field(&mut m, &c, "monitors")?;
        }
        if !c.firmware.is_empty() {
            apply_field(&mut m, &c, "firmware")?;
        }
        m.platform.secure_boot = c.secure_boot;
        let issues = m.validate();
        if !issues.is_empty() {
            return Err(invalid_manifest(&issues));
        }
        if self
            .registry
            .lock()
            .unwrap()
            .find_by_name(&m.name)?
            .is_some()
        {
            return Err(OxeError::with(OxeCode::CFG_NAME_TAKEN, m.name.clone()).into());
        }

        let folder = if req.folder.is_empty() {
            self.data_dir.join("machines").join(m.id.to_string())
        } else {
            PathBuf::from(&req.folder)
        };
        if folder.join(oxbox_store::manifest_io::MANIFEST).exists() {
            return Err(ApiError::new(
                Code::AlreadyExists,
                OxeCode::API_BAD_REQUEST,
                format!("{} already holds a machine", folder.display()),
            ));
        }
        let created_folder = !folder.exists();
        let lock = FolderLock::acquire(&folder)?;
        let summary = MachineSummary::from_manifest(&m, MachineState::PoweredOff);
        let result = (|| -> Result<(), StoreError> {
            write_manifest(&folder, &m)?;
            self.registry
                .lock()
                .unwrap()
                .insert_machine(&m, &folder, &summary)
        })();
        if let Err(e) = result {
            drop(lock);
            if created_folder {
                let _ = std::fs::remove_dir_all(&folder);
            }
            return Err(e.into());
        }
        engine.register(&m).await?;
        self.folder_locks.lock().unwrap().insert(m.id, lock);
        tracing::info!(vm = %m.id, name = %m.name, folder = %folder.display(), "machine created");
        self.emit(
            &format!("machines/{}", m.id),
            "machine.created",
            summary_payload(&summary),
        );
        self.get(&m.id.to_string()).await
    }

    pub async fn update(&self, req: pb::UpdateMachineRequest) -> ApiResult<UpdateOutcome> {
        let row = self.resolve(&req.name)?;
        let paths = req.update_mask.map(|m| m.paths).unwrap_or_default();
        if paths.is_empty() {
            return Err(ApiError::bad_request(
                "update_mask must list at least one field",
            ));
        }
        let config = req.config.unwrap_or_default();
        let guard = self.vm_lock(row.id);
        let _serialized = guard.lock().await;
        // Re-read under the per-VM lock so the etag check sees the latest revision.
        let row = self.resolve(&row.id.to_string())?;
        if req.etag.is_empty() {
            return Err(ApiError::bad_request(
                "etag is required; read it from GetMachine",
            ));
        }
        if req.etag != row.config_rev.to_string() {
            return Err(ApiError {
                etag: Some(row.config_rev.to_string()),
                ..StoreError::Conflict {
                    current: row.config_rev,
                }
                .into()
            });
        }
        let engine = self.engine(row.engine)?;
        let mut manifest = Self::load_manifest(&row)?;
        let mut changed = Vec::new();
        for p in &paths {
            apply_field(&mut manifest, &config, p)?;
            changed.push(
                manifest_path_of(p)
                    .expect("apply_field accepted the path")
                    .to_string(),
            );
        }
        let issues = manifest.validate();
        let mut results = Vec::new();
        for (api, mpath) in paths.iter().zip(&changed) {
            let mode = engine.apply_mode(row.id, mpath).await?;
            results.push(pb::FieldResult {
                path: api.clone(),
                apply_mode: api_apply_mode(mode) as i32,
                issues: issues
                    .iter()
                    .filter(|i| i.path.starts_with(mpath))
                    .map(|i| i.message.clone())
                    .collect(),
            });
        }
        if req.validate_only {
            let summary = row.summary.clone().unwrap_or_else(|| {
                MachineSummary::from_manifest(&manifest, MachineState::PoweredOff)
            });
            return Ok(UpdateOutcome {
                machine: Self::machine_pb(&row, &manifest, &summary),
                field_results: results,
            });
        }
        if !issues.is_empty() {
            return Err(invalid_manifest(&issues));
        }
        if manifest.name != row.name
            && self
                .registry
                .lock()
                .unwrap()
                .find_by_name(&manifest.name)?
                .is_some()
        {
            return Err(OxeError::with(OxeCode::CFG_NAME_TAKEN, manifest.name.clone()).into());
        }
        let blocked: Vec<_> = results
            .iter()
            .filter(|r| r.apply_mode == pb::ApplyMode::NeedsPowerOff as i32)
            .map(|r| r.path.clone())
            .collect();
        if !blocked.is_empty() {
            return Err(OxeError::with(OxeCode::RT_NEEDS_POWER_OFF, blocked.join(", ")).into());
        }
        engine.apply(&manifest, &changed).await?;
        write_manifest(&row.folder, &manifest)?;
        let state = self.current_state(&row).await;
        let summary = MachineSummary::from_manifest(&manifest, state);
        self.registry
            .lock()
            .unwrap()
            .update_machine(&manifest, row.config_rev, &summary)?;
        drop(_serialized);
        let mut payload = summary_payload(&summary);
        payload["etag"] = (row.config_rev + 1).to_string().into();
        payload["changed"] = paths.clone().into();
        self.emit(&format!("machines/{}", row.id), "machine.updated", payload);
        Ok(UpdateOutcome {
            machine: self.get(&row.id.to_string()).await?,
            field_results: results,
        })
    }

    pub async fn delete(&self, name: &str, delete_files: bool) -> ApiResult<()> {
        let row = self.resolve(name)?;
        let guard = self.vm_lock(row.id);
        let _serialized = guard.lock().await;
        if let Some(engine) = self.engines.get(&row.engine) {
            match engine.unregister(row.id).await {
                Ok(()) | Err(oxbox_engine::EngineError::NotFound(_)) => {}
                Err(e) => return Err(e.into()),
            }
        }
        self.registry.lock().unwrap().delete_machine(row.id)?;
        self.folder_locks.lock().unwrap().remove(&row.id);
        self.vm_locks.lock().unwrap().remove(&row.id);
        if delete_files {
            remove_machine_folder(&row.folder)?;
        }
        tracing::info!(vm = %row.id, delete_files, "machine deleted");
        self.emit(
            &format!("machines/{}", row.id),
            "machine.deleted",
            serde_json::json!({ "files_deleted": delete_files }),
        );
        Ok(())
    }

    /// Start or stop: validates the transition, then runs the engine call as an Operation.
    pub async fn run_action(
        self: &Arc<Self>,
        name: &str,
        action: Action,
        force: bool,
    ) -> ApiResult<pb::Operation> {
        let row = self.resolve(name)?;
        let engine = self.engine(row.engine)?;
        let state = engine.state(row.id).await?;
        if MachineState::target_of(action) == Some(state) {
            return Err(OxeError::with(OxeCode::RT_ALREADY_IN_STATE, state.as_str()).into());
        }
        if !state.allows(action) {
            return Err(OxeError::with(
                OxeCode::RT_INVALID_TRANSITION,
                format!("machine is {}", state.as_str()),
            )
            .into());
        }
        let kind = match action {
            Action::Start => "start",
            Action::Stop => "stop",
            other => {
                return Err(ApiError::bad_request(format!(
                    "{other:?} is not supported yet"
                )));
            }
        };
        let (op_name, handle) = self.begin_op(kind, format!("machines/{}", row.id));
        let me = Arc::clone(self);
        let id = row.id;
        let op_for_task = op_name.clone();
        tokio::spawn(async move {
            let guard = me.vm_lock(id);
            let _serialized = guard.lock().await;
            me.set_op(&op_for_task, |o| o.state = "running".into());
            let result = match action {
                Action::Start => engine.start(id, StartOpts { headless: true }, handle).await,
                _ => {
                    engine
                        .stop(
                            id,
                            if force {
                                StopMode::PowerOff
                            } else {
                                StopMode::AcpiShutdown
                            },
                            handle,
                        )
                        .await
                }
            };
            if let Ok(state) = engine.state(id).await {
                me.record_state(id, state);
            }
            me.finish_op(&op_for_task, result.map_err(ApiError::from));
        });
        self.get_op(&op_name)
    }

    fn begin_op(self: &Arc<Self>, kind: &str, target: String) -> (String, OpHandle) {
        let name = format!("operations/{}", Uuid::new_v4());
        let op = pb::Operation {
            name: name.clone(),
            kind: kind.into(),
            target,
            state: "queued".into(),
            cancellable: true,
            started_at: now(),
            ..Default::default()
        };
        if let Err(e) = self
            .registry
            .lock()
            .unwrap()
            .upsert_operation(&op_to_record(&op))
        {
            tracing::warn!(op = %name, error = %e, "failed to persist operation");
        }
        let queued = operation_payload(&op);
        let (tx, _) = watch::channel(op);
        let cancel = CancellationToken::new();
        self.ops.lock().unwrap().insert(
            name.clone(),
            LiveOp {
                tx,
                cancel: cancel.clone(),
                last_progress_event: None,
            },
        );
        self.emit(&name, "operation.updated", queued);
        let me = Arc::downgrade(self);
        let progress_name = name.clone();
        let handle = OpHandle::new(
            move |percent, step| {
                if let Some(me) = me.upgrade() {
                    me.op_progress(&progress_name, percent, step);
                }
            },
            cancel,
        );
        (name, handle)
    }

    /// Progress tick: always visible to WatchOperation; journaled as an event at a bounded rate.
    fn op_progress(&self, name: &str, percent: u32, step: &str) {
        let op = {
            let mut ops = self.ops.lock().unwrap();
            let Some(live) = ops.get_mut(name) else {
                return;
            };
            live.tx.send_modify(|o| {
                o.percent = percent;
                o.step = step.to_string();
            });
            let due = live
                .last_progress_event
                .is_none_or(|t| t.elapsed() >= PROGRESS_EVENT_INTERVAL);
            if !due {
                return;
            }
            live.last_progress_event = Some(Instant::now());
            live.tx.borrow().clone()
        };
        self.emit(name, "operation.updated", operation_payload(&op));
    }

    /// Update a live operation for WatchOperation, without persisting or journaling.
    fn set_op_quiet(
        &self,
        name: &str,
        f: impl FnOnce(&mut pb::Operation),
    ) -> Option<pb::Operation> {
        let ops = self.ops.lock().unwrap();
        let live = ops.get(name)?;
        live.tx.send_modify(f);
        Some(live.tx.borrow().clone())
    }

    /// A state change: persist it and journal an event.
    fn set_op(&self, name: &str, f: impl FnOnce(&mut pb::Operation)) {
        let Some(op) = self.set_op_quiet(name, f) else {
            return;
        };
        if let Err(e) = self
            .registry
            .lock()
            .unwrap()
            .upsert_operation(&op_to_record(&op))
        {
            tracing::warn!(op = %name, error = %e, "failed to persist operation");
        }
        let kind = if op.done {
            "operation.finished"
        } else {
            "operation.updated"
        };
        self.emit(name, kind, operation_payload(&op));
    }

    fn finish_op(self: &Arc<Self>, name: &str, result: ApiResult<()>) {
        self.set_op(name, |o| {
            o.done = true;
            o.cancellable = false;
            o.finished_at = now();
            match &result {
                Ok(()) => {
                    o.state = "succeeded".into();
                    o.percent = 100;
                }
                Err(e) if e.code == OxeCode::RT_CANCELLED => {
                    o.state = "cancelled".into();
                    o.error = Some(op_error(e));
                }
                Err(e) => {
                    o.state = "failed".into();
                    o.error = Some(op_error(e));
                }
            }
        });
        // Late watchers read the final value from memory for a while, then from the registry.
        let me = Arc::downgrade(self);
        let key = name.to_string();
        tokio::spawn(async move {
            tokio::time::sleep(FINISHED_OP_RETENTION).await;
            if let Some(me) = me.upgrade() {
                me.ops.lock().unwrap().remove(&key);
            }
        });
        if let Err(e) = &result {
            tracing::warn!(op = %name, code = e.code.code, detail = ?e.detail, "operation failed");
        }
    }

    pub fn get_op(&self, name: &str) -> ApiResult<pb::Operation> {
        if let Some(live) = self.ops.lock().unwrap().get(name) {
            return Ok(live.tx.borrow().clone());
        }
        self.registry
            .lock()
            .unwrap()
            .get_operation(name)?
            .map(op_from_record)
            .ok_or_else(|| ApiError::not_found(name.to_string()))
    }

    pub fn list_ops(&self, limit: u32) -> ApiResult<Vec<pb::Operation>> {
        let limit = if limit == 0 { 50 } else { limit.min(500) };
        let records = self.registry.lock().unwrap().list_operations(limit)?;
        Ok(records
            .into_iter()
            .map(|r| self.get_op(&r.id).unwrap_or_else(|_| op_from_record(r)))
            .collect())
    }

    /// A receiver for a live operation, or `None` if it already finished in a previous daemon run.
    pub fn watch_op(
        &self,
        name: &str,
    ) -> ApiResult<Result<watch::Receiver<pb::Operation>, pb::Operation>> {
        if let Some(live) = self.ops.lock().unwrap().get(name) {
            return Ok(Ok(live.tx.subscribe()));
        }
        self.get_op(name).map(Err)
    }

    pub fn cancel_op(&self, name: &str) -> ApiResult<pb::Operation> {
        let op = self.get_op(name)?;
        if op.done {
            return Ok(op);
        }
        if let Some(live) = self.ops.lock().unwrap().get(name) {
            live.cancel.cancel();
        }
        self.set_op(name, |o| o.state = "cancelling".into());
        self.get_op(name)
    }
}

/// Remove a machine folder, refusing anything that does not look like one.
fn remove_machine_folder(folder: &Path) -> ApiResult<()> {
    if !folder.join(oxbox_store::manifest_io::MANIFEST).exists() {
        return Err(ApiError::bad_request(format!(
            "{} has no machine.toml; not deleting it",
            folder.display()
        )));
    }
    std::fs::remove_dir_all(folder)
        .map_err(|e| ApiError::new(Code::Internal, OxeCode::ENG_FAILED, e.to_string()))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use oxbox_engine::MockEngine;

    pub(crate) fn daemon(dir: &Path) -> Arc<Daemon> {
        let reg = Registry::open(&dir.join("registry.db")).unwrap();
        let engine: Arc<dyn Engine> = Arc::new(MockEngine::new(Duration::from_millis(20)));
        Daemon::new(dir.to_path_buf(), reg, vec![engine], None)
    }

    fn create_req(name: &str) -> pb::CreateMachineRequest {
        pb::CreateMachineRequest {
            config: Some(pb::MachineConfig {
                display_name: name.into(),
                os_type: "Ubuntu_64".into(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn mask(paths: &[&str]) -> Option<prost_types::FieldMask> {
        Some(prost_types::FieldMask {
            paths: paths.iter().map(|s| s.to_string()).collect(),
        })
    }

    async fn wait_done(d: &Arc<Daemon>, op: &str) -> pb::Operation {
        let mut rx = d.watch_op(op).unwrap().unwrap();
        loop {
            if rx.borrow().done {
                return rx.borrow().clone();
            }
            rx.changed().await.unwrap();
        }
    }

    #[tokio::test]
    async fn update_requires_power_off_for_memory_while_running() {
        let dir = tempfile::tempdir().unwrap();
        let d = daemon(dir.path());
        let vm = d.create(create_req("a")).await.unwrap();
        let op = d.run_action("a", Action::Start, false).await.unwrap();
        assert_eq!(wait_done(&d, &op.name).await.state, "succeeded");
        assert_eq!(d.list("running", "").unwrap().len(), 1);

        let current = d.get("a").await.unwrap();
        let req = |etag: String, validate_only| pb::UpdateMachineRequest {
            name: vm.name.clone(),
            config: Some(pb::MachineConfig {
                memory_mib: 4096,
                ..Default::default()
            }),
            update_mask: mask(&["memory_mib"]),
            etag,
            validate_only,
        };
        let dry = d.update(req(current.etag.clone(), true)).await.unwrap();
        assert_eq!(
            dry.field_results[0].apply_mode,
            pb::ApplyMode::NeedsPowerOff as i32
        );
        let err = d
            .update(req(current.etag.clone(), false))
            .await
            .err()
            .unwrap();
        assert_eq!(err.code, OxeCode::RT_NEEDS_POWER_OFF);

        let err = d.run_action("a", Action::Start, false).await.err().unwrap();
        assert_eq!(err.code, OxeCode::RT_ALREADY_IN_STATE);
    }

    #[tokio::test]
    async fn restart_recovers_truncated_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let folder;
        {
            let d = daemon(dir.path());
            let vm = d.create(create_req("b")).await.unwrap();
            folder = PathBuf::from(&vm.folder);
            let up = pb::UpdateMachineRequest {
                name: vm.name.clone(),
                config: Some(pb::MachineConfig {
                    cpu_count: 6,
                    ..Default::default()
                }),
                update_mask: mask(&["cpu_count"]),
                etag: vm.etag.clone(),
                validate_only: false,
            };
            d.update(up).await.unwrap();
        }
        let path = folder.join("machine.toml");
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, &text[..text.len() / 3]).unwrap();

        let d = daemon(dir.path());
        d.reconcile().await.unwrap();
        let vm = d.get("b").await.unwrap();
        let s = vm.summary.unwrap();
        assert!(s.accessible);
        assert_eq!(s.error_code, OxeCode::CFG_MANIFEST_RECOVERED.code);
        assert_eq!(
            vm.config.unwrap().cpu_count,
            2,
            "restored the previous manifest"
        );
    }
}
