# ADR-0002: 事件采集用"会话文件监听 + 沉默 hooks"混合架构，不起本地 HTTP 服务

- status: accepted
- date: 2026-07-11

## 背景

Dashboard 需要实时知道本机 Claude Code / Codex 会话的存在与状态。Claude Code 会把每个会话实时写成 JSONL（`~/.claude/projects/`），Codex 写 rollout 文件（`~/.codex/sessions/`），但这些是官方未承诺的内部格式，且"正在等待权限/输入"这类瞬时状态不在其中。hooks 信号更直接但需要安装，Codex hooks 仍在快速变化。用户核心顾虑：任何方案都不得增加 Agent 的认知负担，也不得要求用户改变原有的终端启动方式。

被对照的 Supervisor（进程托管者）方案会由产品自己启动 Agent CLI，因此天然同时持有进程句柄与 Agent 返回的 sessionId，能够精确建立 Session 与进程的关联；cc-connect 属于这一类。WhoNeedsMe 的现有定位则是观察用户已经启动的 CLI。官方 Hook 事件虽携带 sessionId、cwd（工作目录）等会话信息，但不直接提供 PID（进程编号），所以 Hook 本身也不等于精确的 Session-PID 关联。

## 决策

WhoNeedsMe 保持 **Observer（旁观者）** 角色：只发现用户已经独立启动的 Agent CLI，不负责启动、托管或控制它们。采集分两层，事件一律落本地文件，**不起任何本地 HTTP/socket 服务**：

1. **底座：监听官方会话文件目录**——零安装即可发现所有会话、项目归属与最近活动；
2. **可选增强：极简沉默 hooks**——只补会话文件给不了的关键信号（Needs You、turn 结束），hook 行为仅为追加写本地事件文件，不输出任何内容进 Agent 上下文；不安装 Hook 时，基础会话列表仍可用；
3. 进程存活检查兜底 Ended 状态，但在 Hook 和会话数据都没有给出 PID 时，不宣称 Session 与进程已精确绑定。

## 理由

- hooks 是 harness 层机制，模型不可见、零 token 开销；"沉默 hook"原则把对 Agent 的影响钉死为零。
- 文件底座让用户不装任何东西就能看到会话列表，安装 hooks 只是增强 Session 身份与状态信号，安装失败产品也不至于全盲。
- Observer 边界保留用户现有的终端与启动习惯，也符合 WhoNeedsMe 只读状态面板的产品定位。
- 本地 HTTP 的实时性收益文件监听同样能给（fs 事件毫秒级），但要求 App 常驻否则丢事件，还引入端口管理——纯代价。
- 业内同类做法印证：AgentPeek 用 hooks，claude-watch-status 用"JSONL 监听 + hooks 加速"混合。
- 被否备选：Supervisor（必须从 WhoNeedsMe 启动 Agent，越过旁观与只读边界）；纯 hooks（不装就全盲、Codex hooks 不稳）；纯文件监听（Needs You 测不准，而它是核心卖点）；本地 HTTP 服务（见上）。

## 后果

- 需要为 Claude / Codex 各维护一个会话文件格式 adapter，格式属内部实现，随版本变化要有 contract test 与降级路径（读不懂时退化为"存在但状态 Working/未知"）。
- hooks 安装器是可选的精度增强（一键安装/检查/修复），不是产品可用的前置条件。
- 无 Hook 时，同一工作目录内多个历史 Session 与活进程的关联可能不唯一；面板筛选、Needs You 与用户呈现规则由 ADR-0007 约束。
- 事件存储与折叠逻辑都在本地文件 + Tauri Rust 端完成，无网络面。
