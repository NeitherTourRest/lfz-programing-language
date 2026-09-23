# release-manager — 工作状态
> 最后更新: 2026-09-23 22:06 by release-manager
## 当前状态
**P3.2（loader）+ P3.6（value/env）已入库并推送至 origin**：三个原子提交已落地 `main` 并 push，工作区干净、本地 HEAD = `refs/heads/main`。
- **远程仓库 URL：https://github.com/NeitherTourRest/lfz-programing-language**（Public，默认分支 `main`）。
- 提交 1（P3.2 代码 + core-dev 文档）：`feat(p3): loader` —— `src/loader.rs` + `core-dev/{STATUS,JOURNAL}.md`（3 files changed, 426 insertions, 25 deletions）；短哈希 `7d41e17`。
- 提交 2（P3.6 代码 + runtime-dev 文档）：`feat(p3): value and env` —— `src/value.rs` + `src/env.rs` + `runtime-dev/{STATUS,JOURNAL}.md`（4 files changed, 1163 insertions, 8 deletions）；短哈希 `49d485f`。
- 提交 3（团队文档）：`docs(team): record P3.6 value/env cross-module interface ADR` —— `.opencode/team/DECISIONS.md` + 本角色收工文档（短哈希见 JOURNAL/汇报）。
- ✅ **P3.2 / P3.6 均已由 team-lead 独立核验**：`cargo build` 0 warning；`cargo test` 47 passed / 0 failed；`src/loader.rs` 15873 B、`src/value.rs` 22760 B、`src/env.rs` 17621 B（提交前实测一致）。
- ⚠️ **暂存纪律**：全部用 `git add <显式文件>`，未用 `git add -A`；提交前 `git diff --cached --name-only` 逐条核对暂存集精确为清单内文件。
- ⚠️ **`target/` 未入库**：`.gitignore:25 /target/` 生效；`--untracked-files=all` 无未跟踪文件。
- **本轮未打任何 tag**（P3 里程碑标签 `v0.2.0` 待 P3.11 收口时打）；tag 仍仅 `v0.1.0`。
- ⚠️ **owner 偏差（历史遗留，需确认）**：任务书曾写 `github.com/MakeChase`，但可认证账号 login = `NeitherTourRest`（display name = `MakeChase`，id 180032968）；GitHub 上另一同名账号 `MakeChase`（id 128372141）无权操作。仓库实际落在 `NeitherTourRest` 名下。
## 进行中
- （无）
## 阻塞 / 需要支持
- **待 team-lead/用户确认 owner 偏差**：若期望仓库落在字面账号 `MakeChase`（id 128372141）名下，需先在该账号完成认证，再 transfer/重建仓库并同步 README——待授权后执行。当前交付在 `NeitherTourRest` 名下、功能完整可访问。
## 下一步计划
- P3 后续子阶段（P3.3+）：各角色交付文件由 team-lead 授权后，我按模块切分原子提交 + `git push`。
- P3.11 收口：核对通过条件后打附注标签 `v0.2.0`（`git tag -a -m`）。
- 每阶段里程碑：原子提交 + 附注标签 + 实时更新 README + `git push`（含标签）。
- P10：交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项）。
## 关键经验（写给未来的自己）
- **gh 认证现状**：本机 gh 配置目录 `%AppData%\GitHub CLI` 与 Windows keyring 均无 token（设备码登录未持久化）。恢复办法：`git credential fill`（token 不落盘打印）→ `gh auth login --with-token`。
- **账号身份**：本机凭据 login=`NeitherTourRest`、display name=`MakeChase`、id=180032968、scope 含 `repo`；另一个用户 `MakeChase`（id=128372141）是**不同账号**。**GitHub URL 用 login，不用 display name**。
- **代理纪律**：`gh` 只认 `HTTP_PROXY`/`HTTPS_PROXY` 环境变量（`$env:HTTPS_PROXY="http://127.0.0.1:7890"`），**不读** git 的 `http.proxy`；`git push` 走 git 自身 `http.proxy`（本仓库已设 `http.proxy`/`https.proxy=http://127.0.0.1:7890`）。
- **PowerShell 假错误**：`git push` 把进度写 stderr，PS 会显示红色 `NativeCommandError`，判据是 `$LASTEXITCODE=0`。
- **CRLF 警告**：仓库含中文 Markdown / Rust 源，`git add` 会打印 `LF will be replaced by CRLF` 警告，属正常；提交用 `git -c core.autocrlf=false commit` 保持行尾稳定。
- **多行提交信息 + 非 ASCII 字符**：body 含 `<`/`>`（如 `Rc<RefCell>`）或 `§` 时，**写 UTF-8 信息文件**（`C:\Users\19170\AppData\Local\Temp\opencode\lfz_msgN.txt`）→ `git -c core.autocrlf=false commit -F <file>`，提交后 `git log -1 --format=%B` 复核正文完整。已验证 `Rc<RefCell>`、`§10.5` 均无损还原。
- **基线事实（更新后）**：提交链 `e060b21`(init) → `6873fe7`(ADR) → `35e5f62`(README url) → `caa66b4`(team docs sync) → `69a57d7`(cargo skeleton) → `4035f86`(p3 span/error) → `4458e75`(docs core-dev P3.1) → 本轮 `7d41e17`(loader) + `49d485f`(value/env) + `docs(team)`(ADR，短哈希见汇报)；附注标签 `v0.1.0` 指向 `e060b21`；远程 `refs/heads/main`=本轮 HEAD。
- **撤单核对流程**：先 `git status --short --untracked-files=all` 与任务书预期清单逐条比对，出现清单外文件即停止汇报；暂存后 `git diff --cached --name-only` 确认暂存集精确；提交前实测文件字节数与任务书声明的尺寸一致（本轮 15873/22760/17621 全部吻合）。
- **禁区确认**：本轮未 force-push、未改默认分支、未打 tag、未 `git add -A`、未提交 `target/`/密钥/临时文件、未改清单外文件。
