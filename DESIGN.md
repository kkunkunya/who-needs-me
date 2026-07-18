# WhoNeedsMe 设计契约

> status: accepted
> 决策负责人: Kun
> token 真理源: `docs/design/design-reference.md`
> 视觉证据: `docs/design/recon/reference-set.md`（夜蓝封面负责色彩与情绪；暗色概念图负责构图；macOS popover 负责局部分组）

## 概览

WhoNeedsMe 是一个本机只读状态托盘应用（Tauri，macOS 优先）。V1 有四个可见 surface：

1. **Session Panel**——菜单栏下拉的完整状态板。
2. **Permission Reminders Settings**——安装或修复可选沉默 Hook 的低频设置面。
3. **Alert Card**——Needs You 触发时出现的自绘置顶提醒。
4. **Tray Icon**——菜单栏常驻的猫猫聚合状态。

开发期另允许一个 **throwaway Session Debug Surface**，只验证 Engine → Tauri command → WebView 的只读数据通道，不属于 V1 成品 family，也不提供成品视觉证据。

## 设计原则

- **安静值守。** 产品调性是系统工具的精确感 70% + 安静陪伴感 20% + 克制提醒感 10%；深夜蓝承担身份，暖珊瑚只在真正需要用户介入时出现。
- **Signal Earns Color。** 暖珊瑚是 Needs You 的唯一 attention signal，面积保持 `<5%`；Working、Idle、普通按钮、Settings 与装饰保持中性。真正错误必须同时使用错误图标与文字，不能只靠颜色。
- **Answer, Then Compare。** Header 先回答当前是否有人等你，再展示可比较的 Session 明细；不以无摘要列表开场。
- **Character Hosts State。** 猫只承载聚合状态与情绪，固定出现在 Tray、Panel Header、Alert、Empty；不进入 Session 数据行。
- **Waiting 跨项目全局置顶。** 每一条 Waiting 行必须说明项目组；非 Waiting Session 才按 Git repository/worktree family 分组，非 Git 目录退化为 normalized cwd。
- **Sections Before Cards。** 紧凑单栏、轻量 section heading、padding 与 hairline 优先；普通项目组和 Session 行不获得独立大卡片或阴影。
- **Provider 是信息，不是装饰。** Session 行首展示可识别的官方 Provider CLI 图标；图标统一约 16px、统一视觉重量并配文字/无障碍名称。终端 App 只作为悬停或详情中的次级返回目标。
- **代码类元数据用 monospace。** git branch、model、shortcut 用 SF Mono 系；项目名、状态与正文用 SF 系无衬线。
- **只读的视觉诚实。** 允许的动作只有打开 Panel、best-effort 前置终端、修改本地项目组显示名、刷新、管理可选 Hook 与退出；不得批准、回复、发送或继续 Agent。

## Token 真理源

- 规范来源：`docs/design/design-reference.md` 的“字体”“颜色”“圆角”“间距”“动效”与“复合色 token”。
- 语义角色：`color-bg-*`、`color-text-*`、`color-attention*`、`color-state-*`、`radius-*`、`space-*`、`text-*`、`shadow-*`、`motion-*`。
- 消费规则：实现侧 CSS variables、Rust 常量与图像导出必须从该文件生成或逐字同步；禁止新增第二套 raw visual values。
- 主题：V1 dark-only。浅色主题属于 post-V1，届时刷新本契约与 token source。

## 共享组件

