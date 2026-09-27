# LFZ 错误模型

> LFZ 采用 **Python 风格**的错误模型：**类名 + 中文消息 + 位置（行/列）+ 源码行 + 插入符**，运行期错误附 `Traceback`。
> 权威定义：[`docs/spec/semantics.md`](../spec/semantics.md) §8（用户可见行为）；[`docs/spec/interface-contract.md`](../spec/interface-contract.md) §8.1（实现映射）。
> 配套：[`README.md`](README.md) ｜ [`tutorial.md`](tutorial.md) ｜ [`reference.md`](reference.md) ｜ [`testing.md`](testing.md)
> 最后更新: 2026-09-27（T11-③ D1–D7：零帧 Traceback 例外、容量溢出、嵌套超限） by docs-writer

---

## 1. 错误类总览（12 类 + 1 基类）

基类 `LfzError` **不直接抛出**；用户可见输出只有**类名**，**没有** `E-xxx` 编号（`SE §8.1`）。

| 错误类 | 什么时候出现 | 中文消息模板 | 阶段 |
|---|---|---|---|
| `CosmosAnswerError` | `.lfz` 文件缺少合法 `#42` 前导 | `你忘记了宇宙的答案` | 加载 |
| `SyntaxError` | 词法/语法错误（非法字符、未闭合串/注释、意外记号、单 `;`、同行两语句、`#` 位置非法、`_` 位置非法、**解析/AST 嵌套超深**等） | 见 §2 细分表 | 加载/解析 |
| `NameError` | 引用未定义的名字 | `未定义的名字 '{name}'` | 运行 |
| `TypeError` | 运算符/条件/调用/参数/格式说明符类型不符；**重绑定 `let` 变量** | 见 §2 细分表 | 运行 |
| `IndexError` | 数组下标越界（含负索引规范化后越界） | `下标 {i} 越界（长度 {n}）` | 运行 |
| `FieldError` | struct 不存在该字段；`del(k, s)` 的键不是数据字段 | `结构体没有字段 '{name}'` | 运行 |
| `ZeroDivisionError` | `/`、`%`、`div(a,b)` 除数为零（含 float） | `除以零` / `对零取模` | 运行 |
| `OverflowError` | `int` 运算超出 i64；`int(float)` 遇 `±Inf`/超界；**容器/字符串构造所需容量超出可分配上限** | `整数溢出：结果超出 i64 范围` / `容量溢出：所需容量超出可分配上限` | 运行 |
| `ValueError` | 显式转换失败；格式说明符非法；空数组取极值；`randInt` 区间非法 | 见 §2 | 运行 |
| `IOError` | `input()` 遇 EOF；不可读文件 | `输入结束（EOF）` / `无法读取：{path}` | 运行 |
| `AssertionError` | **仅** `assert` 失败 或 `fail()`（`check` 失败**不**抛此错） | `断言失败：{msg}` / `{msg}` | 运行 |
| `RecursionError` | 求值帧深度超限（默认 10000 层） | `递归深度超限（超过 10000 层）` | 运行 |

> **运行期 = 10 类**：除 `CosmosAnswerError`、`SyntaxError`（加载/解析期）之外的全部，输出**带 `Traceback` 头**（`SE §8.2`）。
> ⚠️ **零帧例外**：`Traceback` 头**当且仅当求值帧栈非空**。若运行期错误在**任何求值帧建立之前**发生（帧栈为空，典型是 `IOError`：文件不存在 / 不可读），则**不输出** `Traceback` 头——只是**一行** `<类名>: <消息>`（详见 §3.2）。

---

## 2. 常见子消息

**`SyntaxError` 细分**（`SE §8.1`）：

