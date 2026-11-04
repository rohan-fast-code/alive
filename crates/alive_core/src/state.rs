use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Off,
    Arming,
    OnActive,
    OnIdle,
    OnThrottled,
    Restoring,
    Faulted,
}

impl State {
    pub fn is_on(&self) -> bool {
        matches!(self, State::OnActive | State::OnIdle | State::OnThrottled)
    }
}

/// One ordered unit of work the state machine asks the Windows layer to do.
/// Tests assert on these sequences, so the order here is a contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    TakeSnapshot,
    StartGuard,
    ApplyLidAction,
    ApplyTimeouts,
    AcquireHold,
    SubscribePowerEvents,
    StartAgents,
    StopAgents,
    UnsubscribePowerEvents,
    ReleaseHold,
    RestoreSettings,
    ClearDirtyMarker,
    StopGuard,
}

/// The exact Arming order from spec section 5.1.
pub fn arming_commands() -> Vec<Command> {
    vec![
        Command::TakeSnapshot,
        Command::StartGuard,
        Command::ApplyLidAction,
        Command::ApplyTimeouts,
        Command::AcquireHold,
        Command::SubscribePowerEvents,
        Command::StartAgents,
    ]
}

/// The exact Restoring order from spec section 5.1.
pub fn restoring_commands() -> Vec<Command> {
    vec![
        Command::StopAgents,
        Command::ReleaseHold,
        Command::RestoreSettings,
        Command::ClearDirtyMarker,
        Command::StopGuard,
    ]
}

/// The undo command for one completed Arming step, if it has one.
pub fn undo_command(done: Command) -> Option<Command> {
    match done {
        Command::TakeSnapshot => Some(Command::ClearDirtyMarker),
        Command::StartGuard => Some(Command::StopGuard),
        Command::ApplyLidAction => Some(Command::RestoreSettings),
        Command::ApplyTimeouts => Some(Command::RestoreSettings),
        Command::AcquireHold => Some(Command::ReleaseHold),
        Command::SubscribePowerEvents => Some(Command::UnsubscribePowerEvents),
        Command::StartAgents => Some(Command::StopAgents),
        _ => None,
    }
}

/// Undo a list of completed steps in reverse order (spec section 5.1).
pub fn undo_commands(completed: &[Command]) -> Vec<Command> {
    completed
        .iter()
        .rev()
        .filter_map(|c| undo_command(*c))
        .collect()
}
