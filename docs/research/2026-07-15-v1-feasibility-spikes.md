# V1 地基可行性 spike：状态判定 + 终端聚焦跳转

- 日期：2026-07-15
- 状态：完成（两个真机 spike）；配套的官方 Agent View 护城河核实见 `2026-07-15-official-agent-view-scope.md`
- 背景：wayfinder 勘察识别出两个"技术地基未经验证、一旦不可行会直接 invalidate 核心 User Story"的点。本文用**本机真实数据**验证，把"未知可行性"转成"已知可行性 + 可精确陈述的决策"。
- 方法：只读分析本机 882 个 Claude Code JSONL 会话文件 + 运行中的 10 个 claude 进程与终端。未修改任何东西。

---

## Spike A：纯文件（无 hooks）能否可靠判定四态 + 等待原因三选一？

### 发现（有真机数据支撑）

1. **尾部结构过滤元数据噪声后是干净可分类的。** 原始最后一行常是 `bridge-session` / `mode` / `permission-mode` / `ai-title` 等元数据事件（这是"尾部看起来很乱"的来源）；但**只保留 `assistant` / `user` 对话事件**后，末条事件干净暴露：
   - `stop_reason = end_turn` 的 assistant message → turn 干净结束 = **Idle**（近 300 文件抽样中 247 个是这种，符合"历史会话多数正常收尾"）。
   - `stop_reason = tool_use` 且该 tool_use 无对应 `tool_result` → 进行中（Working 或 Waiting，见下）。
   - 每条事件都带 ISO `timestamp`，加文件 mtime 双保险 → **"距上次活动多久"可精确计算**，支撑超时/新鲜度启发式。

2. **Waiting-on-问题 / Waiting-on-计划确认：纯文件可靠判定。** 结构锚点明确 = `AskUserQuestion` / `ExitPlanMode` 的 `tool_use` 且**无对应 `tool_result`**。抽样 400 文件中，`AskUserQuestion` 出现 818 次、其中 817 次有后续 `tool_result`（历史会话都最终被回答），仅 1 次悬空——这证明"有没有 result"这个信号是干净的、可判定的。停在"无 result"快照 = 正在等用户。

3. **Waiting-on-权限批准：纯文件判不准，这是硬边界。** 权限等待在文件里的样子 = 一个普通工具（`Bash`/`Edit`…）的 `tool_use` 没有 `tool_result`——这和"该工具正在执行（Working）"的结构**完全一样**。文件里没有任何"正在弹权限框"的结构事件（`permission-mode` / `mode` 只是会话级配置元数据，不是每次工具的批准请求）。只能靠"工具挂了 N 秒还没 result → 猜它在等权限"的超时启发式，而这必然与"真的在跑一条长命令（build/test）"混淆。

### 对决策的影响

- **"宁缺毋滥"在四种情形下零 hook 即可兑现**：Working、Idle、等回答问题、等计划确认。
- **权限等待是 Claude Code 沉默 hook 唯一不可替代的价值**，而且这个 spike 精确界定了 hook 必须提供什么信号（`PreToolUse` / `Notification` 的"正在请求权限"事件），不是泛泛的"装了更准"。→ 直接喂给决策票 #8（hook 事件契约）。
- **US#29 "等待原因三选一"** 在纯文件下实际是"二选一（问题 / 计划确认）+ 权限批准需 hook"。
- **待拍板的决策 A**：零 hook 的 Claude Code 遇到"疑似在等权限"时如何降级？
  - (a) **漏报**——不点亮 Needs You。安全、守住"宁缺毋滥"，但会错过一整类真实等待（权限批准恰恰是并行开发里最高频的打断）。
  - (b) **超时猜**——挂 N 秒就点亮。可能误报 = 直接破坏核心承诺。
  - _我的建议_：默认 (a)，把权限类 Waiting 的点亮**绑定到装了 hook**——这正好让"装 hook"有一个用户能立刻感知的价值锚点，也和 ADR-0002"文件底座为主、hook 解锁 Needs You"的原意一致。

---

## Spike B：session → 终端窗口 → 前台聚焦，到底能不能做？

### 发现（有真机数据支撑）

