# LFZ 需求矩阵（唯一需求事实源 · v1 对齐版）

> 最后更新: 2026-09-27 by requirements-analyst
> 版本: **v2（对齐 spec v1 冻结 D-016 + P3/P4 已实现事实）**
> 本文件是**唯一需求事实源**，唯一写者 requirements-analyst，他人只读。
> 来源: `task-info.md`（课程作业原始要求）+ 用户指令 + 已定案 ADR（见 `DECISIONS.md`）。

## 0. 使用说明（team-lead / verifier 先读）

- **文档定位**：P9 验收时 verifier 以本文件 §1/§6 为对表依据；每条需求都能落到「**一条可原样执行的命令 + 期望结果**」。
- **状态取值（§1 主矩阵）**：`待办 | 进行中 | 待验收 | 通过 | 失败 | 已撤销`（严格枚举）。
- **覆盖取值（§4/§5 追溯）**：`✅ 已满足 | 🟡 部分 | ⬜ 未做`。
- **验收标准纪律**：可观察、可命令复现、二值可判；**禁止**主观词（优秀/完善/健壮/友好/高效/优雅/基本可用）；性能类用具体数值阈值。
- **命令约定**：所有命令在**项目根**执行。`cargo run -q -- <args>` ≡ 构建后运行 CLI；已 `cargo build` 可直接用 `lfz <args>`。PowerShell 下退出码取 `$LASTEXITCODE`。
- **当前基线（2026-09-27 实测）**：HEAD `7527792`；`cargo build` **0 warning**；`cargo test` **431 passed / 0 failed / 0 ignored**（lib 361 + bin 42 + tests/cli.rs 16 + tests/test_runner.rs 12）；标签 `v0.1.0`、`v0.2.0`。

---

## 1. 需求矩阵（v1 现行）

> 列：R-ID | 需求 | 来源（task-info + ADR）| 验证方式（可命令复现）| 状态 | 交付物 | 评分项。
> **R-1xx** 语言与解释器 · **R-2xx** 自动测试 · **R-3xx** 性能 · **R-4xx** 文档 · **R-5xx** 应用 · **R-6xx** 环境与提交物。

