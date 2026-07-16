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
pub struct ClaudeAdapter;

impl ClaudeAdapter {
    pub const PROVIDER: Provider = Provider::new("claude", "Claude");
    pub const DESCRIPTOR: ProviderDescriptor =
        ProviderDescriptor::new(Self::PROVIDER, "claude", ".claude/projects", "claude");
}

impl Adapter for ClaudeAdapter {
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
        let session_id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .ok_or_else(|| CoreError::InvalidSessionPath(path.to_path_buf()))?
            .to_owned();
        let file = File::open(path).map_err(|source| CoreError::ReadPath {
            path: path.to_path_buf(),
            source,
        })?;
        let mut metadata = SessionMetadata {
            cwd: None,
            cwd_display: None,
            git_branch: None,
            model: None,
            context_tokens: None,
            context_usage_percent: None,
        };
        let mut folded_state = None;
        let mut state = None;
        let mut state_entered_at = None;
        let mut pending_waits = Vec::new();

        for line in BufReader::new(file).lines() {
            let line = line.map_err(|source| CoreError::ReadPath {
                path: path.to_path_buf(),
                source,
            })?;
            if let Ok(event) = serde_json::from_str::<Value>(&line) {
                replace_json_string(&mut metadata.cwd, event.get("cwd"));
                replace_json_string(&mut metadata.git_branch, event.get("gitBranch"));
                replace_json_string(&mut metadata.model, event.pointer("/message/model"));
                let event_state = if event.get("type").and_then(Value::as_str) == Some("assistant")
                {
                    Some(if has_direct_waiting_tool(&event) {
                        SessionState::Waiting
                    } else {
                        match event
                            .pointer("/message/stop_reason")
                            .and_then(Value::as_str)
                        {
                            Some("end_turn") => SessionState::Idle,
                            _ => SessionState::Working,
                        }
                    })
                } else if event.get("type").and_then(Value::as_str) == Some("user") {
                    Some(SessionState::Working)
                } else {
                    None
                };
                if let Some(event_state) = event_state {
                    folded_state = Some(event_state);
                }
                if let Some(usage) = event.pointer("/message/usage") {
                    let context_tokens = [
                        "input_tokens",
                        "cache_creation_input_tokens",
                        "cache_read_input_tokens",
                    ]
                    .into_iter()
                    .filter_map(|field| usage.get(field).and_then(Value::as_u64))
                    .sum();
                    metadata.context_tokens = Some(context_tokens);
                }
                apply_semantic_waiting_events(&event, &mut pending_waits);
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
        }

        metadata.context_usage_percent = metadata
            .model
            .as_deref()
            .and_then(claude_context_window)
            .zip(metadata.context_tokens)
            .map(|(window, tokens)| tokens as f32 / window as f32 * 100.0);

        if metadata.cwd.is_none() {
            metadata.cwd = path
                .parent()
                .map(|parent| parent.to_string_lossy().into_owned());
        }
        metadata.cwd_display = metadata.cwd.as_deref().map(display_cwd);
        let state = state.unwrap_or(SessionState::Working);

        let waiting_reason = pending_waits.last().map(|(_, reason)| *reason);

        Ok(Session {
            provider: Self::PROVIDER,
            session_id,
            state,
            state_entered_at,
            waiting_reason,
            metadata,
        })
    }
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

fn has_direct_waiting_tool(event: &Value) -> bool {
    event
        .pointer("/message/content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|block| {
            block.get("type").and_then(Value::as_str) == Some("tool_use")
                && block.get("id").and_then(Value::as_str).is_some()
                && matches!(
                    block.get("name").and_then(Value::as_str),
                    Some("AskUserQuestion" | "ExitPlanMode")
                )
        })
}

fn claude_context_window(model: &str) -> Option<u64> {
    match model {
        "claude-opus-4-20250514"
        | "claude-opus-4-0"
        | "claude-sonnet-4-20250514"
        | "claude-sonnet-4-0"
        | "claude-sonnet-4-5-20250929"
        | "claude-sonnet-4-5"
        | "claude-haiku-4-5-20251001"
        | "claude-haiku-4-5"
        | "claude-3-7-sonnet-20250219"
        | "claude-3-5-sonnet-20241022"
        | "claude-3-5-haiku-20241022" => Some(200_000),
        _ => None,
    }
}

fn apply_semantic_waiting_events(event: &Value, pending_waits: &mut Vec<(String, WaitingReason)>) {
    let Some(content) = event.pointer("/message/content").and_then(Value::as_array) else {
        return;
    };

    for block in content {
        match block.get("type").and_then(Value::as_str) {
            Some("tool_use") => {
                let Some(tool_use_id) = block.get("id").and_then(Value::as_str) else {
                    continue;
                };
                let reason = match block.get("name").and_then(Value::as_str) {
                    Some("AskUserQuestion") => WaitingReason::AnswerQuestion,
                    Some("ExitPlanMode") => WaitingReason::ConfirmPlan,
                    _ => continue,
                };
                pending_waits.retain(|(pending_id, _)| pending_id != tool_use_id);
                pending_waits.push((tool_use_id.to_owned(), reason));
            }
            Some("tool_result") => {
                let Some(tool_use_id) = block.get("tool_use_id").and_then(Value::as_str) else {
                    continue;
                };
                pending_waits.retain(|(pending_id, _)| pending_id != tool_use_id);
            }
            _ => {}
        }
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
