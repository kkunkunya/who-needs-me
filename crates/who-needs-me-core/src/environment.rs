use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use crate::{CoreError, CoreResult, Provider};

/// The OS boundary used to decide whether a Provider process is still alive.
pub trait ProcessProbe: Send + Sync {
    fn is_alive(&self, provider: Provider, cwd: &Path) -> bool;
}

/// Production probe. V1 is macOS-first, where `pgrep` is available.
#[derive(Debug, Default)]
pub struct SystemProcessProbe;

impl ProcessProbe for SystemProcessProbe {
    fn is_alive(&self, provider: Provider, _cwd: &Path) -> bool {
        Command::new("pgrep")
            .args(["-x", provider.process_name()])
            .status()
            .is_ok_and(|status| status.success())
    }
}

/// Every external input consumed by the collection engine.
pub struct Environment {
    session_data_roots: BTreeMap<Provider, PathBuf>,
    hook_event_roots: BTreeMap<Provider, PathBuf>,
    process_probe: Arc<dyn ProcessProbe>,
}

impl Environment {
    pub fn builder(process_probe: Arc<dyn ProcessProbe>) -> EnvironmentBuilder {
        EnvironmentBuilder {
            session_data_roots: BTreeMap::new(),
            hook_event_roots: BTreeMap::new(),
            process_probe,
        }
    }

    /// Real local paths and a real process probe used when no fixture is injected.
    pub fn production() -> CoreResult<Self> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or(CoreError::MissingHomeDirectory)?;
        let hook_base = home.join(".who-needs-me/hooks");

        Self::builder(Arc::new(SystemProcessProbe))
            .with_session_data_root(Provider::Claude, home.join(".claude/projects"))
            .with_hook_event_root(
                Provider::Claude,
                hook_base.join(Provider::Claude.directory_name()),
            )
            .with_session_data_root(Provider::Codex, home.join(".codex/sessions"))
            .with_hook_event_root(
                Provider::Codex,
                hook_base.join(Provider::Codex.directory_name()),
            )
            .build()
    }

    pub fn session_data_root(&self, provider: Provider) -> CoreResult<&Path> {
        self.session_data_roots
            .get(&provider)
            .map(PathBuf::as_path)
            .ok_or_else(|| CoreError::MissingSessionDataRoot(format!("{provider:?}")))
    }

    pub fn hook_event_root(&self, provider: Provider) -> CoreResult<&Path> {
        self.hook_event_roots
            .get(&provider)
            .map(PathBuf::as_path)
            .ok_or_else(|| CoreError::MissingHookEventRoot(format!("{provider:?}")))
    }

    pub fn process_probe(&self) -> &dyn ProcessProbe {
        self.process_probe.as_ref()
    }
}

pub struct EnvironmentBuilder {
    session_data_roots: BTreeMap<Provider, PathBuf>,
    hook_event_roots: BTreeMap<Provider, PathBuf>,
    process_probe: Arc<dyn ProcessProbe>,
}

impl EnvironmentBuilder {
    pub fn with_session_data_root(mut self, provider: Provider, root: impl Into<PathBuf>) -> Self {
        self.session_data_roots.insert(provider, root.into());
        self
    }

    pub fn with_hook_event_root(mut self, provider: Provider, root: impl Into<PathBuf>) -> Self {
        self.hook_event_roots.insert(provider, root.into());
        self
    }

    pub fn build(self) -> CoreResult<Environment> {
        if self
            .session_data_roots
            .keys()
            .ne(self.hook_event_roots.keys())
        {
            return Err(CoreError::RootSetsDoNotMatch);
        }

        Ok(Environment {
            session_data_roots: self.session_data_roots,
            hook_event_roots: self.hook_event_roots,
            process_probe: self.process_probe,
        })
    }
}
