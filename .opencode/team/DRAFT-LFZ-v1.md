# LFZ 语言 v1 设计草案（DRAFT — 供用户评审）

> 作者: language-architect ｜ 日期: 2026-09-23 ｜ 状态: **草案，非冻结 spec**
> 依据: `task-info.md`、`.opencode/team/REQUIREMENTS.md`、`KICKOFF.md`、`BRAINSTORM.md`、`DECISIONS.md`（D-007 v1 范围、D-011 实现语言 = Rust）
> 本文件是**给用户拍板用**的评审稿：语法、5 个特色、范围、开放问题。评审通过后才会拆成 `docs/spec/{syntax,semantics,interface-contract}.md` 并冻结 v1。
> 注：本草案只定义**语言本身**，不绑定实现；但标注了「对 Rust 实现的直接影响」（D-011）。

---

## 0. 语言速览

```
print("Hello, LFZ!")          // 顶层语句按顺序执行
let name = "世界"
print("你好，${name}！")        // 富字符串插值
```

- 文件后缀 `.lfz`，UTF-8（无 BOM），LF / CRLF 均接受。
- 运行命令（P4 由 tooling-dev 定稿）：`lfz run <file.lfz>`。

---

## 1. 设计哲学 / 身份（LFZ 的"魂"）

LFZ（读作 "elf-zee"）是一门**动态类型、表达式导向的通用脚本语言**，其魂是一句话：

> **「万物皆值，值皆可流；一种结构，两种面孔。」**

**万物皆值**：`if`、块、函数、数组、结构体全都是值；块的值就是它最后一条表达式的值，函数体末尾表达式的值就是返回值（`return` 仅用于提前退出）。**值皆可流**：任何值都能顺着 `|>` 管道流向下一个函数，`_` 占位符可把它精确塞进任意参数位。**一种结构，两种面孔**：复合类型只有两种——有序的 `array` 与无序的 `struct`；而 `struct` 同时拥有「对象的 `.字段` / 方法」面孔与「字典的 `["键"]` 面孔」，两者读写的是同一块存储。

LFZ 拒绝一切"魔法"：没有 truthiness（条件必须是 `bool`）、没有隐式转换（唯一的例外是 int→float 加宽，且被完整写死）、越界 / 缺字段 / 整数溢出 / 类型不符一律**结构化报错**（带错误码、位置、修复建议、可选 JSON）。代价是比 Python 啰嗦一点，回报是**高度可预测、可定位、可被机器自验证**——这让 LFZ 特别适合被 AI Agent 生成代码，并用内建的 `assert` / `check` 自查。

**与现有语言的差异一句话**：它是「F# 的管道（尾参注入）+ Elixir 的 `_` 插位 + Python 的动态易用 + Rust 的"无隐式/溢出即错"纪律 + 一种自成一体的『字典=对象』统一 struct」。

---

## 2. 具体语法 + 完整样例程序

### 2.0 总览（语法要点）

| 主题 | 规则 |
|---|---|
| **变量声明** | `let x = e`（绑定不可重赋值）／ `var x = e`（可重赋值） |
| **赋值** | `x = e`、`s.k = e`、`a[i] = e`；复合赋值 `+= -= *= /= %=` |
| **类型** | `int`(i64)、`float`(f64)、`string`、`bool`、`nil`、`array`、`struct`、`function` |
| **字面量** | `42`、`0xFF`、`0b1010`、`1_000`；`3.14`、`1e10`、`2.5e-3`；`"hi"`、`"a ${x} b"`；`true`/`false`/`nil`；`[1,2,3]`；`Point{ x: 1, y: 2 }`；`{ "k": 1 }`（匿名字典） |
| **运算符优先级** | 见 §2.4（高→低） |
| **注释** | `//` 行注释；`/* ... */` 块注释（**不可嵌套**） |
| **语句终结** | 换行终结；`;` 可作显式分隔符与空语句；`(` `[` 内换行被忽略；`{` 块内换行有效 |
| **分支** | `if` 是**表达式**：`let m = if c { 1 } else { 2 }` |
| **循环** | `while cond { ... }`、`for v in seq { ... }`，配 `break`/`continue` |
| **函数** | `fn name(a,b){ ... }`（末尾表达式即返回值）、`(x) => x*2` 箭头、`fn (x) { ... }` 匿名；一等函数、闭包、递归 |
| **array** | `[a,b,c]`、`a[i]` 读写、负索引从尾部计、内置 `len/push/pop/...` |
| **struct** | `struct Name { 字段, fn 方法(){} }`；实例 `Name{...}`；`s.k` ≡ `s["k"]`；动态加字段；方法隐式 `self` |
| **IO** | `print(...)`、`input(prompt?)`、`str()/int()/float()/type()` |

### 2.1 样例 A —— 学生成绩统计（struct + array + 函数 + 循环 + 插值 + 管道 + assert）

```
// ============================================================
// grades.lfz — 学生成绩统计
// 覆盖：struct/方法、array、函数、while/for/if、字符串插值、
//       管道 |>、assert、基本 IO
// ============================================================

struct Student {
    name:  ""                       // 字段默认值
    score: 0

    fn isTop() => self.score >= 90              // 箭头方法（单表达式）
    fn line()  => "${self.name}: ${self.score:>3}"   // 插值 + 格式说明符
}

// 降序排序：返回新数组（选择排序）
fn sortDesc(xs) {
    var pool = xs
    var out  = []
    while len(pool) > 0 {
        var best = 0
        for i in range(len(pool)) {
            if pool[i].score > pool[best].score { best = i }
        }
        out  = push(pool[best], out)     // push(value, array)  —— 数据末位
        pool = removeAt(best, pool)      // removeAt(index, array)
    }
    out                                   // 末尾表达式 = 返回值
}

// 平均分：0.0 起步做浮点累加
fn average(xs) {
    assert(len(xs) > 0, "average() 需要非空数组")
    var total = 0.0
    for s in xs { total += float(s.score) }   // 显式 int→float
    total / float(len(xs))
}

// ---- 顶层语句按顺序执行（入口点）----
let roster = [
    Student { name: "Alice", score: 93 },
    Student { name: "Bob",   score: 67 },
    Student { name: "Cara",  score: 88 },
]

print("原始名单：")
for s in roster { print("  " + s.line()) }

let ranked = roster |> sortDesc()          // roster |> sortDesc()  ==  sortDesc(roster)

print("")
print("排名：")
var rank = 1
for s in ranked {
    let star = if s.isTop() { "★" } else { "·" }   // if 是表达式
    print("  ${rank}. ${star} ${s.line()}")
    rank += 1
}

print("")
print("平均分 = ${average(roster):.2f}")
let top = ranked |> maxBy((s) => s.score)   // maxBy(keyFn, array)
print("最高分 = ${top.name} (${top.score})")
```

