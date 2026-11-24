//! Rendering: JSON objects (stable field names, part of the scripting contract) and human tables.

use oxbox_api::v1 as pb;
use serde_json::{Value, json};

pub fn summary(s: &pb::MachineSummary) -> Value {
    json!({
        "id": s.id,
        "name": s.display_name,
        "state": s.state,
        "os_type": s.os_type,
        "groups": s.groups,
        "tags": s.tags,
        "memory_mib": s.memory_mib,
        "cpu_count": s.cpu_count,
        "engine": s.engine,
        "snapshot_count": s.snapshot_count,
        "accessible": s.accessible,
        "pending_change_count": s.pending_change_count,
        "error_code": if s.error_code.is_empty() { Value::Null } else { json!(s.error_code) },
    })
}

pub fn machine(m: &pb::Machine) -> Value {
    let c = m.config.clone().unwrap_or_default();
    json!({
        "name": m.name,
        "etag": m.etag,
        "folder": m.folder,
        "summary": m.summary.as_ref().map(summary),
        "pending_changes": m.pending_changes.iter().map(pending).collect::<Vec<_>>(),
        "config": {
            "display_name": c.display_name,
            "os_type": c.os_type,
            "groups": c.groups,
            "tags": c.tags,
            "cpu_count": c.cpu_count,
            "memory_mib": c.memory_mib,
            "vram_mib": c.vram_mib,
            "monitors": c.monitors,
            "firmware": c.firmware,
            "secure_boot": c.secure_boot,
        },
    })
}

pub fn operation(o: &pb::Operation) -> Value {
    json!({
        "name": o.name,
        "kind": o.kind,
        "target": o.target,
        "state": o.state,
        "percent": o.percent,
        "step": o.step,
        "done": o.done,
        "error": o.error.as_ref().map(|e| json!({"code": e.code, "message": e.message, "hint": e.hint, "detail": e.detail})),
        "started_at": o.started_at,
        "finished_at": if o.finished_at.is_empty() { Value::Null } else { json!(o.finished_at) },
    })
}

pub fn apply_mode(mode: i32) -> &'static str {
    match pb::ApplyMode::try_from(mode) {
        Ok(pb::ApplyMode::Live) => "live",
        Ok(pb::ApplyMode::NextBoot) => "next_boot",
        Ok(pb::ApplyMode::NeedsPowerOff) => "needs_power_off",
        _ => "unspecified",
    }
}

pub fn pending(p: &pb::PendingChange) -> Value {
    json!({
        "field": p.field_path,
        "value": serde_json::from_str::<Value>(&p.value_json).unwrap_or(Value::Null),
        "apply_mode": apply_mode(p.apply_mode),
        "created_at": p.created_at,
    })
}

pub fn field_result(r: &pb::FieldResult) -> Value {
    json!({"path": r.path, "apply_mode": apply_mode(r.apply_mode), "queued": r.queued, "issues": r.issues})
}

/// Left-aligned columns sized to their widest cell.
pub fn table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (w, cell) in widths.iter_mut().zip(row) {
            *w = (*w).max(cell.chars().count());
        }
    }
    let fmt_row = |cells: Vec<String>| {
        let line: Vec<String> = cells
            .iter()
            .zip(&widths)
            .map(|(c, w)| format!("{c:<w$}"))
            .collect();
        line.join("  ").trim_end().to_string()
    };
    let mut out = vec![fmt_row(headers.iter().map(|h| h.to_string()).collect())];
    out.extend(rows.iter().cloned().map(fmt_row));
    out.join("\n")
}
