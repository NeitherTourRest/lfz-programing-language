# LFZ 项目 · opencode → DSH 迁移手册

> 生成：2026-09-24 by team-lead ｜ 基线：`main` = `27d3d48`，标签 `v0.1.0` / `v0.2.0`
> 目标运行环境：**DeepSeek Harness (DSH)** — `deepseek-ai/deepseek-harness`（官方，MIT，TypeScript，Cordis「万物皆插件」）
> 用途：照本文件即可在 DSH 下**重建同一个 14 人智能体团队并接着干**。

---

## 0. 30 秒 TL;DR

| 事项 | 结论 |
|---|---|
| 代码/资料要不要搬家？ | **不用**。仓库是本项目的唯一事实源；迁移的是**运行它的 harness**，不是代码。 |
| 什么能原样复用？ | ✅ **`AGENTS.md`（团队宪法）**、✅ **`SKILL.md` 技能**、✅ **全部团队记忆（`.opencode/team/**.md`）**、✅ `docs/spec/`、`src/`、`tests/`、`benchmarks/` |
| 什么必须重写？ | ⚠️ **14 个 agent 定义**（opencode 是 markdown frontmatter，DSH 是 Cordis 插件/agent preset）、**自定义命令**（DSH 用 `ctx.commands.register()`）、**权限模型**（DSH = sandbox 模式 × 审批策略，非 per-glob）、**插件钩子**（DSH 用 `tools/pre-execute`） |
| 最短路径 | ① clone 仓库（含 AGENTS.md）→ ② 建 `%USERPROFILE%\.dsh\cordis.yml`（装 agent-loop/subagent/mcp/skill 等插件）→ ③ 写一个 30 行的 DSH 插件把 14 个 `.opencode/agents/*.md` 注册成 agent → ④ `npx @deepseek-ai/dsh web` |

---

## 1. 项目是什么 / 在哪 / 怎么拿到

- **目标**：设计并实现 **LFZ** 解释型脚本语言（Rust 实现）+ 黑盒测试 + 性能对比 + 人/AI 开发指南 + Agent 应用 + Git 历史 + 答辩 PPT。
- **本地路径**：`D:\XUE\2026fall\Program Design\lfz-programing language design`
- **远程仓库**：https://github.com/NeitherTourRest/lfz-programing-language （**Public**，默认分支 `main`，协议 **MIT**）
  - ⚠️ 认证账号 login = `NeitherTourRest`（display name 为 `MakeChase`，二者非同一账号）。
- **取代码**：`git clone https://github.com/NeitherTourRest/lfz-programing-language.git`
- **Windows 环境实测**：git 2.53 / node 24.14 / Python 3.13.9 / rustc+cargo 1.98.1(`stable-x86_64-pc-windows-msvc`) / uv 0.11 / winget 可用。
  - 坑：`cargo` 常不在 PATH → 先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。

---

## 2. 当前进度快照（截至 `27d3d48`）

### 2.1 里程碑

| 阶段 | 内容 | 状态 | 证据 |
|---|---|---|---|
| P0 | 团队就绪（14 agent 脚手架 + 验证） | ✅ | `scripts/verify_team.py` 15/15 |
| **P0.5** | 版本基线 | ✅ | `git init -b main`；初始提交 `e060b21`；标签 **`v0.1.0`**；GitHub Public 已推送 |
| **P2** | 语言设计冻结 | ✅ | `docs/spec/{syntax,semantics,interface-contract}.md`（910/382/298 行）；ADR **D-016「spec v1 冻结」** |
| **P3** | **核心实现（20 个子阶段）** | ✅ | 见 §2.2；标签 **`v0.2.0`** |
| P1 | 需求矩阵与验收标准 | 🟡 **部分** | `.opencode/team/REQUIREMENTS.md` 已存在，**但尚未按冻结的 spec v1 做同步**（看板仍列待办） |
| P4 | 工具链（REPL / `--json` / `lfz test` runner / 打包） | ⬜ 待办 | 最小 CLI 已在 P3.10 落地（`lfz run <file>`） |
| P5 | 黑盒测试集（**评分项 2 = 20 分**） | ⬜ 待办 | 已有 3 组临时夹具（`docs/reports/fixtures-p3*`，共 25 个 `.lfz`）可作起点 |
| P6 | 性能 LFZ vs Python（**评分项 3 = 10 分**） | ⬜ 待办 | ⚠️ 见 §9 的 resolver 性能债 |
| P7 | 文档（人类手册 + AI skill，**评分项 4 = 20 分**） | ⬜ 待办 | 已冻结的 `docs/spec/` 可直接作为权威来源 |
| P8 | 应用 ≥200 行 LFZ（**评分项 5 = 30 分，权重最高**） | ⬜ 待办 | 选题已定：**排序算法可视化** |
| P9 | 独立验收 | ⬜ 待办 | P3.11 的三轮验收流程可直接复用为模板 |
| P10 | 发布 + 答辩（Git 历史 / 交付清单 / PPT） | ⬜ 待办 | — |

