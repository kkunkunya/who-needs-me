//! Local, read-only collection contracts for WhoNeedsMe.

mod adapter;
mod claude;
mod domain;
mod engine;
mod environment;
mod error;

pub use adapter::{Adapter, SessionArtifact};
pub use claude::ClaudeAdapter;
pub use domain::{Provider, Session, SessionKey, SessionMetadata, SessionState, WaitingReason};
pub use engine::Engine;
pub use environment::{Environment, EnvironmentBuilder, ProcessProbe, SystemProcessProbe};
pub use error::{CoreError, CoreResult};
