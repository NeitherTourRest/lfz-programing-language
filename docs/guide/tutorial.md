# LFZ 教程：从零写一个「成绩分析器」

> **学完你能做什么**：从零掌握 LFZ 的核心语法（`#42`、变量、控制流、函数与递归、闭包、结构体、管道 `|>`、富字符串插值、`;;`、`assert`/`check`、字符串处理），并独立写出一个能运行的小程序。
> 每一步都有**完整可运行代码**和**实测输出**。照着敲，一次就能成功。
> 权威定义见 [`docs/spec/`](../spec/)；本教程只讲怎么用。配套：[`README.md`](README.md) · [`reference.md`](reference.md) · [`errors.md`](errors.md) · [`testing.md`](testing.md)
> 最后更新: 2026-09-27（T11-③ D1–D7：字符串处理 / 对齐 / 循环绑定 / 退出码） by docs-writer

**说明**：所有示例的第一行都是 `#42`（LFZ 文件前导）。本文中的每段代码都已用 `cargo run --quiet -- run <文件>` 实际运行过，输出为**真实结果**。

---

## 准备

在仓库根目录构建一次：

```console
$ cargo build
```

之后每个示例：把代码存成 `xxx.lfz`，然后 `cargo run --quiet -- run xxx.lfz`。

---

## Step 1 — 第一个程序：`#42` 与 `print`

**概念**：每个 `.lfz` 文件的第一行**必须恰好**是 `#42`，其后紧跟换行。这是 LFZ 的"仪式"，缺失会报 `CosmosAnswerError`（`docs/spec/syntax.md` §2.2）。`print` 把参数按显示形式拼接、空格连接、末尾换行，写到 stdout。

**代码**（文件 `s1.lfz`）：

```lfz
#42
print("Hello, LFZ!")
```

**运行**：

```console
$ cargo run --quiet -- run s1.lfz
Hello, LFZ!
```

**常见错误**：把 `#42` 写成 `# 42`（有空格）、`#42abc`、`#43`，或文件第 1 行是空行——都会报 `CosmosAnswerError: 你忘记了宇宙的答案`（`docs/spec/syntax.md` §6-B5）。

---

## Step 2 — 变量：`let` / `var` 与算术

**概念**：`let` 声明**不可重绑定**的变量；`var` 声明**可重绑定**的变量。`+=`、`-=`、`*=`、`/=`、`%=` 是复合赋值。注意：`let` 只锁"重新绑定"，不锁容器内容（`docs/spec/semantics.md` §4.5.2）。

**代码**（文件 `s2.lfz`）：

```lfz
#42
let pi = 3.14
var count = 0
count = count + 1
count += 2
print("pi = ${pi}, count = ${count}, type = ${type(count)}")
```

**运行**：

```console
$ cargo run --quiet -- run s2.lfz
pi = 3.14, count = 3, type = int
```

**常见错误**：对 `let` 变量重新赋值 → `TypeError: 不能重新赋值 let 变量 'pi'；let 只锁重绑定，不锁内容`（`docs/spec/semantics.md` §8.1）。需要可变就声明成 `var`。

---

## Step 3 — 控制流：`if` / `while` / `for`

**概念**：条件**必须**是 `bool`（无 truthiness，`docs/spec/semantics.md` §4.5.0）。`if` 是表达式（分支值可赋给变量）；`while` 循环；`for x in xs` 遍历 `array` 或 `struct`。花括号 `{ }` 定义块；语句用**换行**分隔（不是分号）。

**代码**（文件 `s3.lfz`）：

```lfz
#42
let n = 5
if n > 0 {
    print("${n} 是正数")
} else {
    print("${n} 不是正数")
}

var i = 0
var acc = 0
while i < n {
    acc += i
    i += 1
}
print("0..${n - 1} 的和 = ${acc}")

let xs = [10, 20, 30]
for x in xs {
    print("元素 ${x}")
}
```

**运行**：

```console
$ cargo run --quiet -- run s3.lfz
5 是正数
0..4 的和 = 10
元素 10
元素 20
元素 30
```

**常见错误**：
- 条件写成 `if n { ... }`（`n` 是 `int`）→ `TypeError: 条件必须是 bool，得到 int`。
- 用 `;` 分隔语句 → `SyntaxError: 单独的 ';' 非法；打印变量请用 ';;'`（`docs/spec/syntax.md` §5-A11）。

