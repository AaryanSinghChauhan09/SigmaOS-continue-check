//! Checkpoint definitions and states.
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CheckpointId(pub u64);

impl fmt::Display for CheckpointId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chkpt:{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointStatus {
    Pending,
    Succeeded,
    Failed,
    Skipped,
    RolledBack,
}

impl fmt::Display for CheckpointStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckpointStatus::Pending => write!(f, "PENDING"),
            CheckpointStatus::Succeeded => write!(f, "SUCCEEDED"),
            CheckpointStatus::Failed => write!(f, "FAILED"),
            CheckpointStatus::Skipped => write!(f, "SKIPPED"),
            CheckpointStatus::RolledBack => write!(f, "ROLLED_BACK"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    pub id: CheckpointId,
    pub name: String,
    pub status: CheckpointStatus,
    pub payload: Option<String>,
}

impl Checkpoint {
    pub fn new(id: u64, name: impl Into<String>) -> Self {
        Self {
            id: CheckpointId(id),
            name: name.into(),
            status: CheckpointStatus::Pending,
            payload: None,
        }
    }

    pub fn with_payload(mut self, payload: impl Into<String>) -> Self {
        self.payload = Some(payload.into());
        self
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self.status, CheckpointStatus::Succeeded | CheckpointStatus::Failed | CheckpointStatus::RolledBack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_defaults() {
        // Arrange & Act
        let c = Checkpoint::new(1, "migrate_db");
        // Assert
        assert_eq!(c.id, CheckpointId(1));
        assert_eq!(c.name, "migrate_db");
        assert_eq!(c.status, CheckpointStatus::Pending);
        assert_eq!(c.payload, None);
        assert!(!c.is_terminal());
    }

    #[test]
    fn test_checkpoint_terminal_states() {
        // Arrange
        let mut c = Checkpoint::new(2, "verify");
        // Act & Assert
        c.status = CheckpointStatus::Succeeded;
        assert!(c.is_terminal());
        c.status = CheckpointStatus::Failed;
        assert!(c.is_terminal());
        c.status = CheckpointStatus::Skipped;
        assert!(!c.is_terminal());
    }
}
