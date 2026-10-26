# LFZ 精简语言参考

> 每一条都标注了 **spec 章节号**（`SY` = [`docs/spec/syntax.md`](../spec/syntax.md)，`SE` = [`docs/spec/semantics.md`](../spec/semantics.md)，`IC` = [`docs/spec/interface-contract.md`](../spec/interface-contract.md)）。本参考是**导读**，权威定义以 spec 为准。
> 配套：上手 [`README.md`](README.md) ｜ 教程 [`tutorial.md`](tutorial.md) ｜ 错误 [`errors.md`](errors.md) ｜ 测试 [`testing.md`](testing.md)
> 最后更新: 2026-10-25（T12：1.4.0 参数、代码入口与 JSON） by docs-writer

---

## 1. 词法要点

| 主题 | 规则 | 出处 |
|---|---|---|
| 源文件 | 后缀 `.lfz`；编码 **UTF-8** | SY §2.1 |
| BOM | 文件**开头**的 UTF-8 BOM 静默跳过（仅一个） | SY §2.1 |
| 行终止符 | 接受 `\n` / `\r\n` / `\r`，加载器统一归一化为 `\n` | SY §2.1 |
| **`#42` 前导** | **仅 `.lfz` 文件**要求：第 1 行**恰好** `#42` + 行终止符；无变体；缺失/违规 → `CosmosAnswerError`。非 `.lfz` 文件与 REPL/stdin/`-e` **豁免** | SY §2.2、§6-B1~B12 |
| 缩进 | 纯视觉，不决定块结构（块由 `{}` 决定） | SY §2.1 |
| 行注释 | `// ...` 到行尾 | SY §2.4 |
| 块注释 | `/* ... */`，**不可嵌套**，等价一个空格；未闭合 → `SyntaxError` | SY §2.4 |
| 标识符 | `(letter \| "_") {letter \| digit \| "_"}`，**仅 ASCII**；单独 `_` 是占位符 token | SY §2.6 |
| 关键字 | `let var fn return if else while for in break continue struct true false nil self` | SY §2.6 |
| 保留（未启用） | `match class import from try catch rescue yield async await and or not is` | SY §2.6 |
| 整数字面量 | 十进制 / `0x..` / `0b..` / `0o..`；`1_000` 合法；越界 → `SyntaxError` | SY §2.7 |
| 浮点字面量 | 必须含小数点且点后有数字，或指数式（`1e10`）；`.5` 非法 | SY §2.7 |
| 字符串 | 双引号；转义 `\n \t \r \\ \" \e \$`；未列举转义 → `SyntaxError`；`{`/`}` 是普通字符 | SY §2.8 |
| 插值 | `"${表达式}"`（见下）；`\${` 输出字面量 `${`；`$` 后非 `{` 即普通 `$` | SY §2.8、§5-A14/A15 |
| 注释 / 字符串内的 `#` | 合法（普通字符）；CODE 模式其它位置的 `#` → `SyntaxError` | SY §2.2、§6-B6 |

---

## 2. 语句（每条语句独占一个逻辑行，换行分隔）

| 语句 | 形式 | 出处 |
|---|---|---|
| 绑定 | `let 名 = 表达式` / `var 名 = 表达式` | SY §7 |
| 赋值 | `lvalue = 表达式`；复合：`+= -= *= /= %=` | SY §7 |
| lvalue | `IDENT` 后接若干 `.字段` / `[下标]` | SY §7 |
| 函数声明 | `fn 名(参数表) { ... }` 或 `fn 名(参数表) => 表达式` | SY §7 |
| 结构体声明 | `struct 名 { 成员表 }`；成员 = `字段: 表达式` 或 `fn ...`；逗号必填、尾逗号可选 | SY §7、§5-A7 |
| `if` | `if 条件 { ... } else { ... }`（`if` 是**表达式**） | SY §7 |
| `while` | `while 条件 { ... }` | SY §7 |
| `for` | `for 变量 in 表达式 { ... }`（只能遍历 `array` / `struct`） | SY §7、SE §4.5.4 |
| 跳转 | `return [表达式]` / `break` / `continue`；`return` 行尾 = 返回 `nil` | SY §7、§5-A19 |
| **变量 dump** | 单独一行 `;;`（独占逻辑行，输出通道 stdout） | SE §3.6、SY §5-A10 |
| 表达式语句 | 任意表达式（含语句首 `{ ... }` = 匿名 struct 字面量） | SY §7、§5-A9 |