### 1.1 语言与解释器（评分项 1 = 20 分）

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-101 | 支持 `int` / `float` / `string` / `bool` / `nil` 基本数据类型及其字面量与运算（`/` 恒返回 float；`div` 向下取整；`%` Python 取模） | task-info §1 + D-007/D-011/D-015 A3 | `cargo run -q -- run examples/hello.lfz` → stdout `Hello, LFZ!`、exit 0；`cargo test`（361 库单测含类型/运算用例）全绿 | 通过 | 交付物 1、2；`src/value.rs`,`src/evaluator.rs` | 1 |
| R-102 | 支持数组与 struct 类型（含**合一 struct**：struct 兼作字典/实例模板） | task-info §1 + D-007/D-011 | `cargo run -q -- run docs/reports/fixtures-p3/04_containers.lfz` → exit 0；`cargo test`（value/struct 用例） | 通过 | 交付物 1、2；`src/value.rs` | 1 |
| R-103 | 支持基本输入输出（`print` / `eprint` / `input`；`;;` 变量 dump 到 stdout） | task-info §1 + D-012 | `cargo run -q -- run examples/hello.lfz` → 输出 `Hello, LFZ!`；`cargo test`（`cli`/`test_runner` 集成含 print 流断言） | 通过 | 交付物 1、2；`src/builtins.rs` | 1 |
| R-104 | 支持分支循环控制（`if`/`else` 及 `if` 表达式、`while`、`for-in`） | task-info §1 | `cargo run -q -- run docs/reports/fixtures-p3/02_control.lfz` → exit 0；`cargo test`（parser/evaluator 控制流用例） | 通过 | 交付物 1、2；`src/parser.rs`,`src/evaluator.rs` | 1 |
| R-105 | 支持函数定义与调用（`fn` 声明 / 函数字面量 / 闭包（按 cell 引用捕获，A2）/ 递归） | task-info §1 + D-015 A2 | `cargo run -q -- run docs/reports/fixtures-p3/03_functions.lfz` → exit 0；`cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz` → 5 行 `1 2 3 / 99 99 / [3, 1, 2]  [1, 2, 3] / {me: <cycle>} / true`、exit 0 | 通过 | 交付物 1、2；`src/evaluator.rs`,`src/env.rs` | 1 |
| R-106 | **特色①** 管道 `\|>` + `_` 占位（解析期脱糖为 `Call`；`\|>` 优先级比 `+ - * / %` 松、比比较紧） | task-info §1 注意 + D-007/D-011/D-012 | `cargo run -q -- run docs/reports/fixtures-p3/05_pipe.lfz` → 9 行输出（`4 3 15 15 5` + `[0,2,4,6,8] [1,2,3] [2,4] ABC`）、exit 0 | 通过 | 交付物 1、2；`src/parser.rs` | 1（+4 语法说明） |
| R-107 | **特色②** 合一 struct（`s.k ≡ s["k"]`；方法字段不入数据面，A5） | task-info §1 注意 + D-007/D-011/D-015 A5 | `cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz` 第 4 行 `{me: <cycle>}`；`cargo test`（A5 方法字段用例） | 通过 | 交付物 1、2；`src/value.rs`,`src/builtins.rs` | 1（+4） |
| R-108 | **特色③** 富字符串插值 `${}`（`{` 在字符串内为字面字符；lexer `CODE/STR/INTERP` 模式栈；format_spec 支持） | task-info §1 注意 + D-007/D-012 | `cargo run -q -- run docs/reports/fixtures-p3/06_interp.lfz` → exit 0；`cargo test`（lexer/parser 插值用例） | 通过 | 交付物 1、2；`src/lexer.rs`,`src/parser.rs` | 1（+4） |
| R-109 | **特色④** 结构化错误 + `assert`/`check`/`fail`（`check` **非致命**：警告→stderr、返回 false、不中断；仅 `assert`/`fail` 致命 `AssertionError`） | task-info §1 注意 + D-007/D-011/D-015 A4 | `cargo run -q -- run docs/reports/fixtures-p3/08_builtins.lfz` → exit 0；`cargo test`（`check` 非致命用例） | 通过 | 交付物 1、2；`src/builtins.rs`,`src/evaluator.rs` | 1（+4） |
| R-110 | **特色⑤** 确定性语义「无魔法」（求值顺序左→右 B1；`for` 迭代快照 B2；键 UTF-8 字节序 B3；越界/缺字段/溢出/类型不符一律结构化报错；NaN/±Inf 规则 B5） | task-info §1 注意 + D-007/D-015 B1–B3/B5 | `cargo test`（evaluator 求值顺序/键序/快照用例）全绿；`cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz` | 通过 | 交付物 1、2；`src/evaluator.rs` | 1（+4） |
| R-111 | `.lfz` **文件**首行**恰好** `#42`（三字符 + 行终止符）；**仅按扩展名**（ASCII 大小写不敏感 `.lfz`/`.LFZ`）触发，REPL/stdin/`-e` 豁免；违规 → `CosmosAnswerError: 你忘记了宇宙的答案`（无源码行/插入符） | D-013/D-014/D-016 B2 | `Set-Content -Encoding utf8 _t.lfz 'print(1)'; cargo run -q -- run _t.lfz` → stderr 末行 `CosmosAnswerError: 你忘记了宇宙的答案`、exit 2；`cargo test`（loader/`#42` 用例） | 通过 | 交付物 2；`src/loader.rs`,`src/cli.rs` | 1 |
| R-112 | **Python 风格错误模型**：基类 `LfzError` + **12 个具体错误类**（`CosmosAnswerError`/`SyntaxError`/`NameError`/`TypeError`/`IndexError`/`FieldError`/`ZeroDivisionError`/`OverflowError`/`ValueError`/`IOError`/`AssertionError`/`RecursionError`）；输出 `类名: 中文消息` + `line N, col M` + 源行 + 插入符；运行期错误带 `Traceback`（首 10+尾 30 折叠）；**禁止 `E-xxx`**；退出码 **0/1/2**（成功 / `lfz test` 失败 / 其他错误·环境） | D-008/D-013/D-014/D-015 A7/D-021 | `cargo run -q -- run --json <含 1/0 的程序>` → stdout 单行 JSON，`error` 为 12 类名之一且不含 `E-xxx`、exit 2；`cargo test`（error/cli 格式化用例） | 通过 | 交付物 2；`src/error.rs`,`src/cli.rs` | 1 |
| R-113 | `§10.7` 内置函数表 **54 个**，data-last 约定（回调在前、容器末参；容器更新返回新值 A1） | D-015 B9 | `cargo test`（`BUILTIN_NAMES` 54 项一致性 + HOF 用例）全绿 | 通过 | 交付物 2；`src/builtins.rs` | 1 |
| R-114 | 语法体现**自身特色**（5 特色组合），**不与任何现存语言完全相同** | task-info §1 注意 + D-007 | `docs/spec/` 记录 5 特色；`cargo run -q -- run docs/reports/fixtures-p3/05_pipe.lfz`（`\|>`+`_` 组合）与 `spec_9_4_current.lfz`（环安全 `<cycle>`）可运行 | 通过 | 交付物 1；`docs/spec/` | 1（+4） |
| R-115 | 实现语言 = **Rust**，**仅 std、零第三方依赖** | D-011 | `Get-Content Cargo.toml` 的 `[dependencies]` 段为空；`cargo build` 0 warning | 通过 | 交付物 2；`Cargo.toml` | 1 |

