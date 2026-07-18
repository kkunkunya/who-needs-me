# WhoNeedsMe V1 Reference Set 设计招式表

> status: accepted — 2026-07-17 完成逐项人审
> 配对阅读: `design-reference.md`
> 来源: 用户选定的夜蓝封面艺术 + WhoNeedsMe 暗色概念图 + 三张 macOS menu-bar popover，分析于 2026-07-16—17
> 重点参考集: `docs/design/recon/reference-set.md`
> 模式判定: 从零（Greenfield）——本表记录参考原貌；项目最终 token 在 `design-reference.md`

## 1. Reference Design Map 语义描述

### 1.1 视觉主题

一个 `~approx 380px` logical-width 的 macOS menu-bar utility：顶部先给当前结论，中间用连续高密度状态行回答“谁在等我”，底部才收 freshness 与管理入口。夜蓝封面提供“深夜里安静值守、需要你时亮起一个暖信号”的情绪；暗色概念图提供紧凑单栏构图；macOS popover 提供分组与低频入口纪律。它的性格来自 `~approx 11–16px` 的紧凑字号、`~approx 1px` hairline、面积 `<5%` 的暖珊瑚 signal 和出现在框架位置的猫，而不是 hero、摄影或卡片网格。

### 1.2 调色板

| 颜色 | 参考估值 | 角色 | 视觉面积 |
|---|---|---|---:|
| nocturnal ink | `#0D1627`（`~approx`） | 封面暗部与产品最深背景方向 | `~25%` |
| saturated night blue | `#174DA5`（`~approx`） | 封面主色；实现时主动压暗 | `~45–55%` |
| ice blue | `#73C2EC`（`~approx`） | 封面光区；只转译为冷调层级，不做大面积 UI fill | `~20%` |
| fog white | `#F2F4EF`（`~approx`） | 主文字与远处灯光 | `<10%` |
| signal coral | `#F25561`（`~approx`） | 摄像头/书脊的微小暖信号 | `<1%` |
| hairline | `rgba(180,210,235,0.14)`（项目适配） | section、row、surface 边界 | `<5%` |

项目不逐字复制封面的高亮蓝，而把它压成深墨蓝 / 夜蓝 surface；暖珊瑚只保留为 Needs You signal，面积小于 5%，不铺满 row 或 surface。三张 macOS supporting 的浅色/cyan translucency 只提供结构证据，不属于项目 palette。

### 1.3 字体排印

- Display / body：`SF Pro / system sans`（截图观察），用于 Header、项目名、状态、动作和设置。
- Mono：`SF Mono / system mono`（截图观察），用于 branch、model、shortcut 与工程 metadata。
- 尺度：`~approx 20–22px` current-status heading；`~approx 15–16px` project/action；`~approx 13–14px` state/body；`~approx 11–12px` metadata/footer。
- 字重：`~approx 650–700` heading；`~approx 550–600` project/action；`~approx 400–500` body/metadata。
- 没有数学 display scale，也没有观察到基于 `ch` 的行长控制；这是 4 档紧凑 utility scale。

### 1.4 组件

- Panel outer：`~approx 14–16px` radius，双层黑色 ambient/contact shadow，单列容器。
- Alert outer：`~approx 14–16px` radius，`~approx 2.8:1`，一条事实 + 一行动作。
- Waiting group：`~approx 8–10px` radius；用轻微夜蓝提亮形成边界，暖珊瑚只用于状态点和关键文字。
- State / model chip：`~approx 5–7px` radius，`~approx 11–13px` 文本，边框/底色而非阴影。
- 普通 row / section：零独立 shadow；依赖 heading、padding、`~approx 1px` hairline。

### 1.5 布局

- Container：`~approx 380px` logical width，单列 popover，`~approx 16–20px` 横向 padding。
- Session row：`~approx 5 tracks`——glyph / identity / model / state / elapsed；trailing elapsed 右对齐。
- 间距：`~approx 4 / 8 / 12–16 / 20–24px` 四档；组件内到区块间逐级增加。
- Section：Header current conclusion → Waiting → other rows/groups → Footer；区块主要用 hairline 而非大留白。
- 参考截图均为桌面 menu-bar popover；没有移动端 layout 证据。

### 1.6 生成备注