**运行输出**（预期）：

```
原始名单：
  Alice:  93
  Bob:  67
  Cara:  88

排名：
  1. ★ Alice:  93
  2. · Cara:  88
  3. · Bob:  67

平均分 = 82.67
最高分 = Alice (93)
```

### 2.2 样例 B —— 词频统计（管道链 + 插值 + 合一 struct + check）

```
// ============================================================
// words.lfz — 词频统计
// 覆盖：匿名 struct（字典面孔）、struct 方法、动态字段、
//       管道链、格式说明符、check（非致命自检）
// ============================================================

struct Word {
    text:  ""
    count: 0
    fn bump() { self.count += 1 }               // 方法可修改自身字段
    fn row()  => "${self.text:>10} │ ${self.count:>3}"
}

fn countWords(text) {
    var table = {}                              // 匿名 struct（字典面孔）
    for w in text |> split(" ") {               // split(sep, s)；管道把 s 注入末位
        if has(w, table) {                      // has(key, struct)
            table[w].bump()                     // table["w"] 与 table.w 同源
        } else {
            table[w] = Word { text: w, count: 1 }
        }
    }
    table
}

let text = "the quick the fox the dog a quick fox"
let freq = countWords(text)

check(len(freq) == 5, "期望 5 个不同的词，实得 ${len(freq)}")   // 非致命，失败打警告继续

print("词频 Top 3")
let top3 = freq |> values() |> sortBy((w) => -w.count) |> take(3)
for w in top3 {
    print("  " + w.row())
}

// 格式说明符一览
let pi = 3.14159265
let n  = 42
print("pi=${pi:.3f}  n=${n:05d}  hex=${n:x}  right=${n:>6}")
```

**运行输出**（预期）：

```
词频 Top 3
         the │   3
       quick │   2
         fox │   2
pi=3.142  n=00042  hex=2a  right=    42
```

### 2.3 样例 C —— 排序可视化核心片段（服务 P8 应用，短）

```
// bars.lfz — ASCII 排序可视化核心片段
print("\e[2J\e[H")                       // ANSI 清屏+归位（\e = ESC 0x1B）
seed(12345)                              // 固定随机种子 → 演示可复现
var xs = range(40) |> map((i) => randInt(5, 100))

fn draw(a, hi) {
    var out = "\e[H"
    for i in range(len(a)) {
        let bar = "█" * a[i]              // string * int = 重复
        out += if i == hi { "${bar}  << " } else { bar + "\n" }
    }
    print(out)
}

for i in range(len(xs)) {
    for j in range(len(xs) - i - 1) {
        if xs[j] > xs[j + 1] { xs |> swap(j, j + 1) }   // swap(i, j, array)
        draw(xs, j)
    }
}
print("排序完成")
```

### 2.4 运算符与优先级表（高 → 低，全部左结合）

| 级别 | 运算符 | 说明 | 结合性 |
|---|---|---|---|
| 1（最紧） | `()` `[]` `.` | 调用 / 索引 / 字段 | 左 |
| 2 | `-` `!`（一元） | 取负 / 逻辑非 | 右（前缀） |
| 3 | `*` `/` `%` | 乘 / 除 / 取模 | 左 |
| 4 | `+` `-` | 加 / 减 / 字符串连接 | 左 |
| 5 | **`\|>`** | **管道（推荐放在此处，见开放问题 Q6）** | 左 |
| 6 | `<` `<=` `>` `>=` | 比较 | 左 |
| 7 | `==` `!=` | 相等 | 左 |
| 8 | `&&` | 短路与 | 左 |
| 9（最松） | `\|\|` | 短路或 | 左 |

> 说明：`|>` 被放在**乘除之下、比较之上**（即比 `* / %` 更松、比 `< >` 更紧）。这样：
> - `xs |> sum() > 10` → `(xs |> sum()) > 10` ✅
> - `xs |> sum() + 1` → `xs |> (sum() + 1)` ⚠️（要给管道结果做算术需加括号：`(xs |> sum()) + 1`）
> - `1 + 2 |> f` → `(1 + 2) |> f` ✅
> 直观规则：**`|>` 的优先级 = "比加减乘除都松，但比任何比较/逻辑都紧"**，与 Elixir 的 `|>` 位置一致（已被大量代码验证）。备用方案见 Q6。

类型化运算符（无隐式转换，见特色 5）：

- `+`：`int+int`、`float+float`、`string+string`（连接）；混合 int/float 见 Q4。
- `- * / %`：数值。
- `*`：额外支持 `string * int`（重复，如 `"█" * 5`）。
- 比较：数值之间、`string` 之间（按 UTF-8 字节序）；`bool` 仅 `== !=`；`nil` 仅 `== !=`。
- `==`/`!=`：标量按值；`array`/`struct` 按**深结构相等**；`function` 按**同一性**（见特色 5 边界）。
- 逻辑 `&& ||`：两侧与结果都必须是 `bool`，短路求值。

---

## 3. 五个特色：详细语法、语义、例子、边界

### 特色 1 —— 管道 `|>` + `_` 占位符

#### 1.1 语法形态

```
pipe_expr = mul_expr { "|>" pipe_rhs } ;
pipe_rhs  = postfix | lambda ;        (* 必须是"可调用物"：函数名 / 调用 / 方法 / lambda *)
```

`_` 是一个**保留占位符 token**，只在管道右侧有意义。

#### 1.2 语义规则（parser 阶段脱糖，运行时零开销）

给定 `L |> R`，按下列规则**在解析期**改写成普通调用：

