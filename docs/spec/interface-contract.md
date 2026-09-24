# LFZ v1 接口契约（interface-contract.md）

> 本文件是 LFZ v1 的唯一事实源（冻结于 2026-09-23）。变更须先 ADR，后改文档。
> 作者: language-architect ｜ 状态: **spec v1 = FROZEN** ｜ 冻结 ADR: `.opencode/team/DECISIONS.md` D-016「spec v1 冻结」
> 冻结基线: `.opencode/team/DRAFT-LFZ-v0.5.md`（含 3 处冻结前补钉）
> 同族文件: [syntax.md](./syntax.md) ｜ [semantics.md](./semantics.md)

本文件是 **core-dev / runtime-dev 必须按此实现** 的接口契约：错误类清单（§8.1）、Rust 实现契约（§10：loader 双入口 / `ext(path)` / 唯一 `Span` / `VmFrame` / `TraceFrame` / `LfzError` 变体 / per-scope `ScopeDebug` / §10.7 内置函数表 data-last）、下游角色硬性要求（§11）、旧错误码映射（§13）。**形式**见 [syntax.md](./syntax.md)；**意义**见 [semantics.md](./semantics.md)。

**本文件内容映射**：§8.1（错误类清单）、§10（§10.1–§10.8）、§11（§11.1–§11.5）、§13（附录）。
**不在本文件**：§2–§7 / §9 / §12 / §14 / §15（→ syntax.md）；§3.6 / §3.7 / §4.2 / §4.5（→ semantics.md）；§8.2 / §8.3（→ semantics.md §8，用户可见输出契约）。

---

## 8.1 错误类清单（实现契约）

> **权威来源**：错误类的**触发条件与中文消息**由 [semantics.md](./semantics.md) §8.1 规范性给出，本表为**同一清单的实现侧映射**（类名 ↔ Rust 枚举变体 ↔ 阶段 ↔ `--json` 取值），供 core-dev / runtime-dev 直接落库。两处清单**同源、字形一致**，若有出入以 semantics.md §8.1 的用户可见行为为准并立即报 language-architect。

| 错误类（class，`--json`/`class_name()`） | Rust 变体 | 阶段 | 用户可见消息（中文） |
|---|---|---|---|
| `LfzError`（基类） | （trait/基，不直接抛） | — | — |
| `CosmosAnswerError` | `LfzError::CosmosAnswer { span }` | 加载 | `你忘记了宇宙的答案`（固定；无源码行/插入符/hint） |
| `SyntaxError` | `LfzError::Syntax { msg: SyntaxMsg, span }` | 加载/解析 | 见 [semantics.md](./semantics.md) §8.1 细分表 |
| `NameError` | `LfzError::Name { name, span }` | 运行 | `未定义的名字 '{name}'` |
| `TypeError` | `LfzError::Type { msg: TypeMsg, span }` | 运行 | 见 [semantics.md](./semantics.md) §8.1 细分表 |
| `IndexError` | `LfzError::Index { idx, len, span }` | 运行 | `下标 {i} 越界（长度 {n}）` |
| `FieldError` | `LfzError::Field { name, span }` | 运行 | `结构体没有字段 '{name}'` |
| `ZeroDivisionError` | `LfzError::DivZero { modulo, span }` | 运行 | `除以零` / `对零取模` |
| `OverflowError` | `LfzError::Overflow { span, msg: OverflowMsg }` | 运行 | `整数溢出：结果超出 i64 范围` |
| `ValueError` | `LfzError::Value { msg: ValueMsg, span }` | 运行 | `无法把 {src} 转换为 {dst}（'{text}'）` / `格式说明符非法：'{spec}'` / **`空数组没有极值（{func}）`** / **`区间非法：{lo} >= {hi}`** |
| `IOError` | `LfzError::Io { msg: String, span: Option<Span> }` | 运行 | `输入结束（EOF）` / `无法读取：{path}` |
| `AssertionError` | `LfzError::Assert { msg: String, span }` | 运行 | `断言失败：{msg}` / `{msg}` |
| `RecursionError` | `LfzError::Recursion { depth, limit, span }` | 运行 | `递归深度超限（超过 10000 层）` |