**补充（容易误解的两点）**：
- **`else if` 链可用**：`if a { ... } else if b { ... } else { ... }` 是合法写法，可一路串多个分支。
- **循环体里的 `let` / `var` 每轮都是新绑定**：循环每执行一轮就新建一个变量单元，可以在循环内部放心用 `let`；闭包捕获的是**当轮**那个单元（`docs/spec/semantics.md` §4.5.0、§4.5.3）。实测：

```lfz
#42
var i = 0
var fns = []
while i < 3 {
    let x = i * 2
    fns = push(() => x, fns)
    i += 1
}
for f in fns { print("捕获当轮 cell -> ${f()}") }
```

```console
$ cargo run --quiet -- run s3b.lfz
捕获当轮 cell -> 0
捕获当轮 cell -> 2
捕获当轮 cell -> 4
```
得到 `0 2 4`——证明每轮的 `x` 互不干扰（若共享同一个单元，三个都会打印 `4`）。

---

## Step 4 — 函数与递归

**概念**：`fn 名字(参数) { ... }` 定义函数。函数体**末尾表达式的值就是返回值**，`return` 只用于提前退出（`docs/spec/syntax.md` §1）。函数可以调用自己（递归）。

**代码**（文件 `s4.lfz`）：

```lfz
#42
fn fact(n) {
    if n <= 1 { 1 } else { n * fact(n - 1) }
}
fn fib(n) {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}
print("fact(5) = ${fact(5)}")
print("fib(10) = ${fib(10)}")
```

**运行**：

```console
$ cargo run --quiet -- run s4.lfz
fact(5) = 120
fib(10) = 55
```

**常见错误**：递归太深（超过 10000 层）→ `RecursionError: 递归深度超限（超过 10000 层）`（`docs/spec/semantics.md` §4.5.5）。

---

## Step 5 — 闭包

**概念**：函数是一等值，可以返回另一个函数。内部函数**按 cell 引用捕获**外层变量：闭包与定义处**共享同一个变量单元**，所以计数器能累加（`docs/spec/semantics.md` §4.5.3）。

**代码**（文件 `s5.lfz`）：

```lfz
#42
fn makeCounter() {
    var n = 0
    fn inc() {
        n += 1
        n
    }
    inc
}
let c = makeCounter()
print("${c()} ${c()} ${c()}")
```

**运行**：

```console
$ cargo run --quiet -- run s5.lfz
1 2 3
```

**关键点**：每次调用 `c()` 都在同一个 `n` 上递增——这正是"按 cell 捕获"的效果。

---

## Step 6 — 结构体与方法

**概念**：`struct 名字 { 字段: 默认值, fn 方法() => ... }` 定义模板；用 `名字 { 字段: 值 }` 实例化。字段用 `.字段` 访问，也能用 `["键"]`（对象/字典两种面孔）。方法通过 `self` 引用实例；修改字段用 `s.k = v`（原地修改，**即使 `s` 是 `let` 也合法**，因为不是重绑定，`docs/spec/semantics.md` §4.5.2）。

**代码**（文件 `s6.lfz`）：

```lfz
#42
struct Student {
    name: "",
    score: 0,
    fn isTop() => self.score >= 90,
    fn line() => "${self.name}(${self.score})",
}
let s = Student { name: "Alice", score: 93 }
print(s.line())
print(s.isTop())
s.score = 80
print("${s.name} 现分数 ${s.score}, isTop=${s.isTop()}")
let d = { "k": 1 }
print(d["k"])
```

**运行**：

```console
$ cargo run --quiet -- run s6.lfz
Alice(93)
true
Alice 现分数 80, isTop=false
1
```

**常见错误**：
- 成员之间漏逗号 → `SyntaxError`（`docs/spec/syntax.md` §5-A7）；尾逗号可选。
- 用关键字当字段名（如 `r.self`）→ `SyntaxError`；需要该键时用字符串键 `r["self"]`（`docs/spec/syntax.md` §2.6）。

---

## Step 7 — 管道 `|>` 与 data-last

**概念**：`L |> F(...)` 把 `L` 作为**最后一个参数**注入 `F`（这叫 **data-last**，`docs/spec/interface-contract.md` §10.7）。想插到别的参数位，用 `_` 占位（每条管道最多一个 `_`）。`|>` 比 `+ - * /` 松、比比较运算符紧（`docs/spec/syntax.md` §4.1）。