1. **R 是调用 `F(a₁,…,aₙ)`**：
   - 若参数中**恰好有一个 `_`** → 把该 `_` 替换为 `L`：`F(a₁,…,L,…,aₙ)`（任意位插入）。
   - 若**没有 `_`** → 把 `L` **追加为最后一个实参**：`F(a₁,…,aₙ, L)`（尾参注入 / data-last）。
   - 若有 **≥2 个 `_`** → 语法错误 `E-PIPE-002`。
2. **R 是非调用表达式**（函数名、方法访问、lambda）→ 变成 `R(L)`。
3. **R 求值后不是函数** → 运行时错误 `E-PIPE-001`。
4. `_` 出现在**管道右侧之外** → 错误 `E-PIPE-003`。
5. `L |> R1 |> R2` 左结合 = `R2(R1(L))`；`_` 可出现在任意嵌套深度（`L |> f(g(_))` → `f(g(L))`）。

> **data-last 约定**：因为默认尾参注入，LFZ 的内置库一律把"被操作的数据"放在**最后一个参数**，形成"先写函数/配置、后接数据"的风格：`map(f, xs)`、`filter(p, xs)`、`split(sep, s)`、`take(n, xs)`。这正是 `xs |> map(f)` 读起来自然的原因。

#### 1.3 例子（≥2）

```
// 例 1：链式数据处理
let total = [1, 2, 3, 4, 5] |> filter((x) => x % 2 == 1) |> map((x) => x * x) |> sum()
print(total)                    // 55

// 例 2：任意位插入
print(10 |> div(_, 2))          // 5     （div(a,b) = a/b）
print(2  |> div(10, _))         // 5     （把 2 插到第二参位）
print("abc" |> repeat(3))       // abcabcabc  （repeat(count, str) 的 count 在前 → 尾注入 s）
```

#### 1.4 边界情况

- `_` 是**占位符而非 lambda 参数**：`xs |> maxBy((s) => s.score)` 中 `_` 不出现；若要"取字段作 key"必须写完整 lambda，不能写 `_.score`（那是 `E-PIPE-003`）。
- `x |> f` 与 `x |> f()` 等价（都注入为 `f(x)`）。
- `x |> 5` → `E-PIPE-001`（5 不可调用），**运行时**报错。
- `x |> f() |> g()` 左结合，`_` 每段独立。
- 管道**不能**是赋值目标；`x |> f() = 1` 语法错误。

---

### 特色 2 —— 合一 `struct`（字典 == 对象，一种结构两种面孔）

#### 2.1 语法形态

```
struct_decl = "struct" IDENT "{" [ member { ("," | NEWLINE) member } [","] ] "}" ;
member      = fn_decl | IDENT ":" expression ;     (* 字段默认值 或 方法 *)
struct_lit  = [ IDENT ] "{" [ field_init { "," field_init } [","] ] "}" ;
field_init  = ( IDENT | STRING ) ":" expression ;
```

- **声明**：`struct Student { name: "", score: 0, fn isTop() => ... }` 会把 `Student` 绑定为一个**模板值**。
- **实例化**：`Student { name: "A", score: 9 }` 新建一个实例，先复制模板的字段默认值与方法，再用实参覆盖（**平拷贝**，无原型链——见 Q5）。
- **匿名结构**：`{ "k": v }` / `{}` 直接构造字典式 struct（表达式位置出现 `{` 即 struct 字面量；块位置出现 `{` 才是块）。

#### 2.2 语义规则

1. **`s.k` ≡ `s["k"]`**：字段统一以**字符串**为键存储，`.k` 是 `["k"]` 的语法糖。两者读写同一槽位。
2. **读缺失字段 → 错误 `E-FIELD-001`**（**不返回 nil**，这是"无魔法"）。用 `has(k, s)` 先探测。
3. **写字段会创建或更新**：`s.z = 1` 若 `z` 不存在则**动态新增**（字段是开放的）。
4. **方法 = 函数值字段**：struct 中所有函数字段都是"方法"。**访问函数值字段（无论用 `.` 还是 `[]`）返回已绑定 `self` 的方法闭包**，因此 `p.f` 与 `p["f"]` 结果一致且都可直接调用，`let g = p.norm; g()` 也可用。
5. **`self`**：仅在方法体内可用（词法绑定到调用时的接收者）。方法声明不写 `self` 参数，调用 `p.m(a)` 时自动把 `p` 绑到 `self`。
6. **确定性键序**：`keys(s)` / `values(s)` / `entries(s)` 一律按**键的字节序升序**返回（实现上 struct 用有序/可排序的键表），保证输出可复现。
7. **相等**：`==` 对 struct 为**深结构相等**（键集相同且逐键值相等，与插入顺序无关）。
8. `str(s)` 显示为 `{k1: v1, k2: v2}`，键按升序（确定性）。

#### 2.3 例子（≥2）

```
// 例 1：对象面孔 + 字典面孔是同一次存储
struct Point { x: 0, y: 0, fn norm2() => self.x * self.x + self.y * self.y }

let p = Point { x: 3, y: 4 }
print(p.x)            // 3
print(p["x"])         // 3      —— 与 p.x 同源
p["y"] = 10           // 用字典写法改 y
print(p.y)            // 10     —— 对象写法读到新值
print(p.norm2())      // 109

// 例 2：动态字段 + 方法绑定
let d = {}            // 匿名字典
d.name = "LFZ"
d["year"] = 2026
print(d)                       // {name: LFZ, year: 2026}   （键升序）
let show = d.name              // 仅举字段；方法绑定示例：
let d2 = { "f": fn (n) { return self.name + "-" + str(n) } }
print(d2.f(1))                 // LFZ 风格：self 自动绑定为 d2
```

> 注：`d2.f` 中的函数是"手动放入的方法"，因为访问函数字段会自动绑定 `self`，所以它同样能拿到 `self`。

#### 2.4 边界情况