| 场景 | 消息 |
|---|---|
| 非法字符 | `非法字符 '{c}'` |
| `#` 位置非法 | `'#' 只能出现在文件首行的前导位；(字符串 / 注释 / 格式说明符内的 '#' 除外)` |
| 字符串未闭合 | `字符串字面量在此处未闭合` |
| 块注释未闭合 | `块注释在此处未闭合（缺少 '*/'）` |
| 意外记号 | `这里期待 {expected}，但得到 {got}` |
| 表达式未结束 | `表达式未结束：行尾不能终止表达式；请用括号跨行` |
| 单个 `;` | `单独的 ';' 非法；打印变量请用 ';;'` |
| 同一逻辑行两语句 | `语句之间必须有换行` |
| 赋值目标非法 | `赋值左侧必须是变量、字段或下标` |
| `_` 位置非法 | `占位符 '_' 只能出现在管道右侧的调用实参中` |
| 管道多 `_` | `管道右侧调用最多只能有一个 '_'` |
| break/continue 在循环外 | `'{kw}' 只能出现在循环体内` |
| return 在函数外 | `'return' 只能出现在函数体内` |
| 非 UTF-8 编码 | `文件不是合法的 UTF-8 编码（首个非法字节位于字节偏移 {off}）` |
| 未列举的转义 | `字符串中不支持的转义 '\{c}'` |
| 插值内裸换行 | `插值表达式不能跨行；请把表达式写在一行内` |
| 整数字面量越界 | `整数字面量超出 i64 范围` |
| **解析嵌套超深**（`(`/`[`/`{`/`if`/`while`/`for`/`fn`/`${` 嵌套 > 1000 层） | `嵌套深度超限（超过 1000 层）` |
| **AST / 结构嵌套过深**（左结合长链如 `1+1+…` > 10000 层） | `表达式嵌套过深（超过 10000 层）` |

**`TypeError` 细分**（`SE §8.1`）：

| 场景 | 消息 |
|---|---|
| 运算符类型不符 | `运算符 '{op}' 不支持 {lt} 与 {rt}` |
| 条件非 bool | `条件必须是 bool，得到 {t}` |
| 调用非函数 | `不可调用：{t} 不是函数` |
| 管道右侧非函数 | `管道右侧必须是函数，得到 {t}` |
| 参数个数不符 | `函数 {name} 期待 {n} 个参数，得到 {m}` |
| 格式说明符与值类型不符 | `格式说明符 '{spec}' 不适用于 {t}` |
| 重绑定 `let` 变量 | `不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容` |

**`OverflowError` 细分**（`SE §8.1`、`IC §10.7`）：

| 场景 | 消息 |
|---|---|
| `int` 运算 / 转换越界 | `整数溢出：结果超出 i64 范围` |
| **容器/字符串构造容量超限**（`range` 超大 `n`、`repeat` 结果过长） | `容量溢出：所需容量超出可分配上限` |

**`ValueError` 细分**（`SE §8.1`）：

| 场景 | 消息 |
|---|---|
| 转换失败 | `无法把 {src} 转换为 {dst}（'{text}'）` |
| 格式说明符非法 | `格式说明符非法：'{spec}'` |
| 空数组取极值 | `空数组没有极值（{func}）` |
| `randInt` 区间非法 | `区间非法：{lo} >= {hi}` |

> v1 明确**不输出** `hint` / `提示：` 行（保证输出逐字符稳定，`SE §8.1` 末尾）。

---

## 3. 输出格式

### 3.1 通用帧格式（每帧三行）

```text
  File "<path>", line <N>[, in <func>]
    <源码行原文>
    <插入符行>
```

- 源码行原文前固定缩进 **4 个空格**。
- `<func>`：顶层帧为 `<module>`，命名函数为函数名，匿名函数为 `<fn>`。
- 插入符行 = 4 个空格 + `(列号 - 1)` 个空格 + `^`；`^` 指向引发错误的**最小 AST 节点的首字符**（`SE §8.2`）。
- 行号 / 列号均 **1-based**，列按 **Unicode 标量值**计数。

### 3.2 三种结构

