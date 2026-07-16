use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{Adapter, CoreError, CoreResult, Environment, Session, SessionKey};

pub struct Engine {
    adapters: Vec<Box<dyn Adapter>>,
}

impl Engine {
    pub fn new(adapters: Vec<Box<dyn Adapter>>) -> Self {
        Self { adapters }
    }

    pub fn collect_sessions(&self, environment: &Environment) -> CoreResult<Vec<Session>> {
        let mut candidate_groups = BTreeMap::<
            (crate::Provider, PathBuf),
            (crate::ProviderDescriptor, Vec<Candidate>),
        >::new();

        for adapter in &self.adapters {
            let descriptor = adapter.descriptor();
            let root = environment.session_data_root(descriptor.provider())?;
            for artifact in adapter.discover(root)? {
                let activity_at = fs::metadata(artifact.path())
                    .and_then(|metadata| metadata.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let session = adapter.parse(&artifact)?;
                let cwd = session
                    .metadata
                    .cwd
                    .as_deref()
                    .map(Path::new)
                    .unwrap_or_else(|| Path::new(""));
                let normalized_cwd = normalize_path(cwd);
                candidate_groups
                    .entry((descriptor.provider(), normalized_cwd))
                    .or_insert_with(|| (descriptor, Vec::new()))
                    .1
                    .push(Candidate {
                        session,
                        activity_at,
                    });
            }
        }

        let mut sessions = BTreeMap::<SessionKey, Session>::new();
        for ((_, cwd), (descriptor, mut candidates)) in candidate_groups {
            let live_count = environment
                .process_probe()
                .live_process_count(descriptor, &cwd);
            candidates.sort_by(|left, right| {
                right
                    .activity_at
                    .cmp(&left.activity_at)
                    .then_with(|| right.session.session_id.cmp(&left.session.session_id))
            });

            for candidate in candidates.into_iter().take(live_count) {
                let session = candidate.session;
                let key = session.key();
                if sessions.insert(key.clone(), session).is_some() {
                    return Err(CoreError::DuplicateSession(format!(
                        "{:?}/{}",
                        key.provider, key.session_id
                    )));
                }
            }
        }

        Ok(sessions.into_values().collect())
    }
}

struct Candidate {
    session: Session,
    activity_at: SystemTime,
}

fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
