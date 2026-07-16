use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use who_needs_me_core::{
    Adapter, CoreResult, Engine, Environment, ProcessProbe, Provider, ProviderDescriptor, Session,
    SessionArtifact, SessionMetadata, SessionState,
};

const PROVIDER: Provider = Provider::new("fixture", "Fixture");
const DESCRIPTOR: ProviderDescriptor =
    ProviderDescriptor::new(PROVIDER, "fixture-agent", ".fixture/sessions", "fixture");

struct ControlledProbe {
    counts: Mutex<BTreeMap<PathBuf, usize>>,
}

impl ControlledProbe {
    fn set_live_count(&self, cwd: &str, count: usize) {
        self.counts
            .lock()
            .expect("controlled probe lock")
            .insert(PathBuf::from(cwd), count);
    }
}

impl ProcessProbe for ControlledProbe {
    fn live_process_count(&self, provider: ProviderDescriptor, cwd: &Path) -> usize {
        if provider != DESCRIPTOR {
            return 0;
        }
        self.counts
            .lock()
            .expect("controlled probe lock")
            .get(cwd)
            .copied()
            .unwrap_or_default()
    }
}

struct FixtureAdapter {
    artifacts: Vec<SessionArtifact>,
    sessions: Arc<Mutex<BTreeMap<PathBuf, Session>>>,
}

impl Adapter for FixtureAdapter {
    fn descriptor(&self) -> ProviderDescriptor {
        DESCRIPTOR
    }

    fn discover(&self, _session_data_root: &Path) -> CoreResult<Vec<SessionArtifact>> {
        Ok(self.artifacts.clone())
    }

    fn parse(&self, artifact: &SessionArtifact) -> CoreResult<Session> {
        Ok(self.sessions.lock().expect("fixture sessions lock")[artifact.path()].clone())
    }
}

