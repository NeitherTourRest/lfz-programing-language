---
name: lfz-programming
description: 编写、修改或调试 LFZ 语言（.lfz）程序时使用，也用于回答 LFZ 语法/语义问题。给出 #42 首行铁律、语法速查、12 个错误类清单、LFZ 特有的常见陷阱与可运行示例；在本仓库用 `cargo run --quiet -- run <file.lfz>` 运行。
---

# LFZ 编程指南（AI 专用）

> 读者：**未接触过 LFZ 的编程 Agent**。目标：只凭本文件写出**正确、可运行**的 LFZ 程序。
> 事实源：`docs/spec/{syntax,semantics,interface-contract}.md`（含 T12 §16 / T14 §17 / T15 §18 / T16 §19–21）。本文件是**速查/转述**，不定义新语法；与 spec 冲突时以 spec 为准。
> 使用纪律：**只允许使用本文件列出的语法**。不要用 Python / C / JS 的规则去猜 LFZ——LFZ 在若干处刻意不同（见 §4）。
> **版本对齐（2026-10-29，候选）**：项目内 `dist/lfz.exe` / `target/release/lfz.exe` 为 **1.4.0，73 个内置**（54+15+3+1），支持 argv、JSON、`-e`、stdin源码及非执行`check`。T12 示例实跑证据见 `docs/reports/T12-skill-evidence.md`；项目外 PATH 中的旧解释器不保证具备这些能力。

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
- **退出码（按语境区分，详见 §5）**：`0` 成功；**`lfz run` 下 `assert` / `fail` 失败（运行期错误）→ `2`**；**`lfz test` 下有用例失败 → `1`**；CLI 参数错误 / 语法错误 / 运行时错误 / 用例 error → `2`。
- **1.4.0 入口**：`lfz run FILE [-- ARG...]`、`lfz FILE.lfz [-- ARG...]`、`lfz -e CODE [-- ARG...]`（也可 `run -e`）、`lfz - [-- ARG...]`（也可 `run -`）。`-e`/stdin 源码不加 `#42`；REPL 未实现，无参调用仍报错。参数/JSON 工作流见 §6.2。

---

## 0.5 Agent 标准工作流（6 步）——照做，别跳步

> 目标：把"写 LFZ"变成一条**可执行流程**，每步都有**可验证产出**。任何一步没做，都算没完成。

**第 1 步 · 确认头部**：新建的每个 `.lfz` 文件，**第一行恰好是 `#42` + 一个换行**（见 §1）。片段不算交付。

**第 2 步 · 选特性**：只从本文件已有的语法里挑——变量/类型/运算符看 §3，要用的内置函数逐个到 §6 核对**名字、参数、data-last 顺序**。**这一步查不到的名字 = LFZ 没有这个功能**，不要猜、不要用别的语言替代。拿不准时回到顶部硬纪律。

**第 3 步 · 写代码**：一条语句一行；把 §4 的 21 条陷阱当"负面清单"逐条回避。

**第 4 步 · 先检查**：运行 `lfz check <file.lfz>`，exit0才进入执行阶段；检查不求值，具体边界见§0.6。

**第 5 步 · 实跑（强制，不可省）**：按任务授权运行，核对副作用、stdout/stderr及退出码；正常成功场景**必须看到退出码 `0`**：

```bash
$env:Path += ";$env:USERPROFILE\.cargo\bin"
cargo run --quiet -- run <file.lfz>   # 期望：正确 stdout + exit=0
$LASTEXITCODE                         # 必须打印 0
```

出现报错（退出码 `2`）→ 用 **§4.5「错误类 → 原因 → 修法」**定位，改完**再跑**。**没跑过 = 没写完。**

**第 6 步 · 交付前自检**：逐条过 **§8 自检清单**，全勾后再交。

---

## 0.6 先检查，再执行（T14）

依据三spec §17。以下 `lfz` 指本项目候选解释器；Windows可用 `.\dist\lfz.exe`。项目外PATH中的旧版不保证有check。

```powershell
lfz check job.lfz
lfz check --json -e 'print(1 / 0)'
# stdout: {"ok":true,"mode":"check","executed":false}；exit 0
'print(1 / 0)' | lfz check --json -
# 同上；不打印、不触发除零
```

- 三来源：`check FILE`、`check -e CODE`、`check -`。文件沿run的加载规则，`.lfz`首行仍须精确`#42`加换行；非.lfz文件豁免。CODE/stdin不要求、不消费该前导；stdin读UTF-8到EOF。
- check只加载、词法、语法及现有AST深度检查，不求值。正常成功stdout/stderr均空、exit0；前端失败stderr诊断、stdout空、exit2。源内`print`/`eprint`/`;;`/`input`/文件IO/无限循环不执行；读取待检查源码和stdin仍是宿主IO。它不是安全沙箱，也不检查未定义名、类型、运行期arity、IO路径或终止性。
- 合法check加`--json`：成功严格三键`ok:true,mode:"check",executed:false`；前端失败为`ok:false,mode:"check",executed:false`，再接原`error,message,file,line,col,traceback`字段。stdout恰一行、stderr空。带位置traceback只是单个`<module>`定位记录，不是实际执行栈；无位置IO错误line/col为null、traceback为空。
- CLI参数错误是例外：即使有`--json`，仍stderr用法文本、stdout空、exit2。不要无条件json解析任意失败调用。
- `-e`紧随的一项原样作为CODE，包括空串、`--json`、`--`、`--help`。CODE外`--json`可前后重复；check拒绝CODE外任何`--`及脚本参数、缺源/多源/缺CODE/未知选项和`check --help`。帮助用`lfz --help`。
- check通过只代表前端通过。`print(1 / 0)`检查成功，但run会ZeroDivisionError；先check成功，再按任务授权和预期副作用执行run并核对结果。执行阶段参数才放在`--`后。
- CLI `lfz check`与源码内非致命断言`check(cond,msg)`不同；T14未新增内置；当前T15新增attempt后共73。旧run/test JSON不增加mode/executed键。
- 本轮为1.4.0候选，R304仍OPEN，不是正式发布或整体提速声明。新增指南作者实跑另见`docs/reports/T14-skill-evidence.md`；旧v2的8/8、131/131属于原冻结输入，不重新计分。

