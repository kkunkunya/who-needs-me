use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime},
};

use who_needs_me_core::{
    Adapter, CodexAdapter, Engine, Environment, ProcessProbe, ProviderDescriptor, SessionArtifact,
    SessionState, WaitingReason,
};

fn fixture(name: &str) -> SessionArtifact {
    SessionArtifact::from_path(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/codex/adapter")
            .join(name),
    )
}

#[test]
fn discovery_keeps_only_concrete_codex_cli_rollouts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/codex/discovery");

    let artifacts = CodexAdapter
        .discover(&root)
        .expect("Codex discovery fixture should be readable");

    assert_eq!(artifacts.len(), 1);
    assert_eq!(
        artifacts[0]
            .path()
            .file_name()
            .and_then(|name| name.to_str()),
        Some("cli.jsonl")
    );
}

#[test]
fn completed_codex_turn_is_idle_with_display_metadata() {
    let session = CodexAdapter
        .parse(&fixture("idle-completed-turn.jsonl"))
        .expect("committed Codex fixture should parse");

    assert_eq!(session.session_id, "codex-idle-session");
    assert_eq!(session.provider, CodexAdapter::PROVIDER);
    assert_eq!(session.state, SessionState::Idle);
    assert!(!session.needs_you());
    assert_eq!(
        session.state_entered_at,
        Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1_784_075_403))
    );
    assert_eq!(session.metadata.cwd.as_deref(), Some("/work/acme/payments"));
    assert_eq!(
        session.metadata.cwd_display.as_deref(),
        Some("work/acme/payments")
    );
    assert_eq!(session.metadata.git_branch.as_deref(), Some("feat/codex"));
    assert_eq!(session.metadata.model.as_deref(), Some("gpt-5.6-sol"));
    assert_eq!(session.metadata.context_tokens, Some(64_600));
    assert_eq!(session.metadata.context_usage_percent, Some(25.0));
}

#[test]
fn unanswered_request_user_input_waits_for_an_answer() {
    let session = CodexAdapter
        .parse(&fixture("waiting-question.jsonl"))
        .expect("question fixture should parse");

    assert_eq!(session.state, SessionState::Waiting);
    assert_eq!(session.waiting_reason, Some(WaitingReason::AnswerQuestion));
    assert!(session.needs_you());
    assert_eq!(
        session.state_entered_at,
        Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1_784_075_410))
    );
}

#[test]
fn unfinished_update_plan_waits_for_plan_confirmation() {
    let session = CodexAdapter
        .parse(&fixture("waiting-plan.jsonl"))
        .expect("plan fixture should parse");

    assert_eq!(session.state, SessionState::Waiting);
    assert_eq!(session.waiting_reason, Some(WaitingReason::ConfirmPlan));
    assert!(session.needs_you());
}

#[test]
fn matching_function_output_clears_a_semantic_wait() {
    let session = CodexAdapter
        .parse(&fixture("resolved-question.jsonl"))
        .expect("resolved question fixture should parse");

    assert_eq!(session.state, SessionState::Working);
    assert_eq!(session.waiting_reason, None);
    assert!(!session.needs_you());
}

#[test]
fn matching_custom_output_clears_a_custom_semantic_wait() {
    let session = CodexAdapter
        .parse(&fixture("resolved-custom-question.jsonl"))
        .expect("resolved custom question fixture should parse");

    assert_eq!(session.state, SessionState::Working);
    assert_eq!(session.waiting_reason, None);
    assert!(!session.needs_you());
}

#[test]
fn mismatched_output_variant_degrades_without_fabricating_a_wait() {
    let session = CodexAdapter
        .parse(&fixture("mismatched-output-variant.jsonl"))
        .expect("mismatched output fixture should parse");

    assert_eq!(session.state, SessionState::Working);
    assert_eq!(session.waiting_reason, None);
    assert!(!session.needs_you());
}

#[test]
fn matching_function_output_before_a_question_does_not_fabricate_a_wait() {
    let session = CodexAdapter
        .parse(&fixture("output-before-question.jsonl"))
        .expect("out-of-order question fixture should parse");

    assert_eq!(session.state, SessionState::Working);
    assert_eq!(session.waiting_reason, None);
    assert!(!session.needs_you());
}

#[test]
fn unknown_lifecycle_format_does_not_reuse_a_stale_idle_state() {
    let session = CodexAdapter
        .parse(&fixture("unknown-lifecycle-format.jsonl"))
        .expect("unknown Codex formats should degrade without crashing");

    assert_eq!(session.state, SessionState::Working);
    assert_eq!(session.waiting_reason, None);
    assert!(!session.needs_you());
    assert_eq!(session.metadata.model, None);
    assert_eq!(session.metadata.context_tokens, None);
    assert_eq!(session.metadata.context_usage_percent, None);
}

#[test]
fn missing_or_malformed_cwd_is_not_replaced_with_a_rollout_directory() {
    let missing = CodexAdapter
        .parse(&fixture("missing-cwd.jsonl"))
        .expect("missing-cwd fixture should parse");
    let malformed = CodexAdapter
        .parse(&fixture("unknown-lifecycle-format.jsonl"))
        .expect("malformed-cwd fixture should parse");

    assert_eq!(
        (
            missing.metadata.cwd,
            missing.metadata.cwd_display,
            malformed.metadata.cwd,
            malformed.metadata.cwd_display,
        ),
        (None, None, None, None)
    );
}

