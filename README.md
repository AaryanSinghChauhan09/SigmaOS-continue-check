# SigmaOS Continue-Check

A small in-memory Rust model for recording ordered checkpoints, updating their
statuses, finding the last successful checkpoint, and selecting a checkpoint
from which a caller may resume.

## Library surface

- `Checkpoint` carries an ID, name, status, and optional string payload.
- `CheckpointJournal` rejects duplicate IDs, preserves insertion order, and
  provides lookup and status queries.
- `resume_from()` returns the first checkpoint in insertion order that is
  neither succeeded nor skipped.
- `rollback_from(id)` marks eligible checkpoints from that entry back through
  earlier entries in reverse execution order. It changes in-memory status; it
  does not undo external side effects.

```rust
use sigmaos_continue_check::{Checkpoint, CheckpointId, CheckpointJournal, CheckpointStatus};

let mut journal = CheckpointJournal::new();
journal.record_checkpoint(Checkpoint::new(1, "download")).unwrap();
journal.record_checkpoint(Checkpoint::new(2, "install")).unwrap();
journal.mark_status(CheckpointId(1), CheckpointStatus::Succeeded).unwrap();
let next = journal.resume_from();
# let _ = next;
```

## Persistence and recovery limits

The journal is process-local memory. It has no disk format, crash recovery,
durable transaction log, scheduling, or integration with the operations named
by checkpoints. Applications must persist state and make external actions
idempotent if they need reliable recovery after a process exits.

## Development

This crate has no third party dependencies. Run `cargo check` to compile the
library and `cargo test` to run its unit tests.