## 1. ⚠️ 铁律（第 1 条，最先看）：`.lfz` 文件首行必须是 `#42`

**规则**：任何 **`.lfz` 文件**的**第一行必须恰好是 3 个字符 `#42`**，其后**紧跟一个行终止符**（`\n`、`\r\n`、`\r`）。

- 违反 → 程序以 **`CosmosAnswerError: 你忘记了宇宙的答案`** 终止（退出码 `2`）。
- **只按扩展名触发**：扩展名（ASCII 大小写不敏感）为 `lfz` 才要求，`.lfz` / `.LFZ` / `.Lfz` 都算。
- **豁免**：`run` 加载的非 `.lfz` 文件，以及 `-e` / stdin 源码；后两者不要求、不消费 `#42`。REPL 仍未实现。
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

**字符串插值**：`"${expr}"`；一条字符串可含**多个** `${}`（如 `"${a}-${b}-${a}"`，按书写序求值）。

**格式说明符**（写在 `${}` **内**、紧跟 `:`）：
- **对齐** `align` ∈ { `<` 左, `>` 右, `^` 居中 }，可带 **fill** 字符：`${"ab":<5}` → `[ab   ]`、`${"ab":>5}` → `[   ab]`、`${"ab":^5}` → `[ ab  ]`、`${42:0>5}` → `[00042]`、`"${"ab":*^7}"` → `[**ab***]`。
- **宽度 / 精度 / 类型**：`${f:.2f}`、`${n:05d}`、`${n:x}`。
- **宽度必须是字面数字**：`${"ab":>w}` → `ValueError: 格式说明符非法：'>w'`（**不支持动态宽度**）。需要变宽补齐 → 用 `repeat(n, fill)` 手工拼（注意 §4.6 的 O(n) 提醒）。

转义仅这些：`\n \t \r \\ \" \e`（ESC）、`\$`。**未列出的转义（如 `\q`、`\{`）→ `SyntaxError`**。`{` / `}` 在字符串内是普通字符。

**if（语句 / 表达式）**

