# release-manager — 工作状态
> 最后更新: 2026-09-23 by release-manager
## 当前状态
**P3.0（Cargo 骨架）已入库并推送至 origin**：两个原子提交（代码 + 团队文档）已落地 `main` 并 push，工作区干净、本地 HEAD = `refs/heads/main`。
- **远程仓库 URL：https://github.com/NeitherTourRest/lfz-programing-language**（Public，默认分支 `main`）。
- 提交 1（代码）：`chore(p3): cargo skeleton` —— `Cargo.toml`、`Cargo.lock`、`src/`（12 文件）；短哈希见 JOURNAL/汇报。
- 提交 2（团队文档）：`docs(team): add P3 phase plan and sync core-dev status` —— `PLAN-P3.md` + `core-dev/{STATUS,JOURNAL}.md` + 本角色收工文档。
- ⚠️ **`target/` 未入库**：`.gitignore:25 /target/` 生效，`git ls-files | Select-String '^target/'` = 0。
- **本轮未打任何 tag**（P3 里程碑标签 `v0.2.0` 待 P3.11 收口时打）；tag 仍仅 `v0.1.0`。
- ⚠️ **owner 偏差（历史遗留，需确认）**：任务书曾写 `github.com/MakeChase`，但可认证账号 login = `NeitherTourRest`（display name = `MakeChase`，id 180032968）；GitHub 上另一同名账号 `MakeChase`（id 128372141）无权操作。仓库实际落在 `NeitherTourRest` 名下。
## 进行中
- （无）
## 阻塞 / 需要支持
- **待 team-lead/用户确认 owner 偏差**：若期望仓库落在字面账号 `MakeChase`（id 128372141）名下，需先在该账号完成认证，再 transfer/重建仓库并同步 README——待授权后执行。当前交付在 `NeitherTourRest` 名下、功能完整可访问。
## 下一步计划
- P3 后续子阶段（P3.1+）：各角色交付文件由 team-lead 授权后，我按模块切分原子提交 + `git push`。
- P3.11 收口：核对通过条件后打附注标签 `v0.2.0`（`git tag -a -m`）。
- 每阶段里程碑：原子提交 + 附注标签 + 实时更新 README + `git push`（含标签）。
- P10：交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项）。
## 关键经验（写给未来的自己）
- **gh 认证现状**：本机 gh 配置目录 `%AppData%\GitHub CLI` 与 Windows keyring 均无 token（设备码登录未持久化）。恢复办法：`git credential fill`（token 不落盘打印）→ `gh auth login --with-token`。
- **账号身份**：本机凭据 login=`NeitherTourRest`、display name=`MakeChase`、id=180032968、scope 含 `repo`；另一个用户 `MakeChase`（id=128372141）是**不同账号**。**GitHub URL 用 login，不用 display name**。
- **代理纪律**：`gh` 只认 `HTTP_PROXY`/`HTTPS_PROXY` 环境变量（`$env:HTTPS_PROXY="http://127.0.0.1:7890"`），**不读** git 的 `http.proxy`；`git push` 走 git 自身 `http.proxy`（本仓库已设 `http.proxy`/`https.proxy=http://127.0.0.1:7890`）。
- **PowerShell 假错误**：`git push` 把进度写 stderr，PS 会显示红色 `NativeCommandError`，判据是 `$LASTEXITCODE=0`。
- **CRLF 警告**：仓库含中文 Markdown / Rust 源，`git add` 会打印 `LF will be replaced by CRLF` 警告，属正常；提交用 `git -c core.autocrlf=false commit` 保持行尾稳定。
- **多行提交信息**：PowerShell 下用 `-m "标题" -m "正文"`，正文含 `§` 等非 ASCII 时 `-m` 直传可用（本次已验证）；若遇编码问题改 `git commit -F <UTF-8 文件>`。
- **基线事实（更新后）**：提交链 `e060b21`(init) → `6873fe7`(ADR) → `35e5f62`(README url) → `caa66b4`(team docs sync) → 本轮 `chore(p3)` + `docs(team)` 两条；附注标签 `v0.1.0` 指向 `e060b21`；远程 `refs/heads/main`=本轮 HEAD。
- **撤单核对流程**：先 `git status --short` 与任务书预期清单逐条比对，出现清单外文件即停止汇报；暂存后用 `git status --short --untracked-files=all` 确认暂存集精确。
- **禁区确认**：本轮未 force-push、未改默认分支、未打 tag、未提交 `target/`/密钥/临时文件、未改清单外文件。
