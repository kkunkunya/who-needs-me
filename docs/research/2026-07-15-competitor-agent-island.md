# 竞品拆解：Agent Island（头号正面竞品）

- 日期：2026-07-15
- 竞品：Agent Island（GitHub `tristan666666/agent-island`，官网 agent-island.dev，作者署名 Eric Park）
- 我方：WhoNeedsMe（本机 coding agent CLI 会话的只读状态托盘应用，卖点"谁在等我 Needs You 高亮"）

## 决策问题

WhoNeedsMe 前序把结构性差异化内核收敛为三条：① 跨工具聚合（可插拔 Adapter）；② 零账号 / 离线 / 本地只读（不联网、不起 HTTP、不装 hook 也能看会话列表）；③ 覆盖前台交互式会话。本报告要回答：**这三条里，哪几条 Agent Island 已经堵死、哪几条 WhoNeedsMe 还真能赢。** 结论先行：三条里有两条被 Agent Island 大面积占据，真正还开着的口子比原设想窄——但存在，且其中一条相当锋利。

证据强度标注：`官方一手`（README/官网/release notes）｜`源码实测`（我读了仓库源码）｜`二手转述`｜`未能证实`。

---

## 一、Agent Island 做了什么

### 1.1 定位与呈现形态

- 定位：**"Claude Code 和 Codex 的状态伴侣——常驻你的刘海"**（官方一手，README/官网 hero）。它同时是"状态监视器 + 用量/额度追踪 + 分享战报卡 + 自动续跑器"，比 WhoNeedsMe 的纯状态板更重。
- 呈现形态（多形态，非单一）：
  - macOS 有刘海机型：**刘海（notch）内嵌 pill**；(官方一手 + `源码实测` `Sources/Model/NotchInfo.swift`)
  - macOS 无刘海机型：`NotchInfo.detect` 自动回退到**菜单栏（menu bar）高度的 pill**，不是硬锁刘海；(源码实测)
  - Windows：**顶栏 pill / 可拖拽悬浮 widget + 托盘图标（带用量环）**；(官方一手 release notes + `源码实测` `windows/src/AgentIsland/UI/TrayIcon*.cs`)
  - "轮到你了"时：**前台弹窗 alarm window + 系统通知 + 循环响铃**，直到 dismiss；(官方一手 + 源码实测 `Sources/Model/TurnAlarmWindowController.swift`)
  - 无 Dock 图标。
- 所以"刘海/顶栏/托盘/弹窗"它**四种都有**：刘海是品牌主视觉，托盘/顶栏是无刘海与 Windows 的落地，弹窗是"轮到你了"的强提醒。

### 1.2 核心功能清单（官方一手，除非另注）

1. **实时会话状态**：logo 转动=正在跑，静止=轮次结束，红色脉冲=告警。
2. **状态机（源码实测 `docs/how-agent-island-detects-session-state.md` + `SessionScanner.swift`）**：每个 provider 聚合成一个最紧急态，优先级 `idle < working < your turn(needsYou) < stalled < rate limited < auth required`。注意它有一个 **stalled（跑到一半卡死 5 分钟）** 态和 **rate limited / auth required** 两个 provider 健康态。
3. **"It's your turn" 提醒**：后台/前台会话轮次结束时前台弹窗+通知+响铃，支持多会话排队。
4. **用量与成本追踪**：Claude 5 小时 + 每周额度；Codex 每周额度（现已并为单一周额度）。成本估算、重置倒计时。**调用 provider 官方用量 API**（联网，见 §2.3）。
5. **额度耗尽告警**：与"轮到你了"分开的独立 quota alarm，100% 时响并显示重置时间。
6. **战报卡（v1.6.1+）**：周/月可分享卡片，总 token、"≈ API 价值"、Claude/Codex 占比、TOP-5 模型甜甜圈图、24 周活跃热力图 + streak、**"岛民段位"7 级排名**（Drifter→Legendary Navigator）。明显的社交传播/拟人化玩法。
7. **自动续跑（auto-resume / trigger，源码实测，重要）**：见 §2.4——它**不是只读**。
8. 5 种图表样式；EN + 简体中文双语；Sparkle（macOS）/ GitHub Releases（Windows）自动更新。

### 1.3 支持哪些 agent 工具，各到什么程度

