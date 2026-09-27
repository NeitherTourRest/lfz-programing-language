# LFZ 语言与解释器 — 程序设计课程作业

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
> 最后更新: 2026-09-27 by release-manager
> 仓库地址: https://github.com/NeitherTourRest/lfz-programing-language

LFZ 是一个解释型通用脚本语言及其解释器，配套黑盒测试、性能对比、人/AI 开发指南与 Agent 开发应用。项目由 14 名 opencode AI 开发团队协作完成。

## 当前状态
| 阶段 | 内容 | 状态 |
| -- | -- | -- |
| P0 | 团队就绪（脚手架通过 S1–S5） | ✅ 已完成 |
| P0.5 | 版本基线（git init + 初始提交 + `v0.1.0` 标签） | ✅ 已完成 |
| P2 | 语言设计（`docs/spec/` v1 三件套冻结，D-016） | ✅ 已完成 · 里程碑 **`v0.2.0`** |
| P3 | 解释器核心（loader / lexer / ast / parser / evaluator / builtins(54) / CLI） | ✅ 已完成 |
| P4 | 工具链（CLI、一键测试 runner、打包脚本） | ✅ 已完成 |
| P5 | 黑盒测试集（25 正向 + 56 负例，覆盖矩阵） | ✅ 已完成 · 里程碑 **`v0.3-tested`** |
| P6 | 性能基准（LFZ vs Python，预热 + 多轮 + 中位数） | ✅ 已完成 |
| P7 | 文档（人类手册 5 篇 + AI 指南 / skill） | ✅ 已完成 |
| P8 | 应用（`app/sortviz.lfz` 341 行，5 算法） | ✅ 已完成 · 里程碑 **`v0.4-app`** |
| P9 | 独立验收（交付物级，verifier） | ✅ 已完成 · 结论 **CONCERNS（无阻塞）** |
| P10 | 交付完整性（README 刷新 + 交付清单核对 + 提交） | 🔄 收尾中 |

> **本 README 由 release-manager 在每个阶段里程碑实时更新**，以反映最新交付状态、Git 基线与版本标签。

## 质量基线（实测）
| 项 | 命令 | 实测结果 |
| -- | -- | -- |
| 构建 | `cargo build` | **0 warning / 0 error**（零第三方依赖，仅 std） |
| 单元测试 | `cargo test` | **431 passed / 0 failed / 0 ignored**（lib 361 + main 42 + cli 16 + test_runner 12） |
| 黑盒测试 | `cargo run -- test` | **82 个用例，通过 82，失败 0，错误 0**（exit 0，一个命令跑全部） |
| 端到端 | `cargo run -- run examples/hello.lfz` | `Hello, LFZ!`（exit 0） |
| 应用 | `cargo run -- run app/sortviz.lfz` | exit 0；**5 算法**全部 `[校验通过]`；**341 行** |

## 交付物索引（8 项）
| # | 交付物 | 位置 | 评分项 |
| -- | -- | -- | -- |
| 1 | LFZ 语法规则文档 | `docs/spec/`（syntax / semantics / interface-contract） | — |
| 2 | LFZ 解释器源程序 | `src/`（15 文件，Rust 仅 std） | 1（20 分） |
| 3 | LFZ 完整黑盒测试集 | `tests/`（`lfz/` + `fixtures/` + `cases.json` + `coverage-matrix.md` + `REPORT.md`） | 2（20 分） |
| 4 | LFZ 性能测试报告 | `benchmarks/` + `docs/reports/performance.md` | 3（10 分） |
| 5 | 开发指南（人 + AI） | `docs/guide/` + `docs/guide/ai/` + `.opencode/skills/lfz-programming/` | 4（20 分） |
| 6 | 应用源代码 + 开发记录 | `app/sortviz.lfz` + `app/DEV_RECORD.md` | 5（30 分） |
| 7 | Git 历史记录 | `.git/`（仓库历史 + 版本标签） | — |
| 8 | 系统介绍 PPT | `docs/slides/`（`LFZ-defense.pptx` 14 页 + `demo-script.md` + `qa-prep.md`） | — |

## 验收与证据
| 证据 | 位置 |
| -- | -- |
| P3 解释器核心验收（rev.3 终验 **PASS**，缺陷闭环 10/10，新增回归 0） | [`docs/reports/P3-verification.md`](docs/reports/P3-verification.md) |
| P9 交付物级独立验收（8 项逐项实跑，**CONCERNS（无阻塞）**） | [`docs/reports/P9-verification.md`](docs/reports/P9-verification.md) |
| 黑盒测试报告（覆盖矩阵说明 + 已知偏差自认） | [`tests/REPORT.md`](tests/REPORT.md) |
| 交付完整性核对报告（对照 8 项，≥3 轮） | [`docs/reports/delivery-checklist.md`](docs/reports/delivery-checklist.md) |
| 项目状态检查 | [`docs/reports/status-check.md`](docs/reports/status-check.md) |

> P3 复核夹具与命令原文见 [`docs/reports/fixtures-p3-rev3/`](docs/reports/fixtures-p3-rev3/)。
> P9 报告同时记录 2 项非阻塞遗留（`bug-20260927-01`：`s["k"]()` 未绑定 `self`；`obs-01`：README 陈旧），本 README 刷新即闭合 `obs-01`。

## 版本里程碑
| 标签 | 对应阶段 | 说明 |
| -- | -- | -- |
| `v0.1.0` | P0.5 | 项目脚手架 + 团队宪法 + 冻结 v1 语言规范 |
| `v0.2.0` | P2/P3 | 语言设计冻结 + 解释器核心实现 |
| `v0.3-tested` | P5 | 黑盒测试集完成（54 内置覆盖 + 覆盖矩阵） |
| `v0.4-app` | P8 | 排序算法可视化应用（341 行，5 算法）+ 开发记录 |

> `v1.0-final` 将在 verifier 对 P9 遗留项复验 PASS 后由 team-lead 下令打标。

## 团队说明
本项目由 14 名 AI 开发团队运作。团队宪法见 `AGENTS.md`；团队共享记忆（组织架构、看板、需求、决策、状态）见 `.opencode/team/`。任务入口：`task-info.md`（原始要求）、`.opencode/team/REQUIREMENTS.md`（需求矩阵）、`.opencode/team/PROJECT_STATE.md`（全局状态）。

## 快速开始
```bash
# 克隆仓库
git clone https://github.com/NeitherTourRest/lfz-programing-language.git
cd lfz-programing-language

# 构建（Rust 工具链；无第三方依赖）
cargo build

# 运行 LFZ 脚本
#   注意：.lfz 文件首行必须为 #42（LFZ 头标记），否则报 CosmosAnswerError
cargo run -- run examples/hello.lfz
# 输出: Hello, LFZ!

# 运行单元测试（431 passed）
cargo test

# 运行黑盒测试集（一个命令跑全部，82 用例）
cargo run -- test

# 运行示例应用（排序算法可视化）
cargo run -- run app/sortviz.lfz
```

> `examples/hello.lfz` 内容：
> ```lfz
> #42
> print("Hello, LFZ!")
> ```

## 许可证
本项目采用 **MIT License**，全文见 [`LICENSE`](LICENSE)。

Copyright (c) 2026 MakeChase
