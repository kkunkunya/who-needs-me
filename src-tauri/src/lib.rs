use std::{path::Path, sync::Arc};

use serde::Serialize;
use tauri::{State, image::Image, tray::TrayIconBuilder};
use who_needs_me_core::{
    ClaudeAdapter, CoreResult, Engine, Environment, ProcessProbe, Provider, Session,
    SessionMetadata, SessionState, WaitingReason,
};

struct AppState {
    engine: Engine,
    environment: Environment,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionView {
    provider: &'static str,
    session_id: String,
    state: &'static str,
    waiting_reason: Option<&'static str>,
    metadata: SessionMetadataView,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionMetadataView {
    cwd: Option<String>,
    git_branch: Option<String>,
    model: Option<String>,
    context_usage_percent: Option<f32>,
}

impl From<Session> for SessionView {
    fn from(session: Session) -> Self {
        Self {
            provider: provider_label(session.provider),
            session_id: session.session_id,
            state: state_label(session.state),
            waiting_reason: session.waiting_reason.map(waiting_reason_label),
            metadata: metadata_view(session.metadata),
        }
    }
}

#[tauri::command]
fn list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionView>, String> {
    state
        .engine
        .collect_sessions(&state.environment)
        .map(|sessions| sessions.into_iter().map(SessionView::from).collect())
        .map_err(|error| error.to_string())
}

pub fn run() {
    let environment = runtime_environment().expect("WhoNeedsMe environment should be available");
    let state = AppState {
        engine: Engine::new(vec![Box::new(ClaudeAdapter)]),
        environment,
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![list_sessions])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .setup(|app| {
            TrayIconBuilder::new()
                .icon(placeholder_tray_icon())
                .icon_as_template(true)
                .tooltip("WhoNeedsMe")
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running WhoNeedsMe");
}

fn runtime_environment() -> CoreResult<Environment> {
    let Some(fixture_root) = std::env::var_os("WHO_NEEDS_ME_FIXTURE_ROOT") else {
        return Environment::production();
    };
    let fixture_root = std::path::PathBuf::from(fixture_root);
    let fixture_root = if fixture_root.is_absolute() {
        fixture_root
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(fixture_root)
    };

    Environment::builder(Arc::new(FixtureProcessProbe))
        .with_session_data_root(Provider::Claude, fixture_root.join("sessions"))
        .with_hook_event_root(Provider::Claude, fixture_root.join("hooks"))
        .build()
}

struct FixtureProcessProbe;

impl ProcessProbe for FixtureProcessProbe {
    fn is_alive(&self, _provider: Provider, _cwd: &Path) -> bool {
        true
    }
}

fn placeholder_tray_icon() -> Image<'static> {
    const SIZE: u32 = 16;
    let mut rgba = vec![0; (SIZE * SIZE * 4) as usize];
    for y in 3..13 {
        for x in 3..13 {
            let on_outline = x == 3 || x == 12 || y == 3 || y == 12;
            if on_outline {
                let pixel = ((y * SIZE + x) * 4) as usize;
                rgba[pixel..pixel + 4].copy_from_slice(&[138, 138, 142, 255]);
            }
        }
    }
    Image::new_owned(rgba, SIZE, SIZE)
}

fn provider_label(provider: Provider) -> &'static str {
    match provider {
        Provider::Claude => "Claude",
        Provider::Codex => "Codex",
    }
}

fn state_label(state: SessionState) -> &'static str {
    match state {
        SessionState::Working => "working",
        SessionState::Waiting => "waiting",
        SessionState::Idle => "idle",
        SessionState::Ended => "ended",
    }
}

fn waiting_reason_label(reason: WaitingReason) -> &'static str {
    match reason {
        WaitingReason::PermissionApproval => "permissionApproval",
        WaitingReason::AnswerQuestion => "answerQuestion",
        WaitingReason::ConfirmPlan => "confirmPlan",
    }
}

fn metadata_view(metadata: SessionMetadata) -> SessionMetadataView {
    SessionMetadataView {
        cwd: metadata.cwd,
        git_branch: metadata.git_branch,
        model: metadata.model,
        context_usage_percent: metadata.context_usage_percent,
    }
}
