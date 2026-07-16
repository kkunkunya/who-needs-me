//! Local, read-only collection contracts for WhoNeedsMe.

mod adapter;
mod claude;
mod domain;
mod engine;
mod environment;
mod error;

pub use adapter::{Adapter, ProviderDescriptor, SessionArtifact};
pub use claude::ClaudeAdapter;
pub use domain::{
    PanelSession, PanelSnapshot, Provider, Session, SessionKey, SessionMetadata, SessionState,
    WaitingReason,
};
pub use engine::Engine;
pub use environment::{
    Environment, EnvironmentBuilder, ProcessProbe, ProcessTable, SystemProcessProbe,
};
pub use error::{CoreError, CoreResult};
