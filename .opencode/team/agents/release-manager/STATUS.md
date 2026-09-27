# release-manager — 工作状态
> 最后更新: 2026-09-27 11:09 by release-manager
## 当前状态
**team-lead 现场重跑的「现状测试报告」`docs/reports/status-check.md` → 本轮入库并推送：1 报告 + 本角色收工文档，同一提交**。
- 本轮 **1 个提交**：`docs(reports): current-state test sweep (377 tests green, 25/25 fixtures as expected)`。
- 入库内容 = team-lead 撰写（**仅代为入库、未改内容**）的 `docs/reports/status-check.md`（新文件）+ 本角色收工 `{STATUS,JOURNAL}.md`。
- 报告结论：**P3 无回归、可继续开发**（build 0 warning；`cargo test` 377 全绿；端到端 hello；4 类错误模型；25 夹具 16 正例 exit 0 + 9 负例错误类/消息正确；Span 行号核验）。
- **未 force-push、未打任何新 tag**（这不是里程碑；`v0.1.0`/`v0.2.0` 保留）；`git add` 全为**显式清单**，未 `git add -A`，**未暂存 `src/**`**。
## 校验基线（本轮证据）
- 入清单门禁：`git status --short` 恰 **1 项**（`?? docs/reports/status-check.md`），与任务书逐字一致；**预期的 `M src/cli.rs`/`M src/main.rs`（P4 开工）未出现**；**无其它意外条目**（门禁未触发）。
- 内容取证：`docs/reports/status-check.md` = **4398 B / 50 行**；6 个标题（现状测试报告 / 1. 构建与单测 / 2. 四类错误模型（真实 CLI）/ 3. 夹具全量压测（3 组 / 25 个 `.lfz`）（3.1 正例 16 全 exit 0、3.2 负例 9 全 exit 2 且错误正确）/ 4. 位置信息专项（`Span` 正确性）/ 5. 结论）。
- 基线：提交前 HEAD = 远程 `refs/heads/main` = `f625150`；工作区仅 `?? docs/reports/status-check.md`（无密钥/临时文件；`target/` 已在 `.gitignore`）。
- 终态：`git status --short` 空（或如实列出并行改动 `M src/cli.rs`/`M src/main.rs`）；本地 HEAD = `git ls-remote origin refs/heads/main`。
## 进行中
- （无）
## 阻塞 / 需要支持
- **owner 偏差（历史遗留，待确认）**：可认证账号 login = `NeitherTourRest`（display name = `MakeChase`）。待 team-lead/用户确认；不影响交付。
## 下一步计划
- 用户验收通过后：P10 交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项，至少 3 轮）。
- 后续阶段里程碑（P5 `v0.3-tested` / P8 `v0.4-app` / P9/P10 `v1.0-final`）按需打附注标签（须 team-lead 授权）。
## 关键经验（写给未来的自己）
- **报告类大文档入库 = 单文件原子提交**：任务书给「预期工作区 = 仅此 1 项」时，`git add <显式文件>` 即可，零歧义；门禁（清单外条目即停止）比 `git add -A` 更省心。
- **并行改动要识别、不卷入**：任务书预告「可能并行出现 `M src/cli.rs`/`M src/main.rs`（P4 开工）」，这类条目**不暂存、不中止**，仅在取证 `git status --short` 中如实列出；只有**未预告**的条目才触发停止门禁。
- **内容取证要"声明 vs 实测"对齐**：用 `(Get-Item).Length` + `(Get-Content).Count` + `Select-String '^#{1,3} '` 章节表，一次性证明「文件存在、体量吻合、结构完整」。
- **收工文档并入同一提交**：先改 `STATUS.md`/`JOURNAL.md` 再 `git add`，工作区天然干净、无需二次提交。
- **Journal 只追加、最新条目在最上方**：新条目插在 `# 标题` 说明行之后、上一条 `## [时间]` 之前。
- **代理纪律**：`git push` 走仓库自身 `http.proxy`/`https.proxy=http://127.0.0.1:7890`；PowerShell 下进度写 stderr 显示红色 `NativeCommandError`，判据是 `$LASTEXITCODE=0`。
- **禁区确认**：本轮未 force-push、未打新 tag、未 `git add -A`、未暂存 `src/**` 及清单外文件、未提交 `target/`·密钥·临时文件；未改动 `docs/spec/**`。
