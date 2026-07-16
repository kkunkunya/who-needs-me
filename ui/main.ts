type SessionState = "working" | "waiting" | "idle" | "ended";
type WaitingReason = "permissionApproval" | "answerQuestion" | "confirmPlan";

interface SessionMetadata {
  cwd: string | null;
  gitBranch: string | null;
  model: string | null;
  contextUsagePercent: number | null;
}

interface Session {
  provider: string;
  sessionId: string;
  state: SessionState;
  waitingReason: WaitingReason | null;
  metadata: SessionMetadata;
}

interface Window {
  __TAURI__?: {
    core: {
      invoke<T>(command: string): Promise<T>;
    };
  };
}

const sessionList = requiredElement<HTMLUListElement>("session-list");
const sessionCount = requiredElement<HTMLOutputElement>("session-count");

async function renderSessions(): Promise<void> {
  if (!window.__TAURI__) {
    renderFailure("Tauri bridge is unavailable. Start this view with cargo tauri dev.");
    return;
  }

  try {
    const sessions = await window.__TAURI__.core.invoke<Session[]>("list_sessions");
    sessionList.replaceChildren(...sessions.map(renderSession));
    sessionCount.value = `${sessions.length} session${sessions.length === 1 ? "" : "s"}`;
  } catch (error) {
    renderFailure(error instanceof Error ? error.message : String(error));
  }
}

function renderSession(session: Session): HTMLLIElement {
  const item = document.createElement("li");
  item.className = "session-row";
  item.dataset.state = session.state;

  const identity = document.createElement("div");
  identity.className = "session-identity";

  const title = document.createElement("strong");
  title.textContent = projectIdentity(session.metadata.cwd);
  identity.append(title);

  const path = document.createElement("span");
  path.className = "path";
  path.textContent = session.metadata.cwd ?? "Project path unavailable";
  identity.append(path);

  const metadata = document.createElement("div");
  metadata.className = "session-metadata";
  metadata.append(
    metadataItem(session.provider),
    metadataItem(session.metadata.gitBranch ?? "No branch"),
    metadataItem(session.metadata.model ?? "Model unknown"),
    metadataItem(session.sessionId),
  );

  const state = document.createElement("span");
  state.className = "session-state";
  const dot = document.createElement("span");
  dot.className = "state-dot";
  dot.setAttribute("aria-hidden", "true");
  state.append(dot, document.createTextNode(session.state));

  item.append(identity, metadata, state);
  return item;
}

function metadataItem(value: string): HTMLSpanElement {
  const item = document.createElement("span");
  item.textContent = value;
  return item;
}

function projectIdentity(cwd: string | null): string {
  if (!cwd) return "Unknown project";
  const segments = cwd.split(/[\\/]/).filter(Boolean);
  return segments.slice(-3).join("/") || cwd;
}

function renderFailure(message: string): void {
  const item = document.createElement("li");
  item.className = "debug-error";
  item.textContent = message;
  sessionList.replaceChildren(item);
  sessionCount.value = "Unavailable";
}

function requiredElement<ElementType extends HTMLElement>(id: string): ElementType {
  const element = document.getElementById(id);
  if (!element) throw new Error(`Missing debug UI element: ${id}`);
  return element as ElementType;
}

void renderSessions();