> **换行规则**：一条语句占一条逻辑行；**唯一**的跨行手段是括号 `( )` `[ ]`（含字面量/声明体 `{ }`）。**不允许**行尾运算符续行（SY §3.1、§5-A2）。
> **单个 `;` 永远非法**（`SyntaxError`）——不要用分号分隔语句（SY §5-A11）。

---

## 3. 表达式

| 构造 | 形式 | 出处 |
|---|---|---|
| 字面量 | `123` / `3.14` / `"文本"` / `true` / `false` / `nil` | SY §7 |
| 数组 | `[e1, e2, ...]`（逗号必填，尾逗号可选） | SY §7、§5-A8 |
| 结构体字面量 | `[类型名] { 字段: 值, ... }`（无类型名即匿名） | SY §7 |
| 调用 / 索引 / 字段 | `f(...)` / `a[i]` / `s.k`（同源：`s.k ≡ s["k"]`）；**下标 `[]` 只适用于 `array` / `struct`** | SY §7、SE §4.5.9 |
| 一元 | `-`（取负）/ `!`（逻辑非） | SY §4.1 |
| 二元 | 见优先级表 | SY §4.1 |
| `if` 表达式 | `if c { a } else { b }` 可作为值 | SY §7 |
| 函数字面量 | `fn(参数) { ... }` / `fn(参数) => 表达式` / `(参数) => 表达式` | SY §7 |
| 管道 | `L \|> R`（见 §4） | SY §4.3 |
| 字符串插值 | `"前缀 ${表达式[:格式说明符]} 后缀"` | SY §2.8 |

> ⚠️ **字符串不能下标**：`s[0]` 不是合法用法，会报 `TypeError: 运算符 '[]' 不支持 string 与 array / struct`。要按字符访问，先 `split("", s)` 拆成字符数组（见 §3.5）。
> **零参 `print()` 合法**：`print()` 打印一个空行；`print`/`eprint` 接受任意个（含 0 个）实参。

### 3.1 运算符优先级（高 → 低，均左结合）

| 级别 | 运算符 | 说明 | 出处 |
|---|---|---|---|
| 1（最紧） | `()` `[]` `.` | 调用 / 索引 / 字段（postfix） | SY §4.1 |
| 2 | `-` `!`（前缀） | 取负 / 逻辑非 | SY §4.1 |
| 3 | `*` `/` `%` | 乘 / 除 / 取模 | SY §4.1 |
| 4 | `+` `-` | 加 / 减 / 字符串连接 | SY §4.1 |
| 5 | `\|>` | 管道 | SY §4.1 |
| 6 | `<` `<=` `>` `>=` | 比较 | SY §4.1 |
| 7 | `==` `!=` | 相等 | SY §4.1 |
| 8 | `&&` | 短路与 | SY §4.1 |
| 9（最松） | `\|\|` | 短路或 | SY §4.1 |

> 直观规则：**`|>` 比 `+ - * / %` 松，比任何比较/逻辑运算符紧**（Elixir 位置，SY §4.1）。

### 3.2 类型化运算符（无隐式转换，唯一例外 `int → float`）

| 运算符 | 规则 | 出处 |
|---|---|---|
| `+` | `int+int`、`float+float`、`string+string`（连接） | SE §4.2 |
| `-` `*` | 数值；`*` 额外支持 `string * int`（重复，`"█" * 5`） | SE §4.2 |
| `/` | **恒返回 `float`**（`7 / 2 == 3.5`）；除零 → `ZeroDivisionError` | SE §4.2 |
| `div(a,b)` | 内置，整数**向下取整**，两参须 `int`（`div(7,2)==3`） | SE §4.2 |
| `%` | **Python 取模**，结果符号随除数（`-7 % 3 == 2`） | SE §4.2 |
| `==` `!=` | 标量按值；`array`/`struct` **深相等**（环安全）；`function` 按同一性 | SE §4.2、§4.5.9 |
| `&&` `\|\|` | 两侧与结果都必须 `bool`，短路 | SE §4.2 |
| `int`↔`float` | 算术时 `int` 加宽为 `float`（`\|int\| > 2^53` 可能不精确）；比较按**数学精确值** | SE §4.5.7 |

