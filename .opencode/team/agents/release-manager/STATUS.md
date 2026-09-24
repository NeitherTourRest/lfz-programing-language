# release-manager — 工作状态
> 最后更新: 2026-09-24 12:35 by release-manager
## 当前状态
**team-lead 交付「错峰闸门机制」→ 本轮入库并推送：脚本 + opencode 插件 + ADR 一条，收工文档并入同一提交**。
- 本轮 **1 个提交**（入清单门禁通过）：`chore(team): DeepSeek off-peak work gate (script + opencode plugin)`。
  - 入库内容 = team-lead 撰写（**仅代为入库、未改内容**）的 `scripts/offpeak.ps1`、`.opencode/plugin/offpeak.ts`、`.opencode/team/DECISIONS.md`（+ADR）+ 本角色收工 `{STATUS,JOURNAL}.md`。
- **未 force-push、未打任何新 tag**（`v0.1.0`/`v0.2.0` 保留且远程一致）；`git add` 全为**显式清单**，未 `git add -A`。
## 校验基线（本轮证据）
- 入清单门禁：`git status --short --untracked-files=all` 恰 **3 项**（`M .opencode/team/DECISIONS.md`、`?? .opencode/plugin/offpeak.ts`、`?? scripts/offpeak.ps1`），与任务书逐字一致，**无清单外条目**。
- 内容取证：`scripts/offpeak.ps1` **纯 ASCII**（`bytes>127 = 0`，总 2139 字节）；`powershell -NoProfile -File scripts/offpeak.ps1` → `status: PEAK (stop)`、`$LASTEXITCODE = 3`（默认窗口 00:30–08:30，本机 UTC+8 12:30）；`.opencode/plugin/offpeak.ts` **无任何 import**（仅 `const`/`function`/`export`），`GATED_TOOLS = {task, call_omo_agent}`，仅拦派发、`LFZ_OFFPEAK_ENFORCE=0` 可关。
- 提交前基线 HEAD = 远程 `refs/heads/main` = `7aab824`。
- 终态：`git status --short` 空；本地 HEAD = `git ls-remote origin refs/heads/main`。
## 进行中
- （无）
## 阻塞 / 需要支持
- **owner 偏差（历史遗留，待确认）**：可认证账号 login = `NeitherTourRest`（display name = `MakeChase`）。待 team-lead/用户确认；不影响交付。
- **窗口待复核**：ADR 已注明默认低谷窗口 00:30–08:30（UTC+8）**待确认**（本机联网搜索配额用尽）；若 DeepSeek 调整，改 `LFZ_OFFPEAK_*` 环境变量即可，无需改代码。
## 下一步计划
- 用户验收通过后：P10 交付清单核对报告 `docs/reports/delivery-checklist.md`（对照 `task-info.md` 8 项，至少 3 轮）。
- 后续阶段里程碑（P5 `v0.3-tested` / P8 `v0.4-app` / P9/P10 `v1.0-final`）按需打附注标签（须 team-lead 授权）。
## 关键经验（写给未来的自己）
- **闸门机制要"强制"而非"靠记得"**：`offpeak.ts` 只拦最耗 token 的「派发子智能体」(`task`/`call_omo_agent`)，本地读写/构建/测试/提交不受影响——节流应与产能解耦，避免把本地动作也误伤。
- **Windows 下 BOM-less `.ps1` 会被 PowerShell 5.1 按 ANSI 解码**：纯 ASCII 是硬约束（本轮实测 `bytes>127 = 0`）；退出码语义 `0`=低谷 / `3`=高峰 稳定可脚本化。
- **插件零依赖**：`offpeak.ts` 不 import 任何包，避免项目内解析不到 `@opencode-ai/plugin` 类型导致插件加载失败（钩子签名在 ADR 中留档）。
- **时间戳以 `git log --format=%ci` 为准**；`git add` 一律**显式文件清单**，他人撰写文档**仅代为入库、不改内容**。
- **代理纪律**：`git push` 走仓库自身 `http.proxy`/`https.proxy=http://127.0.0.1:7890`；PowerShell 下进度写 stderr 显示红色 `NativeCommandError`，判据是 `$LASTEXITCODE=0`。
- **禁区确认**：本轮未 force-push、未打新 tag、未 `git add -A`、未暂存清单外文件、未提交 `target/`·密钥·临时文件；未改动 `src/**`、`docs/spec/**`。
