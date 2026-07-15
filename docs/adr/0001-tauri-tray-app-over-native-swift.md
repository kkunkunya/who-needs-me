# ADR-0001: 产品形态用 Tauri 托盘应用，不用原生 Swift/SwiftUI

- status: accepted
- date: 2026-07-11

## 背景

产品简报（who-needs-me-product-brief.md 第 7、8 节）首选原生 Swift + SwiftUI menu bar app，并明确"不建议第一版做浏览器 Dashboard"。但用户有明确的后续诉求：下一个迭代要兼容 Windows。同时项目以 Agent 驱动开发为主，需要 Agent 能快速迭代并自查 UI 效果。

## 决策

产品形态采用 **Tauri**：Rust 壳 + 原生系统托盘 + web 技术（TypeScript）渲染的 UI。不采用原生 Swift/SwiftUI，也不做纯 localhost 网页。

## 理由

- **保住核心交互**：Tauri 原生托盘支持 macOS 菜单栏与 Windows 托盘——图标常驻、显示 Agent 数量、有人等待时高亮，简报的产品灵魂不丢。
- **Windows 兼容几乎免费**：UI 层跨平台复用，第二版只需补 Windows 端的进程检测适配；Swift 方案则要整体重写。
- **Agent 开发效率**：web UI 可用浏览器直接截图自查，编译迭代快；SwiftUI 对无人值守 Agent 的验证回路不友好。
- **被否备选**：
  - 原生 Swift/SwiftUI——最纯正 macOS 体验，但永久锁定 macOS 且分发需 Apple 签名公证；
  - 纯 localhost 网页——开发最快、天生跨平台，但没有托盘常驻提醒，"谁在等我"退化成"我得记得去查"，放弃最锋利的卖点。

## 后果

- 技术栈定为 Rust（壳/采集）+ TypeScript（UI），需要 Rust 工具链。
- 偏离简报的 Swift 首选路线；后续讨论不再重开此题，除非用户主动推翻。
- 简报第 8 节"不启动本地 Web 服务"等技术建议是按 Swift 形态写的，事件采集通道等架构决策需按 Tauri 形态重新评估（另行 ADR）。