### 3.3 字符串格式说明符

```
format_spec = [ [fill] align ] [sign] [width] [ "." precision ] [type]
align = "<" | ">" | "^"      type = "d" | "x" | "X" | "o" | "b" | "f" | "e" | "s"
```
出处：SY §2.8。示例：`${n:05d}` → `00042`，`${n:x}` → `2a`，`${pi:.2f}` → `3.14`，`${n:>6}` → 右对齐宽度 6。非法说明符 → `ValueError`；类型不符 → `TypeError`（SE §8.1）。

**三种对齐可用**，且可带 fill 字符（`fill` 在 `align` 之前）：

| 写法 | 含义 | 实测示例（`s = "ab"`） |
|---|---|---|
| `${s:<5}` | 左对齐，宽度 5 | `[ab   ]` |
| `${s:>5}` | 右对齐，宽度 5 | `[   ab]` |
| `${s:^5}` | 居中，宽度 5 | `[ ab  ]` |
| `${s:*^7}` | fill `*` + 居中 | `[**ab***]` |
| `${s:*<7}` | fill `*` + 左对齐 | `[ab*****]` |
| `${42:05d}` | 补零（`0` 作 fill） | `[00042]` |

> ⚠️ **宽度必须是字面数字，不支持动态宽度**：`"${s:>w}"`（`w` 是变量）→ `ValueError: 格式说明符非法：'>w'`。需要"按变量宽度对齐"时，用`padEnd` / `padStart` 补齐（见 §3.5）。

### 3.4 引用语义与作用域（易错，务必记住）

| 规则 | 内容 | 出处 |
|---|---|---|
| 引用语义 | `array` / `struct` 赋值、传参**共享同一容器**；`a[i]=v` / `s.k=v` **原地修改** | SE §4.5.2 |
| 函数式更新 | 除上述赋值语法外，**所有容器内置返回新值、不改原容器**（`push`/`sort`/`map`...） | SE §4.5.2、IC §10.7 |
| `let` 边界 | `let` 只锁**重绑定**（`a = v` 非法）；容器内容修改合法（`a[0]=1` 合法） | SE §4.5.2 |
| 闭包 | **按 cell 引用捕获**外层变量，与定义处共享单元 | SE §4.5.3 |
| 条件 | 必须 `bool`（无 truthiness）；`if`/`while`/`&&`/`\|\|`/`filter` 谓词等 | SE §4.5.0 |
| **循环每轮新绑定** | 循环体里的 `let`/`var` **每执行一轮就产生一个新绑定**（各自新 cell），可放心在循环内声明；闭包捕获的是**当轮**那个 cell | SE §4.5.0、§4.5.3 |
| 值显示 | `print`/`str`/`;;` 同一显示形式；字符串**顶层原样、嵌套加引号**；环 → `<cycle>` | SE §3.7 |
| **float 显示** | **最短往返优先**：先取能唯一还原该 `f64` 的最短数字串；**仅定点形式**的整值补 `.0`（`1.0`、`100.0`）；`\|x\| ≥ 1e16` 或 `0 < \|x\| < 1e-4` 改用**指数**形式且**不补** `.0`（`1e16`、`1e-5`） | SE §3.7 |

> 实测：`1.0`→`1.0`、`100.0`→`100.0`、`1.5`→`1.5`、`1e16`→`1e16`、`1e-5`→`1e-5`。两条旧读法（"最短往返""整值补 `.0`"）**不再冲突**：`.0` 只管定点形式，越出定点阈值时指数形式就是最短往返形式（SE §3.7 v1.1 补钉）。

### 3.5 字符串处理惯用法（按字符访问 / 长度 / O(n) 构建）

