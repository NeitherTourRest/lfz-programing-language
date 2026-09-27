# ai-dx-engineer — 工作状态
> 最后更新: 2026-09-27 23:10 by ai-dx-engineer

## 当前状态
**P7c 完成**：`.opencode/skills/lfz-programming/` 已升级为「**可直接交付给编程 Agent 使用**」的 Skill 包（增量，**未重做 P7b**）。四件事全部落地并有证据。

## P7c 交付（增量）
1. **对照表**：把「`task-info.md` 第 4 条」拆成 (a)–(d) + 「`interface-contract.md` §11.1 五条硬性要求」逐条映射到 `SKILL.md` 落点，写入 `VERIFICATION.md` §1；**结论：达标、无缺口**（当场无需补）。
2. **新增 `README.md`**（skill 包「安装与使用」最后一公里，5596 B/121 行）：是什么/给谁用/何时加载；**三种安装方式**（① 本项目已就位、重启 opencode 生效；② 全局 `%USERPROFILE%\.config\opencode\skills\lfz-programming\`；③ 其他 harness 如 DSH `dsh-skill-filesystem` 配置片段）；**运行前提**（cargo 的 PATH 坑、`dist\lfz.exe`、`cargo run --quiet -- run`、入口限制、退出码）；**它不做什么**（不定义新语法、冲突以 spec 为准）。
3. **`SKILL.md` 从"文档"变"可执行流程"**：顶部新增 🚫 **硬纪律**「不要凭记忆写 LFZ」；新增 **§0.5 Agent 标准工作流（5 步）**（末步强制 `cargo run --quiet -- run`、必须 exit 0）；新增 **§4.5 错误类 → 最常见原因 → 修法**（26 行，串起 §2 与 §4）；§7 新增 **L4** 综合示例；§10 增补 README 指针。
4. **第 4 次实测**：新写「词频统计」`docs/guide/ai/examples/04_wordcount.lfz`（struct + **带参方法** + `while` + 管道 + 插值），**一次跑通 exit 0**；记录进 `VERIFICATION.md` §7（含用了哪几条 skill、踩坑、解决）。

## 进行中
- （无）

## 阻塞 / 需要支持
- （无）
- ⚠️ 遗留（非我方范围，P7b 已报告 team-lead）：CLI v1 只有 `run <file>` / `test`，**未实现** `-e`/stdin/REPL，而 spec §11.1 将其列为「豁免 `#42`」入口。SKILL/README 已**如实写明实际可用入口**，避免 AI 误用。
- 注：`README.md` §2 方式③的 DSH 片段为**示意**（字段名以 DSH 文档为准），若实际字段名不同需按文档微调。

## 下一步计划
1. 与 **app-dev** 闭环：收集 P8（≥200 行应用）卡点，归类回填 SKILL §4 / §4.5。
2. `docs/spec/` 若变更 → **立即同步** SKILL/README，并重跑最小基线 + 4 个示例。
3. 与 **docs-writer** 核对示例一致性（共享已验证示例，避免重复劳动）。
4. 如需第三方「全新 Agent」实测，请 team-lead 调度。

## 关键经验（写给未来的自己）
- **AI 指南 = 文档 × 流程**：光有语法速查不够，必须给「**5 步工作流 + 强制实跑 + 错误→修法表**」，让 Agent 每步都有可执行的下一步。
- **"安装与使用"是最后一公里**：project/global/harness 三种安装 + PATH 坑 + 入口限制，缺一项 Agent 就可能在"怎么跑起来"上卡死。
- **实测最有价值的是反直觉点**：本次发现格式说明符 `:>2` **必须写在 `${}` 内**（写在外面会输出字面量 `:>`）——来自别语言的直觉，已回填 §7。先写 `probe` 探测再写正式程序，能显著提高一次成型率。
- 事实源边界：`docs/spec/**` 只读；本包任何语法在 spec 找不到出处即违规。
