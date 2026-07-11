# Issue tracker：GitHub

本仓库用 **GitHub issues**（`kkunkunya/who-needs-me`）做 issue tracker。所有操作用 `gh` CLI。本文档告诉其他 skill（如 `code-review` 的 Spec 轴、`triage`、`wayfinder`）怎么读写这张 issue tracker——它是这些 skill 与具体 issue tracker 之间的**适配层**：skill 只说抽象术语（"fetch the relevant ticket"），本文档翻译成具体 `gh` 命令。

仓库归属从 `git remote -v` 推断——`gh` 在 clone 内运行时自动识别，零配置。

## Issue 操作

- **建 issue**：`gh issue create --title "..." --body "..."`。多行 body 用 heredoc。
- **读 issue**（含评论与标签）：`gh issue view <number> --comments`，需要结构化时加 `--json number,title,body,labels,comments --jq '...'`。
- **列 issue**：`gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'`，按需加 `--label` / `--state` 过滤。
- **评论**：`gh issue comment <number> --body "..."`
- **加 / 去标签**：`gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **关闭**：`gh issue close <number> --comment "..."`

## Pull request 操作

- **读 PR**（含评论）：`gh pr view <number> --comments`
- **读 PR diff**：`gh pr diff <number>`
- **列 PR**：`gh pr list --state open --json number,title,body,labels,headRefName,baseRefName,author,authorAssociation,comments`
- **评论 / 标签 / 关闭**：`gh pr comment`、`gh pr edit --add-label` / `--remove-label`、`gh pr close`

GitHub 的 issue 和 PR **共享编号空间**，裸 `#42` 可能是任意一种——用 `gh pr view 42` 和 `gh issue view 42` 各试一次来分辨。

## PR 作为 triage surface

**PR 作为请求入口：no。**

> `no` = owner 自行开发，外部 PR 不作为 feature request。`yes` = 接受外部 PR 作为需求入口，`/triage` 读这个标志。项目开放外部贡献后可改为 `yes`。

设为 `yes` 时，PR 走和 issue 同样的标签与状态机，用 `gh pr` 等价命令；列外部 PR 时只保留 `authorAssociation` 为 `CONTRIBUTOR` / `FIRST_TIME_CONTRIBUTOR` / `NONE` 的（丢掉 `OWNER` / `MEMBER` / `COLLABORATOR`）。

## Sub-issue 与依赖（blocking）关系

`to-issues` 把一份 PRD 拆成子 issue 时，要把每片链到 parent 作 **sub-issue**、把依赖接成 **blocking edge**。GitHub 原生支持（gh CLI v2.94+，2026-06 起的 Issues 2.0）：

- **建 issue 时直接挂关系**：
  - `gh issue create --title "..." --body "..." --parent <PARENT-N>`——作为 PARENT-N 的 sub-issue
  - `--blocked-by <N1,N2>`——标记被这些 issue 阻塞
  - `--blocking <N>`——标记本 issue 阻塞 N
- **给已有 issue 接关系**：
  - `gh issue edit <n> --add-sub-issue <child>` / `--remove-sub-issue <child>`
  - `gh issue edit <n> --set-parent <PARENT-N>` / `--remove-parent`
  - `gh issue edit <n> --add-blocked-by <blocker>` / `--remove-blocked-by <blocker>`
  - `gh issue edit <n> --add-blocking <N>` / `--remove-blocking <N>`
- 编号支持跨仓库 URL；`gh issue view <n>` 会显示 parent / sub-issues（带完成进度）/ blocked-by / blocking。

**旧 gh 不支持原生关系时**，用 issue body 的 `## Parent` 和 `## Blocked by` 段兜底（写 issue 编号引用），`to-issues` 已规定这个 fallback。

## skill 术语映射

- **"publish to the issue tracker"** → 建一个 GitHub issue：`gh issue create`。
- **"fetch the relevant ticket"** → `gh issue view <number> --comments`。
- **"link as a native sub-issue" / "wire a native blocking edge"**（`to-issues`）→ `gh issue create --parent` / `--blocked-by`，或 `gh issue edit --add-sub-issue` / `--add-blocked-by`；不支持就用 body 的 `## Parent` / `## Blocked by` 段兜底。见上"Sub-issue 与依赖关系"段。
- **"the spec for this change"**（`code-review` 的 Spec 轴）→ 从 commit / PR 的 issue 引用（`Closes #<n>` / `Part of #<n>` / `Refs #<n>`）找到驱动 issue，读其正文与评论作为 spec。**只到 issue 这一层**——不要回溯 issue 之前的 discussion / 对话记录，那是未收敛的噪声。都没找到就问用户 spec 在哪；用户说没有，Spec 轴报 "no spec available"。