```lfz
if cond { ... } else { ... }
if a { ... } else if b { ... } else if c { ... } else { ... }   // else if 链：合法
let v = if cond { a } else { b }                                 // if 是表达式，可赋值
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

**赋值**：`=` `+=` `-=` `*=` `/=` `%=`；左侧 lvalue 可为 `x`、`a[i]`、`s.k`，**可链式**：`t[i][j] = v`、`p.inner.x = v`（合法；下标/字段赋值**就地修改**容器）。

**零参 `print()` / `eprint()`**：合法，只打印一个换行（`print(...)` 空格连接各参 + 换行）。

**`&&` / `||` 短路求值**：两侧与结果都必须是 `bool`；`len(xs) > 0 && xs[0] == 1` 在 `xs == []` 时**不会**求右侧（不越界）；`||` 同理。

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

11. **73 个内置按 data-last 约定**（见 §6）；容器/字符串被操作对象恒为末参。

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

22. **字符串不可下标**：`s[i]` 对 `string` → `TypeError: 运算符 '[]' 不支持 string 与 array / struct`。
    - `[]` **只适用于** `array`（`int` 下标，支持负索引）与 `struct`（`string` 键）。**读写同源**（`s[0] = v` 与 `s[0]` 报同一条）。
    - **正确**：先 `split("", s)` **一次性 O(n)** 拆成单字符数组，之后 `cs[i]` / `for c in cs` 都是 **O(1)**：
      ```lfz
      let cs = split("", "abc")   // ["a", "b", "c"]
      print(cs[0])                // a
      for c in cs { print(c) }    // 顺序遍历，整段 O(n)
      ```
    - 复杂度：`split("", s)` = O(n)（n = 字符数）。若改用 `s[i]`（必然实现为 `chars().nth(i)`）遍历会退化成 **O(n²)**——这正是 LFZ 刻意不提供 `s[i]` 的原因。

23. **循环体内 `let` / `var` 每轮都是新绑定**（各自新 cell）：在 `while` / `for` 体里写 `let x = ...` **合法**，**无需**提升到循环外，也**不会**报「不能重新赋值 let」。闭包各自捕获**当轮** cell（A2）：
    ```lfz
    var fs = []
    var i = 0
    while i < 3 {
        let x = i * 10
        fs = push(fn() => x, fs)
        i += 1
    }
    for f in fs { print(f()) }   // 0 / 10 / 20（若共用 cell 会是 20/20/20）
    ```

24. **容器内置返回新值 ⇒ 别在循环里用 `push` / `+` 累积**（A1）：`push(v, xs)` 每次克隆整个数组、`s = s + c` 每次复制整个前缀，**循环累积都是 O(n²)**。大 n 请用 **§4.6** 的 O(n) 写法。

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
| `ValueError: 格式说明符非法：…` | 格式串拼错 / 用了**动态宽度**（§3 插值） | 用 `:>6` `:<6` `:^6` `:0>5`（fill+对齐）、`:.2f`、`:05d`、`:x`；**宽度须字面数字**，动态宽度用 `repeat` 手工补（§3） |
| `TypeError: 运算符 '[]' 不支持 string 与 array / struct` | 对 **string** 取下标（§4-22） | 字符串不可下标 → 先 `split("", s)`，再对数组索引 |
| `ValueError: 区间非法：lo >= hi` | `randInt(lo, hi)` 且 `lo >= hi`（§6） | 保证 `lo < hi`（区间为 `[lo, hi)`） |
| `IOError: 输入结束（EOF）` | 在无输入环境用了 `input()`（§6） | 提供实际输入；stdin 源码模式已读到 EOF，不能再从同一 stdin 提供数据 |
| `AssertionError: 断言失败：…` | `assert`/`fail` 条件不成立（§5） | 复核条件；**软校验用 `check`**（非致命） |
| `RecursionError: 递归深度超限（超过 10000 层）` | 递归无终止条件 / 太深（§2） | 补终止条件，或改写成 `while`/`for` 循环 |

---

## 4.6 性能红线：O(n) 字符串 / 数组构建（避免隐藏 O(n²)）

> 背景：LFZ 的 `string` **不可变**（§4.5.11），`array` 内置**返回新值**（A1）。这两条叠加，使「在循环里逐步累积」极易写成 **O(n²)**。
> 下表为**实测**（当前 `dist\lfz.exe`，循环 N 次，单位毫秒，≈值）。**关键反直觉结论：把 `s = s + c` 换成 `push` 累积并不会变快，反而更慢。**

| 写法（循环 N 次） | N=40000 | N=80000 | N=160000 | N=320000 | 复杂度 |
|---|---|---|---|---|---|
| ❌ `var s=""` + `s = s + "x"` | 61 | 139 | 425 | 1582 | **O(n²)** |
| ❌ `var a=[]` + `a = push("x", a)` + `join("", a)` | 251 | 674 | 4076 | —（超时） | **O(n²)** |
| ✅ `range(n) |> map((i) => "x") |> join("")` | 25 | 35 | 56 | 104 | **O(n)** |
| ✅ 预分配 + 下标写 `arr[i] = ...` + `join` | 33 | — | 85 | 151 | **O(n)** |

**为什么 `s = s + c` 是 O(n²)**：`string + string` 的实现是 `format!("{a}{b}")`——每次生成**新串并复制整个前缀**（源码：`src/evaluator.rs:1496`）。循环 n 次 ⇒ 复制总量 `1+2+…+n = Θ(n²)`。
**为什么 `push` 累积也是 O(n²)**：`push(v, xs)` 遵守 A1「返回新数组」，内部先 `xs.to_vec()` **整体克隆**（源码：`src/builtins.rs:385`）。循环 n 次同样是 Θ(n²)，且常数比字符串拼接更大。**这是最反直觉的点。**

**正确的 O(n) 惯用法（二选一）：**

1. **每段是下标的函数** → `range(n) |> map(f) |> join("")`（`range` + `map` + `join` 各 O(n)，全程只分配一次）：
   ```lfz
   #42
   let n = 5
   let s = range(n) |> map((i) => str(i * i)) |> join(",")
   print(s)   // 0,1,4,9,16
   ```
2. **需要条件累积** → 先**预分配**长度为 n 的数组，再用**下标赋值**（唯一原地写语法 `a[i] = v`，O(1)）逐格写入，最后 `join`：
   ```lfz
   #42
   let n = 3
   let arr = range(n) |> map((i) => "")   // 预分配 n 个空串（O(n)）
   var i = 0
   while i < n {
       arr[i] = str(i * 10)               // 原地写，O(1)
       i += 1
   }
   print(arr |> join("-"))                // 0-10-20
   ```

> 小数据量（n ≤ 几千）差异可忽略；**大 n（≥ 10⁴）务必用上表的 ✅ 写法**。同理，`len(string)` 是 O(n)，长串循环里别反复调用（见 §6 `len`）。

---

## 5. `check` 非致命 / `assert` 致命

| 函数 | 失败时行为 |
|---|---|
| `assert(cond, msg?)` | 抛 **`AssertionError`（致命）**，终止，退出码 `2` |
| `check(cond, msg?)` | 向 **stderr** 写一行 `check 失败：{msg}`，返回 `false`，**继续运行**，**不改变退出码、不产生错误** |
| `fail(msg?)` | 抛 **`AssertionError`（致命）** |

> **退出码按语境区分（消除歧义，实测）**：**`lfz run`** 下 `assert` / `fail` 失败 → `AssertionError` → 退出码 **`2`**（它是一次运行期错误）；**`lfz test`** 下由 runner 把该用例记为 **failure** → 退出码 **`1`**。**同一个含 `assert(1 == 2)` 的程序**：`lfz run` → exit `2`；`lfz test <目录>` → exit `1`（报告 `失败 1`）。不要把两个语境的结果混为一谈。

实测：

```lfz
#42
check(1 == 2, "soft")
print("continue")
```

→ stdout：`continue`；stderr：`check 失败：soft`；**退出码 `0`**。把 `check` 换成 `assert` 则退出码 `2`。

---

## 6. 内置函数 73 个（data-last）

> **约定**：凡对容器 / 字符串操作者，**被操作对象恒为最后一个参数**；`xs |> f(...)` 把 `xs` 注入末参。**凡容器更新一律返回新值、不改原容器**（A1），除非用赋值语法 `a[i]=` / `s.k=`。参数类型/个数不符 → `TypeError`。

**核心 / 数组（22）**：`len` `range` `push` `pop` `removeAt` `insert` `swap` `slice` `min` `max` `sum` `minBy` `maxBy` `sort` `sortBy` `map` `filter` `reduce` `take` `drop` `each` `contains`

- **`len(x) -> int`**：`array` 元素数 / `struct` **数据字段**数（不含方法）/ **`string` 的 Unicode 标量数**（**合法**：`len("abc") == 3`、`len("日本語") == 3`、`len("😀") == 1`、`len("") == 0`）。⚠️ `len(string)` 是 **O(n)**（需扫描标量，非字节数）；长串循环里别反复调用，先 `split("", s)` 后对数组求 `len`（O(1)）。
- **`range(n) -> array[int]`**：`[0, 1, …, n-1]`（**半开区间**，物化为普通数组，O(n) 时间 + O(n) 内存）；`n < 0` → `[]`（`range(0)` 亦为 `[]`）。另支持 **`range(lo, hi) -> array[int]`**：物化半开区间 `[lo,hi)`，`hi <= lo` → `[]`；非空时 O(hi−lo) 时间/内存。只接受 1 或 2 参，均须 int；0 参消息 `函数 range 期待 1 个参数，得到 0`，3 参消息 `函数 range 期待 2 个参数，得到 3`。容量超限 → `OverflowError: 容量溢出：所需容量超出可分配上限`。
- `push(v, xs)`、`pop(xs)`、`removeAt(i, xs)`、`insert(i, v, xs)`、`swap(i, j, xs)`、`slice(from, to, xs)`（**均返回新数组**，见 §4-24 / §4.6）
- `map(f, xs)`、`filter(pred, xs)`（`pred` 必须返回 `bool`）、`reduce(f, init, xs)`（`f(acc, x)`）、`take(n, xs)`、`drop(n, xs)`、`each(f, xs)`
- `min(xs)`/`max(xs)`（全序；空 → `ValueError`）、`minBy(keyFn, xs)`/`maxBy(keyFn, xs)`、`sum(xs)`（全 `int` → `int`，含 `float` → `float`，空 → `0`）、`sort(xs)`/`sortBy(keyFn, xs)`（稳定升序）

**struct（5）**：`keys` `values` `entries` `has` `del`

- `keys(s)`/`values(s)`/`entries(s)` 按键的 **UTF-8 字节序升序**；只含**数据字段**（方法字段不含）。
- `has(k, s) -> bool`；`del(k, s) -> struct`（新 struct；键不是数据字段 → `FieldError`）。

**字符串（13）**：`split` `join` `trim` `upper` `lower` `replace` `repeat` `startsWith` `endsWith` `indexOf` `padEnd` `padStart` `substring`

- `split(sep, s)`（**`sep` 为空串 → 按字符拆成单字符数组**：`split("", "abc") == ["a", "b", "c"]`，一次 **O(n)**；这是字符串「取字符」的正解，见 §4-22）、`join(sep, xs)`（元素须为 string）、`replace(old, new, s)`、`repeat(n, s)`、`startsWith(prefix, s)`。

**数值 / 转换 / IO / 断言（29）**：`abs` `floor` `ceil` `round` `sqrt` `pow` `div` `rand` `randInt` `seed` `str` `int` `float` `type` `print` `eprint` `input` `assert` `check` `fail` `sin` `cos` `log` `exp` `ord` `chr` `readFile` `writeFile` `appendFile`

- `div(a, b) -> int`（向下取整；两参须 `int`）。
- `round(f) -> int`（银行家舍入）；`floor`/`ceil`/`round` 的 `NaN` → `ValueError`，`±Inf`/超界 → `OverflowError`。
- `abs(x)` **同型**（`int→int`、`float→float`，绝不加宽）；`sqrt(f) -> float`（`f<0` → `NaN`）；`pow(a,b) -> float`。
- `int(x)`：`string`/`float`/`bool` → `int`；`float` 向零截断；`NaN` → `ValueError`，`±Inf`/超界 → `OverflowError`。
- `float(x)`：`int`/`string`/`bool` → `float`；支持 `"inf"`/`"-inf"`/`"nan"`。
- `type(x) -> string`：`"int"` / `"float"` / `"string"` / `"bool"` / `"nil"` / `"array"` / `"struct"` / `"function"`。
- `randInt(lo, hi) -> int`（`[lo, hi)`；`lo >= hi` → `ValueError`）；`rand() -> float`（`[0,1)`）；`seed(n)` 固定随机序列。
- `print(...)`（空格连接 + 换行，stdout）、`eprint(...)`（stderr）、`input(prompt?)`（EOF → `IOError`）。

### 6.1 本轮新增 15 个内置：完整签名与边界

被操作的容器/字符串放末参；例如 `"日本語" |> substring(1, 3)`，`"内容" |> writeFile("Temp/out.txt")`。字符串操作返回新串。字符指 **Unicode 标量**，不是 UTF-8 字节或字素簇；组合字符/含 ZWJ 的 emoji 可含多个标量。`s[i]` 仍不合法，读取字符用 `split("", s)[i]`。

| 签名 | 已实现行为与复杂度 |
| -- | -- |
| `contains(v, xs) -> bool` | 仅 array；深结构、环安全 `==` 比较。O(n) 次比较（深比较可大于 O(1)）；N 次循环反复搜索长数组为 O(N·M)。struct 键用 `has(k,s)`。 |
| `endsWith(suffix, s) -> bool` | 空后缀 true；O(m)，m 为后缀长度。 |
| `indexOf(sub, s) -> int` | 首次匹配的标量下标；未找到 -1；空 sub 返回 0。保守最坏 O(n·m)，不宣称 O(n)。 |
| `padEnd(width, fill, s) -> string` | 右补至标量宽度；len(s)>=width 返回原串；fill 重复、按标量截断。O(n+m+输出宽度)，包括主串/fill 计数。 |
| `padStart(width, fill, s) -> string` | 左补，其余同 padEnd。 |
| `substring(from, to, s) -> string` | 标量半开区间，两端夹取到 `[0,len(s)]`，from>=to 返回空串；负数不从尾计。非空结果 O(min(to,len(s)))（to 先夹至非负），UTF-8 需扫描前缀；空区间可直接返回。 |
| `ord(c) -> int` | c 须恰好一个标量的 string；成功 O(1)，非法字符串完整统计 n，错误路径 O(n)。非 string → TypeError；标量数≠1 → `ValueError: ord 的参数必须是单个字符（Unicode 标量数 {n}）`。 |
| `chr(n) -> string` | n 须 int，0..1114111 且排除代理区 55296..57343；O(1)。非 int → TypeError；非法码点 → `ValueError: chr 的参数不是合法的 Unicode 码点：{n}`。 |
| `sin(f) -> float` | int 加宽；正弦；±Inf/NaN → NaN，O(1)。 |
| `cos(f) -> float` | int 加宽；余弦；±Inf/NaN → NaN，O(1)。 |
| `log(f) -> float` | int 加宽；自然对数；log(0) → -Inf，负数 → NaN，O(1)。 |
| `exp(f) -> float` | int 加宽；e 的 f 次方；溢出 → +Inf，O(1)。 |
| `readFile(path) -> string` | path 须 string；UTF-8 全量读取，O(文件大小) 时间/内存。 |
| `writeFile(path, contents) -> nil` | 两参须 string；UTF-8 覆盖写，不存在则创建；O(文件大小)。 |
| `appendFile(path, contents) -> nil` | 两参须 string；UTF-8 追加，不存在则创建；O(追加大小)。 |

`padEnd`/`padStart` 的 fill 为空（即使无需补齐）→ `ValueError: 填充串不能为空`；width/from/to 非 int、字符串形参非 string → TypeError。math 采用 IEEE，NaN/Inf 本身不抛 ValueError/OverflowError；转换/取整规则仍见 §6。int 加宽超过 2^53 可能丢精度。

文件路径原样交给 OS；相对路径基于**进程 CWD**（不是脚本目录），不展开 `~`、不规范化、不自动建父目录。当前项目只在项目内读写，临时件放 `Temp/`；内置本身不实施沙箱。错误均为 `IOError`：读失败 `无法读取：{path}`；非法 UTF-8 `无法读取：{path}（不是合法的 UTF-8 编码）`；写失败 `无法写入：{path}`；追加失败 `无法追加：{path}`。`writeFile(p,42)` 不隐式转换，应先 `str(42)`。外部 IO 依赖环境。大量片段遵循 §4.6，避免循环拼接或 push 的 O(n²)。

完整可运行增量示例及 stdout/退出码见 `docs/reports/T11-v11-skill-evidence.md`，使用项目内 `dist/lfz.exe`（1.4.0）。

---


## 6.2 Agent 数据脚本：argv、JSON 与直接执行（1.4.0）

| API | 签名与边界 |
| -- | -- |
| `argv()` | 零参 → 新 `array[string]`；不含程序/脚本名或 `--`；无参数为 `[]`。每次返回新数组，修改它不会改下次参数。 |
| `jsonParse(text)` | 严格单个 JSON → LFZ 值；`null`→`nil`，object→匿名 struct；必须传 string。 |
| `jsonStringify(value)` | JSON 可表示值 → 无换行紧凑 string；对象数据字段按 UTF-8 键序输出。 |

**复制调用模板（PowerShell，仓库根目录）**。其中 `-e` 的 CODE 是片段，若保存为 `.lfz` 则仍须首行 `#42`：

