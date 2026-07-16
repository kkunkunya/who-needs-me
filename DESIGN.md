# WhoNeedsMe 设计契约

> status: accepted
> 决策负责人: Kun
> token 真理源: 本文件（greenfield，尚无 theme/token 文件；本文件拥有首批 semantic tokens，实现从它生成或逐字同步）
> 视觉证据: `docs/design/reference-panel-dark.png`（用户确认满意的暗色参考图）

## 概览

WhoNeedsMe 是一个本机只读状态托盘应用（Tauri，macOS 优先）。可见界面是三个 **surface**，不是多页路由：

1. **Session Panel**——菜单栏下拉面板，一行一个会话的列表（主界面）。
2. **Alert Card**——Needs You 触发时角落弹出的自绘置顶窗。
3. **Tray Icon**——菜单栏常驻图标（猫 + 聚合状态）。

本契约覆盖这三个 surface 的跨界面稳定规则。产品语义见 `CONTEXT.md` 与 Spec #3；背景决策见 `docs/adr/`。

开发期另允许一个 **throwaway Session Debug Surface**：只用于 walking skeleton 验证本地 Engine → Tauri command → WebView 数据通道，不属于上述 V1 产品 surface，也不承诺成品 Panel 的 anatomy、组件、交互或视觉 fidelity。

## 设计原则

- **单一强调色只服务 Needs-You。** 琥珀/amber 是全应用唯一强调色，**只用于**"需要你"（Waiting 状态、等待原因、Needs-You 高亮、Alert），**永不做装饰**。Working/Idle/Ended 一律中性灰。这是最硬的不变量——它让强调色一出现就等于"该你了"。
- **身份 → 状态 → 操作，顺序固定。** 每行先让人认出"哪个项目/会话"（图标+项目+分支），再读状态，操作永远最后、最轻。
- **Needs-You 永远置顶并 pin。** 排序恒为 Waiting → Working → Idle → 刚结束；Waiting 行钉在最上、带轻微 amber 底色区分。
- **高密度但克制。** near-black 表面、1px hairline 分隔、慷慨行距、无渐变堆砌、无花哨动效。Linear/Raycast 级极简。
- **猫是产品门面角色，有存在感、但不抢"谁在等我"的主角位。** 猫是 WhoNeedsMe 的品牌脸，有一组**表情/姿态**镜像聚合态：全静=蜷着/睡（calm）· 有会话在跑=坐姿警觉/看向屏幕（working）· ≥1 个等你=竖耳+看向你+amber（attention）。它可以**登台**于 Tray Icon（主舞台）、Alert Card（会反应的猫）、空态（较大插画）、面板 header（小头像），可以有戏；但**不进数据行**，且**会话列表 + Needs-You 语义永远是视觉主角**——猫是主持人，不是主角。为与克制 UI 共存：猫**保持单色线条**（非彩色卡通），且**只在 needs-you/attention 态用 amber**（与"强调色只服务 Needs-You"不变量一致），calm/working 态为中性线条。（参考图那只"睡猫配 2 个在等你"语义反了，实现须让猫态跟聚合态一致。）
- **代码类元数据用 monospace。** git 分支、model tag 用等宽字体；项目名与正文用 SF 系无衬线。
- **只读的视觉诚实。** 任何 surface 都不得出现可能"替用户操作 agent"的控件（无"批准/回复/发送"按钮）。允许的动作只有导航类：Open panel、View all sessions、点击行跳转终端。

## Token 真理源

- 规范来源：**本文件**（下表）。实现侧的 CSS variables / Rust 常量从此表生成或逐字同步，不得另立原值。
- 主题：**V1 dark-only**（参考图为暗色、目标用户偏深色；token 按语义命名，light 主题留待 post-V1 再补值，不在 V1 定义 light 值）。

