//! Collapsed VM lifecycle (architecture diagram 30-state-vm).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MachineState {
    PoweredOff,
    Starting,
    Running,
    Paused,
    Saving,
    Saved,
    Restoring,
    Stopping,
    Snapshotting,
    RestoringSnapshot,
    Aborted,
}

/// User-initiated actions that drive the lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Start,
    Stop,
    Pause,
    Resume,
    Delete,
}

impl MachineState {
    /// A state in which an engine operation is in flight; no new action may start.
    pub fn is_transient(self) -> bool {
        matches!(
            self,
            Self::Starting
                | Self::Saving
                | Self::Restoring
                | Self::Stopping
                | Self::Snapshotting
                | Self::RestoringSnapshot
        )
    }

    /// Whether the guest is executing or holds live state (configuration edits may need power-off).
    pub fn is_live(self) -> bool {
        matches!(self, Self::Running | Self::Paused) || self.is_transient()
    }

    /// The state the machine is in once `action` completes, used to detect no-op requests.
    pub fn target_of(action: Action) -> Option<MachineState> {
        match action {
            Action::Start | Action::Resume => Some(Self::Running),
            Action::Stop => Some(Self::PoweredOff),
            Action::Pause => Some(Self::Paused),
            Action::Delete => None,
        }
    }

    /// Whether `action` is allowed from this state.
    pub fn allows(self, action: Action) -> bool {
        match action {
            Action::Start => matches!(self, Self::PoweredOff | Self::Saved | Self::Aborted),
            Action::Stop => matches!(self, Self::Running | Self::Paused | Self::Aborted),
            Action::Pause => self == Self::Running,
            Action::Resume => self == Self::Paused,
            Action::Delete => matches!(self, Self::PoweredOff | Self::Saved | Self::Aborted),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PoweredOff => "powered_off",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Saving => "saving",
            Self::Saved => "saved",
            Self::Restoring => "restoring",
            Self::Stopping => "stopping",
            Self::Snapshotting => "snapshotting",
            Self::RestoringSnapshot => "restoring_snapshot",
            Self::Aborted => "aborted",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_rules() {
        assert!(MachineState::PoweredOff.allows(Action::Start));
        assert!(!MachineState::Running.allows(Action::Start));
        assert!(MachineState::Running.allows(Action::Stop));
        assert!(!MachineState::Starting.allows(Action::Stop));
        assert!(!MachineState::Running.allows(Action::Delete));
        assert!(MachineState::Aborted.allows(Action::Start));
        assert!(MachineState::Starting.is_live());
        assert!(!MachineState::Saved.is_live());
    }
}
