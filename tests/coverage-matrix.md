# LFZ 黑盒测试 — 覆盖矩阵

> 唯一写者: test-engineer ｜ 最后更新: 2026-09-27（P5.1 基础语言特性批次）
> 事实源: `docs/spec/{syntax,semantics,interface-contract}.md`（冻结 v1） ｜ runner 契约: `docs/tooling/runner-contract.md`（v1.2）
> 运行命令: `cargo run --quiet -- test`（默认发现 `tests/**/*.lfz`，排除 `fixtures/`）

## 1. 本批（P5.1）覆盖矩阵

| 特性 | 用例组文件 | 正常用例数 | 边界用例数 | 错误用例数 | 状态 | 证据（运行命令与结果摘要） |
|---|---|---|---|---|---|---|
| 字面量 `int`（十进制 / 前缀 / 下划线 / 显示） | `tests/lfz/test_literals.lfz` | 6 | 4 | 0 | 通过 | `cargo run --quiet -- test` → 29/29 PASS，exit 0 |
| 字面量 `float`（小数点 / 指数 / 整值显示 `.0`） | `tests/lfz/test_literals.lfz` | 5 | 3 | 0 | 通过 | 同上 |
| 字面量 `string` 与转义 `\n \t \r \\ \" \e \$` | `tests/lfz/test_literals.lfz` | 3 | 11 | 0 | 通过 | 同上 |
| 字面量 `bool` | `tests/lfz/test_literals.lfz` | 5 | 0 | 0 | 通过 | 同上 |
| 字面量 `nil` | `tests/lfz/test_literals.lfz` | 3 | 0 | 0 | 通过 | 同上 |
| 标识符 + `let` / `var` + 复合赋值 | `tests/lfz/test_let_var.lfz` | 6 | 6 | 2 | 通过 | 同上；负例 `let_rebind` / `let_compound_rebind` → `TypeError` |
| 算术 `+` `-` `*` | `tests/lfz/test_arithmetic.lfz` | 4 | 3 | 2 | 通过 | 同上；负例 `operator_type_mismatch`(TypeError) / `int_overflow_add`(OverflowError) |
| 比较 `== != < <= > >=` | `tests/lfz/test_arithmetic.lfz` | 6 | 0 | 0 | 通过 | 同上 |
| 逻辑 `&& \|\| !`（含短路） | `tests/lfz/test_arithmetic.lfz` | 7 | 2 | 0 | 通过 | 同上；短路用例内嵌 `1 / 0` 验证右操作数未求值 |
| 一元 `-` | `tests/lfz/test_arithmetic.lfz` | 2 | 0 | 0 | 通过 | 同上 |
| 优先级与结合性 | `tests/lfz/test_precedence.lfz` | 8 | 3 | 0 | 通过 | 同上 |
| `/` 恒为 `float` + `div(a,b)` 向下取整 | `tests/lfz/test_division_modulo.lfz` | 9 | 0 | 2 | 通过 | 同上；负例 `divide_by_zero` / `builtin_div_by_zero` → `ZeroDivisionError` |
| `%` 符号随除数（Python 取模） | `tests/lfz/test_division_modulo.lfz` | 4 | 1 | 1 | 通过 | 同上；负例 `modulo_by_zero` → `ZeroDivisionError` |
| `i64` 边界（`i64::MIN` / `i64::MAX` / 溢出） | `tests/lfz/test_int_min.lfz` | 3 | 5 | 2 | 通过 | 同上；负例 `int_literal_exceeds_i64`(SyntaxError) / `abs_i64_min_overflow`(OverflowError) |
| `#42` 前导严格性（缺 / 尾随空格 / Tab / 无换行 / 空文件） | `tests/lfz/test_preamble.lfz` + `tests/fixtures/*` | 1 | 4 | 5 | 通过 | 同上；负例均 → `CosmosAnswerError` |
| `#42` 豁免（非 `.lfz` 文件不要求前导） | `tests/fixtures/plain_ok.txt` | 2 | 0 | 0 | 通过 | 同上；`non_lfz_file_no_preamble_required` PASS |
| 语法错误模型（单 `;` / `#` 越位 / `_` 绑定 / 循环外 `break` / 非法字符 / 未闭合括号） | `tests/fixtures/*` | 0 | 0 | 6 | 通过 | 同上；6 个 fixture 均 → `SyntaxError` |
| 名字 / 类型 / 除零 / 溢出错误模型 | `tests/fixtures/*` | 0 | 0 | 7 | 通过 | 同上；`NameError`×1、`TypeError`×2、`ZeroDivisionError`×3、`OverflowError`×2（含上表重叠计数） |

