# ADR-0005: V1 Session 展示字段扩展到文件夹/分支/模型/上下文占用/等待原因

- status: accepted
- date: 2026-07-14

## 背景

用户提出 WhoNeedsMe 应该帮用户在回到一个 session 时"快速回忆起自己在做什么"，而不只是知道它在 Waiting。这牵出一组候选展示字段：所属文件夹、git 分支、具体模型名、上下文占用率、以及 Waiting 的具体原因。逐一核查发现：直接读取本机真实的 Claude Code JSONL（`~/.claude/projects/`）与 Codex CLI rollout 文件（`~/.codex/sessions/`），确认 `cwd`、`gitBranch` / `git.branch`、`model`、token usage 都已经是这两种 Provider 会话文件里的既有字段——不需要新增架构、hooks 或联网即可拿到。Waiting 的具体原因也部分可判定：Claude Code 的 `AskUserQuestion` 作为独立的 `tool_use` block name 直接出现在数据里，可精确识别；权限批准/计划确认需要更多一点事件序列判断，但同样在纯本地数据范围内可行。

## 决策

V1 的 Session 展示字段扩展为：Provider、Working/Waiting/Idle/Ended 四态、Needs You、所属文件夹（cwd 截断为 2-3 段路径）、Git 分支、具体模型名、上下文占用率、等待原因（权限批准 / 回答问题 / 计划确认三选一）。

不收进 V1 的：对话内容的语义摘要（"具体在聊什么"）、额度/quota 追踪、灵动岛/顶栏呈现（后者已由 ADR-0004 覆盖，此处重申）——这些留给 V2。

## 理由

- 文件夹/分支/模型名/上下文占用率四项经实测确认是 Adapter 解析四态时能顺手拿到的既有字段，成本接近零，没有理由拖到 V2。
- looppulse（技术栈与 WhoNeedsMe 最接近的竞品）README 明确列出它已经在展示"model, project, runtime, token usage, and context %"这一组几乎相同的字段，说明这不是过度设计，是这个细分里已验证过的合理组合。
- 等待原因只做到"三选一分类"，不做具体内容摘要——分类可以从 `tool_use` 名称等结构化信号判定，不涉及理解/展示会话正文语义，不新增隐私暴露面；内容摘要则需要读取并理解正文语义、可能要接入模型做总结，复杂度和隐私边界跳了一级，值得单独留给 V2 评估。
- 被否备选：把这些字段全部留到 V2、V1 只做最小的四态展示——被否是因为实测显示这些字段的边际成本几乎为零，人为拖延没有技术理由，只会让 V1"帮用户快速回忆"这个目标打折扣。

## 后果

- Adapter 的解析职责从"只解析四态"扩展为"同时解析四态 + 展示元数据（文件夹/分支/模型/上下文占用/等待原因）"，仍在同一次文件读取内完成，不新增采集通道。
- Issue #1/#2（Adapter + 状态机）与 Issue #4（托盘图标 + 会话列表面板 UI）的验收标准需要补上这些字段。
- Codex CLI 的"权限批准"事件具体怎么判定，留给对应 Adapter 实现时再核实细节，不阻塞本决策。
