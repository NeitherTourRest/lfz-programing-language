# LFZ v1 语义规范（semantics.md）

> 本文件是 LFZ v1 的唯一事实源（冻结于 2026-09-23）。变更须先 ADR，后改文档。
> 作者: language-architect ｜ 状态: **spec v1 = FROZEN** ｜ 冻结 ADR: `.opencode/team/DECISIONS.md` D-016「spec v1 冻结」
> 冻结基线: `.opencode/team/DRAFT-LFZ-v0.5.md`（含 3 处冻结前补钉）
> 同族文件: [syntax.md](./syntax.md) ｜ [interface-contract.md](./interface-contract.md)

本文件定义 LFZ 的**意义**：值模型与引用语义、作用域 / 闭包 / cell、求值顺序与确定性语义（§4.5）、错误语义（§8）。**形式**（词法、EBNF、语句 / 表达式 / 块、优先级、歧义审计）见 [syntax.md](./syntax.md)；**实现契约**（`Span` / 帧 / `LfzError` / `ScopeDebug` / loader / 内置函数表）见 [interface-contract.md](./interface-contract.md)。

**本文件内容映射（保留原 DRAFT 章节编号以利交叉引用）**：§3.6、§3.7、§4.2、§4.5（§4.5.0–§4.5.11）、§8（§8.1–§8.4）。
**不在本文件**：§2 / §3.1–§3.5 / §4.1 / §4.3 / §4.4 / §5 / §6 / §7 / §9（→ syntax.md）；§10 / §11 / §13（→ interface-contract.md）；§8.1 的**实现枚举**（→ interface-contract.md §8.1 / §10.4）。

---

## 3.6 `;;` —— 变量 dump 语句（语义）

> 语法（`;;` 的记号化、独立语句、独占逻辑行）见 [syntax.md](./syntax.md) §5-A10；实现契约（per-scope `ScopeDebug`）见 [interface-contract.md](./interface-contract.md) §10.5。

**语义（规范性）**：

1. **触发点**：执行到 `;;` 时，立即输出当前可见变量。
2. **作用域 = 当前可见名字链**：从**最内层作用域向外**遍历（含块作用域、函数局部、闭包捕获、模块顶层）。
3. **顺序**：作用域 **内层 → 外层**；同一作用域内按 **slot 下标升序**；**遮蔽去重**（同名只打印最内层绑定，每名字恰好一行）。
4. **内容**：**所有绑定**，含 `let`/`var`、函数参数、`fn` 名、`struct` 模板名。
5. **行格式（逐字符精确定义）**：`<name> + " ： " + <value>`，分隔串 = `U+0020` `U+FF1A`(全角冒号) `U+0020`。
6. **值渲染** = 与 `print`/`str()` **同一显示形式**（§3.7）。
7. **通道**：**stdout**（与 `print` 同通道，按程序执行顺序交织）。
8. **空链**：无可见变量时输出**零行**，不报错。
9. **可重入**：函数内 `;;` 打印**该次调用**的局部链。

> §14 遗留默认 1/2（`;;` 作用域 = 完整可见链、通道 = stdout）见 [syntax.md](./syntax.md) §14。

---

## 3.7 值的显示形式（供 `print` / `;;` / `str()` 复用）

| 类型 | 顶层 | 嵌套在 array/struct 内 |
|---|---|---|
| `nil` | `nil` | `nil` |
| `bool` | `true`/`false` | 同 |
| `int` | 十进制 | 同 |
| `float` | 最短往返；整值浮点显示 `.0`（`1.0`） | 同 |
| `string` | **原样，不加引号** | **加 `"`，内部按转义输出** |
| `array` | `[e1, e2, …]`（逗号+空格） | 同 |
| `struct` 实例 | `{k1: v1, k2: v2}`，**键按字节序升序**；**不含函数值字段** | 同 |
| `function` | `<fn 名字>`（匿名 `<fn>`） | 同 |
| `struct` 模板 | `<struct 名字>` | 同 |
| 容器自引用 | — | 路径上重复出现的容器输出 **`<cycle>`** |

> **数据面 / 环（v0.5 定稿，A5/A6）**：
> 1. **方法字段不属数据面**：函数值字段（"方法"）**不参与** `keys/values/entries/display/==/has/len`（故 struct 显示只列非函数字段）；但 `s.k ≡ s["k"]` 仍能取到该函数值并调用（`self` 绑定）。
> 2. **环安全**：`print`/`str`/`;;` 渲染时维护"**路径访问集合**"（当前递归路径上的容器身份）；若某容器**已在当前路径上**（自引用环）→ 该处输出字面量 **`<cycle>`**，**不报错、不死循环**。`==` 的环规则见 §4.5.9。