### 2.2 P3 实现明细（已完成）

模块与规模（`src/`，Rust，仅 std，零第三方依赖）：

| 模块 | 职责 |
|---|---|
| `loader.rs` | 读文件 → UTF-8 校验 → 跳 BOM → 归一化行终止符 → `ext(path)` 按扩展名分流 → 仅 `.lfz` 校验/消费 `#42` 前导；`Loaded{text, line_base}` |
| `lexer.rs` | 模式栈 `CODE/STR/INTERP`、最大匹配、数值**原文保留**、`#` 在 CODE → SyntaxError |
| `ast.rs` | 全节点携带 `Span{line,col}`；`Dump{scope: ScopeId}` |
| `parser.rs` | 语句/表达式/优先级/换行模式栈 `SIG/IGN`/`NO_BRACE_LITERAL`/控制流/`fn`/`struct`/**管道 `\|>` 脱糖**/`;;`→`Dump`/**富插值（`format_spec`）**/`i64::MIN` 规则 |
| `value.rs` | `Value` 八类 + `StructDef` 模板；**A1 引用语义**（`Rc<RefCell<…>>`）；显示与 `==` **环安全**；`as_f64` **唯一加宽入口** |
| `env.rs` | 作用域链 + **A2 cell 捕获** + per-scope `ScopeDebug` / `NameInterner` |
| `evaluator.rs` | 树遍历求值；§4.5 确定性八项；A4(`check` 非致命/`assert` 致命)、A5 数据面、A6 环安全 `==`；`RecursionError`；traceback（含**截断折叠**） |
| `builtins.rs` | **§10.7 全表 54/54**（data-last，容器更新一律返回新值） |
| `cli.rs` / `main.rs` | `lfz run <file>`；Python 风格错误输出（类名+中文消息+`line N, col M`+源行+插入符；运行期带 `Traceback`）；退出码 0/1/2 |

**质量基线（可复现）**：
```powershell
cargo build      # 0 warning
cargo test       # 361 passed / 0 failed / 0 ignored （+ bin 9 + cli 7 = 377）
cargo run -- run examples/hello.lfz   # → Hello, LFZ!   exit 0
```
**独立验收**：`docs/reports/P3-verification.md` — 三轮：rev.1 **FAIL(3×🔴)** → rev.2 CONCERNS → **rev.3 PASS（10/10 缺陷闭环、0 回归）**。
缺陷清算：3×🔴（多行块 / 插值 `format_spec` / `if` 表达式）+ 4×🟡（traceback 位置 / 管道消息 / `let` 重绑定 / 巨量 traceback）+ 2×🟢（`.self` / 并入 🟡）+ **1 规范侧**（§9.4 样例自相矛盾）。

### 2.3 提交历史骨架（`git log --oneline` 摘）
```
27d3d48 chore(team): off-peak gate per official DeepSeek pricing + unattended runner + autostart
931e2b2 docs(team): mark P3 complete and update README for v0.2.0     ← v0.2.0
290a518 docs(reports): P3.11 final verification (rev.3 PASS, 10/10 closed)
c6638cc fix(p3): enforce let immutability (ImmutableRebind)
833901d fix(p3): statement-initial brace is an anonymous struct literal (A9)
1e8fd5f fix(p3): blocking defects + runtime error spans
5cdcc1b docs(spec): pin unterminated block comment as SyntaxError (ADR)
711d6b9 docs(spec): close 6 P3.9a contract gaps (ADR + v1 pins)
4c68d18 feat(p3): lexer CODE-mode core tokens
e060b21 chore: initial commit — LFZ scaffold, team constitution, frozen v1 language spec   ← v0.1.0
```

---

## 3. 需求与验收

