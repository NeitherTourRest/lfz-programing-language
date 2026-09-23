# LFZ 动工前调研与决策记录（KICKOFF）

> 日期: 2026-09-23 by team-lead（本会话总调度）
> 状态: **调研完成；四项决策已经用户确认定案**（见 DECISIONS **D-011**）：**实现语言 = Rust**（用户改选，原建议 Python）；v1 范围 = 5 特色 + float；开发方式 = 契约先行 + 双轨 TDD；阶段划分见 D-009。**注意：Rust 需新增「工具链安装」前置任务，且架构按 Rust 调整。**
> 来源: 4 位专家并行调研 —— language-architect / tooling-dev / perf-engineer / librarian（含外部硬数据）。

---

## 一、调研结论摘要

### 1) 实现语言（原 D-004）
**四方独立结论一致 → Python 3.13（纯标准库，零第三方依赖）。**

- **六维加权**（tooling-dev）：Python **4.45** / Go 3.28 / TS 3.18 / **Rust 2.58**。
- **环境事实（决定性）**：本机已装 Python 3.13.9；**`rustc` 与 `go` 均未安装** → 选 Rust/Go 会**阻塞 P2→P3→P4→P5/P8 主关键路径**（需先装整套工具链并验证）。
- **性能真相**（librarian 硬数据 + perf-engineer）：
  - 跨语言基准 ceronman/loxido 显示，**任何树遍历解释器都比 CPython 慢**（Java 树遍历 jlox 比 Python 慢约 2–3×；Python 托管的树遍历器慢 50–1000×）；只有 C/unsafe-Rust **字节码 VM** 才稳定快于 Python。
  - **评分项 3（10 分）考的是报告的"公平方法学 + 诚实瓶颈分析"，不是"跑赢 Python"**。Rust/Go 不为这 10 分加分，却会威胁权重更高的解释器（20）+ 测试（20）+ 应用（30）。
- **写作速度**：Python > Go >> Rust（多个 Rust 版 Lox 作者独立反馈：借用检查器、`Rc` 循环泄漏、内存管理是主要痛苦源）。
- **参照实现**：《Crafting Interpreters》jlox（<2000 行 Java 实现 Lox 全功能）、《Writing an Interpreter in Go》（~3500 行、零依赖、全 TDD）、《Let's Build a Simple Interpreter》（Python）。

### 2) 功能范围（v1）
- 身份叙事：**「万物皆值，值皆可流」** + **「一种结构，两种面孔」**。
- 选入 **5 个特色**（详见 §二）。
- 排除/延后 **11 项**（生成器/惰性流、match、值式错误处理、静态检查器、REPL、fmt/doc/watch 等）。

### 3) 开发方式
- **契约先行**：language-architect 冻结 `docs/spec/interface-contract.md`（AST 节点类型 / `evaluate(node, env) -> Value` / `LFZError` 错误模型），**冻结前 dev 不开工核心**。
- **双轨 TDD**：单测（Python/pytest，`tests/unit/`，core-dev/runtime-dev 唯一写）与黑盒（LFZ 脚本，`tests/`，test-engineer 唯一写）分离。
- **并行**：runtime-dev 用手工构造 AST 先行，不被前端阻塞。
- **测试契约先行**：tooling-dev 先定 **runner 契约**（文件发现 / `assert_*` 接口 / 失败输出 / **0-1-2 退出码** / 汇总格式，附"一过一败"两个联调基准用例），经 team-lead 转 test-engineer 并行推进。
- **Windows 优先**：纯标准库、`scripts/lfz.ps1`+`lfz.bat` 启动器、全链路显式 UTF-8、ANSI 自动降级。

### 4) 阶段划分（关键路径）
`P0.5 git 基线 → P2 spec 冻结 → P3 前端 AST + 求值 MVP → P4 CLI + runner → P5 黑盒测试 / P8 应用 → P9 验证 → P10 发布+答辩`。
（P1 需求实际已基本完成：`REQUIREMENTS.md` 23 条已就绪，P2 前正式标记即可。）

---

## 二、v1 功能范围（提案草案，待用户决策）

