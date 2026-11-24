//! Per-user SQLite registry (WAL). Holds the machine index, the `summary_json` read model and
//! operation history. VM configuration itself lives in each machine's `machine.toml`.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, Row, params};
use uuid::Uuid;
use virtax_model::{EngineKind, MachineManifest, MachineSummary, VtxCode, VtxError};

use crate::{StoreError, now};

/// Ordered schema migrations; version N is `MIGRATIONS[N - 1]`. Never edit a released entry.
const MIGRATIONS: &[&str] = &[r#"
CREATE TABLE machines(
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  folder TEXT NOT NULL,
  manifest_path TEXT NOT NULL,
  engine TEXT NOT NULL CHECK(engine IN ('vbox-legacy','virtax-vmm','mock')),
  os_type TEXT,
  arch TEXT CHECK(arch IN ('x86_64','aarch64')),
  config_rev INTEGER NOT NULL DEFAULT 0,
  summary_json TEXT,
  manifest_hash TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL);
CREATE UNIQUE INDEX machines_name ON machines(name);
CREATE TABLE machine_groups(machine_id TEXT NOT NULL REFERENCES machines(id) ON DELETE CASCADE,
  path TEXT NOT NULL, PRIMARY KEY(machine_id, path));
CREATE TABLE machine_tags(machine_id TEXT NOT NULL REFERENCES machines(id) ON DELETE CASCADE,
  tag TEXT NOT NULL, PRIMARY KEY(machine_id, tag));
CREATE TABLE pending_changes(machine_id TEXT NOT NULL REFERENCES machines(id) ON DELETE CASCADE,
  field_path TEXT NOT NULL, value_json TEXT NOT NULL, apply_mode TEXT NOT NULL, created_at TEXT NOT NULL,
  PRIMARY KEY(machine_id, field_path));
CREATE TABLE snapshots(id TEXT PRIMARY KEY, machine_id TEXT NOT NULL REFERENCES machines(id) ON DELETE CASCADE,
  parent_id TEXT REFERENCES snapshots(id), name TEXT NOT NULL, created_at TEXT NOT NULL,
  online INTEGER NOT NULL DEFAULT 0);
CREATE TABLE media(id TEXT PRIMARY KEY, location TEXT NOT NULL UNIQUE,
  kind TEXT NOT NULL CHECK(kind IN ('hdd','dvd','floppy')), format TEXT, parent_id TEXT REFERENCES media(id),
  logical_size INTEGER, size INTEGER, mtime INTEGER, state TEXT, last_error TEXT, checked_at TEXT);
CREATE TABLE medium_refs(medium_id TEXT NOT NULL REFERENCES media(id), machine_id TEXT NOT NULL REFERENCES machines(id),
  snapshot_id TEXT NOT NULL DEFAULT '', PRIMARY KEY(medium_id, machine_id, snapshot_id));
CREATE TABLE networks(id TEXT PRIMARY KEY, kind TEXT NOT NULL, name TEXT UNIQUE, config_json TEXT NOT NULL);
CREATE TABLE operations(id TEXT PRIMARY KEY, kind TEXT NOT NULL, target TEXT, state TEXT NOT NULL,
  percent INTEGER NOT NULL DEFAULT 0, step TEXT, eta_ms INTEGER, started_at TEXT, finished_at TEXT,
  error_json TEXT, result TEXT);
CREATE TABLE event_journal(seq INTEGER PRIMARY KEY AUTOINCREMENT, ts TEXT NOT NULL,
  resource TEXT NOT NULL, type TEXT NOT NULL, payload BLOB);
CREATE TABLE ui_prefs(scope TEXT NOT NULL, key TEXT NOT NULL, value TEXT, PRIMARY KEY(scope, key));
CREATE INDEX media_parent ON media(parent_id);
CREATE INDEX snapshots_machine ON snapshots(machine_id);
CREATE INDEX journal_resource ON event_journal(resource, seq);
CREATE INDEX operations_started ON operations(started_at);
"#];

#[derive(Debug, Clone, PartialEq)]
pub struct MachineRow {
    pub id: Uuid,
    pub name: String,
    pub folder: PathBuf,
    pub engine: EngineKind,
    pub config_rev: u64,
    pub summary: Option<MachineSummary>,
    /// Number of queued `pending_changes`.
    pub pending_count: u32,
}

/// A configuration change queued until the machine powers off.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingChange {
    /// API field path, e.g. `memory_mib`.
    pub field_path: String,
    pub value_json: String,
    /// `next_boot` or `needs_power_off`.
    pub apply_mode: String,
    pub created_at: String,
}

