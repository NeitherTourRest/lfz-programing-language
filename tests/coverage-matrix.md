# LFZ 黑盒测试 — 覆盖矩阵（定稿）

> 唯一写者: test-engineer ｜ 状态: **定稿（P5.4）** ｜ 最后更新: 2026-09-27
> 事实源: `docs/spec/{syntax,semantics,interface-contract}.md`（冻结 v1） ｜ runner 契约: `docs/tooling/runner-contract.md`（v1.2）
> 断言方式: LFZ 内建 `assert(cond, msg)` / `check(cond, msg)`；负例经 `tests/cases.json` 的 `expect.error` 声明（§8.1 的 12 类名之一）
> 运行命令: `cargo run --quiet -- test`（默认发现 `tests/**/*.lfz`，排除 `fixtures/`）

---

## 0. 定稿总览

### 0.1 用例构成（合计 **82**）

| 构成 | 数量 | 说明 |
|---|---|---|
| 正向自动发现 `.lfz` | **25 文件 / 547 `assert`** | `tests/lfz/**/*.lfz`；首行恒 `#42`；见 §1.1 / §6 |
| ├ P5.1 基础语言特性 | 7 文件 / 114 | `test_literals` `test_let_var` `test_arithmetic` `test_precedence` `test_division_modulo` `test_int_min` `test_preamble` |
| ├ P5.2 控制流/函数/闭包/结构体/管道/插值/`;;`/A1/A6 | 10 文件 / 170 | `test_if_else` `test_loops` `test_functions` `test_closures` `test_structs` `test_pipe` `test_interpolation` `test_dump` `test_reference_semantics` `test_cycle_safety` |
| └ P5.3 内置函数全表 | 8 文件 / 263 | `test_builtins_{array,higher_order,struct,string,math,convert,random,io}` |
| 负例清单 `expect.error` | **56** fixture | `tests/fixtures/**`；逐个见 §5 |
| 正向豁免（非 `.lfz`） | **1**（`fixtures/plain_ok.txt`） | 清单无 `expect`，判正常 PASS（非 `.lfz` 不要求前导） |
| **合计** | **82** | `cargo run --quiet -- test` → **82 PASS / 0 FAIL / 0 ERROR**，exit 0 |

### 0.2 判定口径 / 退出码（runner 契约 §4/§5，D-008）

| 情形 | 判定 |
|---|---|
| 全链路无任何 `LfzError` | **PASS** |
| `check(cond)` 返回 `false`（非致命，A4） | **PASS**（警告写 stderr，不改退出码） |
| 清单声明 `expect.error` 且实际类名一致 | **PASS**（负例） |
| 仅 `AssertionError`（`assert` 失败 / `fail`）且**未**在清单声明 | **FAIL** |
| 其余任一错误类且**未**在清单声明（含缺 `#42` → `CosmosAnswerError`） | **ERROR** |
| 退出码 | `0` 全 PASS；`1` 有 FAIL 无 ERROR；`2` 有 ERROR / 环境错误 |

### 0.3 覆盖结论

- **语言特性**：词法 / 表达式 / 语句 / 函数 / 闭包 / 结构体 / 管道 / 富插值 / `;;` / 引用语义（A1）/ 环安全（A6）等 `docs/spec/` 声明的语言特性**均有用例**（见 §1.1 / §1.2）。
- **内置函数**：`interface-contract.md` §10.7 全表 **54** 个 → **53 个**由 LFZ 黑盒覆盖，**1 个（`input`）跳过**并由 Rust 单测覆盖（见 §2 / §4）。
- **错误模型**：§8.1 **12 个具体错误类** → **11 个**由黑盒负例覆盖，**1 个（`IOError`）** 由 Rust 单测覆盖（见 §3 / §4）；基类 `LfzError` 不直接抛出（spec §8.1），无触发用例。
- **不可断言项**：`;;` 输出文本、错误消息文本、行号/列号、`input`、程序 stdout 捕获等 —— 逐条在 §4 给出「为什么测不到 + 由谁覆盖」。

---

## 1. 特性覆盖表（语言特性）

### 1.1 特性 × 用例组（P5.1 + P5.2 合并）

> 计数口径：**正常** = happy path；**边界** = 空 / 极值 / 单元素 / 嵌套 / 快照 / 极端形态；**错误** = 期望抛错的负例（经 `tests/cases.json` 的 `expect.error` 声明，判 PASS）。同一负例服务多个特性行时存在跨行重复计数——**去重后负例总数以 §5 为准（56）**。

