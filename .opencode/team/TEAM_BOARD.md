# 团队任务看板
> 最后更新: 2026-09-23 by team-lead

本文件为活文档，唯一写者 team-lead。其他角色只读，通过结构化汇报请求 team-lead 更新。

## 🔵 进行中
| ID | 任务 | 负责 | 依赖 | 状态 | 产出/证据 |
| -- | -- | -- | -- | -- | -- |
| （无） |  |  |  |  |  |

## 🟡 待办
| ID | 阶段 | 交付物 | 负责 | 映射评分项 | 通过条件 |
| -- | -- | -- | -- | -- | -- |
| P1 | 需求 | 需求矩阵+验收标准同步（对齐冻结 spec v1） | requirements-analyst | 全部（需求基线） | 矩阵完整 |
| P3 | 核心实现 | lexer/parser/AST/eval/builtins（TDD） | core-dev + runtime-dev | 评分项 1 | `lfz` 能跑 hello world；单测全绿 |
| P4 | 工具链 | CLI/REPL/一键测试 runner/打包 | tooling-dev | 评分项 1（+支撑评分项 2） | `lfz test` 可跑通最小套件 |
| P5 | 黑盒测试 | LFZ 全量测试集+覆盖矩阵 | test-engineer | 评分项 2 | 一个命令跑全部；覆盖全特性 |
| P6 | 性能 | LFZ vs Python 基准+报告 | perf-engineer | 评分项 3 | 报告交付 |
| P7 | 文档 | 人类手册 + AI 指南/skill | docs-writer + ai-dx-engineer | 评分项 4 | 评分项 4 完整 |
| P8 | 应用 | ≥200 行 LFZ 应用 + 开发记录 | app-dev | 评分项 5 | 可运行、≥200 行、记录完整 |
| P9 | 验证 | 独立验收报告 | verifier | — | 全部交付物核验 |
| P10 | 发布+答辩 | git 历史+交付清单+PPT | release-manager + ppt-presenter | — | 8 项交付物齐备 |

> **评分项对照（合计 100 分）**：评分项 1 解释器（20）= P3 + P4；评分项 2 自动测试（20）= P5；评分项 3 性能（10）= P6；评分项 4 语法说明 + 人/AI 指南（20）= P2 + P7；评分项 5 Agent 应用（30）= P8。P0.5/P1/P9/P10 为支撑阶段，无独立分值。
> **版本纪律（自 P0.5 起全程生效）**：release-manager 在每个阶段里程碑做**原子提交 + 附注标签 + 实时更新 README + push**；远程 = https://github.com/NeitherTourRest/lfz-programing-language（Public）。

### P3 子阶段台账（用户协议：每子阶段 → 独立检查 → 原子提交）
| 子阶段 | 内容 | 负责 / 执行者 | 状态 | 提交 |
| -- | -- | -- | -- | -- |
| P3.0 | Cargo 骨架（lib + bin，std only） | core-dev | ✅ | `69a57d7` |
| P3.1 | `span.rs` + `error.rs`（12 错误类 + 方法 + `R<T>`） | core-dev | ✅ | `4035f86` |
| P3.2 | `loader.rs`（UTF-8/BOM/`ext`/`#42`/`line_base`） | core-dev | ✅ | `7d41e17` |
| P3.3a | `lexer.rs` CODE 模式核心记号 | core-dev | ✅ | `4c68d18` |
| P3.3b | `lexer.rs` STR/INTERP + 未闭合块注释 | core-dev | ✅ | `4f2a22e` |
| P3.4a | `ast.rs` 全节点带 `Span`（共享接口门禁） | core-dev | ✅ | `1dab6f8` |
| P3.4b1 | `parser.rs` 骨架 + 语句层 | Sisyphus-Junior（**改派**） | ✅ | `89b8624` |
| P3.4b2a | `parser.rs` 后缀 + 数组/结构体字面量 | Sisyphus-Junior | ✅ | `81a4b8a` |
| P3.4b2b | `parser.rs` 二元运算符 + 赋值 | Sisyphus-Junior | ✅ | `ae88d38` |
| P3.4b3a | `parser.rs` 控制流语句 | Sisyphus-Junior | ✅ | `53ca3e2` |
| P3.4b3b | `parser.rs` `fn`/`struct` 声明 + 函数字面量 | Sisyphus-Junior | ✅ | `0ce5123` |
| P3.5 | `parser.rs` 特色四件套（管道/`;;`/插值/`i64::MIN`） | Sisyphus-Junior | ✅ | `682d1fb` |
| P3.6a | `value.rs` + `env.rs`（A1/A2/`ScopeDebug`） | runtime-dev | ✅ | `49d485f` |
| P3.6b | 值语义辅助（A6 环安全 `==` + §4.5.6 全序） | runtime-dev | ✅ | `88ec1bc` |
| P3.7 | `evaluator.rs` 核心 | runtime-dev | ✅ | `cab79ba` |
| P3.8 | `evaluator.rs` 语义定稿 | runtime-dev | ✅ | `f9aec70` |
| P3.9a | `builtins.rs` 非高阶 47 个 | runtime-dev | ✅ | `b81680b` |
| P3.9b | `builtins.rs` 高阶 7 个（**§10.7 54/54**） | runtime-dev | ✅ | `d51755a` |
| P3.10 | 最小 CLI + `examples/hello.lfz` | tooling-dev | ✅ | `c7627a6` |
| **P3.11** | **独立验收（rev.1 FAIL → rev.2 CONCERNS → rev.3 PASS，10/10 缺陷闭环、0 回归）→ 里程碑标签 `v0.2.0`** | verifier | ✅ | tag `v0.2.0` |
| 规范 | 7 处契约缺口闭合（先 ADR 后改 `docs/spec`） | language-architect | ✅ | `711d6b9` `5cdcc1b` |

> **执行者改派记录**：`parser.rs` 因原负责人 2 次未交付，自 P3.4b1 起由 team-lead 改派 **Sisyphus-Junior**（`unspecified-high`），并将任务切成 ≤1 模块的小批次，每个批次以「**必须贴出文件实际字节数**」为真假完成判据；改派后 6 个批次全部一次落地。

## 🔴 阻塞
| ID | 任务 | 负责 | 阻塞原因 | 需要支持 |
| -- | -- | -- | -- | -- |
| （无） |  |  |  |  |

## ✅ 已完成
| ID | 阶段 | 交付物 | 负责 | 映射评分项 | 通过条件 | 完成日期 |
| -- | -- | -- | -- | -- | -- | -- |
| P0 | 团队就绪 | 脚手架通过 S1–S5 | team-lead | — | 全部场景通过 | 2026-09-22 |
| P2 | 语言设计 | `docs/spec/{syntax,semantics,interface-contract}.md` 冻结（910/382/298 行）+ ADR D-016 | language-architect | 评分项 4（+支撑评分项 1） | 冻结门禁：core-dev PASS；runtime-dev 3 项非架构级已闭合 | 2026-09-23 |
| P0.5 | 版本基线 | 本地 `git init` + 初始提交 `e060b21` + `v0.1.0`；远程 `github.com/NeitherTourRest/lfz-programing-language`（Public） | release-manager | 交付物 7 | `git ls-remote`：`main`=`35e5f62`、tag=`v0.1.0`；本地 = 远程 | 2026-09-23 |
