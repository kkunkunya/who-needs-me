import assert from "node:assert/strict";
import test from "node:test";

class FakeNode {
  children = [];
  className = "";
  dataset = {};
  value = "";
  #text = "";

  get textContent() {
    return this.#text + this.children.map((child) => child.textContent).join("");
  }

  set textContent(value) {
    this.#text = value;
    this.children = [];
  }

  append(...children) {
    this.children.push(
      ...children.map((child) =>
        typeof child === "string" ? Object.assign(new FakeNode(), { textContent: child }) : child,
      ),
    );
  }

  replaceChildren(...children) {
    this.#text = "";
    this.children = children;
  }

  setAttribute() {}
}

function installPage(sessions) {
  const sessionList = new FakeNode();
  const sessionCount = new FakeNode();
  const elements = new Map([
    ["session-list", sessionList],
    ["session-count", sessionCount],
  ]);

  globalThis.document = {
    createElement: () => new FakeNode(),
    createTextNode: (value) => Object.assign(new FakeNode(), { textContent: value }),
    getElementById: (id) => elements.get(id) ?? null,
  };
  globalThis.window = {
    __TAURI__: {
      core: {
        invoke: async (command) => {
          assert.equal(command, "list_sessions");
          return sessions;
        },
      },
    },
  };

  return { sessionList, sessionCount };
}

async function renderCase(name, sessions) {
  const page = installPage(sessions);
  const moduleUrl = new URL(`../dist/main.js?case=${name}`, import.meta.url);
  await import(moduleUrl);
  await waitUntil(() => page.sessionCount.value !== "");
  return page;
}

async function waitUntil(predicate) {
  for (let attempt = 0; attempt < 20; attempt += 1) {
    if (predicate()) return;
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
  throw new Error("debug surface did not finish rendering");
}

const mixedSessions = [
  {
    provider: "Claude",
    sessionId: "session-alpha",
    state: "idle",
    waitingReason: null,
    metadata: {
      cwd: "/fixtures/project-alpha",
      gitBranch: "feat/alpha",
      model: "claude-sonnet-4-5-20250929",
      contextUsagePercent: null,
    },
  },
  {
    provider: "Claude",
    sessionId: "session-beta",
    state: "idle",
    waitingReason: null,
    metadata: {
      cwd: "/fixtures/project-beta",
      gitBranch: "main",
      model: "claude-opus-4-20250514",
      contextUsagePercent: null,
    },
  },
];

test("Session Debug Surface renders mixed fixture identity and the empty result", async () => {
  const mixed = await renderCase("mixed", mixedSessions);
  assert.equal(mixed.sessionCount.value, "2 sessions");
  assert.equal(mixed.sessionList.children.length, 2);
  assert.match(mixed.sessionList.children[0].textContent, /fixtures\/project-alpha/);
  assert.match(mixed.sessionList.children[0].textContent, /Claude/);
  assert.match(mixed.sessionList.children[0].textContent, /session-alpha/);
  assert.match(mixed.sessionList.children[1].textContent, /fixtures\/project-beta/);
  assert.match(mixed.sessionList.children[1].textContent, /Claude/);
  assert.match(mixed.sessionList.children[1].textContent, /session-beta/);
  assert.doesNotMatch(mixed.sessionList.textContent, /uncertain|low confidence|不确定/i);

  const empty = await renderCase("empty", []);
  assert.equal(empty.sessionCount.value, "0 sessions");
  assert.equal(empty.sessionList.children.length, 0);
});