```powershell
.\dist\lfz.exe -e 'print(jsonStringify(argv()))' -- '中文' '--json' ''
# stdout: ["中文","--json",""]
.\dist\lfz.exe -e 'print(jsonStringify(argv()))' --json -- '--json'
# stdout: {"ok":true}；业务 JSON ["--json"] 写 stderr
.\dist\lfz.exe run Temp/job.lfz -- CONFIG.json INPUT.json OUTPUT.json
```

- `--` 后每项原样传给 `argv()`，空串/Unicode/`--json`/`-e`/`--help` 都是数据；无 `--` 的额外参数报 CLI 错误。
- `-e` 下一项**无条件是 CODE**；例如 `lfz -e --json` 把 `--json` 当源码报错，不能把它吞成选项。CODE 之后的 `--json` 才是机器输出选项。
- **业务 JSON**：正常模式 `print(jsonStringify(value))` 写 stdout；**CLI `--json`**：stdout 只写执行状态/错误对象，程序 print/dump/input 提示转 stderr，不返回最后表达式。不要把业务输出与状态对象混在一起解析。
- `lfz -` / `lfz run -` 把 stdin **全部 UTF-8 源码**读到 EOF，错误路径名 `<stdin>`；读失败/非法 UTF-8 为 `IOError: 无法读取标准输入`。此时程序 `input()` 会遇 EOF；代码与数据不要共用这个入口。文件/`-e` 模式仍可用 `input()` 读取单行输入。
- `argv` 是本轮唯一新增的一等内置值：可 `let getArgs = argv` 后调用；`argv == argv` 为 true，`type(argv)` 为 `"function"`，`str(argv)` 为 `"<fn argv>"`。词法变量/形参优先，可遮蔽它；保存的别名仍有效。**JSON 两内置与旧69内置未泛化为一等值**，写 `map((s) => jsonParse(s), texts)`，不要写 `map(jsonParse, texts)`。

