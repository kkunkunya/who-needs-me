# WhoNeedsMe

> A tiny status board for coding agents. 一个极简的 Coding Agent 状态面板。

同时跑着好几个 Claude Code / Codex 会话，却总忘了哪个已经做完、哪个卡住在等你？WhoNeedsMe 只回答一个问题：**谁正在等我？**

- 五列界面：`Agent · Project · State · Elapsed · Needs You`
- 托盘常驻，有会话等你时图标高亮并置顶该行
- 只读、本地运行——不接管、不编排、不保存对话

🚧 早期开发中（work in progress）。

## Walking skeleton

V1 地基由一个纯 Rust core crate、Tauri v2 壳和静态 TypeScript 调试列表组成。前端在启动前编译为 `dist/`，Tauri 直接加载静态资源；项目不启动 Vite、本地 HTTP 或 socket 服务。

```sh
npm install
cargo install tauri-cli --version "^2.0.0" --locked
cargo test
WHO_NEEDS_ME_FIXTURE_ROOT=crates/who-needs-me-core/tests/fixtures/dual-adapter cargo tauri dev
```

不设置 `WHO_NEEDS_ME_FIXTURE_ROOT` 时，生产 `Environment` 默认读取 Claude Code 与 Codex CLI 的本机会话根，并使用真实进程存活探针；测试与手工验收可显式注入包含 `<provider>/sessions` 与 `<provider>/hooks` 的 fixture 根和受控探针。
