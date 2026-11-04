use crate::policy::{Mode, Thresholds};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentKind {
    Cloud,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartPolicy {
    Never,
    OnFailure,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthKind {
    None,
    ProcessAlive,
    HttpUrl,
    HeartbeatFile,
}

/// One user defined agent process (spec section 6.3). Command plus args is
/// always an explicit argument list, never a shell string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub working_dir: String,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub kind: AgentKind,
    #[serde(default)]
    pub heavy: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub autostart: bool,
    pub restart_policy: RestartPolicy,
    pub max_restarts: i64,
    pub backoff_start_seconds: i64,
    pub backoff_max_seconds: i64,
    pub stable_after_seconds: i64,
    pub health: HealthKind,
    #[serde(default)]
    pub health_target: String,
    #[serde(default)]
    pub health_interval_seconds: i64,
}

fn default_true() -> bool {
    true
}

pub const MAX_RESTARTS_LIMIT: i64 = 100;
pub const MAX_ARGS: usize = 100;
pub const MAX_ENV: usize = 100;
pub const MAX_NAME_LEN: usize = 200;

impl AgentConfig {
    /// Validate one agent. Returns a list of human readable problems.
    /// `check_path` controls whether the executable path must exist on this
    /// machine, which tests switch on explicitly instead of depending on
    /// whatever happens to be installed.
    pub fn validate(&self, check_path: bool) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.trim().is_empty() {
            errors.push("agent name is missing".to_string());
        }
        if self.name.len() > MAX_NAME_LEN {
            errors.push("agent name is too long".to_string());
        }
        if self.command.trim().is_empty() {
            errors.push(format!("agent {}: command is missing", self.name));
        } else if check_path && !Path::new(&self.command).exists() {
            errors.push(format!(
                "agent {}: command path does not exist: {}",
                self.name, self.command
            ));
        }
        if self.args.len() > MAX_ARGS {
            errors.push(format!("agent {}: too many arguments", self.name));
        }
        if self.env.len() > MAX_ENV {
            errors.push(format!("agent {}: too many environment values", self.name));
        }
        if self.max_restarts < 0 {
            errors.push(format!("agent {}: max_restarts is negative", self.name));
        }
        if self.max_restarts > MAX_RESTARTS_LIMIT {
            errors.push(format!("agent {}: max_restarts is too large", self.name));
        }
        if self.backoff_start_seconds < 0
            || self.backoff_max_seconds < 0
            || self.stable_after_seconds < 0
        {
            errors.push(format!("agent {}: a time value is negative", self.name));
        }
        if self.backoff_max_seconds < self.backoff_start_seconds {
            errors.push(format!(
                "agent {}: backoff max is smaller than backoff start",
                self.name
            ));
        }
        if self.health_interval_seconds < 0 {
            errors.push(format!("agent {}: health interval is negative", self.name));
        }
        if self.health == HealthKind::HttpUrl && self.health_target.trim().is_empty() {
            errors.push(format!("agent {}: health URL is missing", self.name));
        }
        if self.health == HealthKind::HeartbeatFile && self.health_target.trim().is_empty() {
            errors.push(format!(
                "agent {}: heartbeat file path is missing",
                self.name
            ));
        }
        errors
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliveConfig {
    #[serde(default)]
    pub mode: Mode,
    #[serde(default)]
    pub thresholds: Thresholds,
    #[serde(default)]
    pub agents: Vec<AgentConfig>,
    #[serde(default)]
    pub turn_on_at_start: bool,
    #[serde(default)]
    pub start_with_windows: bool,
    #[serde(default = "default_true")]
    pub start_minimized_to_tray: bool,
    #[serde(default = "default_true")]
    pub notifications: bool,
}

impl Default for AliveConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Strict,
            thresholds: Thresholds::default(),
            agents: Vec::new(),
            turn_on_at_start: false,
            start_with_windows: false,
            start_minimized_to_tray: true,
            notifications: true,
        }
    }
}

impl AliveConfig {
    pub fn validate(&self, check_paths: bool) -> Vec<String> {
        let mut errors = Vec::new();
        if self.thresholds.pause_heavy_percent > 100
            || self.thresholds.critical_percent > 100
            || self.thresholds.hysteresis_percent > 50
        {
            errors.push("a threshold value is out of range".to_string());
        }
        if self.thresholds.critical_percent >= self.thresholds.pause_heavy_percent {
            errors.push("critical threshold must be below the pause heavy threshold".to_string());
        }
        let mut names = std::collections::HashSet::new();
        let mut ids = std::collections::HashSet::new();
        for agent in &self.agents {
            if !names.insert(agent.name.clone()) {
                errors.push(format!("duplicate agent name: {}", agent.name));
            }
            if !ids.insert(agent.id.clone()) {
                errors.push(format!("duplicate agent id: {}", agent.id));
            }
            errors.extend(agent.validate(check_paths));
        }
        errors
    }
}

/// Mask a secret value for logs and the UI (spec section 9).
pub fn mask_secret(value: &str) -> String {
    if value.len() <= 4 {
        return "****".to_string();
    }
    format!("****{}", &value[value.len() - 2..])
}

pub fn is_secret_key(key: &str) -> bool {
    let k = key.to_ascii_uppercase();
    k.contains("KEY") || k.contains("TOKEN") || k.contains("SECRET") || k.contains("PASSWORD")
}
