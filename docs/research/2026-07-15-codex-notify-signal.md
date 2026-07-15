# Codex 的「Needs-You(审批)」沉默信号核实：notify 补不回、PermissionRequest hook 能

- 日期：2026-07-15
- 状态：完成（官方文档 + `openai/codex` 源码实测双证）
- 姊妹文档：Claude Code 侧的同一问题见 `2026-07-15-v1-feasibility-spikes.md` Spike A（权限等待是纯文件的硬边界）。本文是 Codex 侧的对应核实。

---

## 决策问题

WhoNeedsMe（本机只读状态托盘）要判定 Codex CLI 会话「是不是在等用户」。对 Codex 而言最主要的「等你」时刻是**审批提示**（approval prompt：allow this command / apply this patch）。

我方已用本机 566 个真实 rollout 文件验证：**审批请求不会作为独立事件写进 `~/.codex/sessions/` 的 rollout JSONL**（文件里只有 `approval_policy` 配置和批准后的结果），所以纯文件监听判不出「正在等审批」。

需要回答：**Codex 侧有没有一个不增加 agent 认知负担的信号源，能在「需要用户审批」时对外发信号？** 具体拆成 4 问：(1) `notify` 是什么、在哪些事件触发；(2) 有没有别的机制能拿到「正在等审批」；(3) 稳定性；(4) 能否做到对 agent 零认知负担。

**一句话结论（先给）**：能，但**不是靠 `notify`**——`notify` 源码里只有 `agent-turn-complete` 一个事件，永远不在等审批时触发。真正能补回 Needs-You(审批) 信号的是 Codex 的 **hooks 系统里的 `PermissionRequest` hook**：它就在审批路径上、UI 弹出之前触发，可以只发信号不返回任何决策，从而对 agent/model 上下文零注入。

---

## 核实到的事实（逐条带一手源 + 可信度）

### A. `notify` 配置：存在，但只在 `agent-turn-complete` 触发——补不回审批信号

1. **`notify` 就是「spawn 外部程序 + 传一个 JSON 参数」，猜的没错。** 官方 config 文档：`notify` 类型是 `array<string>`，描述为 "Command invoked for notifications; receives a JSON payload from Codex."。用法例：`notify = ["python3", "/path/to/notify.py"]`，JSON 作为**单个 argv 参数**传入。
   一手源：官方 Configuration Reference / Advanced Configuration（`learn.chatgpt.com/docs/config-file/config-reference.md`、`.../config-advanced.md`）。**可信度：官方一手。**

2. **`notify` 当前只支持一个事件类型：`agent-turn-complete`。** 官方 Advanced 文档原文："trigger an external program whenever Codex emits supported events (**currently only `agent-turn-complete`**)"。JSON 字段：`type`、`thread-id`、`turn-id`、`cwd`、`input-messages`、`last-assistant-message`。
   一手源：`learn.chatgpt.com/docs/config-file/config-advanced.md`。**可信度：官方一手。**

3. **源码实测确认：`notify` 的事件枚举物理上只有一个变体。** `codex-rs/hooks/src/legacy_notify.rs` 里 `enum UserNotification` **只有 `AgentTurnComplete` 一个变体**；`legacy_notify_json()` 只 `match HookEvent::AfterAgent`。也就是说「等审批时触发 notify」在当前源码里**根本没有代码路径**，不是配置没开、是没实现。
   注意文件名已是 `legacy_notify`——notify 已被归入「遗留」层，官方主推的是下面的 hooks 系统。
   一手源：`github.com/openai/codex` → `codex-rs/hooks/src/legacy_notify.rs`。**可信度：源码实测。**

4. **官方已明确「不打算把 notify 扩展到审批事件」。** 两个直接对口的 issue 均已关闭：
   - #11808 "Run `notify` hook for approval-request events (not only turn completion)" —— **Closed**。报告人原话："I rely on `notify` to send email alerts when Codex needs input. Right now I receive emails for agent responses, but not when Codex pauses waiting for approval."
   - #12524 "Notify hooks should fire when user input is requested" —— **Closed as not planned**。原话："When Codex enters an interactive prompt ... via `request_user_input`, no notify hook event is emitted, so external notifiers never trigger."
   - #3052 "Notifications for approval prompts, job completion, and file saves" —— **Closed**。
   一手源：`github.com/openai/codex/issues/11808`、`/12524`、`/3052`。**可信度：官方一手（issue 状态 + 引文）。**

   → **结论片段：`notify` 补不回审批信号，这是设计取向，不是暂时缺口。**