/// Pending-queue edits committed together with a configuration revision.
#[derive(Debug, Default)]
pub struct PendingEdit<'a> {
    /// Upserted: a newer value for the same field replaces the older one.
    pub queue: &'a [PendingChange],
    /// Removed, e.g. because the field was just set live or was applied.
    pub clear: &'a [String],
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct OperationRecord {
    pub id: String,
    pub kind: String,
    pub target: Option<String>,
    pub state: String,
    pub percent: u32,
    pub step: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub error_json: Option<String>,
    pub result: Option<String>,
}

/// One entry of the event journal: a by-value event with its global sequence number.
#[derive(Debug, Clone, PartialEq)]
pub struct EventRecord {
    pub seq: u64,
    pub ts: String,
    pub resource: String,
    pub kind: String,
    pub payload: String,
}

pub struct Registry {
    conn: Connection,
}

fn uuid_col(row: &Row<'_>, idx: usize) -> rusqlite::Result<Uuid> {
    let s: String = row.get(idx)?;
    Uuid::parse_str(&s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(idx, rusqlite::types::Type::Text, Box::new(e))
    })
}

fn machine_row(row: &Row<'_>) -> rusqlite::Result<MachineRow> {
    let engine: String = row.get(3)?;
    let summary: Option<String> = row.get(5)?;
    Ok(MachineRow {
        id: uuid_col(row, 0)?,
        name: row.get(1)?,
        folder: PathBuf::from(row.get::<_, String>(2)?),
        engine: EngineKind::parse(&engine).unwrap_or(EngineKind::Mock),
        config_rev: row.get::<_, i64>(4)? as u64,
        summary: summary.and_then(|s| serde_json::from_str(&s).ok()),
        pending_count: row.get::<_, i64>(6)? as u32,
    })
}

fn op_row(row: &Row<'_>) -> rusqlite::Result<OperationRecord> {
    Ok(OperationRecord {
        id: row.get(0)?,
        kind: row.get(1)?,
        target: row.get(2)?,
        state: row.get(3)?,
        percent: row.get::<_, i64>(4)? as u32,
        step: row.get(5)?,
        started_at: row.get(6)?,
        finished_at: row.get(7)?,
        error_json: row.get(8)?,
        result: row.get(9)?,
    })
}

const MACHINE_COLS: &str = "id, name, folder, engine, config_rev, summary_json,
    (SELECT COUNT(*) FROM pending_changes p WHERE p.machine_id = machines.id)";
const OP_COLS: &str =
    "id, kind, target, state, percent, step, started_at, finished_at, error_json, result";

fn name_conflict(e: rusqlite::Error, name: &str) -> StoreError {
    match &e {
        rusqlite::Error::SqliteFailure(f, Some(msg))
            if f.code == rusqlite::ErrorCode::ConstraintViolation
                && msg.contains("machines.name") =>
        {
            VtxError::with(VtxCode::CFG_NAME_TAKEN, name.to_string()).into()
        }
        _ => e.into(),
    }
}