硬证据：单列紧凑 popover、结论前置、hairline 分组、夜蓝主场、暖珊瑚只服务 attention、系统 sans + mono metadata、猫不进入 Session 行。

软证据：截图估算 radius、shadow、字号和 spacing；项目文件 B 应优先采用根 `DESIGN.md` 的精确值，再用这些估算校验视觉量级。

明确不取：浅色/cyan system material、封面绘画纹理进入数据列表、概念图中的 `View all sessions`、Waiting 时睡猫、每项目独立大卡、无目的图标、连续 mascot 动画。

## 2. Tailwind v4 CSS 变量（参考截图估值）

> 下列值是 Reference Set 截图估值，CSS 语法中省略 `~approx`；它们不是项目最终 token。

```css
@theme {
  --color-reference-ink: #0d1627;
  --color-reference-night-blue: #174da5;
  --color-reference-ice-blue: #73c2ec;
  --color-reference-text: #f2f4ef;
  --color-reference-border: rgba(180, 210, 235, 0.14);
  --color-reference-attention: #f25561;

  --font-reference-sans: -apple-system, BlinkMacSystemFont, "SF Pro Text", sans-serif;
  --font-reference-mono: "SFMono-Regular", ui-monospace, monospace;

  --text-reference-heading: 1.3125rem;
  --text-reference-project: 1rem;
  --text-reference-body: 0.875rem;
  --text-reference-meta: 0.75rem;

  --radius-reference-sm: 6px;
  --radius-reference-md: 9px;
  --radius-reference-lg: 15px;

  --spacing: 4px;
  --shadow-reference-popover: 0 2px 6px rgba(0, 0, 0, 0.28), 0 18px 44px rgba(0, 0, 0, 0.34);
}
```

## 3. Design Tokens（JSON；参考截图估值）

```json
{
  "source": "docs/design/recon/reference-set.md",
  "certainty": "~approx screenshot measurement",
  "color": {
    "ink": { "value": "#0D1627", "role": "reference nocturnal shadow" },
    "nightBlue": { "value": "#174DA5", "role": "reference dominant blue; darken for UI" },
    "iceBlue": { "value": "#73C2EC", "role": "reference light plane; do not use as surface fill" },
    "text": { "value": "#F2F4EF", "role": "reference fog-white text" },
    "border": { "value": "rgba(180,210,235,0.14)", "role": "adapted hairline" },
    "attention": { "value": "#F25561", "role": "warm coral signal only; under 5% area" }
  },
  "font": {
    "sans": "SF Pro / system sans",
    "mono": "SF Mono / system mono",
    "scale": [
      { "role": "current-status-heading", "size": "20-22px", "weight": 650 },
      { "role": "project-title-action", "size": "15-16px", "weight": 550 },
      { "role": "state-body", "size": "13-14px", "weight": 400 },
      { "role": "metadata-footer", "size": "11-12px", "weight": 400 }
    ]
  },
  "radius": { "sm": "6px", "md": "9px", "lg": "15px" },
  "spacing": { "base": "4px", "scale": ["4px", "8px", "12px", "16px", "24px"] },
  "shadow": {
    "popover": "0 2px 6px rgba(0,0,0,0.28), 0 18px 44px rgba(0,0,0,0.34)"
  },
  "imageRatios": [
    { "usage": "functional glyph", "ratio": "1:1" },
    { "usage": "alert", "ratio": "2.8:1" },
    { "usage": "panel", "ratio": "0.95:1" }
  ],
  "grid": {
    "maxWidth": "380px logical",
    "columns": 1,
    "sessionRowTracks": 5,
    "gutter": "16px"
  }
}
```

## 4. 风格纪律招式表

### Taste DNA

### Signal Earns Color

- Trigger: 当多个状态、品牌元素与动作共享一个窄 surface 时。
- Decision: 只给 Needs You 使用暖珊瑚，而不是把 signal 扩散到普通标题、Working、Idle 与装饰。
- Reason: 用户可以把颜色当成可信的行动信号，不必逐行重新解释。
- Evidence: 封面暖珊瑚 `#F25561`（`~approx`）只占微小信号面积；其余画面为夜蓝与冷白。

### Sections Before Cards

- Trigger: 当多个项目组与状态组必须装进一个紧凑 popover 时。
- Decision: 使用 section heading、padding 与 hairline，而不是每组一个圆角卡片。
- Reason: 用户连续扫描一张状态表，而不是跨越多个视觉孤岛。
- Evidence: `~approx 1px` hairline；普通行零感知阴影；只有 Waiting 有组容器。