- **原始要求**：仓库根 `task-info.md`（课程作业原文）。
- **需求矩阵 / 验收标准**：`.opencode/team/REQUIREMENTS.md`（唯一写者 requirements-analyst）。
- **不可动摇的硬约束（已冻结，ADR 记录在 `.opencode/team/DECISIONS.md`）**：
  1. 解释器实现语言 = **Rust**，**仅 std，零第三方依赖**；
  2. `.lfz` 文件**首行必须恰好 `#42` + 换行**，否则 `CosmosAnswerError: 你忘记了宇宙的答案`；仅按扩展名（ASCII 大小写不敏感）触发；
  3. 错误模型 = **Python 风格**：`类名: 中文消息` + `line N, col M` + 源行 + 插入符；运行期错误带 `Traceback`；**12 个具体错误类 + 基类 `LfzError`**；**禁止 `E-xxx` 码**；
  4. 退出码：`0` 成功 / `1` 测试失败 / **所有错误 `2`**；
  5. 内置函数表 `§10.7` **54 个**，`data-last` 约定；
  6. v1 特性范围 = 核心 + **5 特色**（管道、合一 struct、富字符串插值、结构化错误 + `assert`/`check`、确定性语义）+ `float`；
  7. 协议 **MIT**；Git 纪律由 release-manager 独占。

### 8 项提交物 × 评分矩阵（合计 100）

| # | 提交物 | 分值 | 负责 | 位置 | 状态 |
|---|---|---|---|---|---|
| 1 | LFZ 语法规则文档 | 20（评分项 4 一半） | language-architect | `docs/spec/` | ✅ 已冻结 |
| 2 | LFZ 解释器源程序 | **20**（评分项 1） | core-dev + runtime-dev | `src/` | ✅ 端到端可用 |
| 3 | LFZ 完整黑盒测试集 | **20**（评分项 2） | test-engineer | `tests/` | ⬜ P5 |
| 4 | LFZ 性能测试报告 | **10**（评分项 3） | perf-engineer | `benchmarks/` + `docs/reports/performance.md` | ⬜ P6 |
| 5 | 开发指南（人 + AI） | 20（评分项 4 一半） | docs-writer + ai-dx-engineer | `docs/guide/` + `<skills>/lfz-programming/` | ⬜ P7 |
| 6 | 应用源码 + 开发记录 | **30**（评分项 5） | app-dev | `app/` + `app/DEV_RECORD.md` | ⬜ P8（选题：排序算法可视化）|
| 7 | Git 历史记录 | 交付完整性 | release-manager | `.git/` + 远程 | ✅ 基线 + `v0.1.0`/`v0.2.0` |
| 8 | 系统介绍 PPT | 答辩载体 | ppt-presenter | `docs/slides/` | ⬜ P10 |

---

## 4. 团队：14 个 agent 全量定义与协作协议

### 4.1 宪法（`AGENTS.md`，96 行，仓库根）

**迁移利好：DSH 原生读取项目 `AGENTS.md`（`dsh-agent-instructions` 默认候选 `['AGENTS.md','CLAUDE.md']`，以 `.git` 为项目根标记，逐级从根到 cwd 加载，还支持 `AGENTS.local.md`）** → 本宪法**几乎零改动即可复用**。
唯一需要留意：DSH 的 `maxBytes` 是**必填**（示例 `65536`），超限会被截断。

宪法要点（原文见 `AGENTS.md`）：
1. 团队使命与任务入口；2. 组织架构（team-lead 唯一用户接口 + 唯一调度者；其余 13 人 `permission.task: deny`，禁止互相调度）；3. **启动协议**（读 `PROJECT_STATE.md` → `TEAM_BOARD.md` → 自己 `STATUS.md` → 按需读他人 → 冲突以看板为准）；4. **收工协议**（覆盖 `STATUS.md` + 追加 `JOURNAL.md` + 结构化汇报 + 必要时追加 ADR）；5. **单一写者规则**；6. **单一事实源**；7. 工作约定（文档中文、技术词英文、证据文化、项目外只读、git 归 release-manager）；8. 辅助 agent（explore/librarian）跳过团队协议；9. 指令路由；10. 质量红线。

### 4.2 14 角色表

