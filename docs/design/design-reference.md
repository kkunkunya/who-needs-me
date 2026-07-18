# 设计方案（WhoNeedsMe V1）

> status: accepted — 2026-07-17 完成逐项设计对齐；Cat 运行时资产仍受 §10 人审门禁
> 配对源: `reference style.md`
> 跨页面契约: 根 `DESIGN.md` —— 本文件提供视觉证据、方向与当前应用；全局 family/route/shared rule 以根文件为准
> 内容依据: `content-blueprint.md` 的 Session Panel、Permission Reminders Settings、Alert Card、Tray Icon
> 重点参考集: `docs/design/recon/reference-set.md` —— anti-slop 先读 manifest，只加载当前 leaf 对应的 primary/supporting
> 模式: 从零（Greenfield）—— 想要的设计方案
> 参考: 用户选定的夜蓝封面艺术 + WhoNeedsMe 暗色概念图 + macOS system/utility popover；取舍见 `reference style.md`

## 0. 参考来源

- Palette / mood primary：`docs/design/recon/nocturnal-blue-cover-color-primary.png`。
- Session Panel / Alert / Tray composition primary：`docs/design/recon/panel-alert-tray-dark-primary.png`。
- Project grouping supporting：`docs/design/recon/project-groups-macos-supporting.png`。
- Current summary / Footer supporting：`docs/design/recon/status-summary-footer-macos-supporting.png`。
- Permission Reminders Settings primary：`docs/design/recon/permission-settings-macos-primary.png`。
- Reference Set route：`docs/design/recon/reference-set.md`。

已锁定的取舍：夜蓝单列 popover、当前结论前置、所有 Waiting 全局置顶、普通项目组用 heading + hairline、暖珊瑚只服务 attention、Provider CLI 官方图标受控进入 Session 行、猫只在 Tray/Header/Alert/Empty、Hook 只在 Settings。

明确排除：浅色/cyan system palette、封面绘画纹理进入数据列表、`View all sessions`、Waiting 时睡猫、每项目独立大卡、无意义图标、连续 mascot 动画。

### Reference Set 使用纪律

- Palette primary 决定夜蓝 / 冷白 / 暖珊瑚关系与“安静值守”的情绪；composition primary 决定单栏构图、视觉层级与暗色密度。
- Supporting 只补 manifest 指定的局部问题，不能覆盖 primary palette 或重新决定内容顺序。
- 内容冲突时 `content-blueprint.md` 胜出；跨 surface 规则冲突时根 `DESIGN.md` 胜出；猫状态冲突时本文 Cat Asset Contract 胜出。
- 截图尺寸/色值均是 `~approx`；项目 token 使用下文精确值，不把截图估算冒充实现值。

## 1. 三旋钮

- `DESIGN_VARIANCE: 严格` —— Session Panel / Alert / Tray 靠拢两张 primary 的夜蓝情绪、单栏构图、密度、单一 attention 与 hairline 纪律；macOS supporting 只按 manifest 补结构。
  **来源:** 取自文件 A §4 的 Answer, Then Compare / Sections Before Cards 与 accepted primary。
- `MOTION_INTENSITY: 微` —— Tray 静态三态；Panel Header 与 Alert 只在状态变化/出现时播放一次 `200ms` SVG 微动效；Empty 静态。
  **来源:** Phase 5 自定（用户确认范围；项目统一采用 `200ms` 与 `cubic-bezier(0.2, 0.8, 0.2, 1)`）。
- `VISUAL_DENSITY: 密集` —— `~380px` popover 在一个 viewport 内优先呈现结论、Waiting 和至少一部分 Other Sessions；依赖 row rhythm 与内部滚动，不用大卡/大留白。
  **来源:** 取自文件 A §1.5 / §4 Answer, Then Compare，并受 content blueprint 硬约束。

## 2. 字体

- `--font-sans: -apple-system, BlinkMacSystemFont, "SF Pro Text", sans-serif` —— Header、项目组名、项目名、状态、原因、动作与 Settings。
  **来源:** 增补（根 `DESIGN.md` 已 accepted；文件 A §1.3 的 macOS system sans 证据）。