### 必做核心（task-info 硬要求）
- **值类型**：`int`（推荐 64-bit 有符号，溢出报错）、`float`（**推荐纳入**）、`string`、`bool`、`nil`、`array`、`struct`、`function`。
- **字面量 / 变量**：整数字面量、字符串（双引号 + 转义 `\n \t \\ \" \e`）、`true/false`、`nil`、`[a,b,c]`、`Name { k: v }`；`let`（绑定不可重赋值）/ `var`（可重赋值）。
- **运算符 + 完整优先级表 + 结合性**（含 `.`/`[]`/调用、一元、`* / %`、`+ -`、比较、`== !=`、`&&`、`||`、`|>`、`=`）。
- **控制流**：`if / else if / else`、`while`、`for v in seq`、`break` / `continue`。
- **函数**：`fn name(params){...}`、`return`、递归、一等函数/闭包、`lambda (x) => e`。
- **基本 IO**：`print(...)`、`input(prompt?)`、显式转换 `int()/float()/str()`。
- **注释**：`//` 行、`/* */` 块（不可嵌套）。
- **语句终结**：换行终结，`;` 可作显式分隔符；`(` `[` 内换行忽略，`{}` 块内换行有效。
- **真值规则**：条件位置**必须是 `bool`**（无 truthiness）。

### 选入的 5 个「惊喜」特色
| # | 特色 | 一句话 | 主要加分 |
|---|------|--------|---------|
| 1 | **管道 `\|>` + `_` 占位** | `xs \|> sum()`；`10 \|> div(_, 2)` 尾参注入/任意位插入；parser 阶段脱糖、运行时零开销 | 应用 30 / 文档 20 |
| 2 | **合一 `struct`** | 数组之外唯一复合类型：`s.k` 与 `s["k"]` 同源、带方法、可动态加字段（dict 支撑、字段名 interned，O(1)） | 解释器 20 / 应用 30 |
| 3 | **富字符串插值** | `"hello ${name}! ${x:.2f}"`（`${expr}` + 格式说明符；parser 脱糖） | 应用 30 / 文档 20 |
| 4 | **结构化错误 + `assert`/`check`** | 错误含 `类别/码/行列/hint`，`--json` 机器可读；语言级自验原语 | 解释器 20 / 应用 30 / 测试 20 |
| 5 | **确定性语义「无魔法」** | 无隐式转换、越界即错、求值顺序写死、条件必须 bool（spec 级纪律，零代码换最大正确率） | 全项 |

### 排除 / 延后（v1 不做）
生成器 + 惰性流（#16）、`match` 模式匹配（#14）、值式错误处理 `rescue/?/expect`（#13）、一等区间/切片（#8）、值式循环 + 标签 break（#15）、`lfz check`（#12）、REPL（#18）、`lfz fmt/doc/init/package/watch`（#17/19/20/21）、模块/import、类/继承、静态类型注解。

> 说明：延后项均"成本高 或 不直接命中评分 或 与 v1 精简目标冲突"；其中生成器/惰性流在 Python 上是**性能负项**（每元素协程开销高），纳入理由是表达力而非性能。

---

## 三、四项决策（经用户确认；见 `DECISIONS.md` D-006~D-009 + **D-011 定案**）

| ADR | 决策 |
|---|---|
| **D-006** | 原建议实现语言 = Python 3.13；**用户改选 Rust（见 D-011，需先装 Rust 工具链）**。 |
| **D-007** | v1 功能范围 = 必做核心 + 上述 5 特色；11 项延后（见 §二）。 |
| **D-008** | 开发方式 = 契约先行 + 双轨 TDD + runner 契约先行 + Windows 优先。 |
| **D-009** | 阶段划分 = P0.5→P2→P3→P4→P5/P8→P9→P10 关键路径 + 各阶段门禁。 |

---

## 四、待用户最终确认（可覆盖，改动即时同步）

1. **实现语言**：默认 **Python 3.13**（推荐，证据充分）。若你更看重"LFZ 性能叙事/答辩 wow"，可改用 **Go**（单静态二进制、分发省事），但我团队会先装工具链（增加前置时间）。
2. **v1 功能范围**：上述 5 特色是否全部纳入？是否有你想加/减的项（例如"加上一等区间"或"砍掉富插值"）？
3. **`float` 是否纳入 v1**：推荐纳入（应用如分形/几何/统计需要；Python 实现近乎零成本）；若想更精简可砍。
4. **应用选题**（P8）：见 `BRAINSTORM.md`，推荐「迷宫工坊 MazeLab」；最终由用户与 app-dev 在 P8 前确认。

---

## 五、下一步

1. 用户确认（或覆盖）上表四项 → 我同步更新 ADR / PROJECT_STATE / REQUIREMENTS。
2. 用户下达**开工令** → 团队执行 **P0.5（git init + 初始提交）**。
3. P1 正式标记完成 → **P2 启动**：language-architect 按本范围冻结 `docs/spec/` 三件套 + interface-contract。
