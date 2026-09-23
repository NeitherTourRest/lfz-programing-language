# P3 核心实现 — 任务书与分工（待用户批准）
> 作者: team-lead ｜ 日期: 2026-09-23 ｜ 状态: **待批准**
> 依据: `docs/spec/{syntax,semantics,interface-contract}.md`（**FROZEN**）、`DECISIONS.md` D-007（v1 范围）/ D-011（Rust）/ D-016（spec 冻结）
> 映射: 评分项 1「解释器实现」（20 分）；支撑评分项 2/3/5

---

## 0. 目标与验收（Definition of Done）

**目标**：交付可运行的 LFZ v1 解释器库 + 最小入口，覆盖 v1 全特性（核心 + 5 特色 + `float`），严格按冻结契约实现。

**验收命令（全部可复现）**
| # | 命令 | 期望 |
|---|---|---|
| 1 | `cargo build` | 成功、无 warning（deny 级） |
| 2 | `cargo test` | 全绿（单测 + 集成测试） |
| 3 | `cargo run -- run examples/hello.lfz` | 正确输出（含 `#42` 首行） |
| 4 | 缺 `#42` 的 `.lfz` | 退出码 **2**，输出 `CosmosAnswerError: 你忘记了宇宙的答案`（无源码行/插入符） |
| 5 | 语法/名字/类型错误 | 退出码 **2**，`类名: 中文消息` + `Traceback`（Python 风格，含 `line/col`） |
| 6 | `--json`（P4 扩展） | 字段见 `semantics.md` §8.3 示例 4 |

---

## 1. 工程结构（Cargo，仅 std，无第三方依赖）

