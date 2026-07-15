# 核实：Anthropic 官方 Agent View / 手机推送 对 WhoNeedsMe 护城河的冲击

- 日期：2026-07-15
- 研究者：research 子代理（一手源优先）
- 前置调研：`docs/research/2026-07-14-whoneedsme-market-research.md`

## 决策问题

此前市场调研中，多个中文信源（小红书 / B站 / 抖音）称 Anthropic 近期上线了官方 "Agent View"（据称"一行一个会话展示状态和最新回复"），以及 Claude Code 支持"任务完成或需要输入时推送手机通知"。若这两点属实且覆盖面广，会直接侵蚀 WhoNeedsMe 的两个独占卖点：

- **(a)** 一屏看到本机所有 agent 会话状态（**含 Codex 等非 Anthropic 工具**），"谁在等我"高亮；
- **(b)** 零安装文件监听、不装 hooks 也能看到会话列表。

本报告用一手源核实这两个官方功能的**真实范围与边界**，判断护城河是否还成立。

---

## 核实到的事实

### 一、官方 Agent View 是什么、范围多大

**事实 1.1 — Agent View 是真实的、官方的 Claude Code CLI 功能。【官方一手】**
它是 Claude Code 终端 TUI 里的一个面板，按左箭头或运行 `claude agents` 打开。官方博客与官方文档均确认存在。
> "Dispatch and manage many Claude Code sessions from one screen. Agent view shows what every session is doing and which ones need your input."
来源（官方一手）：
- 官方博客 https://claude.com/blog/agent-view-in-claude-code
- 官方文档 https://code.claude.com/docs/en/agent-view

**中文信源"一行一个会话展示状态和最新回复"这句转述是准确的。**【官方一手佐证】文档确认每一行显示会话名、状态图标、最新一条回复内容、以及上次交互时间；需要你输入的会话自动置顶。中文转述本身没错——但它容易让人误以为"覆盖全部工具 / 是网页端面板"，这两点是错的（见下）。

**事实 1.2 — 它只展示本机的 Claude Code 会话，且只展示"已被后台化（background）"的会话。【官方一手】**
这是最关键的边界。文档明确：
> "Interactive sessions you have open in other terminals don't appear until you background them. Subagents and teammates a session spawns aren't listed as separate rows."
> "By default the list shows every background session you've **started**, across all your projects…"

含义：
- 普通的、在别的终端里跑着的**前台交互式 `claude` 会话不会自动出现**。必须用 `/bg`、`/background`、`claude --bg`、或在空 prompt 按 `←` 主动把它扔到后台，才会进入 Agent View。
- 会话都在**本机**运行（不是云端）：由 per-user supervisor / daemon 托管，状态存在 `~/.claude/`（`~/.claude/daemon/roster.json`、`~/.claude/jobs/<id>/state.json`）。机器关机会话就停。
来源：https://code.claude.com/docs/en/agent-view

**事实 1.3 — 它不支持任何非 Anthropic agent（Codex CLI / Gemini 等）。【官方一手，已明确确认】**
文档语义上就是"Claude Code sessions"，且明确列出哪些能出现、哪些不能，其中**没有任何第三方工具**。二次核实 WebFetch 得到明确结论：Agent View 与 Codex CLI、Gemini 等其他 AI 编码工具**没有任何集成**，只管 Claude Code 自己的后台会话。
来源：https://code.claude.com/docs/en/agent-view （"Agent View is Claude Code-specific and has no integration with other AI coding tools like Codex CLI, Gemini, or non-Claude agents."）

**事实 1.4 — 它确实做"谁在等我 / needs input"的语义状态判定，不只是 running/done。【官方一手】**
文档定义 6 态，且把"在等你输入"和"空闲"分开：
| 状态 | 图标 | 含义 |
|---|---|---|
| Working | 动画 | 正在跑工具 / 生成回复 |
| **Needs input** | 黄 | 正等你回答某个具体问题或权限决定 |
| Idle | 暗淡 | 没事做，等你下一条 prompt |
| Completed | 绿 | 任务成功完成 |
| Failed | 红 | 出错终止 |
| Stopped | 灰 | 被 `Ctrl+X` / `claude stop` 手动停 |

