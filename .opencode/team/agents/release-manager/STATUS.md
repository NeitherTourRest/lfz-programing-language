# release-manager — 工作状态
> 最后更新: 2026-09-24 12:37 by release-manager
## 当前状态
**team-lead 交付「错峰自动化（官方向正 + 无人值守 runner + 自启安装器）」→ 本轮入库并推送：4 脚本 + 1 插件 + 1 ADR 更正，收工文档并入同一提交**。
- 本轮 **1 个提交**：`chore(team): off-peak gate per official DeepSeek pricing + unattended runner + autostart`。
- 入库内容 = team-lead 撰写（**仅代为入库、未改内容**）的 6 个文件：`scripts/offpeak.ps1`（重写）、`.opencode/plugin/offpeak.ts`（重写）、`scripts/offpeak-runner.ps1`（新）、`scripts/offpeak-start.cmd`（新）、`scripts/offpeak-task.cmd`（新）、`.opencode/team/DECISIONS.md`（+ADR 官方向正）+ 本角色收工 `{STATUS,JOURNAL}.md`。
- **未 force-push、未打任何新 tag**（`v0.1.0`/`v0.2.0` 保留）；`git add` 全为**显式清单**，未 `git add -A`。
## 校验基线（本轮证据）
- 入清单门禁：`git status --short` 恰 **6 项**（`M scripts/offpeak.ps1`、`M .opencode/plugin/offpeak.ts`、`M .opencode/team/DECISIONS.md`、`?? scripts/offpeak-runner.ps1`、`?? scripts/offpeak-start.cmd`、`?? scripts/offpeak-task.cmd`），与任务书逐字一致，**无清单外条目**。
- 内容取证：4 脚本**纯 ASCII**（`gt127=0`：offpeak.ps1 3331B / runner 2791B / start 429B / task 2735B）；`git diff --stat` = 3 files changed, 142 insertions(+), 72 deletions(-)（另有 3 个新文件未入 diff）。
- 关检取证（复议）：`powershell -NoProfile -File scripts/offpeak.ps1` → `now UTC 2026-09-24 04:37 (Thursday)` / `peak = Mon-Fri 01:00-04:00,06:00-10:00 UTC` / `status: OFF-PEAK (may work, 50% price) - peak starts in 1h 23m` / **`$LASTEXITCODE = 0`**；`cmd /c "scripts\offpeak-task.cmd status"` → `auto-start : ENABLED` + `scheduled : none`。
- 提交前基线 HEAD = 远程 `refs/heads/main` = `5a6fbe5`。
- 终态：`git status --short` 空；本地 HEAD = `git ls-remote origin refs/heads/main`。
## 进行中
- （无）
## 阻塞 / 需要支持
- **owner 偏差（历史遗留，待确认）**：可认证账号 login = `NeitherTourRest`（display name = `MakeChase`）。待 team-lead/用户确认；不影响交付。
## 下一步计划
- 用户验收通过后：P10 交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项，至少 3 轮）。
- 后续阶段里程碑（P5 `v0.3-tested` / P8 `v0.4-app` / P9/P10 `v1.0-final`）按需打附注标签（须 team-lead 授权）。
## 关键经验（写给未来的自己）
- **闸门机制要"强制"而非"靠记得"**：`offpeak.ts` 只拦最耗 token 的「派发子智能体」(`task`/`call_omo_agent`)，本地读写/构建/测试/提交不受影响——节流应与产能解耦。
- **官方向正**：DeepSeek 官方口径 = 高峰 **周一至周五 01:00–04:00 与 06:00–10:00 UTC**（北京 09:00–12:00 & 14:00–18:00），其余（含北京 12:00–14:00 午休、夜间、周末、中国法定节假日）全低谷（5 折）；`LFZ_PEAK_UTC`/`LFZ_PEAK_DAYS`/`LFZ_HOLIDAYS` 可覆盖——上一条 ADR 的 00:30–08:30 默认值**已作废**。
- **Windows 下 BOM-less `.ps1` 会被 PowerShell 5.1 按 ANSI 解码**：纯 ASCII 是硬约束（本轮实测四脚本 `gt127 = 0`）；退出码语义 `0`=低谷 / `3`=高峰 稳定可脚本化。
- **插件零依赖**：`offpeak.ts` 不 import 任何包，避免解析不到 `@opencode-ai/plugin` 类型导致插件加载失败。
- **自启免管理员**：`offpeak-task.cmd autostart` 走用户 Startup 文件夹（`%APPDATA%\...\Startup`）复制 `.cmd` 启动器，无需提权；`install` 走计划任务需管理员。
- **代理纪律**：`git push` 走仓库自身 `http.proxy`/`https.proxy=http://127.0.0.1:7890`；PowerShell 下进度写 stderr 显示红色 `NativeCommandError`，判据是 `$LASTEXITCODE=0`。
- **禁区确认**：本轮未 force-push、未打新 tag、未 `git add -A`、未暂存清单外文件、未提交 `target/`·密钥·临时文件；未改动 `src/**`、`docs/spec/**`。
