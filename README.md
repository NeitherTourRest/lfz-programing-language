# LFZ 语言与解释器 — 程序设计课程作业

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
> 最后更新: 2026-09-23 by release-manager
> 仓库地址: https://github.com/NeitherTourRest/lfz-programing-language

LFZ 是一个解释型通用脚本语言及其解释器，配套黑盒测试、性能对比、人/AI 开发指南与 Agent 开发应用。项目由 14 名 opencode AI 开发团队协作完成。

## 当前状态
| 阶段 | 内容 | 状态 |
| -- | -- | -- |
| P0 | 团队就绪（脚手架通过 S1–S5） | ✅ 已完成 |
| P2 | 语言设计（`docs/spec/` v1 三件套冻结，D-016） | ✅ 已完成 |
| P0.5 | 版本基线（git init + 初始提交 + `v0.1.0` 标签） | 🟡 进行中 |
| 下一步 | P1 需求基线同步 / P3 核心实现（Rust，依冻结契约） | ⏭ 待启动 |

> **本 README 由 release-manager 在每个阶段里程碑实时更新**，以反映最新交付状态、Git 基线与版本标签。

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

# 占位：安装与运行命令待工具链阶段（P4）确定
# lfz run <file>   运行 LFZ 脚本
# lfz test         一键执行全部测试
```

## 许可证
本项目采用 **MIT License**，全文见 [`LICENSE`](LICENSE)。

Copyright (c) 2026 MakeChase