### Answer, Then Compare

- Trigger: 当用户只愿意花一眼判断是否中断当前工作时。
- Decision: 先显示 current conclusion，再显示固定轴 Session 行，而不是直接抛出无摘要的自由列表。
- Reason: 第一眼回答“是否需要我”，第二眼比较“哪个项目、等什么、多久”。
- Evidence: Header-to-list `~approx 20–24px`；Session row `~approx 5 tracks`。

### Character Hosts State

- Trigger: 当一只品牌猫需要和高密度工程状态共存时。
- Decision: 猫只出现在 Tray、Header、Alert、Empty，而不是 Session 行或背景装饰。
- Reason: 产品保留角色识别，同时不降低 Session 比较速度。
- Evidence: 无内容摄影；所有图像为 `~approx 1:1` 功能 glyph；猫位于 frame surface。

### 取/不取裁决（已逐项人审）

| 参考集合使用的 | 我们取不取 | 决定 |
|---|---|---|
| 深夜蓝单列 popover | 取 | 封面决定夜蓝情绪，暗色概念图决定菜单栏 utility 构图；项目精确色值由文件 B 决定。 |
| 暖珊瑚面积 `<5%` 且只给 attention | 取 | 保持“颜色即 Needs You”的唯一语义；Waiting row 只用夜蓝提亮，不铺珊瑚底。 |
| Header 先给当前结论 | 取 | 直接承载 blueprint 的一眼价值。 |
| 普通项目组用 heading + hairline，不做独立卡片 | 取 | 避免 banned-tells 中的默认卡片化，并提高窄面板扫描连续性。 |
| Waiting 使用 tint / group boundary | 借鉴 | 取“形成一个全局 P1 区域”，不复制会把它变成卡内卡的重 outline。 |
| `~5 tracks` 固定 Session 横轴 | 借鉴 | 取对齐纪律；字段缺失与 380px 宽度下允许 CSS grid track 收缩/隐藏可选 model。 |
| 双层 popover shadow | 取 | 参考中确有 ambient + contact 深度；避免单层纯黑默认阴影。 |
| macOS supporting 的浅色/cyan translucency | 不取 | 项目 accepted `DESIGN.md` 明确 V1 dark-only；supporting 只负责结构。 |
| primary 的 `View all sessions` | 不取 | content blueprint 已确认 Panel 就是全部在榜 Session，没有目标 surface。 |
| primary 的 Waiting 时睡猫 | 不取 | 与 approved Cat Asset Brief 的 attention 姿态语义冲突。 |
| 猫出现在 Tray/Header/Alert/Empty | 取 | 符合 ADR-0008 与 blueprint；仍禁止进入 Session 行。 |
| settings 的分区、footer 与低频动作位置 | 借鉴 | 取 macOS utility 的管理层级，不复制 Amphetamine 业务、浅色 palette 或持续强提示。 |
| Provider CLI 官方图标 | 取（受控） | 行首约 16px，只表达真实 Provider CLI，统一视觉重量并保留文字/无障碍名称；终端 App 退为次级信息。 |

## 5. 参考集合没有但本项目需要的（增补）

- Cat Asset Brief：micro / compact / illustration 三档；calm / working / attention；Tray 静态，Panel/Alert 一次性微动效，Empty 静态。
- 分层 SVG 与 reduced-motion：稳定的 `eyes / ears / head / body / tail / accent` groups；160–240ms transform/opacity；静态回退。
- 项目组识别与本地别名：Waiting 行显示项目组；Other Sessions 按 Git repository/worktree family 分组；Rename/Reset 轻量菜单。
- Empty 与 stale/error：无活跃 Session、保留 last-known snapshot、`Try again`。
- Permission Reminders：optional / installed / partial / repair / error；失败时才展开 Provider 详情。
- 真实 focus-visible、键盘导航、popover scrolling 与屏幕阅读标签。
- Tray 小尺寸实机验证：template-safe calm/working + coral attention export，普通/高密度显示器均需人审。
- ProviderGlyph 资产：Claude Code / Codex 从可追溯官方来源取得，记录授权说明，制作 16px contact sheet；失败时使用中性 monogram。
