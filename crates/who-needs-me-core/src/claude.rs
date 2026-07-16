use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{
    Adapter, CoreError, CoreResult, Provider, Session, SessionArtifact, SessionMetadata,
    SessionState,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct ClaudeAdapter;

impl Adapter for ClaudeAdapter {
    fn provider(&self) -> Provider {
        Provider::Claude
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
            git_branch: None,
            model: None,
            context_usage_percent: None,
        };

        for line in BufReader::new(file).lines() {
            let line = line.map_err(|source| CoreError::ReadPath {
                path: path.to_path_buf(),
                source,
            })?;
            replace_string(&mut metadata.cwd, extract_json_string(&line, "cwd"));
            replace_string(
                &mut metadata.git_branch,
                extract_json_string(&line, "gitBranch"),
            );
            replace_string(&mut metadata.model, extract_json_string(&line, "model"));
        }

        if metadata.cwd.is_none() {
            metadata.cwd = path
                .parent()
                .map(|parent| parent.to_string_lossy().into_owned());
        }

        Ok(Session {
            provider: Provider::Claude,
            session_id,
            // The walking skeleton deliberately avoids guessing Needs You.
            state: SessionState::Idle,
            waiting_reason: None,
            metadata,
        })
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

fn replace_string(target: &mut Option<String>, candidate: Option<String>) {
    if let Some(value) = candidate {
        *target = Some(value);
    }
}

/// Minimal string-field reader for the walking-skeleton fixtures and Claude JSONL.
/// Full provider state parsing deliberately belongs to the follow-up Adapter slice.
fn extract_json_string(document: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let key_end = document.find(&needle)? + needle.len();
    let after_colon = document[key_end..]
        .trim_start()
        .strip_prefix(':')?
        .trim_start();
    let mut chars = after_colon.strip_prefix('"')?.chars();
    let mut value = String::new();
    while let Some(character) = chars.next() {
        match character {
            '"' => return Some(value),
            '\\' => match chars.next()? {
                '"' => value.push('"'),
                '\\' => value.push('\\'),
                '/' => value.push('/'),
                'b' => value.push('\u{0008}'),
                'f' => value.push('\u{000c}'),
                'n' => value.push('\n'),
                'r' => value.push('\r'),
                't' => value.push('\t'),
                _ => return None,
            },
            other => value.push(other),
        }
    }
    None
}
