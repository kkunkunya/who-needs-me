# ADR-0003: 用可插拔 Provider/Adapter 架构支持任意 CLI 工具，不锁定 Claude Code + Codex CLI

- status: accepted
- date: 2026-07-14

## 背景

ADR-0002 的落地方式是"为 Claude / Codex 各维护一个会话文件格式 adapter"，但表述和 CONTEXT.md 现有术语都把支持范围硬编码为这两家。2026-07-14 的开源市场化调研（`docs/research/2026-07-14-whoneedsme-market-research.md`）发现，本次找到的 15+ 个同类竞品几乎全部硬编码只支持 1-3 个固定 CLI 工具（多数仅 Claude Code），没有一个把自己定位成"任意 coding agent CLI 的通用状态面板"——这是当前竞品格局里的一个空位。用户明确要求 Codex 之外，未来 OpenCode、Gemini CLI 等新工具也要能接入，且不希望每接入一个新工具就要改动核心逻辑。调研同时发现一个真实反例：OpenCode 的会话数据存放在本地 SQLite，不是 Claude/Codex 那种 append-only 文件，说明"会话文件"这个假设本身不够通用。

## 决策

把"每个 CLI 工具一个适配逻辑"正式提炼为可插拔架构：

- **Provider**：WhoNeedsMe 支持的一款具体 CLI 工具（Claude Code、Codex CLI、未来的 OpenCode 等）。
- **Adapter**：针对某个 Provider 实现的适配逻辑，职责收窄为"发现该 Provider 的本地会话数据、解析出 Working/Waiting/Idle/Ended 四态"；是否支持沉默 hook 是每个 Provider 的 Adapter 自行决定的可选能力，不强制所有 Provider 都要有。
- 原"会话文件底座"泛化为"会话数据底座"，不预设存储形式必须是文件，兼容 SQLite 等其他形式。
- 新增一个 Provider 只需新增一个 Adapter 实现，不改动核心状态机（Session/Working/Waiting/Idle/Ended/Needs You）与 UI 渲染逻辑。

## 理由

- 市场调研显示"支持任意 CLI 工具"是同类竞品里的真实空位，不是过度设计——OpenCode 用 SQLite 存会话就是现实中已经存在的反例，证明这不是臆想的未来需求。
- Adapter 边界收窄到"只管会话数据"，是因为不同 Provider 的 hook 机制差异极大且不稳定（ADR-0002 已指出 Codex hooks 仍在快速变化）。把 hook 塞进 Adapter 的必备接口会导致"没有 hook 机制的 Provider"也要实现一个空实现，增加不必要的耦合；让 hook 支持保持独立、可选，也符合 ADR-0002"文件底座为主、hooks 为辅，装 hooks 失败也不至于全盲"的既有原则。
- 被否备选：继续硬编码 Claude/Codex 两家，等真正要接入第三个工具时再重构——被否是因为用户已经明确表达了近期就要接入 Codex 之外工具的诉求，此时不设计可插拔接口，日后重构的返工成本更高，且"可插拔支持任意工具"本身就是调研中识别出的差异化叙事，值得提前做对。

## 后果

- 核心状态机与 UI 不能依赖任何 Provider 专属字段或路径假设，Provider 专属逻辑必须收敛在各自的 Adapter 实现里。
- 新增 Provider 的检查清单变为：写一个新 Adapter，实现"发现本地会话数据"与"解析四态"；可选是否接入该 Provider 自己的 hook 机制。
- "会话数据底座"替代"会话文件底座"作为正式术语，旧称写入 CONTEXT.md 的 `_Avoid_`。