| 域 | 特性 | 用例组文件 | 正常 | 边界 | 错误 | 状态 | 证据 |
|---|---|---|---|---|---|---|---|
| 词法 | `int` 字面量（十进制 / 前缀 / 下划线 / 显示） | `tests/lfz/test_literals.lfz` | 6 | 4 | 0 | 通过 | `cargo run --quiet -- test` → 82/82 PASS，exit 0 |
| 词法 | `float` 字面量（小数点 / 指数 / 整值显示 `.0`） | `tests/lfz/test_literals.lfz` | 5 | 3 | 0 | 通过 | 同上 |
| 词法 | `string` 字面量与转义 `\n \t \r \\ \" \e \$` | `tests/lfz/test_literals.lfz` | 3 | 11 | 0 | 通过 | 同上 |
| 词法 | `bool` | `tests/lfz/test_literals.lfz` | 5 | 0 | 0 | 通过 | 同上 |
| 词法 | `nil` | `tests/lfz/test_literals.lfz` | 3 | 0 | 0 | 通过 | 同上 |
| 词法 | `#42` 前导严格性（缺 / 尾随空格 / Tab / 无换行 / 空文件） | `tests/lfz/test_preamble.lfz` + `tests/fixtures/*` | 1 | 4 | 5 | 通过 | 同上；5 负例均 → `CosmosAnswerError` |
| 词法 | `#42` 豁免（非 `.lfz` 文件不要求前导） | `tests/fixtures/plain_ok.txt` | 2 | 0 | 0 | 通过 | 同上；`non_lfz_file_no_preamble_required` PASS |
| 词法 | `i64` 边界（`i64::MIN` / `i64::MAX` / 溢出） | `tests/lfz/test_int_min.lfz` | 3 | 5 | 2 | 通过 | 同上；负例 `int_literal_exceeds_i64`(SyntaxError) / `abs_i64_min_overflow`(OverflowError) |
| 绑定 | 标识符 + `let` / `var` + 复合赋值 | `tests/lfz/test_let_var.lfz` | 6 | 6 | 2 | 通过 | 同上；负例 `let_rebind` / `let_compound_rebind` → `TypeError` |
| 表达式 | 算术 `+` `-` `*` | `tests/lfz/test_arithmetic.lfz` | 4 | 3 | 2 | 通过 | 同上；负例 `operator_type_mismatch`(TypeError) / `int_overflow_add`(OverflowError) |
| 表达式 | 比较 `== != < <= > >=` | `tests/lfz/test_arithmetic.lfz` | 6 | 0 | 0 | 通过 | 同上 |
| 表达式 | 逻辑 `&& \|\| !`（含短路） | `tests/lfz/test_arithmetic.lfz` | 7 | 2 | 0 | 通过 | 同上；短路用例内嵌 `1 / 0` 验证右操作数未求值 |
| 表达式 | 一元 `-` | `tests/lfz/test_arithmetic.lfz` | 2 | 0 | 0 | 通过 | 同上 |
| 表达式 | 优先级与结合性 | `tests/lfz/test_precedence.lfz` | 8 | 3 | 0 | 通过 | 同上 |
| 表达式 | `/` 恒为 `float` + `div(a,b)` 向下取整 | `tests/lfz/test_division_modulo.lfz` | 9 | 0 | 2 | 通过 | 同上；负例 `divide_by_zero` / `builtin_div_by_zero` → `ZeroDivisionError` |
| 表达式 | `%` 符号随除数（Python 取模） | `tests/lfz/test_division_modulo.lfz` | 4 | 1 | 1 | 通过 | 同上；负例 `modulo_by_zero` → `ZeroDivisionError` |
| 语句 | `if` / `else`（语句形态 + 表达式形态 + `else if` 链 + 嵌套 + 作实参） | `tests/lfz/test_if_else.lfz` | 9 | 2 | 1 | 通过 | 同上；负例 `if_condition_not_bool` → `TypeError`（无 truthiness） |
| 语句 | `while` / `for … in`（array、range）/ `break` / `continue` | `tests/lfz/test_loops.lfz` | 11 | 5 | 2 | 通过 | 同上；负例 `for_over_string`(TypeError) / `syntax_continue_outside_loop`(SyntaxError) |
| 语句 | `for` 迭代快照（改长度不改次数、元素内部改动可见）+ struct 键 UTF-8 字节序 | `tests/lfz/test_loops.lfz` | 3 | 2 | 0 | 通过 | 同上；`'Z'(0x5A) < 'a'(0x61)` 用例验证字节序 |
| 函数 | `fn`：声明 / 调用 / 参数 / 返回值（显式·隐式·提前）/ 无参 / 函数字面量 | `tests/lfz/test_functions.lfz` | 20 | 5 | 1 | 通过 | 同上 |
| 函数 | 递归（阶乘 / 斐波那契）+ 函数作值（type / 显示 / 同一性） | `tests/lfz/test_functions.lfz` | 2 | 6 | 1 | 通过 | 同上；负例 `deep_recursion` → `RecursionError` |
| 函数 | `RecursionError`（深递归 > 10000 帧） | `tests/fixtures/deep_recursion.lfz` | 0 | 0 | 1 | 通过 | 同上；`expect.error = "RecursionError"` |
| 闭包 | 闭包（A2）：counter 共享 cell / 多实例独立 / 多闭包共享 / 循环每轮独立 cell / 嵌套 / 捕获形参 | `tests/lfz/test_closures.lfz` | 11 | 3 | 0 | 通过 | 同上 |
| 结构体 | struct：字面量 / 模板默认值 / 平拷贝（可变默认值不共享）/ 匿名 / 动态字段 | `tests/lfz/test_structs.lfz` | 12 | 5 | 0 | 通过 | 同上 |
| 结构体 | struct 字段 `.k` ≡ `["k"]`（**数据面**） | `tests/lfz/test_structs.lfz` | 3 | 0 | 0 | 通过 | 同上；`p.x == p["x"]` |
| 结构体 | 方法字段 vs 数据面（A5）：`keys/values/entries/has/len/显示` 均不含方法字段 | `tests/lfz/test_structs.lfz` | 7 | 0 | 0 | 通过 | 同上 |
| 结构体 | `del` 仅作用数据面（删数据字段成功；删方法字段 → `FieldError`；删缺失键 → `FieldError`） | `tests/lfz/test_structs.lfz` + `tests/fixtures/*` | 3 | 0 | 2 | 通过 | 同上；负例 `del_method_field` / `del_missing_data_field` → `FieldError` |
| 管道 | `\|>`：data-last 注入 / `_` 占位符 / 链式 / 与 `map/filter/reduce/sum/take/slice` 组合 / 优先级 | `tests/lfz/test_pipe.lfz` | 11 | 3 | 3 | 通过 | 同上；负例 `pipe_rhs_not_function`(TypeError)、`pipe_multiple_placeholders`(SyntaxError)、`pipe_underscore_in_lambda`(SyntaxError) |
| 插值 | 富字符串插值：多段 / 嵌套字符串 / `format_spec`（`:>3` `:<4` `:05d` `:.2f` `:x` `:X`）/ `\$` 转义 | `tests/lfz/test_interpolation.lfz` | 10 | 6 | 2 | 通过 | 同上；负例 `interp_bad_format`(ValueError)、`interp_format_type_mismatch`(TypeError) |
| 调试 | `;;` dump（顶层 / 函数 / 块作用域 / 遮蔽去重） | `tests/lfz/test_dump.lfz` | 5 | 2 | 0 | 通过（**文本未断言**，见 §4） | 同上；执行到 `;;` 不报错 |
| 引用 | A1 引用语义：`a[i]=v` / `s.k=v` 原地可见；`push/pop/removeAt/insert/swap/slice/sort/map/filter/del` 返回新值不改原容器；负索引 | `tests/lfz/test_reference_semantics.lfz` | 20 | 9 | 0 | 通过 | 同上 |
| 引用 | A6 环安全：自引用 struct / array 的 `==` 等价（身份优先 + 重访即相等 + 不死循环） | `tests/lfz/test_cycle_safety.lfz` | 5 | 4 | 0 | 通过（**`<cycle>` 文本未断言**，见 §4） | 同上；`==` 可断言 |
| 错误 | 语法错误模型（单 `;` / `#` 越位 / `_` 绑定 / 循环外 `break` / 非法字符 / 未闭合括号） | `tests/fixtures/*` | 0 | 0 | 6 | 通过 | 同上；6 个 fixture 均 → `SyntaxError` |
| 错误 | 名字 / 类型 / 除零 / 溢出错误模型 | `tests/fixtures/*` | 0 | 0 | 7 | 通过 | 同上；`NameError`×1、`TypeError`×2、`ZeroDivisionError`×3、`OverflowError`×2 |

