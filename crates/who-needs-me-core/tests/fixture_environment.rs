use std::{path::Path, sync::Arc};

use who_needs_me_core::{
    ClaudeAdapter, Engine, Environment, ProcessProbe, Provider, Session, SessionMetadata,
    SessionState,
};

struct ControlledProbe;

impl ProcessProbe for ControlledProbe {
    fn is_alive(&self, provider: Provider, cwd: &Path) -> bool {
        provider == Provider::Claude && cwd.ends_with("project-alpha")
    }
}

#[test]
fn fixture_environment_discovers_claude_sessions() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/claude");
    let session_root = fixture_root.join("sessions");
    let hook_root = fixture_root.join("hooks");
    let environment = Environment::builder(Arc::new(ControlledProbe))
        .with_session_data_root(Provider::Claude, &session_root)
        .with_hook_event_root(Provider::Claude, &hook_root)
        .build()
        .expect("fixture environment should be valid");
    let engine = Engine::new(vec![Box::new(ClaudeAdapter)]);

    let sessions = engine
        .collect_sessions(&environment)
        .expect("fixture sessions should be readable");

    assert_eq!(
        sessions,
        vec![
            Session {
                provider: Provider::Claude,
                session_id: "session-alpha".into(),
                state: SessionState::Idle,
                waiting_reason: None,
                metadata: SessionMetadata {
                    cwd: Some("/fixtures/project-alpha".into()),
                    git_branch: Some("feat/alpha".into()),
                    model: Some("claude-sonnet-4-5-20250929".into()),
                    context_usage_percent: None,
                },
            },
            Session {
                provider: Provider::Claude,
                session_id: "session-beta".into(),
                state: SessionState::Ended,
                waiting_reason: None,
                metadata: SessionMetadata {
                    cwd: Some("/fixtures/project-beta".into()),
                    git_branch: Some("main".into()),
                    model: Some("claude-opus-4-20250514".into()),
                    context_usage_percent: None,
                },
            },
        ]
    );
    assert_eq!(
        environment
            .hook_event_root(Provider::Claude)
            .expect("Claude hook root should be injected"),
        hook_root.as_path()
    );
}