- **共 12 个具体错误类 + 1 基类 `LfzError`**；基类**不直接抛出**。
- **`--json` 的 `error` 取值集合（12 个）**：`CosmosAnswerError` / `SyntaxError` / `NameError` / `TypeError` / `IndexError` / `FieldError` / `ZeroDivisionError` / `OverflowError` / `ValueError` / `IOError` / `AssertionError` / **`RecursionError`**。
- **运行期错误 = 10 类**（除 `CosmosAnswerError`、`SyntaxError` 两个加载/解析期类之外的其余 10 类），在输出中**带 `Traceback` 头**；加载/解析期两类**不带**。见 [semantics.md](./semantics.md) §8.2。
- **`check` 失败不产生任何 `LfzError`**（非致命；stderr 警告 + 返回 `false`），**不**进入 `error` 集合，**不**改变退出码（A4）。
- **退出码**（D-008）：`0` 成功；`1` 测试失败；所有错误类（含 `RecursionError`）→ `2`；`--json` 错误仍 `2`。
- **冻结前补钉口径**：`int(NaN)` → `ValueError`；`int(±Inf)` / 有限浮点截断后超 i64 → `OverflowError`（详见 §10.7 `int` 行与 [semantics.md](./semantics.md) §4.5.7）。

---

## 10. 对 Rust 实现与调试符号的影响（D-011）

### 10.1 新增模块：loader（加载器）

- 新模块 `loader`（建议 `src/loader.rs`），职责（[syntax.md](./syntax.md) §2.2）：读文件 → UTF-8 校验（失败 → `SyntaxError`）→ 跳 BOM → 归一化行终止符 → **按扩展名判定 `ext(path) == "lfz"`（ASCII 大小写不敏感）** → 仅当是 `.lfz` 时校验/消费 `#42` 前导（失败 → `CosmosAnswerError`）。
- 对外暴露两类入口（**v0.4：`load_file` 内部按扩展名分流**）：
  - `load_file(path) -> Result<Loaded, LfzError>`：扩展名 `.lfz` → **要求前导**；否则 → 与 `load_source` 同等（不要求前导）。`Loaded = { text: String, line_base: u32 }`（`line_base` = 1 当且仅当消费了前导行）。
  - `load_source(text) -> Result<Loaded, LfzError>`：**不要求前导**（REPL / stdin / `-e`），`line_base = 0`。
- **`ext(path)` 判定（规范性，与 [syntax.md](./syntax.md) §2.2.0 同源）**：
  ```
  ext(path):
    1. 按 '/' 与 '\' 切分，取最后一段路径分量（'C:' 的 ':' 不参与切分 → Windows 安全）。
    2. 若该段不含 '.' -> 无扩展名。
    3. 否则取【最后一个 '.'】之后的子串为扩展名（可为空串）。
    4. 与 "lfz" 比较时按 ASCII 大小写不敏感。
  ```
  判定**只看路径字符串**：不访问文件系统、不解析符号链接 / 8.3 短名。
- 前导**不产出 token**；行号 = 本地行号 + `line_base`（`.lfz` 程序体自 line 2 起；非 `.lfz` / non-file 自 line 1 起）。

### 10.2 AST 必须携带源码位置（新增硬要求）

- Python 风格报错要求**行列信息**，故 AST 节点**必须**携带 `span`：
  ```rust
  struct Span { line: u32 /*1-based; = 本地行号 + loader.line_base*/, col: u32 /*1-based, Unicode 标量计数*/ }
  ```
- 每个可抛错节点（表达式、语句、记号对应的字面量）**至少**记录其**起始** `Span`。
- 建议在 parser 里维护 `last_span`（当前/最近记号起点），节点构造时填入。
- **这是全项目唯一的 `Span` 定义（B10）**：AST / 错误 / traceback / `--json` **统一用 `line/col`**；**不**使用字节偏移（与 `DRAFT-runtime-arch.md` 的 `Span{start,end}` 旧建议**统一为此定义**）。

### 10.3 调用栈（traceback）与帧命名（B10 / N1 / B11）

- **两类帧必须分开命名**（v0.5 统一定义，B10）：
  - `VmFrame`：**执行 / VM 帧** —— `{ chunk: Rc<Chunk>, ip: usize, base: usize }`（字节码 VM 用；树遍历器可用等价结构）。**不含错误语义**。
  - `TraceFrame`（即语义上的「CallFrame」）：**traceback 帧** —— `{ func_id: u32, span: Span }`。`func_id` 是函数表下标（**不存 `Rc<str>`**，避免每次调用分配）；仅在**报错时**按 `func_id` 反查名字（`<module>` 顶层 / `<fn 名字>` / 匿名 `<fn>`）。