需要你的会话自动升到列表顶部。所以"谁在等我高亮"这一交互，官方**已在 Claude Code 内部原生实现**。
来源：https://code.claude.com/docs/en/agent-view

**事实 1.5 — 跨平台，需付费订阅，需较新版本，仍是 Research Preview。【官方一手】**
- 平台：macOS / Linux / Windows 均支持（macOS 15+ 需本地网络权限等）。
- 订阅层级：官方博客称 Research Preview，覆盖 **Pro、Max、Team、Enterprise、Claude API plans**。
- 版本门槛：需 **Claude Code v2.1.139 或更新**（`claude --version` 查）。
- 限制：后台会话按订阅额度独立计费，跑 10 个约等于 10 倍消耗；机器关机即停。
来源：https://claude.com/blog/agent-view-in-claude-code ；https://code.claude.com/docs/en/agent-view

> 注意：不要把 **Agent View**（终端内 TUI，看本机后台 Claude Code 会话）和 **claude.ai/code 的会话列表**（Remote Control 的网页 / 手机端会话列表）混为一谈。两者都是 Anthropic-only、Claude-Code-only，但形态不同。中文信源有把两者混淆的风险。

---

### 二、Claude Code 的手机推送通知

**事实 2.1 — 真实存在，是官方功能，属于 Remote Control。【官方一手】**
官方文档"Mobile push notifications"章节明确：Remote Control 激活时，Claude 可向手机推送。需 **Claude Code v2.1.110 或更新**。
来源（官方一手）：https://code.claude.com/docs/en/remote-control
官方 Claude Code 账号（@ClaudeDevs）也发推宣布过该功能【官方账号一手，X 平台】：https://x.com/ClaudeDevs/status/2049154855143649315

**事实 2.2 — 它能判定"waiting for user input"这类语义状态。【官方一手】**
`/config` 里有两个开关：
- **Push when Claude decides** — 长任务完成等主动通知；
- **Push when actions required** — **权限提示和提问（即"需要你决定"）时推送**。
官方原文：
> "Claude decides when to push. It typically sends one when a long-running task finishes or when it needs a decision from you to continue."
来源：https://code.claude.com/docs/en/remote-control

**事实 2.3 — 不是纯本机触发，需联网 + claude.ai 账号 + 手机 App，且经 Anthropic 服务器中转。【官方一手】**
- 要求：**Pro / Max / Team / Enterprise 订阅**；**API key 明确不支持**（`"Remote Control requires a claude.ai subscription"`）；需装 Claude 手机 App 并用**同一账号**登录；Team/Enterprise 还要 Owner 先在后台开 Remote Control 开关。
- 传输：本机会话向 Anthropic API 发出站 HTTPS 并轮询；连接期间**会话 transcript 存在 Anthropic 服务器**上用于跨设备同步；**Zero Data Retention 合规组织无法启用**。
- 也就是说：离线、无账号、纯 API key、或 ZDR 合规环境下，官方 push **用不了**。
来源：https://code.claude.com/docs/en/remote-control

**事实 2.4 — 只服务 Claude Code（Anthropic 自家），不覆盖 Codex 等第三方。【官方一手，推断+确认】**
Remote Control / push 全程绑定 claude.ai 账号与 Anthropic API，结构上只能是 Claude Code 自己的会话。
来源：https://code.claude.com/docs/en/remote-control

---

### 三、GitHub anthropics/claude-code#13024（"waiting for input" hook）

