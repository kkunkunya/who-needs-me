# ADR-0002: 事件采集用"会话文件监听 + 沉默 hooks"混合架构，不起本地 HTTP 服务

- status: accepted
- date: 2026-07-11

## 背景

Dashboard 需要实时知道本机 Claude Code / Codex 会话的存在与状态。Claude Code 会把每个会话实时写成 JSONL（`~/.claude/projects/`），Codex 写 rollout 文件（`~/.codex/sessions/`），但这些是官方未承诺的内部格式，且"正在等待权限/输入"这类瞬时状态不在其中。hooks 信号可靠但需要安装，Codex hooks 仍在快速变化。用户核心顾虑：任何方案都不得增加 Agent 的认知负担。

## 决策

采集分两层，事件一律落本地文件，**不起任何本地 HTTP/socket 服务**：

1. **底座：监听官方会话文件目录**——零安装即可发现所有会话、项目归属与最近活动；
2. **补充：极简沉默 hooks**——只补会话文件给不了的关键信号（Needs You、turn 结束），hook 行为仅为追加写本地事件文件，不输出任何内容进 Agent 上下文；
3. 进程存活检查兜底 Ended 状态。

## 理由

- hooks 是 harness 层机制，模型不可见、零 token 开销；"沉默 hook"原则把对 Agent 的影响钉死为零。
- 文件底座让用户不装任何东西就能看到会话列表，安装 hooks 只是"解锁 Needs You"，安装失败产品也不至于全盲。
- 本地 HTTP 的实时性收益文件监听同样能给（fs 事件毫秒级），但要求 App 常驻否则丢事件，还引入端口管理——纯代价。
- 业内同类做法印证：AgentPeek 用 hooks，claude-watch-status 用"JSONL 监听 + hooks 加速"混合。
- 被否备选：纯 hooks（不装就全盲、Codex hooks 不稳）；纯文件监听（Needs You 测不准，而它是核心卖点）；本地 HTTP 服务（见上）。

## 后果

- 需要为 Claude / Codex 各维护一个会话文件格式 adapter，格式属内部实现，随版本变化要有 contract test 与降级路径（读不懂时退化为"存在但状态 Working/未知"）。
- hooks 安装器成为产品必备件（一键安装/检查/修复）。
- 事件存储与折叠逻辑都在本地文件 + Tauri Rust 端完成，无网络面。