- **加载/解析期**（`CosmosAnswerError`、`SyntaxError`）：**无** `Traceback` 头。
  - `SyntaxError`：`File` 帧（含源码行 + 插入符）→ 末行 `SyntaxError: <消息>`。
  - `CosmosAnswerError`：**仅** `File "<path>", line 1` → 末行 `CosmosAnswerError: 你忘记了宇宙的答案`（**无源码行、无插入符、无 hint**）。
- **运行期（帧栈非空）**（其余 10 类）：先 `Traceback (most recent call last):`，逐帧**最外层在前、最内层在后**，末行 `<类名>: <消息>`。
- **运行期（零帧）**（`Traceback` 头**当且仅当帧栈非空**）：错误在**任何求值帧建立之前**发生（帧栈为空，典型 `IOError`：文件不存在/不可读）→ **不输出** `Traceback` 头：
  - 有 `span` → `File` 帧 + 末行；**无 `span`** → **仅**一行 `<类名>: <消息>`（无 `File` 帧、无插入符）。
  - 例：`lfz run nope.lfz` → 仅一行 `IOError: 无法读取：nope.lfz`；`--json` 时 `traceback` 为 `[]`（`SE §8.2` v1.1 补钉）。

### 3.3 深栈折叠（仅人类可读输出）

设帧总数 `T`：`T ≤ 40` 原样输出；`T > 40` → **首 10 帧** → 一行 `  ... 省略 {T−40} 帧 ...` → **尾 30 帧**。`--json` 的 `traceback` 数组**不折叠**（`SE §8.2`、`IC §10.3`）。

### 3.4 退出码

| 退出码 | 条件 |
|---|---|
| `0` | 成功（含 `check` 失败——它不是错误） |
| `1` | 测试有用例失败（`assert`/`fail`，仅 `lfz test`） |
| `2` | **所有错误类**（含 `RecursionError`）；CLI 参数错误；`--json` 错误仍为 `2` |

出处：`SE §8.2`、`IC §8.1`（D-008）。

> **`assert` / `fail` 的退出码看语境**（常见困惑）：
> - 用 **`lfz run`** 直接跑脚本时，`assert` 失败 / `fail()` 抛 `AssertionError`（一个**错误类**）→ 退出码 **`2`**。
> - 用 **`lfz test`** 跑测试时，`assert` 失败的**用例**记为 `FAIL` → 退出码 **`1`**。
> 两者不矛盾：**`1` 专属测试运行器的"用例失败"**，其余任何错误（包括 `run` 下的 `AssertionError`）一律 `2`。

---

## 4. 实测示例（真实运行结果）

> 以下输出用 `cargo run --quiet -- run <file>` 实际运行得到；`<file>` 为演示取的短文件名。

**示例 1 — 语法错（`SyntaxError`，无 Traceback 头）**

`e1_syntax.lfz`：
```lfz
#42
let x = 1 $ 2
```
输出（`$` 在第 2 行第 11 列）：
```text
  File "e1_syntax.lfz", line 2
    let x = 1 $ 2
              ^
SyntaxError: 非法字符 '$'
```
退出码 `2`。

**示例 2 — 运行期错（含调用栈，`ZeroDivisionError`）**

`e2_calc.lfz`：
```lfz
#42
fn half(n) {
    n / 0
}
let r = half(10)
print(r)
```
输出（外层 `half(10)` 在 line 5、第 9 列；内层 `n` 在 line 3、第 5 列）：
```text
Traceback (most recent call last):
  File "e2_calc.lfz", line 5, in <module>
    let r = half(10)
            ^
  File "e2_calc.lfz", line 3, in half
        n / 0
        ^
ZeroDivisionError: 除以零
```
退出码 `2`。

**示例 3 — 缺前导（`CosmosAnswerError`，无 Traceback / 无源码行 / 无插入符）**

`e3_forgot.lfz`（**故意不写 `#42`**，专门演示 `CosmosAnswerError`；这是唯一一种"代码块不以 `#42` 开头"的合法场景）：
```lfz
print("hello")
```
输出：
```text
File "e3_forgot.lfz", line 1
CosmosAnswerError: 你忘记了宇宙的答案
```
退出码 `2`。