**代码**（文件 `s7.lfz`）：

```lfz
#42
let xs = [5, 3, 8, 1]
let total = xs |> sum()
print("总和 = ${total}")
let doubled = xs |> map((x) => x * 2)
print("翻倍 = ${doubled}")
let big = xs |> filter((x) => x > 3)
print("大于 3 = ${big}")
let sorted = xs |> sort()
print("升序 = ${sorted}")
print("用 _ 插位: ${8 |> div(_, 2)}")
```

**运行**：

```console
$ cargo run --quiet -- run s7.lfz
总和 = 17
翻倍 = [10, 6, 16, 2]
大于 3 = [5, 8]
升序 = [1, 3, 5, 8]
用 _ 插位: 4
```

**关键点**：`xs |> sum()` 等价于 `sum(xs)`；`8 |> div(_, 2)` 等价于 `div(8, 2)`。
**常见错误**：管道右侧写成 `xs |> sum() + 1` → `SyntaxError`；要写 `(xs |> sum()) + 1`（`docs/spec/syntax.md` §4.4）。

---

## Step 8 — 富字符串插值

**概念**：字符串里用 `"${表达式}"` 插入值；`:` 后可跟**格式说明符**（`docs/spec/syntax.md` §2.8）。常见说明符：`d`（整数）、`x`（小写十六进制）、`f`（定点小数）、`>`/`<`/`^`（对齐）、宽度、`05`（补零）。字符串内的 `{`、`}` 是普通字符；`\${` 输出字面量 `${`。

**代码**（文件 `s8.lfz`）：

```lfz
#42
let name = "LFZ"
let n = 42
let pi = 3.14159
print("你好，${name}！")
print("n=${n:05d}  hex=${n:x}  right=${n:>6}")
print("pi=${pi:.2f}")
print("字面量花括号: { } 与美元: \${n}")
```

**运行**：

```console
$ cargo run --quiet -- run s8.lfz
你好，LFZ！
n=00042  hex=2a  right=    42
pi=3.14
字面量花括号: { } 与美元: ${n}
```

**常见错误**：非法说明符 → `ValueError: 格式说明符非法：'...'`；说明符与值类型不符 → `TypeError: 格式说明符 '...' 不适用于 ...`（`docs/spec/semantics.md` §8.1）。

**对齐与 fill**：`<`（左）、`>`（右）、`^`（居中）都可用，且可在 `align` 前加一个 **fill 字符**。实测（`s = "ab"`）：

```lfz
#42
let s = "ab"
print("[${s:<5}]")     // 左对齐
print("[${s:>5}]")     // 右对齐
print("[${s:^5}]")     // 居中
print("[${s:*^7}]")    // fill '*' + 居中
print("[${s:*<7}]")    // fill '*' + 左对齐
print("[${42:05d}]")   // 补零
```
```console
$ cargo run --quiet -- run s8b.lfz
[ab   ]
[   ab]
[ ab  ]
[**ab***]
[ab*****]
[00042]
```

> ⚠️ **宽度必须是字面数字**，不支持动态宽度：`"${s:>w}"`（`w` 是变量）→ `ValueError: 格式说明符非法：'>w'`。要按变量宽度对齐，用字符串拼接 + `repeat` 手工补齐（见 Step 13）。

---

## Step 9 — `;;`：一键打印当前可见变量

**概念**：单独一行 `;;`（两分号）是**变量 dump 语句**：执行到它时，立即把当前**所有可见变量**写到 stdout，顺序为**内层作用域 → 外层作用域**，同一层按声明序，同名只打印最内层（`docs/spec/semantics.md` §3.6）。它**独占逻辑行**。

**代码**（文件 `s9.lfz`）：

```lfz
#42
let alpha = 1
var beta = "two"
fn greet(n) => "${n}!"
struct Point { x: 0, y: 0, }
let p = Point { x: 1, y: 2 }
;;
print("done")
```

**运行**：

```console
$ cargo run --quiet -- run s9.lfz
alpha ： 1
beta ： two
greet ： <fn greet>
Point ： <struct Point>
p ： {x: 1, y: 2}
done
```

**关键点**：每行格式为 `<名字> ： <值>`（全角冒号、两侧各一个空格）。`;;` 与 `print` 同走 stdout，按执行顺序交织。

---

## Step 10 — 错误处理：`assert` 与 `check`

