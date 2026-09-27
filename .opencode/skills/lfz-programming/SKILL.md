---
name: lfz-programming
description: 编写、修改或调试 LFZ 语言（.lfz）程序时使用，也用于回答 LFZ 语法/语义问题。给出 #42 首行铁律、语法速查、12 个错误类清单、LFZ 特有的常见陷阱与可运行示例；在本仓库用 `cargo run --quiet -- run <file.lfz>` 运行。
---

# LFZ 编程指南（AI 专用）

> 读者：**未接触过 LFZ 的编程 Agent**。目标：只凭本文件写出**正确、可运行**的 LFZ 程序。
> 事实源：`docs/spec/{syntax,semantics,interface-contract}.md`（冻结 v1）。本文件是**速查/转述**，不定义新语法；与 spec 冲突时以 spec 为准。
> 使用纪律：**只允许使用本文件列出的语法**。不要用 Python / C / JS 的规则去猜 LFZ——LFZ 在若干处刻意不同（见 §4）。

> 🚫 **硬纪律：不要凭记忆写 LFZ。** 本文件**没写的语法一律视为不存在**；拿不准就先查 §10 列出的 spec，或改用本文件已列出的等价写法。宁可用保守写法，也不要"猜一个大概像 Python 的写法"。

> ⚠️ **第一件事（置顶铁律）**：每个 `.lfz` 文件的**第一行必须是恰好 `#42`（三字符）+ 一个换行**，否则程序以 `CosmosAnswerError: 你忘记了宇宙的答案` 终止。**所有代码示例均须遵守**。详见 §1。

## 0. 触发与用法

- 任务涉及 `.lfz` 文件、LFZ 脚本、`lfz run`、或 `cargo run -- run` 时加载本指南。
- 运行命令（仓库根目录，先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`）：
  ```bash
  cargo run --quiet -- run <file.lfz>       # 运行一个 LFZ 程序
  cargo run --quiet -- run --json <file>    # 机器可读 JSON 输出
  cargo run --quiet -- test                 # 运行测试集
  ```
- **退出码**：`0` 成功；`1` 测试失败（`assert` / `fail`）；`2` CLI 参数错误 / 语法或运行时错误 / 用例 error。
- **本 CLI v1 只支持 `lfz run <file>` 与 `lfz test`**：没有 `-e`、没有 stdin 管道、没有 REPL（`run -` 会把 `-` 当文件名 → `IOError`）。**把程序写成 `.lfz` 文件再 `run`**。

---

## 0.5 Agent 标准工作流（5 步）——照做，别跳步

> 目标：把"写 LFZ"变成一条**可执行流程**，每步都有**可验证产出**。任何一步没做，都算没完成。

**第 1 步 · 确认头部**：新建的每个 `.lfz` 文件，**第一行恰好是 `#42` + 一个换行**（见 §1）。片段不算交付。

**第 2 步 · 选特性**：只从本文件已有的语法里挑——变量/类型/运算符看 §3，要用的内置函数逐个到 §6 核对**名字、参数、data-last 顺序**。**这一步查不到的名字 = LFZ 没有这个功能**，不要猜、不要用别的语言替代。拿不准时回到顶部硬纪律。

**第 3 步 · 写代码**：一条语句一行；把 §4 的 21 条陷阱当"负面清单"逐条回避。

**第 4 步 · 立刻跑（强制，不可省）**：写完**马上**运行，**必须看到退出码 `0`**：

```bash
$env:Path += ";$env:USERPROFILE\.cargo\bin"
cargo run --quiet -- run <file.lfz>   # 期望：正确 stdout + exit=0
$LASTEXITCODE                         # 必须打印 0
```

出现报错（退出码 `2`）→ 用 **§4.5「错误类 → 原因 → 修法」**定位，改完**再跑**。**没跑过 = 没写完。**

**第 5 步 · 交付前自检**：逐条过 **§8 自检清单**，全勾后再交。

---

## 1. ⚠️ 铁律（第 1 条，最先看）：`.lfz` 文件首行必须是 `#42`

**规则**：任何 **`.lfz` 文件**的**第一行必须恰好是 3 个字符 `#42`**，其后**紧跟一个行终止符**（`\n`、`\r\n`、`\r`）。

