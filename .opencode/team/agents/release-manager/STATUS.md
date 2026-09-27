# release-manager — 工作状态
> 最后更新: 2026-09-27 by release-manager
## 当前状态
**✅ 完成 —— T11-10 发布收口：重建 `dist/lfz.exe`（710144 B）+ 刷新 README 陈旧计数 + `Cargo.toml` 1.0.1 + 打附注标签 `v1.2.1` + 5 个原子提交 + 推送（main + tag）。**
- 发布产物：`dist/lfz.exe` = **710144 B**，mtime **2026-09-27 22:37:49**，`--version` = `lfz 1.0.1`（由当前源码同源重建，与 verifier 的 `target/release/lfz.exe` 同字节数）；`dist/` 被 `.gitignore` 忽略，不入库。
- 冒烟（3 项全过）：`dist\lfz.exe run examples\hello.lfz` → `Hello, LFZ!`（exit 0）；`dist\lfz.exe test` → **90/90 exit 0**；`dist\lfz.exe run app\sortviz.lfz` → exit 0（5 算法全 `[校验通过]`）。
- README 刷新（唯一写者）：`cargo test` 445→**482**（lib 379 + main 48 + cli 28 + test_runner 12 + unit 15）、`lfz test` 85→**90**、P10「收尾中」→**已完成**、版本里程碑补 `v1.1.0`/`v1.2.0`/`v1.2.1`、证据索引补 T11 报告、`dist\lfz.exe` 体积按实测 **710144 B**。
- 版本：`Cargo.toml` `1.0.0 → 1.0.1`（+ `Cargo.lock` 同步）。
- 5 个原子提交（按逻辑分组）：① 审计证据；② 修复批次源码 + 单测；③ spec 补钉 + ADR；④ 团队记忆 + `.gitignore` + `examples/life.lfz`；⑤ 发布（README + Cargo.toml + Cargo.lock + delivery-checklist + 本角色收工文档）。
- 标签：`git tag -a v1.2.1 -m "..."`（附注，列明 6 条 ADR + `;;`×`--json` 通道修复 + 越界写 span 修复）。
- `examples/test.lfz`（103 B，盲测 agent 越界写入的散落探针）**已删除且不提交**；`examples/life.lfz`（3343 B，用户点单示例）已提交。
- 推送：`git push origin main` + `git push origin v1.2.1`。
## 校验基线（本轮取证）
- 提交前 HEAD = `34e445f`（`docs(adr): record lfz-programming skill package upgrade`）；分支 `main`；原 tags = 7（`v0.1.0`/`v0.2.0`/`v0.3-tested`/`v0.4-app`/`v1.0-final`/`v1.1.0`/`v1.2.0`）。
- 入清单门禁：`git status --short` 实测与任务书背景一致（30 ` M` + 15 `??`），无清单外意外条目。
## 进行中
- （无）
## 阻塞 / 需要支持
- **无阻塞**。
- 上报 team-lead（非阻塞）：`docs/reports/P9-verification.md`（verifier 文件）仍记 `bug-20260927-02` 为「已打开 / 待裁定」——按 T11 复验裁定应为 **closed（不成立 / 不可复现）**；我未改 verifier 文件，仅报告（我自己的 `docs/reports/delivery-checklist.md` 已标注 closed）。
## 下一步计划
- 交付完成。等待 team-lead 决定是否进入 T11-③ **v1.1 迭代**（`PROJECT_STATE.md`/`TEAM_BOARD.md`/README 的迭代状态由 team-lead 更新）。
## 关键经验（写给未来的自己）
- **重建发布物与源码同源证明（三重佐证）**：`cargo build --release` 首跑真编译 + `dist/lfz.exe --version` 打印版本 `lfz 1.0.1` + 字节数 710144（与 verifier 复验产物一致）。
- **版本号改动会带出 `Cargo.lock`**：改 `Cargo.toml` version 后 `cargo build` 自动改 `Cargo.lock`；二者应**同提交**（本轮并入发布提交 ⑤）。
- **5 组提交按逻辑切分**：审计证据 / 修复代码+测试 / spec+ADR / 团队记忆 / 发布；`examples/test.lfz` 越界探针**删除且不提交**。
- **非 ASCII 提交信息**：全程 `git commit -F <UTF-8 文件>`（信息含 `→`/`§`/`；`），无拆词。
- **stale 计数以实跑为准**：`cargo test` 482 / `lfz test` 90（verifier 实测）；不用任务书里的笔误 89。
- 禁区确认：未 force-push、未删既有标签、只打 `v1.2.1` 一个标签、未提交 `Temp/`·`dist/`·`target/`、未改 `src/**` 行为（仅 `Cargo.toml` version）、未改 `docs/spec/**`·`tests/**` 内容。
