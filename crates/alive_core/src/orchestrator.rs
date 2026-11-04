//! The orchestrator runs the state machine steps against a real or mock
//! system, in the exact order the spec mandates, and rolls back in reverse
//! order if any Arming step fails.

use crate::models::{PowerValues, Snapshot};
use crate::state::{arming_commands, undo_commands, Command, State};
use crate::traits::*;
use crate::{CoreError, CoreResult};

pub struct Orchestrator {
    pub state: State,
    pub snapshot: Option<Snapshot>,
    pub modern_standby: bool,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            state: State::Off,
            snapshot: None,
            modern_standby: false,
        }
    }

    pub fn toggle_on<S: AliveSystem>(&mut self, sys: &mut S) -> CoreResult<()> {
        if self.state != State::Off {
            return Err(CoreError::InvalidState(format!(
                "toggle ON from {:?}",
                self.state
            )));
        }
        self.state = State::Arming;
        let mut completed: Vec<Command> = Vec::new();
        let mut snapshot_values: Option<PowerValues> = None;

        for command in arming_commands() {
            let result = match command {
                Command::TakeSnapshot => sys.read_values().map(|v| {
                    snapshot_values = Some(v);
                }),
                Command::StartGuard => sys.start_guard(),
                Command::ApplyLidAction => sys.apply_lid_alive(),
                Command::ApplyTimeouts => sys.apply_timeouts_alive(),
                Command::AcquireHold => sys.acquire(),
                Command::SubscribePowerEvents => sys.subscribe_power_events(),
                Command::StartAgents => sys.start_enabled_agents(),
                _ => Ok(()),
            };
            match result {
                Ok(()) => {
                    completed.push(command);
                    if command == Command::TakeSnapshot {
                        if let Some(v) = snapshot_values {
                            self.snapshot = Some(Snapshot::new(v, self.modern_standby));
                        }
                        // The dirty marker is part of taking the snapshot:
                        // it is written before any setting changes (5.2).
                        let _ = sys.mark_dirty();
                    }
                }
                Err(e) => {
                    // Undo the completed steps in reverse order, best effort.
                    for undo in undo_commands(&completed) {
                        let _ = self.run_undo(undo, sys, snapshot_values);
                    }
                    self.state = State::Off;
                    self.snapshot = None;
                    return Err(e);
                }
            }
        }
        self.state = if sys.working_count() > 0 {
            State::OnActive
        } else {
            State::OnIdle
        };
        Ok(())
    }

    fn run_undo<S: AliveSystem>(
        &self,
        undo: Command,
        sys: &mut S,
        snapshot_values: Option<PowerValues>,
    ) -> CoreResult<()> {
        match undo {
            Command::ClearDirtyMarker => sys.clear_dirty_marker(),
            Command::StopGuard => sys.stop_guard(),
            Command::RestoreSettings => {
                let v = snapshot_values.unwrap_or_else(PowerValues::windows_fallback_defaults);
                sys.restore_all(&v)
            }
            Command::ReleaseHold => sys.release(),
            Command::UnsubscribePowerEvents => sys.unsubscribe_power_events(),
            Command::StopAgents => sys.stop_all_agents(),
            _ => Ok(()),
        }
    }

    /// Toggle OFF. Idempotent: calling it while already OFF, or calling it
    /// twice, is harmless and changes nothing (spec section 5.1).
    pub fn toggle_off<S: AliveSystem>(&mut self, sys: &mut S) -> CoreResult<()> {
        if self.state == State::Off {
            return Ok(());
        }
        if !self.state.is_on() && self.state != State::Faulted {
            return Err(CoreError::InvalidState(format!(
                "toggle OFF from {:?}",
                self.state
            )));
        }
        self.state = State::Restoring;
        let snap_values = self
            .snapshot
            .as_ref()
            .map(|s| s.values)
            .unwrap_or_else(PowerValues::windows_fallback_defaults);

        // Restoring steps in spec order. If one fails the machine goes to
        // Faulted and the caller can retry; the steps themselves are safe
        // to run again.
        let steps: Vec<CoreResult<()>> = vec![
            sys.stop_all_agents(),
            sys.release(),
            sys.restore_all(&snap_values),
            sys.clear_dirty_marker(),
            sys.stop_guard(),
        ];
        // Note: the vec above already executed the steps in order when it
        // was built. Check them in the same order.
        for step in steps {
            if let Err(e) = step {
                self.state = State::Faulted;
                return Err(e);
            }
        }
        let _ = sys.unsubscribe_power_events();
        self.state = State::Off;
        self.snapshot = None;
        Ok(())
    }

    pub fn set_throttled(&mut self, throttled: bool) {
        if throttled && self.state.is_on() {
            self.state = State::OnThrottled;
        } else if !throttled && self.state == State::OnThrottled {
            self.state = State::OnIdle;
        }
    }
}
