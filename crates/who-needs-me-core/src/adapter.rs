use std::path::{Path, PathBuf};

use crate::{CoreResult, Provider, Session};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionArtifact {
    path: PathBuf,
}

impl SessionArtifact {
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Provider-specific discovery and parsing, isolated from the core engine.
pub trait Adapter: Send + Sync {
    fn provider(&self) -> Provider;

    fn discover(&self, session_data_root: &Path) -> CoreResult<Vec<SessionArtifact>>;

    fn parse(&self, artifact: &SessionArtifact) -> CoreResult<Session>;
}
