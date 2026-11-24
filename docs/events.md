# Events

`virtaxd` records every change as a by-value event in the registry's `event_journal` and broadcasts it to
watchers (`Events.Watch` in `crates/virtax-api/proto/virtax/v1/virtax.proto`, `virtaxctl events watch` on the CLI).

## Delivery guarantees

- `seq` is assigned when the event is journaled. It increases by one per event and is never reused, even after
  the journal is trimmed.
- A watcher gets every event after its cursor, in `seq` order, with no gaps and no duplicates (before filters are
  applied). A watcher that falls behind the live buffer (1,024 events) catches up from the journal, so slow
  clients cost journal reads rather than daemon memory.
- Retention is 100,000 events or 24 hours, whichever is smaller. A cursor outside the journal fails with
  `OUT_OF_RANGE` / `VTX-API-410`. The client should reload state (`ListMachines`), then watch from
  `VersionInfo.last_event_seq`.
- When the daemon stops, open streams end. `virtaxctl events watch` reconnects with backoff (100 ms doubling to
  500 ms) and resumes from the last `seq` it printed.

## Filters

- `resource_prefixes`: for example `machines/` or `operations/<id>`.
- `kinds`: an exact kind, or a family such as `machine` (which matches every `machine.*` kind).

## Kinds and payloads

| Kind | Resource | Payload |
|---|---|---|
| `machine.created` | `machines/{id}` | `{"summary": MachineSummary}` |
| `machine.updated` | `machines/{id}` | After `UpdateMachine`: `{"summary", "etag", "changed": [fields applied now], "queued": [fields queued until power-off]}`. When queued changes are applied: the same with `"applied_pending": true`. After `CancelPendingChange`: `{"summary", "etag", "cancelled_pending": [field]}`. After startup reconcile re-imports or flags a machine: `{"summary"}` |
| `machine.state_changed` | `machines/{id}` | `{"state": "running"}` |
| `machine.deleted` | `machines/{id}` | `{"files_deleted": false}` |
| `operation.updated` | `operations/{id}` | `{"kind", "target", "state", "percent", "step", "done": false, "error": null}`. Every state change is recorded; progress ticks are recorded at most 4 times per second per operation |
| `operation.finished` | `operations/{id}` | The same shape, with `done: true` and `error` set on failure or cancellation |

`MachineSummary` uses the field names in `crates/virtax-model/src/summary.rs`.

## CLI

```sh
virtaxctl events watch                         # from now
virtaxctl --json events watch --since 0        # everything retained, as NDJSON
virtaxctl events watch --kind machine --resource machines/ --count 10
```
