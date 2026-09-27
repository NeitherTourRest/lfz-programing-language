# test-engineer — 工作状态
> 最后更新: 2026-09-27 by test-engineer

## 当前状态
**P5.3 ✅ 完成**（黑盒测试第三批：`§10.7` 内置函数全表 **54 个**逐个覆盖）。
`cargo run --quiet -- test` → **82 PASS / 0 FAIL / 0 ERROR，exit 0（全绿）**。
`cargo build` → **0 warning / 0 error**；`cargo test` → **431 passed / 0 failed**。

## 进行中
- （无）

## 已完成
- **P5.3（2026-09-27）**：内置函数全表 54 个逐项覆盖（沿用 v1.2 runner 契约）
  - 正向：新增 **8** 文件 / **263** 条 `assert` —— `test_builtins_array`(70)、`test_builtins_higher_order`(28)、`test_builtins_struct`(26)、`test_builtins_string`(33)、`test_builtins_math`(43)、`test_builtins_convert`(39)、`test_builtins_random`(10)、`test_builtins_io`(14)。
  - 负例：新增 **23** 条 fixture，`cases.json` 扩至 **56** 负例 + 1 正向豁免（共 57 清单条目）。
  - 覆盖矩阵：`tests/coverage-matrix.md` 新增 **§2b 内置 54 个逐个对照表**（含用例文件 / 断言数 / 负例 / 状态 / 说明）+ §3 追加「`input` 由 Rust 单测覆盖」「内置名非一等值」「print/eprint/check 文本不可捕获」诚实标注 + §5.3 负例清单 + §6.3 正向清单 + §7 P5.3 证据 + §8 状态更新。
  - 证据：`cargo run --quiet -- test` → 82/82 PASS，exit 0；`cargo clean -p lfz; cargo build` → 0 warning / 0 error；`cargo test` → 431 passed / 0 failed。
  - **54 内置覆盖结论**：53 个由 LFZ 黑盒覆盖；唯 `input` 跳过（环境相关：runner 无 stdin，可能 EOF/阻塞），按任务授权注明**由 Rust 单测 `src/builtins.rs::tests::input_reads_line_crlf_and_eof` 覆盖**。
- **P5.2（2026-09-27）**：黑盒测试第二批（控制流 / 函数 / 闭包 / 结构体 / 管道 / 富插值 / `;;` / A1 / A6 / `RecursionError`），51/51 PASS。
- **P5.1（2026-09-27）**：`tests/` 黑盒测试基础设施，29/29 PASS。

## 阻塞 / 需要支持
- **缺陷单 bug-20260927-01（已提交 team-lead，未修）**：`p["方法"]()` 经方括号取值再调用时**未绑定 `self`** → `NameError`。本批未新增影响（`test_builtins_struct.lfz` 仍只用 `.get()` 调用方法，未用 `box["get"]()`）。修复后需补端到端用例并回归。
- （本批无新缺陷单。所有负例的实际错误类与 §8.1 期望**逐项吻合**，见 §5.3。）

## 下一步计划
- **P5.4**：值显示与相等语义全量；错误模型 12 类补齐 traceback 折叠 + `--json` 字段逐字符；`IOError`（非 input 路径，如不可读文件）。
- **P5.5**：可移植性（BOM/CRLF/LF/CR、非 UTF-8 → `SyntaxError`）、入口一致性（REPL/stdin/`-e` 豁免）。
- **回归**：bug-20260927-01 修复后补 `p["方法"]()` 端到端用例并复跑全量。

## 关键经验（写给未来的自己）
- **runner 契约**：默认根 = `tests/`；发现 `tests/**/*.lfz` 且按目录名排除 `fixtures`；每个被发现的 `.lfz` 首行必须恰为 `#42`；负例只能放 `tests/fixtures/` 并在 `tests/cases.json` 用 `expect.error`（§8.1 的 **12 类名之一**）声明。
- **判定（重要）**：清单声明 `expect.error` 且实际类名一致 → **PASS**（**含 `AssertionError`**：`fail` 用例在清单声明下判 PASS，不因「AssertionError → FAIL」规则而被记 FAIL）。本轮 `fail_raises` 已验证。
- **stdout/stderr 不可断言**：非 `--json` 下 `print`/`eprint`/`check`/`;;` 写 stdout/stderr，runner 不捕获程序输出 → 只断言返回值/不报错。
- **内置名非一等值**：`type(type)` / 把内置名作实参 → `NameError`；spec §10.7 未声明可作值（示例均传 lambda）。**不要写 `type(type)`**。
- **`reduce` 是左折叠** `f(acc, x)`，`x` 从左到右；`acc*x` 判序用 `acc*10+x`（=123）。注意 `push(v,xs)` 是**尾部追加**，`reduce((acc,x)=>push(x,acc),[],…)` 结果为 `[1,2,3]`（不是 `[3,2,1]`）。
- **LFZ 语法陷阱**：语句分隔只能是换行；条件位/可迭代位禁裸 struct 字面量；`let` 不可重绑定（循环外累加用模块/函数内 `var`）；闭包按 cell 捕获（`var total` + 内部 `fn` 累加在 `each` 中有效）。
- **环境**：PowerShell 下 `cargo` 写 stderr 会被包成 `NativeCommandError`，用 `2>&1 | Out-String` 再筛；批量写临时探针文件时用**单引号**PS 字符串避免 `"` 转义地狱；读完删干净临时文件（探针放 `%TEMP%\opencode`）。