- `--font-mono: "SFMono-Regular", ui-monospace, monospace` —— branch、model、shortcut、可选 token count。
  **来源:** 增补（根 `DESIGN.md` 已 accepted；文件 A §1.3 的 mono metadata 证据）。
- `--text-heading: 20px / 650` —— current conclusion、Settings section heading。
  **来源:** 取自文件 A §1.3 的 `~approx 20–22px / 650–700`，取区间低端适配 `~380px` popover；Phase 5 已确认紧凑密度。
- `--text-project: 15px / 600` —— project / action label。
  **来源:** 取自文件 A §1.3 的 `~approx 15–16px / 550–600`；Phase 5 已确认。
- `--text-body: 13px / 400` —— state、waiting reason、Settings body。
  **来源:** 取自文件 A §1.3 的 `~approx 13–14px / 400–500`；Phase 5 已确认。
- `--text-meta: 11px / 400` —— branch、model、elapsed、footer。
  **来源:** 取自文件 A §1.3 的 `~approx 11–12px / 400–450`；Phase 5 已确认。
- tracking：普通 sans `0`；mono metadata `0`；state label 可用 `0.02em`，不全局 uppercase。
  **来源:** 增补（参考截图无可靠 DOM tracking；避免低分辨率全大写拉宽）。
- 行长：Settings trust note 最大 `44ch`；Session row 依 track ellipsis，不换成多段说明。
  **来源:** 增补（blueprint 的 compact / one-line responsibility）。

## 3. 颜色（双层）

### 页面 token

- `--color-bg-canvas: #0D1424` —— Panel 外暗场与独立窗口最深背景。
  **来源:** 从夜蓝封面的 nocturnal ink 适配，并经 Phase 5 确认。
- `--color-bg-panel: #14223A` —— Panel / Alert elevated surface。
  **来源:** 从封面主蓝主动压暗为可读系统 surface，并经 Phase 5 确认。
- `--color-bg-row-hover: #192B49` —— Session row hover / keyboard focus surface。
  **来源:** 由夜蓝 surface 递进，不引入第二种品牌色。
- `--color-bg-waiting-tint: #1B3152` —— 全局 Needs You section 与 pinned Waiting row 的轻夜蓝提亮。
  **来源:** Phase 5 明确不铺暖珊瑚 row；用 surface 层级区分 Waiting P1。
- `--color-border-hairline: rgba(180,210,235,0.14)` —— section、row、surface boundary。
  **来源:** 夜蓝封面的冷光关系 + Sections Before Cards。
- `--color-text-primary: #F1F5F7` —— current conclusion、项目名、Alert 标题。
  **来源:** 封面 fog-white 适配。
- `--color-text-secondary: #91A6B8` —— footer、group metadata、Settings supporting copy。
  **来源:** 夜蓝上的冷灰蓝层级。
- `--color-text-mono: #B1C3D0` —— branch / model / token metadata。
  **来源:** 在 compact metadata 尺寸保持清晰但低于主文案。
- `--color-text-disabled: #5D7184` —— Ended / unavailable。
  **来源:** 夜蓝中性阶。
- `--color-state-working: #7F9AB1` —— Working 状态点。
  **来源:** 中性灰蓝，不与 attention 竞争。
- `--color-state-idle: #61778A` —— Idle 状态点。
  **来源:** 中性灰蓝。
- `--color-state-ended: #485A6B` —— Ended 状态点。
  **来源:** 中性灰蓝最暗阶。
- `--color-state-error: #FF453A` —— Settings 可恢复错误。
  **来源:** macOS 语义红；必须与错误图标和文字一起出现。

### signal / accent

- `--color-attention: #F05A5D` —— Waiting dot/key text、Alert signal、Tray/compact cat attention accent。
  **来源:** 从封面的微小暖珊瑚信号适配，并经 Phase 5 确认。
- `--color-attention-strong: #FF6C6F` —— attention 自身 hover/press 的受控增强；不用于普通 CTA。
  **来源:** 同一 signal 家族的交互阶。