- **键名规则**：`.k` 的 `k` 必须是标识符；`[expr]` 接受**任意字符串**（`s["a b"]`、`s["1"]` 合法）。键一律按字符串比较，故 `s.x` 与 `s["x"]` 同键。
- **模板 vs 实例**：实例是模板的**平拷贝**，之后改模板不影响已有实例（Q5 可改为原型链/共享方法表）。
- **`self` 逃逸**：`let f = p.norm2; f()` → `self` 仍绑定 `p`（已捕获），合法。
- **函数值也能存在数组里**，但**数组元素不会自动绑定 self**：`[p.norm2][0]()` 中已带 `self`；而 `let a = [fn(){...}]; a[0]()` 无 `self`，方法体内引用 `self` 会 `E-NAME-001`。
- **循环引用**：`a.push(a)` 后对 `a` 做深相等比较会形成环 → `E-RT-002`（见特色 5 边界）。

---

### 特色 3 —— 富字符串插值

#### 3.1 语法形态

```
STRING        = '"' { string_char | interpolation } '"' ;
interpolation = "${" expression [ ":" format_spec ] "}" ;
format_spec   = [ [fill] align ] [sign] [width] [ "." precision ] [type]
              ; align = "<" | ">" | "^" ;  type = "d" | "x" | "X" | "o" | "b" | "f" | "e" | "s"
```

#### 3.2 语义规则（parser 阶段脱糖）

1. 字符串字面量在解析期被拆成"字面段 + 插值段"，插值段脱糖为表达式 + 转 `str` +（可选）按 format_spec 格式化，再整体连接。
2. **转义**：`\n \t \\ \" \e`（`\e` = ESC 0x1B，用于 ANSI）；`\$` = 字面 `$`。因此 `\${name}` 输出字面文本 `${name}`。
3. **裸 `$`**：`$` 后不是 `{` 时按字面处理（`"$5"` = `$5`）。
4. **嵌套**：`${ ... }` 内是完整表达式，可含字符串字面量并**递归插值**：`"${ "x=${x}" }"` 合法。扫描时按 `{}` 深度配对，且跳过内层字符串字面量。
5. **format_spec 的分隔**：第一个**深度为 0** 的 `:` 分隔表达式与说明符（因此 `${ {a:1} }` 里的 `:` 属结构体字面量，不被当分隔符）。
6. **类型默认**：`int`→`d`、`float`→最短往返（`1.0` 显示为 `1.0`）、`string`→`s`、`bool`→`true/false`、`nil`→`nil`、`array`/`struct`→同 `str()`。
7. **不合法说明符** → `E-FMT-001`；说明符与值类型不符（如对 string 用 `d`）→ `E-FMT-002`。

#### 3.3 例子（≥2）

```
// 例 1：基础插值 + 转义
let name = "LFZ"
print("hello ${name}!")        // hello LFZ!
print("\${name}")              // ${name}
print("$5 + ${1 + 2}")         // $5 + 3

// 例 2：格式说明符
let pi = 3.14159265
let n  = 42
print("${pi:.3f}")             // 3.142
print("${n:05d}")              // 00042
print("${n:x}")                // 2a
print("${n:>8}|")              //       42|
print("${n:^8}|")              //   42    |
print("${ {a: 1}.a }")         // 1        （结构体字面量里的 : 不被当分隔符）
```

#### 3.4 边界情况

- `"${}"` / `"${:d}"` → `E-FMT-001`（空表达式）。
- 未闭合 `"${x"` → `E-LEX-002`。
- 嵌套插值深度不限，但实现须防递归深度攻击（`E-RT-001`）。
- 插值表达式内出现换行 → 允许（`${ ... }` 内换行被忽略）。
- `${x:%}` 等不支持的类型符 → `E-FMT-001`。

---

### 特色 4 —— 结构化错误 + `assert` / `check`

#### 4.1 语法形态

```
assert(cond, message?)     // cond 必须 bool；false → 致命错误，终止程序
check(cond, message?)      // cond 必须 bool；false → 打结构化警告，返回 false，继续执行
fail(message?)             // 主动抛出致命错误（等价 panic）
```

#### 4.2 语义规则

1. **错误对象字段**：`category`（类别）、`code`（错误码）、`message`（说明）、`span`（文件:行:列）、`hint`（修复建议，可空）、`frames`（调用栈，可空）。
2. **错误类别**：`lex`（词法）/ `syntax`（语法）/ `name`（名字）/ `type`（类型）/ `arith`（算术）/ `index`（索引）/ `field`（字段）/ `pipe`（管道）/ `io`（输入输出）/ `conv`（转换）/ `fmt`（格式化）/ `assert`（断言）/ `runtime`（运行时）。
3. **错误码**（示例，完整表在 semantics.md 冻结）：

   | 码 | 类别 | 触发条件 | 消息模板（用户可见） |
   |---|---|---|---|
   | `E-LEX-001` | lex | 遇到非法字符 | `非法字符 '{c}'` |
   | `E-LEX-002` | lex | 字符串未闭合 | `字符串字面量在此处未闭合` |
   | `E-SYN-001` | syntax | 意外的 token | `这里期待 {expected}，但得到 {got}` |
   | `E-NAME-001` | name | 变量未定义 | `未定义的变量 '{name}'` |
   | `E-NAME-003` | name | 给 `let` 绑定重新赋值 | `'{name}' 是 let 绑定，不能重新赋值` |
   | `E-TYPE-001` | type | 二元运算类型不匹配 | `运算符 '{op}' 不支持 {lt} 与 {rt}` |
   | `E-TYPE-002` | type | 条件/逻辑位置不是 bool | `此处要求 bool，得到 {t}` |
   | `E-TYPE-004` | type | 实参个数不符 | `'{fn}' 期待 {n} 个参数，传入 {m} 个` |
   | `E-ARITH-001` | arith | i64 溢出 | `整数溢出：{a} {op} {b}` |
   | `E-ARITH-002` | arith | 整数除以零 | `除以零` |
   | `E-INDEX-001` | index | 数组越界 | `索引 {i} 超出数组范围 [0, {n})` |
   | `E-FIELD-001` | field | 读不存在的字段 | `结构体没有字段 '{k}'`（hint: 用 `has` 先探测） |
   | `E-PIPE-001` | pipe | 管道值不可调用 | `管道右侧不是函数` |
   | `E-PIPE-002` | pipe | 多个 `_` | `管道中最多只能有一个 '_' 占位符` |
   | `E-PIPE-003` | pipe | `_` 出现在管道之外 | `'_' 只能在管道右侧使用` |
   | `E-CONV-001` | conv | 转换失败 | `无法把 {v} 转换为 int` |
   | `E-FMT-001` | fmt | 格式说明符非法 | `非法的格式说明符 '{spec}'` |
   | `E-ASSERT-001` | assert | assert 失败 | `断言失败：{msg}` |
   | `E-RT-001` | runtime | 递归/求值栈过深 | `调用栈过深（超过 {limit} 层）` |
   | `E-RT-002` | runtime | 比较循环结构 | `无法对循环引用的结构做相等比较` |