5. **补充：`notify` 的 spawn 是完全静默的。** 源码里对 notify 子进程设 `stdin(null).stdout(null).stderr(null)` 并 detach。所以 notify **确实满足「零 agent 负担」**——只是它只在 turn 完成时响，对「等审批」无用。（这条对 WhoNeedsMe 的用途：`agent-turn-complete` 可作为「Idle / 干净收尾」的辅助确认信号，但不是 Needs-You。）
   一手源：`codex-rs/hooks/src/legacy_notify.rs`（`Stdio::null()` 三连）。**可信度：源码实测。**

### B. hooks 系统的 `PermissionRequest`：这才是能补回审批信号的机制

6. **Codex 现在有一套完整的 lifecycle hooks 系统，独立于 `notify`。** 事件全集（源码里的 `HookEventName`，生成自 Rust 定义）：
   `preToolUse | permissionRequest | postToolUse | preCompact | postCompact | sessionStart | userPromptSubmit | subagentStart | subagentStop | stop`。
   配置位置（官方文档列出的四个实用位）：`~/.codex/hooks.json`、`~/.codex/config.toml`（inline `[hooks]`）、`<repo>/.codex/hooks.json`、`<repo>/.codex/config.toml`。**支持用户级全局配置**（`~/.codex/`），不需要每个 repo 单独装——这对 WhoNeedsMe「装一次全局生效」很关键。
   一手源：`codex-rs/app-server-protocol/schema/typescript/v2/HookEventName.ts`（源码）；官方 Hooks 文档 `learn.chatgpt.com/docs/hooks`、Advanced Configuration。**可信度：源码实测 + 官方一手。**

7. **`PermissionRequest` hook 就是「Codex 即将请求审批」时触发的那个事件。** 源码文件头注释原文："This event runs **in the approval path, before guardian or user approval UI is shown**." 官方文档："PermissionRequest runs when Codex is about to ask for approval, such as a shell escalation or managed-network approval. It does **not** trigger for commands requiring no approval."
   → 这正是我方要的语义：**只在「这条命令/补丁需要审批」时才响，不需要审批的命令不响**，比 `PreToolUse`（每次工具调用都响，噪声大）干净得多。
   一手源：`codex-rs/hooks/src/events/permission_request.rs`（源码注释）；官方 Hooks 文档。**可信度：源码实测 + 官方一手。**

8. **`PermissionRequest` 传给外部命令的 stdin JSON 字段（源码结构体 `PermissionRequestRequest`）：**
   `session_id`、`turn_id`、`cwd`、`model`、`permission_mode`（`default` / `acceptEdits` / `plan` / `dontAsk` / `bypassPermissions`）、`tool_name`（canonical，如 `Bash` / `apply_patch` / `mcp__server__tool`）、`tool_input`（含可选 `description`，人类可读的审批理由）、`transcript_path`、`hook_event_name`、`matcher_aliases`、`subagent`(可选)。
   够 WhoNeedsMe 用：能拿到 `session_id` + `cwd` 把信号绑回具体会话/终端，能拿到 `tool_name`/`description` 显示「在等批准什么」。
   一手源：`codex-rs/hooks/src/events/permission_request.rs`（`struct PermissionRequestRequest`）；官方 Hooks 文档（stdin 字段列表）。**可信度：源码实测 + 官方一手。**

9. **配置语法（`hooks.json` 或 inline `[hooks]`），matcher 按 `tool_name` 匹配：**
   ```json
   "PermissionRequest": [
     {
       "matcher": "Bash",
       "hooks": [{ "type": "command", "command": "/usr/bin/python3 permission_request.py" }]
     }
   ]
   ```
   `matcher` 可用 `Bash` / `apply_patch`（`apply_patch` 也匹配 `Edit`/`Write`）/ `mcp__server__tool`，或用宽匹配覆盖全部。
   一手源：官方 Hooks 文档。**可信度：官方一手。**