- 用法边界：attention 视觉面积保持 `<5%`；禁止用于整行底色、普通 Settings 安装按钮、Working/Idle、项目组标题或猫的 calm/working。
  **来源:** 取自文件 A §1.2 / §4 Signal Earns Color。

## 4. 圆角

- `--radius-sm: 6px` —— state chip、model tag、轻量 input / menu selection。
  **来源:** 取自文件 A §1.4 截图估值；Phase 5 已确认严格 variance。
- `--radius-md: 9px` —— Waiting group boundary、Settings action、Alert action。
  **来源:** 取自文件 A §1.4 截图估值；Phase 5 已确认。
- `--radius-lg: 15px` —— Panel / Alert outer surface。
  **来源:** 取自文件 A §1.4 截图估值；Phase 5 已确认。
- 嵌套规则：子 radius 必须小于父 radius；普通 project group / Session row 不获得独立 card radius。
  **来源:** 取自文件 A §4 Sections Before Cards + Nested Radius 测量。

## 5. 间距

- `--space-1: 4px`、`--space-2: 8px`、`--space-3: 12px`、`--space-4: 16px`、`--space-6: 24px`。
  **来源:** 取自文件 A §1.5 的四档 spacing rhythm；Phase 5 已确认紧凑密度。
- Panel horizontal padding：`16px`；row vertical padding：`12px`；Header/section gap：`20–24px`。
  **来源:** 取自文件 A §1.5；Header 使用 `24px` section gap。
- 组件布局优先使用 `gap`；section 边界使用 padding + hairline；禁止用任意 margin 堆叠制造不可追踪的节奏。
  **来源:** 增补（把文件 A 的 visual rhythm 落成实现纪律）。

## 6. CTA 纪律

- `--shadow-popover: 0 2px 6px rgba(0,0,0,0.28), 0 18px 44px rgba(0,0,0,0.34)` —— Panel / Alert outer 的 contact + ambient 双层阴影；内部 section 与 row 不使用阴影。
  **来源:** 文件 A §1.4 与 Phase 5 第 7 项确认。

- 主路径不是大按钮：Waiting Session 的整行点击是 `Open {project} in terminal`，hover/focus 只用 `bg-row-hover` + 明确 focus ring。
  **来源:** 增补（content blueprint Session Panel §4）。
- Alert 唯一显式 CTA：`Open panel`；使用 secondary bordered action，不用大面积 attention fill。
  **来源:** 取自文件 A primary Alert + blueprint Alert §4。
- 项目组轻量菜单：`Rename project…` / `Reset to detected name`；操作入口置于 group heading 尾端，不与 Session row 主动作竞争。
  **来源:** 增补（content blueprint Session Panel §4）。
- Permission Reminders：正常未安装为中性 `Install for Claude & Codex`；partial/error 才出现 `Repair installation` / `Try again`；不使用 attention coral。
  **来源:** 增补（content blueprint Settings §4 + attention 用法边界）。
- 危险动作：`Quit WhoNeedsMe` 只在 Footer / system menu，退出不影响 Agent；不使用与 Needs You 相同的 coral。
  **来源:** 增补（content blueprint Session Panel §4）。

## 7. 文案语域

- 人称：直接使用 `you`，只在 Needs You 事实与信任承诺中出现；不拟人化 Agent 对话。
  **来源:** 增补（Spec #3 / content blueprint）。
- 语气：短、事实式、当前时态；标题一行，辅助信息一行，错误给可恢复动作。
  **来源:** 取自文件 A §4 Answer, Then Compare + blueprint 固定文案。
- 数字：使用真实 `{N}` / `{M}` / Elapsed；单复数正确；禁止假精确 mock 数字进入成品 fixture 文案。
  **来源:** 增补（真实 Engine 数据与 banned-tells）。
- 状态文案：`Waiting for your approval` / `Waiting for your answer` / `Waiting for plan confirmation`；不显示不确定性标签。
  **来源:** 增补（content blueprint + CONTEXT.md）。