---

## 4.2 类型化运算符（无隐式转换，特色 5；除法 / 取模 v0.5 定稿）

> 优先级 / 结合性（§4.1）、管道语义（§4.3）见 [syntax.md](./syntax.md) §4。

- `+`：`int+int`、`float+float`、`string+string`（连接）。
- `-`：数值（`int` / `float`）。
- `*`：`int*int`、`float*float`，额外支持 `string * int`（重复，`"█" * 5`；`n <= 0` → 空串）。
- **`/`（真除法，A3）**：**永远返回 `float`**（`int / int` 亦得 `float`）。`7 / 2 == 3.5`、`-7 / 2 == -3.5`；除数为零 → `ZeroDivisionError`（**含 float**）。若要**整数向下取整**，用内置 `div(a, b)`（= Python `//`）。
- **`%`（Python 取模，A3）**：`a % b = a - b * floor(a / b)`，**结果符号随除数**；`7 % 3 == 1`、`-7 % 3 == 2`、`7 % -3 == -2`；`b == 0` → `ZeroDivisionError`；`int` / `float` 均适用（混合时先按加宽规则）。
- **`div(a, b)`（内置，整数向下取整）**：两参必须为 `int`，结果 `int`（`div(-7, 2) == -4`）；`b == 0` → `ZeroDivisionError`；非 `int` 参 → `TypeError`。
- 比较：数值之间、`string` 之间（UTF-8 字节序）；`bool` 仅 `== !=`；`nil` 仅 `== !=`。
- `==`/`!=`：标量按值；`array`/`struct` **深结构相等**（**含环安全**，见 §4.5.9）；`function` **同一性**。
- `&& ||`：两侧与结果都必须 `bool`，短路求值。
- **`int`/`float` 混合（v0.5 精确定稿，B13）**：
  - **算术**（`+ - * / %`）：`int` 按 IEEE-754 加宽为 `float`，结果 `float`。加宽在 `|int| > 2^53` 时**可能不精确**（**不再称"无损"**）；这是唯一允许的隐式转换。
  - **比较**（`== != < <= > >=`）：**按两数的数学精确值比较**，**不先加宽**（避免大整数误判）。算法见 §4.5.7。
- **复合类型是引用语义**（A1，详见 §4.5.2）：`array` / `struct` 赋值与传参**共享同一容器**；`a[i] = v` / `s.k = v` **原地修改**；其余内置函数**返回新值、不改原容器**。

---

## 4.5 求值语义规范（v0.5 新增 · 规范性）

> 本节把 v0.4 评审中"按直觉写必翻车"的语义**全部钉死**；实现（core/runtime）不得自行发挥。A1/A2/A4/A5/A6 与 B1–B7/B13 在此集中定义。

#### 4.5.0 值模型与作用域总览（规范性综述，不引入新规则）

> 本节仅**汇总**本文件其它条款已钉死的结论，便于独立阅读；如与下方具体条款冲突，以具体条款为准（无冲突）。

- **值类型（value / 标量）**：`int`（有符号 64 位，`i64`）、`float`（IEEE-754 `f64`）、`bool`、`nil`、`string`（**不可变**，§4.5.11）。标量赋值 / 传参**按值**。
- **引用类型**：`array`、`struct`（实例）。赋值 / 传参**共享同一容器**（§4.5.2）。
- **可调用值**：`function`（含闭包 / λ）；相等仅按**同一性**（§4.2）。
- **模板**：`struct` 模板（非实例），`type(x)` 报 `"struct"`；模板名是绑定（`;;` 可见，§3.6）。
- **条件必须 `bool`**：无 truthiness；`if`/`while`/`&&`/`||`/`filter` 谓词等一律要求 `bool`，否则 `TypeError`（§8.1）。
- **隐式转换**：**唯一**允许的是 `int → float` 加宽（§4.5.7）；除此以外任何类型不符 → `TypeError`。**无** `string → int`、无 `nil` 参与算术等。
- **作用域 = 词法作用域**：名字解析沿**词法可见链**（块 → 函数局部 → 闭包捕获 → 模块顶层）。闭包**按 cell 引用捕获**被捕获局部（§4.5.3）；全局变量不走 cell（直接 global 访问）。循环中每次迭代的块级 `let`/`var` 是**新绑定**（各自新 cell）。