- **`TraceFrame.span` 语义（N1，写死）**：= **该帧当前正在求值的最小 AST 节点的 `Span`**（初始为进入函数体的首节点 span）。**进入一次 Call 时：先把"当前帧"的 `span` 更新为该 `Call` 节点的 `span`（即调用点），再压入新帧**。这与 [semantics.md](./semantics.md) §8.3 示例 2 完全对齐：外层帧 span 指向 `half(10)` 调用点（line 5），内层帧 span 指向 `n`（line 3）。
- 错误发生时把帧栈自**最外层到最内层**序列化进 `LfzError`；错误输出用 `line/col`（`Span{line,col}`），**不**使用字节偏移。
- **traceback 折叠 = 显示层规则（v1 补钉；与序列化分层）**：`LfzError` / `TracedRun` 的帧栈**完整序列化、不折叠**（上一条不变）；**人类可读渲染**（CLI）按 [semantics.md](./semantics.md) §8.2 折叠——帧数 `T > 40` 时输出**首 10 帧** → `  ... 省略 {T−40} 帧 ...` → **尾 30 帧**。规范性常量 **`TRACEBACK_HEAD = 10` / `TRACEBACK_TAIL = 30`（阈值 = 40）**。`--json` 的 `traceback` 数组**不折叠**（[semantics.md](./semantics.md) §8.4）。落点：**tooling-dev** 的 CLI `render_error`；**不改** `LfzError` / `TracedRun`（故**无需 core-dev / runtime-dev 改动**）。
- 闭包需保留**函数名**供 traceback（存 `func_id` → 函数表条目含名字）。

### 10.4 错误枚举

```rust
enum LfzError {
    CosmosAnswer { span: Span },          // span 恒为 line 1, col 1 (B11); 消息固定
    Syntax   { msg: SyntaxMsg, span: Span },
    Name     { name: String, span: Span },
    Type     { msg: TypeMsg, span: Span },
    Index    { idx: i64, len: usize, span: Span },
    Field    { name: String, span: Span },
    DivZero  { modulo: bool, span: Span },
    Overflow { span: Span, msg: OverflowMsg },
    Value    { msg: ValueMsg, span: Span },
    Io       { msg: String, span: Option<Span> },
    Assert   { msg: String, span: Span },
    Recursion{ depth: u32, limit: u32, span: Span },   // v0.5 新增 (A7)
}
```

- `fn class_name(&self) -> &'static str`：返回**类名**（`"SyntaxError"` / `"ZeroDivisionError"` / …），`--json` 用此字段；**不再**有 `E-xxx` 字符串码。
- `fn message(&self) -> String`：返回**中文消息**（[semantics.md](./semantics.md) §8.1）。
- `fn span(&self) -> Option<Span>`：位置。
- **显示层**（CLI）据 `class_name()` 是否为 `"CosmosAnswerError"`/`"SyntaxError"`（加载/解析期）决定是否打印 `Traceback` 头（[semantics.md](./semantics.md) §8.2）。
- **变体 ↔ 类名映射**见 §8.1 表。

### 10.5 DebugSym → per-scope `ScopeDebug`（v0.5 升级，B8）

- **旧 `DebugSym { slots: Vec<String> }` 不足**以支持**块作用域 / 遮蔽 / 闭包捕获**下的 `;;` 可见链。v0.5 升级为**逐作用域**结构：
  ```rust
  struct ScopeDebug {
      first_slot: u32,          // 本作用域首个局部槽位的绝对下标
      names: Vec<FuncNameId>,   // slot → 名字（按声明序；名字 interned，仅在 `;;`/报错时解析）
      parent: Option<ScopeId>,  // 词法可见链的上一层（最外层为 None / 指向模块 scope）
  }
  ```
- **`Dump` 节点携带"最内层活动 scope"的 id**（编译期决议）：`Dump { scope: ScopeId }`。运行时执行 `;;` 时从该 scope 沿 `parent` 链**内→外**遍历，各作用域内按 `first_slot` + 声明序（slot 升序）输出，**遮蔽去重**（同名只取最内层）。语义与 [semantics.md](./semantics.md) §3.6 完全一致。
- **闭包可见链**：每个函数 / 闭包携带其**词法定义处**的 `ScopeDebug` 链（`Rc` 共享），使 `;;` 能看到闭包捕获到的外层名字（[semantics.md](./semantics.md) §3.6 第 2 条）。
- **零热路径开销**：`ScopeDebug` 链**仅在执行 `;;`（DUMP）时被读取**；正常求值路径不访问它（名字解析期构造一次，之后只读）。
- **与 traceback 的区别**：`ScopeDebug` 供**局部变量名**（`;;`）；`TraceFrame.func_id` 供**函数名**（traceback）。两者互补，均**必须**按本文件实现。

### 10.6 其它

