const TAURI_CONFIG: &str = include_str!("../../../src-tauri/tauri.conf.json");
const CAPABILITIES: &str = include_str!("../../../src-tauri/capabilities/default.json");
const APP_BRIDGE: &str = include_str!("../../../src-tauri/src/lib.rs");
const APP_MANIFEST: &str = include_str!("../../../src-tauri/Cargo.toml");
const DEBUG_UI: &str = include_str!("../../../ui/main.ts");
const DEBUG_HTML: &str = include_str!("../../../ui/index.html");
const DEBUG_CSS: &str = include_str!("../../../ui/styles.css");

#[test]
fn runtime_surfaces_are_local_and_read_only() {
    let compact_config: String = TAURI_CONFIG
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    assert!(compact_config.contains(r#""frontendDist":"../dist""#));
    assert!(!compact_config.contains(r#""devUrl""#));

    assert!(CAPABILITIES.contains("core:default"));
    for forbidden_permission in ["http:", "shell:", "process:", "websocket"] {
        assert!(!CAPABILITIES.contains(forbidden_permission));
    }

    for forbidden_dependency in ["reqwest", "hyper", "axum", "warp", "tauri-plugin-http"] {
        assert!(!APP_MANIFEST.contains(forbidden_dependency));
    }

    for forbidden_server_api in ["TcpListener", "TcpStream", "UdpSocket"] {
        assert!(!APP_BRIDGE.contains(forbidden_server_api));
    }
    for forbidden_agent_action in [
        "Command::new(\"claude\")",
        "Command::new(\"codex\")",
        ".stdin(",
    ] {
        assert!(!APP_BRIDGE.contains(forbidden_agent_action));
    }

    assert!(DEBUG_UI.contains(r#"invoke<Session[]>("list_sessions")"#));
    assert!(DEBUG_UI.contains("provider: string;"));
    assert!(!DEBUG_UI.contains(r#""Claude" | "Codex""#));
    assert!(!APP_BRIDGE.contains("fn provider_label"));
    for forbidden_web_api in ["fetch(", "WebSocket", "XMLHttpRequest", "EventSource"] {
        assert!(!DEBUG_UI.contains(forbidden_web_api));
    }
    for forbidden_control in ["<button", "<form", "<input", "contenteditable"] {
        assert!(!DEBUG_HTML.contains(forbidden_control));
    }

    for neutral_token in [
        "--bg-panel: #1C1C1E;",
        "--text-primary: #ECECEC;",
        "--text-secondary: #8A8A8E;",
        "--state-working: #7C7C82;",
        "--state-idle: #5A5A5E;",
        "--state-ended: #48484C;",
    ] {
        assert!(DEBUG_CSS.contains(neutral_token));
    }
    assert!(!DEBUG_CSS.to_uppercase().contains("#E39B3E"));
}

#[test]
fn production_runtime_registers_both_builtin_adapters() {
    assert!(APP_BRIDGE.contains("Box::new(ClaudeAdapter)"));
    assert!(APP_BRIDGE.contains("Box::new(CodexAdapter)"));
}
