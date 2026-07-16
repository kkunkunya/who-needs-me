use std::{path::Path, sync::Arc};

use who_needs_me_core::{
    Adapter, ClaudeAdapter, CoreResult, Engine, Environment, ProcessProbe, Provider,
    ProviderDescriptor, Session, SessionArtifact, SessionMetadata, SessionState,
};

struct ControlledProbe;

impl ProcessProbe for ControlledProbe {
    fn live_process_count(&self, provider: ProviderDescriptor, cwd: &Path) -> usize {
        usize::from(
            provider.provider() == ClaudeAdapter::PROVIDER && cwd.ends_with("project-alpha"),
        )
    }
}

struct AlwaysAliveProbe;

impl ProcessProbe for AlwaysAliveProbe {
    fn live_process_count(&self, _provider: ProviderDescriptor, _cwd: &Path) -> usize {
        1
    }
}

struct OneProcessPerDirectoryProbe;

impl ProcessProbe for OneProcessPerDirectoryProbe {
    fn live_process_count(&self, _provider: ProviderDescriptor, _cwd: &Path) -> usize {
        1
    }
}

const SYNTHETIC_PROVIDER: Provider = Provider::new("synthetic", "Synthetic");
const SYNTHETIC_DESCRIPTOR: ProviderDescriptor = ProviderDescriptor::new(
    SYNTHETIC_PROVIDER,
    "synthetic-agent",
    ".synthetic/sessions",
    "synthetic",
);

struct SyntheticAdapter;

impl Adapter for SyntheticAdapter {
    fn descriptor(&self) -> ProviderDescriptor {
        SYNTHETIC_DESCRIPTOR
    }

    fn discover(&self, _session_data_root: &Path) -> CoreResult<Vec<SessionArtifact>> {
        Ok(vec![SessionArtifact::from_path("synthetic-session.jsonl")])
    }

    fn parse(&self, _artifact: &SessionArtifact) -> CoreResult<Session> {
        Ok(Session {
            provider: SYNTHETIC_PROVIDER,
            session_id: "synthetic-session".into(),
            state: SessionState::Idle,
            waiting_reason: None,
            metadata: SessionMetadata {
                cwd: Some("/fixtures/synthetic".into()),
                cwd_display: None,
                git_branch: None,
                model: None,
                context_tokens: None,
                context_usage_percent: None,
            },
        })
    }
}

#[test]
fn a_new_provider_only_needs_an_adapter_and_descriptor() {
    let environment = Environment::builder(Arc::new(AlwaysAliveProbe))
        .with_session_data_root(SYNTHETIC_PROVIDER, "/unused/sessions")
        .with_hook_event_root(SYNTHETIC_PROVIDER, "/unused/hooks")
        .build()
        .expect("synthetic environment should be valid");
    let engine = Engine::new(vec![Box::new(SyntheticAdapter)]);

    let sessions = engine
        .collect_sessions(&environment)
        .expect("synthetic provider should use the generic engine path");

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].provider, SYNTHETIC_PROVIDER);
    assert_eq!(sessions[0].provider.display_name(), "Synthetic");
}

#[test]
fn fixture_environment_only_lists_live_claude_sessions() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/claude");
    let session_root = fixture_root.join("sessions");
    let hook_root = fixture_root.join("hooks");
    let environment = Environment::builder(Arc::new(ControlledProbe))
        .with_session_data_root(ClaudeAdapter::PROVIDER, &session_root)
        .with_hook_event_root(ClaudeAdapter::PROVIDER, &hook_root)
        .build()
        .expect("fixture environment should be valid");
    let engine = Engine::new(vec![Box::new(ClaudeAdapter)]);

    let sessions = engine
        .collect_sessions(&environment)
        .expect("fixture sessions should be readable");

    assert_eq!(
        sessions,
        vec![Session {
            provider: ClaudeAdapter::PROVIDER,
            session_id: "session-alpha".into(),
            state: SessionState::Idle,
            waiting_reason: None,
            metadata: SessionMetadata {
                cwd: Some("/fixtures/project-alpha".into()),
                cwd_display: Some("fixtures/project-alpha".into()),
                git_branch: Some("feat/alpha".into()),
                model: Some("claude-sonnet-4-5-20250929".into()),
                context_tokens: Some(100),
                context_usage_percent: Some(0.05),
            },
        }]
    );
    assert_eq!(
        environment
            .hook_event_root(ClaudeAdapter::PROVIDER)
            .expect("Claude hook root should be injected"),
        hook_root.as_path()
    );
}

#[test]
fn one_live_process_only_keeps_the_most_recent_same_cwd_session() {
    let fixture_root = std::env::temp_dir().join(format!(
        "who-needs-me-same-cwd-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let session_root = fixture_root.join("sessions/project-shared");
    let hook_root = fixture_root.join("hooks");
    std::fs::create_dir_all(&session_root).expect("session fixture directory should be created");
    std::fs::create_dir_all(&hook_root).expect("hook fixture directory should be created");
    write_claude_fixture(
        &session_root.join("historical-session.jsonl"),
        "historical-session",
        "/fixtures/project-shared",
        1,
    );
    write_claude_fixture(
        &session_root.join("active-session.jsonl"),
        "active-session",
        "/fixtures/project-shared",
        2,
    );

    let environment = Environment::builder(Arc::new(OneProcessPerDirectoryProbe))
        .with_session_data_root(ClaudeAdapter::PROVIDER, fixture_root.join("sessions"))
        .with_hook_event_root(ClaudeAdapter::PROVIDER, &hook_root)
        .build()
        .expect("fixture environment should be valid");
    let sessions = Engine::new(vec![Box::new(ClaudeAdapter)])
        .collect_sessions(&environment)
        .expect("same-cwd fixture sessions should be readable");

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, "active-session");
    assert_eq!(sessions[0].state, SessionState::Working);
    assert!(!sessions[0].needs_you());

    std::fs::remove_dir_all(&fixture_root).expect("temporary fixture should be removed");
}

fn write_claude_fixture(path: &Path, session_id: &str, cwd: &str, activity_second: u64) {
    std::fs::write(
        path,
        format!(r#"{{"cwd":"{cwd}","sessionId":"{session_id}","gitBranch":"main","type":"user"}}"#),
    )
    .expect("session fixture should be written");
    std::fs::File::options()
        .write(true)
        .open(path)
        .expect("session fixture should reopen")
        .set_times(std::fs::FileTimes::new().set_modified(
            std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(activity_second),
        ))
        .expect("session fixture activity time should be controlled");
}
