# LFZ 语言与解释器 — 程序设计课程作业

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
> 最后更新: 2026-09-24 by release-manager
> 仓库地址: https://github.com/NeitherTourRest/lfz-programing-language

LFZ 是一个解释型通用脚本语言及其解释器，配套黑盒测试、性能对比、人/AI 开发指南与 Agent 开发应用。项目由 14 名 opencode AI 开发团队协作完成。

## 当前状态
| 阶段 | 内容 | 状态 |
| -- | -- | -- |
| P0 | 团队就绪（脚手架通过 S1–S5） | ✅ 已完成 |
| P0.5 | 版本基线（git init + 初始提交 + `v0.1.0` 标签） | ✅ 已完成 |
| P2 | 语言设计（`docs/spec/` v1 三件套冻结，D-016） | ✅ 已完成 |
| P3 | 解释器核心（loader / lexer / ast / parser / evaluator / builtins(54) / CLI） | ✅ 已完成 · 里程碑 **`v0.2.0`** |
| 下一步 | P4 工具链 / P5 黑盒测试 / 性能基准（依冻结契约） | ⏭ 待启动 |

> **本 README 由 release-manager 在每个阶段里程碑实时更新**，以反映最新交付状态、Git 基线与版本标签。

## 验收证据
> P3（LFZ v1 解释器核心）经 **verifier 独立验收**（rev.3 终验），依据报告 [**`docs/reports/P3-verification.md`**](docs/reports/P3-verification.md)。

| 项 | 结论 |
| -- | -- |
| 终验结论 | **PASS（可打 `v0.2.0`）**，验收基线 clean HEAD `c6638cc` |
| 缺陷闭环 | **10 / 10（100%）**：3×🔴 + 4×🟡 + 1×🟢 + 1 规范侧全部闭环；**新增回归 0** |
| 构建 | `cargo build` → **0 warning / 0 error** |
| 测试 | `cargo test` → **377 passed / 0 failed / 0 ignored**（库 361 + bin 9 + cli 7；`ignored` 由 1 归 0） |
| 端到端 | `cargo run -- run examples/hello.lfz` → `Hello, LFZ!`（exit 0） |

复核夹具与命令原文见 [`docs/reports/fixtures-p3-rev3/`](docs/reports/fixtures-p3-rev3/)。

## 交付物索引（8 项）
| # | 交付物 | 位置 |
| -- | -- | -- |
| 1 | LFZ 语法规则文档 | `docs/spec/` |
| 2 | LFZ 解释器源程序 | `src/` |
| 3 | LFZ 完整黑盒测试集 | `tests/` |
| 4 | LFZ 性能测试报告 | `benchmarks/` + `docs/reports/performance.md` |
| 5 | 开发指南（人 + AI） | `docs/guide/` + `.opencode/skills/lfz-programming/` |
| 6 | 应用源代码 + 开发记录 | `app/` + `app/DEV_RECORD.md` |
| 7 | Git 历史记录 | `.git/` |
| 8 | 系统介绍 PPT | `docs/slides/` |

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

# 运行测试
cargo test
```

> `examples/hello.lfz` 内容：
> ```lfz
> #42
> print("Hello, LFZ!")
> ```

## 许可证
本项目采用 **MIT License**，全文见 [`LICENSE`](LICENSE)。

Copyright (c) 2026 MakeChase