#### 4.5.1 求值顺序（B1）

**一律从左到右**（除非另有说明）：

- **二元运算**：先左操作数，再右操作数。
- **函数调用**：先求被调表达式（callee），再从左到右求实参。
- **数组字面量**：元素按书写序从左到右。
- **struct 字面量 / 实例化**：字段按书写序从左到右（模板默认值按**声明序**先行求值，再按字面**书写序**求值覆盖值；见 §4.5.8）。
- **索引 `a[i]`**：先 `a`，后 `i`。
- **赋值 `lvalue = rhs`**：先对 `lvalue` 求"基对象与下标 / 键"（`a`、`i` / `s`、`k`，从左到右），**再求 `rhs`**，最后写入。
- **复合赋值 `a[i] += rhs` 等**：等价于「先读 `a[i]` → 求 `rhs` → 写回」；`a` 与 `i` **只求值一次**。
- **管道 `L |> F(…)`**：parser 脱糖为普通 `Call`（[syntax.md](./syntax.md) §4.3），故按调用规则求值（`L` 求值后注入到对应实参位；实参仍左→右）。
- **`&&` / `||`**：**短路**，只在必要时求右操作数。
- **字符串插值**：各插值段从左到右求值。
- **`if`**：只求被选中分支；**`while c`**：每次迭代求值 `c`。

#### 4.5.2 复合类型语义 = 引用语义（A1）

- `array` 与 `struct` 是**引用类型**：`let b = a` 后 `b` 与 `a` 指向**同一容器**；`b[i]=v` / `b.k=v` 对 `a` 可见；**传参亦按引用**（容器共享）。
- 标量（`int` / `float` / `string` / `bool` / `nil`）是**值类型**（`string` 不可变，赋值即共享底层但语义上等同值）。
- **赋值语法原地修改**：`a[i] = v`、`s.k = v`、`s["k"] = v` **修改容器本身**（引用共享可见）。索引写入**不自动扩容**，越界 → `IndexError`。
- **`let` 只锁重绑定，不锁内容**：`let a = [1]; a[0] = 2` **合法**；`a = []` 对 `let` **非法**（重绑定）。
- **其余内置函数一律"返回新值、不改原容器"**（A1）：包括 `push` / `pop` / `removeAt` / `insert` / `swap` / `sort` / `sortBy` / `slice` / `map` / `filter` / `take` / `drop` / `del` 等。即**函数式更新**。例：`xs = push(v, xs)`（**不能**依赖 `push` 原地改 `xs`）；`ys = sort(xs)` **不**改 `xs`。
  - **唯一例外**：`a[i] = v` 与 `s.k = v` 两类**赋值语法**（原地）。

#### 4.5.3 闭包捕获（A2）

- 闭包**按 cell 引用捕获**被捕获变量：被捕获的局部升级为共享可变单元；闭包与定义作用域**共享该单元**——外部后续修改对闭包可见，闭包内修改对外部也可见。
- 例：`fn counter() { var n = 0; fn inc() { n += 1; n }; inc }` → 每次调用 `inc` 递增**同一个** `n`。
- 全局变量不参与 cell 捕获（直接按 global 访问）。
- 循环中每次迭代的块级 `let` / `var` 是**新绑定**（各自新的 cell），闭包**各自捕获当次 cell**。
- （实现自由：未被任何闭包捕获的局部无需装箱，但语义如上。）

#### 4.5.4 `for` 迭代快照（B2）与键序（B3）

- `for v in xs`（**array**）：迭代开始时对元素**取快照**（浅拷贝元素序列）。迭代中改 `xs` 的**长度**（增删）**不影响本次迭代次数**；元素若为可变容器，其内部改动**仍可见**（共享引用）。
- `for k in s`（**struct**）：迭代开始时对**数据字段**（非方法）的**键取快照**，按**键的 UTF-8 字节序升序**迭代。
- `for x in range(n)`：`range` 产生普通 array，按 array 快照规则。
- **可迭代类型**：仅 `array` 与 `struct`；对其它类型（含 `string`）`for` → `TypeError`。
- **键序确定性（B3）**：`for k in s`、`keys(s)`、`values(s)`、`entries(s)`、struct display、struct 深相等的键集比较，**一律按键的 UTF-8 字节序升序**；与插入顺序、哈希种子无关。

#### 4.5.5 函数调用、递归深度与 `RecursionError`（B4 / A7）

