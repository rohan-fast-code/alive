//! alive_core: pure logic for Alive. This crate makes no Windows calls, so the
//! whole state machine, policy engine, snapshot logic, config validation and
//! restart backoff can be unit tested on any operating system with mocks.

pub mod backoff;
pub mod config;
pub mod mocks;
pub mod models;
pub mod orchestrator;
pub mod policy;
pub mod snapshot;
pub mod state;
pub mod traits;

pub use backoff::{Backoff, RestartTracker};
pub use config::{AgentConfig, AgentKind, AliveConfig, HealthKind, RestartPolicy};
pub use models::{PowerValues, Snapshot};
pub use orchestrator::Orchestrator;
pub use policy::{Mode, PolicyDecision, PolicyEngine, PowerSource, Thresholds};
pub use state::{arming_commands, restoring_commands, undo_commands, Command, State};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("step failed: {step}: {message}")]
    StepFailed { step: String, message: String },
    #[error("unsupported on this platform: {0}")]
    Unsupported(String),
    #[error("snapshot is corrupt or has an unsupported version")]
    CorruptSnapshot,
    #[error("io error: {0}")]
    Io(String),
    #[error("invalid state for this operation: {0}")]
    InvalidState(String),
    #[error("injected failure at step: {0}")]
    Injected(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