| Semantic token | 暗色值 | 语义角色 |
|---|---|---|
| `bg-panel` | `#1C1C1E` | 面板 / Alert 卡的 elevated 背景 |
| `bg-row-hover` | `#26262A` | 行 hover |
| `bg-waiting-tint` | `rgba(227,155,62,0.06)` | pinned Waiting 行的极淡 amber 底 |
| `border-hairline` | `rgba(255,255,255,0.08)` | 1px 分隔线 / 卡片描边 |
| `text-primary` | `#ECECEC` | 项目名、标题 |
| `text-secondary` | `#8A8A8E` | 次要文案（footer、计数） |
| `text-mono` | `#A8A8AD` | 分支 / model tag（等宽） |
| `text-disabled` | `#5A5A5E` | Ended / 失效 |
| `accent-attention` | `#E39B3E` | **唯一强调色**：Waiting 点/chip/原因文字、Needs-You、Alert、Tray attention |
| `accent-attention-strong`| `#F0A94A` | 强调色 hover/按压 |
| `state-working` | `#7C7C82` | Working 状态点（中性灰） |
| `state-idle` | `#5A5A5E` | Idle 状态点（更暗灰） |
| `state-ended` | `#48484C` | Ended 状态点（最暗） |

> 不变量：状态点里**只有 Waiting 用 `accent-attention`**，其余全部取 `state-*` 中性灰——编码原则一。

## 共享组件

| 组件 | Anatomy（slots） | 允许的 variants | 必备状态 | 源码或 story |
|---|---|---|---|---|
| `SessionRow` | `[pin?] · [provider glyph] · {项目名 / 分支(mono)} · [model tag] · [state chip] · [elapsed]`；Waiting 追加一行 `[原因文字]` | `default` / `pinned-waiting` | working · waiting(reason ∈ 批准权限/回答问题/确认计划) · idle · ended | _待实现_ |
| `StateChip` | `[dot] [label]` | working / waiting / idle / ended | 同上 | _待实现_ |
| `ModelTag` | 等宽 pill，中性 | — | 有值 / 缺失（隐藏） | _待实现_ |
| `ProviderGlyph` | 单色矢量，按 Provider | claude / codex /（未来）其他 | — | _待实现_ |
| `AlertCard` | `[会反应的猫] [标题:"N sessions are waiting for you"] [主动作:Open panel]` | 1 session / N sessions（合并） | — | _待实现_ |
| `TrayIcon` | 猫 line-art glyph（表情态） | `calm` / `working` / `attention` | calm（无人等）· working（有会话在跑）· attention（≥1 Waiting，竖耳+amber 下划线） | _待实现_ |
| `PanelHeader` | `[猫小头像(聚合态)] [N sessions] [快捷键] [设置]` | — | 随聚合态换头像 | _待实现_ |
| `EmptyState` | `[较大猫插画(蜷睡)] [文案:当前没有活跃会话]` | — | 无活跃会话时 | _待实现_ |

## 交互模式

| Pattern | 触发 | 组成 | 不变量 | 证据 |
|---|---|---|---|---|
| Needs-You Alert | Needs-You false→true | 自绘置顶窗（主，必达、不抢键盘焦点）+ 原生通知（辅，锁屏/跨 Space） | 同一时刻至多一个 Alert；多 session 合并为"N…"；只用 `accent-attention` | ADR-0004、ADR-0007 |
| 点击 → 面板 | 点击 Tray / Alert 卡 | 打开 Session Panel 列表 | 点击一律开列表，不做按数量分叉直跳 | ADR-0007 |
| 行 → 终端聚焦 | 点击某 SessionRow | best-effort 跳转/前置对应终端，跳不准则把终端 App 带前台 | 纯导航、不向 agent 输入；不在必经路径 | ADR-0007 |
| 排序与 pin | 列表渲染 | Waiting 置顶并 pin | 恒为 Waiting→Working→Idle→刚结束 | ADR-0007 |
| Tray 聚合态 | 任一 session 进/出 Waiting | 猫 calm ↔ attention | Tray 只表达聚合"有没有人等你"，不逐个刷 | ADR-0004、ADR-0007 |