- 每次函数 / 闭包 / λ 调用产生一个**求值帧**。
- **递归深度上限 = 10000**（默认，含顶层帧）。超过 → **`RecursionError`**（消息 `递归深度超限（超过 10000 层）`），**不是** `OverflowError`。
- **深结构处理**（`==`、display、`str`）使用显式迭代栈，其深度同样受 10000 上限约束，超限 → `RecursionError`。
- 该上限对树遍历器与字节码 VM **一致**。

#### 4.5.6 `float` IEEE 规则（B5）

- `float` 为 IEEE-754 双精度（f64）。
- **NaN / Inf 产地**：`sqrt(x)` 当 `x < 0` → `NaN`；`pow(a, b)` 或算术溢出为无穷 → `±Inf`；显式构造 `float("inf")` / `float("-inf")` / `float("nan")` **支持**。**除零不产 Inf**：`a / 0.0`、`a % 0.0`（含 `div` 的 `b == 0`）→ `ZeroDivisionError`。
- **比较规则（IEEE）**：`NaN == NaN` → `false`；`NaN != NaN` → `true`；任何 `< <= > >=` 涉及 `NaN` → `false`。`Inf == Inf` → `true`；`+Inf` 大于任何有限值。
- **显示**：`NaN` → `nan`，`±Inf` → `inf` / `-inf`（最短往返，§3.7）。
- **排序 / `min` / `max` 的确定性全序**（仅用于 `sort` / `sortBy` / `min` / `max` / `minBy` / `maxBy`）：规定 `-Inf < 任何有限值 < +Inf < NaN`（NaN 排最后）。此全序与 `==` 的 IEEE 语义**并存**（`==` 仍 `NaN != NaN`，但排序结果确定）。

#### 4.5.7 `int` / `float` 混合：加宽与精确比较（B13）

- **加宽**：`int → float` 为 IEEE 最近偶数舍入；`|int| > 2^53` 时**可能不精确**（故不称"无损"）。混合**算术**结果为 `float`。
- **精确比较算法**（`int i` vs `float f`，用于 `== != < <= > >=`）：
  1. 若 `f` 为 `NaN` → `==` false、`!=` true，`< <= > >=` 均 false。
  2. 若 `f == +Inf` → `i < f` true、`i == f` false、`i > f` false（其余类推）。
  3. 若 `f == -Inf` → `i > f` true，`i < f` false，`i == f` false。
  4. 若 `f >= 2^63` → 任何 i64 `i < f`。若 `f < -2^63` → 任何 i64 `i > f`。
  5. 否则（`f` 在 `(-2^63, 2^63)` 内）：令 `t = trunc(f)`（向零取整，可表示为 i64），`r = f - (float)t`（**精确**）。
     - `i < t` → `i < f`；`i > t` → `i > f`；
     - `i == t` → 比较 `0` 与 `r`：`r > 0` → `i < f`；`r < 0` → `i > f`；`r == 0` → `i == f`。
- 该算法给出**数学精确**结果。反例：`9007199254740993 == 9007199254740992.0` → **false**（右值加宽丢精度，精确比较判不等）。
- **显式转换 `int(x)` 的极窄边界（v1 冻结，规范性；冻结前补钉）**：当 `x` 为 `float` 时——
  1. `x` 为 `NaN` → **`ValueError`**（不是 `OverflowError`）；
  2. `x` 为 `+Inf` / `-Inf` → **`OverflowError`**；
  3. `x` 有限 → **向零截断**（trunc）；截断结果**超出 `[i64::MIN, i64::MAX]`** → **`OverflowError`**。
  - 例：`int(2.9) == 2`、`int(-2.9) == -2`（向零，非向下取整）；`int(1e30)` → `OverflowError`；`int(float("nan"))` → `ValueError`；`int(float("inf"))` → `OverflowError`。
  - 与 [interface-contract.md](./interface-contract.md) §10.7 `int` 行、§8.1 `ValueError`/`OverflowError` 行口径一致；`string`/`bool` 实参不涉及本条。
