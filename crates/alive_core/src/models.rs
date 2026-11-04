use serde::{Deserialize, Serialize};

/// Every power value Alive changes, for AC (plugged in) and DC (battery).
/// Lid values: 0 do nothing, 1 sleep, 2 hibernate, 3 shut down.
/// Timeout values are seconds, where 0 means never.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerValues {
    pub lid_ac: u32,
    pub lid_dc: u32,
    pub standby_ac_secs: u32,
    pub standby_dc_secs: u32,
    pub hibernate_ac_secs: u32,
    pub hibernate_dc_secs: u32,
    pub unattended_ac_secs: u32,
    pub unattended_dc_secs: u32,
}

impl PowerValues {
    /// The values Alive applies while ON.
    pub fn alive_values() -> Self {
        Self {
            lid_ac: 0,
            lid_dc: 0,
            standby_ac_secs: 0,
            standby_dc_secs: 0,
            hibernate_ac_secs: 0,
            hibernate_dc_secs: 0,
            unattended_ac_secs: 0,
            unattended_dc_secs: 0,
        }
    }

    /// Fallback used only when a snapshot is missing or corrupt. The user is
    /// told whenever this fallback is used (spec section 5.2).
    pub fn windows_fallback_defaults() -> Self {
        Self {
            lid_ac: 1,
            lid_dc: 1,
            standby_ac_secs: 1800,
            standby_dc_secs: 900,
            hibernate_ac_secs: 0,
            hibernate_dc_secs: 0,
            unattended_ac_secs: 7200,
            unattended_dc_secs: 7200,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.lid_ac <= 3 && self.lid_dc <= 3
    }
}

/// A persisted snapshot of the power values taken before Alive changes anything.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: u32,
    pub values: PowerValues,
    pub modern_standby: bool,
}

impl Snapshot {
    pub const CURRENT_VERSION: u32 = 1;

    pub fn new(values: PowerValues, modern_standby: bool) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            values,
            modern_standby,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.version == Self::CURRENT_VERSION && self.values.is_valid()
    }
}