### 1.2 特性域状态总览（`docs/spec/` 全集对照）

> `状态` 取值：`通过` / `部分` / `待实现（spec 已定，未测）` / `失败（缺陷单编号）` / `阻塞（解释器未实现）`。

| 特性域 | spec 依据 | 状态 | 备注 |
|---|---|---|---|
| 字面量 / 标识符 / let-var / 算术 / 比较 / 逻辑 / 优先级 / 除法取模 / i64 边界 / `#42` | syntax §2、§4、§6；semantics §4.2 | **通过** | P5.1 ✅ |
| 控制流：`if/else`、`while`、`for`、`break`/`continue`、`return` | syntax §3、§7；semantics §4.5.1 | **通过** | P5.2 ✅ |
| 函数：声明、参数、递归、闭包按 cell 捕获、lambda、`RecursionError` | syntax §7；semantics §4.5.3/§4.5.5 | **通过** | P5.2 ✅ |
| 结构体：字面量/模板/实例化、`.字段`、`["键"]`、方法/`self`、动态字段、数据面（A5）、`del` | syntax §3.3、§7；semantics §4.5.8/§4.5.9 | **部分**（`["方法"]()` 绑定 `self` 见 §7 缺陷单 bug-20260927-01，未修前不作断言） | P5.2 ✅ |
| 管道 `\|>` 与 `_` 占位符（脱糖、data-last、多 `_` → `SyntaxError`、λ 内 `_` 非法） | syntax §4.3/§4.4（A24/M4） | **通过** | P5.2 ✅ |
| 字符串插值 `${}` 与格式说明符（含非法说明符 → `ValueError`、类型不符 → `TypeError`） | syntax §2.8、§7；semantics §3.7 | **通过** | P5.2 ✅ |
| 字符串/数组 `for` 迭代快照、struct 键字节序 | semantics §4.5.4 | **通过** | P5.2 ✅ |
| A1 引用语义（原地修改 + 内置返回新值） | semantics §4.5.2/§4.5.11 | **通过** | P5.2 ✅ |
| A6 环安全（`==`；`<cycle>` 渲染文本不可断言） | semantics §4.5.9 | **通过**（文本见 §4） | P5.2 ✅ |
| `;;` dump（作用域链、内→外、slot 升序、遮蔽去重、通道=stdout） | semantics §3.6 | **部分**（执行不报错已覆盖；**文本通道不可端到端断言**，见 §4） | P5.2 ✅ / 文本待工具支持 |
| 数组：字面量、索引（含负索引）、越界、切片、元素赋值、快照迭代 | syntax §7；semantics §4.5.4 | **通过** | P5.2 ✅ / P5.3 ✅ |
| 内置函数 54 个（array/struct/string/math/转换/IO/断言，data-last） | interface-contract §10.7 | **通过**（53 个黑盒；`input` 由 Rust 单测，见 §2/§4） | P5.3 ✅ |
| 值显示形式（嵌套引号、struct 键字节序、`<cycle>` 环安全） | semantics §3.7/§4.5.9 | **部分**（struct/匿名/标量显示已测；`<cycle>` 文本见 §4） | P5.3 ✅ / 文本见 §4 |
| 相等语义（深结构相等、环安全、function 同一性） | semantics §4.5.9（A6） | **部分**（标量/混合比较、环安全、function 同一性已测；其余见 §4） | P5.2/P5.3 ✅ |
| 错误模型全量（12 类 + traceback 折叠 + `--json` 字段逐字符） | semantics §8；interface-contract §8.1 | **部分**（**11/12 类**黑盒负例；`IOError` 由 Rust 单测；traceback 折叠 / `--json` 逐字符由 Rust 单测 + CLI e2e 覆盖，非黑盒范畴 —— 见 §3/§4） | P5.4 ✅ |
| 可移植性（BOM、CRLF/LF/CR、非 UTF-8 → `SyntaxError`） | syntax §2.1、§6-B4/B7/B8 | 待实现 | P5.5 |
| 入口一致性（REPL / stdin / `-e` 豁免前导；伪路径） | syntax §6-B9 | 待实现 | P5.5 |

---

## 2. 内置函数覆盖表（`interface-contract.md` §10.7，全表 **54** 个逐个）

> 契约：`interface-contract.md` **§10.7**（54 个，`data-last`、返回新值不改原容器 A1）、**§10.7「内置边界补钉」**、**§10.7「数值内置形参加宽」**；错误类见 `semantics.md` **§8.1**。
> 用例组文件：`tests/lfz/test_builtins_{array,higher_order,struct,string,math,convert,random,io}.lfz` —— 共 **8** 文件 / **263** 条 `assert`。
> 负例：`tests/cases.json` 本批新增 **23** 条 `expect.error` fixture；另复用 P5.1/P5.2 的 `div_zero`、`del_method_field`、`del_missing_data_field`。
> 状态口径：`通过` = 有自动发现正向断言 +（如适用）`expect.error` 负例判 PASS；`跳过` = 环境相关/未声明，在「说明」列注明。

