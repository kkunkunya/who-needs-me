use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use crate::{CoreError, CoreResult, Provider, ProviderDescriptor};

/// The OS boundary used to decide whether a Provider process is still alive.
pub trait ProcessProbe: Send + Sync {
    /// Number of live Provider processes whose working directory matches `cwd`.
    ///
    /// The Engine uses this as a per-directory capacity: N processes may keep at
    /// most N recently active Session candidates on the panel.
    fn live_process_count(&self, provider: ProviderDescriptor, cwd: &Path) -> usize;
}

/// A snapshot of process working directories, isolated for deterministic probing.
pub trait ProcessTable: Send + Sync {
    fn process_cwds(&self, executable: &str) -> Vec<PathBuf>;
}

/// Production probe. V1 is macOS-first, where `pgrep` is available.
pub struct SystemProcessProbe {
    process_table: Arc<dyn ProcessTable>,
}

impl Default for SystemProcessProbe {
    fn default() -> Self {
        Self {
            process_table: Arc::new(MacOsProcessTable),
        }
    }
}

impl SystemProcessProbe {
    pub fn with_process_table(process_table: Arc<dyn ProcessTable>) -> Self {
        Self { process_table }
    }
}

impl ProcessProbe for SystemProcessProbe {
    fn live_process_count(&self, provider: ProviderDescriptor, cwd: &Path) -> usize {
        if cwd.as_os_str().is_empty() {
            return 0;
        }

        self.process_table
            .process_cwds(provider.executable())
            .iter()
            .filter(|process_cwd| paths_match(process_cwd, cwd))
            .count()
    }
}

struct MacOsProcessTable;

impl ProcessTable for MacOsProcessTable {
    fn process_cwds(&self, executable: &str) -> Vec<PathBuf> {
        let Ok(processes) = Command::new("pgrep").args(["-x", executable]).output() else {
            return Vec::new();
        };
        if !processes.status.success() {
            return Vec::new();
        }

        String::from_utf8_lossy(&processes.stdout)
            .lines()
            .filter_map(|pid| process_cwd(pid.trim()))
            .collect()
    }
}

fn process_cwd(pid: &str) -> Option<PathBuf> {
    if pid.is_empty() {
        return None;
    }
    let output = Command::new("lsof")
        .args(["-a", "-p", pid, "-d", "cwd", "-Fn"])
        .output()
        .ok()?;
    output.status.success().then_some(())?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix('n').map(PathBuf::from))
}

fn paths_match(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
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
    pub fn production(providers: impl IntoIterator<Item = ProviderDescriptor>) -> CoreResult<Self> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or(CoreError::MissingHomeDirectory)?;
        let hook_base = home.join(".who-needs-me/hooks");
        let mut builder = Self::builder(Arc::new(SystemProcessProbe::default()));
        for descriptor in providers {
            let provider = descriptor.provider();
            builder = builder
                .with_session_data_root(provider, descriptor.session_data_root(&home))
                .with_hook_event_root(provider, descriptor.hook_event_root(&hook_base));
        }
        builder.build()
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

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use super::{ProcessProbe, ProcessTable, SystemProcessProbe};
    use crate::{Provider, ProviderDescriptor};

    struct ControlledProcessTable {
        executable: &'static str,
        cwd: PathBuf,
    }

    impl ProcessTable for ControlledProcessTable {
        fn process_cwds(&self, executable: &str) -> Vec<PathBuf> {
            if executable == self.executable {
                vec![self.cwd.clone()]
            } else {
                Vec::new()
            }
        }
    }

    #[test]
    fn production_probe_does_not_keep_a_historical_session_alive() {
        let probe = SystemProcessProbe::with_process_table(Arc::new(ControlledProcessTable {
            executable: "claude",
            cwd: PathBuf::from("/work/live-session"),
        }));
        let provider = ProviderDescriptor::new(
            Provider::new("claude", "Claude"),
            "claude",
            ".claude/projects",
            "claude",
        );

        assert_eq!(
            probe.live_process_count(provider, PathBuf::from("/work/live-session").as_path()),
            1
        );
        assert_eq!(
            probe.live_process_count(
                provider,
                PathBuf::from("/work/historical-session").as_path()
            ),
            0
        );
    }
}
