# LFZ 开发指南（AI 向）

> 读者：**用 LFZ 写程序的 AI Agent**（以及需要审阅该指南的人类评委）。
> 配套机读版 skill：[`.opencode/skills/lfz-programming/SKILL.md`](../../../.opencode/skills/lfz-programming/SKILL.md)（内容与本文件一致；Agent 通过 skill 加载器读取它）。
> 事实源：`docs/spec/{syntax,semantics,interface-contract}.md`（冻结 v1）。本文件是**速查/转述**，不定义新语法；冲突以 spec 为准。
> 实测证据：`.opencode/skills/lfz-programming/VERIFICATION.md`。

---

## 0. 如何运行 LFZ

在仓库根目录（PowerShell，先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`）：

```bash
cargo run --quiet -- run <file.lfz>       # 运行一个 LFZ 程序
cargo run --quiet -- run --json <file>    # 机器可读 JSON 输出
cargo run --quiet -- test                 # 运行黑盒测试集
```

- **退出码**：`0` 成功；`1` 测试失败（`assert` / `fail`）；`2` CLI 参数错误 / 语法或运行时错误 / 用例 error。
- **本 CLI v1 只支持 `lfz run <file>` 与 `lfz test`**：没有 `-e`、没有 stdin 管道、没有 REPL（`run -` 会把 `-` 当文件名 → `IOError: 无法读取：-`）。**程序一律写成 `.lfz` 文件再 `run`。**
- `--json` 模式下 stdout 只写一行 JSON；被运行程序的 `print` 输出重定向到 **stderr**。

---

## 1. 铁律：`.lfz` 文件首行必须是 `#42`

任何 **`.lfz` 文件**的第一行**必须恰好是 3 个字符 `#42`**，其后**紧跟一个行终止符**（`\n` / `\r\n` / `\r`）。否则程序以 **`CosmosAnswerError: 你忘记了宇宙的答案`** 终止（退出码 `2`）。

- **只按扩展名触发**：扩展名（ASCII 大小写不敏感）为 `lfz` 才要求，`.lfz` / `.LFZ` / `.Lfz` 都算。
- **豁免**：非 `.lfz` 文件（如 `.txt`）不需要；REPL / stdin / `run -e` 在 spec 中被定义为豁免入口（本 CLI v1 未实现后三者）。
- **无任何变体**：`# 42`、`#42 `（尾随空格）、`#42\t`、`##42`、`#43`、`#42abc`、首行前的空行/空格/Tab 全部违反。
- **只有 3 字节 `#42` 而无换行符**、**空 `.lfz` 文件** → 均违反。
- **BOM 豁免**：文件开头可有 UTF-8 BOM，跳过之后再要求 `#42`。
- **`#` 只在首行前导位合法**：程序中间出现 `#` → `SyntaxError`；字符串 / 注释 / 格式说明符里的 `#` 是普通字符。

最小可运行程序：

```lfz
#42
print("Hello, LFZ!")
```

> 片段示例：若只给代码片段，均须注明「作为 `.lfz` 文件时首行须为 `#42`」。

---

## 2. 错误类清单（12 个具体类 + 基类 `LfzError`）

> 用户可见输出**只有类名 + 中文消息**，**不存在任何编号错误码**。

| 错误类 | 触发条件 | 中文消息示例 |
|---|---|---|
| `LfzError`（基类） | 不直接抛出 | — |
| `CosmosAnswerError` | `.lfz` 缺合法 `#42` 前导 | `你忘记了宇宙的答案` |
| `SyntaxError` | 词法/语法错（非法字符、未闭合字符串/块注释、未列举转义、整数越界、意外记号、单 `;`、同行两语句、赋值目标非法、`#`/`_` 位置非法、循环外 `break`、函数外 `return`、非 UTF-8） | `非法字符 '$'` / `单独的 ';' 非法；打印变量请用 ';;'` / `语句之间必须有换行` |
| `NameError` | 引用未定义名字 | `未定义的名字 'x'` |
| `TypeError` | 类型不符；调用非函数；参数个数不符；重绑定 `let` | `条件必须是 bool，得到 int` / `不能重新赋值 let 变量 'a'；let 只锁重绑定，不锁内容` |
| `IndexError` | 下标越界 | `下标 5 越界（长度 2）` |
| `FieldError` | 缺字段（含 `del` 的键非数据字段） | `结构体没有字段 'missing'` |
| `ZeroDivisionError` | `/`、`%`、`div(a,b)` 除零（含 float） | `除以零` / `对零取模` |
| `OverflowError` | i64 溢出；`int(±Inf)` | `整数溢出：结果超出 i64 范围` |
| `ValueError` | 转换失败；非法格式说明符；空数组取极值；`randInt(lo>=hi)` | `无法把 string 转换为 int（'abc'）` / `空数组没有极值（min）` / `区间非法：3 >= 2` |
| `IOError` | `input()` EOF；不可读文件 | `输入结束（EOF）` / `无法读取：{path}` |
| `AssertionError` | **仅** `assert` 失败 / `fail()` | `断言失败：{msg}` |
| `RecursionError` | 递归/深结构超 10000 层 | `递归深度超限（超过 10000 层）` |