### 2.1 核心 / 数组（14 个，`tests/lfz/test_builtins_array.lfz`）

| # | 内置 | 正常+边界断言 | 错误负例（fixture → 期望类） | 状态 | 说明 |
|---|---|---|---|---|---|
| 1 | `len` | 5 | — | 通过 | array 元素数 / struct 数据字段数 / string Unicode 标量数 |
| 2 | `range` | 4 | — | 通过 | `n<0` → 空数组 |
| 3 | `push` | 3 | — | 通过 | **A1** `push_does_not_mutate` |
| 4 | `pop` | 4 | `pop_empty` → `IndexError` | 通过 | `pop([])` → `Index{idx:-1,len:0}`；**A1** |
| 5 | `removeAt` | 4 | `removeAt_out_of_range` / `removeAt_negative_out_of_range` → `IndexError` | 通过 | 支持负索引；**A1** |
| 6 | `insert` | 5 | `insert_negative_index` / `insert_index_too_large` → `IndexError` | 通过 | 合法域 `i∈[0,len]`，**不支持负索引**；**A1** |
| 7 | `swap` | 4 | `swap_out_of_range` → `IndexError` | 通过 | 支持负索引；**A1** |
| 8 | `slice` | 7 | — | 通过 | 下标夹取 `[0,len]`；`from>=to` → 空；**A1** |
| 9 | `min` | 5 | `min_empty` → `ValueError` | 通过 | 确定性全序（-Inf<有限<+Inf<NaN） |
| 10 | `max` | 5 | `max_empty` → `ValueError` | 通过 | 同上 |
| 11 | `sum` | 7 | `sum_overflow` → `OverflowError` | 通过 | 全 int→int；含 float→float；空→`int 0` |
| 12 | `sort` | 8 | `sort_mixed_types` → `TypeError` | 通过 | 升序新数组；NaN 排最后；**稳定性由 `sortBy` 举证** |
| 13 | `take` | 5 | — | 通过 | `n` 夹取 `[0,len]` |
| 14 | `drop` | 4 | — | 通过 | 去前 `n` 个 |

### 2.2 高阶（7 个，`tests/lfz/test_builtins_higher_order.lfz`）

| # | 内置 | 正常+边界断言 | 错误负例（fixture → 期望类） | 状态 | 说明 |
|---|---|---|---|---|---|
| 15 | `map` | 4 | — | 通过 | 逐元素；**A1** |
| 16 | `filter` | 5 | `filter_predicate_not_bool` → `TypeError` | 通过 | 谓词须返回 `bool`；**A1** |
| 17 | `reduce` | 5 | — | 通过 | **左折叠顺序**已用 `acc*10+x`（=123）钉死；空→`init` |
| 18 | `sortBy` | 4 | — | 通过 | **稳定性**：等键元素保持输入先后序；**A1** |
| 19 | `minBy` | 3 | `minBy_empty` → `ValueError` | 通过 | 以 `keyFn` 结果为准 |
| 20 | `maxBy` | 3 | `maxBy_empty` → `ValueError` | 通过 | 同上 |
| 21 | `each` | 4 | — | 通过 | 仅副作用；返回 `nil` |

### 2.3 struct / 字典（5 个，`tests/lfz/test_builtins_struct.lfz`）

| # | 内置 | 正常+边界断言 | 错误负例（fixture → 期望类） | 状态 | 说明 |
|---|---|---|---|---|---|
| 22 | `keys` | 5 | — | 通过 | **字节序升序**（`'A'<'a'<'z'`）；**A5** 不含方法字段 |
| 23 | `values` | 3 | — | 通过 | 与 `keys` 同序；**A5** |
| 24 | `entries` | 4 | — | 通过 | `[k,v]` 二元数组，按 `keys` 序；**A5** |
| 25 | `has` | 6 | — | 通过 | 仅数据字段；方法字段 → `false`（**A5**） |
| 26 | `del` | 5 | `del_method_field` / `del_missing_data_field` → `FieldError` | 通过 | 返回新 struct；**A5**：`del` 成功 ⟺ `has==true` |

### 2.4 字符串（8 个，`tests/lfz/test_builtins_string.lfz`）

| # | 内置 | 正常+边界断言 | 错误负例（fixture → 期望类） | 状态 | 说明 |
|---|---|---|---|---|---|
| 27 | `split` | 5 | — | 通过 | `sep` 空串 → 按字符切分 |
| 28 | `join` | 4 | `join_non_string_element` → `TypeError` | 通过 | 元素须为 string |
| 29 | `trim` | 4 | — | 通过 | 去首尾空白 |
| 30 | `upper` | 3 | — | 通过 | ASCII 大小写 |
| 31 | `lower` | 2 | — | 通过 | 同上 |
| 32 | `replace` | 4 | — | 通过 | 全部替换；返回新串（§4.5.11） |
| 33 | `repeat` | 4 | — | 通过 | `n<=0` → 空串 |
| 34 | `startsWith` | 5 | — | 通过 | 空前缀恒 `true` |

### 2.5 数学（7 个，`tests/lfz/test_builtins_math.lfz`）

| # | 内置 | 正常+边界断言 | 错误负例（fixture → 期望类） | 状态 | 说明 |
|---|---|---|---|---|---|
| 35 | `abs` | 7 | — | 通过 | **同型不加宽**：`type(abs(-3))=="int"`、`type(abs(-3.0))=="float"` |
| 36 | `floor` | 4 | `floor_nan` → `ValueError` | 通过 | `int` 实参加宽；返回 `int` |
| 37 | `ceil` | 4 | `ceil_inf` → `OverflowError` | 通过 | 同上 |
| 38 | `round` | 10 | — | 通过 | **四舍六入五成双**：`round(0.5)==0`、`round(2.5)==2`、`round(-2.5)==-2` |
| 39 | `sqrt` | 6 | — | 通过 | 负数 → `NaN`（`sqrt(-1)!=sqrt(-1)`）；返回 `float` |
| 40 | `pow` | 6 | — | 通过 | 溢出 → `±Inf`（`pow(10.0,400.0)==inf`、`pow(-10.0,401.0)==-inf`） |
| 41 | `div` | 6 | `div_zero`(P5.1) → `ZeroDivisionError`；`div_non_int` → `TypeError` | 通过 | **向下取整**（`div(-7,2)==-4`）；两参须 `int` |

