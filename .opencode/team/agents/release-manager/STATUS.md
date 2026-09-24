# release-manager — 工作状态
> 最后更新: 2026-09-24 10:45 by release-manager
## 当前状态
**用户下令暂停开发、自行验收 → 本轮把工作区收拾到「干净且已推送」：team-lead 交付的 P3 后状态/日志入库，并给出用户自助验收基线**。
- 本轮 **1 个提交**（入清单门禁通过）：`docs(team): sync team-lead status/journal after P3`。
  - 入库内容 = team-lead 撰写（**仅代为入库、未改内容**）的 `.opencode/team/agents/team-lead/{STATUS,JOURNAL}.md` + 本角色收工 `{STATUS,JOURNAL}.md`（并入同一提交，工作区保持干净）。
- **未 force-push、未打任何新 tag**（`v0.1.0`/`v0.2.0` 保留且远程一致）；`git add` 全为**显式清单**，未 `git add -A`。
## 校验基线（交付给用户验收）
- 入清单门禁：`git status --short` 恰 **2 项**（`M .opencode/team/agents/team-lead/JOURNAL.md`、`M .opencode/team/agents/team-lead/STATUS.md`），与任务书逐字一致，**无清单外条目、无未跟踪文件、无 stash**。
- 提交前基线 HEAD = 远程 `refs/heads/main` = `931e2b2`；`git tag -n` = `v0.1.0` + `v0.2.0`（均附注标签）。
- 终态：`git status --short` 空；本地 HEAD = `git ls-remote origin refs/heads/main`。
- `cargo build` → 0 warning / 0 error；`cargo test` → 库 **361 passed / 0 failed / 0 ignored**（+ bin 9 + cli 7 = 377）；`cargo run --quiet -- run examples/hello.lfz` → `Hello, LFZ!`（exit 0）。
## 进行中
- （无）
## 阻塞 / 需要支持
- **owner 偏差（历史遗留，待确认）**：可认证账号 login = `NeitherTourRest`（display name = `MakeChase`）。待 team-lead/用户确认；不影响交付。
## 下一步计划
- 用户验收通过后：P10 交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项，至少 3 轮）。
- 后续阶段里程碑（P5 `v0.3-tested` / P8 `v0.4-app` / P9/P10 `v1.0-final`）按需打附注标签（须 team-lead 授权）。
## 关键经验（写给未来的自己）
- **暂停/交付场景 = 「干净且已推送」是第一要务**：提交前先 `git status --short` 逐项核对清单，出现清单外条目即**停止并汇报**（用户即将验收，树必须干净可解释）。
- **时间戳以 `git log --format=%ci` 为准**；`git commit` 自身打印的 insertions/deletions 受 `core.autocrlf` 影响虚高，判据以 `git show --stat HEAD` 为准。
- **暂存纪律**：`git add` 一律**显式文件清单**；禁 `git add -A`；他人撰写文档**仅代为入库、不改内容**。
- **代理纪律**：`git push` 走仓库自身 `http.proxy`/`https.proxy=http://127.0.0.1:7890`；PowerShell 下进度写 stderr 显示红色 `NativeCommandError`，判据是 `$LASTEXITCODE=0`。
- **禁区确认**：本轮未 force-push、未打新 tag、未 `git add -A`、未暂存清单外文件、未提交 `target/`·密钥·临时文件；未改动 `src/**`、`docs/spec/**`。
