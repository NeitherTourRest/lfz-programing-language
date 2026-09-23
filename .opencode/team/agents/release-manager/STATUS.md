# release-manager — 工作状态
> 最后更新: 2026-09-23 23:45 by release-manager
## 当前状态
**P3.10（最小 CLI + 端到端）：两个原子提交并推送至 origin**（工作区干净、本地 HEAD = `refs/heads/main`）。
- **远程仓库 URL：https://github.com/NeitherTourRest/lfz-programing-language**（Public，默认分支 `main`）。
- **本轮提交（2 个原子提交）**：
  1. `c7627a6` `feat(p3): minimal CLI (lfz run <file>) and hello example`（`src/main.rs` + `src/cli.rs` + `examples/hello.lfz` + `tests/cli.rs` + `agents/tooling-dev/{STATUS,JOURNAL}.md`，6 files changed, **639 insertions(+), 9 deletions(-)**；`git show --stat` 口径）。
  2. 本提交 `docs(team): record P3.10 CLI contract ADR`（`.opencode/team/DECISIONS.md` + 本角色 `{STATUS,JOURNAL}.md`，短哈希见汇报）。
- 前置核验（team-lead 提供）：`cargo run -- run examples/hello.lfz` → `Hello, LFZ!`、退出码 **0**；缺 `#42` → `CosmosAnswerError: 你忘记了宇宙的答案` + 退出码 **2**；`cargo build` **0 warning**；测试 277(库)+8+7(集成) 全绿。
- ⚠️ **入清单门禁**：动手前 `git status --short --untracked-files=all` 恰为任务书预期 **6 组（7 个文件）**（`M src/main.rs`、`?? src/cli.rs`、`?? examples/`、`?? tests/`、`M .opencode/team/DECISIONS.md`、`M agents/tooling-dev/{STATUS,JOURNAL}.md`）；**无清单外条目**，门禁未触发。
- ⚠️ **新增目录内容复核**：`Get-ChildItem -Recurse examples, tests` → `examples/hello.lfz`（唯一）、`tests/cli.rs`（唯一）；**`tests/` 内为 Rust 集成测试（`*.rs`），无 `*.lfz` 黑盒测试**（后者属 test-engineer，P5），符合任务书预期。
- ⚠️ **`src/parser.rs` 未改动**（P3.4b3 未落地），**未入本轮任何提交**。
- ⚠️ **暂存纪律**：全程 `git add <显式文件>`（未用 `git add -A`）；每次提交前 `git diff --cached --name-only` 逐条核对。
- **本轮未打任何 tag**（P3 里程碑标签 `v0.2.0` 待 P3.11 收口时打）；tag 仍仅 `v0.1.0`。
- **未 force-push、未改默认分支**。
- ⚠️ **owner 偏差（历史遗留，需确认）**：可认证账号 login = `NeitherTourRest`（display name = `MakeChase`，id 180032968）；GitHub 上另一同名账号 `MakeChase`（id 128372141）无权操作。仓库实际落在 `NeitherTourRest` 名下。
## 进行中
- （无）
## 阻塞 / 需要支持
- **待 team-lead/用户确认 owner 偏差**：若期望仓库落在字面账号 `MakeChase`（id 128372141）名下，需先在该账号完成认证，再 transfer/重建仓库并同步 README——待授权后执行。当前交付在 `NeitherTourRest` 名下、功能完整可访问。
## 下一步计划
- P3.11 收口：核对通过条件后打附注标签 `v0.2.0`（`git tag -a -m`）。
- 每阶段里程碑：原子提交 + 附注标签 + 实时更新 README + `git push`（含标签）。
- P10：交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项）。
## 关键经验（写给未来的自己）
- **gh 认证现状**：本机 gh 配置目录 `%AppData%\GitHub CLI` 与 Windows keyring 均无 token（设备码登录未持久化）。恢复办法：`git credential fill`（token 不落盘打印）→ `gh auth login --with-token`。
- **账号身份**：本机凭据 login=`NeitherTourRest`、display name=`MakeChase`、id=180032968、scope 含 `repo`；另一个用户 `MakeChase`（id=128372141）是**不同账号**。**GitHub URL 用 login，不用 display name**。
- **代理纪律**：`gh` 只认 `HTTP_PROXY`/`HTTPS_PROXY` 环境变量，**不读** git 的 `http.proxy`；`git push` 走 git 自身 `http.proxy`（本仓库已设 `http.proxy`/`https.proxy=http://127.0.0.1:7890`）。
- **PowerShell 假错误**：`git push` 把进度写 stderr，PS 会显示红色 `NativeCommandError`，判据是 `$LASTEXITCODE=0`。
- **CRLF 警告**：仓库含中文 Markdown / Rust 源，`git add` 会打印 `LF will be replaced by CRLF` 警告，属正常；提交用 `git -c core.autocrlf=false commit` 保持行尾稳定。
- **⚠️ 提交体量数字核对**：`core.autocrlf=false` 下 `git commit` **自身打印**的 insertions/deletions 可能因行尾转换而**虚高**——**判据一律以 `git show --stat HEAD` / `git show HEAD~1..HEAD --numstat` 为准**。**逐文件铁证**：`git hash-object <file>` 与 `git rev-parse HEAD:<file>` 相等 ⇒ 工作区/索引/提交三者一致。
- **多行提交信息 + 非 ASCII 字符**：body 含 `<`/`>`/`§` 时，**写 UTF-8 信息文件**（`C:\Users\19170\AppData\Local\Temp\opencode\lfz_msg*.txt`）→ `git -c core.autocrlf=false commit -F <file>`，提交后 `git log -1 --format=%B` 复核正文完整。
- **新增目录入清单门禁**：任务书给 `?? examples/`、`?? tests/` 这类**目录**条目时，**先 `Get-ChildItem -Recurse` 展开目录内容逐项复核**（确认归属正确、无意外文件），再 `git add <目录>`；提交前 `git diff --cached --name-only` 会展开为具体文件路径，逐条核对。
- **基线事实（更新后）**：提交链 …→ `81a4b8a`(parser postfix/literals) → `ae88d38`(P3 parser ops/assign) → `d51755a`(P3 HOF builtins) → `f26ab80`(P3.9b HOF ADR) → **`c7627a6`(P3.10 CLI) → 本轮第 2 提交 `docs(team): record P3.10 CLI contract ADR`（短哈希见汇报）**；附注标签 `v0.1.0` 指向 `e060b21`；远程 `refs/heads/main`=本轮 HEAD。
- **撤单核对流程**：先 `git status --short --untracked-files=all` 与任务书预期清单逐条比对，出现清单外文件即停止汇报；暂存后 `git diff --cached --name-only` 确认暂存集精确。
- **多提交任务**：一逻辑变更一提交；连续提交时每次单独 `git add` + 单独 `git diff --cached --name-only` 核对，再 `commit -F`；最后一个提交按 team-lead 要求并入本角色收工文档，保持工作区干净。
- **禁区确认**：本轮未 force-push、未改默认分支、未打 tag、未 `git add -A`、未提交 `src/parser.rs`/`target/`/密钥/临时文件、未改清单外文件。