| 组件 | Anatomy | 允许的 variants | 必备状态 | 源码或 story |
|---|---|---|---|---|
| `PanelHeader` | `[compact cat] [current conclusion] [settings]` | calm / working / attention | `N need you` · `All sessions are moving` · `No active sessions` | `docs/design/design-reference.md` §11 |
| `SessionRow` | `[ProviderGlyph] [Session/branch] [current action] [project group] [elapsed]`；Waiting 追加 reason | default / pinned-waiting | working · waiting(approval/answer/plan) · idle · ended | `docs/design/design-reference.md` §11 |
| `ProviderGlyph` | `[official CLI mark] [accessible provider name]` | claude-code / codex / future provider | loaded · fallback monogram | `docs/design/assets/provider-glyphs/README.md` |
| `ProjectGroupHeading` | `[local alias or detected name] [lightweight menu] [hairline]` | detected / renamed | rename · reset | `docs/design/content-blueprint.md` Session Panel |
| `PanelFooter` | `[freshness] [settings] [shortcut] [quit]` | fresh / stale | keyboard focus · unavailable action | `docs/design/content-blueprint.md` Session Panel |
| `PermissionSection` | `[optional explanation] [install/repair action] [trust note] [provider detail?]` | optional / installed / partial / repair / error | loading · success · recoverable error | `docs/design/content-blueprint.md` Permission Reminders |
| `AlertCard` | `[attention cat] [fact] [minimal context] [Open panel]` | single / multiple | enter · visible · dismissed | `docs/design/design-reference.md` §11 |
| `TrayIcon` | `[micro cat glyph]` | calm / working / attention | @1x · @2x · light/dark menu bar | `docs/design/design-reference.md` §10–11 |
| `EmptyState` | `[sleeping cat] [No active sessions] [supporting copy]` | empty | reduced motion | `docs/design/design-reference.md` §10–11 |

窄面板中 `current action` 优先省略；Provider、Session、项目组与 elapsed 必须保留。猫素材的角色比例、分层、导出和人审门禁见 `docs/design/design-reference.md#10-cat-asset-contract`。

## 交互模式

| Pattern | 触发条件 | 组合方式 | 不变量 | 证据 |
|---|---|---|---|---|
| Needs-You Alert | Needs You false→true | 自绘置顶窗 + 原生通知补位 | 同时至多一个；多 Session 合并；不抢键盘焦点 | ADR-0004、ADR-0007 |
| Tray / Alert → Panel | 点击 Tray 或 Alert | 打开完整 Session Panel | 不按数量直跳 Session | ADR-0007 |
| Session → Terminal | 点击 SessionRow | best-effort 前置对应终端；失败则前置终端 App | 只导航，不向 Agent 输入 | ADR-0007、content blueprint |
| Global Waiting | 列表渲染 | 所有 Waiting 先形成 P1 区，再渲染 grouped Other Sessions | Waiting 不被项目分组打散；每行显示项目组 | content blueprint |
| Project alias | 项目组菜单 | Rename project… / Reset to detected name | 只改本地显示名，不改目录或 Git | content blueprint |
| Optional Hook | Settings 中人工触发 | Install / Repair / Try again | 不安装也不影响基础监控；Panel/Tray/Alert 无 Hook badge | content blueprint |
| Cat state motion | 聚合态变化或 Alert 出现 | Tray 静态换帧；Header/Alert 一次 200ms 微动效 | 不循环；reduced-motion 立即换帧 | design reference |

## 页面族

| Page family | 页面开头 anatomy | 必备 slots | 可选 slots | 响应式行为 |
|---|---|---|---|---|
| Panel（状态列表） | `Header conclusion → global Waiting → grouped Other Sessions → Footer` | header · Waiting/Other 或 Empty · footer | stale/error last-known notice | 约 380px 单栏；Header/Waiting 留在首屏，长列表内部滚动 |
| Settings（低频配置） | `title → optional explanation → action → trust note → provider details` | title · action · trust note | provider details · recovery error | 保持紧凑单栏；详情仅在 partial/error 展开 |

## Route / Surface 契约

| Route 或 surface | Family | 首屏目的 | 开头 slots | 允许的差异 | Fixture |
|---|---|---|---|---|---|
| Session Panel | Panel | 一眼看清谁在等你并选择要返回的终端 | `Header → Waiting → Other Sessions → Footer` | Empty 替换列表；stale/error 保留 last-known list | mixed / empty / all-waiting / stale / long-project-names |
| Permission Reminders Settings | Settings | 可选安装或修复沉默 Hook | `Explanation → Action → Trust note → Details?` | Installed 折叠详情；partial/error 展开 provider 结果 | optional / installed / partial / error |
| Alert Card | 独立 surface | 说明有人需要你并打开 Panel | `Cat → fact → context → Open panel` | single 显示项目与原因；multiple 显示 Session/项目数 | single-waiting / multi-project-waiting |
| Tray Icon | 独立 surface | 常驻表达聚合状态 | `micro cat glyph` | calm/working 单色 template PNG（黑+alpha，macOS 按浅/深菜单栏自动反色）；attention 暖珊瑚 `#F05A5D` 非 template；三态靠耳姿+眼型+accent 区分 | calm / working / attention @1x/@2x |
| Session Debug Surface | 开发期例外 | 验证本地只读数据通道 | `debug heading → raw session list` | 只消费中性 token；无成品分组、交互、猫或 attention | Claude mixed fixture / empty root |