## 页面族

surface 数量少，只有一个真正的"列表页面"家族；Alert 与 Tray 是独立 surface，不构成页面族。

| Page family | 页面开头 anatomy | 必备 slots | 可选 slots | 响应式行为 |
|---|---|---|---|---|
| Panel（列表面板） | `header → pinned Waiting 行 → 其余行 → footer` | header · 行列表 · footer | empty 态插画（无活跃会话时替换行列表） | 面板高度随行数增长，超出上限则内部滚动 |

## Route / Surface 契约

| Route 或 surface | Family | 首屏目的 | 开头 slots | 允许的差异 | Fixture |
|---|---|---|---|---|---|
| Session Panel | Panel | 一眼看清谁在等我 + 全局 | `[header: 猫头像 / N sessions / 快捷键 / 设置] → [pinned Waiting 行] → [其余行] → [footer: Updated / View all]` | 无 | 合成会话数据目录（Spec #3 seam） |
| Alert Card | —（独立 surface） | 把"有人等你"推到眼前 | `[会反应的猫] [标题] [Open panel]` | 1 vs N session 文案 | 同上，构造 ≥1 Waiting |
| Tray Icon | —（独立 surface） | 常驻聚合信号 | `[cat glyph + 聚合态]` | calm / working / attention 三态 | 同上 |
| Session Debug Surface（开发期例外） | —（throwaway，不属于 V1 页面族） | 验证本地只读 Session 数据通道 | `[debug heading / session count] → [provider + project identity + 占位元数据/状态的原始列表]` | 仅允许中性 token、无成品组件/交互/amber、后续由 Session Panel 替换 | `crates/who-needs-me-core/tests/fixtures/claude`（mixed）+ 空会话根（empty） |

## 例外与变更协议

- **开发期 route/surface 例外**：Issue #4 的 `Session Debug Surface` 可作为 Tauri 主窗口临时存在，只验证可启动、托盘常驻、fixture 行数/identity、本地只读边界与 neutral token 对齐；不得把它当作 Session Panel 设计证据，也不得为它引入新的共享组件、交互模式、page family 或视觉 token。完成成品 Session Panel 后删除该例外与 throwaway surface。
- 新增 Provider 时 `ProviderGlyph` 加一个单色 variant，不改行结构；新增强调用途前先回本契约确认（强调色只服务 Needs-You 的不变量优先）。
- light 主题、灵动岛/顶栏留待 post-V1；届时刷新本契约而非就地加值。

## 验证矩阵

UI 层无自动化 seam（Spec #3 规定 UI 走人工验收），用固定截图矩阵。V1 dark-only。

| Surface | 视口 | 状态 | fixture / 入口 | 证据路径 |
|---|---:|---|---|---|
| Session Panel | ~380×折叠 | mixed（含 2 Waiting + Working + Idle） | 合成数据目录 | _待截图_ |
| Session Panel | ~380×折叠 | empty（无活跃会话） | 空 fixture | _待截图_ |
| Session Panel | ~380×折叠 | all-waiting（多个需要你） | 全 Waiting fixture | _待截图_ |
| Alert Card | 卡片 | 1 session（含具体原因） | 单 Waiting fixture | _待截图_ |
| Alert Card | 卡片 | N sessions（合并文案） | 多 Waiting fixture | _待截图_ |
| Tray Icon | 菜单栏 | calm / attention | 有/无 Waiting | _待截图_ |
| Session Debug Surface（开发期例外） | Tauri 主窗口 | mixed / empty | 合成 Claude fixture / 空会话根；运行 `npm test`、`npm run build:ui`、Tauri check，并分别启动 fixture 验证行数/identity 与空列表 | PR 验证评论（不进入 V1 成品截图证据） |
