use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
    time::SystemTime,
};

use chrono::DateTime;
use serde_json::Value;

use crate::{
    Adapter, CoreError, CoreResult, Provider, ProviderDescriptor, Session, SessionArtifact,
    SessionMetadata, SessionState, WaitingReason,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct CodexAdapter;

impl CodexAdapter {
    pub const PROVIDER: Provider = Provider::new("codex", "Codex");
    pub const DESCRIPTOR: ProviderDescriptor =
        ProviderDescriptor::new(Self::PROVIDER, "codex", ".codex/sessions", "codex");
}

impl Adapter for CodexAdapter {
    fn descriptor(&self) -> ProviderDescriptor {
        Self::DESCRIPTOR
    }

    fn discover(&self, session_data_root: &Path) -> CoreResult<Vec<SessionArtifact>> {
        let mut artifacts = Vec::new();
        discover_jsonl(session_data_root, &mut artifacts)?;
        artifacts.sort_by(|left, right| left.path().cmp(right.path()));
        Ok(artifacts)
    }

    fn parse(&self, artifact: &SessionArtifact) -> CoreResult<Session> {
        let path = artifact.path();
        let fallback_session_id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .ok_or_else(|| CoreError::InvalidSessionPath(path.to_path_buf()))?
            .to_owned();
        let file = File::open(path).map_err(|source| CoreError::ReadPath {
            path: path.to_path_buf(),
            source,
        })?;
        let mut session_id = None;
        let mut metadata = SessionMetadata {
            cwd: None,
            cwd_display: None,
            git_branch: None,
            model: None,
            context_tokens: None,
            context_usage_percent: None,
        };
        let mut context_window = None;
        let mut folded_state = None;
        let mut state = None;
        let mut state_entered_at = None;
        let mut pending_waits = Vec::new();

        for line in BufReader::new(file).lines() {
            let line = line.map_err(|source| CoreError::ReadPath {
                path: path.to_path_buf(),
                source,
            })?;
            let Ok(event) = serde_json::from_str::<Value>(&line) else {
                continue;
            };

            match event.get("type").and_then(Value::as_str) {
                Some("session_meta") => {
                    replace_json_string(&mut session_id, event.pointer("/payload/id"));
                    metadata.cwd = json_string(event.pointer("/payload/cwd"));
                    metadata.git_branch = json_string(event.pointer("/payload/git/branch"));
                }
                Some("turn_context") => {
                    metadata.cwd = json_string(event.pointer("/payload/cwd"));
                    metadata.model = json_string(event.pointer("/payload/model"));
                }
                Some("event_msg") => {
                    let next_state = match event.pointer("/payload/type").and_then(Value::as_str) {
                        Some("task_started") => Some(SessionState::Working),
                        Some("task_complete" | "turn_aborted") => Some(SessionState::Idle),
                        Some("token_count") => None,
                        Some(_) | None => Some(SessionState::Working),
                    };
                    if let Some(next_state) = next_state {
                        folded_state = Some(next_state);
                    }
                    if event.pointer("/payload/type").and_then(Value::as_str) == Some("token_count")
                    {
                        metadata.context_tokens = event
                            .pointer("/payload/info/last_token_usage/total_tokens")
                            .and_then(Value::as_u64);
                        context_window = event
                            .pointer("/payload/info/model_context_window")
                            .and_then(Value::as_u64);
                    }
                }
                Some("response_item") => {
                    apply_tool_event(&event, &mut pending_waits);
                    if matches!(
                        event.pointer("/payload/type").and_then(Value::as_str),
                        Some("function_call" | "custom_tool_call")
                    ) {
                        folded_state = Some(SessionState::Working);
                    }
                }
                _ => {}
            }

            let exposed_state = if pending_waits.is_empty() {
                folded_state
            } else {
                Some(SessionState::Waiting)
            };
            if let Some(exposed_state) = exposed_state {
                if state != Some(exposed_state) {
                    state_entered_at = event
                        .get("timestamp")
                        .and_then(Value::as_str)
                        .and_then(parse_timestamp);
                }
                state = Some(exposed_state);
            }
        }

        metadata.context_usage_percent = context_window
            .filter(|window| *window > 0)
            .zip(metadata.context_tokens)
            .map(|(window, tokens)| tokens as f32 / window as f32 * 100.0);
        if metadata.cwd.is_none() {
            metadata.cwd = path
                .parent()
                .map(|parent| parent.to_string_lossy().into_owned());
        }
        metadata.cwd_display = metadata.cwd.as_deref().map(display_cwd);

        Ok(Session {
            provider: Self::PROVIDER,
            session_id: session_id.unwrap_or(fallback_session_id),
            state: state.unwrap_or(SessionState::Working),
            state_entered_at,
            waiting_reason: pending_waits.last().map(|(_, reason)| *reason),
            metadata,
        })
    }
}

fn apply_tool_event(event: &Value, pending_waits: &mut Vec<(String, WaitingReason)>) {
    match event.pointer("/payload/type").and_then(Value::as_str) {
        Some("function_call" | "custom_tool_call") => {
            let Some(call_id) = event.pointer("/payload/call_id").and_then(Value::as_str) else {
                return;
            };
            let reason = match event.pointer("/payload/name").and_then(Value::as_str) {
                Some("request_user_input") => WaitingReason::AnswerQuestion,
                Some("update_plan") => WaitingReason::ConfirmPlan,
                _ => return,
            };
            pending_waits.retain(|(pending_id, _)| pending_id != call_id);
            pending_waits.push((call_id.to_owned(), reason));
        }
        Some("function_call_output" | "custom_tool_call_output") => {
            let Some(call_id) = event.pointer("/payload/call_id").and_then(Value::as_str) else {
                return;
            };
            pending_waits.retain(|(pending_id, _)| pending_id != call_id);
        }
        _ => {}
    }
}

fn discover_jsonl(root: &Path, artifacts: &mut Vec<SessionArtifact>) -> CoreResult<()> {
    if !root.exists() {
        return Ok(());
    }
    let entries = fs::read_dir(root).map_err(|source| CoreError::ReadPath {
        path: root.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| CoreError::ReadPath {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| CoreError::ReadPath {
            path: path.clone(),
            source,
        })?;
        if file_type.is_dir() {
            discover_jsonl(&path, artifacts)?;
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "jsonl") {
            artifacts.push(SessionArtifact::from_path(path));
        }
    }
    Ok(())
}

fn replace_json_string(target: &mut Option<String>, candidate: Option<&Value>) {
    if let Some(value) = candidate.and_then(Value::as_str) {
        *target = Some(value.to_owned());
    }
}

fn json_string(candidate: Option<&Value>) -> Option<String> {
    candidate.and_then(Value::as_str).map(str::to_owned)
}

fn parse_timestamp(timestamp: &str) -> Option<SystemTime> {
    DateTime::parse_from_rfc3339(timestamp)
        .ok()
        .map(SystemTime::from)
}

fn display_cwd(cwd: &str) -> String {
    let components = Path::new(cwd)
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => Some(value.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let start = components.len().saturating_sub(3);
    components[start..].join("/")
}