### 1.2 自动测试（评分项 2 = 20 分）

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-201 | 完整**黑盒测试集**，脚本用 LFZ 编写，覆盖**所有功能**，每特性 ≥3 用例（正常/边界/错误） | task-info §2 + D-021 | `cargo run -q -- test` → 汇总「全部 PASS」、exit 0；`tests/coverage-matrix.md` 覆盖全部特性且每特性 ≥3 用例 | 待办 | 交付物 3；`tests/**/*.lfz`,`tests/coverage-matrix.md` | 2 |
| R-202 | **一个命令**执行所有测试（默认发现 `tests/**/*.lfz`，排除 `tests/fixtures/**`；稳定排序；0 用例 → exit 2 不静默成功） | task-info §2 + D-021 + runner-contract §2/§5 | `cargo run -q -- test` 为唯一入口；runner 机制已实现（`cargo test` 中 `test_runner` 12 例全绿）；**待 R-201 用例落地后**该命令应 exit 0 | 进行中 | 交付物 3；`src/test_runner.rs` | 2 |
| R-203 | 测试失败可定位：用例名 + 期望 vs 实际 + `§8.2` 位置（`File "<path>", line N[, in func]` + 源码行 + 插入符 + `类名: 消息`） | task-info §2 + runner-contract §6 | 注入失败用例 → 报告含 `FAIL <name> (<path>:<line>:<col>)` + 缩进错误块；`cargo test`（`tr3_*` 用例） | 通过 | 交付物 3；`src/test_runner.rs`,`src/cli.rs` | 2 |
| R-204 | 负例须经 `tests/cases.json` 的 `expect.error` 声明并使用**非自动发现**夹具（`tests/fixtures/`）；清单类名取 12 类之一 | D-021 + runner-contract §3/§4 | `cargo run -q -- test` 对清单声明的负例判 PASS；类名非法 → exit 2 | 待办 | 交付物 3；`tests/cases.json`,`tests/fixtures/` | 2 |

### 1.3 性能（评分项 3 = 10 分）

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-301 | 提供 LFZ 性能脚本与**等价** Python 脚本 | task-info §3 | `benchmarks/` 下两脚本存在且可运行（LFZ 脚本首行 `#42`） | 待办 | 交付物 4；`benchmarks/` | 3 |
| R-302 | 性能**报告**，含方法/环境/数据/结论，采用**预热 + 多轮 + 中位数**的公平对比 | task-info §3 | `docs/reports/performance.md` 存在，且含上述四要素与数值表 | 待办 | 交付物 4；`docs/reports/performance.md` | 3 |
| R-303 | 性能结论可复现（报告内命令可原样执行） | D-008 | 按报告命令重跑得同量级结果 | 待办 | 交付物 4；`benchmarks/` | 3 |

### 1.4 文档（评分项 4 = 20 分）

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-401 | LFZ **语法规则文档**（完整、含 EBNF 与样例；词法/文法/语义/错误/接口契约） | task-info §4/提交 1 + D-016 | `docs/spec/{syntax,semantics,interface-contract}.md` 均存在且非空（62,389 / 33,931 / 30,592 字节）；文首含「唯一事实源（冻结于 2026-09-23）」 | 通过 | 交付物 1、5；`docs/spec/` | 4 |
| R-402 | **人类向**开发指南/手册（教程 + 语法 + 运行/错误说明） | task-info §4 | `docs/guide/` 存在且含入门、语法、运行与错误章节 | 待办 | 交付物 5；`docs/guide/` | 4 |
| R-403 | **给 AI 的**开发指南/skill，并经**全新 agent 实测**验证可顺利用 LFZ 写代码 | task-info §4 | `.opencode/skills/lfz-programming/`（或等价文档）存在 + 一份实测记录（命令与结果） | 待办 | 交付物 5；`.opencode/skills/lfz-programming/` | 4 |
| R-404 | AI 指南**硬性内容**：写死「`.lfz` 首行 `#42`」+ 12 个错误类清单 + 禁止旧 `E-xxx` + 示例一律带头 | D-013/D-014 §下游硬性要求 | 指南文本含 `#42` 与 12 类名；示例代码块首行均为 `#42` | 待办 | 交付物 5；`.opencode/skills/lfz-programming/` | 4 |

### 1.5 应用（评分项 5 = 30 分）

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-501 | 用 LFZ 实现**功能完整、可运行、代码 ≥200 行**的应用（选题 = 排序算法可视化，ASCII 条形图） | task-info §5 + D-011 | `cargo run -q -- run app/<入口>.lfz` 可运行、exit 0；`app/` 下 `.lfz` 总行数 ≥200 | 待办 | 交付物 6；`app/` | 5 |
| R-502 | **开发记录**（Agent 提示词 / 迭代 / 踩坑 / 修复） | task-info §5 | `app/DEV_RECORD.md` 存在且含上述四类内容 | 待办 | 交付物 6；`app/DEV_RECORD.md` | 5 |
| R-503 | 应用文件合规：`.lfz` 首行 `#42`、错误用类名 + 中文消息、无 `E-xxx` | D-013/D-014 | 扫描 `app/**/*.lfz` 首行均为 `#42`；应用代码不含 `E-` 编号引用 | 待办 | 交付物 6；`app/` | 5 |

