# WhoNeedsMe

本机 Coding Agent CLI 的只读状态面板。核心问题：谁正在等我？

## 会话与状态

**Session**:
一次某个 Provider 的运行实例，面板的展示单位（一行一个 session）。由 `(Provider, sessionId)` 唯一标识；resume 出的新 sessionId 视作新的 Session，不与旧的拼接续接。
_Avoid_: 任务、会话窗口

**Working**:
session 正在思考、调用工具或执行命令的状态。

**Waiting**:
session 停下来等用户回答、权限批准或计划确认的状态；唯一会点亮 Needs You 的状态。

**Idle**:
当前 turn 已结束、等待下一条 prompt 的状态。Idle 不算 Needs You。
_Avoid_: 空闲等待（易与 Waiting 混淆）

**Ended**:
CLI 进程或 session 已经结束的状态。

**Needs You**:
"该 session 正在等用户交互"的布尔标志。判定宁缺毋滥：只有能唯一识别并配对的直接 Session 信号才点亮；不确定时不点亮，绝不把 Idle 错报成 Waiting。语义工具事件缺少 `tool_use.id`（工具调用标识）时无法可靠判断后续是否已有对应结果，按标准状态折叠为 Working，不点亮 Needs You。（ADR-0006）

## 采集

**Observer**:
WhoNeedsMe 的采集角色：发现并读取用户已经独立启动的 Provider 会话与进程，不负责启动、托管或控制这些 CLI。Hook 只增强采集精度，不改变这一边界。（ADR-0002）
_中文_: 旁观者
_Avoid_: Supervisor、进程托管器

**Provider**:
WhoNeedsMe 支持的一款具体 coding agent CLI 工具，如 Claude Code、Codex CLI；每接入一个新 Provider 就新增一个 Adapter，不改动核心状态机与 UI。（ADR-0003）

**Adapter**:
针对某个 Provider 实现的适配逻辑——发现该 Provider 的本地会话数据、从中解析出 Working/Waiting/Idle/Ended 四态。是否支持沉默 hook 由各 Provider 的 Adapter 自行决定，不是 Adapter 的必备能力。（ADR-0003）

**会话数据底座**:
通过各 Provider 的 Adapter 读取其本地会话数据（文件、SQLite 等不限存储形式）获得的零安装事实源；hooks 只是它之上的信号补充。（ADR-0002、ADR-0003）
_Avoid_: 会话文件底座（旧称，ADR-0003 已泛化为不限存储形式）

**沉默 hook**:
只追加写本地事件文件、不向 Agent 上下文输出任何内容、正常退出的 hook。本项目所有 hooks 的硬约束——对 Agent 的认知负担必须为零；是否提供由各 Provider 的 Adapter 自行决定，并非所有 Provider 都要有。（ADR-0002、ADR-0003）

**关联置信度**:
系统内部判断候选 Session 与活进程对应关系把握程度的诊断信息；直接 Session 级信号可形成精确关联，仅有 Provider + cwd（工作目录）时属于推断关联。它只用于筛选、降级和诊断，绝不作为 Session 行上的用户标签。（ADR-0007）
_Avoid_: 关联不确定（面向用户的文案）、数据可能不准

## 呈现

**只读**:
WhoNeedsMe 不代替用户向任何 Provider 输入内容——不发消息、不批准权限、不做任何 Agent 侧操作。窗口聚焦跳转（把对应终端带到前台）不算违反只读，因为不涉及向 Agent 输入任何东西。（ADR-0004）

**Alert**:
Needs You 从 false 变为 true 时触发的前台提醒，形态为一个自绘置顶窗（主提醒）加一条原生系统通知（辅助触达）。同一时刻至多一个 Alert；多个 session 同时进入 Waiting 时合并为一个 Alert，不逐个弹出。（ADR-0004、ADR-0007）

**等待原因**:
Session 处于 Waiting 时进一步区分究竟在等什么——权限批准 / 回答问题 / 计划确认三选一。用于 Alert 与面板的呈现，帮助用户不用切回终端就能判断该不该现在处理；不等同于对话内容摘要。（ADR-0005）
