use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Strict,
    Smart,
    PluggedInOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerSource {
    Ac,
    Battery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thresholds {
    pub pause_heavy_percent: u8,
    pub critical_percent: u8,
    pub hysteresis_percent: u8,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            pause_heavy_percent: 30,
            critical_percent: 15,
            hysteresis_percent: 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    AllowAll,
    PauseHeavy,
    CriticalOff,
    WaitingForPower,
}

/// Stateful policy engine. The heavy paused latch plus the hysteresis band is
/// what stops thresholds from flapping when the battery reading oscillates.
#[derive(Debug, Clone)]
pub struct PolicyEngine {
    pub thresholds: Thresholds,
    heavy_paused: bool,
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new(Thresholds::default())
    }
}

impl PolicyEngine {
    pub fn new(thresholds: Thresholds) -> Self {
        Self {
            thresholds,
            heavy_paused: false,
        }
    }

    pub fn heavy_paused(&self) -> bool {
        self.heavy_paused
    }

    pub fn evaluate(
        &mut self,
        mode: Mode,
        source: PowerSource,
        battery_percent: Option<u8>,
        battery_saver: bool,
    ) -> PolicyDecision {
        // Plugged in only mode: nothing is applied on battery at all.
        if mode == Mode::PluggedInOnly && source == PowerSource::Battery {
            return PolicyDecision::WaitingForPower;
        }
        // On AC every agent is allowed, at any battery level (TC05).
        if source == PowerSource::Ac {
            self.heavy_paused = false;
            return PolicyDecision::AllowAll;
        }
        // On battery from here.
        if let Some(p) = battery_percent {
            // Critical cutoff applies in Strict and in Smart (TC08).
            if p < self.thresholds.critical_percent {
                return PolicyDecision::CriticalOff;
            }
        }
        if mode == Mode::Strict {
            // Strict keeps everything alive subject only to the critical cutoff,
            // except that battery saver still pauses heavy agents is a Smart
            // behaviour in the spec, so Strict does not pause here.
            self.heavy_paused = false;
            return PolicyDecision::AllowAll;
        }
        // Smart mode on battery.
        if battery_saver {
            self.heavy_paused = true;
            return PolicyDecision::PauseHeavy;
        }
        if let Some(p) = battery_percent {
            if self.heavy_paused {
                let release_at = self
                    .thresholds
                    .pause_heavy_percent
                    .saturating_add(self.thresholds.hysteresis_percent);
                if p >= release_at {
                    self.heavy_paused = false;
                    return PolicyDecision::AllowAll;
                }
                return PolicyDecision::PauseHeavy;
            }
            if p < self.thresholds.pause_heavy_percent {
                self.heavy_paused = true;
                return PolicyDecision::PauseHeavy;
            }
        }
        PolicyDecision::AllowAll
    }
}