1. **会话文件里没有 pid / tty / 终端标识，但 OS 层可以重建这条链。** 本机跑着 10 个 claude 进程，全部挂在真实 tty 上（ttys000/002/003…）；`ps` + `lsof` 能拿到每个进程的 tty 与 **cwd**。

2. **session ↔ 进程的匹配键只能用 cwd。** 会话文件有 cwd、进程也有 cwd，可对上；但**同一目录跑多个 session 会歧义**（不同 git worktree 因 cwd 不同则可区分，同目录开两个 claude 则分不清）。文件里的 `sessionId` 在进程侧拿不到，做不了唯一匹配。

3. **"聚焦到具体标签页"是可行的，但每个终端一套方言、覆盖面天然是 best-effort。**
   - **Terminal.app / iTerm2**：AppleScript 暴露 `tty of tab`/`tty of session` → 可用 tty **精确**定位并 select。最理想。
   - **Ghostty（你正在用的终端）**：有 `.sdef`、`NSAppleScriptEnabled=true`，暴露 `select tab` / `activate window` / `focus` 命令**可以切标签页**；但对象模型**只暴露 `working directory`、不暴露 tty** → 只能按 cwd 匹配，落回上面的同目录歧义问题。
   - **tmux / VS Code 内置终端 / Warp / WezTerm / kitty / Alacritty**：各不相同，部分完全不可脚本化（做不到定位标签页，最多把 App 带前台）。

4. **这引入一个当前架构里没有的新概念：终端聚焦 adapter。** 它和 ADR-0003 的"会话数据 Provider/Adapter"**正交**——一个管"从哪读状态"，一个管"点击后往哪个终端跳"。ADR-0004 把 focus-jump 划进 In Scope 时，没有意识到它需要这样一个独立的适配维度。

### 对决策的影响

- ADR-0004 的"单 session 点击 Alert → 聚焦对应终端窗口"（US#21/#24）的**真实可行性 = 分终端 best-effort + cwd 匹配可能歧义 + 部分终端根本做不到**，而不是一个统一能力。
- **待拍板的决策 B**：
  - (1) focus-jump 是否接受"仅支持的终端做精确跳转，其余优雅降级"（降级到：只把终端 App 带前台 / 或退化成打开 WhoNeedsMe 面板让用户自己点）？
  - (2) 要不要正式引入"终端聚焦 adapter"这个新架构维度（V1 先实现 Ghostty + Terminal.app 两个，其余降级）？这会是一条新 ADR。
  - (3) 同 cwd 多 session 的歧义怎么处理（V1 可接受"跳到该目录下任意一个"还是必须精确）？
  - _我的建议_：接受 best-effort + 优雅降级，并补一条 ADR 把"终端聚焦 adapter"和 Provider adapter 分开；V1 覆盖 Ghostty（你自己的终端）+ Terminal.app，iTerm2 顺带（同为 tty 精确）。歧义场景 V1 允许"跳到该 cwd 下最近活跃的那个"。

---

---

## Spike C：Codex rollout 文件能否达到和 Claude 同级的判定？（护城河命根）

> 追加于 2026-07-15。触发原因：`2026-07-15-official-agent-view-scope.md` 把"跨工具聚合"确立为官方结构性不会碰的独占内核，而 Codex 就是那个"跨"。Codex 状态判定做不到位，主打差异化塌一半。基于本机 566 个真实 Codex rollout 文件（`~/.codex/sessions/`）。

### 发现（有真机数据支撑）

1. **Working / Idle / Ended：可靠，且生命周期信号比 Claude 还干净。** Codex 显式写 `task_started` / `task_complete` 事件（不像 Claude 要从 `stop_reason` 推断）。抽样 60 文件中 59 个以 `task_complete` 收尾 = turn 干净结束（Idle/Ended 候选）；末事件是 `task_started` 且无后续 `task_complete` = 仍在进行或在等。另有 `turn_aborted`（被打断）信号。

2. **展示元数据：全部拿得到，个别比 Claude 还好。**
   - `cwd` ✓；**git branch ✓**（近 60 文件中 57 个含 branch → **ADR-0005 对 Codex 的 git.branch 断言成立**，我最初的顶层 grep 漏了，它是嵌套字段）；model 名 ✓（虽然某处是 `$ref` 压缩，但 `gpt-5.6-sol` / `gpt-5.5` 等纯字符串在文件里可直接抽到，US#27 的示例值就在数据里）。
   - **`model_context_window` 直接写在文件里（如 258400）** → 上下文占用率对 Codex **无需 model→window 映射**，比 Claude 那边省一层（勘察清单第 ⑩ 条对 Codex 不成立，对 Claude 仍成立）。

