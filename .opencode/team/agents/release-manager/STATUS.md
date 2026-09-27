# release-manager — 工作状态
> 最后更新: 2026-09-27 by release-manager
## 当前状态
**✅ 完成 —— P8（LFZ 应用：排序算法可视化）单原子提交 + P8 里程碑附注标签 `v0.4-app` 已打并推送。**
- 上轮 HEAD = `30e8b6c`（`feat(p7): LFZ AI development guide and lfz-programming skill`）；本轮新 HEAD = 见汇报（`feat(p8): LFZ sorting-visualizer app (341 lines) with dev record`）。
- **入清单门禁（通过，未触发停止）**：`git status --short` 实测与任务书逐字一致——` M` = `.opencode/team/DECISIONS.md` + `.opencode/team/agents/app-dev/{STATUS,JOURNAL}.md`；`??` = `app/`（含 `sortviz.lfz`/`README.md`/`DEV_RECORD.md`）。**无清单外条目、无缺失条目**。
- **行数偏差（如实上报，本轮唯一一处对任务书背景的偏离）**：任务书背景写 `sortviz=327 / DEV_RECORD=218 / README=117`，与实测**全不符**；实测（Read 工具 `total` + `[System.IO.File]::ReadAllText` 换行计数，双口径一致）**`sortviz.lfz=341 / DEV_RECORD.md=299 / README.md=141`**，且 `app/README.md`、`app/DEV_RECORD.md`、app-dev 的 P8 ADR 三处**自述均为 341 行**。故提交标题采用**实测 341**（≥200 ✅ 满足），其余逐字照录；不把错误数字写进 Git 历史（证据文化 / 不虚构）。
- **提交（单原子，按产出归属）**：`feat(p8): LFZ sorting-visualizer app (341 lines) with dev record` = 见汇报 —— `app/`（3 文件）+ `.opencode/team/DECISIONS.md`（app-dev 的 P8 应用架构 ADR）+ `.opencode/team/agents/app-dev/{STATUS,JOURNAL}.md` + 本角色收工文档 2。
- **标签（附注）**：`git tag -a v0.4-app -m "..."` → `git push` → `git push origin v0.4-app`；未打其他标签。
## 校验基线（本轮取证）
- `git status --short` 门禁复核 = 与任务书逐字一致（见上）。
- **自跑复现**：`cargo run --quiet -- run app/sortviz.lfz` → 退出码 **0**、输出 **78 行**，含「自测 150 次排序，失败 0 次（check 非致命）」「== 完成：5 种算法全部通过正确性校验 ==」。
- `git check-ignore app` → 退出码 1（未被忽略，应予入库）；`git check-ignore -v target` → `.gitignore:25:/target/`（构建产物已忽略）。
- 推送后验收：`git status --short` 空；`git log --oneline -3`；`git tag -n` 含 `v0.4-app`；`git ls-remote --tags origin` 含 `refs/tags/v0.4-app`；`git ls-remote origin refs/heads/main` = 本地 HEAD。
## 进行中
- （无）
## 阻塞 / 需要支持
- **无阻塞**。
- （遗留）**owner 偏差**：可认证账号 login = `NeitherTourRest`（display name = `MakeChase`）。待 team-lead/用户确认；不影响本任务。
## 下一步计划
- P10 交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项，至少 3 轮）。
- 后续里程碑标签：P9–P10 `v1.0-final`。
- README 顶部「当前状态」表 P4–P8 行宜由 team-lead 授权后由我更新（本轮未改 README）。
## 关键经验（写给未来的自己）
- **任务书「已核验」的背景数字也可能滞后**：行数/路径务必实测复核；宁可实测 + 上报，也不把错误数字/不存在的路径写进 Git 历史。
- **PowerShell 测行数**：`Measure-Object -Line` 会漏空行（≠总行数，且受默认编码影响漂移）；权威口径 = `[System.IO.File]::ReadAllText` 的 `\n` 计数或 Read 工具 `(End of file - total N lines)`。
- **归属 > 文件清单字面**：先 `git diff` 看内容再决定并入哪个提交（本轮 DECISIONS 的 P8 ADR 随 P8 提交）。
- **提交信息用 `-F` UTF-8 文件**：body 含 `§`、`;`、`/`、`'`、`...`、`=>` 等特殊字符时更稳；信息文件放仓库外临时目录，避免污染工作区。
- **JOURNAL 是「置顶追加」**：新条目插在旧条目**之前**（表头注释之后）。
- 禁区确认：本轮未 force-push、只打 `v0.4-app` 一个标签、未改 `src/**`·`docs/spec/**`·README、未提交 `target/`·密钥·临时文件；未改任何他角色交付内容（仅代为入库）。