- 项目别名说明：`Only changes the name shown in WhoNeedsMe.`
  **来源:** 增补（content blueprint）。

## 8. 动效

- 允许属性：`transform`、`opacity`；不动画 `width`、`height` 或列表布局。
  **来源:** 增补（approved Cat Asset Brief + 性能/布局稳定要求）。
- Tray：calm / working / attention 三张静态 frame，原子替换，无动画文件。
  **来源:** Phase 5 自定（用户已确认）。
- Panel Header：aggregate state change 播放一次 `200ms`，**整只猫 fade/scale 级**（仅 `transform`/`opacity`）。
  **来源:** 2026-07-17 栅格化后自定（按部件 SVG group 动效已随 raster pivot 取消；easing 为 `cubic-bezier(0.2, 0.8, 0.2, 1)`）。
- Alert：出现时整只猫一次 fade/scale 入场；不循环；Alert 内容与 CTA 不随猫位移。
  **来源:** 2026-07-17 栅格化后自定（整只猫级，非按部件）。
- Empty：静态 sleeping pose。
  **来源:** Phase 5 自定（用户已确认）。
- reduced motion：所有 transition 变成立即静态 state swap。
  **来源:** 增补（Cat Asset Brief 的 accessibility 硬约束）。

## 9. 复合色 token

```css
:root {
  --color-attention-hover: color-mix(in srgb, var(--color-attention) 88%, white);
  --color-attention-pressed: color-mix(in srgb, var(--color-attention) 82%, black);
  --color-row-focus: color-mix(in srgb, var(--color-text-primary) 46%, transparent);
  --color-neutral-hover: color-mix(in srgb, var(--color-text-primary) 7%, transparent);
  --color-disabled-surface: color-mix(in srgb, var(--color-text-disabled) 8%, transparent);
  --motion-state-duration: 200ms;
  --motion-state-easing: cubic-bezier(0.2, 0.8, 0.2, 1);
}
```

**来源:** 增补（单一 attention + neutral focus 纪律；复合态不用额外硬编码 hex）。

## 10. Cat Asset Contract

### 10.1 角色一致性

- 同一只猫的 silhouette、耳朵比例、眼睛语言、line weight 必须跨 micro / compact / illustration 保持一致。
- 猫只在 Tray、Panel Header、Alert Card、Empty State；不得进入 SessionRow、ProviderGlyph 或 project group。
- calm / working 为中性单色；attention 才用暖珊瑚；attention 不能复用 sleeping pose。

**来源:** 增补（ADR-0008、content blueprint、approved Cat Asset Brief）。

### 10.2 必备素材

> **2026-07-17 栅格化**：改为 gpt-image-2 栅格母版（character sheet 作 `--ref`）经 `/produce-assets` 抠透明底 + resize；不再手画分层 SVG，Tray 不再要求 template 单色。

| Detail level | Surface | States | Master | Runtime |
|---|---|---|---|---|
| micro | Tray | calm / working / attention | gpt-image-2 raster master | 全色栅格透明 PNG @1x/@2x（含 attention coral） |
| compact | Panel Header | calm / working / attention | gpt-image-2 raster master | 透明 WebP @1x/@2x |
| compact | Alert Card | attention | same compact character | 复用 compact attention WebP |
| illustration | Empty State | sleeping calm | gpt-image-2 raster master | 透明 WebP @1x/@2x |

版本控制稳定目录：`assets/cat/{masters,micro,compact,illustration}/`；validator：`npm run check:cat-assets`。

### 10.3 栅格母版处理

母版为纯品红底全色栅格，经 `cutout_solid` 抠透明底 + 去污（无白边）→ 为暗色夜蓝面板 retone 成浅色线条图（attention 保留 coral 强调）→ alpha-bbox trim → LANCZOS resize；@2x 恰为 @1x 两倍。**不再拆 SVG group**（整只猫级动效）。构建配方：`scripts/build-cat-assets.py`。

### 10.4 开工前证据