- 违反 → 程序以 **`CosmosAnswerError: 你忘记了宇宙的答案`** 终止（退出码 `2`）。
- **只按扩展名触发**：扩展名（ASCII 大小写不敏感）为 `lfz` 才要求，`.lfz` / `.LFZ` / `.Lfz` 都算。
- **豁免**：非 `.lfz` 文件（如 `.txt`）、以及 REPL / stdin / `run -e`（spec 定义为豁免入口；本 CLI v1 未实现后三者）。
- **无任何变体**（违反即 `CosmosAnswerError`）：`# 42`、`#42 `（尾随空格）、`#42\t`、`##42`、`#43`、`#42abc`、首行前的空行/空格/Tab。
- **只有 3 字节 `#42` 而无换行符** → 也不行。**空 `.lfz` 文件** → 也不行。
- **BOM 豁免**：文件开头可有 UTF-8 BOM（`EF BB BF`），跳过后再要求 `#42`。
- **`#` 的位置**：只在 `.lfz` **首行前导位**合法。程序中间（CODE 模式）出现 `#` → `SyntaxError`（消息 `'#' 只能出现在文件首行的前导位；(字符串 / 注释 / 格式说明符内的 '#' 除外)`）。**字符串 / 注释 / 格式说明符内的 `#` 是普通字符**。
- **片段示例**：若只给代码片段，须注明「作为 `.lfz` 文件时首行须为 `#42`」。

**完整最小程序**（可直接运行）：

```lfz
#42
print("Hello, LFZ!")
```

实测：`cargo run --quiet -- run docs\guide\ai\examples\01_hello.lfz` → `Hello, LFZ!`（退出码 `0`）。

---

## 2. 错误类清单（12 个具体类 + 基类）

> 用户可见输出**只有类名 + 中文消息**，**不存在任何编号错误码**（不要写、也不要期待形如「字母-数字」的内置错误编号）。
> `check` 失败**不抛错**（见 §5）。

| 错误类 | 触发条件 | 中文消息示例 | 阶段 |
|---|---|---|---|
| `LfzError`（基类） | 不直接抛出 | — | — |
| `CosmosAnswerError` | `.lfz` 文件缺合法 `#42` 前导 | `你忘记了宇宙的答案` | 加载 |
| `SyntaxError` | 词法/语法错：非法字符、字符串未闭合、块注释未闭合、未列举转义、整数越界、意外记号、单 `;`、同行两语句、赋值目标非法、`#`/`_` 位置非法、管道多 `_`、`break`/`continue` 在循环外、`return` 在函数外、非 UTF-8 | `非法字符 '$'`；`单独的 ';' 非法；打印变量请用 ';;'`；`语句之间必须有换行`；`文件不是合法的 UTF-8 编码（首个非法字节位于字节偏移 {off}）` | 加载/解析 |
| `NameError` | 引用未定义的名字 | `未定义的名字 'x'` | 运行 |
| `TypeError` | 运算符/条件/调用/参数/格式说明符类型不符；调用非函数；管道右侧非函数；参数个数不符；**重绑定 `let` 变量** | `条件必须是 bool，得到 int`；`运算符 '+' 不支持 int 与 string`；`不能重新赋值 let 变量 'a'；let 只锁重绑定，不锁内容` | 运行 |
| `IndexError` | 数组下标越界（含负索引规范化后越界） | `下标 5 越界（长度 2）` | 运行 |
| `FieldError` | struct 缺该字段（`.k` / `["k"]` 读取缺失键；`del` 的键不是数据字段） | `结构体没有字段 'missing'` | 运行 |
| `ZeroDivisionError` | `/`、`%`、`div(a,b)` 的除数为零（**含 float**） | `除以零` / `对零取模` | 运行 |
| `OverflowError` | `int` 运算超出 i64；`int(±Inf)`；`int(float)` 截断后超 i64 | `整数溢出：结果超出 i64 范围` | 运行 |
| `ValueError` | 转换失败（`int("abc")`、`int(NaN)`）；非法格式说明符；空数组取极值；`randInt(lo>=hi)` | `无法把 string 转换为 int（'abc'）`；`格式说明符非法：'{spec}'`；`空数组没有极值（min）`；`区间非法：3 >= 2` | 运行 |
| `IOError` | `input()` 遇 EOF；不可读文件 | `输入结束（EOF）`；`无法读取：{path}` | 运行 |
| `AssertionError` | **仅** `assert(cond,msg)` 失败 或 `fail(msg)` | `断言失败：{msg}`；`{msg}`（`fail`） | 运行 |
| `RecursionError` | 求值帧深度 > 10000，或深结构处理超限 | `递归深度超限（超过 10000 层）` | 运行 |