**示例 4 — `assert` 致命**

`e4_assert.lfz`：
```lfz
#42
assert(1 == 2, "一比二大？")
print("这行不会执行")
```
输出：
```text
Traceback (most recent call last):
  File "e4_assert.lfz", line 2, in <module>
    assert(1 == 2, "一比二大？")
    ^
AssertionError: 断言失败：一比二大？
```
退出码 `2`。

**示例 5 — `check` 非致命（A4）**

`e5_check.lfz`：
```lfz
#42
check(1 == 2, "软断言示例")
print("继续运行")
```
stdout：
```text
继续运行
```
stderr（一行警告）：
```text
check 失败：软断言示例
```
退出码 `0`（`check` 失败不影响退出码，也不产生 `--json` 错误）。

**示例 6 — 其余错误类**（同一格式，均为运行期 → 带 `Traceback`）

| 代码 | 错误 | 末行消息 |
|---|---|---|
| `print(undefinedName)` | `NameError` | `未定义的名字 'undefinedName'` |
| `print(1 + "a")` | `TypeError` | `运算符 '+' 不支持 int 与 string` |
| `let s = "abc"` + `print(s[0])` | `TypeError` | `运算符 '[]' 不支持 string 与 array / struct` |
| `let xs = [1,2,3]` + `print(xs[10])` | `IndexError` | `下标 10 越界（长度 3）` |
| `let s = { "a": 1 }` + `print(s["b"])` | `FieldError` | `结构体没有字段 'b'` |
| `let x = 1` + `x = 2` | `TypeError` | `不能重新赋值 let 变量 'x'；let 只锁重绑定，不锁内容` |
| `repeat(4611686018427387904, "ab")` | `OverflowError` | `容量溢出：所需容量超出可分配上限` |

**示例 7 — 零帧运行期错误（`IOError`，**无** `Traceback` 头）**

`lfz run nope.lfz`（文件不存在）：
```text
IOError: 无法读取：nope.lfz
```
退出码 `2`。注意：**只有一行**——因为错误发生在任何求值帧建立**之前**，帧栈为空（§3.2「运行期（零帧）」）。

**示例 8 — 解析嵌套超深（`SyntaxError`，加载/解析期 → 无 `Traceback` 头）**

构造：`#42` 之后写 `(` 重复 **1001** 次、中间一个 `1`、再 `)` 重复 1001 次。末行：
```text
SyntaxError: 嵌套深度超限（超过 1000 层）
```
退出码 `2`。同类还有左结合长链 `1+1+…`（> 10000 层，共 10000 项）→ `SyntaxError: 表达式嵌套过深（超过 10000 层）`（两者均为 `SyntaxError`，**不新增错误类**）。

---

## 5. 常见错误 → 怎么改