- **数值内置形参加宽（v1 冻结，规范性；与 §1「唯一隐式转换」同源；冻结前补钉）**：`floor` / `ceil` / `round` / `sqrt` / `pow` 的 `int` 实参**先加宽为 `float`**；`abs` **同型不加宽**。逐条全文见 [interface-contract.md](./interface-contract.md) §10.7「数值内置形参加宽」。
- **`floor` / `ceil` / `round` 的 `NaN` / `±Inf` / 超界边界（v1 冻结，规范性；v1 补钉）**：三者返回 `int`，其边界**完全复用上条 `int(x)` 口径**——`NaN` → **`ValueError`**（经 `ValueMsg::Convert`：`src="float"`、`dst="int"`、`text="nan"`，消息 `无法把 float 转换为 int（'nan'）`）；`±Inf` → **`OverflowError`**；有限浮点**取整（floor/ceil/round）后结果超出 `[i64::MIN, i64::MAX]`** → **`OverflowError`**（消息 `整数溢出：结果超出 i64 范围`）。例：`floor(float("nan"))` → `ValueError`；`ceil(float("inf"))` → `OverflowError`；`round(1e30)` → `OverflowError`。实参为 `int` 时先按上条加宽为 `float`（不会落入本边界）；`abs` **不受本条约束**（同型：`float` 的 `NaN` 原样返回）。口径与 [interface-contract.md](./interface-contract.md) §10.7 `floor`/`ceil`/`round` 行一致。

#### 4.5.8 struct 模板实例化 = 平拷贝（B7）

- `Name { f1: e1, … }`：从模板**平拷贝**——先按模板字段**声明序**求值各**默认值表达式**（**每次实例化重新求值**，故可变默认值如 `[]` 每实例各一份、**不共享**），再按字面**书写序**求值覆盖值并覆盖。
- 模板的**方法**（函数值字段）随实例可用（访问时绑定 `self`），但**不属数据面**（A5）。
- 实例可**动态加字段**：`x.z = v` 在**实例**上新增字段（模板不变）。
- 覆盖提供模板中**不存在**的字段名 → 作为动态字段**新增**（不报错）。
- **无原型链**：改模板**不影响**既有实例。
- 匿名 struct `{}` / `{ "k": v }`：无模板，字段即书写内容。

#### 4.5.9 相等与显示：环安全（A6）与方法字段（A5）

- **`==` 算法**：
  1. 标量（`int`/`float`/`bool`/`nil`/`string`/`function`）按 §4.2、§4.5.6、§4.5.7（`function` 仅**同一性**；`string` 按内容）。
  2. **类型不同**（如 `array` vs `struct`）→ `false`。
  3. 容器（`array` / `struct`）：维护「**已访问有序对集合**」。
     a. 若 `a` 与 `b` **是同一引用** → `true`（身份优先）。
     b. 若 `(a, b)` **已在访问对集合中** → `true`（**重访即视为相等**，环安全）。
     c. 否则把 `(a, b)` 加入集合；比较**形状**（array 长度；struct **数据字段**键集）与逐元素 / 逐字段 `==`；全等 → `true`。
     d. **不报错、不死循环**。
  4. struct 相等**忽略函数值字段**（A5）。
- **显示**（`print` / `str` / `;;`）：递归渲染时维护「**路径访问集合**」（当前路径上的容器身份）；容器**已在路径上** → 输出字面量 **`<cycle>`**；离开时从集合移除。**不报错、不死循环**。
- **`del` 与数据面（A5 扩展补钉，v1 补钉）**：`del(k, s)`（[interface-contract.md](./interface-contract.md) §10.7）**同属数据面操作，仅作用于数据字段**——字段集合与 `keys` / `has` / `len` **完全一致**；`k` 为**方法字段**（函数值字段）时**视为缺失** → **`FieldError`**（消息 `结构体没有字段 '{name}'`）。不变量：**`del(k, s)` 成功 ⟺ `has(k, s) == true`**。字段是否为数据字段**只看值的类型**（函数值字段即方法字段，不论来自模板还是动态添加）。

#### 4.5.10 非致命 `check` 与致命 `assert`（A4）

- `assert(cond, msg?)`：`cond` 必须 `bool`（否则 `TypeError`）；为 `false` → 抛 **`AssertionError`**（**致命**，终止）；消息 `断言失败：{msg}`（`msg` 省略时 `断言失败`）。
- `check(cond, msg?)`：`cond` 必须 `bool`（否则 `TypeError`）；为 `true` → 返回 `true`；为 `false` → 向 **stderr** 写**一行警告**并返回 `false`，**不中断、不抛错、不改变控制流**。警告行格式：`check 失败：{msg}`（`msg` 省略时 `check 失败`）。
- `fail(msg?)`：抛 `AssertionError`（致命）；消息为 `{msg}`（省略时 `fail()`）。
- `check` **不产生任何 `LfzError`**，故不进入 `--json` 的 `error` 集合，也**不改变退出码**。

#### 4.5.11 字符串不可变

- `string` **不可变**；一切"修改"（`+`、`upper`、`replace` …）**返回新字符串**。这利于 `Rc` 廉价 clone 与去别名（与 A1 引用语义配合：字符串按值语义）。

