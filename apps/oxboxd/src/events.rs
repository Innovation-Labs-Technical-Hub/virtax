//! By-value events: every change is appended to `event_journal` (assigning its `seq`) and broadcast.
//!
//! A watcher replays the journal after its cursor, then follows the broadcast. A watcher that falls
//! behind the broadcast buffer does not lose events: it goes back to the journal from its last
//! delivered `seq`. Memory per watcher is therefore bounded by its small outgoing queue.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use oxbox_api::v1 as pb;
use oxbox_model::{MachineSummary, OxeCode, OxeError};
use oxbox_store::EventRecord;
use serde_json::{Value, json};
use tokio::sync::{broadcast, mpsc};

use crate::daemon::Daemon;
use crate::error::{ApiError, ApiResult};

/// Live broadcast buffer shared by all watchers.
pub const BROADCAST_BUFFER: usize = 1024;
/// Journal retention: at most this many events, none older than `JOURNAL_MAX_AGE`.
pub const JOURNAL_MAX_EVENTS: u64 = 100_000;
pub const JOURNAL_MAX_AGE: Duration = Duration::from_secs(24 * 3600);
/// Trim the journal once per this many appended events.
const TRIM_EVERY: u64 = 1000;
const CATCH_UP_BATCH: u32 = 500;
/// Per-watcher outgoing queue.
const WATCHER_QUEUE: usize = 64;

#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    pub resource_prefixes: Vec<String>,
    pub kinds: Vec<String>,
}

impl EventFilter {
    /// A kind filter `machine` matches every `machine.*` kind.
    pub fn matches(&self, e: &pb::Event) -> bool {
        let resource_ok = self.resource_prefixes.is_empty()
            || self
                .resource_prefixes
                .iter()
                .any(|p| e.resource.starts_with(p.as_str()));
        let kind_ok = self.kinds.is_empty()
            || self.kinds.iter().any(|k| {
                e.kind == *k
                    || (e.kind.starts_with(k.as_str()) && e.kind[k.len()..].starts_with('.'))
            });
        resource_ok && kind_ok
    }
}

fn to_pb(r: EventRecord) -> pb::Event {
    pb::Event {
        seq: r.seq,
        ts: r.ts,
        resource: r.resource,
        kind: r.kind,
        payload_json: r.payload,
    }
}

fn resync(detail: impl Into<String>) -> ApiError {
    OxeError::with(OxeCode::API_RESYNC_REQUIRED, detail).into()
}

pub fn summary_payload(s: &MachineSummary) -> Value {
    json!({ "summary": s })
}

pub fn operation_payload(o: &pb::Operation) -> Value {
    json!({
        "kind": o.kind,
        "target": o.target,
        "state": o.state,
        "percent": o.percent,
        "step": o.step,
        "done": o.done,
        "error": o.error.as_ref().map(|e| json!({"code": e.code, "message": e.message, "detail": e.detail})),
    })
}

impl Daemon {
    /// Journal and broadcast an event. Must not be called while holding the registry lock.
    pub fn emit(&self, resource: &str, kind: &str, payload: Value) {
        let mut reg = self.registry.lock().unwrap();
        match reg.append_event(resource, kind, &payload.to_string()) {
            // Broadcast under the registry lock so live order always equals `seq` order.
            Ok(rec) => {
                let _ = self.events.send(to_pb(rec));
            }
            Err(e) => tracing::error!(resource, kind, error = %e, "failed to journal event"),
        }
        if self.appended.fetch_add(1, Ordering::Relaxed) % TRIM_EVERY == TRIM_EVERY - 1
            && let Err(e) = reg.trim_events(JOURNAL_MAX_EVENTS, JOURNAL_MAX_AGE)
        {
            tracing::warn!(error = %e, "failed to trim event journal");
        }
    }

    pub fn last_event_seq(&self) -> ApiResult<u64> {
        Ok(self.registry.lock().unwrap().event_bounds()?.1)
    }

    pub fn trim_journal(&self) -> ApiResult<usize> {
        Ok(self
            .registry
            .lock()
            .unwrap()
            .trim_events(JOURNAL_MAX_EVENTS, JOURNAL_MAX_AGE)?)
    }