4. **文本输出格式**（默认，ANSI 自动降级）：
   ```
   error[E-TYPE-002]: 此处要求 bool，得到 int
     --> grades.lfz:23:9
      |
   23 |     if n { print("x") }
      |        ^ 这里得到 int
      = hint: 改为 `if n != 0` 或 `if n > 0`
   ```
5. **`--json` 机器可读输出**（供工具链/Agent/AI 指南）：
   ```json
   {"status":"error","category":"type","code":"E-TYPE-002",
    "message":"此处要求 bool，得到 int","file":"grades.lfz","line":23,"col":9,
    "hint":"改为 `if n != 0` 或 `if n > 0`","frames":[]}
   ```
6. **退出码**（D-008）：`0` 成功；`1` 测试用例失败（runner 语义）；`2` 语法/运行时/环境错误。
7. **v1 不可恢复**：没有 `try/catch`。唯一"非致命"原语是 `check`（返回 `false` 并打 `warning[E-CHECK-001]` 到 stderr，继续跑）。`assert`/`fail` 致命。是否引入可恢复错误见 Q9。

#### 4.3 例子（≥2）

```
// 例 1：assert 致命 + check 非致命
fn avg(xs) {
    assert(len(xs) > 0, "avg() 需要非空数组")      // 空数组 → E-ASSERT-001，终止
    var t = 0
    for x in xs { t += x }
    t / len(xs)
}
check(avg([1,2,3]) == 2, "平均分算错了")            // true，无输出
let r = check(avg([2,4]) == 9, "期望 3")            // false → 打 warning，r = false，继续
print("仍然在运行，r = ${r}")                        // 仍然在运行，r = false

// 例 2：结构化错误的定位与建议（演示错误路径）
// 下面一行会报错：
//   error[E-TYPE-002]: 此处要求 bool，得到 int
//     --> demo.lfz:2:5
//   = hint: 改为 `if n != 0` 或 `if n > 0`
let n = 3
if n { print("x") }        // ← E-TYPE-002（条件必须是 bool）
```

#### 4.4 边界情况

- `assert` 的 `cond` 非 bool → `E-TYPE-002`（而非当作 truthy）。
- `assert` 失败时 `message` 缺省为 `断言失败`；`message` 会作为 `E-ASSERT-001` 的 message。
- `check` 的失败**不改变控制流**，仅返回 `false`；可用于黑盒测试的"软断言"。
- 错误报告是**首错即停**（v1 不做错误恢复 / 多处报错）。

---

### 特色 5 —— 确定性语义「无魔法」

规格级纪律，几乎零实现代码，换取最大正确率。

#### 5.1 规则清单（每条均为二值可判定）

1. **条件必须 `bool`**：`if` / `while` 条件、`&&` / `||` 两侧、一元 `!` 操作数、`assert`/`check` 首参。非 bool → `E-TYPE-002`。无 truthiness（`if 0`、`while 1` 都是错误）。
2. **无隐式转换**：不存在 `"1" + 1`、`1 + "a"` 之类的自动转换；用 `int()/float()/str()` 显式转换。**唯一例外**：`int` 与 `float` 的算术/比较中，`int` 会被**无损加宽**为 `float`（见 Q4；若 Q4 选严格，则连这条也取消）。
3. **整数**：`int` = 有符号 64 位（i64）。**溢出即错** `E-ARITH-001`（不回绕）。`/` 为整数除法（**向零截断**），`%` 余数符号随被除数。整数除/模零 → `E-ARITH-002` / `E-ARITH-003`。
4. **数组**：0 基索引；**负索引从尾部计**（`a[-1]` = 末元素）；越界（含赋值越界）→ `E-INDEX-001`；**索引写入不会自动扩容**。
5. **`for v in xs` 的快照语义**：进入循环时对 `xs` 的元素列表做**浅拷贝**，循环体修改 `xs`（增删）不影响本轮迭代次数；元素内部若为可变 struct，改动仍可见（共享引用）。
6. **`for k in s`（遍历 struct）**：按**键升序**迭代，保证确定性。
7. **求值顺序写死**：二元运算先左后右；调用先取被调再从左到右求实参；数组/struct 字面量按源顺序求值；`&&`/`||` 短路。
8. **`==` 语义**：标量按值；`array`/`struct` 深结构相等；`function` 仅同一性（`f == f` 真，两个内容相同的 lambda 不等）。对**循环引用**做深相等 → `E-RT-002`（不静默、不栈溢出）。
9. **`float` 遵循 IEEE-754 双精度**：`0.0 / 0.0` = `NaN`、`1.0 / 0.0` = `Inf`（**不报错**，按 IEEE 定义）；`NaN != NaN` 为真，含 NaN 的 `<`/`>` 一律 false。**整数除零才报错**。
10. **递归深度上限**：默认 10000 层求值帧，超出 → `E-RT-001`（保护宿主栈，Rust 实现尤其重要）。
11. **`input()` 语义**：读到 EOF 返回 `nil`；`input(prompt)` 先把 `prompt` 打到 stdout（不换行）并 flush，再读一行，去掉结尾 `\n` / `\r\n`。IO 失败 → `E-IO-002`。
12. **`print` 语义**：各实参按显示形式（`str`）拼接，中间单个空格，末尾单个 `\n`；字符串顶层不加引号，嵌套在 `array`/`struct` 内的字符串显示时加引号以便区分。
13. **`keys`/`values`/`entries` 键序**：一律**字节序升序**（不依赖插入顺序、不依赖哈希种子）。
14. **RNG**：`seed(n)` 固定随机序列；不调用 `seed` 时用时间种子（唯一一处非确定性，且被隔离在显式 API 内）。见 Q12。