---

## 8. 错误模型（v0.3 全量改写 · Python 风格）

### 8.1 错误类清单（规范，共 12 类 + 1 基类；v0.5 新增 `RecursionError`）

> **取消 `E-xxx` 编号**：用户可见输出**不再**出现编号；`--json` 的机器可读字段使用**类名**。基类 `LfzError` 不被直接抛出。
> **实现侧同源清单**（Rust `enum LfzError` 变体映射）见 [interface-contract.md](./interface-contract.md) §8.1 / §10.4。

| 错误类（class） | 触发条件（充分必要） | 中文消息模板 | 阶段 |
|---|---|---|---|
| `LfzError`（基类） | 不直接抛出 | — | — |
| `CosmosAnswerError` | **`.lfz`** 文件缺少合法 `#42` 前导（[syntax.md](./syntax.md) §2.2.2 `is_lfz` 分支 a–c 任一失败） | **`你忘记了宇宙的答案`**（固定，无参数） | 加载 |
| `SyntaxError` | 词法/语法错误：非法字符、字符串未闭合（含插值内裸换行）、块注释未闭合（`/*` 至 EOF 无 `*/` 匹配）、**未列举的转义**、**整数字面量超出 i64 范围**、意外记号、表达式未结束、单 `;`、同行两语句、赋值目标非法、`#` 位置非法、`_` 位置非法、管道多 `_`、break/continue 在循环外、return 在函数外、非 UTF-8 编码 | 见下表（细分消息） | 加载/解析 |
| `NameError` | 引用未定义的名字 | `未定义的名字 '{name}'` | 运行 |
| `TypeError` | 运算符/条件/调用/参数/格式说明符的**类型**不符；调用非函数；管道右侧非函数；参数个数不符 | 见下表 | 运行 |
| `IndexError` | 数组下标越界（含负索引规范化后越界） | `下标 {i} 越界（长度 {n}）` | 运行 |
| `FieldError` | struct 不存在该字段（`.字段` 或 `["键"]` 读取缺失键）；**`del(k, s)` 的键不是数据字段**（方法字段视为缺失，v1 补钉） | `结构体没有字段 '{name}'` | 运行 |
| `ZeroDivisionError` | 整数/浮点 `/`、`%` 或 `div(a,b)` 的除数为零（**含 float**，B5） | `除以零` / `对零取模` | 运行 |
| `OverflowError` | `int` 运算结果超出 i64 范围；`int(float)` 遇 `±Inf` 或有限浮点截断后超 i64 范围（§4.5.7） | `整数溢出：结果超出 i64 范围` | 运行 |
| `ValueError` | 显式转换失败（`int("abc")`、`int(NaN)` 等）；格式说明符语法非法；**空数组取极值**（`min`/`max`/`minBy`/`maxBy`）；**`randInt` 区间非法**（`lo >= hi`） | `无法把 {src} 转换为 {dst}（'{text}'）` / `格式说明符非法：'{spec}'` / **`空数组没有极值（{func}）`** / **`区间非法：{lo} >= {hi}`** | 运行 |
| `IOError` | `input()` 遇 EOF；不可读文件等 | `输入结束（EOF）` / `无法读取：{path}` | 运行 |
| `AssertionError` | **仅** `assert(cond, msg)` 失败 或 `fail(msg)`（`check` 失败**不**抛此错，A4） | `断言失败：{msg}`（省略时 `断言失败`）/ `{msg}`（`fail` 省略时 `fail()`） | 运行 |
| `RecursionError` | 求值帧深度超限（默认 10000 层）或深结构处理超限（A7/B4） | `递归深度超限（超过 10000 层）` | 运行 |

> **`check` 不抛错（A4，v0.5 定稿）**：`check` 失败**不是** `AssertionError`；它向 **stderr** 写一行警告（格式 `check 失败：{msg}`，省略 `msg` 时 `check 失败`）并返回 `false`，程序继续。故 `check` **不**出现在 `--json` 的 `error` 集合中，也**不改变退出码**。（这消除了 v0.4 §8.1 把 check 列为 `AssertionError` 的自相矛盾。）

**`SyntaxError` 细分消息（中文）**：