**完整程序：参数传配置，管道处理，确定性业务 JSON 输出。** 保存为 `Temp/job.lfz`；传入唯一参数 `{"title":"中文🙂","values":[1,2,3,4]}`（操作系统参数分词由调用者负责）：

```lfz
#42
let args = argv()
assert(len(args) == 1, "需要一个 JSON 配置参数")
let config = jsonParse(args[0])
let values = config.values |> filter((x) => x % 2 == 0) |> map((x) => x * 10)
let result = { title: config.title, count: len(values), total: sum(values) }
print(jsonStringify(result))
```

stdout：`{"count":2,"title":"中文🙂","total":60}`，exit 0。来自文件的 JSON 同样用 `readFile(path) |> jsonParse()`，输出用 `writeFile(path, jsonStringify(value))`；先验证/编码完成，再写文件，避免错误路径覆盖已有输出。无需自写 JSON parser，也不要用字符串拼接构造 JSON。

**严格数据边界**（spec 契约 §16.2–16.3）：

- 整数词法按 i64 精确解析，`9007199254740993` 不降为 float；超出 `[-9223372036854775808,9223372036854775807]` 报错。含小数点/指数的数字为有限 f64；允许舍入/下溢至零，保留负零。编码整值 float 补 `.0`，重新解析仍为 float。
- 禁止 BOM、注释、尾逗号、重复**解码后**对象键、非法前导零、NaN/Inf、裸控制字符及孤立 UTF-16 代理；正确代理对、Unicode/NUL 转义可往返。JSON 文本内 `$`/`${...}` 是数据，不进行 LFZ 插值；序列化不产生非法 JSON 转义 `\$`。
- 解析/编码最多同时打开 **128 层 array/object**；标量深度0，128允许，129拒绝。当前路径上的循环引用拒绝；共享但无环的子对象可在多个位置编码。
- struct 按既有 A5 数据面**跳过函数方法字段**，其余键稳定排序；函数根值、数组函数元素和 struct 模板拒绝。不会调用方法，不把非法值改成 null。
- `ValueError: JSON 解析失败（字节偏移 {offset}）：{reason}`。offset 从0计 **UTF-8 字节**，不是字符数；reason 为 `不是合法的 JSON` / `对象键重复` / `整数超出 i64 范围` / `浮点数超出有限范围` / `嵌套深度超过 128 层`。
- `ValueError: JSON 编码失败：{reason}`，reason 为 `不支持函数或结构体定义` / `浮点数必须有限` / `循环引用` / `嵌套深度超过 128 层`。非string解析参数/数量错误为 TypeError。错误位置仍指 LFZ 内置调用点，run exit2；`--json` 提供结构化诊断。

