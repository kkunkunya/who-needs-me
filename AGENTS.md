# WhoNeedsMe

A tiny status board for coding agents（Tauri 托盘应用）。产品定义见 `docs/agents/` 与 issue tracker。

## Agent skills

### Issue tracker

工作单跟踪在 GitHub Issues（`kkunkunya/who-needs-me`），用 `gh` CLI 操作。见 `docs/agents/issue-tracker.md`。

### Triage labels

用默认角色名作为标签字符串（2 类别 + 5 状态）。见 `docs/agents/triage-labels.md`。

### Domain docs

single-context：根目录 `CONTEXT.md` + `docs/adr/`。见 `docs/agents/domain.md`。

### Skills domain

engineering + frontend。见 `docs/agents/skills-domain.md`。

## 设计契约

- 任何新增或修改可见 route、screen、layout、component、style、asset 或响应式行为的任务，在规划或编辑前必须先读根 `DESIGN.md`。
- 实现前，列出本次影响的 tokens、共享组件、交互模式、page family、route/surface contract 与必备 fixture states。
- 复用已登记的契约项；新增共享规则或批准 route 例外时，在同一变更中更新 `DESIGN.md`。
- 请求与 `DESIGN.md` 冲突时，先把冲突交给人决定，再开始实现。
- 完成前，运行设计契约 validator 和验证矩阵中所有受影响的行。