| 子场景 | 消息模板 |
|---|---|
| 非法字符 | `非法字符 '{c}'` |
| `#` 位置非法 | `'#' 只能出现在文件首行的前导位；(字符串 / 注释 / 格式说明符内的 '#' 除外)` |
| 字符串未闭合（EOF 或裸换行） | `字符串字面量在此处未闭合` |
| 块注释未闭合（`/*` 至 EOF 无 `*/` 匹配） | `块注释在此处未闭合（缺少 '*/'）` |
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
| 插值表达式内裸换行 | `插值表达式不能跨行；请把表达式写在一行内` |
| 整数字面量超出 i64 范围 | `整数字面量超出 i64 范围` |

**`TypeError` 细分消息（中文）**：

| 子场景 | 消息模板 |
|---|---|
| 运算符类型不符 | `运算符 '{op}' 不支持 {lt} 与 {rt}` |
| 条件非 bool | `条件必须是 bool，得到 {t}` |
| 调用非函数 | `不可调用：{t} 不是函数` |
| 管道右侧非函数 | `管道右侧必须是函数，得到 {t}` |
| 参数个数不符 | `函数 {name} 期待 {n} 个参数，得到 {m}` |
| 格式说明符与值类型不符 | `格式说明符 '{spec}' 不适用于 {t}` |

**`ValueError` 新增子场景（v1 补钉，规范性；冻结前补钉的延续，不新增错误类）**：

| 子场景 | 消息模板 | 触发 |
|---|---|---|
| 空数组取极值 | `空数组没有极值（{func}）` | `min` / `max` / `minBy` / `maxBy` 收到空 `array`（[interface-contract.md](./interface-contract.md) §10.7） |
| 随机区间非法 | `区间非法：{lo} >= {hi}` | `randInt(lo, hi)` 且 `lo >= hi`（[interface-contract.md](./interface-contract.md) §10.7） |

> 二者**均为 `ValueError`**（不新增错误类）；消息经 `ValueMsg` 承载（实现变体 `EmptyExtremum { func }` / `BadRange { lo, hi }` 见 [interface-contract.md](./interface-contract.md) §10.8）。`{func}` = 内置名（`min`/`max`/`minBy`/`maxBy`）；`{lo}`/`{hi}` = 实参十进制原文。

**`IndexError` 的 `idx` / `len` 取值（v1 补钉，规范性）**：`LfzError::Index { idx, len }`（[interface-contract.md](./interface-contract.md) §10.4）两字段取值钉死如下（消息恒为 `下标 {i} 越界（长度 {n}）`，其中 `i = idx`、`n = len`）：

| 触发 | `idx` | `len` |
|---|---|---|
| `pop([])`（无实参） | `-1`（隐含末元素下标） | `0`（容器长度） |
| 支持负索引的内置（`removeAt` / `swap`）越界 | 触发越界的**实参原值** | 越界时容器长度 |
| `insert(i, v, xs)` 且 `i < 0` 或 `i > len(xs)`（**不支持负索引**） | 实参 `i` | `len(xs)` |

> **通用规则**：`idx` = 触发越界的下标（**有实参者用实参原值**；`pop` 无实参 → 隐含末元素下标 `-1`）；`len` = **越界时**容器长度。

> **hint 行策略（v0.4 定案）**：v1 用户可见输出**明确不输出**任何 `hint`/`提示：` 行（保证黑盒测试逐字符稳定）；`CosmosAnswerError` 亦**禁止**任何 hint。原 v0.3 §14-4 的"是否附可选提示行"议题**已关闭（已定）**；可选 hint 能力**列入 v1.1 backlog**（不在 v1 输出契约内）。结构化错误（特色 4）由「类名 + 中文消息 + 位置 + traceback + `--json` 字段」满足。

### 8.2 输出格式（精确规范）

**通用帧格式**（每帧三行）：

```
  File "<path>", line <N>[, in <func>]
    <源码行原文>
    <插入符行>
```

- 源码行原文前固定缩进 **4 个空格**。
- `<func>`：函数名；**顶层帧**显示 `<module>`。
- `<path>`：命令行给定路径原样；非文件模式用伪路径 `<stdin>` / `<repl>` / `<command>`。
- **插入符行** = 4 个空格 + `(列号 - 1)` 个空格 + `^`；`^` 指向**引发错误的最小 AST 节点的首字符**（1-based 列）。词法/记号错误指向**该记号首字符**。
- **列号定义**：**1-based**，按 **Unicode 标量值**计数。含 CJK 宽字符时插入符**可能视觉错位**——这是显示层已知限制，**不影响** `--json` 的 `col`（同为标量计数，可回归测试）。

**两类输出结构**：

