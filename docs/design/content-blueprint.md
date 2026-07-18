# 内容蓝图 — WhoNeedsMe V1 托盘界面

> status: accepted
> 决策负责人: Kun
> 产出日期: 2026-07-16
> 配对阅读: `docs/design/design-reference.md`（视觉 token；待 `/design-recon` 落盘）
> 产出自 `/layout-priority-design`。视觉锚定交给 `/design-recon`，视觉纪律交给 `/anti-slop-frontend`。

## 跨界面内容规则

- 产品级 core：**一眼发现谁在等你，再由你亲自回到终端处理；WhoNeedsMe 绝不替你操作 Agent。**
- Session Panel 是 Tray 点击后的紧凑 macOS 菜单栏 popover：先给当前结论，再给可行动对象，最后才放设置与退出。
- 所有 Waiting Session 跨项目全局置顶；Waiting 区中的每一行都明确显示所属项目组。其他 Session 才按项目组归拢。
- 项目组自动按 Git 仓库 / worktree 家族识别；同一仓库的 `main`、`test` 等 worktree 属于同组。非 Git 目录退化为 normalized cwd。
- 项目组默认名取仓库名或文件夹名。用户可设置本地显示别名；别名不修改文件夹、Git remote、worktree 或分支。
- Hook 是 Settings 中的可选增强，不在 Panel、Tray 或 Alert 上显示 badge、警告、健康状态或安装催促。
- 所有动作只允许：打开 Panel、best-effort 前置终端、修改 WhoNeedsMe 本地显示名、刷新本地状态、安装可选沉默 Hook、退出 App。不得批准、回复、发送或继续 Agent。
- `View all sessions` 不存在：Session Panel 已经呈现全部在榜 Session，没有第二个“全部列表”目的地。

## 界面:Session Panel

### §1 core

- 服务谁：同时运行多个 Claude Code / Codex CLI Session、正在别的窗口工作，并在“我不知道哪个 Agent 已停下来等我”这一时刻点击托盘的开发者。
- 促成什么动作：看清全局后，点击一条 Waiting Session，best-effort 回到对应终端亲自处理。
- core（一眼价值）：**一眼看清谁在等你、等什么、等了多久。**

### §2 优先序列表

| 排位 | 内容模块 | 排位依据 |
|---:|---|---|
| 1 | 聚合结论：`{N} sessions need you` / `No sessions need you` / `No active sessions` | 直接回答产品 core：“现在有没有人等我”。 |
| 2 | 全局 Waiting Session 行 | 直接承载主动作；所有 Waiting 必须跨项目高于任何 Working / Idle / Ended。 |
| 3 | Waiting 行的项目组名、分支、Provider、等待原因与 Elapsed | 让用户在点击前知道“哪个项目、哪条分支、等什么、多久”，避免盲跳终端。 |
| 4 | Working Session 行 | 补全仍在运行的工作全局，但当前不要求用户行动。 |
| 5 | Idle Session 行 | 帮用户确认哪些 Session 已结束当前 turn，优先级低于仍在工作的 Session。 |
| 6 | Recently Ended Session 行 | 只提供短暂结束反馈，随后退出在榜列表，不能挤占活跃工作。 |
| 7 | 非 Waiting Session 的项目组标题与本地别名 | 帮用户把同一 Git 仓库 / worktree 家族放回一个工作上下文中理解。 |
| 8 | Empty State | 无在榜 Session 时替代列表，说明 App 正常而非加载失败。 |
| 9 | 刷新时间 / 可恢复错误 | 消除“数据是不是旧的”这一信任顾虑；正常时必须弱于 Session 内容。 |
| 10 | 项目组轻量菜单：Rename / Reset | 服务识别效率，但不应抢占“谁在等我”的主路径。 |
| 11 | `Settings…` | 通向可选增强与其他次级配置，不在主列表内展示 Hook 状态。 |
| 12 | 快捷键提示 | 只服务熟练用户，不能占据首要注意力。 |
| 13 | `Quit WhoNeedsMe` | 托盘常驻 App 的必要退出路径，但与本次状态判断无关，排最后。 |

