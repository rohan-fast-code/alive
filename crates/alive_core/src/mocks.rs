//! In memory mock system used by the core tests and by the guard tests.
//! It records every call in order, holds fake power values, and can be told
//! to fail at exactly one named step so rollback can be tested (TC04).

use crate::models::PowerValues;
use crate::traits::*;
use crate::{CoreError, CoreResult};

pub struct MockSystem {
    pub log: Vec<String>,
    pub values: PowerValues,
    pub original: PowerValues,
    pub fail_at: Option<String>,
    hold: bool,
    guard: bool,
    subscribed: bool,
    agents_running: bool,
    pub working_agents: usize,
    dirty: bool,
}

impl MockSystem {
    pub fn new(original: PowerValues) -> Self {
        Self {
            log: Vec::new(),
            values: original,
            original,
            fail_at: None,
            hold: false,
            guard: false,
            subscribed: false,
            agents_running: false,
            working_agents: 0,
            dirty: false,
        }
    }

    pub fn failing(original: PowerValues, step: &str) -> Self {
        let mut m = Self::new(original);
        m.fail_at = Some(step.to_string());
        m
    }

    fn step(&mut self, key: &str, label: &str) -> CoreResult<()> {
        self.log.push(label.to_string());
        if self.fail_at.as_deref() == Some(key) {
            // Fail only once, so the rollback steps that reuse a step key
            // (for example restore) are still allowed to run.
            self.fail_at = None;
            return Err(CoreError::Injected(key.to_string()));
        }
        Ok(())
    }

    pub fn agents_running(&self) -> bool {
        self.agents_running
    }
    pub fn subscribed(&self) -> bool {
        self.subscribed
    }
}

impl PowerSettings for MockSystem {
    fn read_values(&mut self) -> CoreResult<PowerValues> {
        self.step("snapshot", "snapshot")?;
        Ok(self.values)
    }
    fn apply_lid_alive(&mut self) -> CoreResult<()> {
        self.step("lid", "lid")?;
        let alive = PowerValues::alive_values();
        self.values.lid_ac = alive.lid_ac;
        self.values.lid_dc = alive.lid_dc;
        Ok(())
    }
    fn apply_timeouts_alive(&mut self) -> CoreResult<()> {
        self.step("timeouts", "timeouts")?;
        let alive = PowerValues::alive_values();
        self.values.standby_ac_secs = alive.standby_ac_secs;
        self.values.standby_dc_secs = alive.standby_dc_secs;
        self.values.hibernate_ac_secs = alive.hibernate_ac_secs;
        self.values.hibernate_dc_secs = alive.hibernate_dc_secs;
        self.values.unattended_ac_secs = alive.unattended_ac_secs;
        self.values.unattended_dc_secs = alive.unattended_dc_secs;
        Ok(())
    }
    fn restore_all(&mut self, values: &PowerValues) -> CoreResult<()> {
        self.log.push("restore".to_string());
        self.values = *values;
        Ok(())
    }
}

impl KeepAwake for MockSystem {
    fn acquire(&mut self) -> CoreResult<()> {
        self.step("hold", "hold")?;
        self.hold = true;
        Ok(())
    }
    fn release(&mut self) -> CoreResult<()> {
        self.log.push("release_hold".to_string());
        self.hold = false;
        Ok(())
    }
    fn is_held(&self) -> bool {
        self.hold
    }
}

impl GuardControl for MockSystem {
    fn start_guard(&mut self) -> CoreResult<()> {
        self.step("guard", "guard")?;
        self.guard = true;
        Ok(())
    }
    fn stop_guard(&mut self) -> CoreResult<()> {
        self.log.push("stop_guard".to_string());
        self.guard = false;
        Ok(())
    }
    fn guard_running(&self) -> bool {
        self.guard
    }
}

impl PowerEventSource for MockSystem {
    fn subscribe_power_events(&mut self) -> CoreResult<()> {
        self.step("subscribe", "subscribe")?;
        self.subscribed = true;
        Ok(())
    }
    fn unsubscribe_power_events(&mut self) -> CoreResult<()> {
        self.log.push("unsubscribe".to_string());
        self.subscribed = false;
        Ok(())
    }
}

impl AgentRunner for MockSystem {
    fn start_enabled_agents(&mut self) -> CoreResult<()> {
        self.step("agents", "agents")?;
        self.agents_running = true;
        Ok(())
    }
    fn stop_all_agents(&mut self) -> CoreResult<()> {
        self.log.push("stop_agents".to_string());
        self.agents_running = false;
        self.working_agents = 0;
        Ok(())
    }
    fn working_count(&self) -> usize {
        self.working_agents
    }
    fn running_count(&self) -> usize {
        if self.agents_running {
            1
        } else {
            0
        }
    }
}

impl DirtyMarker for MockSystem {
    fn mark_dirty(&mut self) -> CoreResult<()> {
        self.dirty = true;
        Ok(())
    }
    fn clear_dirty_marker(&mut self) -> CoreResult<()> {
        self.log.push("clear_dirty".to_string());
        self.dirty = false;
        Ok(())
    }
    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl AliveSystem for MockSystem {}
