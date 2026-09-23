# team-lead — 工作状态
> 最后更新: 2026-09-23 by team-lead

## 当前状态
P0 团队就绪（脚手架已构建并验证，**尚未纳入 git**）；**P2 语言设计已完成并冻结**——`docs/spec/{syntax,semantics,interface-contract}.md` v1 三件套定稿（ADR `D-016`）。冻结门禁通过：core-dev **PASS**（文法无回溯可实现）；runtime-dev **CONCERNS 3 项**（非架构级、明示不阻塞，冻结时已闭合）。**当前等待用户下达「开工」指令。**

## 进行中
- （无）

## 阻塞 / 需要支持
- （无）

## 下一步计划
- 用户下达开工指令后，按序调度：
  1. **P0.5**：release-manager 执行 `git init` + 初始提交（团队脚手架）——**必须最先做**，保证后续全部工作进入 git 历史；
  2. **P1**：requirements-analyst 做需求基线同步（验收标准对齐冻结的 spec v1：Rust、`#42` 文件头、Python 式中文错误、A1–A7 语义、5 特色 + float）；
  3. **P3**：core-dev + runtime-dev **并行**进入核心实现（严格依 `docs/spec/` 冻结契约，TDD、错误含行列号）；tooling-dev 视契约就绪度跟进 P4。

## 关键经验（写给未来的自己）
- **冻结门禁判定口径**：核心评审人（core-dev = 文法可解析性、runtime-dev = 语义可求值性）给 **PASS**，或给 **CONCERNS 但明示"非架构级、不阻塞实现"**，即视为达标；非架构级小项可在冻结/拆分时一并闭合，不必无限迭代。
- **行数统计陷阱**：PowerShell `Get-Content` 的 `.Count` 对本项目 md 文件会**少报**（实测 `syntax.md` 报 620、实为 910）。权威口径 = `[System.IO.File]::ReadAllText()` + 统计 `\n`；字节数（`Get-ChildItem … .Length`）可靠。**核验证据时以此为准。**
- **单一写者纪律**：`PROJECT_STATE.md` / `TEAM_BOARD.md` 只有我能写。每次子 agent 汇报后，必须**先独立核验证据、再更新看板**（不认口头声明）。
- **用户偏好**：重大决策先请示；阶段转换（尤其"开工"）须等用户明确指令，不擅自推进。
- **14 人团队事实**：除我之外全员 `permission.task: deny`，无法互相调度；协作需求一律经我派发。全员模型 = `deepseek/deepseek-flash`；Oracle 全局禁用。
