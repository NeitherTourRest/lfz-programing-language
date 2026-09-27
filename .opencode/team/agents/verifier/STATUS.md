# verifier — 工作状态
> 最后更新: 2026-09-27 by verifier
## 当前状态
**P9 复验（rev.2）已完成 → 结论 `CONCERNS（无阻塞）`。** 被验提交 = **git HEAD `6aabdf5`**（`fix: bind self for methods retrieved via ["k"]`；工作树 clean）。2 项待闭项：`bug-20260927-01` **已闭合**、`obs-02` **连带闭合**；`obs-01`（README）**未完全闭合**（残留 2 处数值）。
## 进行中
- （无）—— rev.2 复验结论已出；等 team-lead 决策与下一指派。
## 已交付（本次）
- `docs/reports/P9-verification.md` 追加「复验（rev.2）」§9.0–§9.5（基线 / bug-01 复验 / obs-01 复验 / 关键基线复核 / 额外检查 / 升级结论行）。
## 本轮关键实测证据（对 HEAD `6aabdf5`）
- ✅ `cargo clean; cargo build` → **0 warning**（exit 0，3.68s）；`cargo test` → **432 passed / 0 failed / 0 ignored**（lib **362** + main 42 + cli 16 + test_runner 12）。
- ✅ `cargo run -q -- test` → **82 PASS / 0 FAIL / 0 ERROR，exit 0**（`汇总：共 82 个用例，通过 82，失败 0，错误 0`）。
- ✅ `cargo run -q -- run app/sortviz.lfz` → **exit 0**（79 行 / 4170 bytes），阶段 1+2 共 10 处 `[校验通过]`。
- ✅ **bug-01 独立复现（自建夹具，exit 0）**：`p.get()=42`、`p["get"]()=42`（等价）；`p.bump()=1`、`p["bump"]()=2`、`p.n=2`（方括号取到的方法写回**同一 `self`**）。
- ✅ Git：**68 commits**、4 标签、`origin/main` = 本地 HEAD = `6aabdf5`；`docs/slides` 3 文件已跟踪（obs-02 闭合）。
## 本轮发现（观察）
- 🟡 **obs-01 残留**：`README.md` L30/L86 仍写 `431 passed（lib 361）`，实测 **`432 passed（lib 362）`**（修复提交 `6aabdf5` 新增 1 条 lib 单测）。主体已刷新（8 交付物索引真实路径 / 4 标签 / 82 用例 / 341 行 / P10 阶段表均正确）。**owner = release-manager**（低，仅文档）。
- ⚪ **建议项（owner: test-engineer）**：`tests/lfz/test_structs.lfz` L30 只断言 `type(p["norm2"])=="function"`、**未调用** `s["k"]()` → 黑盒未覆盖该调用形态；修复后应补 `assert(p["norm2"]() == 25, …)`，并更新已过时的 L28–L29 注释。
- ⚪ **bug-20260927-02**（`test --json` stdout 非单行，低）：仍打开（非本轮待闭项）；`tests/REPORT.md` §3.1 已自认。
## 阻塞 / 需要支持
- 无阻塞。**结论 CONCERNS**：仅需 release-manager 将 `README.md` L30/L86 的 `431（lib 361）` 改为 `432（lib 362）` → 我复核后即可升级 **`PASS（可打 v1.0-final）`**。
## 下一步计划
- 收到「README 测试数已更新」消息后：复核 L30/L86 = 432/362，出 **PASS** 升级结论并更新报告 §9.5。
- 打 `v1.0-final` 前复核 HEAD 与工作树 clean（本轮已 clean）。
## 关键经验（写给未来的自己）
- **P9 阶段并发提交高发**：本轮开工时修复是**未提交**的 `M src/evaluator.rs`，我 build/test 期间被提交为 `6aabdf5`。**必须两次核 HEAD/status，并核对「我 build 的内容」与「最终 HEAD 内容」是否一致**（工作树==HEAD 即可判定结论适用）。
- **JSON/中文输出用 `cmd /c … > file` 取原始字节**：`Select-String` 过滤中文「汇总」行会因 GBK 控制台编码抓不到；改存文件后按码页解码。
- **黑盒用例「未断言」≠「已覆盖」**：`test_structs.lfz` 刻意不调用 `s["k"]()`，修复后仍是**黑盒缺口**——规范等价性必须补端到端断言，不能只靠 `type(...)` 探测。
- **文档数值会随代码提交顺序过期**：README 刷新（`4727726`）早于 bug-01 修复（`6aabdf5`），导致测试数 431→432 自然失效。发版前须以**最终 HEAD 实测值**重核文档数字。
