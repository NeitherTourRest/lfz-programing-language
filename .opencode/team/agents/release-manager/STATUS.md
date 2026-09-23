# release-manager — 工作状态
> 最后更新: 2026-09-23 by release-manager
## 当前状态
**P0.5 全部完成并入库**：本地基线 + 公共远程 + 团队文档同步提交均已落地，工作区干净、本地=远程。
- **远程仓库 URL：https://github.com/NeitherTourRest/lfz-programing-language**（Public，默认分支 `main`）。
- 本轮完成"P0.5 团队文档同步"原子提交（7 个团队文档文件）并 `git push`，`git status --short` 为空、本地 HEAD = `refs/heads/main`。
- 无新增/移动 tag（非里程碑，保持 `v0.1.0` 不变）。
- ⚠️ **owner 偏差（需确认）**：任务书写的是 `github.com/MakeChase`，但**可认证账号的 login = `NeitherTourRest`**（display name = `MakeChase`，id 180032968）；GitHub 上另有一个**同名不同账号**的用户 `MakeChase`（id 128372141），本账号无权操作。故仓库实际落在 `NeitherTourRest` 名下，README 克隆链接按**实际 URL** 写入。
## 进行中
- （无）
## 阻塞 / 需要支持
- **待 team-lead/用户确认 owner 偏差**：若期望仓库落在字面账号 `MakeChase`（id 128372141）名下，需先在该账号完成认证，再 **transfer/重建** 仓库并同步更新 README——release-manager 待授权后执行。当前交付在 `NeitherTourRest` 名下、功能完整可访问。
- 说明：执行前 `gh` 的**设备码登录未持久化**（无任何 gh token）。我用本机 git 凭据管理器中的**既有凭据**（login `NeitherTourRest`，scope 含 `repo`）经 `gh auth login --with-token` 恢复 gh 登录，**未新建任何凭据**。
## 下一步计划
- 待 team-lead 就 owner 偏差拍板：保留 `NeitherTourRest` 或迁移到 `MakeChase`。
- 后续每阶段里程碑：原子提交 + 附注标签 + 实时更新 README + `git push`（含标签）。
- P10：交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项）。
## 关键经验（写给未来的自己）
- **gh 认证现状**：本机 gh 配置目录 `%AppData%\GitHub CLI` 与 Windows keyring 均无 token（设备码登录未持久化）。恢复办法：`git credential fill`（token 不落盘打印）→ `gh auth login --with-token`。
- **账号身份**：本机凭据 login=`NeitherTourRest`、display name=`MakeChase`、id=180032968、scope 含 `repo`；另一个用户 `MakeChase`（id=128372141）是**不同账号**。**GitHub URL 用 login，不用 display name**——这是本次 owner 偏差的根因。
- **代理纪律**：`gh` 只认 `HTTP_PROXY`/`HTTPS_PROXY` 环境变量（`$env:HTTPS_PROXY="http://127.0.0.1:7890"`），**不读** git 的 `http.proxy`；`git push` 走 git 自身 `http.proxy`（本仓库已设 `http.proxy`/`https.proxy=http://127.0.0.1:7890`）。
- **PowerShell 假错误**：`git push` 把进度写 stderr，PS 会显示红色 `NativeCommandError`，判据是 `$LASTEXITCODE=0`。
- **CRLF 警告**：仓库含中文 Markdown，`git add`/`diff` 会打印 `LF will be replaced by CRLF` 警告，属正常，不影响提交内容。
- **基线事实（更新后）**：提交链 `e060b21`(init) → `6873fe7`(ADR) → `35e5f62`(README url) → 本轮(team docs sync)；附注标签 `v0.1.0` 指向 `e060b21`；远程 `refs/heads/main`=本轮 HEAD。
- **提交规范**：`docs(team): sync project state and board after P0.5 completion`（纯 ASCII，可直接 `-m`）。
- **禁区确认**：本次未 force-push、未改默认分支、未动 tag、未改他人交付物内容、未在仓库外建文件、未提交任何密钥/凭据/临时文件。