**输出形态**：加载/解析期错误（`CosmosAnswerError` / `SyntaxError`）无 `Traceback` 头；运行期错误（其余 10 类）有 `Traceback (most recent call last):` 头。运行期示例：

```
Traceback (most recent call last):
  File "demo.lfz", line 2, in <module>
    let r = 1 / 0
            ^
ZeroDivisionError: 除以零
```

---

## 3. 语法速查

> 以下片段作为 `.lfz` 文件时，首行须为 `#42`。

- **绑定**：`let x = 1`（不可重绑定，内容可改）｜`var y = 2`（可重绑定）。
- **类型**：`int` / `float` / `string` / `bool` / `nil` / `array` / `struct` / `function`。
- **字面量**：`1_000`、`0xFF`、`0b1010`、`0o755`、`3.14`、`1e10`、`"..."`、`true` / `false` / `nil`、`[1, 2]`、`{ k: 1 }`。
- **运算符优先级（高→低，全左结合）**：`() [] .` > 一元 `- !` > `* / %` > `+ -` > `|>` > `< <= > >=` > `== !=` > `&&` > `||`。
- **插值**：`"${expr}"`；格式说明符 `${x:>6}`、`${f:.2f}`、`${n:05d}`、`${n:x}`。转义仅 `\n \t \r \\ \" \e \$`（`\e`=ESC）；未列出的转义 → `SyntaxError`；字符串内 `{` `}` 是普通字符。
- **if（语句/表达式）**：`if cond { ... } else { ... }`；`let v = if cond { a } else { b }`。
- **while / for**：`while cond { ... }`；`for x in xs { ... }`；`for k in s { ... }`（键按字节序升序）。
- **函数 / lambda**：`fn add(a, b) { a + b }`；`fn add(a, b) => a + b`；`let dbl = (x) => x * 2`；`fn(x) { x + 1 }`。
- **结构体**：
  ```lfz
  struct Point {
      x: 0,
      y: 0,
      fn norm2() => self.x * self.x + self.y * self.y,
  }
  let p = Point { x: 3, y: 4 }
  ```
- **管道（data-last）**：`xs |> f(a)` = `f(a, xs)`；`_` 占位注入对应位：`10 |> push(_, [1, 2])` = `push(10, [1, 2])`。
- **`;;`**：独立一行，把当前可见变量按 `name ： value` 写 stdout。
- **注释**：`//` 行注释；`/* ... */` 块注释（不可嵌套，未闭合 → `SyntaxError`）。
- **赋值**：`=` `+=` `-=` `*=` `/=` `%=`；lvalue 可为 `x`、`a[i]`、`s.k`。

---

## 4. 常见陷阱（AI 最易踩；均已实测）