- lexer：模式栈 `CODE/STR/INTERP`（[syntax.md](./syntax.md) §2.8）；最大匹配表（§2.3）；CODE 模式下 `#` → `SyntaxError`。**未闭合块注释 `/*` 至 EOF → `SyntaxError`**（`SyntaxMsg::UnterminatedBlockComment`，消息 `块注释在此处未闭合（缺少 '*/'）`，`span` 指向 `/*` 中的 `/`）；**不得**静默消费至空白（[syntax.md](./syntax.md) §2.4、[semantics.md](./semantics.md) §8.1）。
- parser：换行模式栈 `SIG/IGN`（§3.2）+ NO_BRACE_LITERAL 限制位（§3.4）；`;;` 生成 `Dump` 节点；管道脱糖为 `Call`；每节点填 `Span`。字段名须为 `IDENT`：**保留关键字不可作 `.字段` / `member` / `field_init` 的裸字段名**（如 `r.self` → `SyntaxError`；需要该键用 `r["self"]`，[syntax.md](./syntax.md) §2.6 / §9.4）——**此为正确行为，勿改 parser**。**语句首 `{` 不得按裸块处理**（v1 补钉，bug-20260924-07）：LFZ **无 `block_stmt`**（AST `StmtKind` **无 `Block` 变体**），`parse_stmt_seq` 遇语句首 `{` 须落表达式路径 → `struct_lit`（匿名；[syntax.md](./syntax.md) §3.3 规则 2 / §5-A9）；`{ let x = 1 }` / `{ ;; }` → `SyntaxError`；`{ "k": 1 }` / `{ }` → 合法匿名 struct 字面量表达式语句。
- CLI：仅做**错误格式化**与**退出码映射**（0/1/2）；`--json` 输出 [semantics.md](./semantics.md) §8.3 示例 4 的字段。

### 10.7 内置函数表（v0.5 冻结，B9）

> **约定**：`data-last` —— 被操作的数据（`array` / `struct` / `string`）**恒为最后一个参数**；`xs |> f(...)` 把 `xs` 注入末参（[syntax.md](./syntax.md) §4.3）。**凡涉及容器更新的内置一律"返回新值、不改原容器"（A1）**。参数类型不符或数量不符 → `TypeError`。返回类型如下。

**核心 / 数组**

| 内置 | 签名 | 返回 | 说明 |
|---|---|---|---|
| `len` | `len(x) -> int` | `int` | array 元素数；struct **数据字段**数（不含方法，A5）；string 的 Unicode 标量数 |
| `range` | `range(n) -> array` | `array[int]` | `[0, 1, …, n-1]`；`n < 0` → 空数组 |
| `push` | `push(v, xs) -> array` | `array` | 追加 `v` 的**新**数组（不改 `xs`） |
| `pop` | `pop(xs) -> array` | `array` | 去掉末元素的**新**数组；空 → `IndexError` |
| `removeAt` | `removeAt(i, xs) -> array` | `array` | 去掉下标 `i`（支持负索引）的**新**数组；越界 → `IndexError` |
| `insert` | `insert(i, v, xs) -> array` | `array` | 在 `i` 处插入 `v` 的**新**数组；`i ∈ [0, len]`，否则 `IndexError` |
| `swap` | `swap(i, j, xs) -> array` | `array` | 交换 `i` / `j`（支持负索引）的**新**数组；越界 → `IndexError` |
| `slice` | `slice(from, to, xs) -> array` | `array` | `xs[from..to]` 的**新**数组；下标夹取到 `[0, len]`，`from >= to` → 空 |
| `min` | `min(xs) -> value` | 元素 | 全序（[semantics.md](./semantics.md) §4.5.6）；空 → `ValueError` |
| `max` | `max(xs) -> value` | 元素 | 同上 |
| `sum` | `sum(xs) -> number` | `int` / `float` | 全 `int` → `int`（溢出 → `OverflowError`）；含 `float` → `float`；空 → `int 0` |
| `minBy` | `minBy(keyFn, xs) -> value` | 元素 | 以 `keyFn` 结果为准；空 → `ValueError` |
| `maxBy` | `maxBy(keyFn, xs) -> value` | 元素 | 同上 |
| `sort` | `sort(xs) -> array` | `array` | 升序**新**数组（稳定）；全序（§4.5.6）；元素类型须一致可比，否则 `TypeError` |
| `sortBy` | `sortBy(keyFn, xs) -> array` | `array` | 按 `keyFn` 升序**新**数组（稳定） |
| `map` | `map(f, xs) -> array` | `array` | 逐元素 `f` |
| `filter` | `filter(pred, xs) -> array` | `array` | `pred` 须返回 `bool`，否则 `TypeError` |
| `reduce` | `reduce(f, init, xs) -> value` | 任意 | `f(acc, x)` 折叠 |
| `take` | `take(n, xs) -> array` | `array` | 前 `n` 个（`n` 夹取到 `[0, len]`） |
| `drop` | `drop(n, xs) -> array` | `array` | 去前 `n` 个 |
| `each` | `each(f, xs) -> nil` | `nil` | 仅副作用遍历 |

