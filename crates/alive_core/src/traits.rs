//! Traits the Windows layer implements and tests mock (spec section 5).
use crate::models::PowerValues;
use crate::CoreResult;

pub trait PowerSettings {
    fn read_values(&mut self) -> CoreResult<PowerValues>;
    fn apply_lid_alive(&mut self) -> CoreResult<()>;
    fn apply_timeouts_alive(&mut self) -> CoreResult<()>;
    fn restore_all(&mut self, values: &PowerValues) -> CoreResult<()>;
}

pub trait KeepAwake {
    fn acquire(&mut self) -> CoreResult<()>;
    fn release(&mut self) -> CoreResult<()>;
    fn is_held(&self) -> bool;
}

pub trait GuardControl {
    fn start_guard(&mut self) -> CoreResult<()>;
    fn stop_guard(&mut self) -> CoreResult<()>;
    fn guard_running(&self) -> bool;
}

pub trait PowerEventSource {
    fn subscribe_power_events(&mut self) -> CoreResult<()>;
    fn unsubscribe_power_events(&mut self) -> CoreResult<()>;
}

pub trait AgentRunner {
    fn start_enabled_agents(&mut self) -> CoreResult<()>;
    fn stop_all_agents(&mut self) -> CoreResult<()>;
    fn working_count(&self) -> usize;
    fn running_count(&self) -> usize;
}

pub trait DirtyMarker {
    fn mark_dirty(&mut self) -> CoreResult<()>;
    fn clear_dirty_marker(&mut self) -> CoreResult<()>;
    fn is_dirty(&self) -> bool;
}

pub trait Clock {
    fn now_seconds(&self) -> u64;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_seconds(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

/// One system that can do everything the orchestrator needs. Real code
/// assembles this from the Windows layer; tests use one mock object, which
/// keeps the step order and any rollback in a single shared call log.
pub trait AliveSystem:
    PowerSettings + KeepAwake + GuardControl + PowerEventSource + AgentRunner + DirtyMarker
{
}