### 1.6 环境与提交物（无独立分值）

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-601 | 解释器实现语言**无限制** | 环境要求 1 | `Cargo.toml` 存在，`cargo build` 成功 | 通过 | 交付物 2；`Cargo.toml` | — |
| R-602 | 编程 Agent 及大模型使用**无限制** | 环境要求 2 | 团队架构与 `.opencode/` 存在 | 通过 | 交付物 7；`.opencode/` | — |
| R-603 | **必须使用 GIT** 版本管理 | 环境要求 3 | `git log --oneline` 非空（含 `e060b21`→`7527792`）；标签 `v0.1.0`/`v0.2.0` | 通过 | 交付物 7；`.git/` | — |
| R-604 | 提交物 1：LFZ 语法规则文档 | 提交内容 1 | 同 R-401 | 通过 | `docs/spec/` | 4 |
| R-605 | 提交物 2：可执行 LFZ 的解释器源程序 | 提交内容 2 | 同 R-101 ~ R-115（`cargo run -q -- run examples/hello.lfz` → `Hello, LFZ!`） | 通过 | `src/` | 1 |
| R-606 | 提交物 3：LFZ 完整黑盒测试集 | 提交内容 3 | 同 R-201 ~ R-204 | 待办 | `tests/` | 2 |
| R-607 | 提交物 4：LFZ 性能测试报告 | 提交内容 4 | 同 R-301 ~ R-303 | 待办 | `benchmarks/`、`docs/reports/performance.md` | 3 |
| R-608 | 提交物 5：开发指南文档 + 给 AI 的文档 | 提交内容 5 | 同 R-402 ~ R-404 | 待办 | `docs/guide/`、`.opencode/skills/lfz-programming/` | 4 |
| R-609 | 提交物 6：应用源代码 + 开发记录 | 提交内容 6 | 同 R-501 ~ R-503 | 待办 | `app/`、`app/DEV_RECORD.md` | 5 |
| R-610 | 提交物 7：所有工作的 Git 历史记录 | 提交内容 7 | 同 R-603；远程 `https://github.com/NeitherTourRest/lfz-programing-language` 可达 | 通过 | `.git/` | — |
| R-611 | 提交物 8：系统介绍 PPT | 提交内容 8 | `docs/slides/` 存在且含 `.pptx`（或等价） | 待办 | `docs/slides/` | — |
| R-612 | 线下验收答辩（讲 PPT + 演示 + 回答提问） | 作业验收 | 演示脚本 + 答辩预案齐备；现场演示 `cargo run -- run examples/hello.lfz` 与 `cargo run -- test` 成功 | 待办 | `docs/slides/` + 演示脚本 | — |

---

## 2. 评分权重映射（20/20/10/20/30）

| 评分项 | 分值 | 对应 R-ID | 硬性通过条件 |
|--------|------|-----------|--------------|
| 1. LFZ 解释器实现 | **20** | R-101 ~ R-115（+ R-605、R-601） | `cargo run -q -- run examples/hello.lfz` → `Hello, LFZ!`；`cargo test` 全绿；5 特色可运行；`#42`+Python 错误模型生效 |
| 2. LFZ 解释器自动测试脚本 | **20** | R-201 ~ R-204（+ R-606） | **一个命令**跑全部；覆盖全部功能；每特性 ≥3 用例（正常/边界/错误）；失败可定位 |
| 3. LFZ 解释器性能测试 | **10** | R-301 ~ R-303（+ R-607） | 报告交付；LFZ vs Python **公平对比**（预热/多轮/中位数 + 报告可复现） |
| 4. 语法说明 + 人/AI 开发指南 | **20** | R-401 ~ R-404（+ R-604、R-608） | 语法文档冻结 + 人类手册 + AI 指南/skill 齐备且 AI 指南经实测 |
| 5. 利用编程 Agent 开发程序 | **30** | R-501 ~ R-503（+ R-609） | 应用可运行、≥200 行、功能完整、含开发记录 |
| （支撑）环境与提交完整性 | 无独立分值 | R-601 ~ R-603、R-610 ~ R-612 | Git 历史完整 + 8 项提交物齐备 + 答辩载体 |

---

## 3. 需求变更记录（开工后需求口径变化）

> 记录**开工后**对初版（R-001 ~ R-023，见附录 A）的口径**推翻 / 补充 / 细化**，逐条附 ADR 依据。初版条目**均未撤销**，仅被细化到 R-1xx ~ R-6xx。

