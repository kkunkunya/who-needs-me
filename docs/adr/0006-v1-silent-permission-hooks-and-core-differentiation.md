# ADR-0006: V1 为 Claude/Codex 都接入沉默权限 hook，确立"语义化 Needs-You + 严格只读"为核心差异化

- status: accepted
- date: 2026-07-15
- amended: 2026-07-16（Hook 改为可选增强；补充可配对语义信号门槛）

## 背景

2026-07-15 的 wayfinding 会话做了三个本机真机 spike + 两份竞品/护城河 research，结论都落在 `docs/research/`：

- **官方 Agent View 护城河核实**（`2026-07-15-official-agent-view-scope.md`）：官方 Agent View 是真的、且做 needs-input 语义高亮，但只覆盖**手动 background 过的 Claude Code 会话**，明确不支持 Codex 等非 Anthropic 工具；GitHub #13024（waiting-for-input hook）仍 OPEN，官方没把语义信号开放给第三方。护城河变窄但没被推平。
- **Agent Island 竞品拆解（源码逐文件实测）**（`2026-07-15-competitor-agent-island.md`）：原先设想的四条差异化——零账号、不装 hook 直读 transcript、覆盖前台会话、离线——**Agent Island 已全部占据**（它同样 FSEvents 直读 `~/.claude/projects/*.jsonl` 与 `~/.codex/sessions`，不装 hook、不起服务）。真正还开着的只剩两条：① **语义化 Needs-You 粒度**——Agent Island 只有布尔"轮次完成"+ provider 健康态，源码上**分不清"agent 卡在权限/审批上等你拍板"与"跑死了"**；② **严格只读**——Agent Island 用 `--dangerously-skip-permissions` 无人值守自动续跑，主动越过了只读线。
- **状态判定 spike A/C**（`2026-07-15-v1-feasibility-spikes.md`）：四态与展示元数据（cwd/分支/模型/上下文占用/等待原因中的"问题、计划确认"）纯文件可靠判定；但**权限/审批等待纯文件不可判**——它在文件里和"工具正在执行(Working)"结构完全一样，Claude 与 Codex 是同一堵墙。而这堵墙正是差异化①的关键。
- **Codex notify 核实**（`2026-07-15-codex-notify-signal.md`）：Codex 的 `notify` 只在 turn 完成时触发、不覆盖审批（官方把"让 notify 覆盖审批"的 issue 关为 not planned）；**但 Codex 有稳定、官方文档化的 `PermissionRequest` hook**（2026-04-17 落地，`~/.codex/hooks.json` 用户级配一次全局生效），在审批路径上、审批 UI 弹出前触发，只对需审批的命令/补丁响，stdin JSON 自带 `session_id`/`cwd`/`tool_name`/`description`，且可做到只副作用后 `exit 0`、stdout 留空、不返回决策、不注入 token。

用户在本会话明确拍板：**给 Claude 和 Codex 各装一个 hook 是可接受的（"很多类型的产品都走这样的 hook，只要不乱注入提示词都没问题"）。** 2026-07-16 对照 cc-connect 的 Supervisor（进程托管者）方案后进一步收口：WhoNeedsMe 保持 Observer（旁观者），Hook 可以推荐安装，但不得成为产品可用的前置条件；该边界以 ADR-0002 的修订为准。

## 决策

1. **V1 为 Claude 与 Codex 都提供可选的沉默权限 hook**——Claude 用 `PreToolUse` / `Notification`，Codex 用 `PermissionRequest`（`~/.codex/hooks.json`）。两者都遵守 ADR-0002 的"沉默 hook"硬约束（只追加写本地事件、`exit 0`、不向 Agent 上下文输出、不返回审批决策、不干预原生审批流）。Codex hook 主干（含 `PermissionRequest`）可依赖，边缘字段仍需兼容演进。
2. **Hook 是推荐安装的精度增强，不是产品可用的前置条件**：它是"权限/审批语义 Needs-You"的使能器；零安装文件底座仍可提供会话列表、四态、以及"等回答问题 / 等计划确认"两类 Waiting。只有"权限/审批类 Needs-You"绑定到 Hook；不装则该类不点亮，守住"宁缺毋滥"，不用超时启发式猜。Hook 不改变 WhoNeedsMe 的 Observer 边界，也不负责启动或托管 Agent CLI。（ADR-0002）
3. **确立两条核心差异化，其余降级**：① **语义化 Needs-You**——区分"权限/审批 vs 回答问题 vs 计划确认"，而非布尔"该你了"；② **严格只读信任定位**——绝不替用户操作 agent（与 ADR-0004 的只读边界一致）。零账号、不装 hook 也能用、前台覆盖、离线、跨工具可插拔这五条**不再作为对 Agent Island 的差异化主张**，降级为"合格线 / 加速度"。
4. **纯文件语义 Waiting 必须可配对**：`AskUserQuestion` / `ExitPlanMode` 只有在 `tool_use.id` 存在、能够与后续 `tool_result` 唯一配对时，才可作为 Waiting 的直接信号。缺少 id 时无法证明该请求仍未处理，必须退回 Adapter 的标准状态折叠（当前事件形态为 Working），`waiting_reason` 留空且 Needs You 不点亮。

