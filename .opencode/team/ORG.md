# LFZ 项目组织架构与角色索引
> 最后更新: 2026-09-22 by 团队初始化

本文件描述 LFZ 项目 14 名 AI 开发团队的层级组织、角色索引与协作规则。权威性仅次于 `AGENTS.md`（团队宪法）。冲突时以宪法为准。

## 1. 组织架构（ASCII 树）

```
team-lead（管理组 · primary · 唯一调度者，对最终交付负责）
│
├─ 管理组
│   ├─ team-lead            主调度智能体（本节点，总指挥）
│   └─ requirements-analyst 需求分析师
├─ 设计组
│   └─ language-architect   语言架构师
├─ 实现组
│   ├─ core-dev             核心开发（前端：lexer/parser/AST）
│   ├─ runtime-dev          运行时开发（evaluator/builtins/env）
│   └─ tooling-dev          工具链开发（CLI/测试 runner/打包）
├─ 质量组
│   ├─ test-engineer        测试工程师（黑盒测试集）
│   ├─ perf-engineer        性能工程师（基准对比）
│   └─ verifier             验证工程师（独立验收）
├─ 文档组
│   ├─ docs-writer          文档工程师（人类向）
│   └─ ai-dx-engineer       AI 赋能工程师（AI 向指南/skill）
├─ 应用组
│   └─ app-dev              应用开发工程师（≥200 行 LFZ 应用）
└─ 发布组
    ├─ release-manager      发布经理（git/版本/交付清单）
    └─ ppt-presenter        演示工程师（PPT/演示脚本/答辩预案）
```

## 2. 角色索引表

| 角色 | 文件路径 | 一句话职责 | 主要交付物 |
|------|---------|-----------|-----------|
| team-lead | `.opencode/agents/team-lead.md` | 用户与团队唯一接口，接令/研判/调度/监督/收尾，对最终交付负责 | `TEAM_BOARD.md`、`PROJECT_STATE.md` 维护；调度计划；用户汇报 |
| requirements-analyst | `.opencode/agents/requirements-analyst.md` | 维护唯一需求事实源，把任务翻译为可验收条目 | `REQUIREMENTS.md`（R-ID 矩阵） |
| language-architect | `.opencode/agents/language-architect.md` | LFZ 语法与语义唯一权威，产出语法/语义/接口契约 | `docs/spec/syntax.md`、`docs/spec/semantics.md`、`docs/spec/interface-contract.md`、设计 ADR |
| core-dev | `.opencode/agents/core-dev.md` | 实现词法/语法分析器与 AST（前端） | `src/` 下 lexer/parser/ast + 单元测试 |
| runtime-dev | `.opencode/agents/runtime-dev.md` | 实现求值器/内建函数/作用域环境 | `src/` 下 evaluator/builtins/env + 单元测试 |
| tooling-dev | `.opencode/agents/tooling-dev.md` | 让 LFZ 可运行、可测试、可交付 | `lfz` CLI、REPL、一键测试 runner、打包脚本 |
| test-engineer | `.opencode/agents/test-engineer.md` | 完整黑盒测试集（评分项 2，20 分） | `tests/` 测试脚本、覆盖矩阵、测试报告 |
| perf-engineer | `.opencode/agents/perf-engineer.md` | 性能测试与 Python 对比（评分项 3，10 分） | `benchmarks/`、`docs/reports/performance.md` |
| verifier | `.opencode/agents/verifier.md` | 独立验收（模拟助教），只验证不修复 | `docs/reports/verification.md`、缺陷单、回归记录 |
| docs-writer | `.opencode/agents/docs-writer.md` | 人类向文档（评分项 4 一半） | `docs/guide/` 用户手册、教程 |
| ai-dx-engineer | `.opencode/agents/ai-dx-engineer.md` | AI 向指南/skill，让编程 Agent 能用 LFZ 写代码 | `.opencode/skills/lfz-programming/SKILL.md`、AI 提示模板、实测记录 |
| app-dev | `.opencode/agents/app-dev.md` | 用 LFZ 开发 ≥200 行应用（评分项 5，30 分，权重最高） | `app/` 源码、`app/DEV_RECORD.md` |
| release-manager | `.opencode/agents/release-manager.md` | git 版本管理与交付完整性 | git 历史纪律、版本标签、交付清单、最终打包 |
| ppt-presenter | `.opencode/agents/ppt-presenter.md` | 答辩材料与演示准备 | 系统介绍 PPT、演示脚本、答辩预案 |

## 3. 协作规则

### 3.1 唯一调度者
- team-lead 是唯一用户接口与唯一调度者。用户每一次对话 = 给整个团队下达的指令。
- 调度只能由 team-lead 用 `task` 工具按 `subagent_type` 点名进行；其余 13 名 agent `permission.task: deny`，禁止互相调度。
- 若某 agent 直接收到用户指令，提示用户按 Tab 切换到 team-lead，或把指令完整转述给 team-lead，不得自行行动或调度他人。

### 3.2 升级路径
- 阻塞、冲突、跨角色问题 → 一律汇报 team-lead，由 team-lead 研判解决。
- 同一任务最多重试 2 次，仍失败则升级给用户决策。
- 发现他人交付物有问题 → 报告 team-lead，禁止直接修改他人产物。
- 范围变更（扩缩需求）→ 必须先请示用户，禁止静默变更。

### 3.3 并行原则
- 无依赖任务并行调度；有依赖任务串行调度；大任务拆成小任务。
- 每份任务书包含六要素：【任务】【背景】【输入】【要求】【产出】【禁区】。

### 3.4 单一写者规则
| 文件 | 唯一写者 | 其他角色 |
|------|---------|---------|
| `TEAM_BOARD.md` | team-lead | 只读，通过汇报请求更新 |
| `PROJECT_STATE.md` | team-lead | 只读 |
| `REQUIREMENTS.md` | requirements-analyst | 只读 |
| `DECISIONS.md` | 任何 agent（只追加） | 不改他人条目 |
| `agents/<自己>/STATUS.md`、`JOURNAL.md` | 该 agent 自己 | 他人只读 |
| 交付物源码/文档 | 对应负责人 | 他人不改，发现问题报告 team-lead |
| `tests/unit/`（单元测试） | core-dev / runtime-dev | 他人不改 |
| `tests/*.lfz`、`tests/REPORT.md`、`tests/coverage-matrix.md`（黑盒测试集） | test-engineer | 他人不改 |