#### 5.2 例子（≥2）

```
// 例 1：无 truthiness、无隐式转换、溢出即错
// if 1 { }                       // ✗ E-TYPE-002（条件必须是 bool）
// let s = "a" + 1                // ✗ E-TYPE-001（string + int 不支持）
let x = "a" + str(1)              // ✓ "a1"
// let y = 9223372036854775807 + 1  // ✗ E-ARITH-001（i64 溢出）

// 例 2：整数除法 / 负数索引 / for 快照
print(7 / 2)          // 3     （向零截断）
print(-7 / 2)         // -3
print(7 % 3)          // 1
let a = [10, 20, 30]
print(a[-1])          // 30    （负索引从尾部计）
// print(a[3])         // ✗ E-INDEX-001
for v in a { a = [] } // 循环体清空 a，但本轮仍迭代 3 次（快照）
```

#### 5.3 边界情况

- `-7 % 3` → `-1`（余数随被除数符号）；若用户期望 Python 的 `2`，这是明确的差异点（见 Q3）。
- `a[i] = v` 需要 `i` 已在范围内；想追加用 `push(v, a)`。
- 深相等遇环 → 错误而非无限递归。
- `float` 与 `int` 比较：若 Q4 选"加宽"，`1 == 1.0` 为真。

---

## 4. EBNF 文法草图（ISO EBNF，覆盖 v1 构造）

```
(* ============================================================
   LFZ v1 grammar — ISO EBNF
   约定：
   - NEWLINE 是有效语句分隔符；lexer 在 ( ) 与 [ ] 嵌套深度 >0 时抑制 NEWLINE。
   - 所有二元运算符左结合；precedence 见 §2.4。
   - 终结符用大写或引号；(* * ) 为注释。
   ============================================================ *)

program        = { NEWLINE | statement } ;

statement      = decl_stmt | assign_stmt | fn_decl | struct_decl
               | while_stmt | for_stmt
               | return_stmt | break_stmt | continue_stmt
               | expr_stmt
               | ";" ;                                     (* 空语句 *)

decl_stmt      = ( "let" | "var" ) IDENT "=" expression ;
assign_stmt    = lvalue assign_op expression ;
assign_op      = "=" | "+=" | "-=" | "*=" | "/=" | "%=" ;
lvalue         = IDENT { ( "." IDENT | "[" expression "]" ) } ;

fn_decl        = "fn" IDENT "(" [ params ] ")" block ;
struct_decl    = "struct" IDENT "{" [ members ] "}" ;
members        = member { ( "," | NEWLINE ) member } [ "," ] ;
member         = fn_decl | IDENT ":" expression ;

while_stmt     = "while" expression block ;
for_stmt       = "for" IDENT "in" expression block ;
return_stmt    = "return" [ expression ] ;
break_stmt     = "break" ;
continue_stmt  = "continue" ;
expr_stmt      = expression ;

block          = "{" { NEWLINE | statement ";"? } "}" ;
params         = IDENT { "," IDENT } ;

(* -------- 表达式：高 precedence -> 低 -------- *)
expression     = or_expr ;
or_expr        = and_expr { "||" and_expr } ;
and_expr       = eq_expr { "&&" eq_expr } ;
eq_expr        = cmp_expr { ( "==" | "!=" ) cmp_expr } ;
cmp_expr       = add_expr { ( "<" | "<=" | ">" | ">=" ) add_expr } ;
add_expr       = mul_expr { ( "+" | "-" ) mul_expr } ;
mul_expr       = pipe_expr { ( "*" | "/" | "%" ) pipe_expr } ;
pipe_expr      = unary { "|>" pipe_rhs } ;
unary          = ( "-" | "!" ) unary | postfix ;
postfix        = primary { call | index | field } ;
call           = "(" [ args ] ")" ;
index          = "[" expression "]" ;
field          = "." IDENT ;
args           = expression { "," expression } [ "," ] ;
pipe_rhs       = postfix | lambda ;

primary        = INT | FLOAT | STRING
               | "true" | "false" | "nil" | "self"
               | IDENT | "_"
               | array_lit | struct_lit | if_expr | lambda
               | "(" expression ")" ;

array_lit      = "[" [ args ] "]" ;
struct_lit     = [ IDENT ] "{" [ field_inits ] "}" ;
field_inits    = field_init { "," field_init } [ "," ] ;
field_init     = ( IDENT | STRING ) ":" expression ;

if_expr        = "if" expression block [ "else" ( if_expr | block ) ] ;
lambda         = "fn" "(" [ params ] ")" block
               | "(" [ params ] ")" "=>" ( expression | block ) ;

(* -------- 词法 -------- *)
IDENT          = ( letter | "_" ) { letter | digit | "_" } ;
letter         = "a" | ... | "z" | "A" | ... | "Z" ;        (* Unicode 字母见 Q? *)
digit          = "0" | ... | "9" ;
INT            = dec_int | "0x" hex_digits | "0b" bin_digits | "0o" oct_digits ;
dec_int        = digit { digit | "_" } ;
FLOAT          = digit { digit | "_" } "." digit { digit | "_" } [ exponent ]
               | digit { digit | "_" } exponent ;
exponent       = ( "e" | "E" ) [ "+" | "-" ] digit { digit } ;
STRING         = '"' { string_char | interpolation } '"' ;
interpolation  = "${" expression [ ":" format_spec ] "}" ;
NEWLINE        = "\n" | "\r\n" | "\r" ;
COMMENT        = "//" { any_char_except_newline } | "/*" { any_char } "*/" ;
```

**关键字（v1 保留）**：`let` `var` `fn` `return` `if` `else` `while` `for` `in` `break` `continue` `struct` `true` `false` `nil` `self`
**为将来保留（当前用法即报错）**：`match` `class` `import` `from` `try` `catch` `rescue` `yield` `async` `await` `and` `or` `not` `is`

---

## 5. 功能范围：IN（v1）与 OUT（延后）

### 5.1 IN —— v1 实现