### §3 区域映射与重点落位

| 区域 | 放什么 | 唯一职责 | 优先级 | 次要让位去向 |
|---|---|---|---|---|
| Header | 动态聚合结论；Session 总量作为辅助信息 | 让用户第一眼知道此刻是否有人需要自己 | P1 | 快捷键与 Settings 不与结论同级，收进 Header 末端或 Footer |
| Needs You | 所有 Waiting Session 跨项目全局置顶；每行含项目组名、分支、Provider、等待原因、Elapsed | 让用户选择下一条要亲自处理的 Waiting Session | P1 | 非 Waiting Session 全部下推到 Other Sessions |
| Other Sessions | 按项目组归拢 Working、Idle、Recently Ended；组内按 Working → Idle → Ended | 在不干扰 Waiting 的前提下补全当前工作全局 | P2 | Ended 只短暂出现；长列表内部滚动，不把 Header / Needs You 推出首屏 |
| Empty 变体 | `No active sessions` + 启动 Provider 后会自动出现的说明 | 在列表为空时证明 App 正常且告诉用户如何产生内容 | P1 | 替代 Needs You 与 Other Sessions，不与它们同时出现 |
| Error 变体 | 上次成功状态时间或无法加载说明 + `Try again` | 诚实说明数据新鲜度并提供恢复入口 | P1/P2 | 有旧数据时保留旧列表；技术详情不进主 Panel |
| Footer | `Updated just now`、`Settings…`、快捷键、`Quit WhoNeedsMe` | 承载低频管理动作和数据新鲜度 | P3 | Hook 健康与错误详情只在 Settings；不存在 `View all sessions` |

内容规则：

- Header 文案：有 Waiting 时 `{N} sessions need you`；有活跃 Session 但无人 Waiting 时 `No sessions need you`；完全为空时 `No active sessions`。
- Waiting 原因固定为 `Waiting for your approval`、`Waiting for your answer`、`Waiting for plan confirmation`。
- Waiting 区不按项目组把非 Waiting 兄弟 Session 一并带上来；“所有 Waiting 全局置顶”高于项目组完整展示。
- Other Sessions 按项目组显示。自动组名取仓库名 / 文件夹名；自定义别名优先显示。
- 项目组标题的轻量菜单只有 `Rename project…` 与 `Reset to detected name`。
- Empty 说明：`Start Claude Code or Codex in a terminal and it will appear here.`
- 有旧数据的刷新失败：`Couldn't refresh · Showing status from {time}`；无旧数据：`Couldn't load sessions`。

### §4 行动引导

| CTA（动作） | 价值主张（为什么值得点） | 点击前顾虑及消除 |
|---|---|---|
| 点击 Waiting Session；无障碍名称 `Open {project} in terminal` | 从已经看清的项目、原因与等待时长直接接近对应终端 | 导航是 best-effort；动作只前置终端，不发送命令、不批准、不回复。 |
| `Rename project…` | 用自己熟悉的名字识别同一仓库下的 worktree 与 Session | 输入旁说明 `Only changes the name shown in WhoNeedsMe.`；不修改文件夹或 Git。 |
| `Reset to detected name` | 随时撤销本地别名，回到自动识别结果 | 只删除本地别名，不改变 Session 或仓库。 |
| `Try again` | 重新读取本地 Session 状态，恢复当前视图 | 有旧数据时继续显示旧快照，避免刷新失败造成“全部消失”的误解。 |
| `Settings…` | 进入低频配置与可选增强，不干扰快速状态判断 | Panel 不用警告或 badge 催促安装 Hook。 |
| `Quit WhoNeedsMe` | 明确结束常驻托盘 App | 退出只结束 WhoNeedsMe，不结束或控制任何 Agent CLI。 |

### §5 蓝图自检

- [x] core 一句话清晰
- [x] 优先序列表每条有可追溯依据
- [x] 每个区域只承担一个目标，次要已收敛让位
- [x] 首屏承载 P1
- [x] 每个 CTA 意图清晰、绑 core，点击前主要顾虑已消除（行动引导在内容层）
- [x] 无“都要→都平均”区域