**实测消息样本**（`--json` → 退出码均为 `2`）：

```
int("abc")            → ValueError: 无法把 string 转换为 int（'abc'）
int(float("nan"))     → ValueError: 无法把 float 转换为 int（'nan'）
int(float("inf"))     → OverflowError: 整数溢出：结果超出 i64 范围
min([])               → ValueError: 空数组没有极值（min）
1 / 0                 → ZeroDivisionError: 除以零
[1,2][5]              → IndexError: 下标 5 越界（长度 2）
{ k: 1 }.missing      → FieldError: 结构体没有字段 'missing'
对 let 变量再次赋值    → TypeError: 不能重新赋值 let 变量 'a'；let 只锁重绑定，不锁内容
9223372036854775807+1 → OverflowError: 整数溢出：结果超出 i64 范围
```

**错误输出形态**：加载/解析期错误（`CosmosAnswerError` / `SyntaxError`）**无** `Traceback` 头；运行期错误（其余 10 类）**有** `Traceback (most recent call last):` 头。示例（运行期）：

```
Traceback (most recent call last):
  File "demo.lfz", line 2, in <module>
    let r = 1 / 0
            ^
ZeroDivisionError: 除以零
```

---

## 3. 语法速查（骨架）

> 以下片段若作为 `.lfz` 文件，**首行须为 `#42`**。为省版面，标题外的块示例只在完整程序中标注。

**变量与绑定**

```lfz
let x = 1        // let：不可重绑定（内容可改，见 §4-4）
var y = 2        // var：可重绑定
y = y + 1
```

**字面量与类型**：`int`（`1_000`、`0xFF`、`0b1010`、`0o755`）、`float`（`3.14`、`2.5e-3`、`1e10`）、`string`（`"..."`，**只有双引号**）、`bool`（`true`/`false`）、`nil`、`array`（`[1, 2, 3]`）、`struct`（`{ k: 1 }` 或 `Name { ... }`）、`function`。

**运算符（高→低，全左结合）**：postfix `() [] .` > 一元 `- !` > `* / %` > `+ -` > `|>` > `< <= > >=` > `== !=` > `&&` > `||`。

**字符串插值**：`"${expr}"`；可带格式说明符 `${x:>6}`、`${f:.2f}`、`${n:05d}`、`${n:x}`。
转义仅这些：`\n \t \r \\ \" \e`（ESC）、`\$`。**未列出的转义（如 `\q`、`\{`）→ `SyntaxError`**。`{` / `}` 在字符串内是普通字符。

**if（语句 / 表达式）**

```lfz
if cond { ... } else { ... }
let v = if cond { a } else { b }
```

**while / for**

```lfz
while cond { ... }
for x in xs { ... }        // array（迭代开始时取元素快照）
for k in s { ... }         // struct 数据字段键，按字节序升序
```

**函数 / lambda**

```lfz
fn add(a, b) { a + b }        // 块体：末尾表达式即返回值
fn add(a, b) => a + b         // 表达式体
let dbl = (x) => x * 2        // lambda
let inc = fn(x) { x + 1 }     // 函数字面量
```

**结构体（模板 + 方法）**

```lfz
struct Point {
    x: 0,
    y: 0,
    fn norm2() => self.x * self.x + self.y * self.y,
}
let p = Point { x: 3, y: 4 }
print(p.norm2())
```

**管道 `|>`（data-last）**

```lfz
let r = [1, 2, 3, 4] |> filter((x) => x % 2 == 0) |> map((x) => x * 10) |> sum()
let q = 10 |> push(_, [1, 2])      // _ 占位：注入到该位 → push(10, [1, 2])
```

**`;;`（变量 dump）**：独立一行，把当前可见变量按 `name ： value` 写 **stdout**（内层→外层、遮蔽去重）。

**注释**：`//` 行注释；`/* ... */` 块注释（**不可嵌套**，未闭合到 EOF → `SyntaxError`）。

**赋值**：`=` `+=` `-=` `*=` `/=` `%=`；左侧 lvalue 可为 `x`、`a[i]`、`s.k`。