struct Fixture {
    root: PathBuf,
    sessions: Arc<Mutex<BTreeMap<PathBuf, Session>>>,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "who-needs-me-panel-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&root).expect("fixture root should be created");
        Self {
            root,
            sessions: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    fn add_session(
        &mut self,
        session_id: &str,
        cwd: &str,
        state: SessionState,
        activity_at: SystemTime,
    ) {
        let path = self.root.join(format!("{session_id}.jsonl"));
        std::fs::write(&path, "{}\n").expect("fixture artifact should be written");
        std::fs::File::options()
            .write(true)
            .open(&path)
            .expect("fixture artifact should reopen")
            .set_times(std::fs::FileTimes::new().set_modified(activity_at))
            .expect("fixture mtime should be controlled");
        self.sessions.lock().expect("fixture sessions lock").insert(
            path,
            Session {
                provider: PROVIDER,
                session_id: session_id.into(),
                state,
                state_entered_at: None,
                waiting_reason: None,
                metadata: SessionMetadata {
                    cwd: Some(cwd.into()),
                    cwd_display: None,
                    git_branch: None,
                    model: None,
                    context_tokens: None,
                    context_usage_percent: None,
                },
            },
        );
    }

    fn set_state_entered_at(&self, session_id: &str, state_entered_at: SystemTime) {
        self.sessions
            .lock()
            .expect("fixture sessions lock")
            .values_mut()
            .find(|session| session.session_id == session_id)
            .expect("fixture session should exist")
            .state_entered_at = Some(state_entered_at);
    }

    fn engine(&self) -> Engine {
        self.engine_with_ended_grace(Duration::from_secs(5))
    }

    fn engine_with_ended_grace(&self, ended_grace: Duration) -> Engine {
        let artifacts = self
            .sessions
            .lock()
            .expect("fixture sessions lock")
            .keys()
            .cloned()
            .map(SessionArtifact::from_path)
            .collect();
        Engine::with_ended_grace(
            vec![Box::new(FixtureAdapter {
                artifacts,
                sessions: Arc::clone(&self.sessions),
            })],
            ended_grace,
        )
    }

    fn set_state(&self, session_id: &str, state: SessionState) {
        self.sessions
            .lock()
            .expect("fixture sessions lock")
            .values_mut()
            .find(|session| session.session_id == session_id)
            .expect("fixture session should exist")
            .state = state;
    }

    fn environment(&self, counts: impl IntoIterator<Item = (&'static str, usize)>) -> Environment {
        self.environment_with_probe(counts).0
    }

    fn environment_with_probe(
        &self,
        counts: impl IntoIterator<Item = (&'static str, usize)>,
    ) -> (Environment, Arc<ControlledProbe>) {
        let probe = Arc::new(ControlledProbe {
            counts: Mutex::new(
                counts
                    .into_iter()
                    .map(|(cwd, count)| (PathBuf::from(cwd), count))
                    .collect(),
            ),
        });
        let environment = Environment::builder(probe.clone())
            .with_session_data_root(PROVIDER, &self.root)
            .with_hook_event_root(PROVIDER, self.root.join("hooks"))
            .build()
            .expect("fixture environment should be valid");
        (environment, probe)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn a_live_waiting_session_stays_on_panel_despite_an_old_mtime() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
    let mut fixture = Fixture::new();
    fixture.add_session(
        "waiting-a-long-time",
        "/work/alpha",
        SessionState::Waiting,
        SystemTime::UNIX_EPOCH + Duration::from_secs(10),
    );
    let engine = fixture.engine();
    let environment = fixture.environment([("/work/alpha", 1)]);

    let panel = engine
        .collect_panel_at(&environment, now)
        .expect("panel should be collected");

    assert_eq!(panel.sessions.len(), 1);
    assert_eq!(panel.sessions[0].session.session_id, "waiting-a-long-time");
}

#[test]
fn a_direct_waiting_state_turns_on_needs_you() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
    let mut fixture = Fixture::new();
    fixture.add_session("working", "/work/working", SessionState::Working, now);
    fixture.add_session("waiting", "/work/waiting", SessionState::Waiting, now);
    let engine = fixture.engine();
    let environment = fixture.environment([("/work/working", 1), ("/work/waiting", 1)]);

    let panel = engine
        .collect_panel_at(&environment, now)
        .expect("panel should be collected");

    assert!(panel.needs_you);
}

#[test]
fn panel_orders_waiting_before_working_before_idle() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
    let mut fixture = Fixture::new();
    fixture.add_session("idle", "/work/idle", SessionState::Idle, now);
    fixture.add_session("waiting", "/work/waiting", SessionState::Waiting, now);
    fixture.add_session("working", "/work/working", SessionState::Working, now);
    let engine = fixture.engine();
    let environment = fixture.environment([
        ("/work/idle", 1),
        ("/work/waiting", 1),
        ("/work/working", 1),
    ]);

    let panel = engine
        .collect_panel_at(&environment, now)
        .expect("panel should be collected");

    let states: Vec<_> = panel
        .sessions
        .iter()
        .map(|entry| entry.session.state)
        .collect();
    assert_eq!(
        states,
        vec![
            SessionState::Waiting,
            SessionState::Working,
            SessionState::Idle
        ]
    );
}

#[test]
fn elapsed_tracks_time_since_the_current_state_transition() {
    let started_at = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
    let mut fixture = Fixture::new();
    fixture.add_session(
        "changing-state",
        "/work/changing",
        SessionState::Working,
        started_at,
    );
    let engine = fixture.engine();
    let environment = fixture.environment([("/work/changing", 1)]);

    engine
        .collect_panel_at(&environment, started_at)
        .expect("initial panel should be collected");
    let working_panel = engine
        .collect_panel_at(&environment, started_at + Duration::from_secs(7))
        .expect("working panel should be collected");
    assert_eq!(working_panel.sessions[0].elapsed, Duration::from_secs(7));

    fixture.set_state("changing-state", SessionState::Waiting);
    let transitioned_panel = engine
        .collect_panel_at(&environment, started_at + Duration::from_secs(10))
        .expect("transitioned panel should be collected");
    assert_eq!(transitioned_panel.sessions[0].elapsed, Duration::ZERO);

    let waiting_panel = engine
        .collect_panel_at(&environment, started_at + Duration::from_secs(14))
        .expect("waiting panel should be collected");
    assert_eq!(waiting_panel.sessions[0].elapsed, Duration::from_secs(4));
}

#[test]
fn cold_start_and_restart_preserve_elapsed_from_the_direct_state_timestamp() {
    let observed_at = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
    let waiting_since = observed_at - Duration::from_secs(30 * 60);
    let mut fixture = Fixture::new();
    fixture.add_session(
        "already-waiting",
        "/work/waiting",
        SessionState::Waiting,
        observed_at,
    );
    fixture.set_state_entered_at("already-waiting", waiting_since);
    let environment = fixture.environment([("/work/waiting", 1)]);

    let cold_start = fixture
        .engine()
        .collect_panel_at(&environment, observed_at)
        .expect("cold-start panel should be collected");
    let restarted = fixture
        .engine()
        .collect_panel_at(&environment, observed_at)
        .expect("restarted panel should be collected");

    assert_eq!(cold_start.sessions[0].elapsed, Duration::from_secs(30 * 60));
    assert_eq!(restarted.sessions[0].elapsed, Duration::from_secs(30 * 60));
}

#[test]
fn a_dead_session_is_recently_ended_then_removed_after_grace() {
    let started_at = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
    let mut fixture = Fixture::new();
    fixture.add_session("will-end", "/work/ending", SessionState::Idle, started_at);
    let engine = fixture.engine_with_ended_grace(Duration::from_secs(5));
    let (environment, probe) = fixture.environment_with_probe([("/work/ending", 1)]);

    engine
        .collect_panel_at(&environment, started_at)
        .expect("live panel should be collected");
    probe.set_live_count("/work/ending", 0);

    let recently_ended = engine
        .collect_panel_at(&environment, started_at + Duration::from_secs(1))
        .expect("recently ended panel should be collected");
    assert_eq!(recently_ended.sessions.len(), 1);
    assert_eq!(
        recently_ended.sessions[0].session.state,
        SessionState::Ended
    );
    assert_eq!(recently_ended.sessions[0].elapsed, Duration::ZERO);
    assert!(!recently_ended.needs_you);

    let still_in_grace = engine
        .collect_panel_at(&environment, started_at + Duration::from_secs(5))
        .expect("grace panel should be collected");
    assert_eq!(still_in_grace.sessions[0].elapsed, Duration::from_secs(4));

    let expired = engine
        .collect_panel_at(&environment, started_at + Duration::from_secs(6))
        .expect("expired panel should be collected");
    assert!(expired.sessions.is_empty());
}

#[test]
fn fresh_mtime_without_a_live_process_never_puts_a_session_on_panel() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
    let mut fixture = Fixture::new();
    fixture.add_session("fresh-but-dead", "/work/dead", SessionState::Idle, now);
    let engine = fixture.engine();
    let environment = fixture.environment([("/work/dead", 0)]);

    let panel = engine
        .collect_panel_at(&environment, now)
        .expect("panel should be collected");

    assert!(panel.sessions.is_empty());
    assert!(!panel.needs_you);
}

#[test]
fn live_process_count_limits_same_cwd_sessions_to_the_most_recent_candidates() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
    let mut fixture = Fixture::new();
    fixture.add_session(
        "oldest",
        "/work/shared",
        SessionState::Idle,
        now - Duration::from_secs(30),
    );
    fixture.add_session(
        "middle",
        "/work/shared",
        SessionState::Idle,
        now - Duration::from_secs(20),
    );
    fixture.add_session(
        "newest",
        "/work/shared",
        SessionState::Idle,
        now - Duration::from_secs(10),
    );
    let engine = fixture.engine();
    let environment = fixture.environment([("/work/shared", 2)]);