| 项 | 理由 |
|---|---|
| `int`(i64) / `float`(f64) / `string` / `bool` / `nil` | task-info 硬要求的"基本数据类型"，另纳 float（D-011，应用需要除法/比例） |
| `array` / `struct` | task-info 硬要求的"数组、结构类型" |
| `let` / `var` + 复合赋值 | 可变性模型清晰，支撑应用 |
| 完整运算符 + 优先级表 + 结合性 | 语法自洽的最低要求 |
| 注释 `//` `/* */` | 可读性 |
| `if`(表达式) / `while` / `for..in` / `break` / `continue` | task-info 硬要求"分支循环控制" |
| 函数定义/调用/递归/闭包/箭头 lambda | task-info 硬要求"函数定义和调用" |
| `print` / `input` + `str/int/float/type` | task-info 硬要求"基本输入输出" |
| **特色①管道 `\|>` + `_`** | 身份标识、应用/文档加分 |
| **特色②合一 struct** | 身份标识、解释器/应用加分 |
| **特色③富字符串插值** | 极低成本、应用/文档加分 |
| **特色④结构化错误 + assert/check** | 解释器/测试/应用三向加分 |
| **特色⑤确定性语义** | 全项正确率（spec 级纪律） |
| 内置库：`len/range/push/pop/removeAt/insert/slice/swap/min/max/sum/minBy/maxBy/sort/sortBy/map/filter/reduce/take/drop/each/keys/values/entries/has/del/split/join/trim/upper/lower/replace/repeat/startsWith/abs/floor/ceil/round/sqrt/pow/rand/randInt/seed/assert/check/fail` | 支撑应用与测试；均为 data-last 约定 |
| ANSI 转义经 `\e` 原样输出（`print` 即可） | P8 动画零成本前置（BRAINSTORM §4） |

### 5.2 OUT —— v1 不做（列 v1.1 backlog）

| 延后项 | 一句理由 |
|---|---|
| 模块 / import | 单文件脚本已满足课程；模块系统成本高、收益低 |
| 类 / 继承 / 原型链 | 合一 struct 已覆盖对象需求，避免复杂度 |
| 静态类型注解 / `lfz check` 检查器 | 解释器 20 分优先；检查器成本高 |
| `match` 模式匹配 | 成本最高，裁剪后收益有限 |
| `try/catch` / 值式错误处理 `?`/`rescue` | v1 用 `assert`(致命) + `check`(非致命) 已够；可恢复错误见 Q9 |
| 生成器 / 惰性流 `yield` | 高成本且（原 Python 语境）性能负项，D-007 已延后 |
| 一等区间 / `..` 切片语法 | v1 用内置 `slice(from,to,a)` 代替；区间类型延后 |
| 标签 `break` / 值式循环 | 应用不需要，延后 |
| REPL | 非评分项，P4 可选 |
| `lfz fmt/doc/init/package/watch` | 工具链加分项，非核心 |
| 位运算符 `& \| ^ ~ << >>` | 与 `\|>`/`\|\|` 词法冲突，且应用不需要 |
| 多行字符串（三引号） | 应用可用 `+`/插值绕过；降低 lexer 复杂度 |
| 宏 / 元编程 / 泛型 | 远超课程范围 |
| 运算符重载 | 破坏"无魔法"，明确排除 |

---

## 6. 待用户决策的开放问题（每条：岔路 + 我的推荐 + 影响面）

> 编号即"需拍板清单"。带 ★ 的是会**影响大量代码/内置库签名**的高杠杆决策。

**Q1（★）语句终结：换行 vs 分号**
- A) 换行终结 + `;` 可选（KICKOFF 已倾向）→ 可读、像 Python/Elixir；**实现风险**：lexer/parser 要做换行敏感处理（括号内抑制换行等）。
- B) 强制分号终结 → parser 最简单、最稳，但代码更嘈杂。
- **推荐 A**，但请确认能接受换行敏感的解析复杂度（这是 core-dev 的主要风险点）。

**Q2 可变性模型：`let` / `var` 的力度**
- A) `let` 只锁**绑定**（不可重新赋值），但所引用的 array/struct **内容仍可改**；`var` 可重绑。
- B) `let` 递归不可变（内容也不可改）→ 更安全但更繁琐，且与"方法修改 self"冲突。
- **推荐 A**（与样例一致）。

**Q3（★）整数与除法：宽度、溢出、`/`、`%`**
- 整数宽度：A) i64（推荐）／B) 任意精度大整数。
- 溢出：A) **溢出即错**（推荐，符合"无魔法"）／B) 回绕（Rust release 风格）。
- `/`：A) int/int = **整数除法**（推荐）／B) 永远返回 float（Python3 风格，需引入 `//` 整数除）。
- 负除法舍入：A) **向零截断**（C/Rust，推荐）／B) 向下取整（Python `//`）。
- `%` 符号：随 A 为"随被除数"（`-7 % 3 == -1`）。
- **推荐 A/A/A/A**。若你更看重"初学者直觉/Python 友好"，可改 `//` + 向下取整。

**Q4（★）int/float 混合运算**
- A) **严格禁止**：`1 + 2.0` 报错，必须 `float(1) + 2.0` → 最纯粹，但与"可用性"摩擦大。
- B) **仅在此处加宽**：int 与 float 相遇时 int 无损加宽为 float → 可用性好，且是唯一、完整、无损的"隐式"。
- **推荐 B**，并在 spec 中把这条写成"唯一允许的转换"。请确认是否接受对"无隐式转换"的这一例外。

**Q5 struct 实例化语义**
- 实例是模板的**平拷贝**（推荐，简单确定）／还是**原型链/共享方法表**（省内存、允许改模板影响实例）。
- 以及：是否需要"位置构造" `Point(1, 2)`（推荐**不做**，只用 `Point{ x:1, y:2 }`）。
- **推荐：平拷贝 + 无位置构造**。

**Q6（★）`|>` 的优先级**
- A) **比加减乘除松、比比较/逻辑紧**（= 本草案 §2.4 表，Elixir 位置）→ `1+2 |> f` 正确；但 `xs |> sum() + 1` 需写 `(xs |> sum()) + 1`。
- B) 最低（在 `||` 之下）→ 完全可预测，但 `xs |> sum() > 10` 需加括号。
- C) 最高（紧贴一元）→ `xs |> sum() * 2` 正确，但 `1+2 |> f` 会算成 `1 + (2 |> f)`。
- **推荐 A**（经过 Elixir 大规模验证），请确认。

