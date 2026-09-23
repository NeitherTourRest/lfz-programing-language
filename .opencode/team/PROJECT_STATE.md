# 项目全局状态
> 最后更新: 2026-09-23 by team-lead

本文件为活文档，唯一写者 team-lead。其他角色只读。

## 1. 一句话目标
设计并实现 LFZ 解释型脚本语言及其解释器，配套黑盒测试、性能对比、人/AI 开发指南与 Agent 开发应用，完成 8 项交付物并通过线下验收。

## 2. 当前阶段
**已开工。P0 团队就绪 ✅、P2 语言设计冻结 ✅、P0.5 版本基线（本地 + 远程）✅。**
- **P2 语言设计已冻结**（DECISIONS **D-016**）：`docs/spec/{syntax,semantics,interface-contract}.md` 三件套定稿（60010/28153/26116 字节；910/382/298 行；UTF-8 无 BOM），冻结基线为 `DRAFT-LFZ-v0.5.md`（含 3 处补钉）。冻结门禁：core-dev 复审 **PASS**（文法无回溯可实现）；runtime-dev 复审 CONCERNS 3 项（非架构级，已闭合）。
- **P0.5 版本基线已完成（本地 + 远程）**：`git init -b main`、初始提交 `e060b21`、附注标签 `v0.1.0`、版本管理纪律 ADR；远程仓库 **https://github.com/NeitherTourRest/lfz-programing-language**（**Public**，默认分支 `main`，`main` = `35e5f62`，`v0.1.0` 已推送）；本地与远程一致。⚠️ **认证账号 login = `NeitherTourRest`**（其 display name = `MakeChase`）——GitHub 上另有同名不同账号 `MakeChase`，故仓库 URL 用 `NeitherTourRest`。
- **关键决策**：**实现语言 = Rust**（D-011，工具链已装并 `cargo build` 冒烟通过）；v1 = 5 特色 + `float`；`#42` 文件头 + Python 式中文错误；契约先行 + 双轨 TDD；阶段划分见 D-009；**协议 MIT**；**全程自动版本管理 + README 实时更新**。
- **P3 进行中（拆为 12 个子阶段，每阶段「检查 + 原子提交」）**：已完成 **P3.0**（Cargo 骨架）/ **P3.1**（`span`+`error`）/ **P3.2**（loader）/ **P3.3a**（lexer CODE 模式）/ **P3.3b**（lexer STR/INTERP）/ **P3.6a**（value+env）/ **P3.9a**（47 个非高阶内置）；进行中 **P3.4a**（`ast.rs`）/ **P3.6b**（值语义辅助）；待办 P3.4b/P3.5（parser）/P3.7/P3.8（evaluator）/P3.9b（7 个高阶内置）/P3.10（最小 CLI）/P3.11（验收 → `v0.2.0`）。**测试基线 135 passed / 0 failed / 0 warnings**。子阶段台账见 `TEAM_BOARD.md`。
- **规范侧**：7 处契约缺口已按「先 ADR、后改 `docs/spec`」闭合（`ValueMsg::EmptyExtremum`/`BadRange`、`pop` 的 `idx/len`、`floor/ceil/round` 复用 `int(float)`、`del` 限数据面、`insert` 不接受负索引、未闭合块注释 → `SyntaxError`）。

## 3. 里程碑进度表
| 阶段 | 里程碑 | 状态 | 负责人 |
| -- | -- | -- | -- |
| P0 | 团队就绪（脚手架通过 S1–S5） | ✅ 已完成 | team-lead |
| P0.5 | 版本基线（git init + 初始提交 + `v0.1.0` + GitHub 远程） | ✅ 已完成 | release-manager |
| P1 | 需求矩阵+验收标准 | 🟡 待办 | requirements-analyst |
| P2 | 语言设计（语法+语义+接口契约+ADR） | ✅ 已完成（`docs/spec/` 冻结，D-016） | language-architect |
| P3 | 核心实现（lexer/parser/AST/eval/builtins，TDD） | 🟡 待办 | core-dev + runtime-dev |
| P4 | 工具链（CLI/REPL/一键测试 runner/打包） | 🟡 待办 | tooling-dev |
| P5 | 黑盒测试（全量测试集+覆盖矩阵） | 🟡 待办 | test-engineer |
| P6 | 性能（LFZ vs Python 基准+报告） | 🟡 待办 | perf-engineer |
| P7 | 文档（人类手册 + AI 指南/skill） | 🟡 待办 | docs-writer + ai-dx-engineer |
| P8 | 应用（≥200 行 LFZ 应用 + 开发记录） | 🟡 待办 | app-dev |
| P9 | 验证（独立验收报告） | 🟡 待办 | verifier |
| P10 | 发布+答辩（git 历史+交付清单+PPT） | 🟡 待办 | release-manager + ppt-presenter |