| 你会看到 | 原因 | 改法 |
|---|---|---|
| `CosmosAnswerError: 你忘记了宇宙的答案` | `.lfz` 文件第 1 行不是 `#42` | 第 1 行写 `#42`（三字符，后紧跟换行） |
| `SyntaxError: 单独的 ';' 非法；打印变量请用 ';;'` | 用了单个 `;` | 用换行分隔语句；打印变量用 `;;` |
| `SyntaxError: 语句之间必须有换行` | 一行写了多条语句 | 一条语句一行 |
| `SyntaxError: 表达式未结束：行尾不能终止表达式；请用括号跨行` | 行尾有未完成运算符 | 把整个表达式放进括号 `( ... )` |
| `SyntaxError: 赋值左侧必须是变量、字段或下标` | 对非 lvalue 赋值（如 `f() = 1`） | 只对变量/字段/下标赋值 |
| `TypeError: 条件必须是 bool，得到 ...` | 无 truthiness | 写比较得到 `bool` |
| `TypeError: 不能重新赋值 let 变量 '...'` | 给 `let` 变量重新赋值 | 声明处改用 `var` |
| `IndexError: 下标 i 越界（长度 n）` | 下标超出范围 | 用 `[0, len-1]`；负索引规范化后同样不得越界 |
| `FieldError: 结构体没有字段 '...'` | 访问不存在的键/字段 | 先 `has(k, s)` 判断，或用 `keys(s)` 查看 |
| `ZeroDivisionError: 除以零` | 除数为 0 | 先判除数；整数除法用 `div(a,b)` |
| `OverflowError: 整数溢出：结果超出 i64 范围` | `int` 运算越界 | 改用 `float` 或限制数值范围 |
| `OverflowError: 容量溢出：所需容量超出可分配上限` | `range` / `repeat` 要构造的容器太大 | 别构造超大数组/字符串；超大循环改用 `while` 迭代 |
| `TypeError: 运算符 '[]' 不支持 string 与 array / struct` | 对**字符串**取下标 `s[0]` | 字符串不能下标；用 `split("", s)` 拆成字符数组（见 [`reference.md` §3.5](reference.md)） |
| `ValueError: 格式说明符非法：'>w'` | 格式宽度写了变量（动态宽度） | 宽度只支持字面数字；按变量宽度用 `repeat` 手工补齐 |
| `SyntaxError: 嵌套深度超限（超过 1000 层）` | 括号/块嵌套超过 1000 层 | 拆小表达式、减少嵌套 |
| `SyntaxError: 表达式嵌套过深（超过 10000 层）` | 左结合长链过长（如超长 `1+1+…`） | 拆成多条语句、或循环累加 |
| `ValueError: 空数组没有极值（min）` | 对空数组取 `min/max/minBy/maxBy` | 先判 `len(xs) > 0` |
| `RecursionError: 递归深度超限（超过 10000 层）` | 递归太深 | 改迭代，或检查递归是否有终止条件 |

---

## 6. `--json` 机器可读输出

`--json` 是全局开关（可放子命令任意位置）。开启后 **stdout 只写一行 JSON**，人类可读诊断（含程序 `print` 输出）改到 stderr（`runner-contract.md §9.3`）。

字段（`SE §8.3` 示例 4 / §8.4）：`ok`(bool) / `error`(类名，仅失败时) / `message`(中文) / `file` / `line` / `col` / `traceback`(数组，元素 `{file,line,func}`，**不折叠**)。

**成功**（`run --json greeting.lfz`，程序输出 2 行到 stderr）：
```json
{"ok":true}
```

**失败 — 缺前导**（`run --json forgot.lfz`）：
```json
{"ok":false,"error":"CosmosAnswerError","message":"你忘记了宇宙的答案","file":"forgot.lfz","line":1,"col":1,"traceback":[{"file":"forgot.lfz","line":1,"func":"<module>"}]}
```

**失败 — 运行期错**（`run --json calc.lfz`，`half(10)` 在 line 5、内层 `n` 在 line 3）：
```json
{"ok":false,"error":"ZeroDivisionError","message":"除以零","file":"calc.lfz","line":3,"col":5,"traceback":[{"file":"calc.lfz","line":5,"func":"<module>"},{"file":"calc.lfz","line":3,"func":"half"}]}
```

- `error` 取值恒为 §1 的 **12 个类名之一**；`check` 失败**不**产生 `error`。
- `line`/`col` 为 `null` 当错误无位置（如 `IOError` 的部分情形）。
- 退出码仍按 §3.4。

---

## 7. 设计原则（为什么这样设计）

- **无魔法**：条件必须 `bool`，除 `int → float` 加宽外无隐式转换；越界/缺字段/溢出/类型不符一律结构化报错（`SE §4.5.0`）。
- **可定位**：每个错误带 `类名 + 中文消息 + 行/列 + 源码行 + 插入符`，运行期还带完整调用栈，`--json` 提供机器可读版本（特色 4，`SE §8`）。
- **可测试**：v1 输出**逐字符稳定**（无 hint 行），黑盒测试可依赖（`SE §8.1` 末尾、`IC §11.2`）。
