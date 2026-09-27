# release-manager — 工作状态
> 最后更新: 2026-09-27 by release-manager
## 当前状态
**✅ 完成 —— P5.4（黑盒测试集定稿）单个原子提交 + P5 里程碑附注标签 `v0.3-tested` 已打并推送。**
- 上轮 HEAD = 远程 `refs/heads/main` = `f40419c`（`test(p5): black-box suite — all 54 builtins covered`）。
- **入清单门禁（通过）**：`git status --short -uall` 实测 **5 项**（4 个 ` M` + 1 个 `??`），与任务书预期逐字一致：` M` = `tests/coverage-matrix.md`、`.opencode/team/DECISIONS.md`、`.opencode/team/agents/test-engineer/{STATUS,JOURNAL}.md`；`??` = `tests/REPORT.md`。**无清单外条目、无缺失条目**。
- **提交（P5.4）**= `git add tests/coverage-matrix.md tests/REPORT.md .opencode/team/DECISIONS.md .opencode/team/agents/test-engineer/{STATUS,JOURNAL}.md` + 本角色收工文档（`release-manager/{STATUS,JOURNAL}.md`）→ `test(p5): finalize black-box suite (coverage matrix + report)`。
- **里程碑标签**= `git tag -a v0.3-tested -m "v0.3-tested — P5 black-box test suite complete: ..."`（附注标签，非轻量），已在 push 提交后单独 `git push origin v0.3-tested`。
- 先完成收工协议（覆盖本 `STATUS.md`、置顶追加本 `JOURNAL.md`）再提交，使收工文档并入同一提交，工作区提交后保持干净。
## 校验基线（本轮取证）
- 提交前 `git status --short -uall` = 5 项；`git check-ignore -v target` → `.gitignore:25:/target/`（构建产物已忽略）。
- 复议 `cargo run --quiet -- test`（提交前）→ `汇总：共 82 个用例，通过 82，失败 0，错误 0`，退出码 0（P5 通过条件：一个命令跑完全部且全绿）。
- 打 tag 前 `git tag -n` 仅 `v0.1.0`/`v0.2.0`；打 tag 后应含 `v0.1.0`/`v0.2.0`/**`v0.3-tested`**（验收要求）。
## 进行中
- （无）
## 阻塞 / 需要支持
- **无阻塞**。
- （遗留）**owner 偏差**：可认证账号 login = `NeitherTourRest`（display name = `MakeChase`）。待 team-lead/用户确认；不影响本任务。
## 下一步计划
- P10 交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项，至少 3 轮）。
- 后续里程碑标签：P8 `v0.4-app` / P9-P10 `v1.0-final`。
- 与 tooling-dev 衔接打包；核对报告与打包产物一致性核对。
- README 顶部「当前状态」表的 P5 行宜由 team-lead 授权后由我更新（本提交未改 README）。
## 关键经验（写给未来的自己）
- **JOURNAL 是「置顶追加」**：本角色 `JOURNAL.md` 最新条目在**最上方**（表头注释之后），新条目插在旧条目**之前**。
- **门禁在 `git add` 之前**：先 `git status --short -uall` + 如实报告，与任务书清单逐字比对，再动暂存区。
- **里程碑 tag 流程**：commit → push → `git tag -a` → `git push origin <tag>`；附注标签必须用 `-a -m`；打 tag 前确认工作区干净且 HEAD = 远程。
- **提交信息用 `-F` UTF-8 文件**：body 含 `;;`、`_`、`...`、`=>`、`/`、`§` 等特殊字符时，`git commit -F <文件>` 比命令行 `-m` 更稳。
- CRLF 预警（`LF will be replaced by CRLF`）为 Windows 行尾提示，无害；提交信息用 UTF-8。
- 禁区确认：本轮未 force-push、只打 `v0.3-tested` 一个标签、未改 `src/**`·`docs/spec/**`、未提交 `target/`·密钥·临时文件；未改任何他角色交付内容（仅代为入库）。