    /// Start a watcher. Errors after the stream started (e.g. the journal was trimmed past a slow
    /// watcher) arrive as the stream's last item.
    pub fn watch_events(
        self: &Arc<Self>,
        since: Option<u64>,
        filter: EventFilter,
    ) -> ApiResult<mpsc::Receiver<ApiResult<pb::Event>>> {
        // Subscribe before reading the bounds so nothing falls between replay and live.
        let mut live = self.events.subscribe();
        let (oldest, last) = self.registry.lock().unwrap().event_bounds()?;
        let mut cursor = match since {
            None => last,
            Some(s) if s > last => {
                return Err(resync(format!(
                    "cursor {s} is ahead of the journal (last seq {last})"
                )));
            }
            Some(s) if s + 1 < oldest => {
                return Err(resync(format!(
                    "cursor {s} is older than the oldest retained event {oldest}"
                )));
            }
            Some(s) => s,
        };

        let (tx, rx) = mpsc::channel(WATCHER_QUEUE);
        let daemon = Arc::downgrade(self);
        let shutdown = self.shutdown.clone();
        tokio::spawn(async move {
            loop {
                // Replay from the journal until caught up.
                loop {
                    let Some(d) = daemon.upgrade() else { return };
                    let batch = d
                        .registry
                        .lock()
                        .unwrap()
                        .events_after(cursor, CATCH_UP_BATCH);
                    drop(d);
                    let batch = match batch {
                        Ok(b) => b,
                        Err(e) => {
                            let _ = tx.send(Err(e.into())).await;
                            return;
                        }
                    };
                    let Some(first) = batch.first() else { break };
                    if first.seq != cursor + 1 {
                        let _ = tx
                            .send(Err(resync(format!("events after {cursor} were trimmed"))))
                            .await;
                        return;
                    }
                    for rec in batch {
                        cursor = rec.seq;
                        let e = to_pb(rec);
                        if filter.matches(&e) && tx.send(Ok(e)).await.is_err() {
                            return;
                        }
                    }
                }
                // Follow live events; on any gap, go back to the journal.
                loop {
                    let next = tokio::select! {
                        _ = shutdown.cancelled() => return,
                        next = live.recv() => next,
                    };
                    match next {
                        Ok(e) if e.seq <= cursor => continue,
                        Ok(e) if e.seq == cursor + 1 => {
                            cursor = e.seq;
                            if filter.matches(&e) && tx.send(Ok(e)).await.is_err() {
                                return;
                            }
                        }
                        Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => break,
                        Err(broadcast::error::RecvError::Closed) => return,
                    }
                }
            }
        });
        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon::tests::daemon;

    async fn collect(rx: &mut mpsc::Receiver<ApiResult<pb::Event>>, n: usize) -> Vec<pb::Event> {
        let mut out = Vec::new();
        while out.len() < n {
            let e = tokio::time::timeout(Duration::from_secs(5), rx.recv())
                .await
                .expect("event in time");
            out.push(
                e.expect("stream open")
                    .map_err(|e| e.code)
                    .expect("no error"),
            );
        }
        out
    }

    #[tokio::test]
    async fn burst_beyond_buffer_loses_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let d = daemon(dir.path());
        let mut rx = d.watch_events(None, EventFilter::default()).unwrap();
        let n = BROADCAST_BUFFER * 3;
        for i in 0..n {
            d.emit("machines/x", "machine.updated", json!({ "i": i }));
        }
        let got = collect(&mut rx, n).await;
        let seqs: Vec<u64> = got.iter().map(|e| e.seq).collect();
        assert_eq!(
            seqs,
            (1..=n as u64).collect::<Vec<_>>(),
            "contiguous, no loss, no duplicates"
        );
    }

    #[tokio::test]
    async fn replay_then_live_with_filters() {
        let dir = tempfile::tempdir().unwrap();
        let d = daemon(dir.path());
        d.emit("machines/a", "machine.created", json!({}));
        d.emit("operations/1", "operation.updated", json!({}));
        d.emit("machines/a", "machine.state_changed", json!({}));
        let filter = EventFilter {
            kinds: vec!["machine".into()],
            ..Default::default()
        };
        let mut rx = d.watch_events(Some(1), filter).unwrap();
        d.emit("machines/b", "machine.deleted", json!({}));
        d.emit("operations/2", "operation.finished", json!({}));
        let got = collect(&mut rx, 2).await;
        assert_eq!(
            got.iter()
                .map(|e| (e.seq, e.kind.as_str()))
                .collect::<Vec<_>>(),
            vec![(3, "machine.state_changed"), (4, "machine.deleted")]
        );
    }

    #[tokio::test]
    async fn stale_or_future_cursor_requires_resync() {
        let dir = tempfile::tempdir().unwrap();
        let d = daemon(dir.path());
        for _ in 0..10 {
            d.emit("machines/x", "machine.updated", json!({}));
        }
        d.registry
            .lock()
            .unwrap()
            .trim_events(3, JOURNAL_MAX_AGE)
            .unwrap();
        assert_eq!(
            d.watch_events(Some(2), EventFilter::default())
                .err()
                .unwrap()
                .code,
            OxeCode::API_RESYNC_REQUIRED
        );
        assert_eq!(
            d.watch_events(Some(99), EventFilter::default())
                .err()
                .unwrap()
                .code,
            OxeCode::API_RESYNC_REQUIRED
        );
        let mut rx = d.watch_events(Some(7), EventFilter::default()).unwrap();
        assert_eq!(collect(&mut rx, 3).await[0].seq, 8);
    }

    #[test]
    fn kind_prefix_needs_a_dot_boundary() {
        let f = EventFilter {
            kinds: vec!["machine".into()],
            ..Default::default()
        };
        let e = |kind: &str| pb::Event {
            kind: kind.into(),
            ..Default::default()
        };
        assert!(f.matches(&e("machine.created")));
        assert!(!f.matches(&e("machines.created")));
        assert!(!f.matches(&e("operation.updated")));
    }
}