> 计数口径：**正常** = happy path；**边界** = 空/极值/转义/短路/前缀进制的极端形态；**错误** = 期望抛错的负例（经 `tests/cases.json` 的 `expect.error` 声明，判为 PASS）。同一负例可服务多个特性行，故错误列在跨行处存在重复计数——**去重后负例总数以 §3 清单为准**。

## 2. 本批用例清单

### 2.1 正向用例（自动发现域 `tests/lfz/**/*.lfz`，首行恒 `#42`）

| 文件 | 用例名（runner 默认 = 路径） | 断言数 | 覆盖特性 |
|---|---|---|---|
| `tests/lfz/test_literals.lfz` | `tests/lfz/test_literals.lfz` | 40 | int/float/string(转义)/bool/nil |
| `tests/lfz/test_let_var.lfz` | `tests/lfz/test_let_var.lfz` | 12 | 标识符、let/var、复合赋值、A1 |
| `tests/lfz/test_arithmetic.lfz` | `tests/lfz/test_arithmetic.lfz` | 24 | 算术/比较/逻辑/一元/短路 |
| `tests/lfz/test_precedence.lfz` | `tests/lfz/test_precedence.lfz` | 11 | 优先级与结合性 |
| `tests/lfz/test_division_modulo.lfz` | `tests/lfz/test_division_modulo.lfz` | 14 | `/` `div` `%`（A3） |
| `tests/lfz/test_int_min.lfz` | `tests/lfz/test_int_min.lfz` | 8 | i64 边界（B12） |
| `tests/lfz/test_preamble.lfz` | `tests/lfz/test_preamble.lfz` | 5 | `#42` 与 `#` 的词法边界 |

### 2.2 负例与豁免夹具（非自动发现，经 `tests/cases.json` 声明）

`fixtures/` 目录被 runner 按名排除（`runner-contract.md` §2.3），故所有负例**不污染**正向集；`.txt` 夹具按扩展名豁免 `#42`（§2.6）。

## 3. 负例清单（`tests/cases.json`，共 21 条 `expect.error` + 1 条正向豁免）

| # | fixture 路径 | manifest 名 | 期望错误类 | 覆盖特性 |
|---|---|---|---|---|
| 1 | `fixtures/missing_preamble.lfz` | `preamble_missing_first_line` | `CosmosAnswerError` | `#42` 缺失 |
| 2 | `fixtures/preamble_trailing_space.lfz` | `preamble_trailing_space` | `CosmosAnswerError` | `#42 ` 尾随空格 |
| 3 | `fixtures/preamble_trailing_tab.lfz` | `preamble_trailing_tab` | `CosmosAnswerError` | `#42\t` 尾随 Tab |
| 4 | `fixtures/preamble_no_newline.lfz` | `preamble_no_newline_eof` | `CosmosAnswerError` | `#42` 无行终止符（EOF） |
| 5 | `fixtures/empty.lfz` | `preamble_empty_file` | `CosmosAnswerError` | 空 `.lfz` 文件 |
| 6 | `fixtures/single_semi.lfz` | `syntax_single_semicolon` | `SyntaxError` | 单个 `;` |
| 7 | `fixtures/hash_mid_program.lfz` | `syntax_hash_mid_program` | `SyntaxError` | 程序中部 `#` |
| 8 | `fixtures/placeholder_binding.lfz` | `syntax_placeholder_in_binding` | `SyntaxError` | `_` 出现在绑定位 |
| 9 | `fixtures/break_outside_loop.lfz` | `syntax_break_outside_loop` | `SyntaxError` | 循环外 `break` |
| 10 | `fixtures/syntax_illegal_char.lfz` | `syntax_illegal_character` | `SyntaxError` | 非法字符 `$` |
| 11 | `fixtures/syntax_unclosed_paren.lfz` | `syntax_unclosed_paren` | `SyntaxError` | 未闭合括号 |
| 12 | `fixtures/int_literal_overflow.lfz` | `int_literal_exceeds_i64` | `SyntaxError` | `9223372036854775808` |
| 13 | `fixtures/let_rebind.lfz` | `let_rebind_is_type_error` | `TypeError` | `let` 重绑定 |
| 14 | `fixtures/let_compound_rebind.lfz` | `let_compound_rebind_is_type_error` | `TypeError` | `let` 复合赋值 |
| 15 | `fixtures/type_error_add.lfz` | `operator_type_mismatch` | `TypeError` | `int + string` |
| 16 | `fixtures/name_error.lfz` | `undefined_name` | `NameError` | 未定义名字 |
| 17 | `fixtures/zero_division.lfz` | `divide_by_zero` | `ZeroDivisionError` | `1 / 0` |
| 18 | `fixtures/mod_zero.lfz` | `modulo_by_zero` | `ZeroDivisionError` | `1 % 0` |
| 19 | `fixtures/div_zero.lfz` | `builtin_div_by_zero` | `ZeroDivisionError` | `div(1, 0)` |
| 20 | `fixtures/overflow_add.lfz` | `int_overflow_add` | `OverflowError` | `i64::MAX + 1` |
| 21 | `fixtures/abs_i64_min.lfz` | `abs_i64_min_overflow` | `OverflowError` | `abs(i64::MIN)` |
| — | `fixtures/plain_ok.txt` | `non_lfz_file_no_preamble_required` | （无，正向） | 非 `.lfz` 豁免 `#42` |