**效率约束**：继续使用 §4.6 的 map 或预分配，不用循环 push/字符串累积。JSON 单遍字节游标与一次容器构造；解析 O(n+Σk log k)，编码包含展开输出和键排序成本（字符串比较另计）。struct 内部字段索引改善**平均**查找，不承诺最坏 O(1)，排序/复制/编码也不因此变成 O(1)。性能实测由 `docs/reports/T12-performance.md` 提供，本指南不把源码改动直接称为加速。

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

### L5 — 字符串与索引惯用法（覆盖 §4-22 / §4-23 / §4.6 与插值对齐）

> 本示例为 SKILL 内联（人类向示例在 `docs/guide/ai/examples/`）；实测证据见 `VERIFICATION.md` §8。

```lfz
#42
// 字符串不可下标 → 先 split("", s)，之后索引 O(1)
let cs = split("", "LFZ")            // ["L", "F", "Z"]
print(cs)
print(cs[0])
print(cs[2])

// 循环体内 let 每轮新绑定；闭包各自捕获当轮 cell
var fs = []
var i = 0
while i < len(cs) {
    let c = cs[i]
    let f = fn() => upper(c)
    fs = push(f, fs)
    i += 1
}
for f in fs { print(f()) }

// 对齐说明符（< 左 / > 右 / ^ 居中 + fill）；宽度须字面数字
let rows = [["ab", 7], ["c", 128]]
for r in rows {
    print("[${r[0]:<4}|${r[1]:>5}|${r[0]:*^6}]")
}

// else if + 零参 print() + && / || 短路
fn bucket(n) {
    if n >= 100 { "big" }
    else if n >= 10 { "mid" }
    else { "small" }
}
print(bucket(7))
print()
let xs = []
print(len(xs) > 0 && xs[0] == 1)     // false，短路不越界
print(len(xs) == 0 || xs[0] == 1)    // true
```