impl Registry {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self, StoreError> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(std::time::Duration::from_millis(5000))?;
        let mut reg = Self { conn };
        reg.migrate()?;
        Ok(reg)
    }

    pub fn schema_version(&self) -> Result<u32, StoreError> {
        Ok(self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |r| r.get(0),
        )?)
    }

    fn migrate(&mut self) -> Result<(), StoreError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations(version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);",
        )?;
        let current = self.schema_version()? as usize;
        if current > MIGRATIONS.len() {
            return Err(VtxError::with(
                VtxCode::CFG_REGISTRY_TOO_NEW,
                format!(
                    "registry version {current}, this daemon knows {}",
                    MIGRATIONS.len()
                ),
            )
            .into());
        }
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(current) {
            let version = i + 1;
            let tx = self.conn.transaction()?;
            let applied = tx.execute_batch(sql).and_then(|()| {
                tx.execute(
                    "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                    params![version as i64, now()],
                )
            });
            match applied {
                Ok(_) => tx.commit()?,
                Err(e) => {
                    drop(tx); // rolls back
                    return Err(VtxError::with(
                        VtxCode::CFG_MIGRATION_FAILED,
                        format!("migration {version}: {e}"),
                    )
                    .into());
                }
            }
        }
        Ok(())
    }

    fn replace_groups_and_tags(
        tx: &rusqlite::Transaction<'_>,
        m: &MachineManifest,
    ) -> rusqlite::Result<()> {
        let id = m.id.to_string();
        tx.execute("DELETE FROM machine_groups WHERE machine_id = ?1", [&id])?;
        tx.execute("DELETE FROM machine_tags WHERE machine_id = ?1", [&id])?;
        for g in &m.groups {
            tx.execute(
                "INSERT OR IGNORE INTO machine_groups(machine_id, path) VALUES (?1, ?2)",
                params![id, g],
            )?;
        }
        for t in &m.tags {
            tx.execute(
                "INSERT OR IGNORE INTO machine_tags(machine_id, tag) VALUES (?1, ?2)",
                params![id, t],
            )?;
        }
        Ok(())
    }

    /// Register a new machine at revision 1.
    pub fn insert_machine(
        &mut self,
        m: &MachineManifest,
        folder: &Path,
        summary: &MachineSummary,
    ) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        let ts = now();
        tx.execute(
            "INSERT INTO machines(id, name, folder, manifest_path, engine, os_type, arch, config_rev, summary_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9, ?9)",
            params![
                m.id.to_string(),
                m.name,
                folder.to_string_lossy(),
                folder.join(crate::manifest_io::MANIFEST).to_string_lossy(),
                m.engine.as_str(),
                m.os_type,
                m.platform.arch.as_str(),
                serde_json::to_string(summary).expect("summary serializes"),
                ts,
            ],
        )
        .map_err(|e| name_conflict(e, &m.name))?;
        Self::replace_groups_and_tags(&tx, m)?;
        tx.commit()?;
        Ok(())
    }

    /// Record a configuration change if `expected_rev` is still current; returns the new revision.
    pub fn update_machine(
        &mut self,
        m: &MachineManifest,
        expected_rev: u64,
        summary: &MachineSummary,
    ) -> Result<u64, StoreError> {
        self.update_machine_with_pending(m, expected_rev, summary, PendingEdit::default())
    }

    /// Like `update_machine`, also queuing and clearing pending changes in the same transaction.
    pub fn update_machine_with_pending(
        &mut self,
        m: &MachineManifest,
        expected_rev: u64,
        summary: &MachineSummary,
        pending: PendingEdit<'_>,
    ) -> Result<u64, StoreError> {
        let tx = self.conn.transaction()?;
        let id = m.id.to_string();
        let current: Option<i64> = tx
            .query_row(
                "SELECT config_rev FROM machines WHERE id = ?1",
                [&id],
                |r| r.get(0),
            )
            .optional()?;
        let current = current.ok_or_else(|| StoreError::NotFound(id.clone()))? as u64;
        if current != expected_rev {
            return Err(StoreError::Conflict { current });
        }
        let next = current + 1;
        tx.execute(
            "UPDATE machines SET name = ?2, os_type = ?3, arch = ?4, config_rev = ?5, summary_json = ?6, updated_at = ?7 WHERE id = ?1",
            params![
                id,
                m.name,
                m.os_type,
                m.platform.arch.as_str(),
                next as i64,
                serde_json::to_string(summary).expect("summary serializes"),
                now()
            ],
        )
        .map_err(|e| name_conflict(e, &m.name))?;
        Self::replace_groups_and_tags(&tx, m)?;
        for path in pending.clear {
            tx.execute(
                "DELETE FROM pending_changes WHERE machine_id = ?1 AND field_path = ?2",
                params![id, path],
            )?;
        }
        for p in pending.queue {
            tx.execute(
                "INSERT INTO pending_changes(machine_id, field_path, value_json, apply_mode, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(machine_id, field_path) DO UPDATE SET value_json = excluded.value_json,
                   apply_mode = excluded.apply_mode, created_at = excluded.created_at",
                params![id, p.field_path, p.value_json, p.apply_mode, p.created_at],
            )?;
        }
        tx.commit()?;
        Ok(next)
    }

    /// Queued changes for a machine, oldest first.
    pub fn pending_changes(&self, id: Uuid) -> Result<Vec<PendingChange>, StoreError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT field_path, value_json, apply_mode, created_at FROM pending_changes
             WHERE machine_id = ?1 ORDER BY created_at, field_path",
        )?;
        let rows = stmt
            .query_map([id.to_string()], |r| {
                Ok(PendingChange {
                    field_path: r.get(0)?,
                    value_json: r.get(1)?,
                    apply_mode: r.get(2)?,
                    created_at: r.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Refresh the cached read model without touching the configuration revision.
    pub fn set_summary(&mut self, id: Uuid, summary: &MachineSummary) -> Result<(), StoreError> {
        let n = self.conn.execute(
            "UPDATE machines SET summary_json = ?2 WHERE id = ?1",
            params![
                id.to_string(),
                serde_json::to_string(summary).expect("summary serializes")
            ],
        )?;
        if n == 0 {
            Err(StoreError::NotFound(id.to_string()))
        } else {
            Ok(())
        }
    }

    pub fn get_machine(&self, id: Uuid) -> Result<Option<MachineRow>, StoreError> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {MACHINE_COLS} FROM machines WHERE id = ?1"),
                [id.to_string()],
                machine_row,
            )
            .optional()?)
    }

    pub fn find_by_name(&self, name: &str) -> Result<Option<MachineRow>, StoreError> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {MACHINE_COLS} FROM machines WHERE name = ?1"),
                [name],
                machine_row,
            )
            .optional()?)
    }

    pub fn list_machines(&self) -> Result<Vec<MachineRow>, StoreError> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {MACHINE_COLS} FROM machines ORDER BY name"
        ))?;
        let rows = stmt
            .query_map([], machine_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn delete_machine(&mut self, id: Uuid) -> Result<(), StoreError> {
        let n = self
            .conn
            .execute("DELETE FROM machines WHERE id = ?1", [id.to_string()])?;
        if n == 0 {
            Err(StoreError::NotFound(id.to_string()))
        } else {
            Ok(())
        }
    }

    pub fn upsert_operation(&mut self, op: &OperationRecord) -> Result<(), StoreError> {
        self.conn.execute(
            &format!(
                "INSERT INTO operations({OP_COLS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(id) DO UPDATE SET state = excluded.state, percent = excluded.percent, step = excluded.step,
                   finished_at = excluded.finished_at, error_json = excluded.error_json, result = excluded.result"
            ),
            params![
                op.id,
                op.kind,
                op.target,
                op.state,
                op.percent,
                op.step,
                op.started_at,
                op.finished_at,
                op.error_json,
                op.result
            ],
        )?;
        Ok(())
    }

    pub fn get_operation(&self, id: &str) -> Result<Option<OperationRecord>, StoreError> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {OP_COLS} FROM operations WHERE id = ?1"),
                [id],
                op_row,
            )
            .optional()?)
    }

    /// Most recent operations first.
    pub fn list_operations(&self, limit: u32) -> Result<Vec<OperationRecord>, StoreError> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {OP_COLS} FROM operations ORDER BY started_at DESC, id LIMIT ?1"
        ))?;
        let rows = stmt
            .query_map([limit], op_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Mark operations left unfinished by a previous daemon as failed (their worker is gone).
    pub fn fail_orphaned_operations(&mut self, error_json: &str) -> Result<usize, StoreError> {
        Ok(self.conn.execute(
            "UPDATE operations SET state = 'failed', finished_at = ?1, error_json = ?2
             WHERE state IN ('queued', 'running', 'cancelling')",
            params![now(), error_json],
        )?)
    }
}

