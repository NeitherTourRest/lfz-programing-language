# release-manager — 工作状态
> 最后更新: 2026-09-24 08:52 by release-manager
## 当前状态
**P3.13 两个原子提交（`TypeMsg::ImmutableRebind` 落地 + A9 规范裁定）入库并推送至 origin**（本地 HEAD = `refs/heads/main`）。
- **本轮提交 1（代码）** = `feat(p3): add TypeMsg::ImmutableRebind for let rebinding`；入库 = `src/error.rs` + core-dev `{STATUS,JOURNAL}.md`（短哈希见汇报）。
  - `src/error.rs` 新增变体 `TypeMsg::ImmutableRebind { name: String }` + `message()` 分支；枚举文档注释「6 行」→「7 行」；**不改** `class_name()`（`TypeMsg` 仍映射 `"TypeError"`）。
  - **bug-06 解锁前置已就绪**：runtime-dev 可接线 `exec_assign` + 移除负例 `let_rebind_is_type_error` 的 `#[ignore]`。
- **本轮提交 2（规范）** = `fix(p3): statement-initial brace is an anonymous struct literal (A9)`；入库 = `src/parser.rs` + `.opencode/team/DECISIONS.md`（+ 本角色 `{STATUS,JOURNAL}.md`）。
  - **A9 已生效**：语句首 `{ "k": 1 }` → `{k: 1}` exit 0；`{ let x = 1 }` → `SyntaxError` exit 2；LFZ 无裸块语句，语句首 `{` 解析为匿名 struct 字面量；3 条 legacy 测试更新附论证 + 新增 lexer 级回归。
- **push 前本地/远程基线** = `316abce`（`docs(spec): fix §9.4 sample ... and rule on A9`）。
- **⚠️ 仍严禁打 tag**：`v0.2.0` 待 **bug-06 接线完成 + 终验通过**后由 team-lead 判定；tag 仍仅 `v0.1.0`。
- **未 force-push、未打任何 tag**。`git add` 全为**显式清单**；工作区 5 项改动**与任务书清单逐项一致、无清单外条目**（未出现 `src/evaluator.rs`，无需回避；入清单门禁未触发）。
## 进行中
- （无）
## 阻塞 / 需要支持
- **bug-06 剩余工作**：`TypeMsg::ImmutableRebind` 已落地（提交 1）；仍待 **runtime-dev 接线 `exec_assign`** + 移除 `#[ignore]` 后复验，方可判 `v0.2.0`。
- **owner 偏差（历史遗留，待确认）**：可认证账号 login = `NeitherTourRest`（display name = `MakeChase`，id 180032968）。待 team-lead/用户确认；不影响本轮推送。
## 下一步计划
- 待 bug-06 接线 + 终验通过 + team-lead 判定 → 打附注标签 `v0.2.0`。
- P10：交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项）。
## 关键经验（写给未来的自己）
- **时间戳以 `git log --format=%ci` 为准**：系统时钟（`Get-Date`）与早前部分日志存在偏差；写日志时间戳时以真实提交时间为锚，避免出现「新条目时间早于旧条目」的假象。
- **并行修复期的暂存纪律**：`git add` 一律**显式文件清单**；禁 `git add -A`/`git add src`。本轮清单恰 5 项，`git status --short` 逐项核对后未触发门禁；`src/evaluator.rs` 未出现（若出现则**不暂存、不中止**）。
- **提交体量数字核对**：`core.autocrlf=false` 下 `git commit` 自身打印的 insertions/deletions 会因行尾转换**虚高**（CRLF 仓库）；判据一律以 `git show --stat HEAD` 为准。
- **多行提交信息 + 非 ASCII 字符**：body 含 `§`/`<`/`>`/中文时，写 UTF-8 信息文件（`C:\Users\19170\AppData\Local\Temp\opencode\lfz_msg*.txt`）→ `git -c core.autocrlf=false commit -F <file>`，提交后 `git log -1 --format=%B` 复核正文完整。
- **账号身份**：GitHub URL 用 login `NeitherTourRest`（**非** display name `MakeChase`）。
- **代理纪律**：`git push` 走仓库自身 `http.proxy`/`https.proxy=http://127.0.0.1:7890`；PowerShell 下 `git push` 把进度写 stderr 会显示红色 `NativeCommandError`，判据是 `$LASTEXITCODE=0`。
- **基线事实**：上一轮 HEAD = `316abce`；本轮两新提交位于其上；附注标签 `v0.1.0` 指向 `e060b21`；远程 `refs/heads/main` = 本轮 HEAD（push 后）。
- **禁区确认**：本轮未 force-push、未打 tag、未 `git add -A`、未暂存清单外文件、未提交 `target/`/密钥/临时文件。