```
Cargo.toml            # [lib] lfz  +  [[bin]] lfz ；edition 2021；dependencies = {}（std only）
src/
  lib.rs              # 模块声明 + 对外 API（run_file / run_source）
  span.rs             # Span{line,col}          ← §10.2（全项目唯一定义）
  error.rs            # LfzError 12 变体 + R<T>  ← §10.4 / §10.8
  loader.rs           # load_file/load_source/ext/path §10.1  ← core-dev
  lexer.rs            # 模式栈 CODE/STR/INTERP  §2 / §10.6   ← core-dev
  ast.rs              # 全节点带 Span            §10.2        ← core-dev
  parser.rs           # 换行模式栈 SIG/IGN、NO_BRACE_LITERAL、`;;`→Dump、管道→Call  §3 ← core-dev
  value.rs            # enum Value + Rc<RefCell<…>>  A1 引用语义        ← runtime-dev
  env.rs              # 作用域/cell 捕获 A2 + ScopeDebug 链 §10.5        ← runtime-dev
  evaluator.rs        # 求值语义 §4 + §4.5 确定性八项 + Dump(`;;`)       ← runtime-dev
  builtins.rs         # §10.7 内置表（data-last，~40 个）                ← runtime-dev
  cli.rs / main.rs    # 最小 CLI：`lfz run <file>` + 错误格式化 + 退出码 0/1/2  ← tooling-dev
examples/hello.lfz    # 冒烟样例（含 `#42`）
tests/
  *.rs                # Rust 集成测试（cargo 自动发现）        ← core-dev/runtime-dev
  lfz/**.lfz          # P5 黑盒测试集（cargo 忽略，无 main.rs） ← test-engineer（P5）
  fixtures/           # 负例夹具（T-R2）                       ← test-engineer（P5）
```

> **目录冲突预防**：Rust 集成测试用 `tests/*.rs`；LFZ 黑盒测试放 `tests/lfz/**.lfz`（cargo 不会扫描该子目录，因其无 `main.rs`）；负例夹具放 `tests/fixtures/`。**三方互不覆盖。**

---

## 2. 分工与边界

| 角色 | 拥有文件（唯一写者） | 不碰 |
|---|---|---|
| **core-dev** | `Cargo.toml`、`src/{lib,span,error,loader,lexer,ast,parser}.rs` | `value/env/evaluator/builtins`、`cli/main` |
| **runtime-dev** | `src/{value,env,evaluator,builtins}.rs` | `span/error/loader/lexer/ast/parser`（**只读消费**） |
| **tooling-dev** | `src/{cli,main}.rs`、`examples/`、打包脚本 | 解释器内核模块 |

- `span.rs` / `error.rs` 由 **core-dev 一次性写全**（契约 §10.2/§10.4 已给完整类型），runtime-dev **只构造变体、不改定义** → 避免双写者冲突。
- 单元测试写在**各自模块内** `#[cfg(test)]`（对应宪法 `tests/unit/` 的角色）；集成测试 `tests/*.rs` 由两位按域分摊。

---

## 3. 阶段分解（每阶段：实现 → 检查 → 提交）

> **用户批准的推进协议**：P3 拆为下列**细分子阶段**；**每完成一个子阶段** → ① team-lead 按验收要点**独立检查证据** → ② 派 release-manager **原子提交一次**（英文 conventional 信息）。
> **求值器路线（用户已定）**：**P3 采用树遍历求值器**，接口保留字节码 VM 形态；**P6 拿到基准数据后再决定是否上 VM**。

| 子阶段 | 内容 | 负责 | 依赖 | 验收（检查）要点 | 提交信息 |
|---|---|---|---|---|---|
| **P3.0** | Cargo 骨架：`Cargo.toml`（lib + bin、edition 2021、std only）+ `lib.rs` + 空模块 | core-dev | — | `cargo build` 成功；`cargo test` 可跑 | `chore(p3): cargo skeleton` |
| **P3.1** | 基座类型：`span.rs` + `error.rs`（12 变体 + `class_name`/`message`/`span` + `R<T>`） | core-dev | P3.0 | 单测逐类断言 12 个 `class_name()` 与中文消息 | `feat(p3): span and error model` |
| **P3.2** | `loader.rs`（§10.1：UTF-8 / BOM / 行终止符 / `ext` / `#42` / `line_base`） | core-dev | P3.1 | 单测：`ext` 矩阵、BOM、CRLF·LF·CR、缺 `#42` → `CosmosAnswer`(line1,col1)、`line_base` | `feat(p3): loader` |
| **P3.3** | `lexer.rs`（§2：模式栈 CODE·STR·INTERP、最大匹配、CODE 中 `#` → SyntaxError、INT 承载原文） | core-dev | P3.2 | 单测：模式栈、插值、`;;`、i64::MIN 字面量 | `feat(p3): lexer` |
| **P3.4** | `ast.rs` + `parser.rs` **核心**（语句 / 表达式 / 块 / 换行模式 SIG·IGN / 优先级；节点带 `Span`） | core-dev | P3.3 | 单测：块、优先级、换行规则 | `feat(p3): ast and parser core` |
| **P3.5** | `parser.rs` **特色**（管道脱糖为 `Call`、`;;` → `Dump`、富插值、NO_BRACE_LITERAL、`i64::MIN` 规则） | core-dev | P3.4 | 单测：5 特色各 ≥1 例 | `feat(p3): parser v1 features` |
| **P3.6** | `value.rs` + `env.rs`（`enum Value` + `Rc<RefCell>`、A2 cell 捕获、`ScopeDebug` 链） | runtime-dev | P3.1 | 单测：引用语义、闭包 cell、`ScopeDebug` 遮蔽 | `feat(p3): value and env` |
| **P3.7** | `evaluator.rs` **核心**（表达式 / 语句 / 控制流 / 函数与闭包 / 管道调用） | runtime-dev | P3.6 | 单测：函数、闭包、递归、控制流 | `feat(p3): evaluator core` |
| **P3.8** | `evaluator.rs` **语义定稿**（§4.5 确定性八项、A4/A5/A6、`RecursionError`、`;;` Dump） | runtime-dev | P3.7 | 单测：确定性八项各 1 例、环安全、深递归 | `feat(p3): v1 semantics` |
| **P3.9** | `builtins.rs`（§10.7 全表，data-last，约 40 个） | runtime-dev | P3.6 | 单测：每个内置 ≥1 例（含负例） | `feat(p3): builtins` |
| **P3.10** | 最小 CLI + 端到端：`cli.rs` / `main.rs` + `examples/hello.lfz` | tooling-dev | P3.5, P3.9 | 验收命令 1–5 全通过 | `feat(p3): minimal cli` |
| **P3.11** | 独立验收（对照验收命令 1–5 + 契约 §8.1/§10） | verifier | P3.10 | 出 `docs/reports/P3-verification.md`；无 🔴 缺陷 | （里程碑）打 `v0.2.0` |

**并行调度**：`P3.0 → P3.1` 串行（**门禁**，先定死 `Span` 与 `LzError`）；此后 **core-dev 链（P3.2→P3.3→P3.4→P3.5）** 与 **runtime-dev 链（P3.6→P3.7→P3.8→P3.9）** 可**并行**推进；P3.10 需两条链完成；P3.11 收口。
**每阶段收尾动作（固定）**：team-lead 核验证据 → release-manager 原子提交 → （里程碑处）打附注标签 + 更新 README + push。

---

## 4. 风险与对策

| 风险 | 对策 |
|---|---|
| Rust 借用检查器返工（`Rc<RefCell>` 值模型） | 契约先行 + P3.0 先定类型 + 小步提交 + 每步 `cargo test`；**禁止 `unsafe`**（除 std 必需） |
| `error.rs` 双写冲突 | 由 core-dev 一次写全，runtime-dev 只读消费（见 §2） |
| `tests/` 目录三方争用 | 见 §1 冲突预防（`.rs` / `lfz/` / `fixtures/` 三层隔离） |
| 语义细节漂移（A1 引用 / A5 数据面 / A6 环安全） | 实现前必读 `semantics.md` §4.5；每个 A 决策配 1 条针对性单测 |
| 性能（评分项 3 在 P6 才测） | P3 只求**正确性**；但值模型与槽位解析按 `DRAFT-runtime-arch.md` 的字节码 VM 目标设计（允许先树遍历，接口不变） |

---

## 5. 不在 P3 范围（防范围蔓延）

- ❌ REPL、`--json`、`lfz test` runner、打包 → **P4**（tooling-dev）
- ❌ `tests/*.lfz` 黑盒测试集与覆盖矩阵 → **P5**（test-engineer）
- ❌ 性能基准与报告 → **P6**（perf-engineer）
- ❌ 文档/指南/skill → **P7**；应用 → **P8**
- ❌ **任何语法/语义/契约改动**：如实现中发现契约缺口 → **停工，报 team-lead → architect 走 ADR**，不得就地改设计。

---

## 6. 批准后立即执行的调度

1. 派 **core-dev**（P3.0，串行门禁）→ 验收 `cargo build` + `cargo test`。
2. 门禁通过 → **同一轮并行**派 **core-dev（P3.1 前端）** ‖ **runtime-dev（P3.1 后端）**。
3. 二者完成 → 派 **tooling-dev**（P3.2 最小 CLI + 端到端）。
4. 完成后 → 派 **verifier**（P3.3 独立验收，对照验收命令 1–5）。
5. 每步由 **release-manager** 原子提交 + 里程碑附注标签（建议 `v0.2.0`）+ README 更新 + push。
