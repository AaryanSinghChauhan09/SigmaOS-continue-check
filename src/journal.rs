//! Checkpoint journal for transaction logging and resume verification.
use crate::checkpoint::{Checkpoint, CheckpointId, CheckpointStatus};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum JournalError {
    CheckpointNotFound(CheckpointId),
    InvalidTransition { from: CheckpointStatus, to: CheckpointStatus },
    DuplicateId(CheckpointId),
}

impl fmt::Display for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JournalError::CheckpointNotFound(id) => write!(f, "Checkpoint {id} not found"),
            JournalError::InvalidTransition { from, to } => {
                write!(f, "Invalid status transition from {from} to {to}")
            }
            JournalError::DuplicateId(id) => write!(f, "Duplicate checkpoint ID {id}"),
        }
    }
}

pub struct CheckpointJournal {
    checkpoints: BTreeMap<CheckpointId, Checkpoint>,
    execution_order: Vec<CheckpointId>,
}

impl CheckpointJournal {
    pub fn new() -> Self {
        Self {
            checkpoints: BTreeMap::new(),
            execution_order: Vec::new(),
        }
    }

    // ── Queries (Command-Query Separation) ──────────────────────────────────
    pub fn count(&self) -> usize {
        self.checkpoints.len()
    }

    pub fn get(&self, id: CheckpointId) -> Option<&Checkpoint> {
        self.checkpoints.get(&id)
    }

    pub fn last_succeeded(&self) -> Option<&Checkpoint> {
        self.execution_order
            .iter()
            .rev()
            .filter_map(|id| self.checkpoints.get(id))
            .find(|c| c.status == CheckpointStatus::Succeeded)
    }

    pub fn resume_from(&self) -> Option<CheckpointId> {
        // First non-succeeded checkpoint in execution order
        self.execution_order
            .iter()
            .find(|id| {
                self.checkpoints
                    .get(id)
                    .map(|c| c.status != CheckpointStatus::Succeeded && c.status != CheckpointStatus::Skipped)
                    .unwrap_or(false)
            })
            .copied()
    }

    // ── Commands (Command-Query Separation) ──────────────────────────────────
    pub fn record_checkpoint(&mut self, checkpoint: Checkpoint) -> Result<(), JournalError> {
        let id = checkpoint.id;
        if self.checkpoints.contains_key(&id) {
            return Err(JournalError::DuplicateId(id));
        }
        self.checkpoints.insert(id, checkpoint);
        self.execution_order.push(id);
        Ok(())
    }

    pub fn mark_status(&mut self, id: CheckpointId, new_status: CheckpointStatus) -> Result<(), JournalError> {
        let entry = self.checkpoints.get_mut(&id).ok_or(JournalError::CheckpointNotFound(id))?;
        
        // Defensive validation of transition
        match (entry.status, new_status) {
            (CheckpointStatus::Succeeded, CheckpointStatus::Pending) => {
                return Err(JournalError::InvalidTransition {
                    from: entry.status,
                    to: new_status,
                });
            }
            _ => {
                entry.status = new_status;
                Ok(())
            }
        }
    }

    pub fn rollback_from(&mut self, start_id: CheckpointId) -> Result<usize, JournalError> {
        if !self.checkpoints.contains_key(&start_id) {
            return Err(JournalError::CheckpointNotFound(start_id));
        }

        let mut count = 0;
        let mut active = false;

        for id in self.execution_order.iter().rev() {
            if *id == start_id {
                active = true;
            }
            if active {
                if let Some(c) = self.checkpoints.get_mut(id) {
                    if c.status == CheckpointStatus::Succeeded || c.status == CheckpointStatus::Pending {
                        c.status = CheckpointStatus::RolledBack;
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }
}

impl Default for CheckpointJournal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_query_checkpoints() {
        // Arrange
        let mut journal = CheckpointJournal::new();
        let c1 = Checkpoint::new(1, "step_one");
        let c2 = Checkpoint::new(2, "step_two");

        // Act
        journal.record_checkpoint(c1).unwrap();
        journal.record_checkpoint(c2).unwrap();

        // Assert
        assert_eq!(journal.count(), 2);
        assert_eq!(journal.get(CheckpointId(1)).unwrap().name, "step_one");
    }

    #[test]
    fn test_mark_status_and_resume_calculation() {
        // Arrange
        let mut journal = CheckpointJournal::new();
        journal.record_checkpoint(Checkpoint::new(1, "download")).unwrap();
        journal.record_checkpoint(Checkpoint::new(2, "unpack")).unwrap();
        journal.record_checkpoint(Checkpoint::new(3, "install")).unwrap();

        // Act
        journal.mark_status(CheckpointId(1), CheckpointStatus::Succeeded).unwrap();

        // Assert
        assert_eq!(journal.last_succeeded().unwrap().name, "download");
        assert_eq!(journal.resume_from(), Some(CheckpointId(2)));
    }

    #[test]
    fn test_rollback_marks_subsequent_steps() {
        // Arrange
        let mut journal = CheckpointJournal::new();
        journal.record_checkpoint(Checkpoint::new(1, "prep")).unwrap();
        journal.record_checkpoint(Checkpoint::new(2, "apply")).unwrap();
        journal.mark_status(CheckpointId(1), CheckpointStatus::Succeeded).unwrap();
        journal.mark_status(CheckpointId(2), CheckpointStatus::Succeeded).unwrap();

        // Act - rollback from step 2
        let rolled_back = journal.rollback_from(CheckpointId(2)).unwrap();

        // Assert
        assert_eq!(rolled_back, 2);
        assert_eq!(journal.get(CheckpointId(2)).unwrap().status, CheckpointStatus::RolledBack);
        assert_eq!(journal.get(CheckpointId(1)).unwrap().status, CheckpointStatus::RolledBack);
    }

    #[test]
    fn test_duplicate_id_rejected() {
        // Arrange
        let mut journal = CheckpointJournal::new();
        journal.record_checkpoint(Checkpoint::new(10, "init")).unwrap();

        // Act
        let res = journal.record_checkpoint(Checkpoint::new(10, "init_dup"));

        // Assert
        assert_eq!(res, Err(JournalError::DuplicateId(CheckpointId(10))));
    }
}