| 工具 | 状态监测 | 用量/额度 | 自动续跑 | 证据 |
|---|---|---|---|---|
| **Claude Code** | ✅ 完整 | ✅ 5h+周 | ✅ `claude --resume` | 源码实测 |
| **Codex (OpenAI)** | ✅ 完整 | ✅ 周 | ❌ **已下线**（2026-07-13，因 Codex 取消了 5 小时窗口） | 源码实测 `TriggerEngine.fire()` 注释 |
| **Claude Desktop** | 仅借它的 session store 拿会话标题/归档标记 | — | — | 源码实测 `SessionScanner.swift` |

**关键**：支持的工具是**硬编码的 `enum TriggerTool { claude, codex }`**（源码实测 `Sources/Trigger/Trigger.swift`），不是可插拔 Adapter 架构。加一个 Gemini CLI / OpenCode 需要改 enum + `SessionTurnState` + `SessionScanner` 三处源码。

---

## 二、它怎么拿到状态（状态采集机制）

### 2.1 靠文件监听，不装 hook，不起服务（源码实测，高置信）

- 直接读工具**自己已经在写**的本地 JSONL transcript：
  - `~/.claude/projects/*.jsonl`（Claude 主会话）
  - `~/.codex/sessions/YYYY/MM/DD/*.jsonl`（Codex）
  - `~/Library/Application Support/Claude/claude-code-sessions/local_*.json`（仅取标题/归档标记）
- 监听方式：**macOS FSEvents / Windows FileSystemWatcher**（源码实测 `TranscriptEventStream.swift` / `.cs`）+ 每 **6 秒**兜底轮询扫描。读文件时只 tail 末尾 128KB / 200 行。
- **不需要用户装任何 hook，不改工作流，不起本地 HTTP 服务，不需要注册 Agent Island 账号。** 仓库全局搜 "hook" 只有 6 处命中，全是代码 lifecycle hook，无一是 Claude Code hook 机制。(源码实测)

> 结论：Agent Island 拿会话状态的路径，和 WhoNeedsMe 设想的"直接读 Provider 已在写的本地会话数据、不装 hook"**几乎完全一致**。这条不是 WhoNeedsMe 独有。

### 2.2 它能不能判"waiting for input"？——只有布尔轮次完成，判不了"在等你批权限"（源码实测 + 源码推断）

这是最要害的一点，需要精确表述：

- 它的核心状态是 `SessionTurnStatus.isDone: Bool`（源码实测 `SessionTurnState.swift`）。
- Claude 判 done 的依据：最后一条非 sidechain 的 `type:assistant` 且 `stop_reason ∈ {end_turn, stop_sequence, stop}`，且不是 `isApiErrorMessage`。Codex：`payload.type ∈ {task_complete, turn/completed}`。
- **"轮到你了" = 助手把整个轮次跑完停下了**，不是"agent 卡在一个权限/审批弹窗上等你点同意"。
- `源码推断`（高置信，未做运行时实测）：Claude Code 在工具调用处等待用户批准时，那条 assistant 消息的 `stop_reason` 是 `tool_use`，**不在**它的 done 集合里。于是"正在等你批权限"的会话会被判成 `working`→（18s 无写入后仍 working，到 5 分钟）→ `stalled(卡死)`，**永远不会**升成"你的回合"。也就是说它**分不清"在等你拍板"和"跑死了"**——两者都塌缩成 working/stalled。
- 它有的"原因"只有 **provider 健康类**（rate limited / auth required / stalled 卡死），**没有轮次级的"为什么在等你"语义**。

### 2.3 用量/额度这块是联网的（源码实测，纠正"全离线"印象）

- 状态监测（§2.1）确实全离线、纯本地读文件。
- 但**用量/额度追踪要联网**：`UsageFetcher` 调 provider 官方用量 API，复用你本地已有的凭证（`~/.claude`、`~/.codex`），还内置浏览器重新登录流程（`ClaudeWebLogin.swift`）。官方措辞"全程在本机以你的身份运行，不上传任何东西"——指的是不往它自己的服务器传数据，**不等于不联网**。(源码实测 + 官方一手)
- 所以：**零 Agent Island 账号=真**（复用本地凭证，不注册）；**全程离线=假**（额度功能会打 provider API）。

### 2.4 它不是只读——会替你跑 agent（源码实测，重要差异点）

- `TriggerEngine.fire()` 会 **spawn 一个真实进程**：
  - Claude：`claude --resume <id> -p <message> --dangerously-skip-permissions`
  - Codex（已下线）：`codex exec resume <id> <msg> --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check`
