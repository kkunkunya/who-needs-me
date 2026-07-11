# Triage Labels

skill 用 **2 个类别角色** + **5 个状态角色**说话。每个 triaged issue 带恰好一个类别 + 一个状态。本文件把这些角色映射到本仓库 issue tracker 里实际用的标签字符串——本仓库**用默认标签**（角色名直接当标签字符串）。

## 类别（category）——每个 issue 恰好一个

| skill 里的标签 | 本 tracker 用的标签 | 含义 |
| --- | --- | --- |
| `bug` | `bug` | 坏了 |
| `enhancement` | `enhancement` | 新功能或改进 |

## 状态（state）——每个 issue 恰好一个

| skill 里的标签 | 本 tracker 用的标签 | 含义 |
| --- | --- | --- |
| `needs-triage` | `needs-triage` | 维护者需评估这个 issue |
| `needs-info` | `needs-info` | 等报告者补更多信息 |
| `ready-for-agent` | `ready-for-agent` | executable leaf 已完全 spec 好并通过领域 readiness，可给无人值守（AFK）agent 接 |
| `ready-for-human` | `ready-for-human` | 需人实现 |
| `wontfix` | `wontfix` | 不会做 |

## 状态流转

```
新 issue（无标签）→ needs-triage
                     ├→ needs-info  ──报告者回复──→ 回 needs-triage
                     ├→ ready-for-agent
                     ├→ ready-for-human
                     └→ wontfix
```

maintainer 可随时 override 状态。PR 若开 PR-as-triage-surface 也走同一机器（PR 是"带代码的 issue"）——本仓库当前为 no，见 `issue-tracker.md`。

## 怎么用

当 skill 提到某个角色（如"打 `ready-for-agent` 标签"），直接用同名标签字符串（`gh issue edit <n> --add-label "<标签>"`，命令见 `issue-tracker.md`）。