**事实 3.1 — 该 issue 到今天仍是 OPEN，未解决。【官方一手，gh CLI 实测】**
- 标题：`[FEATURE] Add hook for when Claude is waiting for user input`
- 状态：**OPEN**；26 条评论；2025-12-04 开；最后活动 2026-07-05（仍有人 +1）。
- labels：enhancement / has repro / area:core。
来源：https://github.com/anthropics/claude-code/issues/13024

**事实 3.2 — 官方至今没提供"一等公民"的 WaitingForInput / yield hook。【一手，issue 讨论】**
社区反复指出现有机制不够：`Stop` hook 会在把活交给后台 subagent/shell 时也误触发，无法干净区分"turn 干净结束"与"用户成了瓶颈"；`Notification` 的 `idle_prompt` 要等 60s+，太慢。多位开发者（含 Dash 项目 nthomsencph）说，缺这个 hook 导致上层多任务 UI 里，调 `AskUserQuestion` 的 agent 会"永远显示 busy"，形成假阳性。

**事实 3.3 — 第三方工具（含 Agent Island）至今仍靠 transcript / process 扫描来判断 waiting 状态。【一手，issue 评论】**
issue 里 tristan666666 自述其开源 macOS 应用 **Agent Island** 就是为此而建：对**本地 Claude / Codex 会话**，把 running / your-turn / stuck 状态投射到 MacBook 刘海（notch）。他明确说，缺原生 yield hook，"叠在 Claude Code 之上的工具只能不断重造脆弱的 process/transcript 检查"。
来源：https://github.com/anthropics/claude-code/issues/13024（评论）；Agent Island 仓库 https://github.com/tristan666666/agent-island

> 结论性含义：官方在**产品层面**（Agent View 面板 + 手机 push）已经能显示"谁在等我"，但**没有把这个语义信号作为开放 hook/API 暴露给第三方工具**。所以 WhoNeedsMe 这类外部工具想拿到"waiting"状态，仍然只能走文件 / transcript 监听——这条技术路线依然是必要且成立的。

---

## 对护城河的结论

先给判断：**护城河变窄了，但没有被推平。** 官方吃掉的是"单一 Claude Code、付费、愿意改用后台会话工作流"的那一块；WhoNeedsMe 真正独占的内核（跨工具聚合 + 零账号离线 + 覆盖前台会话）官方**结构性地不会做**。

### 差异点 (a)：一屏看本机所有会话（含 Codex 等非 Anthropic 工具）

| 维度 | 官方是否覆盖 | 依据 |
|---|---|---|
| Claude Code 后台会话、needs-input 语义、最新回复、置顶 | **已覆盖** | 事实 1.1 / 1.4 |
| Codex CLI 及任何非 Anthropic agent | **不覆盖，且不会做** | 事实 1.3 |
| 普通前台 / 交互式 `claude` 会话（未手动 background） | **不自动覆盖** | 事实 1.2 |
| 跨平台 | 已覆盖（mac/win/linux） | 事实 1.5 |

结论：(a) 的**跨工具聚合内核仍成立**——Anthropic 不可能给 Codex 做面板，这是 WhoNeedsMe 的结构性优势。但"我只用 Claude Code、都用后台会话"这一细分人群，官方 Agent View 已经原生满足，这部分市场被侵蚀。WhoNeedsMe 需把卖点从"看 Claude Code 会话"上移到"**同时看 Claude Code + Codex + 其他工具，且不用你改工作流去 background**"。

### 差异点 (b)：零安装文件监听、不装 hooks 也能看会话列表

| 维度 | 官方 Agent View / push | WhoNeedsMe |
|---|---|---|
| 是否要订阅 / 账号 | 要（push 连 API key 都不行） | 不要 |
| 是否要联网 / 云中转 | push 要，transcript 上传 Anthropic 服务器 | 不要，纯本地只读 |
| 是否要改工作流 | 要（逐个 background 会话 / 开 Remote Control） | 不要 |
| 是否要装 hook | Agent View 不需 hook（官方内建），但第三方拿不到 waiting 信号仍需自建 | 只读文件监听，不装 hook |
| ZDR 合规 / 纯离线环境 | 用不了 | 能用 |

