use std::time::Duration;

/// A supported coding-agent CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Provider {
    id: &'static str,
    display_name: &'static str,
}

impl Provider {
    pub const fn new(id: &'static str, display_name: &'static str) -> Self {
        Self { id, display_name }
    }

    pub const fn id(self) -> &'static str {
        self.id
    }

    pub const fn display_name(self) -> &'static str {
        self.display_name
    }
}

/// The stable primary key for a session.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionKey {
    pub provider: Provider,
    pub session_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Working,
    Waiting,
    Idle,
    Ended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitingReason {
    PermissionApproval,
    AnswerQuestion,
    ConfirmPlan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionMetadata {
    pub cwd: Option<String>,
    pub cwd_display: Option<String>,
    pub git_branch: Option<String>,
    pub model: Option<String>,
    pub context_tokens: Option<u64>,
    pub context_usage_percent: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub provider: Provider,
    pub session_id: String,
    pub state: SessionState,
    pub waiting_reason: Option<WaitingReason>,
    pub metadata: SessionMetadata,
}

impl Session {
    pub fn key(&self) -> SessionKey {
        SessionKey {
            provider: self.provider,
            session_id: self.session_id.clone(),
        }
    }

    pub fn needs_you(&self) -> bool {
        self.state == SessionState::Waiting
    }
}

/// One Session as presented by the Engine's panel seam.
#[derive(Debug, Clone, PartialEq)]
pub struct PanelSession {
    pub session: Session,
    pub elapsed: Duration,
}

/// The complete Engine snapshot consumed by panel-like surfaces.
#[derive(Debug, Clone, PartialEq)]
pub struct PanelSnapshot {
    pub sessions: Vec<PanelSession>,
    pub needs_you: bool,
}