1. 人审通过的 character sheet：`artifacts/design-qa/cat-assets/who-needs-me-cat-character-sheet-v1.png`（2026-07-17，Kun approved）。
2. gpt-image-2 栅格母版：`artifacts/design-qa/cat-assets/generation/`（committed provenance）。
3. Tray 普通/高密度显示 scale 实机截图（真实 macOS menu bar，停靠 Kun 实机补）。
4. 无标签 state contact sheet：`artifacts/design-qa/cat-assets/contact-sheet.png`。
5. Panel state change / Alert entrance 的 start/end frame、duration、easing、reduced-motion note：`artifacts/design-qa/cat-assets/motion/`。

## 11. Surface 当前应用

### Session Panel

- Header current conclusion → global Waiting rows → Other Sessions grouped by project family → Footer。
- Waiting 每行始终显示 project group；Other group 用 heading + hairline，不做 card。
- SessionRow 固定轨道为 ProviderGlyph → Session/branch → current action → project group → elapsed；窄宽优先省略 current action。
- ProviderGlyph 使用约 16px 的 Claude Code / Codex 官方图标，统一视觉重量并保留文字/无障碍名称；获批 masters 与消费规则见 `docs/design/assets/provider-glyphs/README.md`；终端 App 只在 tooltip/detail 表达。
- Empty/Error 替代或保留 last-known list，遵循 content blueprint 固定文案。

### Permission Reminders Settings

- 普通 Settings section，不做 hero、不进 Panel 提示。
- Optional explanation → one install/repair action → trust note；健康时折叠 provider details，失败才展开。

### Alert Card

- Single：`{project} needs you` + reason/elapsed + `Open panel`。
- Multiple：`{N} sessions need you` + `{M} projects` + `Open panel`。
- 猫不改变卡片信息层级，不循环动。

### Tray Icon

- calm / working / attention 三张全色栅格透明 PNG（非 template 单色）；attention 带 coral 强调。
- 已接受风险：全色栅格在浅色菜单栏重着色不如 template 干净——Kun 明确选择全栅格，实机由 Kun 验证。
- Tooltip 只解释 aggregate state；点击一律开 Panel。

## 12. 备注

硬约束：blueprint content order、所有 Waiting 全局置顶、暖珊瑚只服务 attention、V1 夜蓝 dark-only、Provider CLI 图标是受控信息位、猫不进 Session rows、Hook 不进 Panel/Tray/Alert、只读动作边界、Cat 动效范围。

实现量级：`6/9/15px` radius、`4/8/12/16/24px` spacing、`20/15/13/11px` type scale、双层 popover shadow、`200ms` state motion。需要偏离时先回根 `DESIGN.md` 的例外协议。

明确不取：浅色/cyan palette、封面纹理进入列表、`View all sessions`、每项目独立 card、单层纯黑 shadow、过饱和/多 accent、无意义图标、连续 mascot loop、GIF、Lottie。

## 13. 实现前人审门禁

- [x] Phase 5 确认三旋钮与全部取/不取项。
- [x] Phase 5 确认 radius / spacing / type scale、双层 shadow 与 `200ms` motion。
- [x] 根 `DESIGN.md` 已同步 project grouping、Settings、Footer、ProviderGlyph、Cat Asset 与验证矩阵。
- [x] 生成并人审 Cat character sheet；角色方向已于 2026-07-17 通过，后续 SVG masters 以该母版为一致性锚点。
- [x] 取得 Claude Code / Codex 官方候选并人审 16px contact sheet；2026-07-17 视觉选择为 Claude Code A（官方 Spark command glyph）与 Codex A（官方 dark-mode Codex App icon）。来源与授权调查见 `docs/research/provider-cli-icons.md`。
- [x] Kun 于 2026-07-17 确认已取得上述 A / A 官方原件用于 WhoNeedsMe 第三方产品与公开仓库的使用及再分发授权；owner attestation、masters、来源 hash 与消费规则见 `docs/design/assets/provider-glyphs/README.md`。
- [ ] 以真实 macOS menu bar 验证 micro cat 的 template/non-template state swap。