**Q7（★）管道注入位置：尾参 vs 首位**
- A) **尾参注入 / data-last**（D-007 原文"尾参注入"，推荐）→ 内置库写成 `map(f, xs)`、`split(sep, s)`；`xs |> map(f)` 自然，`_` 用于插任意位。
- B) **首位注入 / data-first**（Elixir 风格）→ 内置库写成 `map(xs, f)`、`split(s, sep)`；直觉更接近 `xs.map(f)`，但 `_` 的作用变小。
- **推荐 A**（更"管道原教旨"、更独特），但它决定**所有内置函数的参数顺序**，请务必确认。

**Q8 struct 字段名规则**
- A) `.` 后必须是标识符；`["任意字符串"]` 可作键；键统一为 string（推荐）→ 满足"s.x ≡ s['x']"。
- B) 只允许标识符键 → 更简单，但失去字典灵活性。
- **推荐 A**。

**Q9 错误可恢复性**
- A) v1 **不可恢复**：`assert`/`fail` 致命、`check` 非致命并返回 bool（推荐）→ 实现简单、测试友好。
- B) v1 引入 `try/catch` → 表达力强，但成本高、与"首错即停"的报告冲突。
- **推荐 A**；请确认 `check` 的"打印警告 + 返回 false + 继续"语义符合你的预期。

**Q10 块/函数隐式返回 + `if` 作为表达式**
- A) 块的值 = 最后一条表达式的值；函数末尾表达式即返回；`if` 是表达式（推荐，"万物皆值"）→ 代码简洁。
- B) 必须显式 `return`；`if` 是语句 → 更传统、实现更直接。
- **推荐 A**；若选 B，§2 的两个样例需要改写（`return` 补全、去掉 `let star = if ...`）。

**Q11 负索引与相等语义（打包确认）**
- 负索引 `a[-1]`：**允许**（推荐）／一律越界。
- `==` 对 array/struct：**深结构相等**（推荐）／只允许标量比较。
- 循环结构深比较：**报 `E-RT-002`**（推荐）／按同一性短路。
- **推荐：允许 / 深相等 / 报错**。

**Q12 RNG 与"确定性"叙事**
- A) 内置 `rand/randInt/seed`，默认时间种子，**显式 `seed` 才可复现**（推荐；排序可视化演示需要）→ 唯一非确定来源，被隔离。
- B) v1 不提供 RNG（严格确定性）→ 应用只能演示固定输入。
- **推荐 A**。

**Q13 入口点**
- A) **顶层语句顺序执行**（推荐，像 Python/脚本）。
- B) 必须定义 `main()` 才执行。
- **推荐 A**（样例已按 A 写）。

**Q14 标识符字符集（次要）**
- A) ASCII 字母/数字/`_`（推荐，lexer 最简）。
- B) 允许 Unicode 字母（可写中文变量名）。
- **推荐 A**，除非你想演示中文标识符。

---

## 7. 对 Rust 实现的直接影响（D-011，供 core-dev/runtime-dev 参考，非本草案主体）

- 运行时值：`enum Value { Int(i64), Float(f64), Str(Rc<str>), Bool(bool), Nil, Array(Rc<RefCell<Vec<Value>>>), Struct(Rc<RefCell<StructObj>>), Func(Rc<Closure>) }`。
- `StructObj`：字段用 `BTreeMap<String, Value>`（天然按 key 升序 → 满足特色 5 的确定性键序，且免第三方 crate）＋可选模板引用（若 Q5 选原型链）。
- 闭包/环境：捕获的变量用 `Rc<RefCell<Value>>` 单元；方法绑定 `self` 时生成绑定闭包。
- 错误：`enum LfzError { Lex{..}, Syntax{..}, Name{..}, Type{..}, Arith{..}, Index{..}, Field{..}, Pipe{..}, Io{..}, Conv{..}, Fmt{..}, Assert{..}, Runtime{..} }`，实现 `code()/category()/span()/hint()`，统一转文本/JSON。
- 求值：`fn eval(node, env) -> Result<Value, LfzError>`；递归深度计数 → `E-RT-001`。
- 管道在 **parser 阶段脱糖**为普通 `Call` 节点（运行时零开销，满足"高性能"目标）。

---

## 8. 需用户拍板（速览）

- **Q1 语句终结**：换行 + 可选 `;`（推荐）／强制分号。
- **Q2 可变性**：`let` 只锁绑定、内容可改（推荐）／深不可变。
- **Q3 整数**：i64 + 溢出报错 + `/` 整数除（向零截断）+ `%` 随被除数（推荐）／大整数 / 永远浮点除。
- **Q4 int/float 混合**：仅算术/比较加宽（推荐）／严格禁止。
- **Q5 struct 实例**：模板平拷贝、无位置构造（推荐）／原型链。
- **Q6 `\|>` 优先级**：比算术松、比比较紧（推荐，Elixir 位置）／最低／最高。
- **Q7 管道注入位**：**尾参 data-last**（推荐，决定全部内置签名）／首位。
- **Q8 struct 字段名**：`.` 需标识符、`[]` 任意串、键为 string（推荐）。
- **Q9 错误恢复**：v1 不可恢复，`assert` 致命 + `check` 非致命返回 bool（推荐）／引入 `try/catch`。
- **Q10 值语义**：块/函数隐式返回 + `if` 是表达式（推荐）／必须 `return`。
- **Q11 负索引 & 深相等**：允许 / 深相等 / 环报错（推荐）。
- **Q12 RNG**：内置 `rand/randInt/seed`，显式 seed 可复现（推荐）／不提供。
- **Q13 入口**：顶层语句执行（推荐）／必须 `main()`。
- **Q14 标识符**：ASCII（推荐）／允许 Unicode。

> 其中 **Q3 / Q4 / Q6 / Q7 / Q10** 是最影响代码手感与内置库形态的五条；**Q1** 最影响解析器风险。请优先回这六条，其余可按推荐默认。