结论：(b) 作为**技术实现路径依然成立且被 #13024 反向印证**——官方没开放 waiting hook，第三方只能文件/transcript 监听。作为**卖点**，它的锋利度取决于对手：对比官方 Remote Control/push，WhoNeedsMe 的真实优势是"**无订阅、无账号、离线、本地只读、覆盖前台会话与 Codex**"。

### 仍然独占、官方短期不会覆盖的部分（建议主打）

1. **跨工具聚合**：Claude Code + Codex CLI + 其他，一屏。官方结构性不做。
2. **零账号 / 离线 / 本地只读**：API-key-only 用户、ZDR 合规组织、不愿联网的人，官方 push / Remote Control 全用不了。
3. **覆盖前台 / 交互式会话**：不需要用户为了被看见而把每个会话 `/bg`。
4. **常驻托盘 / notch，独立于终端**：不进 Claude Code TUI 就能一眼看到（但注意：**Agent Island 已占这个生态位**，且已支持 Claude+Codex——它才是比官方更直接的正面竞品）。

### 主要威胁

- 对"单 Claude Code + 付费 + 后台会话工作流"用户，官方已双管齐下（终端内 Agent View + 手机 push）原生回答"谁在等我"。WhoNeedsMe 不要在这条正面赛道和官方硬碰。
- 真正的近身竞品不是 Anthropic，而是 **Agent Island**（开源、native、已做 Claude+Codex 的 notch 状态投射）。护城河讨论应把它当头号对手，而非官方 Agent View。

---

## 未能证实 / 存疑的部分

- **中文信源的精确措辞未逐条溯源**：小红书/B站/抖音"一行一个会话展示状态和最新回复"这一描述，方向上被官方文档证实（事实 1.1/1.4），但中文源是否把 Agent View 说成"网页端/桌面端面板"或"覆盖所有工具"——这类具体夸大**未找到可核对的原帖**，按二手转述对待，勿据其判断官方范围。【二手转述，方向属实但细节未证实】
- **Agent View 在 "Claude API plans" 的确切可用形态**：官方博客把 API plans 列入可用范围，但 Remote Control/push 明确不支持 API key。二者是不同功能，Agent View（本机后台会话）与 push（需 claude.ai 账号）的订阅门槛不一致，具体 API-plan 用户能用 Agent View 到什么程度，**未在文档中逐字确认**。【官方博客一手，但存在功能口径差异，需实测】
- **官方是否即将开放 WaitingForInput hook**：#13024 仍 OPEN，无官方 Anthropic 成员承诺时间表。官方是否会把 Agent View 的语义状态作为 hook/API 对第三方开放，**无一手证据，属未知**。这是护城河 (b) 最大的不确定变量——若官方哪天开放该 hook，第三方文件监听的必要性会下降。【未能证实】
- **@ClaudeDevs 推文**为官方 Claude Code 账号发布，作宣传佐证；但功能细节以官方 docs 为准（X 平台内容易删改，可信度低于 docs）。【官方账号一手，证据强度次于文档】

---

## 一手源清单

| 源 | 类型 | 可信度 |
|---|---|---|
| https://claude.com/blog/agent-view-in-claude-code | Anthropic 官方博客 | 官方一手 |
| https://code.claude.com/docs/en/agent-view | Anthropic 官方文档 | 官方一手 |
| https://code.claude.com/docs/en/remote-control | Anthropic 官方文档 | 官方一手 |
| https://github.com/anthropics/claude-code/issues/13024 | 官方仓库 issue（gh CLI 实测 OPEN） | 官方一手 |
| https://x.com/ClaudeDevs/status/2049154855143649315 | 官方 Claude Code X 账号 | 官方账号一手（次于 docs） |
| https://github.com/tristan666666/agent-island | 竞品 Agent Island 仓库 | 第三方一手 |