| # | 变更项 | 初版口径（R-0xx） | 现行 v1 口径 | 依据 ADR | 影响 |
|---|--------|-------------------|--------------|----------|------|
| 1 | **实现语言** | R-012「语言无限制」，默认建议 Python 3.13（D-004/D-006 提案） | **Rust**，cargo 工程，**仅 std、零第三方依赖** | D-006（提案）→ **D-011（用户拍板改选 Rust）** | R-115、R-601；架构按 `enum Value` + `Rc<RefCell<…>>` + `Result<T, LfzError>` 重规划 |
| 2 | **v1 功能范围** | R-006「有特色即可」 | **5 特色全纳入 + `float`**（管道/合一 struct/富插值/结构化错误+assert·check/确定性语义）；11 项延后 v1.1（生成器、match、REPL、模块等） | D-007（提案）→ **D-011 确认** | 细化为 R-106 ~ R-110 |
| 3 | **语法口径** | 未规定块/语句/注释风格 | C 风格 `{}` 块；**换行终结**语句（无分隔符）；`;;` 变量 dump；`;` 单独出现恒 `SyntaxError`；多行续行**仅括号内**；`${}` 插值 | D-012（v0.2） | R-103、R-106、R-108 |
| 4 | **文件前导 `#42`** | 无此要求 | `.lfz` 文件首行**恰好** `#42`；**仅扩展名触发**（ASCII 大小写不敏感）；REPL/stdin/`-e` 豁免；违规 → `CosmosAnswerError: 你忘记了宇宙的答案` | D-013（草案）→ **D-014 定案** | R-111；下游 test/docs/ai-dx/app 全部 `.lfz` 须带头 |
| 5 | **错误模型** | 初版无错误模型规定；D-012 曾引入 `E-xxx` 编号 | **Python 风格**：类名 + 中文消息 + `line/col` + 源行 + 插入符；**12 类 + 基类**；运行期带 `Traceback`；**取消 `E-xxx`**；退出码 **0/1/2** | D-013（取代 D-012 的 `E-xxx` 部分）/ D-014 / D-015 A7 新增 `RecursionError` / D-008 退出码 | R-112 |
| 6 | **可移植性** | 无 | 行终止符 `\n`/`\r\n`/`\r`（归一化）；文件开头 UTF-8 BOM 静默跳过；**非 UTF-8 → `SyntaxError`**（先编码校验后前导校验） | D-013/D-014 | R-111、R-112 |
| 7 | **语义定稿 A1–A7** | 无 | 引用语义（A1）；按 cell 捕获（A2）；除法/取模 Python 一致（A3）；`check` 非致命（A4）；方法不入数据面（A5）；环安全 `<cycle>`（A6）；`RecursionError`（A7） | **D-015** | R-101、R-105、R-107、R-109、R-110 |
| 8 | **求值语义补钉 B1–B13** | 无 | 左→右求值；`for` 快照；键 UTF-8 序；递归上限 10000；NaN/Inf 规则；`input()` EOF→`IOError`；struct 平拷贝；54 内置 data-last 等 | **D-015 B** | R-110、R-113 |
| 9 | **可解析性补钉 M1–M6 / A9 / A27** | 无 | `${…}` 内禁裸换行；`=> {…}` 恒为块；`_` 只绑定最近管道 RHS；**无裸块语句**（语句首 `{` = 匿名 struct 字面量）；`.self` 非法（用 `s["self"]`） | D-015 C + P3.11 裁定（bug-08 / bug-07） | R-106、R-107 |
| 10 | **spec v1 冻结** | 无 | `docs/spec/{syntax,semantics,interface-contract}.md` 于 2026-09-23 冻结，**变更须先 ADR 后改文档** | **D-016** | R-401、R-114 |
| 11 | **冻结后补钉（post-v1）** | 无 | `min([])`/`max([])` → `ValueError::EmptyExtremum`「空数组没有极值（{func}）」；`randInt(lo≥hi)` → `ValueError::BadRange`；`pop([])` → `Index{idx:-1,len:0}`；`insert` 不支持负索引；`del` 仅数据字段；未闭合块注释 → `SyntaxError`；`let` 重绑定 → `TypeError::ImmutableRebind`；traceback 折叠（首 10+尾 30） | DECISIONS 2026-09-24 各条（契约缺口闭合 / 未闭合块注释裁定 / P3.11 裁定 1/3） | R-109、R-112、R-113 |
| 12 | **运行命令契约** | 待定（PROJECT_STATE §5 旧注） | `lfz run <file>` / `lfz test [路径…]` / `--json` 全局开关 / `--help`/`--version`；`--json` 下 stdout 恒为单行 JSON | D-008 + runner-contract v1.2（P4.1/P4.2/P4.2-fix） | R-202、R-203、R-204、R-112 |
| 13 | **交付物位置与判定** | 初版未指定路径 | `docs/spec/`、`src/`、`tests/`、`benchmarks/`、`docs/guide/`、`app/`、`.git/`、`docs/slides/` | D-009 / PROJECT_STATE §4 | R-6xx 全部 |
| 14 | **8 项提交物** | R-015 ~ R-022 逐项 | 保留 8 项，逐项配可命令验证 | task-info + D-016 | R-604 ~ R-611 |