    let panel = engine
        .collect_panel_at(&environment, now)
        .expect("panel should be collected");

    let session_ids: Vec<_> = panel
        .sessions
        .iter()
        .map(|entry| entry.session.session_id.as_str())
        .collect();
    assert_eq!(session_ids, vec!["middle", "newest"]);
}

#[test]
fn all_non_waiting_sessions_keep_needs_you_off() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
    let mut fixture = Fixture::new();
    fixture.add_session("working", "/work/working", SessionState::Working, now);
    fixture.add_session("idle", "/work/idle", SessionState::Idle, now);
    let engine = fixture.engine();
    let environment = fixture.environment([("/work/working", 1), ("/work/idle", 1)]);

    let panel = engine
        .collect_panel_at(&environment, now)
        .expect("panel should be collected");

    assert!(!panel.needs_you);
}

#[test]
fn recently_ended_sessions_sort_after_waiting_working_and_idle() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
    let mut fixture = Fixture::new();
    fixture.add_session("waiting", "/work/waiting", SessionState::Waiting, now);
    fixture.add_session("working", "/work/working", SessionState::Working, now);
    fixture.add_session("idle", "/work/idle", SessionState::Idle, now);
    fixture.add_session("ending", "/work/ending", SessionState::Idle, now);
    let engine = fixture.engine();
    let (environment, probe) = fixture.environment_with_probe([
        ("/work/waiting", 1),
        ("/work/working", 1),
        ("/work/idle", 1),
        ("/work/ending", 1),
    ]);
    engine
        .collect_panel_at(&environment, now)
        .expect("initial panel should be collected");
    probe.set_live_count("/work/ending", 0);

    let panel = engine
        .collect_panel_at(&environment, now + Duration::from_secs(1))
        .expect("panel with recently ended session should be collected");

    let states: Vec<_> = panel
        .sessions
        .iter()
        .map(|entry| entry.session.state)
        .collect();
    assert_eq!(
        states,
        vec![
            SessionState::Waiting,
            SessionState::Working,
            SessionState::Idle,
            SessionState::Ended,
        ]
    );
}