#[test]
fn unfinished_ordinary_tool_call_remains_working_without_a_hook() {
    let session = CodexAdapter
        .parse(&fixture("working-ordinary-tool.jsonl"))
        .expect("ordinary tool fixture should parse");

    assert_eq!(session.state, SessionState::Working);
    assert_eq!(session.waiting_reason, None);
    assert!(!session.needs_you());
}

#[test]
fn semantic_tools_without_call_ids_remain_conservatively_working() {
    let session = CodexAdapter
        .parse(&fixture("missing-id-semantic-tools.jsonl"))
        .expect("missing-id fixture should parse");

    assert_eq!(session.state, SessionState::Working);
    assert_eq!(session.waiting_reason, None);
    assert!(!session.needs_you());
}

#[test]
fn duplicate_call_ids_do_not_fabricate_a_semantic_wait() {
    let session = CodexAdapter
        .parse(&fixture("duplicate-call-id.jsonl"))
        .expect("duplicate call-id fixture should parse");

    assert_eq!(session.state, SessionState::Working);
    assert_eq!(session.waiting_reason, None);
    assert!(!session.needs_you());
}

#[test]
fn unrelated_function_output_preserves_the_wait_and_elapsed_anchor() {
    let session = CodexAdapter
        .parse(&fixture("waiting-with-unrelated-output.jsonl"))
        .expect("unrelated output fixture should parse");

    assert_eq!(session.state, SessionState::Waiting);
    assert_eq!(session.waiting_reason, Some(WaitingReason::AnswerQuestion));
    assert_eq!(
        session.state_entered_at,
        Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1_784_075_410))
    );
}

#[test]
fn unavailable_context_window_keeps_tokens_without_inventing_a_percentage() {
    let session = CodexAdapter
        .parse(&fixture("tokens-without-window.jsonl"))
        .expect("missing context-window fixture should parse");

    assert_eq!(session.metadata.context_tokens, Some(42_000));
    assert_eq!(session.metadata.context_usage_percent, None);
}

struct TwoLiveCodexProcesses;

impl ProcessProbe for TwoLiveCodexProcesses {
    fn live_process_count(&self, provider: ProviderDescriptor, cwd: &Path) -> usize {
        usize::from(
            provider == CodexAdapter::DESCRIPTOR && cwd == Path::new("/fixtures/codex-project"),
        ) * 2
    }
}

struct ControlledCodexProcesses(AtomicUsize);

impl ControlledCodexProcesses {
    fn set(&self, count: usize) {
        self.0.store(count, Ordering::SeqCst);
    }
}

impl ProcessProbe for ControlledCodexProcesses {
    fn live_process_count(&self, provider: ProviderDescriptor, cwd: &Path) -> usize {
        if provider == CodexAdapter::DESCRIPTOR && cwd == Path::new("/fixtures/codex-project") {
            self.0.load(Ordering::SeqCst)
        } else {
            0
        }
    }
}

#[test]
fn codex_fixtures_reach_the_panel_through_the_public_environment_seam() {
    let fixture_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/codex/environment");
    let environment = Environment::builder(Arc::new(TwoLiveCodexProcesses))
        .with_session_data_root(CodexAdapter::PROVIDER, fixture_root.join("sessions"))
        .with_hook_event_root(CodexAdapter::PROVIDER, fixture_root.join("hooks"))
        .build()
        .expect("Codex fixture environment should be valid");

    let panel = Engine::new(vec![Box::new(CodexAdapter)])
        .collect_panel_at(
            &environment,
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_784_075_500),
        )
        .expect("Codex fixture panel should collect");

    assert!(panel.needs_you);
    assert_eq!(panel.sessions.len(), 2);
    assert_eq!(
        panel.sessions[0].session.session_id,
        "codex-resumed-session"
    );
    assert_eq!(panel.sessions[0].session.state, SessionState::Waiting);
    assert_eq!(
        panel.sessions[0].session.waiting_reason,
        Some(WaitingReason::AnswerQuestion)
    );
    assert_eq!(panel.sessions[0].elapsed, Duration::from_secs(90));
    assert_eq!(
        panel.sessions[1].session.session_id,
        "codex-original-session"
    );
    assert_eq!(panel.sessions[1].session.state, SessionState::Idle);
}

#[test]
fn codex_sessions_obey_same_cwd_capacity_and_ended_grace() {
    let fixture_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/codex/environment");
    let probe = Arc::new(ControlledCodexProcesses(AtomicUsize::new(1)));
    let environment = Environment::builder(probe.clone())
        .with_session_data_root(CodexAdapter::PROVIDER, fixture_root.join("sessions"))
        .with_hook_event_root(CodexAdapter::PROVIDER, fixture_root.join("hooks"))
        .build()
        .expect("Codex fixture environment should be valid");
    let engine = Engine::new(vec![Box::new(CodexAdapter)]);
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_784_075_500);

    let one_process = engine
        .collect_panel_at(&environment, now)
        .expect("one live Codex process should cap the shared cwd at one Session");
    assert_eq!(one_process.sessions.len(), 1);

    probe.set(0);
    let ended = engine
        .collect_panel_at(&environment, now + Duration::from_secs(1))
        .expect("dead Codex process should produce the Ended grace state");
    assert_eq!(ended.sessions.len(), 1);
    assert_eq!(ended.sessions[0].session.state, SessionState::Ended);
    assert_eq!(ended.sessions[0].elapsed, Duration::ZERO);
    assert!(!ended.needs_you);

    let expired = engine
        .collect_panel_at(&environment, now + Duration::from_secs(6))
        .expect("Codex Ended grace should expire");
    assert!(expired.sessions.is_empty());
}