## 界面:Permission Reminders Settings

### §1 core

- 服务谁：已进入 Settings、希望补上“等待权限批准”提醒，但不想 Hook 干扰 Agent、占用 token 或成为基础功能前置条件的用户。
- 促成什么动作：在理解它是可选、本地、沉默的前提下，快速安装或修复 Claude 与 Codex 的 Permission reminders。
- core（一眼价值）：**可选安装本地沉默 Hook，补上权限等待提醒；不安装也不影响基础监控。**

### §2 优先序列表

| 排位 | 内容模块 | 排位依据 |
|---:|---|---|
| 1 | `Permission reminders (optional)` + 一句话用途 | 先说明这是什么以及“可选”，避免把增强能力误解成故障修复。 |
| 2 | 安装 / 修复动作 | 直接服务本界面 core，让用户无需手工拼 Provider 配置。 |
| 3 | 本地、沉默、只读承诺 | 这是点击安装前的主要信任顾虑，必须在动作附近消除。 |
| 4 | 安装完成状态 | 给出动作已生效的明确反馈，不升级成全局显眼标记。 |
| 5 | Claude / Codex 单独状态 | 只在部分安装、损坏或失败时帮助定位恢复对象。 |
| 6 | 错误详情 | 仅服务失败恢复，正常路径不展示。 |
| 7 | `Check again` | 供用户在外部修改配置后手动复核，低于直接安装 / 修复。 |

### §3 区域映射与重点落位

| 区域 | 放什么 | 唯一职责 | 优先级 | 次要让位去向 |
|---|---|---|---|---|
| Settings section | `Permission reminders (optional)` + 一句话说明 | 解释可选增强的价值与边界 | P1 | 不做全页 hero、不进入 Panel 提示 |
| Action row | `Install for Claude & Codex` / `Repair installation` / `Installed` | 让用户用一次动作完成当前需要的安装状态变化 | P1 | Provider 逐项操作仅在失败时展开 |
| Trust note | Hook 的本地、沉默、无 Agent 输入承诺 | 在点击前消除“Hook 会不会干预 Agent”的顾虑 | P1 | 详细技术字段收进错误详情 |
| Recovery details | Claude / Codex 状态、错误详情、`Try again`、`Check again` | 只处理部分安装或失败恢复 | P2/P3 | 健康时折叠，不常驻制造警告感 |

固定说明：

`Install local hooks to also detect when Claude or Codex is waiting for approval. WhoNeedsMe works without them.`

固定信任承诺：

`Hooks only append local events. They never approve, reply, or send input.`

### §4 行动引导

| CTA（动作） | 价值主张（为什么值得点） | 点击前顾虑及消除 |
|---|---|---|
| `Install for Claude & Codex` | 一次安装两种 Provider 的权限等待增强，不用手工编辑配置 | 动作前明确 Hook 可选、只写本地事件、不改变 Agent 行为。 |
| `Repair installation` | 保留无关用户配置的同时恢复 WhoNeedsMe 自有 Hook | 明确修复幂等，只处理 WhoNeedsMe 管理的配置片段。 |
| `Try again` | 从失败状态重新执行当前安装 / 修复动作 | 错误详情说明失败不会影响基础 Session 列表、问题或计划等待。 |
| `Check again` | 重新检查外部配置变化 | 只读检查，不在未确认时改写配置。 |

### §5 蓝图自检

- [x] core 一句话清晰
- [x] 优先序列表每条有可追溯依据
- [x] 每个区域只承担一个目标，次要已收敛让位
- [x] 首屏承载 P1
- [x] 每个 CTA 意图清晰、绑 core，点击前主要顾虑已消除（行动引导在内容层）
- [x] 无“都要→都平均”区域

## 界面:Alert Card

### §1 core

- 服务谁：正在其他窗口专注工作、刚有一个或多个 Session 从不需要交互变成 Waiting 的用户。
- 促成什么动作：点击 `Open panel`，查看全部 Waiting Session 后选择先处理哪一个。
- core（一眼价值）：**有人开始等你时立即被看见，并一次查看全部待处理 Session。**

### §2 优先序列表