3. **入口区分：`originator` 字段。** 抽样 200 文件中 197 个是 `codex-tui`（终端 CLI，正是 WhoNeedsMe 的目标），3 个是 `Codex Desktop`（ChatGPT 桌面 App 内的 Codex，不住在可聚焦的终端窗口里）。→ Codex Adapter 应当 originator-aware：非 codex-tui 的入口要不要显示、能不能 focus-jump，是 adapter 设计决策。

4. **🔴 护城河命根的缺口：Codex 的"审批等待"不是持久化事件，纯文件判不出。** 精确搜索 150 个文件的 `event_msg.payload.type`，**审批请求（approval / exec / apply_patch / elicit 类）零命中**——Codex 的批准提示发生在实时 TUI 里，不写进 rollout 文件（文件里只有 config 里的 `approval_policy` 和批准后的结果）。所以"Codex 正在等你批准某条命令/补丁"在文件里长得和"该命令正在执行（Working）"一模一样，**和 Claude 的权限等待是同一个硬边界**。

### 对决策的影响（这条最重要）

- Codex 的**主要"Needs You"时刻恰恰是审批等待**（allow this command / apply this patch），而这正是纯文件判不出的那一类。
- **且 Codex 比 Claude 更糟一层**：ADR-0002/PRD 定了"V1 Codex 无 hook、纯文件解析"。Claude 至少能靠沉默 hook 补回权限信号，**Codex V1 连这个后备都没有**。→ Codex V1 能可靠显示 Working/Idle/Ended + 丰富元数据，但它的"谁在等我"在审批这一类上是**半盲**——而护城河偏偏压在 Codex 上。
- **待拍板的决策 C（新）**：护城河=跨工具，但 Codex 的核心 Needs-You（审批）文件不可见、V1 又无 hook。Codex V1 的 Needs-You 达标线定在哪？
  - (a) **best-effort**：Codex 只保证 Working/Idle/Ended + 元数据，审批类 Waiting 先不点亮（守宁缺毋滥，但 Codex 的"谁在等我"打折）。
  - (b) **验证并接入 Codex 的 `notify` 机制**：Codex config 有一个 `notify` 程序钩子（据称会在需要审批/任务完成时 spawn 一个外部程序并传 JSON）——如果它在"请求审批"时触发，这就是 Codex 侧等价于沉默 hook 的信号源，能把 Needs-You 补回来。**这是个需要一手核实的事实**（Codex 变化快，不能凭记忆断言），值得起一个 research。
  - (c) 超时启发式猜——同 Claude，有误报风险，破坏宁缺毋滥。
  - _我的倾向_：先做 (b) 的核实，因为它可能一举把 Codex 从"半盲"救成"和 Claude 同级"；若 `notify` 不覆盖审批，再退回 (a) 并明确写进 spec 的 scope 预期。

---

## 结论

- **状态判定的地基整体是稳的**——Claude 与 Codex 的 Working/Idle/Ended + 元数据都零 hook 可靠拿到；Codex 的生命周期信号甚至更干净、context window 还免了一层映射。
- **两处同源的硬边界**：Claude 与 Codex 的**权限/审批等待都在纯文件里不可见**。Claude 有沉默 hook 兜底，**Codex V1 目前没有兜底**——而护城河压在 Codex 上，所以 Codex 的 `notify` 机制值不值得接，从"锦上添花"变成了"可能决定护城河成色"。
- **终端聚焦不是"能/不能"的二元问题**，而是"分终端 best-effort + 需要一个新的 adapter 维度 + cwd 匹配有歧义"。ADR-0004 把它当统一能力划进 In Scope 偏乐观，进 to-spec 前需要一条新决策。
- 三条地基都从"未验证、可能推翻 US"变成了"可精确陈述、只等拍 scope 决策（A 权限降级 / B 终端聚焦 best-effort / C Codex Needs-You 达标线）"。
