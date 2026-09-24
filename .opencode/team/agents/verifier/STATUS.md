# verifier — 工作状态
> 最后更新: 2026-09-24 09:20 by verifier
## 当前状态
**P3.11 复验（rev.2）已完成 → 结论 CONCERNS（列非阻塞），可推进 `v0.2.0`。** 被验提交 `1e8fd5f`（代码）+ `05e42d9`（规范裁定）；复验时 HEAD=`05e42d9`，工作树 clean。报告: `docs/reports/P3-verification.md` §7「复验（rev.2）」。
## 进行中
- （无）
## 已交付（本次）
- `docs/reports/P3-verification.md`（**追加 §7 复验（rev.2）**：逐项复验 + 残留独立确认 + 夹具复跑 + 新规范侧发现 + 更新回归表 + 新结论行）
- `docs/reports/fixtures-p3-rev2/`（15 份：bug01–07·09 最小复现 / 前导空行·注释回归 / span 回归 / `nohdr.lfz` / `spec_9_4_refs_fixed.lfz`）
## 复验结论要点（对 HEAD `05e42d9`，代码=`1e8fd5f`）
- ✅ **原 3×🔴 全修**：多行块（exit 0）、插值 `format_spec`（`  1`）、`if` 表达式（`1`）；均经真实 CLI `cargo run -- run` 复现通过。
- ✅ **原 🟡 bug-04/05 亦修**：`s.missing` 位置 line 3；`3 |> 5` → `管道右侧必须是函数，得到 int`（字节级校验）。
- ✅ **基线**：`cargo build` clean **0 warning**；`cargo test` **373 passed / 0 failed**（358+8+7 = 369+4 真实源码回归用例）；无回归（hello/无 `#42`/运行错位置全部保持）。
- ⚠️ **非阻塞残留**（已独立实测，与声明一致）：bug-06 `let` 重绑定→仍 exit 0/`2`；bug-07 语句首 `{`→仍 SyntaxError（有意推迟）；bug-09 深递归 stderr **30005 行**（折叠未落地）；bug-08 由规范侧闭合（改样例 `r.me`，实现无误）。
- 🟡 **新发现（规范侧，非实现缺陷）**: **spec-20260924-01** —— `docs/spec/syntax.md` §9.4 第 787 行样例 `fn inc() { n += 1; n }` 用单个 `;`，与 A11「单个 `;` 永远 SyntaxError」自相矛盾。建议 owner **language-architect**。**同时是夹具 `spec_9_4_refs.lfz` 失败的根因**（bug-01 已修，此冲突取而代之；属夹具/规范问题，**非解释器缺陷**）。
## 阻塞 / 需要支持
- 无（验证本身不阻塞）。**3×🔴 已清零，无阻塞项**；`v0.2.0` 可打。
- 需 team-lead 转交：spec-01 → language-architect；bug-06 → core-dev+runtime-dev；bug-09 → tooling-dev；bug-07 待 A9 确认后 → core-dev。
## 下一步计划
- 收到 spec-01 修订 / bug-06·09 落地后：复现原用例并更新 §7.7 回归表（已修复 / 仍失败）。
- P4 起对工具链（`lfz test`、`--json`、traceback 折叠渲染）做阶段性抽查验收。
## 关键经验（写给未来的自己）
- **修复复验要"复现原用例 + 反证新根因"**：`spec_9_4_refs.lfz` 从"失败"到"失败"但**根因已换**（bug-01→规范自冲突）——只报"仍失败"会冤枉实现；必须用改写副本（`spec_9_4_refs_fixed.lfz`）反证解释器正确。
- **区分三类缺陷**：解释器缺陷 / 规范缺陷 / 夹具缺陷——本轮的 §9.4 `;` 属**规范**缺陷，须与实现缺陷分开归因与派单。
- 残留项独立实测（bug-06 输出 `2`、bug-09 30005 行）比采信"未修"声明更可靠；数字（行数/退出码/行号）是最硬的证据。
- Windows 控制台会把原生进程中文输出按本地代码页重编码：**判定中文消息一律字节级 hexdump**（本轮 bug-04/05 均以此确认）。