## 4. 交付物对照表（8 项提交物）
| # | 提交物 | 评分权重 | 状态 | 负责人 | 位置 |
| -- | -- | -- | -- | -- | -- |
| 1 | LFZ 语法规则文档 | 20（评分项 4 一部分） | ✅ 已冻结（v1 三件套） | language-architect | `docs/spec/` |
| 2 | LFZ 解释器源程序 | 20（评分项 1） | 未开始 | core-dev + runtime-dev | `src/` |
| 3 | LFZ 完整黑盒测试集 | 20（评分项 2） | 未开始 | test-engineer | `tests/` |
| 4 | LFZ 性能测试报告 | 10（评分项 3） | 未开始 | perf-engineer | `benchmarks/` + `docs/reports/performance.md` |
| 5 | 开发指南（人 + AI） | 20（评分项 4 一部分） | 未开始 | docs-writer + ai-dx-engineer | `docs/guide/` + `.opencode/skills/lfz-programming/` |
| 6 | 应用源代码 + 开发记录 | 30（评分项 5） | 未开始 | app-dev | `app/` + `app/DEV_RECORD.md` |
| 7 | Git 历史记录 | 无独立分值（交付完整性） | ✅ 已建立（本地 + 远程） | release-manager | `.git/` + https://github.com/NeitherTourRest/lfz-programing-language |
| 8 | 系统介绍 PPT | 无独立分值（答辩载体） | 未开始 | ppt-presenter | `docs/slides/` |

## 5. 关键事实
- **技术栈（已确认，D-011）**：解释器用 **Rust**（cargo 工程，优先仅用 std）。**工具链已装并验证**：rustc/cargo 1.98.1、`stable-x86_64-pc-windows-msvc`、链接器可用、`cargo build` 冒烟通过。Rust 实现需重规划运行时值模型（`enum Value` + `Rc<RefCell<...>>`）与 `Result` 错误处理。
- **目录约定**：`docs/spec/`（语法/语义/接口契约）、`src/`（lexer/parser/ast/evaluator/builtins/env）、`tests/`（黑盒测试）、`benchmarks/`（性能脚本）、`docs/guide/`（人类文档）、`docs/reports/`（各类报告）、`app/`（应用）、`.opencode/`（团队与技能）。
- **运行命令**：待定（P4 由 tooling-dev 定义 `lfz run <file>`、`lfz test`）。
- **Git 与远程**：仓库根 = 项目根；默认分支 `main`；`origin` = https://github.com/NeitherTourRest/lfz-programing-language.git（Public）；协议 **MIT**（`LICENSE`，`Copyright (c) 2026 MakeChase`）。
- **环境坑（重要）**：本机 git 配了代理 `http://127.0.0.1:7890`，但 **gh CLI 不读 git 配置**，调用 gh 前须设 `HTTPS_PROXY`/`HTTP_PROXY` 同值，否则直连 `github.com:443` 超时。

## 6. 当前风险与开放决策
- **已消解（P0.5 完成）**：脚手架与全部工作已纳入 git 并推送远程（交付物 7 已建立基线）。版本管理纪律（main / 里程碑原子提交 / conventional commits / 附注标签自 `v0.1.0` 起 / 禁 force-push / README 由 release-manager 实时更新）见 DECISIONS。
- **D-004 已定案（用户）**：实现语言 = **Rust**（D-011）。工具链已装并验证 ✅。**风险：Rust 返工率高于 Python，须以契约先行 + 编译驱动 + 小步提交 + 频繁 `cargo test` 控制。**
- **应用选题已定（用户）**：排序算法可视化（P8；由 app-dev 展开设计）。
- **已确认（D-011）**：v1 功能范围 = 5 特色**全纳入**、`float` **纳入**、应用选题 = **排序算法可视化**（P8）。
- **遗留（不阻塞实现）**：4 项保留默认（`;;` 作用域 = 全可见链、`;;` 通道 = stdout、多行续行 = 仅括号内、单个 `;` = 永远 SyntaxError），实现中如需变更须先 ADR 后改文档。
- **开放决策 2**：`external_directory` 不加硬限制，采用行为约束 + 构建期 S4 验证。
- **开放决策 3**：应用选题由 app-dev 在 P8 前提出 2–3 个方案，经 team-lead 与用户确认。
- **风险**：应用（≥200 行，评分 30 权重最高）依赖解释器稳定，P3/P4 需尽早完成以释放 app-dev。