输出（实测，exit `0`）：
```
["L", "F", "Z"]
L
Z
L
F
Z
[ab  |    7|**ab**]
[c   |  128|**c***]
small

false
true
```

---

## 8. 交付前自检清单（逐项打勾）

- [ ] 已先check再run，分别保存检查和执行结果？check成功未当成运行成功？
- [ ] CLI check与源码check断言区分，usage错误的--json文本例外已处理？

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
- [ ] **没有对 `string` 取下标**？（`s[i]` → `TypeError`；先 `split("", s)` 再对数组索引，见 §4-22）
- [ ] 脚本参数是否放在 `--` 后？业务 JSON 是否与 CLI `--json` 状态区分？JSON 是否用内置而非手写拼接/解析？
- [ ] **大 n 下没有在循环里用 `s = s + ...` 或 `push` 累积**？（O(n²)；用 §4.6 的 `range |> map |> join` 或「预分配 + 下标写」）
- [ ] 循环体内用了 `let`？（**可以**，每轮新绑定——不要因此改用 `var` 提升到循环外，见 §4-23）
- [ ] 退出码是否按语境理解？（`lfz run` 下 assert 失败 → `2`；`lfz test` 下用例失败 → `1`）
- [ ] 格式说明符宽度是否为**字面数字**？（动态宽度不支持 → 用 `repeat` 手工补，见 §3）

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
3. **不要对字符串取下标**（`s[i]` 非法，用 `split("", s)`）；**大 n 时不要在循环里 `s = s + …` 或 `push` 累积**（O(n²)，见指南 §4.6）；
4. 先check再按任务授权run，保存两步真实结果；check不执行，不保证运行成功。
5. 给出预期的 stdout 输出。
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

- 语法：`docs/spec/syntax.md`｜语义：`docs/spec/semantics.md`｜接口/错误/内置表：`docs/spec/interface-contract.md`（含 T12 §16 / T14 §17 / T15 §18 / T16 §19–21）。
- runner/CLI 契约：`docs/tooling/runner-contract.md`。
- 本指南的实测验证记录：同目录 `VERIFICATION.md`（§8 为 T11-③ 的 D1–D7 修订实测证据）。
- 本 skill 包的**安装与使用**：同目录 `README.md`（三种安装方式 + 运行前提 + 不做什么）。
- 示例源码目录：`docs/guide/ai/examples/`（`01_hello` / `02_basics` / `03_students` / **`04_wordcount`**）。
- AI指南入口：`docs/guide/ai/README.md`（与本文件内容同步）；人类向手册：`docs/guide/README.md`。


## 6.3 T15：把业务错误变为可检查的值

事实源为三spec §18；语言内置总数为73（原72加attempt），并未使内置普遍成为一等函数。先用项目内候选 `lfz check` 检查，再run，最后核结果和副作用。

`attempt(f)`恰一参，f必须是可零参调用的函数；只调用一次。成功为新struct `{ok:true,value:v,error:nil}`，失败为 `{ok:false,value:nil,error:e}`，三字段插入序固定。成功nil仍是ok=true；不要用value是否nil判断成功。value保持原引用与函数身份，不深复制。

失败e字段插入序为 `class,message,line,col,traceback`；位置1-based整数、未知位置nil。traceback为完整外→内数组，每帧字段序 `func,line,col`，函数名含`<module>`/`<fn>`。无file字段（解释器上下文没有源路径），它不同于CLI错误JSON的traceback协议。记录是普通数据，改记录不会改解释器内部栈。

```lfz
#42
let lines = ["{\"n\":2}", "bad", "{\"n\":5}"]
var values = []
var errors = []
for line in lines {
 let r = attempt(() => jsonParse(line))
 if r.ok { values = push(r.value, values) }
 else { errors = push(r.error, errors) }
}
assert(len(values) == 2 && len(errors) == 1)
assert(errors[0]["class"] == "ValueError")
print(jsonStringify({values: values, errors: errors}))
```

`class`是保留字，必须用`r.error["class"]`，不能写`r.error.class`。lambda内空struct须写`({}).missing`，`=> {}`本来就是块。

捕获白名单为NameError、TypeError、IndexError、FieldError、ZeroDivisionError、ValueError、IOError、AssertionError、整数越界OverflowError。JSON128深度输入验证属ValueError。容量溢出、RecursionError、SyntaxError、CosmosAnswerError以及宿主panic/abort/分配失败不捕获；不得用attempt吞资源终止。IOError沿原运行时类别，不保证识别所有底层OS资源故障。

参数求值、attempt参数数量/类型和f非零形参校验均在本次捕获边界外，直接报原错误；外层attempt可捕内部发生的TypeError。旧内置非一等值，写`attempt(() => jsonParse(text))`，不要写`attempt(jsonParse)`。argv是既有特例，`attempt(argv)`合法并返回参数快照。attempt自己不是一等函数，不能`let saved=attempt`；可用命名调用/管道，也可按原词法规则遮蔽。

