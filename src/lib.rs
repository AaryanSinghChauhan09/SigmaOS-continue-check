//! SigmaOS Continue-Check — transaction checkpoint and state resumption utility.
//!
//! Principles: Fail-Fast, Defensive Programming, Command-Query Separation,
//! Single Responsibility, Deterministic Testing, Safe Concurrency.

pub mod checkpoint;
pub mod journal;

pub use checkpoint::{Checkpoint, CheckpointId, CheckpointStatus};
pub use journal::{CheckpointJournal, JournalError};
