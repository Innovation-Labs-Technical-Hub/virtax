# Events

`oxboxd` records every change as a by-value event in the registry's `event_journal` and broadcasts it to
watchers (`Events.Watch` in `crates/oxbox-api/proto/oxbox/v1/oxbox.proto`, `oxctl events watch` on the CLI).

## Delivery guarantees

- `seq` is assigned when the event is journaled. It increases by one per event and is never reused, even after
  the journal is trimmed.
- A watcher gets every event after its cursor, in `seq` order, with no gaps and no duplicates (before filters are
  applied). A watcher that falls behind the live buffer (1,024 events) catches up from the journal, so slow
  clients cost journal reads rather than daemon memory.
- Retention is 100,000 events or 24 hours, whichever is smaller. A cursor outside the journal fails with
  `OUT_OF_RANGE` / `OXE-API-410`. The client should reload state (`ListMachines`), then watch from
  `VersionInfo.last_event_seq`.
- When the daemon stops, open streams end. `oxctl events watch` reconnects with backoff (100 ms doubling to
  500 ms) and resumes from the last `seq` it printed.

## Filters

- `resource_prefixes`: for example `machines/` or `operations/<id>`.
- `kinds`: an exact kind, or a family such as `machine` (which matches every `machine.*` kind).

## Kinds and payloads

| Kind | Resource | Payload |
|---|---|---|
| `machine.created` | `machines/{id}` | `{"summary": MachineSummary}` |
| `machine.updated` | `machines/{id}` | `{"summary": …, "etag": "<new>", "changed": ["memory_mib", …]}` after `UpdateMachine`; `{"summary": …}` when startup reconcile re-imported a manifest or flagged the machine |
| `machine.state_changed` | `machines/{id}` | `{"state": "running"}` |
| `machine.deleted` | `machines/{id}` | `{"files_deleted": false}` |
| `operation.updated` | `operations/{id}` | `{"kind", "target", "state", "percent", "step", "done": false, "error": null}`. Every state change is recorded; progress ticks are recorded at most 4 times per second per operation |
| `operation.finished` | `operations/{id}` | The same shape, with `done: true` and `error` set on failure or cancellation |

`MachineSummary` uses the field names in `crates/oxbox-model/src/summary.rs`.

## CLI

```sh
oxctl events watch                         # from now
oxctl --json events watch --since 0        # everything retained, as NDJSON
oxctl events watch --kind machine --resource machines/ --count 10
```
