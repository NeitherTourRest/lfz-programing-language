# release-manager — 工作状态
> 最后更新: 2026-09-27 by release-manager
## 当前状态
**✅ 完成 —— P5.3（黑盒测试集：§10.7 内置 54 个全覆盖）单个原子提交已执行并推送。**
- 上轮 HEAD = 远程 `refs/heads/main` = `52909b4`（`test(p5): black-box suite — core language features`）。
- **入清单门禁（通过）**：`git status --short -uall` 实测 **35 项**（4 个 ` M` + 31 个 `??`），与任务书预期逐字一致：` M` = `tests/cases.json`、`tests/coverage-matrix.md`、`.opencode/team/agents/test-engineer/{STATUS,JOURNAL}.md`；`??` = `tests/lfz/`（8 个新 `.lfz`）+ `tests/fixtures/`（23 个新夹具）。**无清单外条目、无缺失条目**。
- **提交（P5.3）**= `git add tests/cases.json tests/coverage-matrix.md tests/lfz tests/fixtures .opencode/team/agents/test-engineer/{STATUS,JOURNAL}.md` + 本角色收工文档（`release-manager/{STATUS,JOURNAL}.md`）→ `test(p5): black-box suite — all 54 builtins covered`。
- 先完成收工协议（覆盖本 `STATUS.md`、置顶追加本 `JOURNAL.md`）再提交，使收工文档并入该提交，工作区提交后保持干净。
## 校验基线（本轮取证）
- 提交前 `git status --short -uall` = 35 项；`git check-ignore -v target` → `.gitignore:25:/target/`（构建产物已忽略）；`git tag -n` 仍仅 `v0.1.0` / `v0.2.0`（**本轮未打任何 tag**）。
- 复议 `cargo run --quiet -- test`（提交前 / push 后）→ `汇总：共 82 个用例，通过 82，失败 0，错误 0`，退出码 0。
## 进行中
- （无）
## 阻塞 / 需要支持
- **无阻塞**。
- （遗留）**owner 偏差**：可认证账号 login = `NeitherTourRest`（display name = `MakeChase`）。待 team-lead/用户确认；不影响本任务。
## 下一步计划
- P10 交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项，至少 3 轮）。
- 后续里程碑标签：P5 `v0.3-tested`（待 P5 全部批次完成由 team-lead 判定）/ P8 `v0.4-app` / P9-P10 `v1.0-final`。
- 与 tooling-dev 衔接打包；核对报告与打包产物一致性核对。
## 关键经验（写给未来的自己）
- **JOURNAL 是「置顶追加」**：本角色 `JOURNAL.md` 最新条目在**最上方**（第 3 行起、表头注释之后），新条目插在旧条目**之前**，不是追加到文件末尾——与多数日志相反，务必注意。
- **门禁在 `git add` 之前**：先 `git status --short -uall` + 如实报告，与任务书清单逐字比对，再动暂存区；`-uall` 展开未跟踪目录以确认无遗漏。
- **收工文档排序要点**：当收工文档须并入同一提交、且验收要求「push 后 `git status --short` 空」时，**必须先写 STATUS/JOURNAL 再 commit**，否则 push 后再改会造成工作区 dirty 与验收项矛盾。
- **提交信息用 `-F` UTF-8 文件**：body 含 `;;`、`_`、`...`、`=>`、`#42`、`±Inf`、`§` 等特殊字符时，`git commit -F <文件>` 比命令行 `-m` 更稳，避免 PowerShell 引号/通配解析干扰。
- CRLF 预警（`LF will be replaced by CRLF`）为 Windows 行尾提示，无害；提交信息用 UTF-8。
- 禁区确认：本轮未 force-push、未打 tag、未改 `src/**`·`docs/spec/**`、未提交 `target/`·密钥·临时文件；未改任何他角色交付内容（仅代为入库）。
