use std::{path::Path, sync::Arc};

use who_needs_me_core::{
    Adapter, ClaudeAdapter, CoreResult, Engine, Environment, ProcessProbe, Provider,
    ProviderDescriptor, Session, SessionArtifact, SessionMetadata, SessionState,
};

struct ControlledProbe;

impl ProcessProbe for ControlledProbe {
    fn is_alive(&self, provider: ProviderDescriptor, cwd: &Path) -> bool {
        provider.provider() == ClaudeAdapter::PROVIDER && cwd.ends_with("project-alpha")
    }
}

struct AlwaysAliveProbe;

impl ProcessProbe for AlwaysAliveProbe {
    fn is_alive(&self, _provider: ProviderDescriptor, _cwd: &Path) -> bool {
        true
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
                git_branch: None,
                model: None,
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
                git_branch: Some("feat/alpha".into()),
                model: Some("claude-sonnet-4-5-20250929".into()),
                context_usage_percent: None,
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