---

## 4. 追溯矩阵（需求 ↔ 交付物 ↔ 评分项 ↔ 当前状态）

| R-ID | 交付物（位置） | 评分项 | 状态 | 复核命令 |
|------|----------------|--------|------|----------|
| R-101 | 交付物 2 `src/value.rs`,`src/evaluator.rs` | 1 | ✅ 已满足 | `cargo run -q -- run examples/hello.lfz` |
| R-102 | 交付物 1、2 `src/value.rs` | 1 | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3/04_containers.lfz` |
| R-103 | 交付物 2 `src/builtins.rs` | 1 | ✅ 已满足 | `cargo run -q -- run examples/hello.lfz` |
| R-104 | 交付物 2 `src/parser.rs`,`src/evaluator.rs` | 1 | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3/02_control.lfz` |
| R-105 | 交付物 2 `src/evaluator.rs`,`src/env.rs` | 1 | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz` |
| R-106 | 交付物 1、2 `src/parser.rs` | 1（+4） | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3/05_pipe.lfz` |
| R-107 | 交付物 1、2 `src/value.rs`,`src/builtins.rs` | 1（+4） | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz` |
| R-108 | 交付物 1、2 `src/lexer.rs`,`src/parser.rs` | 1（+4） | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3/06_interp.lfz` |
| R-109 | 交付物 1、2 `src/builtins.rs`,`src/evaluator.rs` | 1（+4） | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3/08_builtins.lfz` |
| R-110 | 交付物 1、2 `src/evaluator.rs` | 1（+4） | ✅ 已满足 | `cargo test` |
| R-111 | 交付物 2 `src/loader.rs`,`src/cli.rs` | 1 | ✅ 已满足 | 构造无 `#42` 的 `.lfz` → `cargo run -q -- run <f>`（exit 2 + `CosmosAnswerError`） |
| R-112 | 交付物 2 `src/error.rs`,`src/cli.rs` | 1 | ✅ 已满足 | `cargo run -q -- run --json <含 1/0 的程序>` |
| R-113 | 交付物 2 `src/builtins.rs` | 1 | ✅ 已满足 | `cargo test` |
| R-114 | 交付物 1 `docs/spec/` | 1（+4） | ✅ 已满足 | 见 §6 命令 2/4 |
| R-115 | 交付物 2 `Cargo.toml` | 1 | ✅ 已满足 | `Get-Content Cargo.toml`（空 `[dependencies]`）+ `cargo build` |
| R-201 | 交付物 3 `tests/**/*.lfz` | 2 | ⬜ 未做 | `cargo run -q -- test`（P5 后应 exit 0） |
| R-202 | 交付物 3 `src/test_runner.rs` | 2 | 🟡 部分 | `cargo run -q -- test` |
| R-203 | 交付物 3 `src/test_runner.rs`,`src/cli.rs` | 2 | ✅ 已满足 | `cargo test`（`tr3_*`） |
| R-204 | 交付物 3 `tests/cases.json` | 2 | ⬜ 未做 | `cargo run -q -- test` |
| R-301 | 交付物 4 `benchmarks/` | 3 | ⬜ 未做 | — |
| R-302 | 交付物 4 `docs/reports/performance.md` | 3 | ⬜ 未做 | `Test-Path docs/reports/performance.md` |
| R-303 | 交付物 4 `benchmarks/` | 3 | ⬜ 未做 | — |
| R-401 | 交付物 1、5 `docs/spec/` | 4 | ✅ 已满足 | `Test-Path docs/spec/syntax.md,docs/spec/semantics.md,docs/spec/interface-contract.md` |
| R-402 | 交付物 5 `docs/guide/` | 4 | ⬜ 未做 | `Test-Path docs/guide` |
| R-403 | 交付物 5 `.opencode/skills/lfz-programming/` | 4 | ⬜ 未做 | `Test-Path .opencode/skills/lfz-programming` |
| R-404 | 交付物 5 同上 | 4 | ⬜ 未做 | 指南含 `#42` + 12 类名 |
| R-501 | 交付物 6 `app/` | 5 | ⬜ 未做 | `cargo run -q -- run app/<入口>.lfz`；行数 ≥200 |
| R-502 | 交付物 6 `app/DEV_RECORD.md` | 5 | ⬜ 未做 | `Test-Path app/DEV_RECORD.md` |
| R-503 | 交付物 6 `app/` | 5 | ⬜ 未做 | 扫描 `app/**/*.lfz` 首行 = `#42` |
| R-601 | 交付物 2 `Cargo.toml` | — | ✅ 已满足 | `cargo build` |
| R-602 | 交付物 7 `.opencode/` | — | ✅ 已满足 | `Test-Path .opencode` |
| R-603 | 交付物 7 `.git/` | — | ✅ 已满足 | `git log --oneline` |
| R-604 | `docs/spec/` | 4 | ✅ 已满足 | 同 R-401 |
| R-605 | `src/` | 1 | ✅ 已满足 | `cargo run -q -- run examples/hello.lfz` |
| R-606 | `tests/` | 2 | ⬜ 未做 | 同 R-201 |
| R-607 | `benchmarks/`、`docs/reports/performance.md` | 3 | ⬜ 未做 | 同 R-302 |
| R-608 | `docs/guide/`、skills | 4 | ⬜ 未做 | 同 R-402/R-403 |
| R-609 | `app/` | 5 | ⬜ 未做 | 同 R-501 |
| R-610 | `.git/` | — | ✅ 已满足 | `git ls-remote origin`（连通时） |
| R-611 | `docs/slides/` | — | ⬜ 未做 | `Test-Path docs/slides` |
| R-612 | `docs/slides/` + 演示脚本 | — | ⬜ 未做 | 现场演示 |

