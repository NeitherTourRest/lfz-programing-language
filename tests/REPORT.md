# LFZ 黑盒测试集 — 测试报告（P5）

> 交付物：评分项 2（20 分）｜ 负责人: test-engineer ｜ 状态: **定稿（P5.4）+ 等价路径补测（P5.6）** ｜ 最后更新: 2026-09-27
> 测试对象: LFZ 解释器（`cargo run -- test` runner + `lfz` 解释器） ｜ 基线: P3 解释器核心 `v0.2.0`
> 事实源: `docs/spec/{syntax,semantics,interface-contract}.md`（冻结 v1） ｜ runner 契约: `docs/tooling/runner-contract.md`（v1.2）
> 配套文件: 覆盖矩阵 `tests/coverage-matrix.md`；用例 `tests/lfz/**/*.lfz` + `tests/fixtures/**` + 清单 `tests/cases.json`

---

## 1. 概述

本测试集是 LFZ 语言的**黑盒测试集**：只依赖 `docs/spec/` 声明的**公开语法与语义**判定解释器行为是否正确，**不读解释器实现代码**、**不修改解释器源码**。目标是证明 LFZ 解释器实现了 spec 声明的**全部语言特性**，且行为符合 spec。

**定稿结论**：`cargo run --quiet -- test` → **85 用例全绿（85 PASS / 0 FAIL / 0 ERROR，退出码 0）**；覆盖 `docs/spec/` 声明的语言特性全集、`interface-contract.md` §10.7 全表 **54** 个内置（53 黑盒 + 1 由 Rust 单测）、§8.1 **12** 个错误类（11 黑盒 + 1 由 Rust 单测）。**P5.6** 依 verifier《P9-verification.md》rev.2 §9.4 建议，为规范声明「两种写法等价」的路径补**双路**用例（专节见覆盖矩阵 §1.3）。

---

## 2. 测试方法

### 2.1 黑盒原则

- 只用 LFZ **语言本身**编写正向用例（`tests/lfz/**/*.lfz`，用内建 `assert`/`check` 表达断言）。
- 只依赖 spec 公开行为；spec 未定义处不自行假设（如溢出语义以 spec 补钉为准）。
- **不读解释器实现、不改解释器源码**；发现缺陷只出缺陷单（§7），由 core-dev / runtime-dev 修复。
- 每个用例可单独运行、互不依赖；用例文件之间不共享全局状态假设。

### 2.2 runner 契约（`runner-contract.md` v1.2，T-R1 … T-R4）

| 条目 | 内容 | 本测试集的遵循方式 |
|---|---|---|
| **T-R1** | 用例执行链与 `lfz run` 一致；缺/非法 `#42` → `CosmosAnswerError`（判 ERROR，退出码 2） | 所有 `.lfz` 首行恒为 `#42` |
| **T-R2** | 负例经 `cases.json` 的 `expect.error` 声明（取值必须是 §8.1 的 12 类名之一）；夹具放非自动发现处 | 58 负例放 `tests/fixtures/`，在 `tests/cases.json` 声明 |
| **T-R3** | 仅 `AssertionError`（`assert` 失败 / `fail`）判 FAIL；`check` 失败非致命（PASS）；`expect` 不符判 FAIL | 错误类断言以「类名一致」为准；`check` 用例断言非致命 |
| **T-R4** | 默认发现根 `tests/`；递归 `*.lfz`（大小写不敏感）；排除 `fixtures/`；结果稳定排序 | 26 个正向 `.lfz` 放自动发现域，58 负例放 `fixtures/` |

### 2.3 断言与判定口径

- **正向**：`assert(cond, msg)`（失败致命 → `AssertionError`）；`check(cond, msg)`（软检查，读返回值，非致命，A4）。
- **负例**：只断言**错误类**（§8.1 的 12 类名之一），**不逐字断言中文消息 / 行号 / 列号**（理由见 §6）。
- 判定与退出码（D-008 / 契约 §4–§5）见 §3.2。

### 2.4 每特性 ≥3 用例（正常 / 边界 / 错误）

以 `docs/spec/` 特性清单为基准逐特性建用例组：**正常**（happy path）、**边界**（空 / 极值 / 单元素 / 嵌套 / 快照 / 极端形态）、**错误**（期望抛错的负例）。命名：用例组文件 `test_<特性>.lfz`；断言消息 `<特性>_<场景>_<期望>`，保证失败可定位。详见覆盖矩阵 §1–§3。

---

## 3. 运行方式

### 3.1 命令

```bash
# 一键运行全部用例（在仓库根目录）
cargo run --quiet -- test
```

