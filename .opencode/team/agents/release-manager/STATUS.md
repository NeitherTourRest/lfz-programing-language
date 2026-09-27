# release-manager — 工作状态
> 最后更新: 2026-09-27 by release-manager
## 当前状态
**✅ 完成 —— 最终发布收口：三个原子提交 + `v1.0-final` 附注标签 + 推送。**
- HEAD（收口前）= `6aabdf5`（`fix: bind self for methods retrieved via ["k"]`）；本轮三个提交：提交 1 = `7787da7`（`test(p5)`）、提交 2 = `34de77f`（`docs(p9)`）、提交 3 = 见汇报（`docs(release)`）。
- **入清单门禁（通过，未触发停止）**：`git status --short` 实测与任务书背景**逐字一致**（10 ` M` + 3 `??` = 13 项），无清单外条目、无缺失条目。
- **基线复跑（与任务书权威数字全部一致，无停止）**：`cargo build` 0 warning；`cargo test` **432**（362+42+16+12）；黑盒 **85/85** exit 0；`app/sortviz.lfz` **LF=341 行**。
- 版本：`Cargo.toml` 由 `0.1.0` → **`1.0.0`**（唯一授权代码侧改动）；`Cargo.lock` 随之更新为 `1.0.0`。
## 校验基线（本轮取证）
- `cargo build` → `Finished`，**0 warning / 0 error**（版本变更后 `cargo clean -p lfz` 重建复跑仍 0 warning）。
- `cargo test` → **432 passed / 0 failed / 0 ignored**（lib **362** + main 42 + cli 16 + test_runner 12）。
- `cargo run --quiet -- test` → **85 个用例 / 通过 85 / 失败 0 / 错误 0**，exit 0。
- `app/sortviz.lfz` = **341 行**（LF=341, CR=0；PS5.1 下 `Get-Content .Count`/`Measure-Object -Line` 对 UTF-8 误读为 327/302，已弃用）。
- 标签：`v0.1.0` / `v0.2.0` / `v0.3-tested` / `v0.4-app` / **`v1.0-final`**（本轮新增，附注标签）。
## 进行中
- （无）
## 阻塞 / 需要支持
- **无阻塞**。
- （遗留）**owner 偏差**：可认证账号 login = `NeitherTourRest`（display name = `MakeChase`）。待 team-lead/用户确认；不影响交付。
## 下一步计划
- 交付完成（`v1.0-final` 已打并推送）。如需，配合 team-lead 做最终 standup / 答辩。
## 关键经验（写给未来的自己）
- **PS5.1 原生参数会把提交信息里的双引号拆词**：`git commit -m '...(s["k"])...'` → `pathspec 'blind' did not match`。含 `"` 的提交信息一律走 `git commit -F <UTF-8 文件>`（本次提交 1 即如此）。
- **行数一律用 LF 计数，勿信 `Get-Content .Count`**：PS5.1 默认编码对 UTF-8 会少行（`app/sortviz.lfz` 实测 LF=341，被 `Get-Content` 误读为 327、`Measure-Object -Line` 为 302）；以 `[System.IO.File]::ReadAllBytes` 数 `\n`（或 Python 迭代）为准。
- **版本 bump 后必须复跑**：改 `Cargo.toml` 的 `version` 会使 `Cargo.lock` 自动更新（须一并入库），且必须重跑 `cargo build` + `cargo test` + `cargo run --quiet -- test` 确认全绿（本次三绿）。
- 禁区确认：本轮**只打 `v1.0-final` 一个标签**、无 force-push、未改 `docs/spec/**`·`src/**`（仅 `Cargo.toml` 的 `version`）、未提交 `target/`·密钥·临时文件。
- README 对齐中超出任务书枚举的**事实性修正**（P5 行 `25/56` → `26 正向 + 58 负例 + 1 豁免`；「版本里程碑」补 `v1.0-final` 行）已如实汇报；**未触碰 P9 叙述行**（非数字，留待 team-lead 裁定）。