---

## 5. 覆盖状态汇总

**总计 41 条**（R-101 ~ R-115 共 15；R-201 ~ R-204 共 4；R-301 ~ R-303 共 3；R-401 ~ R-404 共 4；R-501 ~ R-503 共 3；R-601 ~ R-612 共 12）。

| 覆盖状态 | 条数 | 占比 | 主要缺口 |
|----------|------|------|----------|
| ✅ 已满足 | **23** | 56% | —（P3+P4 已交付部分） |
| 🟡 部分 | **1** | 2% | R-202（runner 机制已就绪，缺 LFZ 黑盒用例） |
| ⬜ 未做 | **17** | 41% | 黑盒测试集（P5）、性能（P6）、指南（P7）、应用（P8）、PPT（P10） |

**按评分项看**：

| 评分项 | 状态 |
|--------|------|
| 1. 解释器（20） | ✅ 主体已满足（R-101~R-115 全通过）；P3 独立验收 rev.3 PASS（10/10 缺陷闭环） |
| 2. 自动测试（20） | 🟡 runner 基础设施已就绪（R-202/R-203），**待 P5 落地 LFZ 黑盒用例（R-201/R-204）** |
| 3. 性能（10） | ⬜ 未做（P6） |
| 4. 文档（20） | 🟡 语法文档 ✅（R-401）；**人/AI 指南 ⬜（R-402~R-404，P7）** |
| 5. 应用（30） | ⬜ 未做（P8） |
| 支撑（Git/提交完整性） | 🟡 Git ✅（R-603/R-610）；PPT/答辩 ⬜（R-611/R-612，P10） |

**8 项提交物交付状态**：

| # | 提交物 | 状态 |
|---|--------|------|
| 1 | LFZ 语法规则文档 | ✅ 已冻结（`docs/spec/`，D-016） |
| 2 | 解释器源程序 | ✅ 已交付（`src/`，431 测试全绿，`cargo build` 0 warning） |
| 3 | 完整黑盒测试集 | 🟡 runner 已就绪；LFZ 用例集待 P5 |
| 4 | 性能测试报告 | ⬜ 待 P6 |
| 5 | 开发指南（人 + AI） | ⬜ 待 P7 |
| 6 | 应用源码 + 开发记录 | ⬜ 待 P8 |
| 7 | Git 历史记录 | ✅ 已建立（本地 + 远程 Public） |
| 8 | 系统介绍 PPT | ⬜ 待 P10 |

---

## 6. 验收清单（可直接照跑）

> 在**项目根**运行。命令中 `cargo run -q --` 抑制 cargo 自身输出；`$LASTEXITCODE` 取退出码。

```powershell
# 0. 构建：期望 0 warning / 0 error
cargo build

# 1. 全量测试：期望 431 passed / 0 failed / 0 ignored（数值随迭代增长，关键为 0 failed）
cargo test

# 2. 端到端 Hello：期望 stdout 'Hello, LFZ!'，exit 0
cargo run -q -- run examples/hello.lfz

# 3. 综合样例（引用语义 A1 / 闭包捕获 A2 / 环安全 A6）：
#    期望 5 行：1 2 3 / 99 99 / [3, 1, 2]  [1, 2, 3] / {me: <cycle>} / true；exit 0
cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz

# 4. 5 特色演示（管道 |>/_ 等）：期望 9 行输出、exit 0
cargo run -q -- run docs/reports/fixtures-p3/05_pipe.lfz

# 5. `#42` 前导强制（R-111）：期望 stderr 含 'CosmosAnswerError: 你忘记了宇宙的答案'、exit 2
Set-Content -Encoding utf8 _accept_nohdr.lfz 'print(1)'
cargo run -q -- run _accept_nohdr.lfz; "exit=$LASTEXITCODE"
Remove-Item _accept_nohdr.lfz