LFZ 的 `string` 是**不可变**的 UTF-8 字符串，**不能下标**。下面三条是日常写法（全部实测）。

**① 按字符访问 → `split("", s)`（O(n) 一次）**

`split("", s)` 把字符串按字符切成 `array[string]`，之后用**数组下标**（O(1)）即可：

```lfz
#42
let s = "abc"
let cs = split("", s)          // ["a", "b", "c"]，一次 O(n)
print(cs[0])                   // a
for c in cs { print(c) }       // 顺序遍历，总 O(n)
```

> **为什么这样而不是 `s[i]`**：LFZ 的 string 是不可变 UTF-8，按标量取第 `i` 个字符只能 `chars().nth(i)` = **O(i)**；若支持 `s[i]`，一个 `for i in range(len(s))` 的遍历就会退化 **O(n²)**。所以语言**不提供** `s[i]`（`s[0]` 直接报 `TypeError`），统一用 `split("", s)`（**一次 O(n)**，之后 O(1)/次）。

**② `len(s)` 合法，按 Unicode 标量计数**

`len` 对 string 求的是 **Unicode 标量个数**（不是字节数）：`len("abc")==3`、`len("你好")==2`、`len("a你b")==3`。注意 `len(s)` 本身是 **O(n)**；别在循环里反复对同一长串求 `len`——先 `split("", s)` 得到数组后，对数组 `len` 是 **O(1)**。

**③ 逐字符构建字符串 → `map` 一次构造数组，再 `join`**

循环内 `s = s + c` 每轮复制前缀，是 O(n²)。循环 `parts = push(c, parts)` 也会复制数组前缀，同样是 O(n²)。用 map 一次构造数组，或预分配数组后下标写入，再一次 join：

```lfz
#42
let cs = split("", "abc")
let parts = map((c) => upper(c), cs)
print(join("", parts))             // ABC
let out = map((c) => "", cs)
for i in range(len(cs)) {
    out[i] = upper(cs[i])
}
print(join("", out))               // ABC
```

若每个片段处理成本为 O(1)，以上构建为 O(片段数 + 总输出大小)。push 返回新数组，不适合长循环累积。

**按变量宽度补齐**：已实现 padEnd / padStart，宽度按 Unicode 标量计数。

```lfz
#42
let width = 5
print("[" + padEnd(width, " ", "ab") + "]")  // [ab   ]
print("[" + padStart(width, " ", "ab") + "]") // [   ab]
```

### 3.6 容易"以为不支持、其实支持"的语法（正面示例）

下面这些构造 **v1 全部可用**，但常被漏写，导致读者主动绕开：

| 构造 | 写法 | 实测结果 |
|---|---|---|
| **链式下标赋值** | `t[1][0] = 7` | `[[0, 0], [7, 0]]`（逐层定位后原地写） |
| **`else if` 链** | `if a { ... } else if b { ... } else { ... }` | 命中分支正常执行 |
| **一条串多个 `${}`** | `"${a}+${b}=${a + b}，再来一个 ${b}"` | `1+2=3，再来一个 2` |
| **零参 `print()`** | `print()` | 输出一个空行 |
| **`&&` / `\|\|` 短路** | `len(xs) > 0 && xs[0] == 1` | 左侧为假即不评估右侧（不越界）；`\|\|` 同理 |

```lfz
#42
let t = [[0, 0], [0, 0]]
t[1][0] = 7                       // 链式下标赋值
let xs = []
print(len(xs) > 0 && xs[0] == 1)  // false，短路不越界
print(len(xs) == 0 || xs[0] == 1) // true
print()                           // 空行
```

> `&&` / `||` 两侧与结果都必须是 `bool` 且**短路**（SE §4.2）。

---

## 4. 管道 `|>`（特色）

