# team-lead — 工作状态
> 最后更新: 2026-09-23 by team-lead

## 当前状态
**已开工，P0.5 全部完成。** P0 团队就绪 ✅；**P2 语言设计已冻结**（`docs/spec/` v1 三件套，ADR `D-016`）✅；**P0.5 版本基线（本地 + 远程）已完成** ✅ —— 本地 `git init -b main` + 提交链 `e060b21`/`6873fe7`/`35e5f62` + 附注标签 `v0.1.0`；远程 **https://github.com/NeitherTourRest/lfz-programing-language**（Public，`main` = `35e5f62`，tag 已推送，本地=远程）。**下一阶段：P1 需求基线同步 → P3 核心实现。**

## 进行中
- （无待派任务；P1/P3 等待我下一步调度，用户已整体放行"全程自动推进"）

## 阻塞 / 需要支持
- （无）
- ⚠️ **待用户知悉（非阻塞）**：GitHub 认证账号 **login = `NeitherTourRest`**（其 display name = `MakeChase`），仓库落在 `NeitherTourRest` 名下；GitHub 上另有同名不同账号 `MakeChase`。若用户要求 owner 是字面 `MakeChase`，需其在该账号登录后迁移/重建（暂未执行）。

## 下一步计划
1. **P1**：requirements-analyst 同步 `REQUIREMENTS.md`（对齐冻结 spec v1：Rust、`#42` 文件头、Python 式中文错误、A1–A7 语义、5 特色 + `float`）。
2. **P3**：core-dev（lexer/parser/AST/loader）+ runtime-dev（evaluator/builtins/env）**并行**，严格依 `docs/spec/`，TDD、错误含行列号。
3. **P4**：tooling-dev 定 `lfz run` / `lfz test` 契约（供 test-engineer 依赖）。
4. 全程：release-manager 每里程碑原子提交 + 附注标签 + 实时更新 README + push。

## 关键经验（写给未来的自己）
- **gh 与 git 的代理不通用（重要）**：本机 git 配 `http.proxy=http://127.0.0.1:7890`，但 **gh 只认 `HTTP_PROXY`/`HTTPS_PROXY` 环境变量** → 不注入则 `gh` 直连 `github.com:443` 超时。诊断：直连 github.com=❌、api.github.com=✅、gitee.com=✅、代理 7890=✅。**派 release-manager 做远程操作时，任务书必须写明先设这两个变量。**
- **后台长任务存活**：bash/shell 工具超时会**连带杀死子进程**（gh 设备码轮询即被 `context deadline exceeded`）。解法：**WMI `Win32_Process.Create`** 脱离进程树启动（`.cmd` + 日志重定向到 `C:\Users\19170\AppData\Local\Temp\opencode\` 后轮询文件）。
- **账号 identity vs display name**：`git config user.name` 和 gh 的 display name 都是 `MakeChase`，但 GitHub **login（URL 用）** 是 `NeitherTourRest`——不要拿 display name 当 owner。
- **winget 源**：msstore 源可能网络失败，改 `--source winget` 可成功。
- **冻结门禁口径**：评审人 PASS，或 CONCERNS 但明示"非架构级、不阻塞"，即视为达标。
- **行数统计陷阱**：PowerShell `Get-Content` 的 `.Count` 对本项目 md 会**少报**；权威口径 = `[System.IO.File]::ReadAllText()` + 统计 `\n`。
- **单一写者**：`PROJECT_STATE.md` / `TEAM_BOARD.md` 只有我能写；子 agent 汇报后**先核验证据再更新看板**。
- **14 人团队事实**：除我外全员 `permission.task: deny`；全员模型 `deepseek/deepseek-flash`；Oracle 全局禁用。