- 默认发现根 = cwd 下 `tests/`；递归收集 `tests/**/*.lfz`（**排除 `tests/fixtures/`**）；读取 `tests/cases.json` 声明负例。
- 每用例一行 `PASS/FAIL/ERROR <name>`；末尾汇总行逐字符稳定：`汇总：共 <N> 个用例，通过 <P>，失败 <F>，错误 <E>`。
  - 机器可读：`cargo run --quiet -- test --json` → **末行**为汇总 JSON（`ok/total/passed/failed/errored/cases`；本测试集实测 `{"ok":true,"total":85,"passed":85,"failed":0,"errored":0,...}`）。
  - **注意（已知工具链偏差，待 team-lead / tooling-dev 裁定）**：契约 §9.3 的输出重定向仅覆盖内建 `print` / `input` 提示，**未覆盖 `;;` dump 指令**——`;;` 文本仍写 stdout，含 `;;` 的用例（`test_dump.lfz`）会使 `--json` 的 stdout 出现**多行**（JSON 恒在**最后一行**）。此为 **runner/CLI 契约与实现间隙**（非黑盒测试集缺陷，`src/**` 未改）；机器消费建议取 stdout 最后一行。主要交付命令为**非 `--json`** 的 `cargo run --quiet -- test`，不受此影响。

### 3.2 退出码语义（`0` / `1` / `2`）

| 退出码 | 条件 |
|---|---|
| `0` | 全部用例 **PASS**（本测试集目标状态） |
| `1` | 无 `ERROR`，但存在 **FAIL**（测试失败：仅 `AssertionError` 且未在清单声明） |
| `2` | 存在 **ERROR**（任一非 `AssertionError` 未被清单声明的错误类，含缺 `#42`）；或 runner 环境错误（参数 / 缺目录 / 清单非法 / 未发现用例） |

### 3.3 环境

- 平台: Windows（win32）；Shell: PowerShell。**无第三方依赖**，仅需 Rust 工具链。
- 构建: `cargo build`；测试运行器为 CLI 内建 `lfz test` 子命令。

---

## 4. 用例总数与构成（**85**）

| 构成 | 数量 | 说明 |
|---|---|---|
| 正向自动发现 `.lfz` | **26 文件 / 597 `assert`** | `tests/lfz/**/*.lfz`，首行恒 `#42` |
| ├ 基础语言特性（P5.1） | 7 文件 / 114 | 字面量 / let-var / 算术 / 优先级 / 除法取模 / i64 边界 / `#42` |
| ├ 控制流·函数·闭包·结构体·管道·插值·`;;`·A1·A6（P5.2） | 10 文件 / 179 | 见覆盖矩阵 §1 |
| ├ 内置函数全表（P5.3） | 8 文件 / 263 | 见覆盖矩阵 §2 |
| └ 等价路径双路补测（P5.6） | 1 文件 / 41 | `test_equivalence_paths.lfz`；见覆盖矩阵 §1.3 |
| 负例清单 `expect.error` | **58** fixture | `tests/fixtures/**` + `tests/cases.json` |
| 正向豁免（非 `.lfz`） | **1**（`fixtures/plain_ok.txt`） | 非 `.lfz` 不要求 `#42`，判正常 PASS |
| **合计** | **85** | `cargo run --quiet -- test` → **85 PASS / 0 FAIL / 0 ERROR**，exit 0 |

**错误类分布**（`expect.error` 逐类计数，合计 58）：`CosmosAnswerError` 5、`SyntaxError` 11、`NameError` 1、`TypeError` 11、`IndexError` 6、`FieldError` 4、`ZeroDivisionError` 3、`OverflowError` 5、`ValueError` 10、`AssertionError` 1、`RecursionError` 1、`IOError` 0（见 §5.3 / §6）。

---

## 5. 覆盖矩阵摘要

> 完整矩阵见 `tests/coverage-matrix.md`。

### 5.1 语言特性覆盖

