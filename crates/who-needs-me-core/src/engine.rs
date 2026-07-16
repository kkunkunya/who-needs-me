use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, SystemTime},
};

use crate::{
    Adapter, CoreError, CoreResult, Environment, PanelSession, PanelSnapshot, Session, SessionKey,
    SessionState,
};

pub struct Engine {
    adapters: Vec<Box<dyn Adapter>>,
    panel_state: Mutex<BTreeMap<SessionKey, TrackedSession>>,
    ended_grace: Duration,
}

impl Engine {
    pub const DEFAULT_ENDED_GRACE: Duration = Duration::from_secs(5);

    pub fn new(adapters: Vec<Box<dyn Adapter>>) -> Self {
        Self::with_ended_grace(adapters, Self::DEFAULT_ENDED_GRACE)
    }

    pub fn with_ended_grace(adapters: Vec<Box<dyn Adapter>>, ended_grace: Duration) -> Self {
        Self {
            adapters,
            panel_state: Mutex::new(BTreeMap::new()),
            ended_grace,
        }
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

    /// Collects the current stateful panel snapshot using the system clock.
    pub fn collect_panel(&self, environment: &Environment) -> CoreResult<PanelSnapshot> {
        self.collect_panel_at(environment, SystemTime::now())
    }

    /// Deterministic panel seam used when the caller controls observation time.
    pub fn collect_panel_at(
        &self,
        environment: &Environment,
        now: SystemTime,
    ) -> CoreResult<PanelSnapshot> {
        let live_sessions = self.collect_sessions(environment)?;
        let live_keys: BTreeSet<_> = live_sessions.iter().map(Session::key).collect();
        let mut panel_state = self
            .panel_state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for session in live_sessions {
            let key = session.key();
            let tracked = panel_state.entry(key).or_insert_with(|| TrackedSession {
                session: session.clone(),
                state_since: now,
            });
            if tracked.session.state != session.state {
                tracked.state_since = now;
            }
            tracked.session = session;
        }

        for (key, tracked) in panel_state.iter_mut() {
            if !live_keys.contains(key) && tracked.session.state != SessionState::Ended {
                tracked.session.state = SessionState::Ended;
                tracked.session.waiting_reason = None;
                tracked.state_since = now;
            }
        }
        panel_state.retain(|key, tracked| {
            live_keys.contains(key)
                || now.duration_since(tracked.state_since).unwrap_or_default() < self.ended_grace
        });

        let mut sessions: Vec<_> = panel_state
            .values()
            .map(|tracked| PanelSession {
                session: tracked.session.clone(),
                elapsed: now.duration_since(tracked.state_since).unwrap_or_default(),
            })
            .collect();
        sessions.sort_by_key(|entry| state_rank(entry.session.state));
        let needs_you = sessions.iter().any(|entry| entry.session.needs_you());
        Ok(PanelSnapshot {
            sessions,
            needs_you,
        })
    }
}

struct TrackedSession {
    session: Session,
    state_since: SystemTime,
}

fn state_rank(state: SessionState) -> u8 {
    match state {
        SessionState::Waiting => 0,
        SessionState::Working => 1,
        SessionState::Idle => 2,
        SessionState::Ended => 3,
    }
}

struct Candidate {
    session: Session,
    activity_at: SystemTime,
}

fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