---

## 4. 常见陷阱（AI 最易踩；均已实测）

> 这些是 LFZ 与 Python/C 的**刻意差异**，逐条给「错误写法 → 结果 → 正确写法」。
> 本节片段若作为 `.lfz` 文件，首行须为 `#42`。

1. **`;` 是非法语句分隔符**（不是必需，也不是可省略）。
   - 错误：`let x = 1; let y = 2` → `SyntaxError: 语句之间必须有换行`；单独 `;` → `SyntaxError: 单独的 ';' 非法；打印变量请用 ';;'`。
   - 正确：**一条语句一行**，不要写 `;`。**打印变量用 `;;`**（独立一行）。

2. **`/` 恒为真除法，返回 `float`**：`7 / 2 == 3.5`（不是 3）。
   - 整数向下取整用内置 **`div(a, b)`**：`div(7, 2) == 3`、`div(-7, 2) == -4`。

3. **`%` 为 Python 取模，结果符号随除数**：`-7 % 3 == 2`、`7 % -3 == -2`。

4. **`let` 不可重绑定，`var` 可以**：`let a = 1` 后再 `a = 2` → `TypeError`。
   - 但**容器内容可原地改**：`let a = [1]; a[0] = 2` **合法**（`let` 只锁重绑定，不锁内容）。

5. **`if` 是表达式**，可赋值：`let g = if c { "high" } else { "low" }`。

6. **换行终结语句；续行只能用括号**（`()` / `[]` / 字面体 `{}`）。
   - 错误：`let x = 1 +\n2`、行首以 `|>` 或 `.` 续行 → `SyntaxError`。
   - 正确：`let x = (1 +\n 2)`。

7. **`=>` 后紧跟的 `{` 恒为块**：`fn f() => { 1 }` 是块体。
   - 要在箭头位返回 struct 字面量必须加括号：`fn f() => ({ k: 1 })`。

8. **`${...}` 内是 CODE 模式**：里面的字符串用**普通双引号**。
   - 正确：`print("x = ${"a" + "b"}")`。
   - 错误：在 `${}` 内写 `\"`（`\` 在 CODE 模式非法）→ `SyntaxError: 非法字符 '\'`。插值内**禁止换行**。

9. **管道是 data-last**：`xs |> f(a)` = `f(a, xs)`（无 `_` 时把 `xs` 追加为**末参**）。
   - `_` 占位：`10 |> push(_, [1, 2])` = `push(10, [1, 2])`。
   - `_` 只能出现在**管道右侧调用的实参**中，且**不得穿 lambda**：`xs |> map((x) => x + _)` → `SyntaxError`。

10. **容器内置返回新值，不改原容器**（函数式更新）：
    - `let ys = push(9, xs)` —— **不**改 `xs`；`sort(xs)` 不改 `xs`。
    - 唯一原地改的是**赋值语法** `a[i] = v` / `s.k = v`。

11. **54 个内置按 data-last 约定**（见 §6）；容器/字符串被操作对象恒为末参。

12. **`round` 是银行家舍入（四舍六入五成双）**：`round(2.5) == 2`、`round(3.5) == 4`。

13. **`int(float)` 边界**：向零截断 `int(2.9) == 2`、`int(-2.9) == -2`；`int(float("nan"))` → `ValueError`；`int(float("inf"))` / `int(1e30)` → `OverflowError`。

14. **`float` 特值**：`sqrt(-1)` → `nan`；`float("inf")` / `float("nan")` 合法；显示为 `nan` / `inf` / `-inf`；整值浮点带 `.0`（`str(1.0) == "1.0"`）。

15. **关键字不能作字段名**：`let` `var` `fn` `return` `if` `else` `while` `for` `in` `break` `continue` `struct` `true` `false` `nil` `self`。
    - 错误：`r.self`、`{ self: 1 }` → `SyntaxError`。
    - 正确：用字符串键 `r["self"]`、`{ "self": 1 }`。

16. **无 truthiness、无隐式转换**：`if`/`while`/`&&`/`||`/`filter` 谓词等**必须是 `bool`**，否则 `TypeError`。唯一隐式转换是 `int → float` 加宽（`|int| > 2^53` 时可能不精确）。

17. **`==` 是深结构相等且环安全**：`array`/`struct` 按内容；`function` 仅**同一性**；自引用容器显示 `<cycle>`，不报错、不死循环。