**struct / 字典**

| 内置 | 签名 | 返回 | 说明 |
|---|---|---|---|
| `keys` | `keys(s) -> array` | `array[string]` | **数据字段**键，**字节序升序**（A5/B3） |
| `values` | `values(s) -> array` | `array` | 与 `keys` 同序（A5） |
| `entries` | `entries(s) -> array` | `array[[k,v]]` | 元素为 `[key, value]` 二元数组，按 `keys` 序 |
| `has` | `has(k, s) -> bool` | `bool` | 仅**数据字段**；方法 → `false`（A5） |
| `del` | `del(k, s) -> struct` | `struct` | 去掉 `k` 的**新** struct；缺失 → `FieldError` |

**字符串**

| 内置 | 签名 | 返回 | 说明 |
|---|---|---|---|
| `split` | `split(sep, s) -> array` | `array[string]` | `sep` 为空串 → 按字符切分 |
| `join` | `join(sep, xs) -> string` | `string` | 元素须为 string，否则 `TypeError` |
| `trim` | `trim(s) -> string` | `string` | 去首尾空白 |
| `upper` / `lower` | `upper(s) -> string` / `lower(s) -> string` | `string` | ASCII 大小写（Unicode 折叠见 v1.1） |
| `replace` | `replace(old, new, s) -> string` | `string` | 全部替换 |
| `repeat` | `repeat(n, s) -> string` | `string` | `n <= 0` → 空串；溢出 → `OverflowError` |
| `startsWith` | `startsWith(prefix, s) -> bool` | `bool` | |

**数学 / 随机 / 转换 / IO / 断言**

| 内置 | 签名 | 返回 | 说明 |
|---|---|---|---|
| `abs` | `abs(x) -> number` | **同型** | **参数与返回同型、绝不加宽**（`int→int`、`float→float`）；`int` 的 `i64::MIN` → `OverflowError` |
| `floor` | `floor(f) -> int` | `int` | 向下取整 |
| `ceil` | `ceil(f) -> int` | `int` | 向上取整 |
| `round` | `round(f) -> int` | `int` | **四舍六入五成双**（Python banker's rounding） |
| `sqrt` | `sqrt(f) -> float` | `float` | `f < 0` → `NaN` |
| `pow` | `pow(a, b) -> float` | `float` | 数值幂；溢出 → `±Inf` |
| `div` | `div(a, b) -> int` | `int` | 两参须 `int`；**向下取整**；`b == 0` → `ZeroDivisionError` |
| `rand` | `rand() -> float` | `float` | `[0.0, 1.0)` |
| `randInt` | `randInt(lo, hi) -> int` | `int` | `[lo, hi)`；`lo >= hi` → `ValueError` |
| `seed` | `seed(n) -> nil` | `nil` | 固定随机序列；未调用时用时间种子（唯一非确定源） |
| `str` | `str(x) -> string` | `string` | 显示形式（[semantics.md](./semantics.md) §3.7） |
| `int` | `int(x) -> int` | `int` | `string`/`float`/`bool` → `int`；`float` **向零截断**；**`NaN` → `ValueError`；`±Inf` → `OverflowError`；有限浮点截断后超 i64 → `OverflowError`**（[semantics.md](./semantics.md) §4.5.7）；非法串 → `ValueError` |
| `float` | `float(x) -> float` | `float` | `int`/`string`/`bool` → `float`；非法串 → `ValueError`；支持 `"inf"`/`"-inf"`/`"nan"` |
| `type` | `type(x) -> string` | `string` | `"int"`/`"float"`/`"string"`/`"bool"`/`"nil"`/`"array"`/`"struct"`/`"function"` |
| `print` | `print(...) -> nil` | `nil` | 各参数按显示形式拼接、**空格连接** + 末尾 `\n`（stdout） |
| `eprint` | `eprint(...) -> nil` | `nil` | 同 `print`，但写 **stderr** |
| `input` | `input(prompt?) -> string` | `string` | 先打印 `prompt`（无换行）并 flush；**EOF → `IOError`**；去掉行尾 `\n`/`\r\n` |
| `assert` | `assert(cond, msg?) -> nil` | `nil` | 失败 → `AssertionError`（致命） |
| `check` | `check(cond, msg?) -> bool` | `bool` | 失败 → stderr 警告 + 返回 `false`，**不中断**（A4） |
| `fail` | `fail(msg?) -> never` | — | 抛 `AssertionError`（致命） |

