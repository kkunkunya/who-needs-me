use std::path::{Path, PathBuf};

use crate::{CoreResult, Provider, Session};

/// Provider-specific discovery and process details owned by an Adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderDescriptor {
    provider: Provider,
    executable: &'static str,
    session_data_subpath: &'static str,
    hook_event_directory: &'static str,
}

impl ProviderDescriptor {
    pub const fn new(
        provider: Provider,
        executable: &'static str,
        session_data_subpath: &'static str,
        hook_event_directory: &'static str,
    ) -> Self {
        Self {
            provider,
            executable,
            session_data_subpath,
            hook_event_directory,
        }
    }

    pub const fn provider(self) -> Provider {
        self.provider
    }

    pub const fn executable(self) -> &'static str {
        self.executable
    }

    pub fn session_data_root(self, home: &Path) -> PathBuf {
        home.join(self.session_data_subpath)
    }

    pub fn hook_event_root(self, hook_base: &Path) -> PathBuf {
        hook_base.join(self.hook_event_directory)
    }
}

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
    fn descriptor(&self) -> ProviderDescriptor;

    fn discover(&self, session_data_root: &Path) -> CoreResult<Vec<SessionArtifact>>;

    fn parse(&self, artifact: &SessionArtifact) -> CoreResult<Session>;
}