## 4. 运行证据（P5.1）

```
$ cargo run --quiet -- test
PASS  abs_i64_min_overflow
PASS  syntax_break_outside_loop
...（共 29 行 PASS）...
PASS  tests/lfz/test_precedence.lfz

汇总：共 29 个用例，通过 29，失败 0，错误 0
$ echo $LASTEXITCODE
0
```

- **用例总数：29** = 正向自动发现 **7** + 清单（负例 21 + 非 `.lfz` 正向豁免 1）。
- 退出码 **0**（全绿）。
- 构建：`cargo build` **0 warning**；`cargo test` 全绿（不破坏既有集成用例）。

## 5. 后续批次待覆盖特性（骨架，P5.2+ 逐批填满）

> 本表为 **`docs/spec/` 特性全集对照**；`状态` 取值：`通过` / `待实现（spec 已定，未测）` / `失败（缺陷单编号）` / `阻塞（解释器未实现）`。

| 特性域 | spec 依据 | 状态 | 计划批次 |
|---|---|---|---|
| 字面量 / 标识符 / let-var / 算术 / 比较 / 逻辑 / 优先级 / 除法取模 / i64 边界 / `#42` | syntax §2、§4、§6；semantics §4.2 | **通过**（本批） | P5.1 ✅ |
| 字符串插值 `${}` 与格式说明符（含非法说明符 → `ValueError`/`TypeError`） | syntax §2.8、§7；semantics §3.7 | 待实现 | P5.2 |
| 数组：字面量、索引（含负索引）、越界、切片、元素赋值、快照迭代 | syntax §7；semantics §4.5.4 | 待实现 | P5.2 |
| struct：字面量/模板/实例化、`.字段`、`["键"]`、方法/`self`、动态字段、数据面 | syntax §3.3、§7；semantics §4.5.8/§4.5.9 | 待实现 | P5.2 |
| 控制流：`if/else`、`while`、`for`、`break`/`continue`、`return` | syntax §3、§7；semantics §4.5.1 | 待实现 | P5.3 |
| 函数：声明、参数、递归、闭包按 cell 捕获、lambda、`RecursionError` | syntax §7；semantics §4.5.3/§4.5.5 | 待实现 | P5.3 |
| 管道 `\|>` 与 `_` 占位符（脱糖、data-last、多 `_` → `SyntaxError`、λ 内 `_` 非法） | syntax §4.3/§4.4（A24/M4） | 待实现 | P5.3 |
| `;;` dump（作用域链、内→外、slot 升序、遮蔽去重、通道=stdout） | semantics §3.6 | 待实现 | P5.3 |
| 内置函数 54 个（array/struct/string/math/转换/IO/断言，data-last） | interface-contract §10.7 | 待实现 | P5.4 |
| 值显示形式（嵌套引号、struct 键字节序、`<cycle>` 环安全） | semantics §3.7/§4.5.9 | 待实现 | P5.4 |
| 相等语义（深结构相等、环安全、function 同一性） | semantics §4.5.9（A6） | 待实现 | P5.4 |
| 错误模型全量（12 类 + traceback 折叠 + `--json` 字段逐字符） | semantics §8；interface-contract §8.1 | 部分（本批 5 类） | P5.4 |
| 可移植性（BOM、CRLF/LF/CR、非 UTF-8 → `SyntaxError`） | syntax §2.1、§6-B4/B7/B8 | 待实现 | P5.5 |
| 入口一致性（REPL / stdin / `-e` 豁免前导；伪路径） | syntax §6-B9 | 待实现 | P5.5 |