## 例外与变更协议

- Session Debug Surface 只验证数据链与首批 token 同步；完成 Session Panel 后删除该例外，不得把其宽版布局提升为 shared rule。
- ProviderGlyph 必须来自可追溯的官方资产源并记录授权说明；无法取得时使用中性文字 monogram，不自行仿画品牌标志。
- 新 Provider 只增加 ProviderGlyph variant，不改变 SessionRow 固定轨道；第二个真实消费者出现前不提升新局部模式。
- **Cat runtime assets 走已批准的 gpt-image-2 栅格例外**（2026-07-17 Kun 拍板）：以人审通过的 character sheet 为锚，用 gpt-image-2 生成栅格母版，经 `/produce-assets` 抠透明底 + resize 交付透明 PNG/WebP @1x/@2x。**compact(32px) 与 illustration(200px) 两档保持全色栅格**（Kun 已认可栅格）。
- **Tray micro(18px) 回退为 template 单色 glyph**（2026-07-18 Kun 拍板）：实机预览确认浅色菜单栏上重着色的浅猫栅格几乎隐形，且单张定色栅格物理上无法同时适配浅+深两种菜单栏。micro 三态改为从 SVG 几何派生的极简头+耳剪影 PNG：calm/working 为纯黑+alpha 的 macOS **template image**（由系统按浅/深栏自动反色，两种背景均清晰），attention 保留暖珊瑚 `#F05A5D` 非 template（Signal Earns Color）；三态靠耳姿（放松 vs 竖起）+眼型（闭合 vs 睁开）+ accent 区分。仅 Tray micro 走此路，compact/illustration 不变。**人审门禁保留**：character sheet、栅格母版、Tray 实机截图与 motion note 未经人审前，不得把临时 raster 概念图当运行时资产。
- 新 attention 用途、light theme、不同 Panel family 或持续循环动效都必须先回到人审，不得作为局部例外静默加入。

## 验证矩阵

| Surface | Viewport | 状态 | 命令或 fixture | 证据路径 |
|---|---:|---|---|---|
| Design contract | repo | token/source/surface coverage | `python3 /Users/kunkun/.agents/kun-agent-mono/main/skills/frontend/project-design-prompt/scripts/validate-design-contract.py "$PWD"` + `npm run check:design` | 命令输出 |
| Session Panel | 约 380px | mixed / empty / all-waiting / stale / long names | 对应 deterministic fixtures | `artifacts/design-qa/session-panel/` |
| Session Panel | 约 380px | keyboard / focus / internal scroll | mixed + long-list fixtures | `artifacts/design-qa/session-panel-interaction/` |
| Permission Reminders Settings | 约 380px | optional / installed / partial / error | Provider install-state fixtures | `artifacts/design-qa/permission-settings/` |
| Alert Card | compact card | single / multiple / reduced-motion | Waiting fixtures | `artifacts/design-qa/alert-card/` |
| Tray Icon | macOS menu bar | calm / working / attention；@1x/@2x | aggregate-state fixtures | `artifacts/design-qa/tray-icon/` |
| ProviderGlyph | 16px | Claude Code / Codex / fallback | asset contact sheet | `artifacts/design-qa/provider-glyphs/` |
| Cat asset family | micro / compact / illustration | state sheet + start/end frames；`npm run check:cat-assets` | approved character sheet / raster masters (gpt-image-2) | `artifacts/design-qa/cat-assets/` |
| Session Debug Surface | desktop | mixed / empty | `npm test` + fixture launch | PR test output |