10. **关键：`PermissionRequest` hook 可以做到对 agent/model 上下文零注入。** hook 的返回值语义：
    - 返回 `allow` / `deny{message}` → 会**替 Codex 做审批决定**（deny 优先，any-deny-wins）。
    - **返回空 / 不匹配 → defer**，正常审批流照常走（用户照常看到审批框）。
    官方文档原文："output only affects behavior if JSON is returned; **no default injection into model context occurs**"；"Return nothing ... normal approval prompt proceeds"。
    → **对 WhoNeedsMe 的用法**：写一个只做副作用（往 WhoNeedsMe 发一个本机信号，如写文件/UDP/触发 IPC）然后 **exit 0、stdout 为空** 的 hook。它 **不返回决策 = 不干预审批流**，**不返回 systemMessage = 不往 model 上下文塞任何 token**。这满足我方「对 agent 认知负担为零」的硬约束。
    一手源：官方 Hooks 文档；`codex-rs/hooks/src/events/permission_request.rs`（decision 折叠逻辑：any deny wins / 无输出则 `decision: None`）。**可信度：官方一手 + 源码实测。**
    - 一点诚实的边角：源码把 hook 产物描述为 "transcript-visible output"，且 TUI 会渲染一行「hook 正在运行」的 UI row（`preview()`/hook rows）。这是**用户终端里的 UI 元素**，不是 model 上下文；对「零 token / 零 agent 认知负担」不构成违反，但会在用户 TUI 里留一个很小的可见痕迹。可信度：源码实测。

### C. 稳定性：已是文档化的正式功能，但仍在活跃演进；有一个已知语义缺口

11. **hooks 系统不是实验性 flag 门控的**（没搜到 `experimental_hooks` / `enable_hooks` 之类开关），靠配置存在即生效；官方文档把它当**正式功能**呈现，无 beta/experimental 标注。
    一手源：源码搜索（无 feature-flag）；官方 Hooks / Advanced 文档（无 experimental 标注）。**可信度：源码实测 + 官方一手。**

12. **时间线（源码 git 史，评估「变化多快」）：**
    - 2026-02-10 hooks 抽成独立 crate（#11311）——系统本身已存在约 5 个月。
    - **2026-04-17 `PermissionRequest` hook 落地（#17563）**——我方要的这个事件约 3 个月历史。
    - 之后持续加料：apply_patch 触发 hook（4-22）、MCP 工具支持（4-23）、compact 生命周期（5-07）、PreToolUse input 改写（5-12）、subagent 身份注入（5-21）。
    - 当前 release：`rust-v0.144.4`（2026-07-14，stable）+ `v0.145.0-alpha.*`。
    → **判断：核心 lifecycle（含 `PermissionRequest`）已稳定成形并文档化，但整套 hooks 仍在活跃加字段。我方 ADR 旧结论「Codex hooks 仍在快速变化」需更新为：主干已稳、边缘仍在扩展；`PermissionRequest` 的存在与 stdin 字段这一年内被移除的风险低，但字段可能继续增加。**
    一手源：`gh api repos/openai/codex/commits`（各 PR 日期）、`releases`。**可信度：源码实测（git 史）。**

13. **已知语义缺口（open issue，会影响误报率）：** #28833 "PermissionRequest hooks need a user-facing approval signal" —— **Open**。核心问题：`PermissionRequest` 在 **guardian / auto-review（"Approve for me"）路由决定之前**就触发，所以当用户配了自动审批器时，hook 会**在「其实不需要人」的情况下也响** → 对「正在等人」而言是**假阳**。报告人诉求是新增 `requires_user_input` 标志或单独的 `UserApprovalRequest` 事件。
    → **对 WhoNeedsMe 的影响**：在**默认场景（没配 auto-reviewer/guardian）**下，`PermissionRequest` 触发 ⟺ 用户会被弹审批框 = 干净的 Needs-You 信号。只有当用户自己配了自动审批器时才会假阳。可接受，且方向与「宁缺毋滥」一致（可在文档提示这一情形）。
    一手源：`github.com/openai/codex/issues/28833`。**可信度：官方一手（issue）。**

---

## 结论：Codex 的 Needs-You(审批) 信号能不能补回来，靠什么

**能补回，靠 hooks 系统的 `PermissionRequest` hook，不是靠 `notify`。**

- **`notify` 出局**：源码里事件枚举只有 `agent-turn-complete`，等审批时无任何代码路径触发；官方已把「notify 覆盖审批」的 issue 关闭（#11808 / #12524 as not planned）。`notify` 只能给我方提供「turn 干净结束 → Idle」的辅助信号。
- **`PermissionRequest` hook 命中需求**：它就在审批路径上、审批 UI 弹出之前触发；只对「需要审批的命令/补丁」响（不像 `PreToolUse` 每次工具都响）；stdin JSON 自带 `session_id` + `cwd` + `tool_name` + `description`，够把信号绑回具体会话并显示「在等批准什么」。
- **满足零认知负担硬约束**：hook 命令只做副作用后 `exit 0`、stdout 留空 → 不返回决策（不干预审批流）、不返回 systemMessage（不往 model 上下文注入任何 token）。唯一残留是用户 TUI 里一行极小的 hook UI row，非 model 上下文。
- **落地方式**：在 `~/.codex/hooks.json`（用户级、全局生效）注册一个宽 matcher 的 `PermissionRequest` command hook，指向 WhoNeedsMe 的 tiny signal emitter。装一次覆盖所有会话。
- **稳定性判断**：`PermissionRequest` 自 2026-04-17 落地、已官方文档化、非实验 flag、无移除迹象；整套 hooks 主干稳、边缘仍在加字段。旧 ADR「Codex hooks 仍在快速变化」应更新为「主干已足够稳，可依赖；预留字段可能新增」。

