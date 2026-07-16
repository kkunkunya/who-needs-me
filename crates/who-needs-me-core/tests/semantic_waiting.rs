use std::path::Path;

use who_needs_me_core::{Adapter, ClaudeAdapter, SessionArtifact, SessionState, WaitingReason};

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