18. **空数组取极值报错**：`min([])` / `max([])` / `minBy(...)` / `maxBy(...)` → `ValueError: 空数组没有极值（min）`。

19. **字符串只用双引号**，没有单引号；不存在 `->`、`++`、`..`、`|`、`&`、`::`、`>>` 等记号（会拆开或报错）。

20. **`insert` 不支持负索引**（合法域 `i ∈ [0, len]`）；`removeAt`/`swap` 支持负索引。`pop([])` → `IndexError: 下标 -1 越界（长度 0）`。

21. **`del(k, s)` 只作用于数据字段**（方法字段视为缺失 → `FieldError`）：不变量 `del(k,s) 成功 ⟺ has(k,s) == true`。

---

## 4.5 错误类 → 最常见原因 → 修法（把 §2 的错误类与 §4 的陷阱串起来）

> 用法：**先看报错类名**（§2），再到本表定位"你大概率踩了哪条"，按"改法"动手，然后回 §0.5 第 4 步重跑。
> 记住：**报错信息里没有编号**，只有**类名 + 中文消息**；消息里的中文关键字是排查的第一线索。

| 你看到的（类名 / 消息关键字） | 大概率踩了 | 改法（正确写法） |
|---|---|---|
| `CosmosAnswerError: 你忘记了宇宙的答案` | 首行不是恰好 `#42`+换行（§1） | 首行**只有** `#42` 三字符（无空格、无尾随、无前导空行） |
| `SyntaxError: 语句之间必须有换行` | 用 `;` 分隔语句 / 一行两语句（§4-1） | **一条语句一行**，删掉 `;` |
| `SyntaxError: 单独的 ';' 非法；打印变量请用 ';;'` | 想用 `;` 打印变量（§4-1） | 打印变量用**独立一行的 `;;`** |
| `SyntaxError: 非法字符 '\'` | `${...}` 内写了 `\"`（§4-8） | `${}` 内是 CODE 模式 → 用**普通双引号 `"`**；插值内禁止换行 |
| `SyntaxError: 非法字符 '…'`（其它符号） | 用了别的语言的记号（`->` `++` `..` `\|` `&` `::` `>>` `'`）（§4-19） | 换 LFZ 等价写法；字符串**只用双引号** |
| `SyntaxError` 出现在跨行处 | 行尾挂运算符 / 行首 `.` `\|>` 续行（§4-6） | 断行**必须放进括号内**（`()` `[]` `{}`） |
| `SyntaxError: '#' 只能出现在文件首行的前导位…` | 在程序中间写了 `#`（§1） | `#` 只允许在 `.lfz` 首行前导位 |
| `SyntaxError`（块注释/字符串未闭合相关） | 忘记 `*/` / `"`，或块注释嵌套（§3） | 补闭合；块注释**不可嵌套** |
| `SyntaxError`（字段位） | `.self` / `{ self: 1 }` 等关键字当字段名（§4-15） | 用**字符串键**：`r["self"]` / `{ "self": 1 }` |
| `SyntaxError`（管道内） | `_` 出现在多参 / 穿 lambda（§4-9） | `_` 只能在**管道右侧调用的实参**里，且**不得穿 lambda** |
| `TypeError: 不能重新赋值 let 变量 'x'` | 对 `let` 变量重绑定（§4-4） | 需要重赋值改用 **`var`**；只改内容用 `a[i]=` / `s.k=`（合法，`let` 只锁重绑定） |
| `TypeError: 条件必须是 bool，得到 int` | 用了 truthiness（§4-16） | 显式写 `x != 0` / `len(xs) > 0` |
| `TypeError: 运算符 '+' 不支持 int 与 string` | 无隐式转换（§4-16） | 用 **`str(x)`** 显式转换再拼接 |
| `TypeError`（调用非函数 / 参数个数不符 / 管道右侧非函数） | 名字拼错，或参数顺序没按 **data-last**（§3 管道、§6） | 到 **§6** 核对签名；管道默认把左值注入**末参** |
| `NameError: 未定义的名字 'x'` | 变量/函数名拼错，或未定义就用（§2） | 先定义后使用；检查拼写与作用域 |
| `IndexError: 下标 … 越界（长度 …）` | 下标越界 / `pop([])` / `insert` 负索引（§4-20） | 用 `len` 判界；`insert` 合法域 `[0, len]` |
| `FieldError: 结构体没有字段 'k'` | 读了缺失键，或 `del` 的是方法字段（§4-21） | 先 **`has(k, s)`** 判断再读 |
| `ZeroDivisionError: 除以零` / `对零取模` | `/` `%` `div` 除数为 0（§2） | 先判零；整除用 `div(a,b)` |
| `OverflowError: 整数溢出…` | `int` 越 i64，或 `int(±Inf)` / `int(1e30)`（§2） | 收窄数值，或改用 `float` |
| `ValueError: 无法把 string 转换为 int（'abc'）` | `int("abc")` 转换失败（§2） | 只对数字串 `int`，或先校验字符 |
| `ValueError: 空数组没有极值（min）` | 对空数组 `min/max/minBy/maxBy`（§4-18） | 先判 `len(xs) > 0` |
| `ValueError: 格式说明符非法：…` | 格式串拼错（§3 插值） | 用 `:>6` `:.2f` `:05d` `:x` 这些形式 |
| `ValueError: 区间非法：lo >= hi` | `randInt(lo, hi)` 且 `lo >= hi`（§6） | 保证 `lo < hi`（区间为 `[lo, hi)`） |
| `IOError: 输入结束（EOF）` | 在无输入环境用了 `input()`（§6） | 别依赖 stdin（本 CLI v1 无 stdin 管道） |
| `AssertionError: 断言失败：…` | `assert`/`fail` 条件不成立（§5） | 复核条件；**软校验用 `check`**（非致命） |
| `RecursionError: 递归深度超限（超过 10000 层）` | 递归无终止条件 / 太深（§2） | 补终止条件，或改写成 `while`/`for` 循环 |