- 语法：`L |> R`，其中 `R` 是调用、函数名、方法或 lambda；解析期脱糖（运行时零开销，SY §4.3）。
- **data-last**：`L |> F(a1,...,an)` 把 `L` **追加为末参**；**无 `_`** 时如此。
- **`_` 占位**：右侧调用中**恰有一个** `_` → `L` 替换该位；**≥2 个 `_`** → `SyntaxError`。
- `L |> R`（`R` 非调用）→ `R(L)`；`R` 不是函数 → `TypeError: 管道右侧必须是函数，得到 ...`。
- `_` **只绑定词法上直接包含它的最近管道的 RHS**，且**不得出现在 lambda 体内**（SY §4.3 规则 6、§5-A24）。
- 左结合：`L |> R1 |> R2` = `R2(R1(L))`。

> 示例：`xs |> sum()` ≡ `sum(xs)`；`xs |> map((x) => x * 2)` ≡ `map((x) => x * 2, xs)`；`8 |> div(_, 2)` ≡ `div(8, 2)`。

---

## 5. 内置函数速查（72 个，全部 **data-last**）

> 出处：IC §10.7。约定：**被操作数据恒为最后一个参数**；`xs |> f(...)` 把 `xs` 注入末参。**除赋值语法外一律"返回新值、不改原容器"**（A1，SE §4.5.2）。参数类型/数量不符 → `TypeError`。

### 5.1 核心 / 数组（22）

| 内置 | 签名 | 返回 | 说明 |
|---|---|---|---|
| `len` | `len(x) -> int` | `int` | array 元素数 / struct **数据字段**数 / **string 的 Unicode 标量数**（`len("abc")==3`、`len("你好")==2`）；对 string 是 **O(n)** |
| `range` | `range(n) -> array[int]` / `range(lo, hi) -> array[int]` | `array[int]` | `[0,n)` / `[lo,hi)`；n<=0 / hi<=lo → 空；须 int，只有 1/2 参，无 step。物化数组，时间/内存 O(结果长度)，超容量 → OverflowError |
| `push` | `push(v, xs) -> array` | `array` | 追加（新数组） |
| `pop` | `pop(xs) -> array` | `array` | 去掉末元素；空 → `IndexError` |
| `removeAt` | `removeAt(i, xs) -> array` | `array` | 去下标 `i`（支持负索引）；越界 → `IndexError` |
| `insert` | `insert(i, v, xs) -> array` | `array` | 在 `i` 插入；`i ∈ [0,len]`（不支持负索引） |
| `swap` | `swap(i, j, xs) -> array` | `array` | 交换（支持负索引） |
| `slice` | `slice(from, to, xs) -> array` | `array` | `xs[from..to]`；越界夹取 |
| `min` `max` | `min(xs)` / `max(xs)` | 元素 | 全序；空 → `ValueError` |
| `sum` | `sum(xs) -> number` | `int`/`float` | 全 int → int（溢出 → `OverflowError`）；空 → `0` |
| `minBy` `maxBy` | `minBy(keyFn, xs)` / `maxBy(keyFn, xs)` | 元素 | 以 `keyFn` 结果为准；空 → `ValueError` |
| `sort` | `sort(xs) -> array` | `array` | 升序（稳定）；元素须同型可比 |
| `sortBy` | `sortBy(keyFn, xs) -> array` | `array` | 按 key 升序（稳定） |
| `map` | `map(f, xs) -> array` | `array` | 逐元素 `f` |
| `filter` | `filter(pred, xs) -> array` | `array` | `pred` 须返回 `bool` |
| `reduce` | `reduce(f, init, xs) -> value` | 任意 | `f(acc, x)` 左折叠 |
| `take` `drop` | `take(n, xs)` / `drop(n, xs)` | `array` | 前 `n` 个 / 去前 `n` 个 |
| `each` | `each(f, xs) -> nil` | `nil` | 仅副作用遍历 |
| `contains` | `contains(v, xs) -> bool` | `bool` | 仅 array；用深相等、环安全的 == 查找。O(n) 线性扫描；容器元素比较另计成本。避免循环内反复扫描长数组；struct 键用 has |

### 5.2 struct / 字典（5）

