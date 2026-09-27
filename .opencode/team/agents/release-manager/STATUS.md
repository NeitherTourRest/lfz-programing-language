# release-manager — 工作状态
> 最后更新: 2026-09-27 by release-manager
## 当前状态
**✅ 完成 —— P6（性能）/ P7a（人类文档）/ P7b（AI 指南+skill）按产出归属切分为三个原子提交并推送（本轮未打 tag）。**
- 上轮 HEAD = 远程 `refs/heads/main` = `f794d6c`（`test(p5): finalize black-box suite (coverage matrix + report)`）。
- **入清单门禁（通过）**：`git status --short -uall` 复核实测与任务书预期逐字一致——` M` = `.opencode/team/DECISIONS.md` + `agents/{perf-engineer,docs-writer,ai-dx-engineer}/{STATUS,JOURNAL}.md`（7 项）；`??` = `benchmarks/`、`docs/guide/`、`docs/reports/performance.md`、`.opencode/skills/`。**无清单外条目、无缺失条目**，未触发停止条件。
- **提交切分（按产出归属，非按文件时间）**：
  1. `perf(p6): LFZ vs Python benchmark suite and report` = `958170c` —— `benchmarks/`（18 文件）+ `docs/reports/performance.md` + `perf-engineer/{STATUS,JOURNAL}.md` + **`.opencode/team/DECISIONS.md`（P6 性能 ADR，见下方「归属修正」）**。
  2. `docs(p7): human-facing guide (...)` = `60cbb96` —— `docs/guide/{README,tutorial,reference,errors,testing}.md` + `docs-writer/{STATUS,JOURNAL}.md`。
  3. `feat(p7): LFZ AI development guide and lfz-programming skill` = 见汇报 —— `.opencode/skills/`（3 文件）+ `docs/guide/ai/`（4 文件）+ `ai-dx-engineer/{STATUS,JOURNAL}.md` + 本角色收工文档。
- **归属修正（重要，任务书已预判）**：任务书把 `DECISIONS.md` 列在提交 3，并注明「若含三方 ADR 归属提交 3；如含性能专属 ADR 请拆分说明」。`git diff .opencode/team/DECISIONS.md` 实测**只含 1 条 perf-engineer 的 P6 性能 ADR**（无三方 ADR），故按「产出归属」原则**改并入提交 1（P6）**。这是本轮唯一的清单重排，已如实上报。
- **提交信息偏差（已修正，如实上报）**：任务书 body 写「raw data in `benchmarks/raw.json`」，但实测原始数据实际位于 **`benchmarks/results/raw.json`**（与 perf ADR 一致）。为避免在 Git 历史中写入不存在的路径，提交 1 body 采用**实际路径 `benchmarks/results/raw.json`**，其余逐字照录。此为唯一一处对任务书原文的偏离。
## 校验基线（本轮取证）
- `git status --short -uall` 门禁复核 = 与任务书逐字一致（见上）。
- `git check-ignore -v target` → `.gitignore:25:/target/`（构建产物已忽略）；`git diff --cached --name-status` 每提交前逐项核对，无 `target/`·密钥·临时文件。
- 推送后验收：`git status --short` 空；`git log --oneline -5`；`git ls-remote origin refs/heads/main` = 本地 HEAD；复议 `cargo run --quiet -- test`（汇总行 + 退出码，见汇报）。
- 未打新 tag（`git tag -n` 仍 `v0.1.0`/`v0.2.0`/`v0.3-tested`）；P8 收口再打 `v0.4-app`。
## 进行中
- （无）
## 阻塞 / 需要支持
- **无阻塞**。
- （遗留）**owner 偏差**：可认证账号 login = `NeitherTourRest`（display name = `MakeChase`）。待 team-lead/用户确认；不影响本任务。
## 下一步计划
- P10 交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项，至少 3 轮）。
- 后续里程碑标签：P8 `v0.4-app` / P9-P10 `v1.0-final`。
- README 顶部「当前状态」表的 P3/P5/P6/P7 行宜由 team-lead 授权后由我更新（本轮未改 README）。
## 关键经验（写给未来的自己）
- **归属 > 文件清单字面**：任务书给的是「文件 → 提交」候选映射，但因 ADR 归属已预判分叉，务必先 `git diff` 看内容再决定并入哪个提交。
- **提交信息路径必须与实际一致**：body 里的路径要 `git status` 复核；宁可改路径并上报，也不把不存在的路径写进历史（证据文化）。
- **JOURNAL 是「置顶追加」**：本角色 `JOURNAL.md` 最新条目在**最上方**（表头注释之后），新条目插在旧条目**之前**。
- **提交信息用 `-F` UTF-8 文件**：body 含 `§`、`;`、`/`、`'`、`...`、`=>` 等特殊字符时，`git commit -F <文件>` 比命令行 `-m` 更稳；信息文件放仓库外的临时目录，避免污染工作区。
- CRLF 预警（`LF will be replaced by CRLF`）为 Windows 行尾提示，无害；提交信息用 UTF-8。
- 禁区确认：本轮未 force-push、**未打 tag**、未改 `src/**`·`docs/spec/**`、未提交 `target/`·密钥·临时文件；未改任何他角色交付内容（仅代为入库）。