- 触发条件：额度窗口重置时（`afterReset`）或每 N 小时（`everyHours`），自动把指定会话用 **`--dangerously-skip-permissions`** 续跑（默认消息 "Continue"）。
- 有安全闸：`executionEnabled`（默认开）+ per-project 信任 allowlist（**默认空**，必须显式把项目加白）+ 只在 normal 模式跑 + Codex 已整条下线。所以**不设置就不会自己动**，但一旦用户开了 trigger + 信任项目，它就是在**无人值守、跳过所有权限确认**地替你写代码/跑命令。作者自己在注释里把"周额度边界无人值守续跑"称为 **"a budget hazard"**。

---

## 三、逐条对比 WhoNeedsMe 三条差异化内核

图例：**已占据** = Agent Island 已完整做到，WhoNeedsMe 在这点上没有结构性优势；**部分占据** = 沾边但有明显空隙；**没做** = Agent Island 明确没做，是 WhoNeedsMe 的空地。

| WhoNeedsMe 差异化内核 | Agent Island 现状 | 判定 | 说明与证据 |
|---|---|---|---|
| **① 跨工具聚合 · 可插拔 Adapter** | 聚合 Claude Code + Codex 两工具，但 `enum` 硬编码，非插件架构 | **部分占据** | 当前覆盖它已做（源码实测）；"可插拔/易扩展"是 WNM 的差异，但属**架构差异**非"能看到的功能"，且更广市场（X Island/Ping Island/CodeIsland 已覆盖 4–13 个工具）在稀释"跨工具"卖点 |
| **②a 零账号** | 复用本地凭证，无需注册 Agent Island 账号 | **已占据** | 官方一手 + 源码实测。**不是差异点** |
| **②b 离线 / 不联网** | 状态监测全离线；**用量/额度功能联网**打 provider API | **部分占据** | WNM 的"全程不碰网"只有在**放弃用量/额度功能**时才成立；对"只看会话列表"这个用例，Agent Island 也是离线的 |
| **②c 不装 hook 也能看会话列表** | 同样直读 transcript，**不装 hook、不起服务** | **已占据** | 源码实测。**这条被完全堵死**，Agent Island 和 WNM 机制几乎相同 |
| **②d 本地只读（不替你操作）** | **不是只读**：`--dangerously-skip-permissions` 自动续跑 | **没做（反向）** | 源码实测。**WNM 真正能赢的一条**：严格只读=信任/安全定位，Agent Island 主动放弃了 |
| **③ 覆盖前台交互式会话** | 基于 transcript 检测，前台会话同样覆盖，无需手动 background | **已占据** | 源码实测（检测不区分前台/后台，读的是活的 transcript）。**不是差异点** |
| **（衍生）"谁在等我"语义粒度** | 只有布尔"轮次完成"+ provider 健康态；**分不清"等你批权限"与"卡死"** | **没做** | 源码实测 + 源码推断。**WNM 最锋利的空地**（见 §4） |

---

## 四、它的缝（seam）/ WhoNeedsMe 还能赢的点

按"锋利度"排序：

1. **【最锋利】"等待原因"的语义粒度是空的。** Agent Island 的"轮到你了"= 整轮跑完停下，它**判不了"agent 此刻卡在权限/审批/提问上等你拍板"**（会误判成 working/stalled）。WhoNeedsMe 设想的"区分权限 / 问题 / 计划"的 Needs-You 语义，是 Agent Island 结构上没占的地。**前提**：WNM 得真的能从同一批 transcript 里把 `stop_reason:tool_use` 挂起、plan-mode 待确认、AskUser 类提问识别出来——技术上可行（信号就在 JSONL 里），但不是白捡，是 WNM 要啃下来的活。

2. **【干净】严格只读 = 信任定位。** Agent Island 跨过了只读线（无人值守 `--dangerously-skip-permissions` 续跑，作者自评"budget hazard"）。WhoNeedsMe"只看不动、绝不替你操作"是一个 Agent Island 主动放弃、且**无法在不改定位的前提下夺回**的位置——对在意安全/审计的用户是清晰卖点。

3. **【中等，市场竞争】可插拔 Adapter 架构。** Agent Island 两工具硬编码。WNM 的插件化在"加第三第四个工具"时更快。但注意这是架构优势而非即时功能优势，且更广赛道已有多工具竞品在抢"跨工具"叙事——**别把它当护城河，当加速度**。