> **说明**：`sqrt` / `pow` 之外的 `math` 内置（如 `sin` / `log` / `exp`）**v1 不提供**（列 v1.1 backlog），故**不存在**这些函数的形参加宽问题。`div` 与 `/` 的区别见 [semantics.md](./semantics.md) §4.2；`minBy`/`maxBy` 因 §9 样例使用而保留（D-007 v1 IN 清单）。

**数值内置形参加宽（v1 冻结，规范性；与 §1「唯一隐式转换」、[semantics.md](./semantics.md) §4.5.7 同源；冻结前补钉）**：

- **加宽（int→float）的数值内置** —— `floor` / `ceil` / `round` / `sqrt` / `pow`：若实参为 `int`，先按 **§1 / §4.5.7 的全局唯一隐式转换 `int → float`** 加宽为 `float`，再执行；`float` 实参直接使用。故 `floor(3)` = `floor(3.0)` = `3`、`sqrt(4)` = `sqrt(4.0)` = `2.0`。加宽在 `|int| > 2^53` 时**可能不精确**（§4.5.7）。
- **同型（不加宽）的数值内置** —— `abs`：**参数与返回值同型**（`int → int`、`float → float`），**绝不加宽**：`abs(-3) == 3`（`int`）、`abs(-3.0) == 3.0`（`float`）。
- **一句话钉死差异**：`floor` / `ceil` / `round` / `sqrt` / `pow` **接受 `int` 并把其实参加宽为 `float`**（返回类型见上表：前三者 `int`，后二者 `float`）；**`abs` 接受什么类型就返回什么类型，从不加宽**。全文关于数值内置的实参处理**以此为准**，不再有"未声明是否加宽"的情形。

**内置边界补钉（v1，规范性；v1 补钉，与 [semantics.md](./semantics.md) §8.1 同源）**：

- **`min` / `max` / `minBy` / `maxBy` 收到空 `array`** → `ValueError`，消息 **`空数组没有极值（{func}）`**（`{func}` = 内置名）。
- **`randInt(lo, hi)` 且 `lo >= hi`** → `ValueError`，消息 **`区间非法：{lo} >= {hi}`**。
- **`floor` / `ceil` / `round` 的 `NaN` / `±Inf` / 超界** → **复用 `int(float)` 口径**（[semantics.md](./semantics.md) §4.5.7）：`NaN` → `ValueError`（`ValueMsg::Convert`，`src="float"`、`dst="int"`、`text="nan"`）；`±Inf` 或取整后结果超 `[i64::MIN, i64::MAX]` → `OverflowError`。实参 `int` 先加宽为 `float`。
- **`IndexError` 的 `idx` / `len`（钉死）**：`idx` = 触发越界的下标（**有实参者用实参原值**；`pop` 无实参 → 隐含末元素下标 `-1`）；`len` = **越界时**容器长度。故 **`pop([])` → `Index { idx: -1, len: 0 }`**（消息 `下标 -1 越界（长度 0）`）。
- **`insert` 不支持负索引（钉死）**：合法域恒为 `i ∈ [0, len]`（`len = len(xs)`）；`i < 0` 或 `i > len` → `Index { idx: i, len }`。**与 `removeAt` / `swap` 的支持负索引显式区分**。
- **`del(k, s)` 仅作用于数据字段（钉死；A5 数据面，与 `keys` / `has` / `len` 同集合）**：`k` 为方法字段（函数值字段）→ **`FieldError`**（现有 `LfzError::Field`，消息 `结构体没有字段 '{name}'`）；即 **`del(k, s)` 成功 ⟺ `has(k, s) == true`**。

### 10.8 错误实现建议（v0.5，B11 / B12 / B13）

- **返回类型**：`type R<T> = Result<T, Box<LfzError>>` —— `Err` 侧为 `Box`，使 `R<Value>` 保持寄存器友好（避免 `Result` 膨胀）。★ 已由 runtime-dev 确认。
- **错误构造冷路径**：所有 `LfzError` 构造器标 `#[cold]` + `#[inline(never)]`（`Box::new` 只在出错时执行），热路径只走 `Ok(v)`。
- **`CosmosAnswer` 变体**：携带 `span = Span { line: 1, col: 1 }`（恒为第 1 行；输出时**不显示源码行 / 插入符**，[semantics.md](./semantics.md) §8.2）。
- **VM 帧不每调用分配 `Rc<str>`**：`TraceFrame` 存 `func_id: u32`（§10.3），仅报错时按函数表反查名字。
- **`i64::MIN` 字面量（B12）**：词法 `INT` 记号**承载原文**（或 `u64`/`i128`），**不**在词法期直接转 i64。解析规则：
  1. 若 `INT` **紧邻**一元前缀 `-`，且其正值为 `2^63` → 合法，产出 `Int(i64::MIN)`；
  2. 否则按 i64 解析；数值超出 `[i64::MIN, i64::MAX]`，或十六 / 二 / 八进制超出 `u64` 范围 → **`SyntaxError`**（子消息 `整数字面量超出 i64 范围`）。
  - 例：`-9223372036854775808` **合法**；`9223372036854775808`（无负号）→ `SyntaxError`。