| 内置 | 签名 | 返回 | 说明 |
|---|---|---|---|
| `keys` | `keys(s) -> array[string]` | `array` | **数据字段**键，**字节序升序** |
| `values` | `values(s) -> array` | `array` | 与 `keys` 同序 |
| `entries` | `entries(s) -> array` | `array[[k,v]]` | 元素 `[key, value]`，按 `keys` 序 |
| `has` | `has(k, s) -> bool` | `bool` | 仅数据字段；方法 → `false` |
| `del` | `del(k, s) -> struct` | `struct` | 去 `k`（新 struct）；缺失/方法 → `FieldError` |

> **数据面**：方法（函数值字段）**不参与** `keys`/`values`/`entries`/`has`/`len`/`del`/`==`/display（A5，SE §3.7、§4.5.9）。

### 5.3 字符串（13）

| 内置 | 签名 | 说明 |
|---|---|---|
| `split` | `split(sep, s) -> array[string]` | `sep` 空串 → 按字符切分 |
| `join` | `join(sep, xs) -> string` | 元素须为 string |
| `trim` | `trim(s) -> string` | 去首尾空白 |
| `upper` `lower` | `upper(s)` / `lower(s)` | ASCII 大小写 |
| `replace` | `replace(old, new, s) -> string` | 全部替换 |
| `repeat` | `repeat(n, s) -> string` | `n<=0` → 空串 |
| `startsWith` | `startsWith(prefix, s) -> bool` | 前缀判断 |
| `endsWith` | `endsWith(suffix, s) -> bool` | 空 suffix → true；O(m)，m 为后缀长度 |
| `indexOf` | `indexOf(sub, s) -> int` | 首次匹配的 Unicode 标量下标；无匹配 → -1；空 sub → 0。最坏 O(n·m) |
| `substring` | `substring(from, to, s) -> string` | 标量半开区间；两端夹取至 [0,len(s)]，负数夹至 0；from>=to → 空串。非空结果 O(min(to,len(s)))，须扫描 UTF-8 前缀；空区间可直接返回 |
| `padEnd` | `padEnd(width, fill, s) -> string` | 右侧重复 fill，按标量截断至 width；width<=len(s) 返回原串；fill 空 → ValueError |
| `padStart` | `padStart(width, fill, s) -> string` | 左侧补齐，其余同 padEnd |

> 新增字符串内置全 data-last；width/from/to 须 int，其余参数须 string。pad* 复杂度 O(n+m+输出宽度)，包含 s、fill 的标量计数。Unicode 标量不是字节或用户感知字形（组合字符、表情序列可能含多个标量）。

### 5.4 数学 / 随机 / 转换 / IO / 断言（29）

| 内置 | 签名 | 说明 |
|---|---|---|
| `abs` | `abs(x) -> number` | **同型、绝不加宽**；`i64::MIN` → `OverflowError` |
| `floor` `ceil` | `floor(f)` / `ceil(f) -> int` | 向下 / 向上取整（int 实参先加宽） |
| `round` | `round(f) -> int` | 四舍六入五成双（banker's rounding） |
| `sqrt` | `sqrt(f) -> float` | `f<0` → `NaN` |
| `pow` | `pow(a, b) -> float` | 溢出 → `±Inf` |
| `div` | `div(a, b) -> int` | 两参须 int；向下取整；`b==0` → `ZeroDivisionError` |
| `rand` | `rand() -> float` | `[0.0, 1.0)` |
| `randInt` | `randInt(lo, hi) -> int` | `[lo, hi)`；`lo>=hi` → `ValueError` |
| `seed` | `seed(n) -> nil` | 固定随机序列 |
| `str` | `str(x) -> string` | 显示形式（SE §3.7） |
| `int` | `int(x) -> int` | 向零截断；`NaN` → `ValueError`；`±Inf`/超 i64 → `OverflowError` |
| `float` | `float(x) -> float` | 支持 `"inf"`/`"-inf"`/`"nan"` |
| `type` | `type(x) -> string` | `"int"/"float"/"string"/"bool"/"nil"/"array"/"struct"/"function"` |
| `print` `eprint` | `print(...)` / `eprint(...) -> nil` | 显示形式拼接、空格连接、末尾换行（stdout / stderr） |
| `input` | `input(prompt?) -> string` | 先打印 prompt；EOF → `IOError`；去行尾换行 |
| `assert` | `assert(cond, msg?) -> nil` | 失败 → `AssertionError`（致命） |
| `check` | `check(cond, msg?) -> bool` | 失败 → stderr 警告 + 返回 `false`，**不中断** |
| `fail` | `fail(msg?) -> never` | 抛 `AssertionError`（致命） |
| `sin` / `cos` | `sin(f) -> float` / `cos(f) -> float` | 弧度；O(1)；±Inf/NaN → NaN |
| `log` | `log(f) -> float` | 自然对数；O(1)；负数 → NaN，0 → -Inf（不抛除零错误） |
| `exp` | `exp(f) -> float` | e 的 f 次幂；O(1)；溢出 → +Inf |
| `ord` | `ord(c) -> int` | c 须恰一个 Unicode 标量的 string；成功 O(1)；非法串 ValueError 需报告完整标量数，O(n)；非 string → TypeError，O(1) |
| `chr` | `chr(n) -> string` | n 须 int，合法范围 0..0x10FFFF，排除代理区 0xD800..0xDFFF；非法 → ValueError；O(1) |
| `readFile` | `readFile(path) -> string` | 整文件 UTF-8 读取；非法 UTF-8 / 读取失败 → IOError |
| `writeFile` | `writeFile(path, contents) -> nil` | UTF-8 覆盖写，创建/截断文件；失败 → IOError |
| `appendFile` | `appendFile(path, contents) -> nil` | UTF-8 追加；不存在则创建；失败 → IOError |