成功及可恢复失败还原调用前depth与完整trace（包括调用者span），嵌套各自独立；不可恢复错保留故障栈传播。func_names/next_func_id不回滚，失败回调写出的逃逸闭包仍有效。

**没有事务回滚**：容器/全局/cell写入、文件写入、print/eprint、随机状态和闭包分配均保留。check(false)只是警告和false，attempt仍成功；assert/fail在边界内是失败值。成功value可含函数或环，不能承诺整个Result一定能jsonStringify。结构化业务失败可以让run退出0，必须自行检查每条ok，不能以进程exit0代表所有记录成功。

实用完整应用见`app/jsonbatch.lfz`；当前readFile整文件驻内存，不是流式读取。新作者实跑见`docs/reports/T15-skill-evidence.md`，不覆盖旧v2冻结成绩。R304仍OPEN，无T15性能测窗，不声明提速或正式发布。


## 6.4 T16-C：本地模块的Agent工作流

事实源三spec §19–21及T16-C-design；C候选支持本地模块执行，73内置不增加。先为每个.lfz保留#42首行，写静态import/export，先check整个根模块图（不初始化、不执行IO），通过后才run并核业务输出。check成功不保证依赖初始化或入口求值成功。不要把多个文件文本拼接成共享globals。

同目录`counter.lfz`：

```lfz
#42
var base = 40
let shared = [1]
fn add(x) => base + x
fn bump() { base += 1
 base }
export base, shared, add, bump
```

同目录`main.lfz`：

```lfz
#42
import lib from "./counter.lfz"
var base = 900
assert(lib.add(2) == 42)
assert(lib.bump() == 41)
assert(lib.base == 40)
lib.shared[0] = 7
print(jsonStringify({answer: lib.add(2), args: argv(), shared: lib.shared}))
```

执行`lfz check --json main.lfz`，应只得到`{"ok":true,"mode":"check","executed":false}`；再`lfz run main.lfz -- '中文'`，业务输出为`{"answer":43,"args":["中文"],"shared":[7]}`。先检查后执行两步都须保留证据。

- 语法仅`import alias from "./relative.lfz"`、`export a, b`，都是直接顶层声明。所有import须在其它语句之前，export也算其它语句；导出只限本模块直接顶层绑定。无动态/远程import、包搜索、from-import或live循环绑定。
- 路径相对导入者文件目录；-e/stdin根以开始捕获cwd为基址。必须无插值相对.lfz路径（可含..）；不补扩展名、不接受绝对/网络路径。不把symlink/..解析当权限沙箱。依赖按canonical身份去重，每次执行独立缓存；同模块多alias/菱形只初始化一次，按导入书写序依赖后序串行执行。
- 模块全局独立。导出函数在定义模块home globals中执行，调用者同名变量不污染它；捕获局部cell和self保持原规则。namespace是普通struct，标量为初始化完成时快照，容器/函数共享原引用拓扑。改namespace标量字段不回写原模块绑定；多alias共享namespace对象。keys/JSON沿A5过滤函数，不是完整函数导出反射。
- 所有模块使用同次argv输入快照，argv()每次仍新数组。跨执行不复用初始化缓存。初始化失败立即停止，尚未运行入口内attempt无法捕获依赖初始化错误，无隐式重试。
- 跨模块未捕获错误的CLI human/JSON使用真实定义文件及逐帧来源，参数/arity错误属于调用者。不要拼路径到func名。attempt仍是既有三/五/三字段且无file，只有各帧行列，不能由它声称完整文件定位；恢复还原module/source上下文但不回滚用户副作用、实例或函数ID。
- 导出模板先`let P = lib.Point`再`P{}`，不支持`lib.Point{}`。graph模板默认表达式在定义globals/captured cells/self上下文、按字段序每次重新求值，可变默认每实例独立；全部默认先完成，显式覆盖才在caller Scope按书写序求值。错误可见`Point::<defaults>`及真实定义文件。无模块legacy模板保留原caller默认值规则，不新增默认帧。
- check完整图在初始化前拒绝循环、别名/绑定冲突、缺导出、坏路径/前导/UTF8/语法；读取未导出namespace字段是运行时FieldError。图上限1024节点/含根128层/单文件2MiB/总原始16MiB，根admission例外与精确顺序以§20为准；不宣称OS资源隔离。
- Rust嵌入：旧eval_module*只保留无模块行为，带模块Program仍guard拒绝。构建ModuleGraph，在on_eval_stack内调用eval_graph_traced_on_thread_with_args(&graph,&args)，parse/build/eval/drop均同大栈；不能重读图文件或克隆所有Program。外层GraphExecutionError::InvalidGraph/Origins是宿主不变量错误（CLI stdout空/stderr宿主异常/exit2），内层TracedRun.result才是LFZ错误，不被attempt吞宿主错。UserFn手写literal按§19补home_module；StructDef按§21补default_context，旧struct_def/struct_template helper保持None，graph使用TemplateContext与struct_template_in_context。ModuleId与SourceId不可互换，无Loaded不能借其它文件画下划线。

完整人类示例见docs/guide/modules.md；本AI新增作者实跑见docs/reports/T16-C-skill-evidence.md。R304仍OPEN，无C性能成绩，不声明正式发布；不改写旧v2成绩与输入。