| name | 中文角色 | 职责边界 | 定义文件 |
|---|---|---|---|
| `team-lead` | 主调度智能体 | 唯一用户接口 + 唯一调度者；**唯一拥有 `task`/`question`/`todowrite` 权限**；`PROJECT_STATE.md` / `TEAM_BOARD.md` 唯一写者 | `.opencode/agents/team-lead.md` |
| `requirements-analyst` | 需求分析师 | 需求 → 可验收条目；`REQUIREMENTS.md` **唯一写者** | `.opencode/agents/requirements-analyst.md` |
| `language-architect` | 语言架构师 | 语法/语义**唯一权威**；`docs/spec/` + 设计 ADR | `.opencode/agents/language-architect.md` |
| `core-dev` | 核心开发 | 前端：`loader`/`lexer`/`ast`/`parser`（严格按 interface-contract） | `.opencode/agents/core-dev.md` |
| `runtime-dev` | 运行时开发 | 后端：`value`/`env`/`evaluator`/`builtins` | `.opencode/agents/runtime-dev.md` |
| `tooling-dev` | 工具链开发 | CLI / REPL / 一键测试 runner / 打包 | `.opencode/agents/tooling-dev.md` |
| `test-engineer` | 测试工程师 | `tests/` 黑盒测试集 + 覆盖矩阵（**评分项 2**） | `.opencode/agents/test-engineer.md` |
| `perf-engineer` | 性能工程师 | `benchmarks/` LFZ vs Python + 报告（**评分项 3**） | `.opencode/agents/perf-engineer.md` |
| `verifier` | 验证工程师 | **只验证不修复**；缺陷单（最小复现）+ 回归记录 | `.opencode/agents/verifier.md` |
| `docs-writer` | 文档工程师 | 人类向 README / 手册 / 教程 | `.opencode/agents/docs-writer.md` |
| `ai-dx-engineer` | AI 赋能工程师 | AI 向指南 + `lfz-programming` SKILL 并**实测验证** | `.opencode/agents/ai-dx-engineer.md` |
| `app-dev` | 应用开发工程师 | ≥200 行 LFZ 应用 + 开发记录（**评分项 5，30 分**） | `.opencode/agents/app-dev.md` |
| `release-manager` | 发布经理 | git 纪律、版本标签、交付清单、**README 唯一写者** | `.opencode/agents/release-manager.md` |
| `ppt-presenter` | 演示工程师 | 答辩 PPT + 演示脚本 + 问答预案 | `.opencode/agents/ppt-presenter.md` |

> 每个文件的 frontmatter 形如：`description` / `mode`(primary|subagent) / `model` / `permission`（如 `task: deny`）；**body 即该 agent 的 system prompt**。

### 4.3 调度规则（team-lead 专用）

- **决策树**：需求/验收 → requirements-analyst；语法语义 → language-architect；前端 → core-dev；后端 → runtime-dev；CLI/runner → tooling-dev；黑盒测试 → test-engineer；性能 → perf-engineer；独立验收 → verifier；文档 → docs-writer / ai-dx-engineer；应用 → app-dev；发布 → release-manager；PPT → ppt-presenter；外部调研 → explore / librarian。
- **并行**：无依赖同轮派发（如 core-dev ‖ runtime-dev；docs-writer ‖ ai-dx-engineer）。**串行**：有依赖必须等前置通过验收。
- **重试纪律**：同一任务**最多重试 2 次**；第 2 次必须换策略（改派 / 拆小 / 会诊）；仍失败 → 标记阻塞并升级用户。
- **验收纪律**：无证据不算完成（必须附文件路径 + 命令与结果）；关键交付物必须由 **verifier 独立复验**。

### 4.4 单一写者（务必保留）

| 文件 | 唯一写者 |
|---|---|
| `PROJECT_STATE.md` / `TEAM_BOARD.md` | team-lead |
| `REQUIREMENTS.md` | requirements-analyst |
| `DECISIONS.md` | 任何 agent（**只追加**） |
| `agents/<自己>/STATUS.md`、`JOURNAL.md` | 该 agent |
| `docs/spec/` | language-architect |
| `README.md` | release-manager |
| 交付物源码 | 对应负责人 |

### 4.5 团队记忆清单（DSH 下继续沿用）

| 文件 | 作用 |
|---|---|
| `.opencode/team/PROJECT_STATE.md` | 全局状态（阶段/里程碑/交付物对照/风险） |
| `.opencode/team/TEAM_BOARD.md` | 看板（进行中/待办/阻塞/已完成 + **P3 子阶段台账含提交哈希**） |
| `.opencode/team/REQUIREMENTS.md` | 需求矩阵与验收标准 |
| `.opencode/team/DECISIONS.md` | **ADR 全集（D-001…D-0xx，只追加）**——含「先问用户」D-010、实现语言 D-011、spec 冻结 D-016、各轮裁决 |
| `.opencode/team/TEAM_SPEC.md` / `ORG.md` / `KICKOFF.md` / `BRAINSTORM.md` | 团队规格 / 组织 / 开工调研 / 头脑风暴 |
| `.opencode/team/DRAFT-LFZ-v0.{2,3,4,5}.md`、`DRAFT-runtime-arch.md` | 设计历史与 Rust 架构草案 |
| `.opencode/team/PLAN-P3.md` | P3 子阶段任务书与分工 |
| `.opencode/team/agents/<name>/{STATUS,JOURNAL}.md` | 14 人各自状态与日志 |