**这与 Claude Code 侧的结论对称**：两个 agent 的「等审批」都是纯文件判不出的硬边界，都靠各自的 hook 补回（Claude Code 的 `PreToolUse`/`Notification`，Codex 的 `PermissionRequest`），且都能做到静默、零 model 注入。→ 直接喂给决策票 #8（hook 事件契约）。

---

## 存疑 / 未证实

1. **`request_user_input` / plan-mode 澄清提问不被任何 hook 覆盖。** 现有 hook 事件全集里没有「agent 向用户提问」的事件；#12524（request_user_input）被 Closed as not planned、#13478（plan mode 等待用户回答的通知）仍是诉求。所以 `PermissionRequest` 只覆盖**命令/补丁审批**这一类「等你」，**不覆盖** agent 主动向用户提问式的等待。对本任务而言主目标（approval prompt）已覆盖，但「等你」并非只有审批一种——这一类目前 Codex 侧无干净 hook 信号。**可信度：官方一手（issue 状态）+ 源码事件枚举反证；「完全无覆盖」这点未在源码逐一确认 request_user_input 是否走 PreToolUse，标为待核实。**
2. **auto-review/guardian 场景下的假阳程度未实测。** #28833 描述了机制上会假阳，但我方未在本机实配 auto-reviewer 复现，量级未知。**可信度：官方一手（issue）；实测未做。**
3. **hook 命令的进程开销 / 触发延迟未实测。** 每次审批 spawn 一个进程；对高频审批场景的性能与是否阻塞审批 UI 未测。**未证实。**
4. **本文的官方文档取自 `learn.chatgpt.com/docs/...`**（`developers.openai.com/codex/...` 会 308 跳转到此域）。已与 `openai/codex` 源码交叉验证一致，但文档托管域近期迁移，URL 稳定性存疑。**可信度：官方一手，URL 可能再变。**

---

## 一手源清单

官方文档（OpenAI / ChatGPT Learn，`developers.openai.com/codex/*` 会跳到此）：
- Hooks（含 `PermissionRequest` 语义、stdin 字段、返回值/静默说明）：`https://learn.chatgpt.com/docs/hooks`（= `https://developers.openai.com/codex/hooks`）
- Advanced Configuration（`notify` 只支持 `agent-turn-complete`、hooks 配置位）：`https://learn.chatgpt.com/docs/config-file/config-advanced.md`（= `.../codex/config-advanced`）
- Configuration Reference（`notify` / `[tui].notifications` 键定义、hook 事件列表）：`https://learn.chatgpt.com/docs/config-file/config-reference.md`

`openai/codex` 源码（GitHub，`main` 分支）：
- `codex-rs/hooks/src/legacy_notify.rs` —— `notify` 事件枚举只有 `AgentTurnComplete`；静默 spawn（stdio null）
- `codex-rs/hooks/src/events/permission_request.rs` —— `PermissionRequest` 语义注释 + `struct PermissionRequestRequest` 字段 + allow/deny/defer 折叠
- `codex-rs/app-server-protocol/schema/typescript/v2/HookEventName.ts` —— hook 事件全集
- git 史：#11311（2026-02-10 hooks 独立 crate）、#17563（2026-04-17 PermissionRequest 落地）、#18391/#18385/#19905/#20527/#22882（后续演进）；release `rust-v0.144.4`（2026-07-14）

`openai/codex` issues（判定官方取向 / 缺口）：
- #11808 Run `notify` hook for approval-request events —— Closed
- #12524 Notify hooks should fire when user input is requested —— Closed as not planned
- #3052 Notifications for approval prompts, job completion, and file saves —— Closed
- #28833 PermissionRequest hooks need a user-facing approval signal —— Open（auto-review 假阳缺口）
- #13478 notification when plan mode is waiting for user answers —— 诉求（request_user_input 类缺口佐证）