> 文件 IO 的 path / contents 须 string，不隐式转换；写数字先 str(42)。相对路径基于解释器进程当前工作目录，与脚本所在目录无关。路径原样交给 OS，不展开 ~，不自动创建父目录，不做规范化或沙箱；二进制/目录 API 不提供。读返回 string，写/追加返回 nil；时间及内存按文件大小计；结果依赖文件系统与 cwd。多片段拼接用 map 或预分配数组后 join，循环 push 也会复制前缀。
> sin/cos/log/exp 接受 int 并加宽为 float；大于 2^53 的整数加宽可能失精度。NaN/Inf 遵 IEEE-754；`/ 0` 和 `% 0` 仍抛 ZeroDivisionError。
> `seed` 未调用时用时间种子；文件 IO 还依赖文件系统与 cwd。需要可复现时，固定种子、输入内容及显式参数（IC §10.7、§16）。
> 数值内置加宽口径：`floor`/`ceil`/`round`/`sqrt`/`pow`/`sin`/`cos`/`log`/`exp` 接受 `int` 并加宽为 `float`；`abs` 同型不加宽（IC §10.7「数值内置形参加宽」）。

### 5.5 参数与 JSON（3；1.4.0，IC §16）

| 内置 | 签名 | 说明 |
| -- | -- | -- |
| `argv` | `argv() -> array[string]` | 本次执行 `--` 后参数，不含解释器/脚本名；每次返回新数组，无参数为 `[]` |
| `jsonParse` | `jsonParse(text) -> value` | 严格单个 JSON 文本；null/bool/string/array/object → nil/bool/string/array/匿名 struct |
| `jsonStringify` | `jsonStringify(value) -> string` | 紧凑 JSON，无尾换行；struct 数据字段按 UTF-8 字节序排列 |