- **`s[k]` 与 `.k` 的键统一为 string**；`has` / `keys` 只认数据字段（A5）。
- **int→float 加宽（B13）**：见 [semantics.md](./semantics.md) §4.5.7（加宽可能不精确；混合比较按数学精确值）。
- **`ValueMsg` 变体（实现侧，v1 补钉）**：`Convert { src, dst, text }`、`BadFormatSpec { spec }`、**`EmptyExtremum { func: String }`**（消息 `空数组没有极值（{func}）`）、**`BadRange { lo: i64, hi: i64 }`**（消息 `区间非法：{lo} >= {hi}`）。后两者为本轮补钉**新增**，供 core-dev 在 `src/error.rs` 落地；均归类 `ValueError`（**不新增错误类**）。
- **`SyntaxMsg` 新增变体（实现侧，v1 补钉）**：**`UnterminatedBlockComment`**（**无字段**，消息 `块注释在此处未闭合（缺少 '*/'）`）——**未闭合块注释 `/*` 至 EOF**（[syntax.md](./syntax.md) §2.4、[semantics.md](./semantics.md) §8.1）；归 `LfzError::Syntax` → `SyntaxError`，`span` 指向 `/*` 中的 `/`（**不新增错误类**）。
- **`TypeMsg` 新增变体（实现侧，v1 补钉；P3.11）**：**`ImmutableRebind { name: String }`**（消息 `不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容`）——**对显式 `let` 绑定的重绑定**（[semantics.md](./semantics.md) §4.5.2 / §8.1）；归 `LfzError::Type` → **`TypeError`**（**不新增错误类**；`TypeMsg` 由 6 条增至 **7** 条），`span` = 赋值目标变量名首字符。

---

## 11. 对下游角色的硬性要求（规范，供 ai-dx / test-engineer / docs / app 执行）★ v0.3 新增；v0.4 修订（11.1/11.2/11.5，按扩展名）

### 11.1 AI 开发指南 / skill（ai-dx-engineer）

AI 指南与 `lfz-programming` skill（计划路径 `.opencode/skills/lfz-programming/`）**必须写死**以下内容：

1. **`#42` 首行规则（置顶）**：任何 **`.lfz` 文件**的**第一行必须恰好是 `#42`** 三个字符，其后紧跟一个行终止符；否则程序以
   `CosmosAnswerError: 你忘记了宇宙的答案` 终止。**触发看扩展名**：仅 `.lfz`（大小写不敏感）；非 `.lfz` 文件与 REPL / stdin / `lfz run -e` **都不需要** `#42`。（若给出片段示例，须注明"作为 `.lfz` 文件时首行须为 `#42`"。）
2. **错误类清单（逐条列出）**：`CosmosAnswerError` / `SyntaxError` / `NameError` / `TypeError` / `IndexError` / `FieldError` / `ZeroDivisionError` / `OverflowError` / `ValueError` / `IOError` / `AssertionError` / **`RecursionError`**（+ 基类 `LfzError`），每条附**触发条件**与**中文消息示例**。
3. **所有代码示例必须含 `#42` 首行**（含片段示例；若为片段则注明"文件首行须为 `#42`"）。
4. **禁止出现旧 `E-xxx` 错误码**（已取消）；错误一律以类名 + 中文消息呈现。
5. **`check` 非致命 / `assert` 致命（A4）**：`check` 失败 → **stderr** 一行 `check 失败：{msg}` + 返回 `false` + **继续**；**仅** `assert`/`fail` 抛 `AssertionError`（致命）。

### 11.2 test-runner 契约（tooling-dev 定稿 → test-engineer 使用）