---

## 5. 文件地图

```
<repo>/
├── AGENTS.md                     ← 团队宪法（DSH 原生读取）
├── MIGRATION-TO-DSH.md           ← 本文件
├── README.md                     ← release-manager 维护（MIT + 状态 + 验收证据）
├── LICENSE                       ← MIT
├── Cargo.toml / Cargo.lock / src/  ← 交付物 2（解释器）
├── examples/hello.lfz            ← 端到端样例
├── tests/                        ← Rust 集成测试（*.rs）；P5 将新增 tests/lfz/**.lfz
├── docs/
│   ├── spec/{syntax,semantics,interface-contract}.md   ← 交付物 1（已冻结）
│   ├── reports/P3-verification.md + fixtures-p3*/      ← 验收证据与夹具
│   └── (guide/ reports/ slides/ 由 P7/P10 补齐)
├── benchmarks/                   ← P6
├── app/                          ← P8
├── scripts/
│   ├── verify_team.py            ← 团队脚手架自检（15 项）
│   ├── offpeak.ps1 / offpeak-runner.ps1 / offpeak-start.cmd / offpeak-task.cmd  ← 错峰闸门与自动开工
└── .opencode/
    ├── opencode.json             ← 【需迁移】harness 配置（default_agent 等）
    ├── agents/*.md (14)          ← 【需迁移】agent 定义
    ├── command/standup.md        ← 【需迁移】自定义命令
    ├── plugin/offpeak.ts         ← 【需迁移】插件钩子（高峰拦截派发）
    ├── skills/                   ← 【需迁移】技能（`SKILL.md`，DSH 兼容）
    └── team/**                   ← ✅ 纯 markdown 团队记忆，**无需改动**
```

---

## 6. opencode → DSH 逐项映射