- JSON 整数词法必须在 i64 范围，超出即 `ValueError`，不退化成浮点。带 `.`/`e`/`E` 的词法为有限 f64；有限舍入和下溢遵 IEEE-754，保留浮点负零。序列化 float 保留类型，例如 `1.0` 和 `-0.0`，不等同于通用 `str` 的格式规则。
- 禁止 BOM、注释、尾逗号、单引号、裸键、前导 `+`/非法零、NaN/Infinity、裸控制字符；空白仅空格/TAB/LF/CR。正确解码 UTF-16 代理对，拒孤立代理。解码后重复键拒绝，如 `"a"` 和 `"\u0061"`。
- 解析内容里的 `$`、`${...}` 是普通数据。序列化不转义 `$`、`/` 和其它 Unicode；控制字符、双引号和反斜杠按 JSON 转义。
- 同时打开的 array/object 容器最多128层；标量为0层，空容器为1层，解析与编码同口径。循环引用拒绝；共享但无环子树可重复编码。
- struct 使用数据面，忽略函数值方法字段；根函数、数组中的函数和结构体模板拒绝，NaN/Inf拒绝；不执行方法、不修改输入、不返回部分结果。错误和精确偏移见 [errors.md §8](errors.md)。
- `argv` 是一等函数值，`type(argv)=="function"`、`str(argv)=="<fn argv>"`，同次执行身份稳定；词法变量仍优先，可遮蔽。只有该内置在本轮新增取值能力；其它内置作回调须用 lambda 包裹，例如 `map((s) => jsonParse(s), texts)`。
- 解析时间 O(n + Σk log k)，n为输入字节数、k为各对象键数，键比较成本另计；空间O(n)，递归128有界。编码时间按展开输出字节数与各对象键排序计，共享子树每次出现都计入输出。struct字段索引为平均查找优化，不能称最坏O(1)；不暴露哈希迭代顺序。

代码入口为 `lfz [run] -e CODE [-- ARG...]` 和 `lfz [run] - [-- ARG...]`，分别显示伪路径 `<command>` / `<stdin>`，源码行从1开始且无需文件 `#42` 前导。stdin读取UTF-8到EOF，随后 `input()` 读EOF报IOError。文件入口原前导规则不变。CLI `--json` 为状态协议，不自动返回最后表达式；`-e` 下一项及 `--` 之后均不解析选项。

---

## 6. 一页纸自检清单

- [ ] 文件第 1 行是 `#42`（且仅 `.lfz` 文件需要）。
- [ ] 用**换行**分隔语句，不用 `;`。
- [ ] `if`/`while` 条件一定是 `bool`。
- [ ] 不会重绑定的用 `let`，要改值用 `var`。
- [ ] `array`/`struct` 是引用；想"复制"需自己构造新容器。
- [ ] 容器内置返回**新值**：`xs = push(v, xs)`。
- [ ] 除法语义：`/` → float；整数除法用 `div`；`%` 是 Python 取模。
- [ ] 管道 data-last：`xs |> f(a)` = `f(a, xs)`。
- [ ] 跨行只靠括号；字符串/插值内**不能**换行。
- [ ] 字符串**不能下标** `s[0]`；按字符访问用 `split("", s)`（§3.5）。
- [ ] 循环体里的 `let`/`var` **每轮都是新绑定**，可放心在循环内声明。
- [ ] 逐字符**别**用 `s = s + c`（O(n²)）；改用 map 或预分配后下标写入，再 join（§3.5）；循环 push 同样是 O(n²)。
- [ ] 格式对齐用 `<`/`>`/`^`（可带 fill）；**动态宽度不支持**，用 `repeat` 手工补。
- [ ] `range` 接受 1 或 2 个 int 参数；无 step 参数。
- [ ] 对齐/退出码等细节：退出码 `0` 成功 / `1` **测试用例失败** / `2` **任何错误**；`assert` 在 `run` 下走错误类 → 退出 **2**（见 [`errors.md` §3.4](errors.md)、[`testing.md` §5](testing.md)）。

---

## 7. spec 章节索引

| 你想查 | 看 |
|---|---|
| 词法、`#42`、注释、字面量、字符串插值 | `syntax.md` §2、§6 |
| EBNF 文法 | `syntax.md` §7 |
| 语句 / 块 / 换行模式 | `syntax.md` §3 |
| 优先级、管道脱糖 | `syntax.md` §4 |
| 27 条歧义消解（A1–A27） | `syntax.md` §5 |
| `;;` 语义、值显示、求值语义、错误模型 | `semantics.md` §3.6、§3.7、§4.5、§8 |
| 72 个内置函数表 | `interface-contract.md` §10.7、§16 |
| 错误类实现映射、loader、`Span`、帧 | `interface-contract.md` §8.1、§10 |
| 下游硬性要求（含 docs / runner 契约） | `interface-contract.md` §11 |