---

## 5. `check` 非致命 / `assert` 致命

| 函数 | 失败时行为 |
|---|---|
| `assert(cond, msg?)` | 抛 **`AssertionError`（致命）**，终止，退出码 `2` |
| `check(cond, msg?)` | 向 **stderr** 写一行 `check 失败：{msg}`，返回 `false`，**继续运行**，**不改变退出码、不产生错误** |
| `fail(msg?)` | 抛 **`AssertionError`（致命）** |

实测：

```lfz
#42
check(1 == 2, "soft")
print("continue")
```

→ stdout：`continue`；stderr：`check 失败：soft`；**退出码 `0`**。把 `check` 换成 `assert` 则退出码 `2`。

---

## 6. 内置函数 54 个（data-last）

> **约定**：凡对容器 / 字符串操作者，**被操作对象恒为最后一个参数**；`xs |> f(...)` 把 `xs` 注入末参。**凡容器更新一律返回新值、不改原容器**（A1），除非用赋值语法 `a[i]=` / `s.k=`。参数类型/个数不符 → `TypeError`。

**核心 / 数组（21）**：`len` `range` `push` `pop` `removeAt` `insert` `swap` `slice` `min` `max` `sum` `minBy` `maxBy` `sort` `sortBy` `map` `filter` `reduce` `take` `drop` `each`

- `push(v, xs)`、`pop(xs)`、`removeAt(i, xs)`、`insert(i, v, xs)`、`swap(i, j, xs)`、`slice(from, to, xs)`
- `map(f, xs)`、`filter(pred, xs)`（`pred` 必须返回 `bool`）、`reduce(f, init, xs)`（`f(acc, x)`）、`take(n, xs)`、`drop(n, xs)`、`each(f, xs)`
- `min(xs)`/`max(xs)`（全序；空 → `ValueError`）、`minBy(keyFn, xs)`/`maxBy(keyFn, xs)`、`sum(xs)`（全 `int` → `int`，含 `float` → `float`，空 → `0`）、`sort(xs)`/`sortBy(keyFn, xs)`（稳定升序）

**struct（5）**：`keys` `values` `entries` `has` `del`

- `keys(s)`/`values(s)`/`entries(s)` 按键的 **UTF-8 字节序升序**；只含**数据字段**（方法字段不含）。
- `has(k, s) -> bool`；`del(k, s) -> struct`（新 struct；键不是数据字段 → `FieldError`）。

**字符串（8）**：`split` `join` `trim` `upper` `lower` `replace` `repeat` `startsWith`