- **T-R1**：所有被 runner 作为 LFZ 程序加载的 **`.lfz` 测试文件**必须是带 `#42` 首行的合法文件；缺失 → 该用例记为 **error**（退出码 2）。**非 `.lfz` 夹具按扩展名豁免**（不校验前导）。
- **T-R2**：对"缺前导 / 非法前导"这类**负例**，**不得**放入自动发现目录（否则会污染正常用例集）；须用**非自动发现的夹具**（建议 `tests/fixtures/`）并在清单（如 `tests/cases.json`）中声明 `"expect": {"error": "CosmosAnswerError"}`。
- **T-R3**：runner 的失败输出使用 [semantics.md](./semantics.md) §8.2 的位置信息；**仅 `assert`/`fail`** 抛 `AssertionError`，以此记 failure/error，退出码按 D-008（测试失败 = 1）。**`check` 失败不致命**（stderr 警告 + 返回 `false`），测试如需"软断言"可自行读取 `check` 的返回值（A4）。
- **T-R4**：runner 发现测试文件的规则须明确写入契约（建议 glob 如 `tests/**/*.lfz`，并排除 `tests/fixtures/**`）；**仅 `.lfz` 文件要求前导**（与 [syntax.md](./syntax.md) §2.2.0 一致）。

### 11.3 人类文档（docs-writer）

- 手册必须包含 **`#42` 仪式**（为什么、唯一合法形式、REPL/stdin/`-e` 例外）与 **Python 风格错误模型**（类清单 + 3 个输出示例 + `--json`）。
- 手册中所有 LFZ 代码块**必须含 `#42` 首行**。

### 11.4 应用（app-dev）

- `app/` 下全部 `.lfz` 源文件**必须含 `#42` 首行**；片段复用（[syntax.md](./syntax.md) §9.3）作为文件时亦然。

### 11.5 spec 三件套（language-architect 本人）——已完成

- 本草案已按此拆分冻结入 `docs/spec/`（**当前实现**）：
  - [syntax.md](./syntax.md)：§2–§7 + §9 + §12 + §14 + §15 + §0/§1。
  - [semantics.md](./semantics.md)：§3.6–§3.7、§4.2、**§4.5 求值语义规范**（A1/A2/A4/A5/A6 + B1–B7/B13）、§8。
  - interface-contract.md（本文件）：§8.1 错误类 + §10 的 `Span`（统一 `line/col`）/ `VmFrame`/`TraceFrame` / `LfzError`（含 `RecursionError`）/ **per-scope `ScopeDebug`** / loader 双入口 + **`ext(path)` 扩展名判定** + **`Loaded{text,line_base}` / `line_base` 行号语义** + **§10.7 内置函数表（data-last）** + §11 下游要求 + §13 旧码映射。

---

## 13. 附录：旧错误码 → 新错误类映射（仅追溯用，**不进入用户输出**）

| v0.2 错误码 | v0.3 错误类 | 备注 |
|---|---|---|
| `E-LEX-001` 非法字符 | `SyntaxError` | 含 `#` 位置非法 |
| `E-LEX-002` 字符串未闭合 | `SyntaxError` | |
| `E-SYN-001` 意外 token | `SyntaxError` | |
| `E-SYN-002` 表达式行尾不完整 | `SyntaxError` | |
| `E-SYN-003` 单个 `;` | `SyntaxError` | |
| `E-SYN-005` 同行两语句 | `SyntaxError` | |
| `E-SYN-006` 赋值目标非法 | `SyntaxError` | |
| `E-PIPE-001` 管道右侧非函数 | `TypeError` | |
| `E-PIPE-002` 管道多个 `_` | `SyntaxError` | 解析期 |
| `E-PIPE-003` `_` 位置非法 | `SyntaxError` | 解析期 |
| `E-FMT-001` 非法说明符 | `ValueError` | |
| `E-FMT-002` 说明符与值不符 | `TypeError` | |
| （v1）越界 | `IndexError` | |
| （v1）缺字段 | `FieldError` | |
| （v1）名字未定义 | `NameError` | |
| （v1）类型不符 | `TypeError` | |
| （v1）除以零 | `ZeroDivisionError` | |
| （v1）i64 溢出 | `OverflowError` | |
| （v1）转换失败 | `ValueError` | |
| （v1）IO | `IOError` | |
| （v1）assert | `AssertionError` | `check` 失败**不**在此列（A4；非致命） |
| —（新增） | `CosmosAnswerError` | 缺 `#42` 前导；消息固定「你忘记了宇宙的答案」 |
| —（v0.5 新增） | `RecursionError` | 深递归 / 超深结构超限（原 v1 `E-RT-001`） |

> `E-SYN-004` 在 v0.2 即保留未用；v0.3 随编号制一并取消。原 v1 草案「环比较 → `E-RT-002`」被 **A6** 取代（`==` 环安全，不再报错）。

---

*（本文件为 LFZ v1 接口契约，冻结于 2026-09-23；与 [syntax.md](./syntax.md)、[semantics.md](./semantics.md) 同族。任何语法/语义/契约变更须先写 ADR，再由 language-architect 改 `docs/spec`。）*
