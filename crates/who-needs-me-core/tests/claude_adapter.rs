use std::{
    path::Path,
    time::{Duration, SystemTime},
};

use who_needs_me_core::{Adapter, ClaudeAdapter, SessionArtifact, SessionState, WaitingReason};

fn fixture(name: &str) -> SessionArtifact {
    SessionArtifact::from_path(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/claude/adapter")
            .join(name),
    )
}

#[test]
fn completed_turn_is_idle_with_claude_display_metadata() {
    let session = ClaudeAdapter
        .parse(&fixture("idle-known-model.jsonl"))
        .expect("committed Claude fixture should parse");

    assert_eq!(session.session_id, "idle-known-model");
    assert_eq!(session.state, SessionState::Idle);
    assert!(!session.needs_you());
    assert_eq!(session.metadata.cwd.as_deref(), Some("/work/acme/payments"));
    assert_eq!(
        session.metadata.cwd_display.as_deref(),
        Some("work/acme/payments")
    );
    assert_eq!(
        session.metadata.git_branch.as_deref(),
        Some("feat/checkout")
    );
    assert_eq!(
        session.metadata.model.as_deref(),
        Some("claude-sonnet-4-5-20250929")
    );
    assert_eq!(session.metadata.context_tokens, Some(100_000));
    assert_eq!(session.metadata.context_usage_percent, Some(50.0));
}

#[test]
fn unknown_model_keeps_context_tokens_without_inventing_a_percentage() {
    let session = ClaudeAdapter
        .parse(&fixture("idle-unknown-model.jsonl"))
        .expect("unknown Claude models should degrade gracefully");

    assert_eq!(session.metadata.model.as_deref(), Some("claude-future-9"));
    assert_eq!(session.metadata.context_tokens, Some(30_000));
    assert_eq!(session.metadata.context_usage_percent, None);
}

#[test]
fn unknown_version_in_a_known_model_family_does_not_reuse_a_stale_window() {
    let session = ClaudeAdapter
        .parse(&fixture("idle-unknown-family-version.jsonl"))
        .expect("future Claude versions should degrade gracefully");

    assert_eq!(session.metadata.context_tokens, Some(20_000));
    assert_eq!(session.metadata.context_usage_percent, None);
}

#[test]
fn unresolved_ordinary_tool_call_is_working_and_never_needs_you() {
    let session = ClaudeAdapter
        .parse(&fixture("working-tool.jsonl"))
        .expect("working Claude fixture should parse");

    assert_eq!(session.state, SessionState::Working);
    assert!(!session.needs_you());
    assert_eq!(session.waiting_reason, None);
}

#[test]
fn unresolved_question_tool_is_waiting_from_a_direct_session_signal() {
    let session = ClaudeAdapter
        .parse(&fixture("waiting-question.jsonl"))
        .expect("waiting Claude fixture should parse");

    assert_eq!(session.state, SessionState::Waiting);
    assert_eq!(
        session.state_entered_at,
        Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1_784_118_601))
    );
    assert!(session.needs_you());
    assert_eq!(session.waiting_reason, Some(WaitingReason::AnswerQuestion));
}

#[test]
fn tool_payload_fields_cannot_overwrite_provider_metadata() {
    let session = ClaudeAdapter
        .parse(&fixture("working-metadata-noise.jsonl"))
        .expect("metadata noise fixture should parse");

    assert_eq!(session.state, SessionState::Working);
    assert_eq!(
        session.metadata.model.as_deref(),
        Some("claude-opus-4-20250514")
    );
    assert_eq!(session.metadata.cwd.as_deref(), Some("/work/acme/noise"));
    assert_eq!(session.metadata.git_branch.as_deref(), Some("feat/real"));
}