# 6. Python 风格错误 + 机器可读（R-112）：期望 stdout 单行 JSON（error 为 12 类名之一、无 E-xxx）、exit 2
Set-Content -Encoding utf8 _accept_div.lfz "#42`nprint(1 / 0)"
cargo run -q -- run --json _accept_div.lfz; "exit=$LASTEXITCODE"
Remove-Item _accept_div.lfz

# 7. 一键黑盒测试（R-201/R-202）：P5 交付后应 exit 0 且全部 PASS；
#    当前无 LFZ 用例 → exit 2 且 stderr 'lfz test: 未发现任何测试用例'
cargo run -q -- test; "exit=$LASTEXITCODE"

# 8. 版本与帮助：期望 'lfz 0.1.0'、exit 0
cargo run -q -- --version

# 9. 交付物存在性断言（期望全部 True）
Test-Path docs/spec/syntax.md, docs/spec/semantics.md, docs/spec/interface-contract.md, examples/hello.lfz, src/lib.rs

# 10. 第三方依赖为零（R-115）：期望 [dependencies] 段为空
Get-Content Cargo.toml | Select-String -Pattern '\[dependencies\]' -Context 0,3

# 11. Git 历史（R-603/R-610）：期望非空、含 v0.1.0 / v0.2.0 标签
git log --oneline -5; git tag
```

**P9 验收对表要点**：verifier 对 R-1xx（评分项 1）以命令 2–6 为主；R-2xx 以命令 7 为主（P5 交付后）；R-4xx 以命令 9 + 人工核 `docs/guide/`、skills；R-5xx 以 `cargo run -- run app/<入口>.lfz` + 行数统计；R-6xx 以命令 9–11。

**已知输入依赖**：命令 5/6 需 PowerShell 以 UTF-8 写文件（`Set-Content -Encoding utf8`）；若在 Windows PowerShell 5.1 下 BOM 干扰，可改用 `[IO.File]::WriteAllText("_accept_div.lfz","#42`nprint(1 / 0)")`（无 BOM）。

---

## 附录 A. 初版需求条目索引（R-001 ~ R-023，保留）

> 初版条目**均未撤销**，作为 task-info 原始要求的来源锚点保留；其口径已细化到 §1 的 R-1xx ~ R-6xx（见「细化到」列）。外部引用（如 `BRAINSTORM.md` 的 R-007/R-008/R-023、`docs/tooling/runner-contract.md` 的 R-008）**依然有效**。

| 初版 R-ID | 需求（保留原文摘要） | 来源 | 细化到 |
|-----------|----------------------|------|--------|
| R-001 | 支持整数、字符串等基本数据类型 | task-info §1 | R-101 |
| R-002 | 支持数组、结构类型 | task-info §1 | R-102 |
| R-003 | 支持基本输入输出 | task-info §1 | R-103 |
| R-004 | 支持分支循环控制 | task-info §1 | R-104 |
| R-005 | 支持函数定义和调用 | task-info §1 | R-105 |
| R-006 | 语法体现自身特色，不与现存语言完全相同 | task-info §1 注意 | R-106 ~ R-110、R-114 |
| R-007 | 完整黑盒测试集，覆盖所有功能 | task-info §2 | R-201 |
| R-008 | 一个命令执行所有测试 | task-info §2 | R-202、R-203 |
| R-009 | 性能测试脚本并与 Python 对比 | task-info §3 | R-301 ~ R-303 |
| R-010 | 开发指南供编程 Agent 使用 | task-info §4 | R-402、R-403 |
| R-011 | Agent 辅助开发 ≥200 行应用 | task-info §5 | R-501、R-502 |
| R-012 | 解释器实现语言无限制 | 环境要求 1 | R-115、R-601 |
| R-013 | 编程 Agent 及大模型使用无限制 | 环境要求 2 | R-602 |
| R-014 | 必须使用 GIT 版本管理 | 环境要求 3 | R-603 |
| R-015 | LFZ 语法规则文档 | 提交内容 1 | R-401、R-604 |
| R-016 | LFZ 解释器源程序 | 提交内容 2 | R-605 |
| R-017 | LFZ 完整黑盒测试集 | 提交内容 3 | R-606 |
| R-018 | LFZ 性能测试报告 | 提交内容 4 | R-607 |
| R-019 | 开发指南文档 + AI 文档 | 提交内容 5 | R-608 |
| R-020 | 应用源代码 + 开发记录 | 提交内容 6 | R-609 |
| R-021 | Git 历史记录 | 提交内容 7 | R-610 |
| R-022 | 系统介绍 PPT | 提交内容 8 | R-611 |
| R-023 | 线下验收答辩 | 作业验收 | R-612 |