- **加载 / 解析期错误**（`CosmosAnswerError`、`SyntaxError`）：**无** `Traceback` 头。
  - `SyntaxError`：`File` 帧（含源码行 + 插入符）→ 末行 `<类名>: <消息>`。
  - `CosmosAnswerError`：**仅** `File "<path>", line 1` → 末行 `CosmosAnswerError: 你忘记了宇宙的答案`（**无源码行、无插入符**）。
- **运行期错误**（其余 **10** 类）：**有** `Traceback (most recent call last):` 头 → 逐帧（**最外层在前、最内层在后**）→ 末行 `<类名>: <消息>`。

**退出码（沿用 D-008）**：`0` 成功；`1` 测试失败；**所有错误类（含 `RecursionError`）→ `2`**。`--json` 时错误仍退出 `2`。**`check` 失败不是错误**，不影响退出码（A4）。

### 8.3 报错输出示例（≥3，逐字符精确）

**示例 1 — 语法错（`SyntaxError`，加载/解析期，无 Traceback 头）**

文件 `demo.lfz`（line 1 = `#42`）：
```
#42
let x = 1 $ 2
```
输出（`$` 位于第 2 行第 11 列，1-based）：
```
  File "demo.lfz", line 2
    let x = 1 $ 2
              ^
SyntaxError: 非法字符 '$'
```

**示例 2 — 运行时错（含调用栈，`ZeroDivisionError`）**

文件 `calc.lfz`：
```
#42
fn half(n) {
    n / 0
}
let r = half(10)
print(r)
```
输出（外层 `half(10)` 在 line 5、第 9 列；内层 `n` 在 line 3、第 5 列）：
```
Traceback (most recent call last):
  File "calc.lfz", line 5, in <module>
    let r = half(10)
            ^
  File "calc.lfz", line 3, in half
    n / 0
    ^
ZeroDivisionError: 除以零
```

**示例 3 — 缺前导（`CosmosAnswerError`，无 Traceback 头、无源码行/插入符）**

文件 `forgot.lfz`：
```
print("hello")
```
输出：
```
File "forgot.lfz", line 1
CosmosAnswerError: 你忘记了宇宙的答案
```

**示例 4 — `--json`（内部用类名；用于机器可读 / 黑盒测试）**

```
lfz run --json forgot.lfz
```
输出（单行 JSON；`ok:false`）：
```json
{"ok":false,"error":"CosmosAnswerError","message":"你忘记了宇宙的答案","file":"forgot.lfz","line":1,"col":1,"traceback":[{"file":"forgot.lfz","line":1,"func":"<module>"}]}
```

错误类不同则 `error` 字段不同，例如示例 2：
```json
{"ok":false,"error":"ZeroDivisionError","message":"除以零","file":"calc.lfz","line":3,"col":5,"traceback":[{"file":"calc.lfz","line":5,"func":"<module>"},{"file":"calc.lfz","line":3,"func":"half"}]}
```

> **`--json` 的 `error` 字段取值集合（v0.5）**：`CosmosAnswerError` | `SyntaxError` | `NameError` | `TypeError` | `IndexError` | `FieldError` | `ZeroDivisionError` | `OverflowError` | `ValueError` | `IOError` | `AssertionError` | **`RecursionError`**（共 **12** 个）。`check` 失败**不**产生 `error`（见 §8.1）。

**示例 5 — `check` 非致命（A4）**

文件 `chk.lfz`：
```
#42
check(1 == 2, "软断言示例")
print("继续运行")
```
stdout：
```
继续运行
```
stderr（一行）：
```
check 失败：软断言示例
```
退出码：`0`（`check` 失败不影响退出码，也不产生 `--json` 错误）。

### 8.4 `--json` 字段与实现枚举（指针）

- `--json` 单行对象字段：`ok`(bool) / `error`(类名，仅 `ok:false`) / `message`(中文消息) / `file` / `line`(1-based) / `col`(1-based，Unicode 标量) / `traceback`(数组，元素 `{file,line,func}`)。
- `func` 取值：顶层 `<module>`、命名函数为函数名、匿名函数 `<fn>`；闭包保留其**定义名**供 traceback。
- **类名 → Rust 变体映射**与 `class_name()` / `message()` / `span()` 契约见 [interface-contract.md](./interface-contract.md) §8.1 / §10.4。

---

*（本文件为 LFZ v1 语义规范，冻结于 2026-09-23；与 [syntax.md](./syntax.md)、[interface-contract.md](./interface-contract.md) 同族。任何语法/语义/契约变更须先写 ADR，再由 language-architect 改 `docs/spec`。）*
