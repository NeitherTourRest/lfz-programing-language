# ai-dx-engineer — 工作状态
> 最后更新: 2026-09-27 22:00 by ai-dx-engineer

## 当前状态
**P7b 完成**：AI 向开发指南 + `lfz-programming` SKILL 已交付并**实测通过**。
- 交付物：`.opencode/skills/lfz-programming/{SKILL.md, prompt-template.md, VERIFICATION.md}`、`docs/guide/ai/{README.md, examples/*.lfz}`。
- `interface-contract.md` §11.1 五条硬性要求**逐条写死**并有对照表（见 VERIFICATION.md §1）。
- 实测：3 个示例 + 最小基线全部 exit 0；13 类错误场景与头部规则边界实测留证。

## 进行中
- （无）

## 阻塞 / 需要支持
- （无）
- ⚠️ 已知**实现间隙**（非我方改动范围，已报告 team-lead）：CLI v1 只实现 `lfz run <file>` / `lfz test`，**未实现** `-e` / stdin / REPL；而 spec §11.1/B9 将后三者列为「豁免 `#42`」的入口。当前 SKILL 已如实写明实际可用入口，避免 AI 误用。建议 tooling-dev 决定是否在 v1 补 `-e`/stdin，或在 spec/文档中标注为 v1.1 backlog。

## 下一步计划
1. 与 app-dev 闭环：收集其在 P8（≥200 行应用）中遇到的卡点，归类回填 SKILL §4「常见陷阱」。
2. `docs/spec/` 若变更 → 立即同步 SKILL/README，并重跑最小任务基线（见 SKILL §9）。
3. 与 docs-writer 核对示例一致性（共享已验证示例，避免重复劳动）。
4. 如需第三方「全新 Agent」实测，请 team-lead 调度。

## 关键经验（写给未来的自己）
- **AI 指南三把刀**：正反例对照 + 常见陷阱前置 + 完整可运行示例；本次实测证明「`${}` 内不能写 `\"`」「管道 data-last」这类**反直觉点**最易让 AI 卡壳，必须配正反例。
- **别照抄 spec 措辞当示例**：`let a = 1; a = 2` 会被 `;` 先拦成 SyntaxError，不是 TypeError；示例必须**实跑**再写进指南。
- **先跑再写**：所有消息样本、退出码、边界均来自真实 `cargo run`；初稿 6 处错误全部由实测暴露（见 VERIFICATION.md §4）。
- CLI 事实源：`cargo run --quiet -- --help`；错误输出形态：加载/解析期无 Traceback 头、运行期有。
- `;;` 文本走 **stdout**（`--json` 下**不**重定向）；`print` 在 `--json` 下走 stderr。
