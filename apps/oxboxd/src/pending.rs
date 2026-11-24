//! Changes queued while a machine runs (`pending_changes`), applied once it is powered off.
//!
//! Queued values are stored per API field as JSON. They are applied when the daemon sees the machine
//! reach PoweredOff or Aborted, at startup for machines that are off, and before every start.

use std::sync::Arc;

use oxbox_api::v1 as pb;
use oxbox_model::{MachineState, MachineSummary, OxeCode};
use oxbox_store::{PendingChange, PendingEdit};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::daemon::{Daemon, apply_field, invalid_manifest, manifest_path_of};
use crate::error::{ApiError, ApiResult};
use crate::events::summary_payload;

macro_rules! config_fields {
    ($($field:ident),* $(,)?) => {
        /// The value of one `MachineConfig` field as JSON.
        pub fn config_value(c: &pb::MachineConfig, path: &str) -> ApiResult<Value> {
            match path {
                $(stringify!($field) => Ok(json!(c.$field)),)*
                other => Err(ApiError::bad_request(format!("unknown field `{other}`"))),
            }
        }

        /// Set one `MachineConfig` field from JSON.
        pub fn set_config_value(c: &mut pb::MachineConfig, path: &str, v: Value) -> ApiResult<()> {
            match path {
                $(stringify!($field) => {
                    c.$field = serde_json::from_value(v)
                        .map_err(|e| ApiError::bad_request(format!("bad queued value for `{path}`: {e}")))?;
                    Ok(())
                })*
                other => Err(ApiError::bad_request(format!("unknown field `{other}`"))),
            }
        }
    };
}

config_fields!(
    display_name,
    os_type,
    groups,
    tags,
    cpu_count,
    memory_mib,
    vram_mib,
    monitors,
    firmware,
    secure_boot
);

pub fn apply_mode_name(mode: pb::ApplyMode) -> &'static str {
    match mode {
        pb::ApplyMode::NextBoot => "next_boot",
        pb::ApplyMode::NeedsPowerOff => "needs_power_off",
        pb::ApplyMode::Live => "live",
        pb::ApplyMode::Unspecified => "unspecified",
    }
}

pub fn pending_to_pb(p: &PendingChange) -> pb::PendingChange {
    let mode = match p.apply_mode.as_str() {
        "next_boot" => pb::ApplyMode::NextBoot,
        "needs_power_off" => pb::ApplyMode::NeedsPowerOff,
        _ => pb::ApplyMode::Unspecified,
    };
    pb::PendingChange {
        field_path: p.field_path.clone(),
        value_json: p.value_json.clone(),
        apply_mode: mode as i32,
        created_at: p.created_at.clone(),
    }
}

/// States in which queued hardware changes may be applied (no live or saved guest state).
fn can_apply(state: MachineState) -> bool {
    matches!(state, MachineState::PoweredOff | MachineState::Aborted)
}

impl Daemon {
    pub fn pending_changes(&self, id: Uuid) -> ApiResult<Vec<pb::PendingChange>> {
        let pending = self.registry.lock().unwrap().pending_changes(id)?;
        Ok(pending.iter().map(pending_to_pb).collect())
    }

    /// Apply the queue if the machine is off. Takes the per-VM lock.
    pub async fn apply_pending(&self, id: Uuid) -> ApiResult<bool> {
        let guard = self.vm_lock(id);
        let _serialized = guard.lock().await;
        self.apply_pending_locked(id).await
    }

    /// Apply the queue if the machine is off; the caller holds the per-VM lock.
    /// Returns whether anything was applied. On error the queue is left intact.
    pub async fn apply_pending_locked(&self, id: Uuid) -> ApiResult<bool> {
        let pending = self.registry.lock().unwrap().pending_changes(id)?;
        if pending.is_empty() {
            return Ok(false);
        }
        let row = self.resolve(&id.to_string())?;
        let engine = self.engine(row.engine)?;
        let state = engine.state(id).await?;
        if !can_apply(state) {
            return Ok(false);
        }
        let mut manifest = Self::load_manifest(&row)?;
        let mut config = pb::MachineConfig::default();
        let mut paths = Vec::new();
        let mut changed = Vec::new();
        for p in &pending {
            let value = serde_json::from_str(&p.value_json).map_err(|e| {
                ApiError::bad_request(format!("bad queued value for `{}`: {e}", p.field_path))
            })?;
            set_config_value(&mut config, &p.field_path, value)?;
            apply_field(&mut manifest, &config, &p.field_path)?;
            paths.push(p.field_path.clone());
            changed.push(
                manifest_path_of(&p.field_path)
                    .expect("queued paths are known")
                    .to_string(),
            );
        }
        let issues = manifest.validate();
        if !issues.is_empty() {
            return Err(invalid_manifest(&issues));
        }
        engine.apply(&manifest, &changed).await?;
        oxbox_store::write_manifest(&row.folder, &manifest)?;
        let summary = MachineSummary::from_manifest(&manifest, state);
        let rev = self.registry.lock().unwrap().update_machine_with_pending(
            &manifest,
            row.config_rev,
            &summary,
            PendingEdit {
                queue: &[],
                clear: &paths,
            },
        )?;
        tracing::info!(vm = %id, fields = ?paths, "applied pending changes");
        let mut payload = summary_payload(&summary);
        payload["etag"] = rev.to_string().into();
        payload["changed"] = paths.into();
        payload["applied_pending"] = true.into();
        self.emit(&format!("machines/{id}"), "machine.updated", payload);
        Ok(true)
    }

    /// Apply the queue in the background once the machine has stopped.
    pub fn spawn_apply_pending(self: &Arc<Self>, id: Uuid) {
        let me = Arc::clone(self);
        tokio::spawn(async move {
            if let Err(e) = me.apply_pending(id).await {
                tracing::warn!(vm = %id, code = e.code.code, detail = ?e.detail, "pending changes not applied; they stay queued");
            }
        });
    }

    /// Drop one queued change; bumps the configuration revision.
    pub async fn cancel_pending(&self, name: &str, field_path: &str) -> ApiResult<pb::Machine> {
        let row = self.resolve(name)?;
        let guard = self.vm_lock(row.id);
        let serialized = guard.lock().await;
        let row = self.resolve(&row.id.to_string())?;
        let pending = self.registry.lock().unwrap().pending_changes(row.id)?;
        if !pending.iter().any(|p| p.field_path == field_path) {
            return Err(ApiError::not_found(format!(
                "machines/{}/pending_changes/{field_path}",
                row.id
            )));
        }
        let manifest = Self::load_manifest(&row)?;
        let summary = row
            .summary
            .clone()
            .unwrap_or_else(|| MachineSummary::from_manifest(&manifest, MachineState::PoweredOff));
        let clear = [field_path.to_string()];
        let rev = self.registry.lock().unwrap().update_machine_with_pending(
            &manifest,
            row.config_rev,
            &summary,
            PendingEdit {
                queue: &[],
                clear: &clear,
            },
        )?;
        drop(serialized);
        let mut payload = summary_payload(&summary);
        payload["etag"] = rev.to_string().into();
        payload["cancelled_pending"] = json!(clear);
        self.emit(&format!("machines/{}", row.id), "machine.updated", payload);
        self.get(&row.id.to_string()).await
    }
}

/// Reject a queued value whose code would otherwise surface as a generic error.
pub fn needs_power_off(blocked: &[String]) -> ApiError {
    oxbox_model::OxeError::with(OxeCode::RT_NEEDS_POWER_OFF, blocked.join(", ")).into()
}
