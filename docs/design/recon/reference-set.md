# Reference Set — WhoNeedsMe V1

> status: accepted — 2026-07-17
> 内容依据：`docs/design/content-blueprint.md` 的 Session Panel、Permission Reminders Settings、Alert Card、Tray Icon
> 截图 brief：`.design-recon/screenshot-brief.md`（本地工作材料，不进 Git）
> 原始候选：`.design-recon/inbox/`（不进 Git）

## Candidate audit

| Candidate | 结论 | 画面 / 状态 | 主要布局 | 最值得借鉴的问题 | 蓝图映射 |
|---|---|---|---|---|---|
| `nocturnal-blue-cover-color-primary.png` | 接受 | 夜蓝空间、黑猫与白色陪伴者、极小暖珊瑚信号 | 大面积冷色空间 + 单一暖焦点 | 产品色彩、安静值守情绪、猫的非萌系陪伴气质 | 全局 palette / Cat mood |
| `panel-alert-tray-dark-primary.png` | 接受 | 暗色 mixed Panel、2-session Alert、Tray cat | 顶部结论、Waiting block、状态行、footer、独立 Alert | 紧凑单栏构图、Waiting 权重、Panel/Alert/Tray 同一语域 | Session Panel、Alert Card、Tray Icon |
| `project-groups-macos-supporting.png` | 接受 | macOS Wi-Fi popover | 顶部状态、section heading、紧凑行、footer settings | 项目组标题与组内行如何不依赖卡片分组 | Session Panel / Other Sessions |
| `status-summary-footer-macos-supporting.png` | 接受 | macOS Battery popover | 当前结论、分区列表、末尾 settings | Header 先给结论、Footer 收低频入口 | Session Panel / Header + Footer |
| `permission-settings-macos-primary.png` | 接受 | 第三方 menu-bar utility settings | 当前状态、checkbox/action、分隔区、settings/about/quit | 可选设置与恢复动作如何留在低频区域 | Permission Reminders Settings |

全部候选都清晰、完整、非重复，并能映射到至少一个目标 surface；本轮没有拒绝项。封面只作为参考证据，不是产品运行时资产。

## 全局视觉语言

| 优先级 | 文件 | 重点参考 | 明确不参考 |
|---|---|---|---|
| primary — palette / mood | `nocturnal-blue-cover-color-primary.png` | 深夜蓝、冷白、微小暖珊瑚信号；安静值守与非萌系猫气质 | 书籍文字、机器人角色、绘画纹理、明亮蓝色直接铺满 UI |
| primary — composition | `panel-alert-tray-dark-primary.png` | 单列暗色 surface、hairline、高密度状态行、Panel/Alert/Tray 同源构图 | `View all sessions`、具体示例文案、Waiting 时睡猫、背景壁纸 |
| supporting — grouping | `project-groups-macos-supporting.png` | 系统级 section heading、无卡片分组、行高与分隔节奏 | 浅色 palette、Wi-Fi 品牌/开关/锁图标、系统背景材质 |
| supporting — summary/footer | `status-summary-footer-macos-supporting.png` | 当前结论优先、状态分区、footer settings 入口 | 浅色 palette、电池业务信息、彩色 App 图标 |

## 目标映射

| 目标 surface | 角色 | 文件 | 只借什么 | 冲突裁决 |
|---|---|---|---|---|
| Global palette / mood | primary | `nocturnal-blue-cover-color-primary.png` | 夜蓝主场、冷白层级、暖珊瑚 signal `<5%`、安静值守 | 项目 token 主动压暗封面高亮蓝；不复制纹理或角色 |
| Session Panel | primary | `panel-alert-tray-dark-primary.png` | Panel 构图、暗色密度、Waiting 权重、状态行节奏 | 内容顺序服从 blueprint；palette 服从封面与 design reference |
| Session Panel / Other Sessions | supporting | `project-groups-macos-supporting.png` | section heading、组间分隔、紧凑行层级 | 只覆盖项目组表现，不覆盖 palette 或全局 Waiting 排序 |
| Session Panel / Header + Footer | supporting | `status-summary-footer-macos-supporting.png` | 当前结论前置、footer 收 Settings 的内容框架 | 只覆盖 Header/Footer 层级，不覆盖暗色语域 |
| Permission Reminders Settings | primary | `permission-settings-macos-primary.png` | 可选设置的分区、动作与低频恢复层级 | 只借结构与密度；不复制浅色材质、业务文案或持续强提示 |
| Alert Card | primary | `panel-alert-tray-dark-primary.png` | 单事实 + 一行动作的紧凑卡片 | 蓝图的 single/multiple 文案胜出；Alert 不显示完整列表 |
| Tray Icon | primary | `panel-alert-tray-dark-primary.png` | 猫作为 menu-bar 聚合 glyph 的位置与状态关系 | Cat Asset Contract 胜出；不采用 Waiting 时睡猫 |
| Cat asset family | primary | `nocturnal-blue-cover-color-primary.png` + `panel-alert-tray-dark-primary.png` | 前者给黑猫剪影与陪伴气质，后者给 Tray/Header/Alert 的角色位置 | 只作为角色方向；姿态、detail level、动画与导出服从 Cat Asset Contract |

## 冲突裁决

1. **配色**：夜蓝封面负责 palette / mood；暗色概念图负责 composition；macOS 截图不引入浅色 / translucent palette。
2. **Waiting 排序**：content blueprint 胜出。所有 Waiting 跨项目全局置顶，且每行说明项目组。
3. **Waiting 背景**：使用轻微夜蓝提亮形成 P1 区域；暖珊瑚只用于状态点、关键文字和猫的 attention 细节，不铺满 row。
4. **项目分组**：Wi-Fi supporting 只补 section heading 与分隔；不把每个项目做成独立大卡。
5. **Footer**：content blueprint 胜出。删除 `View all sessions`，使用 freshness、Settings、快捷键与 Quit。
6. **猫状态**：Cat Asset Contract 胜出。attention 不能使用睡猫；Tray 静态三态，Panel/Alert 只允许一次 `200ms` SVG 微动效，Empty 静态。
7. **Hook 权重**：content blueprint 胜出。Hook 只在 Settings 中作为可选增强，不进入 Panel/Tray/Alert 标记。
8. **Provider 图标**：允许真实 Claude Code / Codex 官方图标作为约 16px 行首信息位；统一视觉重量、配文字与无障碍名称，终端 App 退为次级信息。

## 未覆盖与实现门禁

- Cat calm / working / attention 的完整 character sheet、layered SVG 与 Tray 小尺寸可读性，必须单独生成人审。
- Claude Code / Codex 官方图标必须记录来源与授权说明，并在 16px contact sheet 中人审；无法取得时使用中性 monogram。
- Session Panel empty、stale/error 与 long-project-name 使用 blueprint fixture 补齐，不从无关参考拼接。
- `Rename project…` / `Reset to detected name` 使用 macOS-native 轻量菜单与输入状态，不扩成项目管理页。
- Permission Reminders 的 partial / repair / error 以低频分区纪律增补，健康时折叠 Provider 详情。
- Panel Header / Alert 只动画 `transform` / `opacity`；reduced-motion 立即静态换帧。