### 2.6 随机（3 个，`tests/lfz/test_builtins_random.lfz`）

| # | 内置 | 正常+边界断言 | 错误负例（fixture → 期望类） | 状态 | 说明 |
|---|---|---|---|---|---|
| 42 | `seed` | 5 | — | 通过 | `seed(42)` 两次得到**同一序列**（3 次 `rand` 比对）；返回 `nil` |
| 43 | `rand` | 2 | — | 通过 | 落在 `[0.0,1.0)`（20 次循环断言）；返回 `float` |
| 44 | `randInt` | 3 | `randInt_bad_range` → `ValueError` | 通过 | 落在 `[lo,hi)`（50 次循环断言）；同种子可复现 |

### 2.7 转换（4 个，`tests/lfz/test_builtins_convert.lfz`）

| # | 内置 | 正常+边界断言 | 错误负例（fixture → 期望类） | 状态 | 说明 |
|---|---|---|---|---|---|
| 45 | `str` | 14 | — | 通过 | 显示形式（§3.7）：`nan`/`inf`/`-inf`/数组/struct |
| 46 | `int` | 8 | `int_nan`→`ValueError`；`int_inf`→`OverflowError`；`int_bad_string`→`ValueError` | 通过 | float **向零截断**；`string`/`bool`→`int` |
| 47 | `float` | 9 | `float_bad_string` → `ValueError` | 通过 | 支持 `"inf"`/`"-inf"`/`"nan"` |
| 48 | `type` | 8 | — | 通过 | 8 种类型名（int/float/string/bool/nil/array/struct/function） |

### 2.8 IO / 断言（5 个，`tests/lfz/test_builtins_io.lfz`）+ `input`