- `split(sep, s)`（`sep` 为空串 → 按字符）、`join(sep, xs)`（元素须为 string）、`replace(old, new, s)`、`repeat(n, s)`、`startsWith(prefix, s)`。

**数值 / 转换 / IO / 断言（20）**：`abs` `floor` `ceil` `round` `sqrt` `pow` `div` `rand` `randInt` `seed` `str` `int` `float` `type` `print` `eprint` `input` `assert` `check` `fail`

- `div(a, b) -> int`（向下取整；两参须 `int`）。
- `round(f) -> int`（银行家舍入）；`floor`/`ceil`/`round` 的 `NaN` → `ValueError`，`±Inf`/超界 → `OverflowError`。
- `abs(x)` **同型**（`int→int`、`float→float`，绝不加宽）；`sqrt(f) -> float`（`f<0` → `NaN`）；`pow(a,b) -> float`。
- `int(x)`：`string`/`float`/`bool` → `int`；`float` 向零截断；`NaN` → `ValueError`，`±Inf`/超界 → `OverflowError`。
- `float(x)`：`int`/`string`/`bool` → `float`；支持 `"inf"`/`"-inf"`/`"nan"`。
- `type(x) -> string`：`"int"` / `"float"` / `"string"` / `"bool"` / `"nil"` / `"array"` / `"struct"` / `"function"`。
- `randInt(lo, hi) -> int`（`[lo, hi)`；`lo >= hi` → `ValueError`）；`rand() -> float`（`[0,1)`）；`seed(n)` 固定随机序列。
- `print(...)`（空格连接 + 换行，stdout）、`eprint(...)`（stderr）、`input(prompt?)`（EOF → `IOError`）。

> `sqrt` / `pow` 之外的数学内置（`sin` / `log` / `exp` 等）**v1 不提供**。

---

## 7. 可运行示例（分级，含真实输出）

> 全部示例在仓库根目录以 `cargo run --quiet -- run <path>` 实跑通过（退出码 `0`）。源文件见 `docs/guide/ai/examples/`。

### L1 — Hello

```lfz
#42
print("Hello, LFZ!")
```

输出：
```
Hello, LFZ!
```

### L2 — 类型 / 算术陷阱 / `if` 表达式 / 插值 / `;;`

```lfz
#42
let name = "LFZ"
var count = 3
count = count + 1

print("hello, ${name}")
print("7 / 2 = ${7 / 2}")
print("div(7, 2) = ${div(7, 2)}")
print("-7 % 3 = ${-7 % 3}")
print("round(2.5) = ${round(2.5)}")
print("round(3.5) = ${round(3.5)}")

let grade = if count > 3 { "high" } else { "low" }
print("grade = ${grade}")

;;
```

输出：
```
hello, LFZ
7 / 2 = 3.5
div(7, 2) = 3
-7 % 3 = 2
round(2.5) = 2
round(3.5) = 4
grade = high
name ： LFZ
count ： 4
grade ： high
```

### L3 — struct + 方法 + 管道 + lambda + 插值（稍复杂综合）

```lfz
#42
struct Student {
    name: "",
    score: 0,
    fn isTop() => self.score >= 90,
    fn line() => "${self.name}: ${self.score:>3}",
}

fn average(xs) {
    var total = 0.0
    for s in xs { total += float(s.score) }
    total / float(len(xs))
}

let roster = [
    Student { name: "Alice", score: 93 },
    Student { name: "Bob", score: 67 },
    Student { name: "Cara", score: 88 },
]

let ranked = roster |> sortBy((s) => -s.score)
print("ranking (desc):")
for s in ranked { print("  " + s.line()) }

let names = roster |> map((s) => s.name) |> join(", ")
print("members: ${names}")
print("average: ${average(roster):.2f}")
let top = roster |> maxBy((s) => s.score)
print("top: ${top.name} (${top.score})")
```

输出：
```
ranking (desc):
  Alice:  93
  Cara:  88
  Bob:  67
members: Alice, Bob, Cara
average: 82.67
top: Alice (93)
```

### L4 — struct + 方法（带参）+ `while` + 管道 + 插值（词频统计，综合工作流范例）

> 源文件：`docs/guide/ai/examples/04_wordcount.lfz`。演示 §0.5 工作流的完整闭环。

