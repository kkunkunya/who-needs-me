use std::{
    path::Path,
    sync::Arc,
    time::{Duration, SystemTime},
};

use who_needs_me_core::{
    Adapter, ClaudeAdapter, Engine, Environment, ProcessProbe, ProviderDescriptor, SessionArtifact,
    SessionState, WaitingReason,
};

fn parse_fixture(relative_path: &str) -> who_needs_me_core::Session {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/claude/semantic-waiting")
        .join(relative_path);

    ClaudeAdapter
        .parse(&SessionArtifact::from_path(path))
        .expect("Claude semantic-waiting fixture should parse")
}

#[test]
fn unanswered_ask_user_question_waits_for_an_answer() {
    let session = parse_fixture("unanswered-question.jsonl");

    assert_eq!(session.state, SessionState::Waiting);
    assert_eq!(session.waiting_reason, Some(WaitingReason::AnswerQuestion));
    assert!(session.needs_you());
}

#[test]
fn unconfirmed_exit_plan_mode_waits_for_plan_confirmation() {
    let session = parse_fixture("unconfirmed-plan.jsonl");

    assert_eq!(session.state, SessionState::Waiting);
    assert_eq!(session.waiting_reason, Some(WaitingReason::ConfirmPlan));
    assert!(session.needs_you());
}

#[test]
fn matching_tool_results_clear_semantic_waits_and_restore_the_working_turn() {
    let session = parse_fixture("resolved-semantic-waits.jsonl");

    assert_eq!(session.state, SessionState::Working);
    assert_eq!(session.waiting_reason, None);
    assert!(!session.needs_you());
}

#[test]
fn unmatched_permission_capable_tool_does_not_claim_needs_you_without_a_hook() {
    let session = parse_fixture("permission-unknown-without-hook.jsonl");

    assert_eq!(session.waiting_reason, None);
    assert!(!session.needs_you());
}

#[test]
fn malformed_and_unknown_input_remains_conservatively_working() {
    let session = parse_fixture("malformed-and-unknown-input.jsonl");

    assert_eq!(session.state, SessionState::Working);
    assert_eq!(session.waiting_reason, None);
    assert!(!session.needs_you());
}

#[test]
fn semantic_tools_without_ids_remain_conservatively_working() {
    let session = parse_fixture("missing-id-semantic-tools.jsonl");

    assert_eq!(session.state, SessionState::Working);
    assert_eq!(session.waiting_reason, None);
    assert!(!session.needs_you());
}

struct LiveClaudeProcess;

impl ProcessProbe for LiveClaudeProcess {
    fn live_process_count(&self, _provider: ProviderDescriptor, _cwd: &Path) -> usize {
        1
    }
}

#[test]
fn unrelated_tool_result_preserves_the_pending_wait_and_elapsed_anchor() {
    let relative_path = "unmatched-result/pending-question-with-unrelated-result.jsonl";
    let session = parse_fixture(relative_path);
    let waiting_since = SystemTime::UNIX_EPOCH + Duration::from_secs(60);

    assert_eq!(session.state, SessionState::Waiting);
    assert_eq!(session.waiting_reason, Some(WaitingReason::AnswerQuestion));
    assert!(session.needs_you());
    assert_eq!(session.state_entered_at, Some(waiting_since));

    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/claude/semantic-waiting/unmatched-result");
    let environment = Environment::builder(Arc::new(LiveClaudeProcess))
        .with_session_data_root(ClaudeAdapter::PROVIDER, &fixture_root)
        .with_hook_event_root(ClaudeAdapter::PROVIDER, fixture_root.join("hooks"))
        .build()
        .expect("semantic-waiting environment should be valid");
    let panel = Engine::new(vec![Box::new(ClaudeAdapter)])
        .collect_panel_at(
            &environment,
            SystemTime::UNIX_EPOCH + Duration::from_secs(120),
        )
        .expect("semantic-waiting panel should collect");

    assert_eq!(panel.sessions.len(), 1);
    assert_eq!(panel.sessions[0].session.state, SessionState::Waiting);
    assert_eq!(
        panel.sessions[0].session.waiting_reason,
        Some(WaitingReason::AnswerQuestion)
    );
    assert!(panel.needs_you);
    assert_eq!(panel.sessions[0].elapsed, Duration::from_secs(60));
}