## 理由

- 用户的硬约束（不乱注入提示词）与 ADR-0002 的"沉默 hook"原则完全一致，装 hook 不违背项目哲学；而两份 research 表明，不装 hook 就拿不到"权限/审批 Needs-You"，而这恰是唯一能对 Agent Island 形成结构性优势的功能。
- Agent Island 拆解证明"直读 transcript、零账号、前台覆盖"已是这个细分的**桌面赌注（table stakes）**，不是卖点。把叙事上移到"为什么在等你 + 绝不碰你的 agent"，才落在竞品结构上够不到的地方。
- "等回答问题 / 等计划确认"纯文件即可判（Spike A：`AskUserQuestion`/`ExitPlanMode` 无 tool_result），意味着即使一个 hook 都不装，WhoNeedsMe 在这两类上已经赢 Agent Island 一个身位（后者把它们也归进 busy/stalled）。hook 补齐的是第三类（权限/审批），把差异化①做满。
- 被否备选：维持"Codex 无 hook 纯文件"——被否是因为 Codex 是"跨工具聚合"里的那个"跨"，而它的主要 Needs-You 时刻恰是审批；纯文件让 Codex 的核心价值半盲，等于自废唯一护城河的一半，而 `PermissionRequest` hook 已被核实为稳定可依赖，没有理由不接。

## 后果

- PRD（Issue #2）需修订：实现决策里"V1 沉默 hook 只接 Claude Code" → "Claude 与 Codex 均接沉默权限 hook（Claude `PreToolUse`/`Notification`、Codex `PermissionRequest`）"；Codex Adapter 的"退化为状态未知"降级路径相应收窄到"未装 hook 时权限/审批类不点亮"。
- hooks 安装器是可选增强的入口，范围为"Claude + Codex 各一份沉默 hook 的一键安装/检查/修复"；未安装时基础产品仍可用。（ADR-0002）
- ~~残留：Codex 的"agent 主动提问"可判性未验证~~ **（2026-07-15 grilling 已核实解除）**：Codex 的 `request_user_input` 是 rollout 文件里的工具调用（150 文件抽样中 45 次），`update_plan` 也在（13 文件），故"Codex agent 在问你问题 / 等计划"**纯文件可判**，与 Claude 的 `AskUserQuestion`/`ExitPlanMode` 对称。结论：**Codex 现在四态 + 等待原因三类（问题/计划文件可判、审批走 `PermissionRequest` hook）与 Claude 齐平**，不再是半盲。
- **差异化① 的实现要求**：Adapter 必须真能从 JSONL + hook 事件里解出"权限/审批 vs 问题 vs 计划"三分——这是 WhoNeedsMe 要啃下的核心工程活，不是白捡的信号。
- 语义工具 fixture 与 contract test 必须覆盖缺少 `tool_use.id` 的异常事件，防止出现 `Needs You=true` 但 `waiting_reason=None` 的不完整状态。
- 终端聚焦跳转（ADR-0004 划进 In Scope）经 Spike B 判定为"分终端 best-effort + 需要独立的『终端聚焦 adapter』维度 + cwd 匹配歧义"，需要单独一条 ADR 收口（本会话的决策 B，仍未拍），本 ADR 不覆盖。
- Agent Island 有一个可被验证的对外弱点（等权限时被误判为 working/stalled），若日后要做对比传播，建议先本机复现坐实（见竞品拆解 §五"源码推断"一条）。