```lfz
#42
struct Counter {
    counts: {},
    total: 0,
    fn bump(w) {
        if has(w, self.counts) {
            self.counts[w] = self.counts[w] + 1
        } else {
            self.counts[w] = 1
        }
        self.total += 1
    },
    fn top(n) => self.counts |> entries() |> sortBy((e) => -e[1]) |> take(n),
    fn report() => "total=${self.total}, distinct=${len(self.counts)}",
}

fn count_all(ws) {
    let c = Counter {}
    var i = 0
    while i < len(ws) {
        c.bump(ws[i])
        i += 1
    }
    c
}

let text = "the quick brown fox jumps over the lazy dog the fox jumps the"
let words = text |> split(" ")

let c = count_all(words)
print(c.report())
print("top 3 words (desc):")
for e in c.top(3) {
    print("  ${e[0]}: ${e[1]:>2}")
}
```

输出（实测，exit `0`）：
```
total=13, distinct=8
top 3 words (desc):
  the:  4
  fox:  2
  jumps:  2
```

> 注意点：`entries(s)` 返回 `[[key, value], ...]`，按**键升序**；`sortBy` 稳定，故并列计数按键序；`take(n, xs)` 用管道写作 `... |> take(n)`（data-last）。

---

## 8. 交付前自检清单（逐项打勾）

- [ ] 每个 `.lfz` 文件**首行恰好 `#42` + 换行**？（无变体、无前导空白）
- [ ] 没有用 `;` 作语句分隔或打印变量？（打印变量用 `;;`，独立一行）
- [ ] 是否把 `/` 误当整除？（整除用 `div`）
- [ ] 有没有对 `let` 变量重绑定？（不可；改用 `var`，或只改容器内容 `a[i]=`）
- [ ] 所有条件/逻辑运算/`filter` 谓词都是 `bool`？（无 truthiness）
- [ ] 跨行是否都在括号内？（行尾运算符、行首 `.`/`|>` 不续行）
- [ ] `${}` 内是否误写 `\"`？（应写普通 `"`；插值内不换行）
- [ ] 管道是否 data-last / `_` 位置合法（在 RHS 实参内、不穿 lambda）？
- [ ] 是否依赖容器内置**原地修改**？（须接收返回的新值）
- [ ] 字段名是否误用了关键字？（用 `r["self"]`）
- [ ] 字符串是否用了**单引号**？（LFZ 只有双引号）
- [ ] 是否写/期待了旧的编号错误码？（**禁止**；只有类名 + 中文消息）
- [ ] 代码块是否都含 `#42` 首行？（片段是否注明）

---

## 9. 提示模板（复制即用）

```
你是一名 LFZ 程序员。只允许使用后面给出的《LFZ 编程指南》中列出的语法，
禁止联想到 Python/C/JS 的语法来补全。输出必须是**完整可运行**的 `.lfz` 文件
（含首行 #42），不要伪代码、不要零散片段。

任务：<在此描述你要写的小程序，例如：读取一个整数数组，用管道 filter/map/reduce 求和并打印>

交付要求：
1. 每个 `.lfz` 文件首行恰好为 #42；
2. 自己对照指南 §8 自检清单逐项检查；
3. 给出预期的 stdout 输出。
```

**最小任务基线**（用来验证「只看指南能否写对」）：写一个函数，接收整数数组，返回偶数元素之和；用管道 `filter`/`reduce` 实现；打印结果。`[1,2,3,4,5]` → `6`。参考解（实测通过）：

```lfz
#42
fn sum_even(xs) {
    xs |> filter((x) => x % 2 == 0) |> reduce((acc, x) => acc + x, 0)
}
print(sum_even([1, 2, 3, 4, 5]))
```

输出：`6`

---

## 10. 权威来源

- 语法：`docs/spec/syntax.md`｜语义：`docs/spec/semantics.md`｜接口/错误/内置表：`docs/spec/interface-contract.md`（冻结 v1）。
- runner/CLI 契约：`docs/tooling/runner-contract.md`。
- 本指南的实测验证记录：同目录 `VERIFICATION.md`。
- 本 skill 包的**安装与使用**：同目录 `README.md`（三种安装方式 + 运行前提 + 不做什么）。
- 示例源码目录：`docs/guide/ai/examples/`（`01_hello` / `02_basics` / `03_students` / **`04_wordcount`**）。
- 人类向正文档：`docs/guide/ai/README.md`（与本文件内容一致）。
