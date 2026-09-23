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
| P0.5 | 版本基线 | git init + 初始提交（团队脚手架，按 TEAM_SPEC §12 顺序） | release-manager | 交付物 7 | 仓库初始化；`git log` 含初始提交；工作区干净 |
| P1 | 需求 | 需求矩阵+验收标准 | requirements-analyst | 全部（需求基线） | 矩阵完整 |
| P3 | 核心实现 | lexer/parser/AST/eval/builtins（TDD） | core-dev + runtime-dev | 评分项 1 | `lfz` 能跑 hello world；单测全绿 |
| P4 | 工具链 | CLI/REPL/一键测试 runner/打包 | tooling-dev | 评分项 1（+支撑评分项 2） | `lfz test` 可跑通最小套件 |
| P5 | 黑盒测试 | LFZ 全量测试集+覆盖矩阵 | test-engineer | 评分项 2 | 一个命令跑全部；覆盖全特性 |
| P6 | 性能 | LFZ vs Python 基准+报告 | perf-engineer | 评分项 3 | 报告交付 |
| P7 | 文档 | 人类手册 + AI 指南/skill | docs-writer + ai-dx-engineer | 评分项 4 | 评分项 4 完整 |
| P8 | 应用 | ≥200 行 LFZ 应用 + 开发记录 | app-dev | 评分项 5 | 可运行、≥200 行、记录完整 |
| P9 | 验证 | 独立验收报告 | verifier | — | 全部交付物核验 |
| P10 | 发布+答辩 | git 历史+交付清单+PPT | release-manager + ppt-presenter | — | 8 项交付物齐备 |

> **评分项对照（合计 100 分）**：评分项 1 解释器（20）= P3 + P4；评分项 2 自动测试（20）= P5；评分项 3 性能（10）= P6；评分项 4 语法说明 + 人/AI 指南（20）= P2 + P7；评分项 5 Agent 应用（30）= P8。P0.5/P1/P9/P10 为支撑阶段，无独立分值。
> **开工首步**：P0.5（git init + 初始提交）是用户下达开工指令后 team-lead 派发的**第一个任务**（release-manager 的第一任务），先于 P1 执行，保证后续所有工作进入 git 历史。

## 🔴 阻塞
| ID | 任务 | 负责 | 阻塞原因 | 需要支持 |
| -- | -- | -- | -- | -- |
| （无） |  |  |  |  |

## ✅ 已完成
| ID | 阶段 | 交付物 | 负责 | 映射评分项 | 通过条件 | 完成日期 |
| -- | -- | -- | -- | -- | -- | -- |
| P0 | 团队就绪 | 脚手架通过 S1–S5（尚未提交 git，待 P0.5） | team-lead | — | 全部场景通过 | 2026-09-22 |
| P2 | 语言设计 | `docs/spec/{syntax,semantics,interface-contract}.md` 冻结（910/382/298 行）+ ADR D-016 | language-architect | 评分项 4（+支撑评分项 1） | 冻结门禁：core-dev PASS；runtime-dev 3 项非架构级已闭合 | 2026-09-23 |