4. **【定位/审美，非能力】去刘海、去拟人化。** Agent Island 整个品牌是"刘海剧场"+ 岛民段位/热力图等拟人化社交玩法。它技术上**并没有硬锁刘海**（无刘海 Mac 回退菜单栏、外接屏有 target-display 设置、Windows 走顶栏/托盘——`源码实测`）。所以"锁死刘海"**不是**一个能攻的技术缝；WNM 刻意的"托盘 + 朴素 Alert、不做拟人化"是**审美/品味的差异化**，对反感花哨的用户有效，但不构成功能护城河。

5. **【弱】维护面 / 成熟度。** 仓库 2026-06-17 建（约 4 周），45 star / 5 fork / 13 open issue（多为本地化与文档整理类 good-first-issue，社区氛围健康但小），单一维护者（tristan666666 213 提交 + `claude` bot 26 提交，**高度 AI 代写**），日更活跃。bus factor=1、非常年轻——不是 WNM 的直接优势，但意味着定位战窗口还开着。

---

## 五、未能证实 / 存疑

- **"目前唯一的 Windows 同类实现"**：`未能证实`。当前 EN/中文 README **均未出现**"唯一/首个 Windows"字样，只说"一个应用，macOS 和 Windows 原生"。仓库近期有多个提交在清理"retired-feature copy"、并有 issue #15"给退役功能声明加公开文案守卫"——说明他们在**主动下架过度宣称**。Windows 原生支持本身为真（`windows/` 目录、WPF、CI workflow 均源码实测），但"唯一"这个最高级在现存一手源里查无实据，疑为前序调研看到的旧文案。
- **是否签名/公证**：`未能证实`。未找到公证/签名说明；分发走 brew tap / DMG（macOS）与 Scoop/winget/ZIP（Windows）。是否 notarized 未知。
- **"等待权限时被误判为 stalled"**：`源码推断`（高置信），基于 `stop_reason:tool_use` 不在 done 集合的代码事实，**未做运行时实测**。若要作为对外打击点，建议 WNM 侧真机复现一次 Claude Code 权限弹窗，确认 Agent Island 当时显示的是 working/stalled 而非"你的回合"。
- **Windows 端是否也有 auto-resume**：`部分证实`。`windows/.../Trigger/TriggerEngine.cs` 存在，机制应一致，但未逐行核对其 spawn 命令与安全闸。

---

## 六、一手源清单

- 仓库：https://github.com/tristan666666/agent-island （`官方一手` + `源码实测`）
  - `README.md` / `README.zh-CN.md`（hero、功能、安装、隐私声明）
  - `docs/how-agent-island-detects-session-state.md`（官方状态检测机制自述）
  - `Sources/Trigger/SessionTurnState.swift`（轮次完成判定：`stop_reason`/`task_complete`）
  - `Sources/Trigger/SessionScanner.swift`（transcript 路径、阈值、状态聚合）
  - `Sources/Trigger/TranscriptEventStream.swift` / `windows/.../Core/TranscriptEventStream.cs`（FSEvents / FileSystemWatcher，无 hook）
  - `Sources/Trigger/Trigger.swift`（`enum TriggerTool` 硬编码 claude/codex；`CLILocator`）
  - `Sources/Trigger/TriggerEngine.swift`（`--dangerously-skip-permissions` 自动续跑，非只读）
  - `Sources/Trigger/TriggerSafetyStore.swift`（执行开关默认开、信任 allowlist 默认空）
  - `Sources/Model/NotchInfo.swift`（无刘海回退菜单栏，未硬锁刘海）
  - `Sources/Usage/UsageFetcher.swift` / `ClaudeWebLogin.swift`（用量走 provider API，联网 + 浏览器重登）
  - GitHub API：45 star / 5 fork / 13 open issue / 建于 2026-06-17 / 末次 push 2026-07-14 / MIT / 主语言 C#（Windows WPF 体量）+ Swift（macOS）
  - Release notes v1.4.1–v1.6.1（功能演进、Codex auto-resume 下线记录）
- 官网：https://agent-island.dev （`官方一手`，与 README 一致）
- 更广赛道对照（`二手转述`，来自 web search，未逐一核验）：X Island（xisland.app，称支持 Claude Code/Codex/Gemini CLI/OpenCode）、Ping Island（`erha19/ping-island`，hook 驱动多工具）、CodeIsland（`wxtsky/CodeIsland`，Unix socket 连 13 工具）——说明"刘海/Dynamic Island 监视器 + 跨工具"这一形态本身赛道拥挤。