impl Registry {
    /// Append an event; returns it with its assigned `seq`.
    pub fn append_event(
        &mut self,
        resource: &str,
        kind: &str,
        payload: &str,
    ) -> Result<EventRecord, StoreError> {
        let ts = now();
        self.conn.execute(
            "INSERT INTO event_journal(ts, resource, type, payload) VALUES (?1, ?2, ?3, ?4)",
            params![ts, resource, kind, payload.as_bytes()],
        )?;
        Ok(EventRecord {
            seq: self.conn.last_insert_rowid() as u64,
            ts,
            resource: resource.into(),
            kind: kind.into(),
            payload: payload.into(),
        })
    }

    /// Events with `seq > after`, oldest first.
    pub fn events_after(&self, after: u64, limit: u32) -> Result<Vec<EventRecord>, StoreError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT seq, ts, resource, type, payload FROM event_journal WHERE seq > ?1 ORDER BY seq LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![after as i64, limit], |r| {
                let payload: Vec<u8> = r.get::<_, Option<Vec<u8>>>(4)?.unwrap_or_default();
                Ok(EventRecord {
                    seq: r.get::<_, i64>(0)? as u64,
                    ts: r.get(1)?,
                    resource: r.get(2)?,
                    kind: r.get(3)?,
                    payload: String::from_utf8_lossy(&payload).into_owned(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// `(oldest retained seq, last assigned seq)`. With an empty journal the oldest is `last + 1`.
    /// The last seq survives trimming because AUTOINCREMENT never reuses numbers.
    pub fn event_bounds(&self) -> Result<(u64, u64), StoreError> {
        let last: i64 = self
            .conn
            .query_row(
                "SELECT seq FROM sqlite_sequence WHERE name = 'event_journal'",
                [],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(0);
        let oldest: Option<i64> =
            self.conn
                .query_row("SELECT MIN(seq) FROM event_journal", [], |r| r.get(0))?;
        Ok((oldest.unwrap_or(last + 1) as u64, last as u64))
    }

    /// Keep at most `max_events` events, none older than `max_age`. Returns how many were removed.
    pub fn trim_events(
        &mut self,
        max_events: u64,
        max_age: std::time::Duration,
    ) -> Result<usize, StoreError> {
        let (_, last) = self.event_bounds()?;
        let cutoff = chrono::Utc::now()
            - chrono::Duration::from_std(max_age).unwrap_or(chrono::Duration::MAX);
        let cutoff = cutoff.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        Ok(self.conn.execute(
            "DELETE FROM event_journal WHERE seq <= ?1 OR ts < ?2",
            params![last.saturating_sub(max_events) as i64, cutoff],
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use virtax_model::MachineState;

    fn sample(name: &str) -> (MachineManifest, MachineSummary) {
        let mut m = MachineManifest::new(name, "Linux_64", EngineKind::Mock);
        m.groups = vec!["/lab".into()];
        let s = MachineSummary::from_manifest(&m, MachineState::PoweredOff);
        (m, s)
    }

    #[test]
    fn migrates_once_and_reopens() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("registry.db");
        assert_eq!(
            Registry::open(&path).unwrap().schema_version().unwrap(),
            MIGRATIONS.len() as u32
        );
        assert_eq!(
            Registry::open(&path).unwrap().schema_version().unwrap(),
            MIGRATIONS.len() as u32
        );
    }

    #[test]
    fn refuses_newer_registry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("registry.db");
        let reg = Registry::open(&path).unwrap();
        reg.conn
            .execute("INSERT INTO schema_migrations VALUES (999, 'x')", [])
            .unwrap();
        drop(reg);
        match Registry::open(&path) {
            Err(StoreError::Vtx(e)) => assert_eq!(e.code, VtxCode::CFG_REGISTRY_TOO_NEW),
            Err(other) => panic!("unexpected {other:?}"),
            Ok(_) => panic!("newer registry must be refused"),
        }
    }

    #[test]
    fn etag_conflict_and_name_uniqueness() {
        let mut reg = Registry::open_in_memory().unwrap();
        let (mut m, s) = sample("a");
        reg.insert_machine(&m, Path::new("/tmp/a"), &s).unwrap();
        m.cpu.count = 4;
        assert_eq!(reg.update_machine(&m, 1, &s).unwrap(), 2);
        assert!(matches!(
            reg.update_machine(&m, 1, &s),
            Err(StoreError::Conflict { current: 2 })
        ));

        let (m2, s2) = sample("a");
        match reg.insert_machine(&m2, Path::new("/tmp/a2"), &s2) {
            Err(StoreError::Vtx(e)) => assert_eq!(e.code, VtxCode::CFG_NAME_TAKEN),
            other => panic!("unexpected {other:?}"),
        }
        let row = reg.find_by_name("a").unwrap().unwrap();
        assert_eq!(row.config_rev, 2);
        assert_eq!(row.summary.unwrap().groups, vec!["/lab".to_string()]);
        reg.delete_machine(m.id).unwrap();
        assert!(reg.list_machines().unwrap().is_empty());
    }

    #[test]
    fn event_journal_append_read_and_trim() {
        let mut reg = Registry::open_in_memory().unwrap();
        assert_eq!(reg.event_bounds().unwrap(), (1, 0));
        for i in 0..10 {
            let e = reg
                .append_event("machines/x", "machine.updated", &format!("{{\"i\":{i}}}"))
                .unwrap();
            assert_eq!(e.seq, i + 1);
        }
        let after = reg.events_after(7, 100).unwrap();
        assert_eq!(
            after.iter().map(|e| e.seq).collect::<Vec<_>>(),
            vec![8, 9, 10]
        );
        assert_eq!(after[0].payload, "{\"i\":7}");

        assert_eq!(
            reg.trim_events(4, std::time::Duration::from_secs(3600))
                .unwrap(),
            6
        );
        assert_eq!(reg.event_bounds().unwrap(), (7, 10));
        assert_eq!(
            reg.trim_events(0, std::time::Duration::from_secs(3600))
                .unwrap(),
            4
        );
        assert_eq!(
            reg.event_bounds().unwrap(),
            (11, 10),
            "numbering continues after a full trim"
        );
        assert_eq!(reg.append_event("r", "k", "{}").unwrap().seq, 11);
    }

    #[test]
    fn pending_changes_are_upserted_cleared_and_cascade() {
        let mut reg = Registry::open_in_memory().unwrap();
        let (m, s) = sample("p");
        reg.insert_machine(&m, Path::new("/tmp/p"), &s).unwrap();
        let change = |path: &str, v: &str| PendingChange {
            field_path: path.into(),
            value_json: v.into(),
            apply_mode: "needs_power_off".into(),
            created_at: now(),
        };
        let q = [change("memory_mib", "4096"), change("cpu_count", "4")];
        reg.update_machine_with_pending(
            &m,
            1,
            &s,
            PendingEdit {
                queue: &q,
                clear: &[],
            },
        )
        .unwrap();
        let q = [change("memory_mib", "8192")];
        let clear = ["cpu_count".to_string()];
        reg.update_machine_with_pending(
            &m,
            2,
            &s,
            PendingEdit {
                queue: &q,
                clear: &clear,
            },
        )
        .unwrap();
        let pending = reg.pending_changes(m.id).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(
            pending[0].value_json, "8192",
            "newer value replaces the older one"
        );
        assert_eq!(reg.get_machine(m.id).unwrap().unwrap().pending_count, 1);
        reg.delete_machine(m.id).unwrap();
        assert!(reg.pending_changes(m.id).unwrap().is_empty());
    }

    #[test]
    fn operations_round_trip_and_orphans_fail() {
        let mut reg = Registry::open_in_memory().unwrap();
        let mut op = OperationRecord {
            id: "op1".into(),
            kind: "start".into(),
            state: "running".into(),
            started_at: Some(now()),
            ..Default::default()
        };
        reg.upsert_operation(&op).unwrap();
        op.percent = 50;
        reg.upsert_operation(&op).unwrap();
        assert_eq!(reg.get_operation("op1").unwrap().unwrap().percent, 50);
        assert_eq!(reg.fail_orphaned_operations("{}").unwrap(), 1);
        assert_eq!(reg.list_operations(10).unwrap()[0].state, "failed");
    }
}