1. **`;` 非法** —— 一条语句一行，不要写 `;`；**打印变量用 `;;`**。单独 `;` → `SyntaxError`。
2. **`/` 恒为 float**（`7 / 2 == 3.5`）；整除用 **`div(7, 2) == 3`**。
3. **`%` 符号随除数**：`-7 % 3 == 2`、`7 % -3 == -2`。
4. **`let` 不可重绑定，`var` 可以**；但容器内容可原地改：`let a = [1]; a[0] = 2` 合法。
5. **`if` 是表达式**，可赋值 `let g = if c { "a" } else { "b" }`。
6. **换行终结语句；续行只能用括号**（`()`/`[]`/`{}`）；行尾运算符、行首 `.`/`|>` 续行 → `SyntaxError`。
7. **`=> {` 恒为块**；箭头位返回 struct 要加括号：`fn f() => ({ k: 1 })`。
8. **`${}` 内是 CODE 模式**：写普通 `"..."`，不要写 `\"`（`\` 非法）；插值内禁止换行。
9. **管道 data-last**；`_` 只能在管道右侧调用实参中，且不得穿 lambda。
10. **容器内置返回新值、不改原容器**（`ys = push(9, xs)`；`sort(xs)` 不改 `xs`）；只有 `a[i] = v` / `s.k = v` 原地改。
11. **54 个内置按 data-last 约定**（见 §6）。
12. **`round` 银行家舍入**：`round(2.5) == 2`、`round(3.5) == 4`。
13. **`int(float)`**：向零截断（`int(2.9) == 2`）；`int(float("nan"))` → `ValueError`；`int(float("inf"))` / `int(1e30)` → `OverflowError`。
14. **`float` 特值**：`sqrt(-1)` → `nan`；`float("inf")` / `float("nan")` 合法；整值浮点带 `.0`（`str(1.0) == "1.0"`）。
15. **关键字不能作字段名**（`self` `let` `fn` …）：用字符串键 `r["self"]`、`{ "self": 1 }`。
16. **无 truthiness、无隐式转换**：条件/逻辑/`filter` 谓词必须 `bool`；唯一隐式转换是 `int → float` 加宽。
17. **`==` 深结构相等且环安全**；`function` 仅同一性；自引用显示 `<cycle>`。
18. **空数组取极值报错**：`min([])` → `ValueError: 空数组没有极值（min）`。
19. **字符串只有双引号**；不存在 `->` `++` `..` `|` `&` `::` `>>` 等记号。
20. **`insert` 不支持负索引**（`removeAt`/`swap` 支持）；`pop([])` → `IndexError: 下标 -1 越界（长度 0）`。
21. **`del(k, s)` 只作用于数据字段**：`del(k,s)` 成功 ⟺ `has(k,s) == true`。

---

## 5. `check` 非致命 / `assert` 致命

| 函数 | 失败时行为 |
|---|---|
| `assert(cond, msg?)` | 抛 `AssertionError`（致命），退出码 `2` |
| `check(cond, msg?)` | stderr 一行 `check 失败：{msg}` + 返回 `false` + **继续运行**，退出码不变 |
| `fail(msg?)` | 抛 `AssertionError`（致命） |

实测：

```lfz
#42
check(1 == 2, "soft")
print("continue")
```

→ stdout `continue`；stderr `check 失败：soft`；退出码 `0`。

---

## 6. 内置函数 54 个（data-last）

> 凡对容器 / 字符串操作者，被操作对象恒为**最后一个参数**（data-last）；`xs |> f(...)` 注入末参。容器更新一律返回新值、不改原容器。

- **核心 / 数组（21）**：`len` `range` `push` `pop` `removeAt` `insert` `swap` `slice` `min` `max` `sum` `minBy` `maxBy` `sort` `sortBy` `map` `filter` `reduce` `take` `drop` `each`
- **struct（5）**：`keys` `values` `entries` `has` `del`（键按 UTF-8 字节序升序；只含数据字段）
- **字符串（8）**：`split` `join` `trim` `upper` `lower` `replace` `repeat` `startsWith`
- **数值 / 转换 / IO / 断言（20）**：`abs` `floor` `ceil` `round` `sqrt` `pow` `div` `rand` `randInt` `seed` `str` `int` `float` `type` `print` `eprint` `input` `assert` `check` `fail`

要点：`div` 整数向下取整；`round` 银行家舍入；`abs` 同型不加宽；`sqrt`/`pow` 外无数学函数；`type(x)` ∈ `int/float/string/bool/nil/array/struct/function`；`randInt(lo,hi)` 的 `[lo,hi)` 且 `lo>=hi` → `ValueError`；`input()` EOF → `IOError`。

---

## 7. 可运行示例

> 源文件：`docs/guide/ai/examples/`。均实测通过（退出码 `0`）。

### L1 — Hello

```lfz
#42
print("Hello, LFZ!")
```
输出：`Hello, LFZ!`

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

### 最小任务基线（参考解）

```lfz
#42
fn sum_even(xs) {
    xs |> filter((x) => x % 2 == 0) |> reduce((acc, x) => acc + x, 0)
}
print(sum_even([1, 2, 3, 4, 5]))
```
输出：`6`

---

## 8. 自检清单（交付前逐项打勾）

- [ ] 每个 `.lfz` 文件首行恰好 `#42` + 换行（无变体、无前导空白）？
- [ ] 没有用 `;` 作分隔或打印变量（打印变量用 `;;`）？
- [ ] 没有把 `/` 当整除（整除用 `div`）？
- [ ] 没有对 `let` 重绑定（改用 `var` 或只改容器内容）？
- [ ] 条件/逻辑/`filter` 谓词都是 `bool`？
- [ ] 跨行都在括号内？
- [ ] `${}` 内没有误写 `\"`？
- [ ] 管道 data-last / `_` 位置合法？
- [ ] 没有依赖容器内置原地修改（已接收返回值）？
- [ ] 字段名没有用关键字？
- [ ] 字符串没有用单引号？
- [ ] 没有出现/期待旧的编号错误码？
- [ ] 代码块都含 `#42` 首行（片段已注明）？

---

## 9. 权威来源与相关文件

- 语法：`docs/spec/syntax.md`｜语义：`docs/spec/semantics.md`｜接口/错误/内置：`docs/spec/interface-contract.md`（冻结 v1）。
- CLI/runner 契约：`docs/tooling/runner-contract.md`。
- 机读版 skill：`.opencode/skills/lfz-programming/SKILL.md`；提示模板：`.opencode/skills/lfz-programming/prompt-template.md`；实测记录：`.opencode/skills/lfz-programming/VERIFICATION.md`。
- 可运行示例：`docs/guide/ai/examples/`。