| 特性域 | 状态 | 证据 |
|---|---|---|
| 词法：int/float/string(+转义)/bool/nil 字面量、`#42` 前导严格性、i64 边界 | ✅ 通过 | 覆盖矩阵 §1.1（词法行） |
| 绑定：标识符、`let`/`var`、复合赋值 | ✅ 通过 | 同上 |
| 表达式：算术、比较、逻辑（含短路）、一元、优先级、`/`（float）、`div`、`%` | ✅ 通过 | 同上 |
| 语句：`if`/`else`（语句+表达式）、`while`、`for`（array/range）、`break`/`continue`、`return` | ✅ 通过 | 同上 |
| 函数：声明/调用/参数/返回值/递归/函数字面量/函数作值/同一性 | ✅ 通过 | 同上 |
| 闭包（A2）：按 cell 捕获、多实例/多闭包/循环每轮 | ✅ 通过 | 同上 |
| 结构体：字面量/模板/平拷贝/匿名/动态字段/`.k ≡ ["k"]`/方法 vs 数据面（A5）/`del` | ✅ 通过（`s.k≡s["k"]` **读/写/调用双路**；bug-20260927-01 已修复） | 同上（覆盖矩阵 §1.3） |
| 管道 `\|>`：data-last / `_` 占位 / 链式 / 组合 | ✅ 通过 | 同上 |
| 富字符串插值：多段/嵌套/格式说明符/`\$` | ✅ 通过 | 同上 |
| `;;` dump：作用域链/遮蔽去重 | ⚠️ 部分（执行不报错；文本见 §6） | 同上 |
| 引用语义（A1）：原地修改 + 内置返回新值 + 负索引 | ✅ 通过 | 同上 |
| 环安全（A6）：自引用容器 `==` 等价 | ✅ 通过（`<cycle>` 文本见 §6） | 同上 |
| **等价路径**：`s.k≡s["k"]`（读/写/调用）、复合赋值脱糖、管道 data-last 脱糖、块注释≡空格、`a--b`、短路 | ✅ 通过（两写法均有用例且结果一致） | 覆盖矩阵 §1.3（P5.6） |
| 错误模型（§8.1） | ⚠️ 部分（11/12 黑盒 + `IOError` 由 Rust 单测） | §5.3 / §6 |
| 可移植性 / 入口一致性 | ⏭ 待实现（P5.5） | 覆盖矩阵 §9 |

### 5.2 内置函数覆盖（§10.7 全表 **54** 个）

- **覆盖数：53 / 54**（正向断言 + `expect.error` 负例逐项覆盖）。
- **跳过数：1** —— `input`：runner 不提供 stdin 约定，`input()` 可能 EOF 或**阻塞**（不可复现）→ 跳过自动发现用例，**由 Rust 单测覆盖**（`src/builtins.rs::tests::input_reads_line_crlf_and_eof`，EOF→`IOError`）。
- 关键不变量：A1「返回新值不改原容器」在 10 个容器更新内置有 `*_does_not_mutate` 断言；A5 数据面在 `keys/values/entries/has/len/del` 断言排除方法字段；`data-last` 经 `test_pipe.lfz` 组合验证。
- 逐项表见覆盖矩阵 §2。

### 5.3 错误类覆盖（§8.1，**12** 个具体类 + 基类）

| 错误类 | 数量 | 状态 |
|---|---|---|
| `CosmosAnswerError` | 5 | ✅ 黑盒通过 |
| `SyntaxError` | 11 | ✅ 黑盒通过 |
| `NameError` | 1 | ✅ 黑盒通过 |
| `TypeError` | 11 | ✅ 黑盒通过 |
| `IndexError` | 6 | ✅ 黑盒通过 |
| `FieldError` | 4 | ✅ 黑盒通过（`del`×2 + 缺失键读 `.k`/`["k"]`×2） |
| `ZeroDivisionError` | 3 | ✅ 黑盒通过 |
| `OverflowError` | 5 | ✅ 黑盒通过 |
| `ValueError` | 10 | ✅ 黑盒通过 |
| `IOError` | 0 | ⏭ 黑盒跳过（环境相关）→ **Rust 单测覆盖**（见 §6） |
| `AssertionError` | 1 | ✅ 黑盒通过 |
| `RecursionError` | 1 | ✅ 黑盒通过 |
| `LfzError`（基类） | — | 不适用（基类不直接抛出，spec §8.1） |

> 结论：**11/12 类黑盒覆盖；`IOError` 由 Rust 单测覆盖**（任务授权允许）。逐类证据（fixture → manifest 名）见覆盖矩阵 §3 / §5。

---

## 6. 不可断言项与理由

runner 契约 §6 / §9.3：非 `--json` 模式下程序 `print` / `;;` 写 stdout、`check` 警告写 stderr，runner **不捕获程序 stdout**；`--json` 下程序输出重定向到 stderr，但 JSON 仅含 verdict / 错误字段。故下列项**无法在黑盒 LFZ `assert` 中逐字判定**，采用「诚实标注 + 保底不报错 + 注明替代覆盖者」：