| # | 能力 | opencode（现状） | DSH（目标） | 依据 |
|---|---|---|---|---|
| 1 | 项目指令 | `AGENTS.md`（自动加载） | **`AGENTS.md` 同名同用**；`dsh-agent-instructions`，候选 `['AGENTS.md','CLAUDE.md']`，根标记 `.git`，支持 `AGENTS.local.md`；**`maxBytes` 必填** | 官方源码 `packages/context/agent-instructions` |
| 2 | 全局配置 | `~/.config/opencode/opencode.json`(JSONC) | `%USERPROFILE%\.dsh\`（`$DSH_HOME`）：`cordis.yml`(YAML 组合) + `settings.yaml`(热载) + `.credentials.yaml` + `profiles/<name>/` | `packages/util/home-paths`、`apps/cli/README.md` |
| 3 | 项目配置 | `.opencode/opencode.json` | 项目内 `cordis.yml` / profile（YAML） | 同上 |
| 4 | 默认 agent / 模型 | `default_agent: team-lead`；模型写在 agent frontmatter | `dsh-agent-default-model`(`provider`+`model`)；agent 预设注册表 `dsh-agent-preset-registry`(`config.default`) | `packages/core/agent-default-model`、`packages/preset/agent-preset` |
| 5 | **14 个 agent** | `.opencode/agents/<name>.md`（frontmatter+body=prompt） | ⚠️ **无 markdown agent**：`dsh-agent-loop` 的 `agents[]`（`id/provider/model/cwd/sessionId`）+ **personas 需 `ctx.agents.create({setup})` 或 agent preset** | `packages/core/agent-loop`、`packages/subagent/subagent` |
| 6 | 子智能体委派 | `task(subagent_type=…)` | `dsh-subagent` + `dsh-tool-subagent`（工具名 `subagent`，另有 `subagent_fork`） | `packages/subagent/*` |
| 7 | 技能 | `.opencode/skills/<n>/SKILL.md` | ✅ **`SKILL.md` 兼容**（`name`+`description` frontmatter）；由 `dsh-skill-filesystem` 发现 + `dsh-tool-skill` 加载 | `packages/skill/*` |
| 8 | 自定义命令 | `.opencode/command/*.md` | ⚠️ **代码注册**：`ctx.commands.register({name, description, input, handler})` | `packages/interaction/commands` |
| 9 | MCP | `.opencode/opencode.json` 的 `mcp` 对象 | `dsh-mcp-client`，每个 server 一行；`transport: stdio|streamable-http`；工具名 `mcp__<server>__<tool>` | `packages/mcp/mcp-client` |
| 10 | 插件钩子 | `.opencode/plugin/*.ts`，`tool.execute.before` | Cordis 插件（`name`/`inject`/`apply(ctx)`）；**`tools/pre-execute` 瀑布**可「检查并拒绝」工具调用 | `docs/subsystems/tools.md`、`docs/tool-execution-pipeline.md` |
| 11 | 权限 | frontmatter `permission: {task: deny, edit: …}`（per-glob allow/ask/deny） | ⚠️ **模型不同**：`dsh-user-approval`(`policy: ask|never`) × `dsh-permission-presets`(sandbox 模式 + 审批)；**per-glob 白名单未证实** → 自定义用 `tools/pre-execute` | `packages/interaction/{user-approval,permission-presets}` |
| 12 | 无头/一键运行 | `opencode run --agent X --dir Y --auto "<prompt>"` | `dsh --profile headless "<job>"`（`--json` / `--session-id`） | `packages/bundle/headless` |
| 13 | 会话续接 | `-c/--continue`、`--session <id>` | `--session-id <id>`；或 `agents[].resumeSessionId` / `ctx.agents.resume()` | 同上 |
| 14 | 团队记忆 | `.opencode/team/**.md` | ✅ **原样保留**（纯 markdown，任何人/任何 harness 都能读） | — |

---

## 7. 迁移步骤（可勾选）

### 步骤 1 — 准备仓库（不变）
```powershell
git clone https://github.com/NeitherTourRest/lfz-programing-language.git
cd lfz-programing-language
$env:Path += ";$env:USERPROFILE\.cargo\bin"
cargo test           # 期望 361 passed / 0 failed / 0 ignored
```

### 步骤 2 — 安装 DSH
```powershell
# Node.js 已具备（24.x）
npx @deepseek-ai/dsh web     # → http://127.0.0.1:3080
```

### 步骤 3 — 写 `%USERPROFILE%\.dsh\cordis.yml`（骨架）
```yaml
# 说明：DSH 是 developer preview，插件包名以你安装到的版本为准；先用 dsh 的 skill
#      `cordis-plugin-development` 或 docs 核对包名后再落地。
- id: settings
  name: '@deepseek-ai/dsh-settings-file'
- id: agent-instructions                 # ← 读项目 AGENTS.md（宪法直接复用）
  name: '@deepseek-ai/dsh-agent-instructions'
  config: { maxBytes: 65536 }            # maxBytes 必填
- id: llm-deepseek
  name: '@deepseek-ai/dsh-llm-deepseek'
- id: agent-default-model
  name: '@deepseek-ai/dsh-agent-default-model'
  config: { provider: deepseek, model: deepseek-chat }
- id: agent-loop
  name: '@deepseek-ai/dsh-agent-loop'
  config:
    agents:
      - { id: main, provider: deepseek, model: deepseek-chat, cwd: . }
- id: subagent
  name: '@deepseek-ai/dsh-subagent'
- id: subagent-spawn
  name: '@deepseek-ai/dsh-subagent-spawn-in-process'
- id: tool-subagent
  name: '@deepseek-ai/dsh-tool-subagent'
  config: { provider: spawn, toolName: subagent }
- id: skill-fs
  name: '@deepseek-ai/dsh-skill-filesystem'
  config: { paths: ['.opencode/skills'] }   # 复用既有 SKILL.md
- id: tool-skill
  name: '@deepseek-ai/dsh-tool-skill'
- id: perm-presets
  name: '@deepseek-ai/dsh-permission-presets'
  config:
    presets:
      workspace-write:     { sandbox: workspace-write,   approval: ask  }
      danger-full-access:  { sandbox: danger-full-access, approval: never }
    defaultPreset: workspace-write
```
> 📌 **包名/字段以你的 DSH 版本为准**（developer preview，破坏性变更频繁）。落地前请用 DSH 自带的 `cordis-plugin-development` skill 或 `https://deepseek-harness.github.io/deepseek-harness/` 核对。

### 步骤 4 — 写「14 人团队」桥插件（关键一步）
新建 `%USERPROFILE%\.dsh\plugins\lfz-team.ts`，把 `.opencode/agents/*.md` **原样加载**成 DSH agent（**不复制内容**，避免双份事实源）：
```ts
// 伪代码骨架：读 frontmatter(name/description/model/permission) + body 作为 system prompt，
// 用 ctx.agents.create({ id, provider, model, setup }) 或 agent preset 注册 14 个 agent；
// team-lead 注册为「主 agent」，其余 13 个注册为可被 `subagent` 工具委派的 agent。
// 具体 API：见官方 docs「cordis-primer」与 packages/core/agent-loop/README.md。
```
要点：
- `mode: primary` 的（仅 `team-lead`）→ DSH 主 agent；`mode: subagent` 的 13 个 → 子 agent。
- 原 `permission: { task: deny }` 的语义（"其余 13 人不得调度"）在 DSH 里用 **不给他们挂 `subagent` 工具** 来等价实现。
- 各 agent 的 `STATUS.md` / `JOURNAL.md` 协议**不变**（纯文件读写）。

### 步骤 5 — 移植插件钩子（错峰闸门）
`.opencode/plugin/offpeak.ts`（opencode 版 `tool.execute.before`）→ DSH 版 `tools/pre-execute`：
```ts
import type { Context } from '@deepseek-ai/cordis'
export const name = 'lfz-offpeak'
export const inject = ['tools']
const PEAK = '01:00-04:00,06:00-10:00'           // UTC, Mon–Fri
export function apply(ctx: Context) {
  ctx.on('tools/pre-execute', async (exec, next) => {
    if (exec.name === 'subagent' && isPeakUtc()) {
      return { kind: 'deny', reason: '[offpeak] 现在是 DeepSeek 高峰，已阻止派发子智能体' }
    }
    return next()
  })
}
```
> 逻辑可整段复用 `scripts/offpeak.ps1`（高峰期判定已按官方口径实现并可配置）。

### 步骤 6 — 自定义命令
`/standup`（`.opencode/command/standup.md`）→ 用 `ctx.commands.register({ name: 'standup', handler })` 实现：读 `.opencode/team/PROJECT_STATE.md` + `TEAM_BOARD.md` + 14 份 `STATUS.md`，输出汇总报告。

### 步骤 7 — 无头自动开工（等价现在的错峰 runner）
```powershell
# 低谷窗口内循环跑；高峰退出/等待。替代 scripts/offpeak-runner.ps1 里的 `opencode run`
dsh --profile headless --session-id lfz-main "继续推进：读 .opencode/team/PROJECT_STATE.md 与 TEAM_BOARD.md，完成下一个子阶段（派发→独立检查→原子提交）。重大决策停下记录。"
```

### 步骤 8 — 保留 Git 纪律
**不变**：release-manager 独占 commit/tag/push；里程碑打附注标签；README 由其维护；`main` 分支、禁 force-push。

---

## 8. 迁移后自检（逐条可复现）

```powershell
# 1) 项目指令被加载：让 DSH 复述宪法要点（team-lead 唯一调度者 / 单一写者 / 收工协议）
# 2) 14 个 agent 可见：列出可用 agent，应含 team-lead + 13 个角色
# 3) 子智能体委派可用：派 verifier 跑一次只读验收
# 4) 技能可加载：列出 skills，应含 lfz-programming（P7 产出后）
# 5) 错峰钩子生效：高峰期尝试派发子智能体 → 应被 deny
# 6) 解释器仍可用（与 harness 无关，纯验证仓库完整）：
cargo build ; cargo test
cargo run -- run examples/hello.lfz          # → Hello, LFZ!
# 7) 错误模型自检：写一个不带 #42 的 .lfz → 退出码 2 + CosmosAnswerError
```

---

## 9. 已知坑与实战教训（**建议 DSH 下的 team-lead 先读这段**）

**技术债 / 风险**
1. **性能债（P6 必查）**：P3 的求值器用**树遍历 + 运行时查名**（无 resolver），这正是 `DRAFT-runtime-arch.md` 想消除的开销。P6 需评估是否补 **resolver / 槽位缓存 / 字节码 VM**。
2. **DSH 是 developer preview**：官方明示「WILL THERE BE COMPATIBILITY-BREAKING CHANGES」→ 插件包名与配置项会变，**迁移时以 `cordis-plugin-development` skill 核对**。
3. **DSH 没有 markdown agent 概念**：14 人的定义要么写成 preset，要么写插件动态注册。**建议动态注册**（读 `.opencode/agents/*.md`），保证单一事实源。
4. **权限模型不可移植**：opencode 的 per-glob allow/ask/deny → DSH 是 sandbox 模式 × 审批策略；如需细粒度，用 `tools/pre-execute`。

**流程教训（本项目用血换来的，务必在 DSH 下继续执行）**
5. **"绿了 ≠ 对"**：373 个单测全绿时，解释器**连多行函数都跑不了**。→ 关键交付物**必须由 verifier 独立跑真实程序**，不能只看测试。
6. **真假完成防线**：把「**必须贴出文件实际字节数**」写进任务书验收项 —— 本项目靠它连续抓出 **3 次"跑完不落盘"的假完成**。
7. **任务切小**：执行者在**大而开放**的任务上会"推演到超时/空跑"；切成 **≤1 个模块**的小批次后 6 批全部一次落地。
8. **失败即换策略**：同一任务**最多重试 2 次**，第 2 次必须改派/拆小；原 parser 负责人 2 次未交付后**改派**（`unspecified-high` 档）即成功。
9. **执行者全挂时 team-lead 亲自收尾**：曾发生「2 任务同时超时 + 子 agent 会话 `Insufficient Balance`」，工作区留下 2 warnings + 3 failed 的半成品 → 处置原则：**先恢复绿灯**（保留写对的部分、回退夹带的高风险改动），再谈新增。
10. **不篡改测试**：宁可**推迟一个 🟡 缺陷**（bug-07 就是如此），也不改既有测试来"变绿"。
11. **测试接缝**：parser 单测曾**手工构造 token 流、绕过 lexer** → `Colon`/`newline` 接缝无人校验。新增回归一律**先 `lexer::lex` 再 `parse`**。
12. **规范自相矛盾也算缺陷**：§9.4 样例自身用了被禁止的单个 `;`，使唯一引用语义样例跑不通 → 走「先 ADR 后改 spec」修正。
13. **标签只作"通过"的证据**：验收 FAIL 阶段**一律不打里程碑标签**。
14. **环境坑**：① `cargo` 常不在 PATH；② Windows PowerShell 5.1 把**无 BOM 的 `.ps1` 按 ANSI 解码** → 脚本含中文会解析崩（本项目因此把所有 `.ps1` 改为**纯 ASCII**）；③ `gh` **不读** git 的 `http.proxy`，需注入 `HTTPS_PROXY`；④ `schtasks /Create /SC ONLOGON` **需管理员**，免管理员替代 = Startup 文件夹自启。

---

## 10. 待确认 / 由迁移执行者补齐

1. DSH 当前版本的**插件包名与配置字段**（developer preview，可能与本文件 §7 骨架有出入）→ 用 `cordis-plugin-development` skill 核对。
2. **DSH 是否有 per-glob 权限白名单**（librarian 标记为 UNCONFIRMED）→ 若无，用 `tools/pre-execute` 自实现。
3. DSH 的**会话列表/管理**命令（仅确认了 `--session-id` 续接）。
4. 迁移后**重新核对 `AGENTS.md` 是否超出 `maxBytes`**（宪法 96 行，远小于 65536，风险低）。
5. `.opencode/` 目录名是否改名为 `.dsh/`：**团队记忆建议保留原地**（内容与 harness 无关）；仅 `agents/`、`command/`、`plugin/` 三个子目录需要按 DSH 方式重建。

---

## 11. 迁移检查清单（打印版）

- [ ] clone 仓库并 `cargo test` 绿
- [ ] 安装/启动 DSH（`npx @deepseek-ai/dsh web`）
- [ ] `%USERPROFILE%\.dsh\cordis.yml` 落地（核对包名）
- [ ] `agent-instructions` 插件生效（DSH 能读到 `AGENTS.md`）
- [ ] 写 `lfz-team` 插件：14 个 agent 从 `.opencode/agents/*.md` 动态注册
- [ ] 13 个非 lead agent **不挂** `subagent` 工具（等价 `permission.task: deny`）
- [ ] `skill-filesystem` 指向 `.opencode/skills`
- [ ] 移植 `offpeak` 钩子（`tools/pre-execute`）
- [ ] 移植 `/standup` 命令
- [ ] 无头自动开工改用 `dsh --profile headless`
- [ ] 跑 §8 自检 7 条
- [ ] 确认 Git 纪律不变（release-manager 独占提交）
- [ ] 从 `PROJECT_STATE.md` / `TEAM_BOARD.md` **接着 P4 干**（P1 需求同步仍待办）

---

*本文件由 team-lead 生成，随仓库入库。若 DSH 规范更新，请更新 §6/§7 并追加一条 ADR 到 `.opencode/team/DECISIONS.md`（只追加）。*