**概念**（`docs/spec/semantics.md` §4.5.10）：
- `assert(cond, msg?)`：**致命**。`cond` 为 `false` → 抛 `AssertionError` 并终止程序。
- `check(cond, msg?)`：**非致命**。返回 `bool`；失败时向 **stderr** 写一行警告并返回 `false`，程序**继续**，不改变退出码。
- `fail(msg?)`：直接抛 `AssertionError`（致命）。

`assert` 用来断言语义不变量；`check` 用来"软检查"可接受的边界情况。

**代码 A**（文件 `s10a.lfz`，`check` 不中断）：

```lfz
#42
check(1 == 1, "相等")
check(1 == 2, "不相等（软断言，不致命）")
print("check 之后继续运行")
assert(2 + 2 == 4, "算术应当正确")
print("assert 通过，继续")
```

**运行**（stdout 与 stderr 分开看）：

```console
$ cargo run --quiet -- run s10a.lfz
check 之后继续运行
assert 通过，继续
```
stderr（一行警告）：
```text
check 失败：不相等（软断言，不致命）
```
退出码 `0`。

**代码 B**（文件 `s10b.lfz`，`assert` 致命）：

```lfz
#42
assert(1 == 2, "一比二大？")
print("这行不会执行")
```

**运行**：

```console
$ cargo run --quiet -- run s10b.lfz
Traceback (most recent call last):
  File "s10b.lfz", line 2, in <module>
    assert(1 == 2, "一比二大？")
    ^
AssertionError: 断言失败：一比二大？
```
退出码 `2`（错误类统一退出码 2，`docs/spec/semantics.md` §8.2）。

---

## Step 11 — 跑测试

LFZ 内建测试运行器。把断言写进 `tests/**/*.lfz`（首行仍是 `#42`），然后一条命令跑全量：

```console
$ cargo run --quiet -- test
...
汇总：共 90 个用例，通过 90，失败 0，错误 0
```

- `assert` 失败 → 该用例 `FAIL`（退出码 `1`）。
- 期望报错的**负例**放 `tests/fixtures/` 并在 `tests/cases.json` 用 `expect.error` 声明，会判 `PASS`。

完整写法与契约见 [`testing.md`](testing.md)。

---

## Step 12 — 完整程序：成绩分析器

把前 11 步的知识合起来，写一个真正能用的小程序：读一批学生成绩，打印名单、按分数排名、算平均分、找最高分。

**代码**（文件 `grades.lfz`）：

```lfz
#42
// grades.lfz — 成绩分析器（教程最终程序）
struct Student {
    name: "",
    score: 0,
    fn grade() => if self.score >= 90 { "A" } else { if self.score >= 80 { "B" } else { "C" } },
    fn line() => "${self.name}(${self.score}): ${self.grade()}",
}

fn average(sts) {
    assert(len(sts) > 0, "average() 需要非空数组")
    var total = 0
    for s in sts { total += s.score }
    total / len(sts)
}

let roster = [
    Student { name: "Alice", score: 93 },
    Student { name: "Bob", score: 67 },
    Student { name: "Cara", score: 88 },
    Student { name: "Dan", score: 100 },
]

;;

print("名单：")
for s in roster { print("  " + s.line()) }

let ranked = roster |> sortBy((s) => -s.score)
print("")
print("排名：")
var rank = 1
for s in ranked {
    print("  ${rank}. ${s.line()}")
    rank += 1
}

print("")
print("平均分 = ${average(roster):.2f}")
let top = ranked |> maxBy((s) => s.score)
print("最高分 = ${top.name}（${top.score}）")
```

**运行**：

```console
$ cargo run --quiet -- run grades.lfz
Student ： <struct Student>
average ： <fn average>
roster ： [{name: "Alice", score: 93}, {name: "Bob", score: 67}, {name: "Cara", score: 88}, {name: "Dan", score: 100}]
名单：
  Alice(93): A
  Bob(67): C
  Cara(88): B
  Dan(100): A

排名：
  1. Dan(100): A
  2. Alice(93): A
  3. Cara(88): B
  4. Bob(67): C

平均分 = 87.00
最高分 = Dan（100）
```

**这段程序用到了**：`struct` + 方法 + `self`（Step 6）、`if` 表达式（Step 3）、函数与 `assert`（Step 4/10）、`array` 与 `for`（Step 3）、管道 `|> sortBy` / `|> maxBy`（Step 7）、lambda `(s) => -s.score`（Step 7）、富字符串插值与 `:.2f`（Step 8）、`;;`（Step 9）、`total / len(sts)`（`/` 返回 `float`，Step 2）。

