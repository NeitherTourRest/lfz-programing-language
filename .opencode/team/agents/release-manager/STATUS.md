# release-manager — 工作状态
> 最后更新: 2026-09-27 10:58 by release-manager
## 当前状态
**team-lead 交付「opencode→DSH 迁移手册」`MIGRATION-TO-DSH.md`（309 行 / 29675 B）→ 本轮入库并推送：1 手册 + 本角色收工文档，同一提交**。
- 本轮 **1 个提交**：`docs(migration): opencode -> DeepSeek Harness migration manual`。
- 入库内容 = team-lead 撰写（**仅代为入库、未改内容**）的 `MIGRATION-TO-DSH.md`（新文件）+ 本角色收工 `{STATUS,JOURNAL}.md`。
- **未 force-push、未打任何新 tag**（这不是里程碑；`v0.1.0`/`v0.2.0` 保留）；`git add` 全为**显式清单**，未 `git add -A`。
## 校验基线（本轮证据）
- 入清单门禁：`git status --short` 恰 **1 项**（`?? MIGRATION-TO-DSH.md`），与任务书逐字一致，**无清单外条目**（门禁未触发）。
- 内容取证：`MIGRATION-TO-DSH.md` = **29675 B / 309 行**（与任务书声明一致）；11 个 `##` 章节（0. TL;DR / 1. 项目 / 2. 进度快照 / 3. 需求与评分 / 4. 14 人团队 / 5. 文件地图 / 6. 能力映射 / 7. 迁移步骤 / 8. 迁移后自检 / 9. 已知坑与教训 / 10. 待确认 / 11. 打印版清单）；含 `cordis.yml`（5 处）、可勾选步骤（13 处 `- [ ]`）。
- 基线：提交前 HEAD = 远程 `refs/heads/main` = `27d3d48`；工作区仅 `?? MIGRATION-TO-DSH.md`（无密钥/临时文件；`target/` 已在 `.gitignore`）。
- 终态：`git status --short` 空；本地 HEAD = `git ls-remote origin refs/heads/main`。
## 进行中
- （无）
## 阻塞 / 需要支持
- **owner 偏差（历史遗留，待确认）**：可认证账号 login = `NeitherTourRest`（display name = `MakeChase`）。待 team-lead/用户确认；不影响交付。
## 下一步计划
- 用户验收通过后：P10 交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项，至少 3 轮）。
- 后续阶段里程碑（P5 `v0.3-tested` / P8 `v0.4-app` / P9/P10 `v1.0-final`）按需打附注标签（须 team-lead 授权）。
## 关键经验（写给未来的自己）
- **手册类大文档入库 = 单文件原子提交**：任务书给「预期工作区 = 仅此 1 项」时，`git add <显式文件>` 即可，零歧义；门禁（清单外条目即停止）比 `git add -A` 更省心。
- **内容取证要"声明 vs 实测"对齐**：用 `(Get-Item).Length` + `(Get-Content).Count` + `Select-String '^#{1,3} '` 章节表，一次性证明「文件存在、体量吻合、结构完整」，比只报"已提交"更有说服力。
- **收工文档并入同一提交**：先改 `STATUS.md`/`JOURNAL.md` 再 `git add`，工作区天然干净、无需二次提交。
- **Journal 只追加、最新条目在最上方**：新条目插在 `# 标题` 说明行之后、上一条 `## [时间]` 之前（本文件顶部注释即约定）。
- **代理纪律**：`git push` 走仓库自身 `http.proxy`/`https.proxy=http://127.0.0.1:7890`；PowerShell 下进度写 stderr 显示红色 `NativeCommandError`，判据是 `$LASTEXITCODE=0`。
- **禁区确认**：本轮未 force-push、未打新 tag、未 `git add -A`、未暂存清单外文件、未提交 `target/`·密钥·临时文件；未改动 `src/**`、`docs/spec/**`。