| # | 不可断言项 | 为什么测不到 | 由谁覆盖 |
|---|---|---|---|
| 1 | `;;` dump 的**输出文本**（行格式、内→外顺序、遮蔽去重） | `;;` 写程序 stdout，runner 不捕获；`--json` 下亦不重定向（契约 §9.3 仅覆盖 `print`/`input` 提示，见 §3.1 注意） | 黑盒保底「执行不报错」+ Rust 单测渲染逻辑 + 带外人工观察 |
| 2 | 自引用容器的 **`<cycle>` 渲染字面量** | 写 stdout；渲染文本非可返回值 | 黑盒 `==` 环等价 + Rust 单测显示渲染 |
| 3 | 错误用例的**中文消息文本** | 消息措辞无契约稳定性保证；黑盒无读取消息通道 | 黑盒只断言**错误类**；Rust 单测逐条断言消息（`src/error.rs::tests`） |
| 4 | 错误的**行号 / 列号**（span） | LFZ 程序内无反射自身 span 能力 | Rust 单测（`src/error.rs`）+ CLI e2e（`tests/cli.rs`） |
| 5 | `input()` 的端到端行为 | runner 不提供 stdin；`input()` 可能 EOF 或**阻塞** | **跳过**；Rust 单测 `src/builtins.rs::tests::input_reads_line_crlf_and_eof` |
| 6 | 程序 **stdout 捕获 / 逐字比对**（`print`/`eprint`/`check` 警告） | 契约 §6：程序输出不捕获 | 黑盒仅断言返回值/不报错；CLI e2e `tests/cli.rs::json_run_print_*` |
| 7 | 内置名作**一等值**（`type(type)`） | spec §10.7 未声明（示例均传 lambda） | 不作断言（仅 `type(<lambda>)=="function"`） |
| 8 | `traceback` **折叠** / `--json` 字段**逐字符** | 属 runner/CLI 输出契约，非语言内可观察量 | Rust 单测 + CLI e2e |

---

## 7. 已知缺陷

| 缺陷单 | 现象 | 严重度 | 影响用例 | 状态 |
|---|---|---|---|---|
| **bug-20260927-01** | 经方括号取得的方法值再调用时不绑定 `self`：`p["norm2"]()` → `NameError`（`.字段()` 调用点正常） | 中 | `test_structs.lfz` 中「方括号调用方法」未断言 | ✅ **已修复**（commit `6aabdf5`）→ **P5.6 补端到端用例并回归**：`struct_method_call_bracket` / `method_bracket_mutates_same_instance` + `equiv_method_two_call_forms_equal` / `equiv_self_mutation_*` |

> 缺陷修复前，对应用例在矩阵中标注为「未断言 / 阻塞」，**不删除、不放宽**任何已有断言。本批为修复**后**补测，属新增断言，未改动既有断言。

---

## 8. 如何复现（可复制命令）

在仓库根目录（Windows PowerShell）执行：

```powershell
# 0) 确保 Rust 工具链在 PATH（如未配置）
$env:Path += ";$env:USERPROFILE\.cargo\bin"

# 1) 构建（应 0 warning / 0 error）
cargo build

# 2) 一键运行全部黑盒用例（应 85 PASS / 0 FAIL / 0 ERROR，退出码 0）
cargo run --quiet -- test
#   查看退出码：
#   echo $LASTEXITCODE     # → 0

# 3) 机器可读输出（选做；stdout 为唯一一行 JSON）
cargo run --quiet -- test --json

# 4) 解释器单元/集成测试回归（应 432 passed / 0 failed）
cargo test
```

**预期输出（末尾汇总行，逐字符稳定）**：

```
汇总：共 85 个用例，通过 85，失败 0，错误 0
```

---

## 9. 回归记录

| 日期 | 批次 | 命令 | 结果 |
|---|---|---|---|
| 2026-09-27 | P5.1 | `cargo run --quiet -- test` | 29/29 PASS，exit 0 |
| 2026-09-27 | P5.2 | `cargo run --quiet -- test` | 51/51 PASS，exit 0 |
| 2026-09-27 | P5.3 | `cargo run --quiet -- test` | 82/82 PASS，exit 0；`cargo test` 431 passed |
| 2026-09-27 | **P5.4（定稿）** | `cargo clean -p lfz; cargo build` | **0 warning / 0 error** |
| 2026-09-27 | **P5.4（定稿）** | `cargo run --quiet -- test` | **82 PASS / 0 FAIL / 0 ERROR，exit 0** |
| 2026-09-27 | **P5.4（定稿）** | `cargo test` | **431 passed / 0 failed / 0 ignored**（361+42+16+12） |
| 2026-09-27 | **P5.6（等价路径补测）** | `cargo clean -p lfz; cargo build` | **0 warning / 0 error** |
| 2026-09-27 | **P5.6（等价路径补测）** | `cargo run --quiet -- test` | **85 PASS / 0 FAIL / 0 ERROR，exit 0**（+3 用例） |
| 2026-09-27 | **P5.6（等价路径补测）** | `cargo test` | **432 passed / 0 failed / 0 ignored**（lib 362+42+16+12；`src/**` 未改） |

> 解释器每次变更后，本测试集须复跑全量并把结果追加到本表；回归结果同时作为 verifier 独立验收（P9）的输入。