| # | 内置 | 正常+边界断言 | 错误负例（fixture → 期望类） | 状态 | 说明 |
|---|---|---|---|---|---|
| 49 | `print` | 4 | — | 通过（**文本未断言**，见 §4） | 仅断言返回 `nil`；stdout 文本不可捕获 |
| 50 | `eprint` | 2 | — | 通过（**文本未断言**，见 §4） | 同上（stderr） |
| 51 | `check` | 4 | — | 通过 | **失败非致命**：`check(false)==false` 后程序继续（A4） |
| 52 | `assert` | 2 | `fail_raises` → `AssertionError`（`assert` 失败同为 `AssertionError`） | 通过 | 成功返回 `nil`；失败→`AssertionError`（致命） |
| 53 | `fail` | 0 | `fail_raises` → `AssertionError` | 通过 | 纯负例（恒抛 `AssertionError`） |
| 54 | `input` | 0 | — | **跳过（环境相关）** | runner 不提供 stdin（可能 EOF/**阻塞**），不稳定；**由 Rust 单测覆盖**：`src/builtins.rs::tests::input_reads_line_crlf_and_eof`（EOF→`IOError`）。见 §4 |

> **54 个内置覆盖结论**：**53** 个由 LFZ 黑盒测试集（正向断言 + `expect.error` 负例）逐项覆盖；`input` 因环境依赖按任务授权**跳过并注明由 Rust 单测覆盖**（§4）。
> **关键不变量覆盖**：A1「返回新值不改原容器」在 `push/pop/removeAt/insert/swap/slice/sort/map/filter/del` 均有 `*_does_not_mutate` 断言；A5 数据面在 `keys/values/entries/has/len/del` 均断言排除方法字段。`data-last` 经 P5.2 `test_pipe.lfz` 的 `xs |> map/filter/reduce/sum/take/slice` 组合验证。

---

## 3. 错误类覆盖表（`interface-contract.md` §8.1，12 个具体类 + 基类，逐个）

> 权威来源：错误类触发条件与消息由 `semantics.md` §8.1 规范性给出；本表为**黑盒触发证据**对照。
> 断言口径：错误用例**只断言错误类**（§8.1 的 12 类名之一），**不逐字断言中文消息 / 行号 / 列号**（理由见 §4）。
> 计数：`tests/cases.json` 中 `expect.error` 逐类出现次数，合计 **56**。

| # | 错误类（`--json` / `class_name()`） | 阶段 | 触发用例（fixture → manifest 名） | 数量 | 状态 |
|---|---|---|---|---|---|
| 1 | `CosmosAnswerError` | 加载 | `missing_preamble`(preamble_missing_first_line) / `preamble_trailing_space` / `preamble_trailing_tab` / `preamble_no_newline`(preamble_no_newline_eof) / `empty`(preamble_empty_file) | 5 | **通过** |
| 2 | `SyntaxError` | 加载/解析 | `single_semi` / `hash_mid_program` / `placeholder_binding` / `break_outside_loop` / `syntax_illegal_char` / `syntax_unclosed_paren` / `int_literal_overflow` / `pipe_multiple_placeholders` / `pipe_underscore_in_lambda` / `continue_outside_loop` / `return_outside_fn` | 11 | **通过** |
| 3 | `NameError` | 运行 | `name_error`(undefined_name) | 1 | **通过** |
| 4 | `TypeError` | 运行 | `let_rebind` / `let_compound_rebind` / `type_error_add` / `if_condition_not_bool` / `for_over_string` / `pipe_rhs_not_function` / `interp_format_type_mismatch` / `filter_predicate_not_bool` / `join_non_string_element` / `sort_mixed_types` / `div_non_int` | 11 | **通过** |
| 5 | `IndexError` | 运行 | `pop_empty` / `removeAt_out_of_range` / `removeAt_negative_out_of_range` / `insert_negative_index` / `insert_index_too_large` / `swap_out_of_range` | 6 | **通过** |
| 6 | `FieldError` | 运行 | `del_method_field` / `del_missing_data_field` | 2 | **通过** |
| 7 | `ZeroDivisionError` | 运行 | `zero_division` / `mod_zero` / `div_zero` | 3 | **通过** |
| 8 | `OverflowError` | 运行 | `overflow_add` / `abs_i64_min` / `int_inf` / `sum_overflow` / `ceil_inf` | 5 | **通过** |
| 9 | `ValueError` | 运行 | `interp_bad_format` / `min_empty` / `max_empty` / `minBy_empty` / `maxBy_empty` / `int_nan` / `int_bad_string` / `float_bad_string` / `randInt_bad_range` / `floor_nan` | 10 | **通过** |
| 10 | `IOError` | 运行 | —（黑盒**跳过**） | 0 | **跳过（环境相关）**：runner 不提供 stdin；`input()` 可能 EOF 或**阻塞**，不可复现。**由 Rust 单测覆盖**：`src/builtins.rs::tests::input_reads_line_crlf_and_eof`（EOF→`IOError`）+ `src/error.rs::tests::class_name_all_twelve_match_spec_exactly`。见 §4 |
| 11 | `AssertionError` | 运行 | `fail_raises`(builtin_fail_assertion_error) | 1 | **通过**（`fail(msg)`；`assert(false)` 同类） |
| 12 | `RecursionError` | 运行 | `deep_recursion`(deep_recursion_exceeds_limit) | 1 | **通过** |
| — | `LfzError`（基类） | — | —（基类**不直接抛出**，spec §8.1） | — | **不适用** |
| | | | | **合计 56** | **11/12 类黑盒覆盖 + 1 类（`IOError`）由 Rust 单测** |

> `check(false)` **不产生**任何 `LfzError`（非致命，A4）→ 不属于本表，其行为在 §1.1 / §2.8 以「返回值 + 不中断」断言。

---

## 4. 不可断言项清单（为什么测不到 + 由谁覆盖）

runner 契约（`runner-contract.md` §6 / §9.3）规定：非 `--json` 模式下解释器内建 `print` / `;;`（含 `check` 警告）写 **stdout/stderr**，runner **不对程序 stdout 做捕获与比对**（报告本身也写 stdout）；`--json` 模式下程序输出重定向到 stderr，但 JSON 仅含 verdict / 错误字段，**不含程序输出文本**。故以下项**无法在黑盒 LFZ `assert` 中逐字判定**，采用「诚实标注 + 保底不报错 + 注明替代覆盖者」策略：

| # | 不可断言项 | 为什么测不到 | 由谁覆盖（替代证据） | spec 依据 |
|---|---|---|---|---|
| 1 | `;;` dump 的**输出文本**（`<name> ： <value>` 行、内→外顺序、遮蔽去重） | `;;` 写程序 stdout；runner 不捕获程序 stdout；`--json` 下亦**不**重定向（契约 §9.3 仅覆盖 `print`/`input` 提示） | ① 黑盒保底：`tests/lfz/test_dump.lfz` 执行到 `;;` **不报错**（7 条断言均 PASS）；② Rust 单测覆盖 dump 渲染逻辑；③ 带外证据：人工 `cargo run -- run` 观察 | semantics.md §3.6 |
| 2 | 自引用容器的 **`<cycle>` 渲染字面量** | 同上（写 stdout）；渲染文本非 `assert` 可直接比对的返回值 | ① 黑盒：`test_cycle_safety.lfz` 断言 `==` 环等价 + `str(cycle)` 不抛错；② Rust 单测覆盖显示渲染 | semantics.md §3.7 / §4.5.9 |
| 3 | 所有错误用例的**中文消息文本** | 消息措辞的稳定性无契约保证；黑盒无「读取错误消息」通道 | ① 黑盒：只断言**错误类**（§8.1 的 12 类名之一）；② Rust 单测逐条断言消息（`src/error.rs::tests`） | interface-contract.md §8.1 |
| 4 | 错误的**行号 / 列号**（span） | LFZ 程序内无「反射自身 span」能力；runner 报告中的位置仅供人工定位 | Rust 单测（`src/error.rs` span 断言）+ CLI e2e（`tests/cli.rs` 断言报告位置） | semantics.md §8.2 / §8.4 |
| 5 | `input()` 的端到端行为 | runner 不提供 stdin 约定；`input()` 可能 EOF 或**阻塞**（结果不可复现） | **跳过自动发现用例**；Rust 单测：`src/builtins.rs::tests::input_reads_line_crlf_and_eof`（CRLF 去尾 + EOF→`IOError`） | interface-contract §10.7 `input`；semantics §8.1 `IOError` |
| 6 | 程序 **stdout 捕获 / 逐字比对**（`print` / `eprint` / `check` 警告文本） | runner 契约 §6：程序输出不捕获（`--json` 下重定向到 stderr，但 JSON 不含程序输出文本） | ① 黑盒：仅断言返回值（`nil`/`bool`）与「执行不报错」；② CLI e2e：`tests/cli.rs::json_run_print_*` 断言重定向行为 | semantics.md §4.5.10；interface-contract §10.7 |
| 7 | 内置名作为**一等值**传递（`type(type)` / `map(type, xs)`） | spec §10.7 仅给出调用签名，未声明内置名可作实参（示例均传 lambda）；实现将其作为「仅调用名」 | 不作断言；黑盒仅断言 `type(<lambda>)=="function"` | interface-contract.md §10.7 |
| 8 | `traceback` **折叠**与 `--json` 字段**逐字符** | 属 runner / CLI 输出契约，非 LFZ 程序内可观察量 | Rust 单测（`src/error.rs`、`src/json.rs`）+ CLI e2e（`tests/cli.rs::json_*`、`tests/test_runner.rs::json_*`） | semantics §8.3/§8.4；interface-contract §10.6 |

> 若后续需要端到端断言第 1/2/6 项，须由 tooling-dev 在 runner 侧提供「捕获 / 断言程序 stdout」能力；当前不扩权，故按上表标注。

---

## 5. 负例清单（`tests/cases.json`，共 **56** 条 `expect.error` + 1 条正向豁免）

### 5.1 P5.1（21 条）

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

### 5.2 P5.2（12 条）

| # | fixture 路径 | manifest 名 | 期望错误类 | 覆盖特性 |
|---|---|---|---|---|
| 22 | `fixtures/if_condition_not_bool.lfz` | `if_condition_not_bool` | `TypeError` | 条件非 bool（无 truthiness） |
| 23 | `fixtures/for_over_string.lfz` | `for_over_string_not_iterable` | `TypeError` | 可迭代仅 array/struct |
| 24 | `fixtures/del_method_field.lfz` | `del_method_field_is_field_error` | `FieldError` | `del` 方法字段（A5 数据面） |
| 25 | `fixtures/del_missing_data_field.lfz` | `del_missing_data_field` | `FieldError` | `del` 缺失数据字段 |
| 26 | `fixtures/pipe_rhs_not_function.lfz` | `pipe_rhs_not_function` | `TypeError` | 管道右侧非函数 |
| 27 | `fixtures/interp_format_type_mismatch.lfz` | `interp_format_type_mismatch` | `TypeError` | 说明符与值类型不符 |
| 28 | `fixtures/interp_bad_format.lfz` | `interp_bad_format_specifier` | `ValueError` | 非法格式说明符 |
| 29 | `fixtures/pipe_multiple_placeholders.lfz` | `pipe_multiple_placeholders` | `SyntaxError` | 管道 ≥2 个 `_` |
| 30 | `fixtures/pipe_underscore_in_lambda.lfz` | `pipe_underscore_in_lambda` | `SyntaxError` | `_` 穿 λ 体（M4） |
| 31 | `fixtures/continue_outside_loop.lfz` | `syntax_continue_outside_loop` | `SyntaxError` | 循环外 `continue` |
| 32 | `fixtures/return_outside_fn.lfz` | `syntax_return_outside_function` | `SyntaxError` | 函数外 `return` |
| 33 | `fixtures/deep_recursion.lfz` | `deep_recursion_exceeds_limit` | `RecursionError` | 深递归 > 10000 帧 |

### 5.3 P5.3（23 条；内置函数边界/错误）

| # | fixture 路径 | manifest 名 | 期望错误类 | 覆盖特性 |
|---|---|---|---|---|
| 34 | `fixtures/pop_empty.lfz` | `builtin_pop_empty_index_error` | `IndexError` | `pop([])` |
| 35 | `fixtures/removeAt_out_of_range.lfz` | `builtin_removeAt_out_of_range` | `IndexError` | `removeAt(5,[1,2,3])` |
| 36 | `fixtures/removeAt_negative_out_of_range.lfz` | `builtin_removeAt_negative_out_of_range` | `IndexError` | `removeAt(-9,…)` |
| 37 | `fixtures/insert_negative_index.lfz` | `builtin_insert_negative_index` | `IndexError` | `insert` 不支持负索引 |
| 38 | `fixtures/insert_index_too_large.lfz` | `builtin_insert_index_too_large` | `IndexError` | `insert(9,…)` 超 `len` |
| 39 | `fixtures/swap_out_of_range.lfz` | `builtin_swap_out_of_range` | `IndexError` | `swap(0,9,…)` |
| 40 | `fixtures/min_empty.lfz` | `builtin_min_empty_value_error` | `ValueError` | `min([])` |
| 41 | `fixtures/max_empty.lfz` | `builtin_max_empty_value_error` | `ValueError` | `max([])` |
| 42 | `fixtures/minBy_empty.lfz` | `builtin_minBy_empty_value_error` | `ValueError` | `minBy(_,[])` |
| 43 | `fixtures/maxBy_empty.lfz` | `builtin_maxBy_empty_value_error` | `ValueError` | `maxBy(_,[])` |
| 44 | `fixtures/filter_predicate_not_bool.lfz` | `builtin_filter_predicate_not_bool` | `TypeError` | `filter` 谓词非 `bool` |
| 45 | `fixtures/join_non_string_element.lfz` | `builtin_join_non_string_element` | `TypeError` | `join` 元素非 string |
| 46 | `fixtures/int_nan.lfz` | `builtin_int_nan_value_error` | `ValueError` | `int(NaN)` |
| 47 | `fixtures/int_inf.lfz` | `builtin_int_inf_overflow_error` | `OverflowError` | `int(±Inf)` |
| 48 | `fixtures/int_bad_string.lfz` | `builtin_int_bad_string` | `ValueError` | `int("abc")` |
| 49 | `fixtures/float_bad_string.lfz` | `builtin_float_bad_string` | `ValueError` | `float("notanumber")` |
| 50 | `fixtures/randInt_bad_range.lfz` | `builtin_randInt_bad_range` | `ValueError` | `randInt(5,5)`（`lo>=hi`） |
| 51 | `fixtures/fail_raises.lfz` | `builtin_fail_assertion_error` | `AssertionError` | `fail(msg)` |
| 52 | `fixtures/sort_mixed_types.lfz` | `builtin_sort_mixed_types` | `TypeError` | `sort([1,"a"])` |
| 53 | `fixtures/sum_overflow.lfz` | `builtin_sum_overflow` | `OverflowError` | `sum([i64::MAX,1])` |
| 54 | `fixtures/floor_nan.lfz` | `builtin_floor_nan_value_error` | `ValueError` | `floor(NaN)` |
| 55 | `fixtures/ceil_inf.lfz` | `builtin_ceil_inf_overflow_error` | `OverflowError` | `ceil(±Inf)` |
| 56 | `fixtures/div_non_int.lfz` | `builtin_div_non_int_type_error` | `TypeError` | `div(1.0,2)` 非 `int` |

> 另有 `fixtures/plain_ok.txt` → `non_lfz_file_no_preamble_required`（**无** `expect`，正向豁免，判正常 PASS）。

---

## 6. 正向用例清单

### 6.1 P5.1 正向（7 文件 / 114 条 `assert`）

| 文件 | 断言数 | 覆盖特性 |
|---|---|---|
| `tests/lfz/test_literals.lfz` | 40 | int/float/string(+转义)/bool/nil 字面量 |
| `tests/lfz/test_let_var.lfz` | 12 | `let`/`var`/复合赋值 |
| `tests/lfz/test_arithmetic.lfz` | 24 | `+ - *`、比较、逻辑（短路）、一元 `-` |
| `tests/lfz/test_precedence.lfz` | 11 | 优先级与结合性 |
| `tests/lfz/test_division_modulo.lfz` | 14 | `/`（float）、`div`（向下取整）、`%`（符号随除数） |
| `tests/lfz/test_int_min.lfz` | 8 | `i64::MIN`/`MAX` 边界 |
| `tests/lfz/test_preamble.lfz` | 5 | `#42` 前导 |

### 6.2 P5.2 正向（10 文件 / 170 条 `assert`）

| 文件 | 断言数 | 覆盖特性 |
|---|---|---|
| `tests/lfz/test_if_else.lfz` | 11 | `if` / `else`（语句 + 表达式 + 嵌套） |
| `tests/lfz/test_loops.lfz` | 16 | `while` / `for` / `break` / `continue` / 快照 / struct 键序 |
| `tests/lfz/test_functions.lfz` | 26 | `fn` / 递归 / 函数字面量 / 高阶 / 同一性 |
| `tests/lfz/test_closures.lfz` | 14 | 闭包 cell 捕获（A2） |
| `tests/lfz/test_structs.lfz` | 27 | struct / 字段 / 方法 vs 数据面（A5）/ `del` |
| `tests/lfz/test_pipe.lfz` | 15 | 管道 `\|>` / `_` / 链式 |
| `tests/lfz/test_interpolation.lfz` | 16 | 富字符串插值 / `format_spec` / `\$` |
| `tests/lfz/test_dump.lfz` | 7 | `;;` dump（保底不报错） |
| `tests/lfz/test_reference_semantics.lfz` | 29 | A1 引用语义 |
| `tests/lfz/test_cycle_safety.lfz` | 9 | A6 环安全 |

### 6.3 P5.3 正向（8 文件 / 263 条 `assert`，内置函数全表）

| 文件 | 断言数 | 覆盖内置 |
|---|---|---|
| `tests/lfz/test_builtins_array.lfz` | 70 | `len` `range` `push` `pop` `removeAt` `insert` `swap` `slice` `min` `max` `sum` `sort` `take` `drop` |
| `tests/lfz/test_builtins_higher_order.lfz` | 28 | `map` `filter` `reduce` `sortBy` `minBy` `maxBy` `each` |
| `tests/lfz/test_builtins_struct.lfz` | 26 | `keys` `values` `entries` `has` `del`（A5 数据面） |
| `tests/lfz/test_builtins_string.lfz` | 33 | `split` `join` `trim` `upper` `lower` `replace` `repeat` `startsWith` |
| `tests/lfz/test_builtins_math.lfz` | 43 | `abs` `floor` `ceil` `round` `sqrt` `pow` `div` |
| `tests/lfz/test_builtins_convert.lfz` | 39 | `str` `int` `float` `type` |
| `tests/lfz/test_builtins_random.lfz` | 10 | `seed` `rand` `randInt` |
| `tests/lfz/test_builtins_io.lfz` | 14 | `print` `eprint` `check` `assert`（`fail` 为负例） |

> 正向合计：**25 文件 / 547 条 `assert`**。

---

## 7. 已知偏差（缺陷单）

| 缺陷单 | 现象 | spec 依据 | 影响用例 | 状态 |
|---|---|---|---|---|
| **bug-20260927-01** | 经方括号取得的方法值**再调用时不绑定 `self`**：`p["norm2"]()` → `NameError: 未定义的名字 'self'`（`.字段()` 调用点则绑定正常） | semantics.md §3.7「`s.k ≡ s["k"]` 仍能取到该函数值并**调用（`self` 绑定）**」；§4.5.8「方法…访问时绑定 `self`」；ADR D-015 A5（DECISIONS.md） | `test_structs.lfz` 中「方括号调用方法」未断言（仅断言 `type(p["norm2"]) == "function"` 可取到函数值）；已提交 team-lead | **未修**（待 core/runtime 修复后补端到端用例并回归） |

---

## 8. 运行证据（P5.4 定稿）

```
$ cargo run --quiet -- test
PASS  abs_i64_min_overflow
PASS  syntax_break_outside_loop
...
PASS  tests/lfz/test_builtins_io.lfz
PASS  tests/lfz/test_structs.lfz

汇总：共 82 个用例，通过 82，失败 0，错误 0
$ echo $LASTEXITCODE
0
```

- **用例总数：82** = 正向自动发现 **25**（P5.1 的 7 + P5.2 的 10 + P5.3 的 8）+ 清单（负例 **56** + 非 `.lfz` 正向豁免 1）。
- 正向断言总数：**547**（114 + 170 + 263）。
- 退出码 **0**（全绿）。
- 构建：`cargo clean -p lfz` 后 `cargo build` → **0 warning / 0 error**。
- 回归：`cargo test` → **431 passed / 0 failed / 0 ignored**（361 + 42 + 16 + 12；未被本批破坏）。

---

## 9. 后续批次待覆盖特性（P5.5+）

| 特性域 | spec 依据 | 状态 | 计划批次 |
|---|---|---|---|
| 可移植性（BOM、CRLF/LF/CR、非 UTF-8 → `SyntaxError`） | syntax §2.1、§6-B4/B7/B8 | 待实现 | P5.5 |
| 入口一致性（REPL / stdin / `-e` 豁免前导；伪路径） | syntax §6-B9 | 待实现 | P5.5 |
| `;;` / `print` 输出文本、错误消息文本、span 的端到端断言 | semantics §3.6/§3.7/§8.2 | 待工具支持（见 §4） | 待 tooling-dev 扩权 |