---

## Step 13 — 处理字符串：不可下标、`split` 与 O(n) 构建

**概念**：LFZ 的 `string` 是**不可变**的 UTF-8 字符串，**不能下标**——`s[0]` 会报 `TypeError`。按字符访问要先把字符串用 `split("", s)` 拆成字符数组（一次 **O(n)**，之后下标 **O(1)**）。另外，**字符串不可变**意味着循环里 `s = s + c` 每轮复制整个前缀，是 **O(n²)**；正确做法是先 `push` 到数组，最后 `join`（**O(n)**）。

**代码**（文件 `s13.lfz`）：

```lfz
#42
let s = "abc"
let cs = split("", s)              // ["a", "b", "c"]，一次 O(n)
print("len(s) = ${len(s)}")        // Unicode 标量数
print("cs[0] = ${cs[0]}")          // 之后下标 O(1)

// 逐字符转大写再拼回：push + join，O(n)
var parts = []
for c in cs {
    parts = push(upper(c), parts)
}
print("out = ${join("", parts)}")
```

**运行**：

```console
$ cargo run --quiet -- run s13.lfz
len(s) = 3
cs[0] = a
out = ABC
```

**常见错误**：
- 对字符串取下标 → `TypeError: 运算符 '[]' 不支持 string 与 array / struct`（`s[0]` 非法）。改用 `split("", s)`。
- 循环里用 `s = s + c` 拼字符串 → 结果对，但**大输入会慢到 O(n²)**。改用数组 `push` + `join("", parts)`。
- `len(s)` 是 **O(n)**；别在循环里反复对同一长串求 `len`（先 `split` 后对数组 `len` 是 O(1)）。

**按变量宽度补齐**（动态宽度不被格式说明符支持时的替代）：

```lfz
#42
fn padRight(w, s) => if w > len(s) { s + repeat(w - len(s), " ") } else { s }
print("[" + padRight(5, "ab") + "]")   // [ab   ]
```

---

## 练习

1. 给 `Student` 加一个方法 `isPass() => self.score >= 60`（返回 `bool`），并在名单里标注是否及格。
2. 用 `check` 统计"不及格人数"，要求不及格时**不中断**程序，只打印警告。
3. 用 `filter` + 管道取出所有 A 档学生，打印数量。
4. 把 `average` 改成用 `reduce` 实现。

### 参考答案

```lfz
#42
// 练习参考：在 grades.lfz 基础上加了 isPass / check / filter / reduce
struct Student {
    name: "",
    score: 0,
    fn grade() => if self.score >= 90 { "A" } else { if self.score >= 80 { "B" } else { "C" } },
    fn isPass() => self.score >= 60,
    fn line() => "${self.name}(${self.score}): ${self.grade()} ${if self.isPass() { "及格" } else { "不及格" }}",
}

fn averageReduce(sts) {
    assert(len(sts) > 0, "averageReduce() 需要非空数组")
    let total = reduce((acc, s) => acc + s.score, 0, sts)
    total / len(sts)
}

let roster = [
    Student { name: "Alice", score: 93 },
    Student { name: "Bob", score: 67 },
    Student { name: "Cara", score: 88 },
    Student { name: "Dan", score: 100 },
    Student { name: "Eve", score: 55 },
]

print("名单：")
for s in roster { print("  " + s.line()) }

let failed = roster |> filter((s) => !s.isPass())
check(len(failed) == 0, "有 ${len(failed)} 人不及格")

let aStudents = roster |> filter((s) => s.score >= 90)
print("A 档人数 = ${len(aStudents)}")
print("平均分 = ${averageReduce(roster):.2f}")
```

运行参考（`eve` 不及格，`check` 只警告不中断）：

```console
$ cargo run --quiet -- run exercise.lfz
名单：
  Alice(93): A 及格
  Bob(67): C 及格
  Cara(88): B 及格
  Dan(100): A 及格
  Eve(55): C 不及格
A 档人数 = 2
平均分 = 80.60
```
stderr：`check 失败：有 1 人不及格`

---

**下一步**：查语法细节看 [`reference.md`](reference.md)；搞懂报错看 [`errors.md`](errors.md)；写自己的测试看 [`testing.md`](testing.md)。
