# release-manager — 工作状态
> 最后更新: 2026-09-24 00:29 by release-manager
## 当前状态
**P3.11 验收报告 + 团队状态刷新：单个原子提交并推送至 origin**（本地 HEAD = `refs/heads/main`）。
- **远程仓库 URL：https://github.com/NeitherTourRest/lfz-programing-language**（Public，默认分支 `main`）。
- **本轮提交（1 个原子提交）**：`docs(team): P3.11 verification report (FAIL: 3 blocking bugs) and board update`；入库文件 = 显式清单：`docs/reports/P3-verification.md` + `docs/reports/fixtures-p3/`（9 个 `.lfz` 夹具）+ `TEAM_BOARD.md` + `PROJECT_STATE.md` + `verifier/{STATUS,JOURNAL}.md` + 本角色 `{STATUS,JOURNAL}.md`。
- **⚠️ P3.11 验收结论 = FAIL**（verifier 独立验收：3×🔴 阻塞 / 4×🟡 / 2×🟢；369 单元测试全绿但 3 个阻塞缺陷：多行块、插值 `format_spec`、if-表达式）。**因此本轮严禁打 tag，`v0.2.0` 不得打**；tag 仍仅 `v0.1.0`。
- ⚠️ **并行修复任务在改 `src/**`**：本轮 `git status --short` 未见 `M src/**`（显式 `git add` 清单未涉及 `src/**`）；任务书允许出现 `src/**` 清单外条目，**出现时不中止、但一律不暂存**——本轮全程 `git add <显式文件>`，未用 `git add -A`/`git add src`。
- ⚠️ **夹具数量事实核对**：任务书称夹具「10 个 `.lfz`」；**实测 `docs/reports/fixtures-p3/` 恰 9 个 `.lfz`**（01–08 + `spec_9_4_refs.lfz`），与验收报告 §1.2 表列一致。以实测为准，如实登记（差 1 项已记入本轮汇报，非缺失交付——报告口径即 9）。
- ⚠️ **owner 偏差（历史遗留，需确认）**：可认证账号 login = `NeitherTourRest`（display name = `MakeChase`，id 180032968）；GitHub 上另一同名账号 `MakeChase`（id 128372141）无权操作。仓库实际落在 `NeitherTourRest` 名下。
- **未 force-push、未改默认分支、未打任何 tag**。
## 进行中
- （无）
## 阻塞 / 需要支持
- **待 team-lead/用户确认 owner 偏差**：若期望仓库落在字面账号 `MakeChase`（id 128372141）名下，需先在该账号完成认证，再 transfer/重建仓库并同步 README——待授权后执行。当前交付在 `NeitherTourRest` 名下、功能完整可访问。
- **P3.11 结论 FAIL**：3 个阻塞缺陷修复 + 复核通过前，`v0.2.0` 标签与 P3 里程碑收口均挂起；由 team-lead 派工对应角色修复，我负责修复后重新提交（不自行修复代码）。
## 下一步计划
- 待 3 个阻塞缺陷修复并经复核后：由 release-manager 提交修复（`fix(...)` 原子提交）+ push；确认通过条件达成后由 team-lead 判定 → 打附注标签 `v0.2.0`。
- 每阶段里程碑：原子提交 + 附注标签 + 实时更新 README + `git push`（含标签）。
- P10：交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项）。
## 关键经验（写给未来的自己）
- **验收 FAIL 阶段禁止打 tag**：只有阶段通过条件达成且有证据（team-lead 判定）才打标签；FAIL 一律不打，标签本身就是"通过"的进度证据，不可虚打。
- **本轮入库口径**：任务书给的是「显式文件清单 + `docs/reports`（目录）」，且**明确允许并行 `src/**` 清单外条目**——处置 = 只 `git add` 显式路径，`src/**` 一律不暂存；`git status` 如实列出遗留项。提交前 `git diff --cached --name-only` 逐条核对暂存集。
- **目录交付入清单**：`git add docs/reports` 前先 `Get-ChildItem -Recurse docs/reports` 展开、逐项核对归属与意外文件；提交前 `git diff --cached --name-only` 展开确认（本轮 = 1 报告 + 9 夹具）。
- **gh 认证现状**：本机 gh 配置目录 `%AppData%\GitHub CLI` 与 Windows keyring 均无 token（设备码登录未持久化）。恢复办法：`git credential fill`（token 不落盘打印）→ `gh auth login --with-token`。
- **账号身份**：本机凭据 login=`NeitherTourRest`、display name=`MakeChase`、id=180032968、scope 含 `repo`；另一个用户 `MakeChase`（id=128372141）是**不同账号**。**GitHub URL 用 login，不用 display name**。
- **代理纪律**：`gh` 只认 `HTTP_PROXY`/`HTTPS_PROXY` 环境变量，**不读** git 的 `http.proxy`；`git push` 走 git 自身 `http.proxy`（本仓库已设 `http.proxy`/`https.proxy=http://127.0.0.1:7890`）。
- **PowerShell 假错误**：`git push` 把进度写 stderr，PS 会显示红色 `NativeCommandError`，判据是 `$LASTEXITCODE=0`。
- **CRLF 警告**：仓库含中文 Markdown / Rust 源，`git add` 会打印 `LF will be replaced by CRLF` 警告，属正常；提交用 `git -c core.autocrlf=false commit` 保持行尾稳定。
- **⚠️ 提交体量数字核对**：`core.autocrlf=false` 下 `git commit` **自身打印**的 insertions/deletions 可能因行尾转换而**虚高**——**判据一律以 `git show --stat HEAD` / `git show HEAD~1..HEAD --numstat` 为准**。**逐文件铁证**：`git hash-object <file>` 与 `git rev-parse HEAD:<file>` 相等 ⇒ 工作区/索引/提交三者一致。
- **多行提交信息 + 非 ASCII 字符**：body 含 `<`/`>`/`§` 时，**写 UTF-8 信息文件**（`C:\Users\19170\AppData\Local\Temp\opencode\lfz_msg*.txt`）→ `git -c core.autocrlf=false commit -F <file>`，提交后 `git log -1 --format=%B` 复核正文完整。
- **基线事实（更新后）**：上一轮 `feat(p3): parser v1 features …` = 短哈希 `682d1fb`；本轮提交在其之上；附注标签 `v0.1.0` 指向 `e060b21`；远程 `refs/heads/main` = 本轮 HEAD（本轮提交后）。
- **撤单核对流程**：先 `git status --short --untracked-files=all` 与任务书预期清单逐条比对；本轮**允许**含 `src/**` 的已知并行条目（不触发停止），但暂存集必须精确等于显式清单；暂存后 `git diff --cached --name-only` 确认。
- **禁区确认**：本轮未 force-push、未改默认分支、未打 tag、未 `git add -A`、未暂存 `src/**`/`docs/spec/**`、未提交 `target/`/密钥/临时文件、未改清单外文件（除本角色收工文档）。