| 排位 | 内容模块 | 排位依据 |
|---:|---|---|
| 1 | 单 Session 项目身份或多 Session 数量 | 第一眼说明“谁 / 有多少人在等”，直接服务提醒价值。 |
| 2 | 等待原因与 Elapsed，或跨项目数量 | 帮用户判断是否值得现在打断手头工作，但不在 Alert 内展开列表。 |
| 3 | `Open panel` | 把用户带到可靠的全局列表主路径。 |
| 4 | 猫的聚合反应态 | 提供品牌识别与状态反馈，但内容优先级低于等待事实与动作。 |

### §3 区域映射与重点落位

| 区域 | 放什么 | 唯一职责 | 优先级 | 次要让位去向 |
|---|---|---|---|---|
| Alert message | 单 Session：`{project} needs you`；多 Session：`{N} sessions need you` | 立即说明提醒发生的原因 | P1 | 不放 Session 列表、设置或 Hook 状态 |
| Alert context | 单 Session：等待原因 + Elapsed；多 Session：`Across {M} projects` | 给出是否立刻处理所需的最小上下文 | P2 | 详细 Session 信息进入 Panel |
| Alert action | `Open panel` | 把用户带到全部 Waiting 与全局状态 | P1 | 不做直接批准、回复或单 Session 直跳终端 |

### §4 行动引导

| CTA（动作） | 价值主张（为什么值得点） | 点击前顾虑及消除 |
|---|---|---|
| `Open panel` | 一次查看全部 Waiting Session、项目与等待原因，再由用户决定先处理谁 | 点击只打开 WhoNeedsMe Panel，不直接跳错终端，也不向 Agent 输入。 |

### §5 蓝图自检

- [x] core 一句话清晰
- [x] 优先序列表每条有可追溯依据
- [x] 每个区域只承担一个目标，次要已收敛让位
- [x] 首屏承载 P1
- [x] 每个 CTA 意图清晰、绑 core，点击前主要顾虑已消除（行动引导在内容层）
- [x] 无“都要→都平均”区域

## 界面:Tray Icon

### §1 core

- 服务谁：正在其他窗口工作、不想逐个切回终端检查状态的多 Agent 用户。
- 促成什么动作：看到聚合态后，在需要全局详情时点击 Tray 打开 Session Panel。
- core（一眼价值）：**不切窗口就知道此刻是否有 Session 需要你。**

### §2 优先序列表

| 排位 | 内容模块 | 排位依据 |
|---:|---|---|
| 1 | calm / working / attention 聚合状态 | Tray 存在的首要理由，是持续回答“有没有人需要我”。 |
| 2 | 状态 tooltip | 在不打开 Panel 时用文字确认图标语义，辅助可理解性。 |
| 3 | 点击打开 Panel | 当用户需要详情时进入唯一可靠的全局列表。 |

### §3 区域映射与重点落位

| 区域 | 放什么 | 唯一职责 | 优先级 | 次要让位去向 |
|---|---|---|---|---|
| Tray glyph | calm / working / attention | 常驻表达 Needs-You 聚合状态 | P1 | 数量、项目、Hook 状态全部进入 Panel / Settings |
| Tooltip | attention：`WhoNeedsMe — {N} sessions need you`；其他：`WhoNeedsMe — no sessions need you` | 用文字解释当前聚合态 | P2 | 不展开单 Session 详情 |

### §4 行动引导

| CTA（动作） | 价值主张（为什么值得点） | 点击前顾虑及消除 |
|---|---|---|
| 点击 Tray 打开 Session Panel | 一次看到全部 Waiting 与其他活跃 Session，而不是逐个检查终端 | 点击只打开 Panel；不按 Session 数量分叉、不直接跳终端、不控制 Agent。 |

### §5 蓝图自检

- [x] core 一句话清晰
- [x] 优先序列表每条有可追溯依据
- [x] 每个区域只承担一个目标，次要已收敛让位
- [x] 首屏承载 P1
- [x] 每个 CTA 意图清晰、绑 core，点击前主要顾虑已消除（行动引导在内容层）
- [x] 无“都要→都平均”区域
