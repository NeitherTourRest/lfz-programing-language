# LFZ v1 接口契约（interface-contract.md）

> 本文件是 LFZ v1 的唯一事实源（冻结于 2026-09-23）。变更须先 ADR，后改文档。
> 作者: language-architect ｜ 状态: **spec v1 = FROZEN** ｜ 冻结 ADR: `.opencode/team/DECISIONS.md` D-016「spec v1 冻结」
> 冻结基线: `.opencode/team/DRAFT-LFZ-v0.5.md`（含 3 处冻结前补钉）
> 同族文件: [syntax.md](./syntax.md) ｜ [semantics.md](./semantics.md)

本文件是 **core-dev / runtime-dev 必须按此实现** 的接口契约：错误类清单（§8.1）、Rust 实现契约（§10：loader 双入口 / `ext(path)` / 唯一 `Span` / `VmFrame` / `TraceFrame` / `LfzError` 变体 / per-scope `ScopeDebug` / §10.7 内置函数表 data-last）、下游角色硬性要求（§11）、旧错误码映射（§13）。**形式**见 [syntax.md](./syntax.md)；**意义**见 [semantics.md](./semantics.md)。

**本文件内容映射**：§8.1（错误类清单）、§10（§10.1–§10.9）、§11（§11.1–§11.5）、§13（附录）。
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
| `OverflowError` | `LfzError::Overflow { span, msg: OverflowMsg }` | 运行 | `整数溢出：结果超出 i64 范围` / `容量溢出：所需容量超出可分配上限`（容器/字符串构造，v1.1 补钉） |
| `ValueError` | `LfzError::Value { msg: ValueMsg, span }` | 运行 | `无法把 {src} 转换为 {dst}（'{text}'）` / `格式说明符非法：'{spec}'` / **`空数组没有极值（{func}）`** / **`区间非法：{lo} >= {hi}`** / **`ord 的参数必须是单个字符（Unicode 标量数 {n}）`** / **`chr 的参数不是合法的 Unicode 码点：{n}`** / **`填充串不能为空`**（后三条 v1.1 补钉） |
| `IOError` | `LfzError::Io { msg: String, span: Option<Span> }` | 运行 | `输入结束（EOF）` / `无法读取：{path}` / **`无法读取：{path}（不是合法的 UTF-8 编码）`** / **`无法写入：{path}`** / **`无法追加：{path}`**（后三条 v1.1 补钉，见 §10.7「文件 IO」） |
| `AssertionError` | `LfzError::Assert { msg: String, span }` | 运行 | `断言失败：{msg}` / `{msg}` |
| `RecursionError` | `LfzError::Recursion { depth, limit, span }` | 运行 | `递归深度超限（超过 10000 层）` |

- **共 12 个具体错误类 + 1 基类 `LfzError`**；基类**不直接抛出**。
- **`--json` 的 `error` 取值集合（12 个）**：`CosmosAnswerError` / `SyntaxError` / `NameError` / `TypeError` / `IndexError` / `FieldError` / `ZeroDivisionError` / `OverflowError` / `ValueError` / `IOError` / `AssertionError` / **`RecursionError`**。
- **运行期错误 = 10 类**（除 `CosmosAnswerError`、`SyntaxError` 两个加载/解析期类之外的其余 10 类），在输出中**带 `Traceback` 头**（**零帧情形除外**：帧栈为空时不带，如文件不存在的 `IOError`，见 [semantics.md](./semantics.md) §8.2「零帧情形」，v1.1 补钉）；加载/解析期两类**不带**。见 [semantics.md](./semantics.md) §8.2。
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
| `range` | `range(lo, hi) -> array` | `array[int]` | `[lo, lo+1, …, hi-1]`（**半开区间** `[lo, hi)`）；`hi <= lo` → 空数组（v1.1 补钉；**3 参 `range(lo,hi,step)` = OUT**） |
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
| `contains` | `contains(v, xs) -> bool` | `bool` | `xs` 中是否存在与 `v` **相等**的元素；相等用 `==`（[semantics.md](./semantics.md) §4.5.9，深结构、环安全）。**仅 `array`**（非 array → `TypeError`）；struct 键判定用 `has(k, s)`。**O(n)** 线性扫描（v1.1 补钉；**须标注**，**不得**放进循环对长数组反复调用） |

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
| `repeat` | `repeat(n, s) -> string` | `string` | `n <= 0` → 空串；结果所需容量（`n * len(s)` 字节）超出运行时可分配上限 → `OverflowError`（消息 `容量溢出：所需容量超出可分配上限`，v1.1 补钉）；**任何 `n` / `s` 均不得使进程 panic** |
| `startsWith` | `startsWith(prefix, s) -> bool` | `bool` | |
| `endsWith` | `endsWith(suffix, s) -> bool` | `bool` | `s` 是否以 `suffix` 结尾；`suffix == ""` → `true`。**O(m)**（v1.1 补钉） |
| `indexOf` | `indexOf(sub, s) -> int` | `int` | `sub` 首次出现的**标量下标**（与 `len`/`substring` 同口径）；未找到 → `-1`；`sub == ""` → `0`。**最坏 O(n·m)**（n=主串标量数、m=子串标量数；**须标注**，不得宣称 O(n)）（v1.1 补钉） |
| `padEnd` | `padEnd(width, fill, s) -> string` | `string` | 右侧补 `fill` 至**标量**总宽 `width`；`len(s) >= width` → 原串；`fill` 重复并**按标量截断**（v1.1 补钉） |
| `padStart` | `padStart(width, fill, s) -> string` | `string` | 左侧补 `fill`（规则同 `padEnd`）（v1.1 补钉） |
| `substring` | `substring(from, to, s) -> string` | `string` | `s` 在 `[from, to)` 的子串（**标量半开**）；下标**夹取**到 `[0, len(s)]`；`from >= to` → 空串（**与 `slice` 同口径**）（v1.1 补钉） |

**数学 / 随机 / 转换 / IO / 断言**

| 内置 | 签名 | 返回 | 说明 |
|---|---|---|---|
| `abs` | `abs(x) -> number` | **同型** | **参数与返回同型、绝不加宽**（`int→int`、`float→float`）；`int` 的 `i64::MIN` → `OverflowError` |
| `floor` | `floor(f) -> int` | `int` | 向下取整 |
| `ceil` | `ceil(f) -> int` | `int` | 向上取整 |
| `round` | `round(f) -> int` | `int` | **四舍六入五成双**（Python banker's rounding） |
| `sqrt` | `sqrt(f) -> float` | `float` | `f < 0` → `NaN` |
| `pow` | `pow(a, b) -> float` | `float` | 数值幂；溢出 → `±Inf` |
| `sin` | `sin(f) -> float` | `float` | 正弦；`int` 实参加宽；IEEE（`sin(±Inf)`/`sin(NaN)` → `NaN`）。**O(1)**（v1.1 补钉） |
| `cos` | `cos(f) -> float` | `float` | 余弦；口径同 `sin`。**O(1)**（v1.1 补钉） |
| `log` | `log(f) -> float` | `float` | 自然对数；`int` 实参加宽；`log(x<0)` → `NaN`、**`log(0)` → `-Inf`**（超越函数极限，**非** `ZeroDivisionError`）。**O(1)**（v1.1 补钉） |
| `exp` | `exp(f) -> float` | `float` | `e^f`；`int` 实参加宽；溢出 → `+Inf`。**O(1)**（v1.1 补钉） |
| `div` | `div(a, b) -> int` | `int` | 两参须 `int`；**向下取整**；`b == 0` → `ZeroDivisionError` |
| `rand` | `rand() -> float` | `float` | `[0.0, 1.0)` |
| `randInt` | `randInt(lo, hi) -> int` | `int` | `[lo, hi)`；`lo >= hi` → `ValueError` |
| `seed` | `seed(n) -> nil` | `nil` | 固定随机序列；未调用时用时间种子（唯一非确定源） |
| `str` | `str(x) -> string` | `string` | 显示形式（[semantics.md](./semantics.md) §3.7） |
| `int` | `int(x) -> int` | `int` | `string`/`float`/`bool` → `int`；`float` **向零截断**；**`NaN` → `ValueError`；`±Inf` → `OverflowError`；有限浮点截断后超 i64 → `OverflowError`**（[semantics.md](./semantics.md) §4.5.7）；非法串 → `ValueError` |
| `float` | `float(x) -> float` | `float` | `int`/`string`/`bool` → `float`；非法串 → `ValueError`；支持 `"inf"`/`"-inf"`/`"nan"` |
| `ord` | `ord(c) -> int` | `int` | `c` 须为**恰好 1 个 Unicode 标量**的 `string` → 其码点（`0..=0x10FFFF`）；非 `string` → `TypeError`；标量数 ≠ 1 → `ValueError`（消息 `ord 的参数必须是单个字符（Unicode 标量数 {n}）`）。**成功 O(1)；非法字符串为完整报告标量数 n，错误路径 O(n)**（非 string 的 TypeError 为 O(1)；v1.1 补钉） |
| `chr` | `chr(n) -> string` | `string` | `n` 须 `int` 且为**合法 Unicode 码点**（`0 <= n <= 0x10FFFF` 且非代理区 `0xD800..=0xDFFF`）→ 该字符；非 `int` → `TypeError`；非法码点 → `ValueError`（消息 `chr 的参数不是合法的 Unicode 码点：{n}`）。**O(1)**（v1.1 补钉） |
| `type` | `type(x) -> string` | `string` | `"int"`/`"float"`/`"string"`/`"bool"`/`"nil"`/`"array"`/`"struct"`/`"function"` |
| `print` | `print(...) -> nil` | `nil` | 各参数按显示形式拼接、**空格连接** + 末尾 `\n`（stdout） |
| `eprint` | `eprint(...) -> nil` | `nil` | 同 `print`，但写 **stderr** |
| `input` | `input(prompt?) -> string` | `string` | 先打印 `prompt`（无换行）并 flush；**EOF → `IOError`**；去掉行尾 `\n`/`\r\n` |
| `assert` | `assert(cond, msg?) -> nil` | `nil` | 失败 → `AssertionError`（致命） |
| `check` | `check(cond, msg?) -> bool` | `bool` | 失败 → stderr 警告 + 返回 `false`，**不中断**（A4） |
| `fail` | `fail(msg?) -> never` | — | 抛 `AssertionError`（致命） |

> **说明**：**v1.1 补钉已提供** `sin` / `cos` / `log` / `exp`（加宽口径见下「数值内置形参加宽」）；其余 `math` 内置（如 `tan` / `atan2` / `log2` / `log10`）仍属 v1.1 backlog。`div` 与 `/` 的区别见 [semantics.md](./semantics.md) §4.2；`minBy`/`maxBy` 因 §9 样例使用而保留（D-007 v1 IN 清单）。

**数值内置形参加宽（v1 冻结，规范性；与 §1「唯一隐式转换」、[semantics.md](./semantics.md) §4.5.7 同源；冻结前补钉）**：

- **加宽（int→float）的数值内置** —— `floor` / `ceil` / `round` / `sqrt` / `pow` / **`sin` / `cos` / `log` / `exp`（v1.1 补钉新增）**：若实参为 `int`，先按 **§1 / §4.5.7 的全局唯一隐式转换 `int → float`** 加宽为 `float`，再执行；`float` 实参直接使用。故 `floor(3)` = `floor(3.0)` = `3`、`sqrt(4)` = `sqrt(4.0)` = `2.0`。加宽在 `|int| > 2^53` 时**可能不精确**（§4.5.7）。
- **同型（不加宽）的数值内置** —— `abs`：**参数与返回值同型**（`int → int`、`float → float`），**绝不加宽**：`abs(-3) == 3`（`int`）、`abs(-3.0) == 3.0`（`float`）。
- **一句话钉死差异**：`floor` / `ceil` / `round` / `sqrt` / `pow` / `sin` / `cos` / `log` / `exp` **接受 `int` 并把其实参加宽为 `float`**（返回类型见上表：`floor`/`ceil`/`round` 为 `int`，其余为 `float`）；**`abs` 接受什么类型就返回什么类型，从不加宽**。全文关于数值内置的实参处理**以此为准**，不再有"未声明是否加宽"的情形。

**内置边界补钉（v1，规范性；v1 补钉，与 [semantics.md](./semantics.md) §8.1 同源）**：

- **`min` / `max` / `minBy` / `maxBy` 收到空 `array`** → `ValueError`，消息 **`空数组没有极值（{func}）`**（`{func}` = 内置名）。
- **`randInt(lo, hi)` 且 `lo >= hi`** → `ValueError`，消息 **`区间非法：{lo} >= {hi}`**。
- **`floor` / `ceil` / `round` 的 `NaN` / `±Inf` / 超界** → **复用 `int(float)` 口径**（[semantics.md](./semantics.md) §4.5.7）：`NaN` → `ValueError`（`ValueMsg::Convert`，`src="float"`、`dst="int"`、`text="nan"`）；`±Inf` 或取整后结果超 `[i64::MIN, i64::MAX]` → `OverflowError`。实参 `int` 先加宽为 `float`。
- **`IndexError` 的 `idx` / `len`（钉死）**：`idx` = 触发越界的下标（**有实参者用实参原值**；`pop` 无实参 → 隐含末元素下标 `-1`）；`len` = **越界时**容器长度。故 **`pop([])` → `Index { idx: -1, len: 0 }`**（消息 `下标 -1 越界（长度 0）`）。
- **`insert` 不支持负索引（钉死）**：合法域恒为 `i ∈ [0, len]`（`len = len(xs)`）；`i < 0` 或 `i > len` → `Index { idx: i, len }`。**与 `removeAt` / `swap` 的支持负索引显式区分**。
- **`del(k, s)` 仅作用于数据字段（钉死；A5 数据面，与 `keys` / `has` / `len` 同集合）**：`k` 为方法字段（函数值字段）→ **`FieldError`**（现有 `LfzError::Field`，消息 `结构体没有字段 '{name}'`）；即 **`del(k, s)` 成功 ⟺ `has(k, s) == true`**。

**文件 IO（v1.1 补钉，规范性）**

> 新增 3 个内置；**data-last**：被操作的 `string` 数据（`contents`）在末参。

| 内置 | 签名 | 返回 | 说明 |
|---|---|---|---|
| `readFile` | `readFile(path) -> string` | `string` | 以 **UTF-8** 读入整个文件；`path` 须 `string` |
| `writeFile` | `writeFile(path, contents) -> nil` | `nil` | 以 **UTF-8** **覆盖写**（创建 / 截断）；`path`、`contents` 须 `string` |
| `appendFile` | `appendFile(path, contents) -> nil` | `nil` | 以 **UTF-8** **追加**（不存在则创建）；两参须 `string` |

> **语义（规范性）**：
> - **路径**：`path` **原样**交给操作系统；**相对路径**相对**解释器进程的当前工作目录**解析；**不**规范化、**不**沙箱、**不**展开 `~`、**不**自动创建父目录。
> - **编码**：UTF-8。`readFile` 读到**非法 UTF-8** → `IOError: 无法读取：{path}（不是合法的 UTF-8 编码）`。
> - **错误（均 `IOError`，运行期，帧栈非空时带 `Traceback`）**：`readFile` 失败 → `无法读取：{path}`；`writeFile` 失败 → `无法写入：{path}`；`appendFile` 失败 → `无法追加：{path}`。**二进制 / 目录 IO = OUT**。
> - **返回值**：读 → `string`；写 / 追加 → `nil`。
> - **与 `input()` 同属"外部 IO"**：结果依赖运行环境（文件系统 / 工作目录），**不在** [semantics.md](./semantics.md) §4.5 确定性「无魔法」纪律讨论内（该纪律约束**求值顺序 / 类型 / 无隐式转换**，外部 IO 从来排除）；`seed` 的"唯一非确定源"表述**不变**（文件 IO 是**环境依赖**，非随机性）。
> - **不得引入隐式转换**：`path` / `contents` 必须 `string`；`writeFile(p, 42)` → `TypeError`（**不**自动 `str`），需写数字请先 `str(42)`。
> - **复杂度**：O(文件大小) 时间 + O(文件大小) 内存；**无隐藏 O(n²)**。把多次读取结果 `s = s + chunk` 累积为 **O(n²)**；循环 `push` 同样会克隆数组前缀，亦为 **O(n²)**。应预分配数组后用下标写入，或用 `map` 一次构造数组，再 `join("", arr)`（时间 O(片段数 + 总输出大小)；沿用 ADR [2026-09-27 23:40]）。
> - **`span`** = 调用节点（内置名）首字符；属**运行期 10 类**之一。

**v1.1 补钉内置的边界与复杂度（规范性）**

- **`range` 参数个数**：仅接受 **1 或 2** 个参数（`min_args = 1, max_args = 2`）；`m < 1` → `函数 range 期待 1 个参数，得到 {m}`；`m > 2` → `函数 range 期待 2 个参数，得到 {m}`；非 `int` 实参 → `TypeError`（`运算符 'range' 不支持 {t} 与 int`）。两形均**物化普通 `array[int]`**（沿用 §4.5.4 array 快照）；构造结果所需容量超上限 → `OverflowError: 容量溢出：所需容量超出可分配上限`；**任何 `n`/`lo`/`hi` 均不得使进程 panic**。**复杂度** `O(n)` / `O(hi−lo)` 时间与内存，与既有同阶。
- **字符串方法族**（`indexOf` / `endsWith` / `padEnd` / `padStart` / `substring`）：全 **data-last**、**返回新值**（string 不可变 §4.5.11）。`padEnd`/`padStart` 的 `fill == ""` → `ValueError: 填充串不能为空`；`width`/`from`/`to` 非 `int` → `TypeError`；`substring` 下标**夹取**（负数 / 越界不作 `IndexError`，与 `slice` 同口径）。**复杂度**：`indexOf` **最坏 `O(n·m)`**（n=主串标量数、m=子串标量数；**必须标注**，不得宣称 `O(n)`；实现用 `str::find` 可期望 `O(n+m)`，但**规范按保守最坏 `O(n·m)` 标注**）；`endsWith` `O(m)`；`pad*` `O(n+m+输出宽度)`（n、m 分别为 s、fill 的标量数，包含计数成本；不能仅按 width 标注）；`substring` 非空结果 `O(min(to, len(s)))`（此处 to 已把负数夹至 0；UTF-8 须扫描到右端点或串尾，不能仅按输出子串长度标注）。`from`/`to` 均夹取到 `[0, len(s)]`；空区间可直接返回空串，不要求先遍历全串计数。**不引入隐藏 `O(n²)`**。
- **`ord` / `chr`**：`ord` **成功 O(1)**；非法字符串须完整报告 Unicode 标量数 n，故**错误路径 O(n)**（窥前 2 个标量只能判错，不能取得消息中的 n）；非字符串类型错误 O(1)。`chr` **O(1)**。`ord` 非 `string` → `TypeError`、标量数 ≠ 1 → `ValueError`；`chr` 非 `int` → `TypeError`、非法码点 → `ValueError`。
- **`sin` / `cos` / `log` / `exp`**：**O(1)**；`int` 实参加宽；IEEE 口径（`log(0)` → `-Inf`、`log(x<0)` → `NaN`、`exp` 溢出 → `+Inf`、`sin`/`cos` 遇 `±Inf`/`NaN` → `NaN`），见 [semantics.md](./semantics.md) §4.5.6。
- **`contains`**：**O(n)** 线性扫描（**必须标注**；元素为容器时每次 `==` 可 > O(1)，深结构比较继承 §4.5.9 环安全）；`xs` 非 `array` → `TypeError`；**不得**放进 N 次循环对长数组反复调用（否则 `O(N·M)`）；struct 键判定用 `has(k, s)`，二者不重叠。

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
- **`OverflowMsg` 新增变体（实现侧，v1.1 补钉）**：**`Capacity`**（**无字段**，消息 `容量溢出：所需容量超出可分配上限`）——**容器 / 字符串构造所需容量超出可分配上限**（`range` 超大 `n` / `repeat` 结果过长，见 §10.7）；归 `LfzError::Overflow` → **`OverflowError`**（**不新增错误类**；`OverflowMsg` 由 1 条增至 **2** 条，供 core-dev 在 `src/error.rs` 落地）。
- **`ValueMsg` 变体（实现侧，v1 补钉）**：`Convert { src, dst, text }`、`BadFormatSpec { spec }`、**`EmptyExtremum { func: String }`**（消息 `空数组没有极值（{func}）`）、**`BadRange { lo: i64, hi: i64 }`**（消息 `区间非法：{lo} >= {hi}`）。后两者为本轮补钉**新增**，供 core-dev 在 `src/error.rs` 落地；均归类 `ValueError`（**不新增错误类**）。
- **`ValueMsg` 新增变体（实现侧，v1.1 补钉）**：**`NotSingleScalar { n: usize }`**（消息 `ord 的参数必须是单个字符（Unicode 标量数 {n}）`）——`ord` 实参 Unicode 标量数 ≠ 1；**`BadCodepoint { n: i64 }`**（消息 `chr 的参数不是合法的 Unicode 码点：{n}`）——`chr` 实参越出 `0..=0x10FFFF` 或落在代理区；**`EmptyFill`**（**无字段**，消息 `填充串不能为空`）——`padEnd`/`padStart` 的 `fill` 为空串（§10.7）。三者均归类 `ValueError` → **`ValueError`**（**不新增错误类**；`ValueMsg` 由 4 条增至 **7** 条），供 core-dev 在 `src/error.rs` 落地。
- **`SyntaxMsg` 新增变体（实现侧，v1 补钉）**：**`UnterminatedBlockComment`**（**无字段**，消息 `块注释在此处未闭合（缺少 '*/'）`）——**未闭合块注释 `/*` 至 EOF**（[syntax.md](./syntax.md) §2.4、[semantics.md](./semantics.md) §8.1）；归 `LfzError::Syntax` → `SyntaxError`，`span` 指向 `/*` 中的 `/`（**不新增错误类**）。
- **`SyntaxMsg` 新增变体（实现侧，v1.1 补钉）**：**`NestingTooDeep`**（**无字段**，消息逐字符 `嵌套深度超限（超过 1000 层）`）——**解析嵌套深度超过 `PARSE_DEPTH_LIMIT = 1000`**（[syntax.md](./syntax.md) §3.8、[semantics.md](./semantics.md) §8.1）；归 `LfzError::Syntax` → **`SyntaxError`**（**不新增错误类**；`SyntaxMsg` 由 17 条增至 **18** 条），`span` = **第 1001 层嵌套的开启记号首字符**。
- **`SyntaxMsg` 新增变体（实现侧，v1.1 补钉）**：**`ExprTooDeep`**（**无字段**，消息逐字符 `表达式嵌套过深（超过 10000 层）`）——**AST 节点深度超过 `AST_DEPTH_LIMIT = 10000`**（[syntax.md](./syntax.md) §3.9、[semantics.md](./semantics.md) §8.1）；归 `LfzError::Syntax` → **`SyntaxError`**（**不新增错误类**；`SyntaxMsg` 由 **18** 条增至 **19** 条），`span` = **首次使 AST 深度超过上限的节点首字符**。
- **`TypeMsg` 新增变体（实现侧，v1 补钉；P3.11）**：**`ImmutableRebind { name: String }`**（消息 `不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容`）——**对显式 `let` 绑定的重绑定**（[semantics.md](./semantics.md) §4.5.2 / §8.1）；归 `LfzError::Type` → **`TypeError`**（**不新增错误类**；`TypeMsg` 由 6 条增至 **7** 条），`span` = 赋值目标变量名首字符。

### 10.9 解析与 AST 消费的栈安全（v1.1 补钉，规范性）

> 目标：兑现「**任何合法输入都不得使解释器因栈耗尽而崩溃 / abort**」（实测反例：`(`×N 深嵌套 → `thread 'main' has overflowed its stack`，exit `-1073741571`）。

- **要求 R-S1（解析嵌套）**：parser 必须以 **`PARSE_DEPTH_LIMIT = 1000`** 限制解析嵌套深度，超限抛 `SyntaxError`（新变体 `SyntaxMsg::NestingTooDeep`；定义见 [syntax.md](./syntax.md) §3.8，消息见 [semantics.md](./semantics.md) §8.1）。
- **要求 R-S2（栈容量）**：解析须在**栈容量 ≥ 64 MiB** 的线程上运行；**推荐**把整条 `load → lex → parse → eval`（含 `Program` 的**析构**）流水线放到既有的 **256 MiB** 大栈线程（`EVAL_STACK_SIZE`，`src/evaluator.rs`）上——同一线程更省事，且使深 AST 的析构也在大栈上完成。
- **要求 R-S3（AST 深度）**：parser 必须在**解析期**（构建中自底向上累加，或紧接解析后以**显式栈迭代遍历**）检查 **AST 节点深度**，超过 **`AST_DEPTH_LIMIT = 10000`** 即抛 `SyntaxError`（新变体 `SyntaxMsg::ExprTooDeep`，消息 `表达式嵌套过深（超过 10000 层）`；定义见 [syntax.md](./syntax.md) §3.9，消息见 [semantics.md](./semantics.md) §8.1），**求值之前**拒绝。**该检查本身不得以 AST 深度递归**。R-S3 收口 R-S1 **不覆盖**的**左结合长链**（迭代解析、深左偏 AST），使求值 / 析构递归深度均有界。**AST 迭代析构非必需**（上限已使析构深度 ≤ 10000）；若日后放宽该上限，再以迭代析构加固。
- **量化依据（实测 2026-09-27，debug 构建；256 MiB 大栈线程）**：
  - 二进制主线程栈 `SizeOfStackReserve = **1 MiB**`（PE 实测；release 同为 1 MiB）。
  - **每层解析嵌套最坏 ≈ 18.3 KiB**（struct 字面量 `{"a":`）：分组 `(` ≈ 16.5 KiB、插值 `"${` ≈ 17.7 KiB、数组 `[` ≈ 17.1 KiB、lambda `fn(){` ≈ 16.8 KiB（均 debug）；release 最坏 ≈ 5.1 KiB。
  - 故 1 MiB 主线程在 **约 56 层**即溢出（实测 `(` 第 62 层 abort、`{"a":` 第 56 层 abort）。
  - `1000 × 18.3 KiB ≈ 17.9 MiB` → 在 **64 MiB** 上有 **≥ 3.6×** 余量、在 **256 MiB** 上有 **≥ 14×** 余量（release 余量更大）。
  - **每层 AST 求值递归 ≈ 2.93 KiB**（实测：纯 `1+1+…` 链在 **91650 项** 正常、**91700 项** abort ⇒ `256 MiB ÷ 91675 ≈ 2.93 KiB`）——远小于解析帧，故 R-S3 的 **10000** 上限 ⇒ `10000 × 2.93 KiB ≈ 28.6 MiB`，在 256 MiB 上 **≈ 9.2×** 余量（release 更小 ⇒ 余量更大）。
  - **每层 AST 析构（`Drop`）≈ 0.27 KiB**（实测：深链置于未执行分支，`800000` 项析构正常、`1600000` 项 abort ⇒ `256 MiB ÷ ≈10⁶ ≈ 0.27 KiB`）。R-S3 使析构深度 ≤ 10000 ⇒ `≈ 2.7 MiB`，**≈ 95×** 余量 ∴ **AST 迭代析构非必需**。
  - **函数调用帧 ≈ 14.2 KiB**（由组合实测反推：`9999` 层函数递归 + `42000` 项深表达式仍正常、`45000` 项 abort ⇒ `256 MiB − 42000 × 2.93 KiB ≈ 138.7 MiB ÷ 10⁴ ≈ 14.2 KiB`）。**组合最坏**（`RECURSION_LIMIT` 与 `AST_DEPTH_LIMIT` 同时取满）= `10000 × 14.2 KiB + 10000 × 2.93 KiB ≈ 167 MiB ≤ 256 MiB`（**≈ 1.5×** 余量），**不溢出**。
- **第 4 类残余（左结合长链 / 深左偏 AST）——已裁定并收口（v1.1）**：左结合链（`1+1+…`、`a[0][0]…`、`f(a)(b)…`、`a.b.c…`、`x |> f |> g…`）由**迭代循环**解析，**不受 R-S1 约束**，但会产出**深左偏 AST**；其实测（256 MiB 同栈、debug）**T = 91700 项即求值递归栈溢出**（`print` 前即 abort，非仅析构；T = 91650 仍正常）。收口方式 = **R-S3**：解析期以 `AST_DEPTH_LIMIT = 10000` 拒绝 AST 深度超限（`SyntaxError: 表达式嵌套过深（超过 10000 层）`，[syntax.md](./syntax.md) §3.9），从而使**求值 / 析构递归深度均 ≤ 10000**（上量化），**不再**依赖「同栈大栈」兜底。**两度量的口径不得合并**：R-S1 计**解析嵌套**（递归下降同时活跃层，保护 parser 栈），R-S3 计 **AST 节点深度**（保护求值 / 析构栈）；二者**正交不可互推**（`(((1)))` 前者大后者小；`1+1+…` 前者小后者大），故**分别设限**、各自取值。

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

## 16. T12 Agent 数据脚本接口（2026-10-25 冻结；实现状态见 PROJECT_STATE）

本节新增三内置，名称总数 69→72；不新增 AST、关键字或错误类。沿用词法变量优先、允许遮蔽内置的既有语义；三个新名称不是关键字或不可覆盖的保留名。只有 `argv` 本轮提供一等内置值，旧69内置及两JSON内置不因本节获得独立取值能力。

### 16.1 参数上下文与求值入口

- `argv() -> array[string]`，严格零参数；不含解释器名、脚本名、`-e` 源码和分隔符 `--`。无脚本参数返回 `[]`。
- 各次求值独立持有只读参数快照，不读全局 `std::env::args`、不使用进程级可变 argv。每次调用返回全新数组；修改返回数组不得影响下次调用。
- argv 是本轮新增的一等内置函数值，别名、数组/struct 存储、闭包和高阶调用均按函数值调用规则工作；不得只匹配源码中的直接 `argv()` 文本。既有内置的直接调用/管道派发不等于其本来已是一等值。
- 增加 `evaluator::eval_module_traced_with_args(program: &Program, args: &[String]) -> TracedRun` 和 `eval_module_traced_on_thread_with_args(program: &Program, args: &[String]) -> TracedRun`。前者通过既有大栈入口，后者供已在大栈的 CLI 使用。旧 `eval_module` / `eval_module_traced` / `_on_thread` / `run` 签名不变，默认为空参数。
- 内置派发实现可增加上下文回调，但必须保持既有 `builtins::call` / `call_with` 入口可用；无求值上下文的直接内置调用 argv 返回空数组。参数个数错误沿用 `函数 argv 期待 0 个参数，得到 {m}`。
- 时间/额外空间 O(参数个数+UTF-8 总字节数) 为保守上界；可共享不可变字符串，但容器必须全新。

### 16.2 严格 JSON 值映射

| 内置 | 参数/返回 | 行为 |
| -- | -- | -- |
| `jsonParse(text)` | string → 任意 JSON 可表示值 | 单个完整 JSON 文本；空白允许，尾随非空白拒绝 |
| `jsonStringify(value)` | 任意值 → string | 紧凑、确定性 JSON；无尾换行 |

- JSON `null` ↔ LFZ `nil`；bool/string/array/object 分别映射 bool/string/array/匿名 struct。对象可含任意字符串键，保留原键和值；不推断 LFZ 方法、模板或插值。
- 严格 JSON 文法：只允许空白 U+0020、TAB、LF、CR；禁止注释、尾逗号、单引号、裸键、前导 `+`、非法前导零、NaN/Infinity、裸控制字符和 BOM。解析内容不是 LFZ 源码，`$` 和 `${...}` 都是普通字符串字符。
- JSON 字符串允许 `\"`、`\\`、`\/`、`\b`、`\f`、`\n`、`\r`、`\t`、`\uXXXX`，必须正确组合 UTF-16 代理对；孤立高/低代理拒绝，原始 Unicode 标量包括非 BMP、NUL 转义均可往返。
- 不含小数点/指数的数字词法按 i64 精确转换；超范围拒绝，绝不降为 float。`-0` 为 int 0。含 `.` / `e` / `E` 的词法转换为 f64；结果必须有限，拒溢出为 Inf；有限舍入与下溢至零按 IEEE-754，保留浮点负零。因此这类数不保证十进制原文精度，但 int 不丢精度。
- 同一对象中**解码后相同**键重复即拒绝，包括 `"a"` 与 `"\u0061"`。不同对象同名键合法。
- 解析结果 array/struct 采用一次构造。重复键检查最坏 O(k log k) 或更优；不得反复调用线性 `set` 形成 O(k²)。单遍游标消费输入，不对每个字符反复从头扫描 UTF-8。

### 16.3 序列化、边界与精确错误

- 支持 nil/bool/int/有限 float/string/array/struct 实例。整数十进制精确输出。浮点选最短可往返表示；若该表示无 `.`/`e`/`E`，补 `.0`，并保留负零（`-0.0`），使重新解析仍为 float。
- string 转义双引号/反斜杠与 U+0000–U+001F（五种标准短转义优先，其余用小写十六进制 `\u00xx`）；不转义 `/`、`$` 或其它 Unicode。`\$` 不是合法 JSON 转义，不得产生。
- struct 使用 A5 **数据面**：忽略函数值方法字段，其余字段按 UTF-8 字节序递增编码；所有数据字段递归检查。因此 `{ "x": fn() { 1 } }` 编为 `{}`，而 `[fn() { 1 }]` 不可编码；模板不是实例，拒绝。不会执行方法或用户回调。
- 共享但无环的引用在各位置独立编码；只拒当前递归路径上的环，不能用全局 visited 错拒共享子树。不修改输入容器。
- `JSON_DEPTH_LIMIT = 128`，只计同时打开的 array/object 容器数：标量 0，`[]`/`{}` 为 1；128 成功，进入第 129 层前拒绝，解析和序列化同口径。错误清理/析构也必须安全。
- jsonParse 的非 string 参数 → 既有 `TypeError::BadOperands { op: "jsonParse", lt: 实际类型, rt: "string" }`；两内置严格 1 参数，个数错误沿用既有模板。
- 新增 `ValueMsg::JsonParse { offset: usize, reason: String }` → `ValueError: JSON 解析失败（字节偏移 {offset}）：{reason}`。offset 为 UTF-8 输入从 0 开始的错误字节位置，EOF 可等于字节长度。语法错误在首次无法继续消费位置；重复键在第二个键开启双引号位置；数字转换错误在该数字起始位置；深度错误在第129个容器开启符位置。
- JsonParse reason 仅为以下固定文本：`不是合法的 JSON`、`对象键重复`、`整数超出 i64 范围`、`浮点数超出有限范围`、`嵌套深度超过 128 层`。
- 新增 `ValueMsg::JsonStringify { reason: String }` → `ValueError: JSON 编码失败：{reason}`；reason 仅为 `不支持函数或结构体定义`、`浮点数必须有限`、`循环引用`、`嵌套深度超过 128 层`。遇多处非法值时按数组顺序/struct 已排序数据字段深度优先检查，先遇者报错。
- 所有错误 span 为内置调用点，仍按既有 traceback / `--json` / exit 2 传播；不截断为 null、不跳过非法数组元素、不写出部分结果。
- 独立 lib codec（建议 `src/json_codec.rs`）与 bin 的历史 `src/json.rs` 分离，保持 runner JSON 协议。解析时间 O(n + Σk log k)，n 为输入字节数、k 为各对象键数（键比较成本另计）；空间 O(n)，递归栈有界128。编码 O(展开输出字节数 + Σk log k)，包含重复共享子树展开和键比较成本；内存含输出、对象排序临时表和深度栈，不承诺仅按唯一对象数线性。

### 16.4 StructObj 内部字段索引（效率约束）

可增内部 `HashMap<String, usize>` 映射字段名到 `fields` 槽位，改善 get/set/contains 平均查找成本；不得暴露随机迭代顺序。`fields` 仍保留插入序，同键 set 原位替换、动态加字段追加；所有创建/复制/删除路径须保持索引一致。数据面函数过滤、键排序、引用同一性和别名写入均不变。JSON 一次批量创建字段并一次建立索引，禁止 O(k²) 构造。HashMap 平均复杂度不等于最坏 O(1)，文档必须准确标注。
### 16.5 CLI 入口契约

完整命令形态、选项区边界、stdin错误模板及 `--json` 兼容要求冻结于 [syntax.md](./syntax.md) §16；本节规范性引用该处，禁止另行猜测。tooling-dev 使用本节16.1新求值入口，并保持现有 runner `eval_case` 为空argv的文件入口。
### 16.6 argv 一等函数载荷与词法优先补钉

- 仅 argv 新增一等取值能力；不得泛化旧69内置或 jsonParse/jsonStringify。`Closure` 可新增 `builtin: Option<&'static str>`，本轮仅合法值 `Some("argv")`。所有旧用户函数/占位构造器默认None；用户载荷与builtin标记互斥。仅显示名相同或user=None不是内置判据。
- 每个Interp持有同一个 `Rc<Closure>` argv函数值，名字为argv，独立于用户词法env；Ident与直接调用均先查用户词法变量，未命中才回退argv。未遮蔽时 `argv == argv` 为true；`let f=argv` 后 `f==argv` 为true。函数同一性、type为 `function`、str显示 `<fn argv>` 均沿既有Func规则；A5将其视函数方法字段过滤，JSON根值/数组元素仍拒函数。
- 用户可以用let/var/形参/函数名遮蔽argv，行为与其它名字相同；已保存别名仍指原argv函数。不能把argv直接放入globals形成新的可见debug绑定或让变量查找错误地跳过词法变量。不同执行的函数值不承诺同一性。
- `call_user` 在占位拒绝/用户帧逻辑前识别显式builtin标记，统一直接/别名/闭包/容器/高阶路径。argv零参检查后返回当前Interp参数的新数组；方法取调用带来的self不改变argv参数。该内置不压用户函数帧、不增加用户函数递归计数；错误span及traceback按既有内置错误冒泡，别名arity消息仍使用 `argv`。
- `Closure::named` / `anonymous` 的历史无函数体占位仍不可调用；用户定义同名argv函数没有builtin标记，仍正常执行其函数体。其它builtin标记不得无声明地开放。
### 16.7 无直接声明 while 体的空环境消除（内部优化许可）

只对 `exec_while`：循环开始前一次扫描body的**直接语句**，若无 `StmtKind::Decl` / `FnDecl` / `StructDecl`，可每轮 `exec_stmts(&body.stmts, scope)`，省略原本不含绑定的物理child Env；有任意直接声明（即使不可达）仍用原每轮新child。扫描不求值、不改变条件求值次数；不得在每轮重复O(m)扫描。其它exec_block和for迭代环境、函数调用帧一律保持现状。

这不是语言作用域合并：逻辑块仍存在，只是不物化零槽位/零绑定的Env。当前未知名赋值报NameError、不创建新绑定；表达式/调用不会在调用者当前env定义名。嵌套if/for/while及函数自身按原规则建环境，所有现有绑定cell、捕获、自身self、let可变性、dump内→外声明序/遮蔽去重及first_slot编号均须不变。不得修改AST Dump.scope或丢失ScopeDebug逻辑链；未来新增绑定语句/表达式、动态eval或作用域身份反射时，必须重新审核判定器。

优化不得改变Flow传播（continue/break/return）、错误/traceback、程序输出或LFZ可达递归限制；成功判据为等价性回归与独立验证，再加既定同输入性能门禁。没有观测到加速也不应改语言语义或降低性能验收。
## 17. T14 前端检查CLI契约（2026-10-27 冻结）

### 17.1 输入与执行路径

规范性命令及CODE/option/分隔符边界见 [syntax.md](./syntax.md) §17；非执行及非静态承诺见 [semantics.md](./semantics.md) §17。tooling-dev实现CLI `check FILE` / `check -e CODE` / `check -`；不需要新增lib公开API或修改核心文法。

- FILE使用load_file，保留扩展名/UTF8/BOM/换行/前导与错误顺序。CODE及stdin使用load_source，伪路径分别 `<command>`、`<stdin>`；stdin读取失败及非法UTF8沿既有无span IOError消息 `无法读取标准输入`。
- 加载→lex→parse（含AST深度检查）与Program析构全在现 `evaluator::on_eval_stack` 大栈包装内执行；使用线程包装器本身不是LFZ求值。不得把深Program移出该线程到主线程析构，成功结果只需无AST的标量状态。
- 不得调用 `eval_module*`、求值用的 `run`、`Interp::new`或内置执行入口；不建立用户运行帧、不计算argv、不重定向LFZ程序输出。错误持有Loaded及原LfzError用于诊断，不构造TracedRun。
- CODE原样保护；CODE外 `--`（包括空尾）拒绝；`check --help`拒绝，多源/缺源/缺CODE/未知option沿CLI参数错误。CLI错误写原usage风格stderr文本、stdout空、exit2，即使存在 `--json` 也如此；不伪造LFZ错误类。唯一JSON保证限于已识别且参数合法的check命令。

### 17.2 结果与精确JSON

正常模式：通过时stdout/stderr均空，exit0；任一前端错误按原人类错误格式写stderr、stdout空，exit2。不能把输入里没有执行的assert/fail/check当成exit1或运行失败。

合法check的 `--json` 成功stdout恰一行，UTF8，无额外程序输出：

```json
{"ok":true,"mode":"check","executed":false}
```

失败stdout恰一行，字段及顺序为：`ok:false`、`mode:"check"`、`executed:false`，再接原错误协议的 `error`、`message`、`file`、`line`、`col`、`traceback`。其中error仍是既有LFZ类名，message/span与原前端错误相同；file为原路径或伪路径，line/col为1-based或无位置时null。复用既有JSON编码与 `push_error_fields`，不能手写未转义的JSON或改旧run/test对象。

前端错误不产生真实运行栈。复用原前端错误协议：有span时traceback是单个 `{file,line,func:"<module>"}` 位置记录；无span的IOError为空数组。`executed:false`明确其不是执行trace。成功仅上述三键，失败不删错误字段、不输出业务值。JSON模式不额外输出人类错误块，stderr应空（宿主进程异常除外）。

### 17.3 非执行与兼容验收

必须验证文件/-e/stdin三来源的合法和非法源码；CODE含 `--json`/`--`/空串/Unicode、选项前后重复、缺源/多源/缺CODE/`check --help`/CODE外 `--`；文件前导/UTF8/BOM/CRLF及准确位置；现parser上下文和深度边界。

非执行见证至少包含：源内print/eprint/dump/input无程序输出/提示；writeFile/appendFile目标不存在时不创建、存在时字节不变；readFile路径错误不被运行；无限循环/递归入口只解析并在合理进程超时内返回；副作用语句之后出现词法/语法错也不产生副作用。需结合真实进程证据和调用路径审查，不只凭源码关键词搜索。

原CLI测试、run/test JSON、脚本argv、loader/parser行为及内置check均不得回归。原T13源码hash和性能证据冻结不回写；T14实现后的新产物另记hash和验证结果。本节冻结的是接口目标，不宣称已经实现。

## 18. T15 attempt 实现契约（2026-10-29 冻结）

### 18.1 注册及ABI

内置集合由72增至73，BUILTIN_NAMES字母序含attempt、is_builtin("attempt")为true。它是需要Interp上下文的特殊内置，不伪装普通Builtin/Hof；lookup/lookup_hof不登记attempt。无Interp的builtins::call/call_with("attempt",...)保持通用表未找到名字的NameError，不模拟空trace。旧Builtin/Hof/Invoke公开ABI不改，不泛化函数值。

eval_call的Ident路径先执行原变量遮蔽查找；仅无绑定且名为attempt时按原eval_args一次求完实参，控制流仍按原规则传播，再调用私有Interp helper。不得通过再次求值AST重跑实参。helper依semantics §18先校验1参/Func/零形参（argv允许、无用户体placeholder拒绝），校验错误span取attempt调用Call节点。未进入f不新增帧；helper内部经原call_value/call_user调用f，保留fresh环境、self捕获、参数/cell及原深度检查，不造attempt帧。

### 18.2 状态、错误与数据

semantics §18的三字段Result、五字段error、三字段traceframe及插入顺序是规范性协议；构造用Value普通struct/array/string/int/nil，不手拼JSON。错误span与消息逐字复用LzError。函数显示名复用现规则；错误记录在恢复前取完整trace，存普通Value字符串，不保留对可变内部trace的别名。成功value保留原引用拓扑。

可恢复分类须匹配Rust错误variant，OverflowMsg::IntegerOutOfRange可恢复，Capacity不可；不靠中文message字符串判断。snapshot trace.clone与depth在实际调用前获取；成功/可恢复Err恢复整个快照，不回滚func_names/next_func_id/用户状态。不可恢复Err原样返回，不能先清帧。此操作不要求更改旧call_user的错误保帧行为；仅新helper在受控边界处理。

### 18.3 最低验收矩阵

成功nil/标量/共享容器/返回函数；多行坏JSON混批；全部可捕获类及白名单外错误（资源错误优先单元构造验证，禁止真实abort压测）；outer arity/nonfunc/callback arity/实参求值错误位于边界外；argv、遮蔽与管道；嵌套成功/失败/外层接住内部校验错误；连续大量失败后成功、恢复后正常递归；完整trace名称/位置及失败后后续未捕获错误无旧帧；捕获失败前写出的闭包后续调用及trace标识不串；数组/cell/输出/IO不回滚；check(false)仍警告且成功；旧542测试和CLI/check/JSON协议回归。人/AI指南只引用本节并实跑，不以失败数等于0代替上述状态恢复见证。

## 19. T16-A 来源身份与定义模块基础（2026-10-29 冻结）

### 19.1 范围与责任

本节仅为模块前置基础；A不解析import/export、不加载模块/IO、不切globals、不改CLI输出、不新增内置。Span、LzError、TraceFrame、Loaded、Program旧形状及T15 Result/error/trace三/五/三字段保持。73内置、旧入口及旧无模块程序逐字节兼容。core-dev唯一写src/source.rs及src/lib.rs声明；runtime-dev唯一写src/value.rs、src/evaluator.rs；tooling本批只读。B/C/D另冻结，不以本批冒称模块可用。

### 19.2 source.rs 精确公开签名

新增不同newtype SourceId(pub u32)、ModuleId(pub u32)，均derive Clone/Copy/Debug/PartialEq/Eq/Hash，各自impl关联常量ENTRY=Self(0)。两编号分域，不可按相同整数推断同一身份，ModuleId只在所属session有意义。

SourceRecord derive Clone/Debug，公开字段：
- identity: Option<std::path::PathBuf>
- display_path: String
- loaded: Option<crate::loader::Loaded>

SourceMap derive Clone/Debug/Default，私有Vec<SourceRecord>，公开方法：
- pub fn new() -> Self
- pub fn len(&self) -> usize
- pub fn is_empty(&self) -> bool
- pub fn get(&self, id: SourceId) -> Option<&SourceRecord>
- pub fn insert(&mut self, record: SourceRecord) -> Result<SourceId, SourceMetadataError>

SourceMetadataError derive Clone/Debug/PartialEq/Eq，公开enum变体：
- TooManySources
- FrameCount { expected: usize, actual: usize }
- UnknownSource { id: SourceId }
- UnexpectedErrorSource
- MissingErrorSource

LocatedFailure derive Debug，公开字段source_id: SourceId、error: Box<crate::error::LzError>。只关联既有错误与来源，不改语言错误类/消息。ID在拥有SourceMap的边界验证，不是跨map全局身份。

insert先checked转换当前len为u32，失败返回TooManySources且不修改map；首次成功ID为ENTRY，后续按插入序稳定递增，无删除/重排。禁止截断cast；此检查不宣称捕获宿主OOM。get未知ID为None，不panic、不回退ENTRY。测试编号上限用有界辅助逻辑，禁止为测试分配u32::MAX个记录。

SourceMap只持元数据/可选Loaded，不持Program、Env、Rc用户值或求值器；不扩大现Transfer跨线程假设。A不canonicalize/读文件/去重/小写化路径，identity只是调用者提供身份，display_path允许虚拟路径。别名合并和路径验证归后继图层。加载失败允许loaded=None，禁止伪造入口文本给该错误画下划线。A不提供文本渲染。

### 19.3 运行时扩展签名与验证

UserFn新增公开home_module: crate::source::ModuleId。make_closure记录Interp私有active_module，Interp::new初始化active_module=ModuleId::ENTRY；A全部实际执行仅ENTRY，不注册/切换模块，不改查名、globals父链、self、cell、func_id和函数ID表。只存整数不Rc持home Env，避免普通模块必然引用环。没有新增跨session函数值调用API。

UserFn公开结构新增字段属于Rust构造API迁移：仓库唯一make_closure构造点机械补字段，外部手写literal须补home_module，不能声称完全无Rust API变化。TracedRun本已有私有func_names，外部原本不能literal构造；仓库唯一构造点补新私有origins=None。

TracedRun私有origins: Option<RunOrigins>，RunOrigins由runtime私有实现，持Rc<SourceMap>、Vec<SourceId>逐帧来源及Option<SourceId>错误来源。旧eval_module*入口全部origins=None，不自动生成来源表；CLI仍旧路径不读新sidecar。

新增TracedRun公开方法：
- pub fn attach_origins(&mut self, sources: std::rc::Rc<crate::source::SourceMap>, frame_sources: Vec<crate::source::SourceId>, error_source: Option<crate::source::SourceId>) -> Result<(), crate::source::SourceMetadataError>
- pub fn frame_source(&self, index: usize) -> Option<&crate::source::SourceRecord>
- pub fn error_source(&self) -> Option<&crate::source::SourceRecord>

attach_origins先完整校验再原子替换；任一失败保留旧sidecar。校验顺序规范性：1)frame_sources.len与self.frames.len相同，否则FrameCount{expected:self.frames.len(),actual:frame_sources.len()}；2)按帧序每ID须存在，否则UnknownSource；3)结果Ok时error_source须None，否则UnexpectedErrorSource；结果Err须Some，否则MissingErrorSource；4)存在的error_source也须有效，否则UnknownSource。返回结构化宿主元数据错误，不panic、不伪装LzError、不把坏来源悄悄改为入口文件。

frame_source用实际帧下标，不是func_id；无sidecar/越界/宿主后改公开frames导致无法匹配时None。error_source在无sidecar或当前结果非Err时None；查询每次经SourceMap::get安全访问。宿主后改公开frames/result可能使旧attach保证失效，访问器仍须安全。后继诊断调用者必须显式处理None，不能拿其它文件代替出错文件。

A只附加载荷，不根据active状态在冒泡后猜真实错误source；后继B/C/D在错误发生/压帧时记source再附加。T15 attempt继续只序列化旧TraceFrame固定字段，完全忽略origins；A active_module始终ENTRY，成功/恢复后仍ENTRY。真正模块切换与恢复快照扩展归C，不在A先轮换globals。

### 19.4 最低验收及后继

core新增unit覆盖空map/插入稳定ID/loaded=None/未知ID/虚拟路径/LocatedFailure保留错误与位置。runtime新增unit覆盖legacy None、成功与错误附加、不同帧不同来源、所有拒绝变体与原子不修改、越界None、home_module ENTRY及attempt恢复后ENTRY/旧trace保持。允许合成多来源元数据，不能把它当模块执行证据。旧549测试及CLI/check/attempt/JSON/app行为继续通过；新计时前不得宣称性能改善，R304仍OPEN。

B另冻结ImportDecl/ExportDecl、模块图及受控上限；C另冻结session缓存/独立globals/home切换；D接真实跨文件CLI诊断。不得预拼源码、隐式global注入或多个Interp互调绕过身份约束。

## 20. T16-B 静态模块图与非执行检查（2026-10-29 冻结）

### 20.1 范围和语法

B只实现模块声明前端与完整静态图检查，不初始化模块；C前模块执行明确拒绝。A §19所有类型形状不改，T15 Result/error/trace三/五/三字段不改。73内置保持，旧无模块程序行为/错误字节保持。

顶层语法：import IDENT from STRING_LITERAL；export IDENT (, IDENT)*。路径不允许插值（即使常量表达式）、空串、NUL、绝对/根/盘符/UNC路径；必须显式.lfz（ASCII不分大小写），允许..，不搜索或补扩展名。import/from仍用现Reserved词token；不新增全局export关键字。仅语句起始Ident("export")后同一语句紧接Ident才识别导出声明，旧export变量/赋值/调用/字段/下标/表达式均不变。两声明只允许Program直接顶层，不能用仅fn_depth/loop_depth代替完整上下文校验。import须先于任何非import顶层语句（export也是非import）；注释空行不影响。原语句终结NEWLINE/}/EOF规则保持。

StmtKind新增ImportDecl(ImportDecl)、ExportDecl(ExportDecl)，结构derive沿现AST，字段精确为：
- ImportDecl { alias: String, alias_span: Span, path: String, path_span: Span }
- ExportDecl { names: Vec<ExportName> }
- ExportName { name: String, span: Span }

外层Stmt.span指import或上下文export起始；path为解码后字符串，path_span指引号起始。AST不持SourceId，Program旧形状不变，深度visitor把两新节点视作叶。导出名必须对应本模块直接let/var/fn/struct/import绑定，可前向列名；不隐式导出builtin或块内局部，重复导出拒绝。import alias之间、与任何其它顶层直接绑定重名拒绝；不涉及import的普通顶层重声明仍按旧规则。

### 20.2 loader增量接口与raw计数

core-dev新增公开CountedLoaded { pub loaded: Loaded, pub raw_bytes: usize }（derive Clone/Debug），以及：
- pub fn load_file_counted(path: &str) -> R<CountedLoaded>
- pub fn load_source_counted(text: &str) -> R<CountedLoaded>
- pub fn load_bytes(path: &str, bytes: Vec<u8>) -> R<Loaded>

load_bytes按现load_file读取之后的UTF8校验→BOM/换行归一→扩展名/前导逻辑；旧load_file/load_source签名和错误保持，可内部委托复用。raw_bytes在任何归一化和前导消费前按实际读入字节或text.as_bytes().len取得，不事后metadata、不重读。根的counted读取仍无新上限；只有成功parse且确含模块声明才进入图admission。旧无模块大root不被2MiB截断，图预算不是root读取/解析沙箱。

依赖由图层File+Read::take(limit+1)有界读取，再load_bytes，不先无界load_file再检查。limit=min(2MiB,16MiB-当前唯一源码总量)，多读1字节用于超限检测。单文件优先于总量错误，原始计数checked_add；已缓存身份不再读/计数。失败读取沿下述模块IO消息，不修改旧root IO消息。

### 20.3 modules.rs类型、API与来源归属

core-dev新增pub mod modules。下列宿主结构字段均pub，derive Debug（需要克隆的叶结构可Clone；Graph不要求Clone）：
- ParsedEntry { program: Program, loaded: Loaded, display_path: String, identity: Option<PathBuf>, base_dir: PathBuf, raw_bytes: usize }
- ResolvedImport { alias: String, alias_span: Span, path_span: Span, target: ModuleId, span: Span }
- ExportBinding { name: String, span: Span }
- ModuleUnit { id: ModuleId, source_id: SourceId, program: Program, imports: Vec<ResolvedImport>, exports: Vec<ExportBinding> }
- ModuleGraph { sources: Rc<SourceMap>, units: Vec<ModuleUnit>, entry: ModuleId, initialization_order: Vec<ModuleId> }
- GraphFailure { sources: Rc<SourceMap>, failure: LocatedFailure }

pub fn module_declaration_span(program: &Program) -> Option<Span>返回首个直接模块声明span；pub fn build_graph(entry: ParsedEntry) -> Result<ModuleGraph, GraphFailure>消费已parse的根快照，不再次读/parse。调用者必须在on_eval_stack内构图并析构所有Program，不能把深AST带回主线程drop。无声明root不调用图，不受图限制。

ModuleId只索引units，SourceId只索引SourceMap，两ENTRY各0，不可互转。模块图拥有AST，SourceMap仅metadata/Loaded，不能持Env/用户值。Loaded诊断副本会增加内存，不能声称仅一份源码。图构建期间用owned SourceMap，成功/失败返回时Rc包装，不反复Rc::make_mut克隆全表。先局部load，成功insert Some(Loaded)，失败insert None记录即返回；不修改A增加setter或伪造文本。新依赖身份成功parse后才有ModuleUnit，失败来源也可占SourceId，故两ID不保证同序。

文件root base是当前文件绝对父目录，CODE/stdin base为开始捕获cwd。根display_path保留原CLI路径或<command>/<stdin>以兼容旧诊断；依赖采用首次发现canonical绝对UTF8路径。identity用canonical PathBuf去重/跟随symlink，不手写lowercase。无法canonicalize的依赖用base.join(path)绝对词法路径作失败显示名（保留..允许）；无法表示为UTF8归ModulePath，定位导入者。源码快照每身份只读一次，C必须消费同图，不重读；不保证文件系统多文件原子快照。相对..及symlink不构成权限沙箱。

### 20.4 限额与确定性迭代DFS

冻结常量：
- MAX_MODULES: usize = 1024，含root，唯一canonical身份计一次。
- MAX_MODULE_DEPTH: usize = 128，显式DFS栈长度含root=1；129边在读目标前拒绝。
- MAX_SOURCE_BYTES: usize = 2*1024*1024，单个进入图的原始源码。
- MAX_GRAPH_BYTES: usize = 16*1024*1024，唯一原始源码总和。

这些是保守初版工程限额，不是RSS上限或性能证明；token/AST/副本会放大。root前端已经成功后才图admission，超限不允许自动放宽/重试。边界127/128/129链、1023/1024/1025节点及bytes等于/多1必须有界测试，禁止真实OOM。

显式栈Frame{module_id,next_import_index}及Visiting/Ready颜色，不用Rust递归DFS。先根admission及本地静态核验，然后按每模块import书写序DFS。每条边先路径/canonical身份；Visiting回边报cycle；Ready加共享边，不加深度/节点；Unseen先节点再深度预算，再限读/源码预算、parse、本地静态核验、入栈。全部边完成标Ready、append initialization_order、pop。后序稳定，同身份一次；错误不靠HashMap遍历决定先后。

错误先后：每文件load/lex/parse完整成功后才静态检查；静态检查先收集全文件direct bindings，再按源码声明/名字次序选首个违规（冲突定位第二次发生的名字span，前向导出可用）。在进入依赖前完成当前文件本地核验；root图admission先于其本地核验。节点先于深度、单文件先于总bytes，循环/Ready识别先于新增预算。图错误终止无初始化。

### 20.5 规范性错误和显示路径

外层LzError仍12类；core扩SyntaxMsg以下变体与精确message：
- ModuleDeclTopLevel => "模块声明只能出现在文件顶层"
- ImportOrder => "import 必须位于其它顶层语句之前"
- ModulePath => "模块路径必须是非空的相对 .lfz 字符串（不含插值或 NUL）"
- ImportConflict { name: String } => "模块导入名冲突：{name}"
- DuplicateExport { name: String } => "重复导出：{name}"
- MissingExport { name: String } => "导出名没有顶层声明：{name}"
- ModuleCycle { chain: String } => "模块循环依赖：{chain}"
- ModuleLimit { kind: String, limit: usize } => "模块图超限：{kind}（上限 {limit}）"，kind仅节点数/依赖深度/单文件字节/源码总字节，limit用十进制上述常量
- ModuleExecutionUnavailable => "模块执行尚未启用"

非法声明位置/顺序在关键字span；路径在path_span；冲突/重复/缺导出在对应name span。cycle在导入者当前边span，chain按显式栈从首次重复模块到当前再闭合，用canonical显示路径和" → "分隔。节点/深度/依赖bytes超限定位导入者边span，root自身bytes超限为根Span::START。模板语法不完整（缺IDENT/from/字符串/逗号后缺名等）沿原Expected/TwoStatements等parser错误规则，不伪造成功。

依赖canonicalize/open/read失败用原Io variant，msg精确"无法读取模块：{target}（导入于 {importer}:{line}:{col}）"，span=None，failure.source_id指目标失败来源（loaded=None）；root读取失败保持原IOError字节。依赖UTF8/前导/lex/parse保持原类/消息/span但source_id指真实依赖。SourceMap插入编号理论溢出在图1024限制下不可达，不panic模拟或与LFZ业务错误混为一谈。

tooling从GraphFailure.sources找到failure.source_id，用其display_path/loaded做前端诊断和JSON；无Loaded不显示其它文件的下划线。前端JSON仍现字段序，file为出错文件；traceback带span时为单<module>定位项，IO无span为空。导入链在消息而非伪造运行栈；check success仍{ok:true,mode:"check",executed:false}。

### 20.6 check、run、旧eval与分工

tooling：同一根前端只parse一次，成功后若无模块声明走旧路径；有声明时check build_graph并在大栈drop，无模块初始化、求值/Interp/输出。root CountedLoaded/Program/base移交图，错误使用真实来源。源码内writeFile/input/while true在check完全不执行。

C前run和所有公开旧eval入口对整个root Program的模块声明做执行前preflight，首个模块声明返回ModuleExecutionUnavailable的SyntaxError，任何LFZ输出/IO/初始化之前失败。仅export也拒绝；即使前面有print/writeFile也不能先执行。B run不承诺构图优先检查其它依赖错误，直接unsupported；不要默默忽略新节点。runtime exec_stmt仍对两个新变体防御性拒绝。parser禁止嵌套声明，若宿主手工构造非法AST不扩展旧前端保证。

core拥有AST/parser/error消息/modules/loader计数与bytes复用/lib登记；runtime仅拥有eval preflight和两个新节点防御match，不启用globals/session；tooling拥有CLI根计数、base、check图与GraphFailure渲染/JSON。单写者串行编译：core先，runtime/tooling适配exhaustive match后唯一编译。三spec及所有旧行为测试不得由实现角色私改；仅明确新节点引起的机械元数据测试迁移先报team-lead。

### 20.7 验收与C衔接

必须验证旧A最终基线/549及新增、T15/T14/JSON/apps；contextual export旧变量/表达式不变；三文件/菱形/重复身份/symlink可用性；每次图隔离；前向导出/冲突/缺名；缺文件/非法UTF8/#42/parse/cycle真实路径和位置；全部限额边界/legacy大root例外；check对依赖IO/循环/递归无执行；run任意模块声明零副作用拒绝。所有fixture在项目内，不因runner默认发现把库文件当顶层独立用例。

C需另冻结Session/独立Env/namespace导出快照及函数home_module切换、初始化失败与attempt恢复active module/source状态；必须消费本图一次性快照。B交付不得称模块运行完成，不新增T15Result字段，不以功能测试替代R304性能验收。

### 20.8 B执行预检返回载荷与补充owner
src/error.rs的SyntaxMsg模块变体及message匹配由core-dev唯一写，外层LzError类不增加。runtime所有旧eval入口汇合到eval_module_on_thread_with_args，在Interp::new及任何exec之前调用module_declaration_span。存在声明时立即返回TracedRun：result=Err(Syntax(ModuleExecutionUnavailable,first_decl_span))，frames=[TraceFrame{func_id:MODULE_FUNC_ID,span:first_decl_span}]，func_names=[None]，origins=None；其它旧封装按原result丢弃或暴露trace。exec_stmt两个变体另作同类错误防御。load_bytes精确接owned Vec<u8>，可直接String::from_utf8转移所有权；counted入口在移交bytes前保存len，不额外clone或重读。

### 20.9 冲突定位补钉（旧AST保形）
旧Decl/FnDecl/StructDecl没有独立name_span，禁止从名字长度或关键字列猜算。import-import冲突定位后一个ImportDecl.alias_span；import与普通直接let/var/fn/struct绑定冲突定位该普通声明Stmt.span（关键字），不扩旧AST字段。重复/缺失导出仍用精确ExportName.span。本条细化20.4/20.5名字定位，优先适用；原其它节点形状保持。

### 20.10 有界读取的已观测字节与错误优先（2026-10-30 补钉）

依赖读取仍为take(min(单文件限额,源码总量剩余额度)+1)的有界读取，上限、错误类和既有消息不变。单文件先于总量的优先级只对本次实际已读字节成立：先用实际bytes.len判断是否已证明单文件超限，再判断是否已证明总量超限；不能声称了解未读取的文件尾部，也不能为决定“真实总文件是否也超单文件”继续读取突破预算。

例如总量仅余100字节、单文件限额2MiB时，读取最多101字节；若读到101字节，可判源码总字节超限，不能由这101字节推断整文件是否也超过2MiB，不因此补读至2MiB。若本次实际已读字节确已同时越过单文件与总量，仍按单文件优先。哨兵字节用于判定，不接纳为成功图内容。此条细化20.2/20.4的可观测范围，不改限额、消息、快照、legacy例外或A接口。

## 21. T16-C 单session模块执行（2026-10-30 冻结）

### 21.1 激活边界与公开宿主API

B独立PASS后方可实现/启用本节。只激活已由B检查的本地ModuleGraph，不新增语法、内置、动态import、远程包、循环live binding或并行初始化。check仍仅构图不执行。旧无模块CLI及旧eval_module*入口完全保持；旧eval入口遇模块Program继续§20 guard，不能绕过图验证。

runtime新增evaluator公开（GraphExecutionError derive Debug，并实现Display/std::error::Error）：
- pub enum GraphExecutionError { InvalidGraph { reason: String }, Origins(crate::source::SourceMetadataError) }
- pub fn eval_graph_traced_on_thread_with_args(graph: &crate::modules::ModuleGraph, args: &[String]) -> Result<TracedRun, GraphExecutionError>

该函数借用B图，调用方已在on_eval_stack内，parse/build/eval/drop graph全在该大栈。禁止重读图文件、重复parse或克隆全部Program。外层Err仅宿主坏图/元数据不变量，内部TracedRun.result保留LFZ成功/错误。InvalidGraph Display为reason，Origins Display为“来源元数据不一致：{error:?}”；不新增LFZ错误类，不让attempt捕获宿主错。

CLI宿主Err为stdout空、stderr精确“lfz: 模块执行上下文无效：{detail}\n”、exit2，属于合法JSON承诺中的宿主异常例外，不伪造LFZ业务成功/ValueError或InternalError大类。普通build_graph产生的合法图不得走该异常分支。

### 21.2 执行前图验证及内部状态

任何LFZ初始化之前完整验证：
1) entry与所有units.id有效，units[i].id=ModuleId(i)，每unit唯一source_id且存在、loaded为Some；
2) initialization_order是units的无重复全覆盖排列，entry最后；每import target存在且在导入者之前；
3) 每unit.imports与Program直接ImportDecl数量/书写序/alias及span一致，alias不重复且不与其它直接绑定冲突；export metadata与Program导出名/顺序/span一致，名字有直接顶层绑定且不重复；import仍先于非import。
失败返回InvalidGraph，reason可取稳定宿主原因，不改语言消息，零LFZ副作用。该检查是结构一致性，不重新parse或证明宿主手工伪造AST与文本对应，不构成安全沙箱。metadata无法通过A校验返回Origins，不expect/unwrap掩盖不变量。

Session保持单Interp，新增私有模块实例表，实例持该模块Rc<RefCell<Env>>、可选namespace及Uninitialized/Initializing/Ready/Failed状态。模块实例表只归本次执行，不用全局static缓存。graph由入口借用，Env不持整个Program。现globals作为active模块Env别名，active_module与它一致；UserFn只整数home_module及captured cells，不强Rc持home Env。func_names/next_func_id统一分配，模块间不重置，不被attempt回滚。

### 21.3 初始化、导出与结果

按B initialization_order依赖后序串行初始化，每模块新独立globals。初始化前按ResolvedImport书写序绑定不可重绑定alias，值为目标Ready namespace同一Rc；未Ready是InvalidGraph，不给空对象继续或重复执行。graph模式ImportDecl/ExportDecl仅声明动作，返回Flow::Value(Nil)，不做IO或加载；legacy模式继续guard。普通语句按旧规则，结果为该Program最后Flow Value/Return，防御性Break/Continue仍Nil。

模块顶层push独立MODULE_FUNC_ID=0帧，名称仍<module>、source为该unit真实来源；初始化成功清理该模块帧，下一模块不嵌套伪造根执行栈。失败立即停止整个session，不执行剩余计划或entry；故障trace只含实际已进入的模块/函数。图阶段与依赖初始化错误不在尚未运行的entry attempt边界内。无隐式初始化重试。

成功后按ExportBinding书写序读取最终模块直接绑定，构造新普通struct namespace，Ready。无导出为空struct。标量是完成时快照，数组/struct/函数按现引用拓扑共享；namespace字段可按原规则修改，不回写模块变量绑定。多alias/菱形复用同namespace与一次初始化副作用。keys/JSON仍A5过滤方法，不作为导出函数反射。entry namespace也可建立，但最终TracedRun.result是entry求值结果，不自动返回namespace或业务JSON。

所有模块argv使用同一显式args快照，每次argv调用仍新数组。module身份与cache只在本次session有效，不新增跨session函数Value调用API。IO/RNG及初始化次序可观察，不并行。

### 21.4 UserFn home切换与故障来源

UserFn现home_module在graph make_closure记录active_module。call_user先沿旧顺序做argv/placeholder/arity/递归检查；普通函数在定义模块globals下建fresh参数帧，捕获cell及self_override优先级保持。保存调用者active_module/globals，切home后执行，但必须显式收集outcome而非让?绕过上下文清理。成功和Err都恢复调用者active/globals；成功才沿旧规则pop trace/depth，Err保留故障trace/depth。

新增trace_sources与trace严格同长度，每个module/函数/默认诊断帧push/pop同步。实际错误处保存fault_source，或从尚完整的故障帧来源获取；恢复active之后不可用当前caller猜错误文件。未压callee之前的参数/arity错误属于caller。最初故障来源在冒泡中不能覆盖为外层来源。

attempt保存并在成功/可恢复失败后还原trace_sources、fault_source、active_module/globals及旧完整trace/depth；先用故障trace构造旧Result。不可捕获错误保留故障帧/来源，callee的active/globals安全清理仍执行。func_names/next_func_id、module实例及namespace、用户cell/IO/RNG不回滚。T15 Result/error/frame字段三/五/三不增file；跨模块Result只有各帧真实行列，无文件名，指南不得宣称完整文件定位或把路径拼进func字段绕过协议。

图执行结束构造TracedRun并一次attach_origins(graph.sources,trace_sources,Err时Some(fault_source))；成功error_source=None；A校验失败向外Origins传播，不panic或错误文件fallback。legacy无模块入口继续origins=None。

### 21.5 StructDef默认值定义上下文

不新增lib.Point{}语法；导出模板用“let P=lib.Point”再“P{}”。模板/默认值实际所有权由runtime统一实现。

value.rs新增公开TemplateContext（derive Clone/Debug）：
- pub home_module: crate::source::ModuleId
- pub captured: Rc<Vec<CapturedVar>>
- pub self_val: Option<Value>
- pub defaults_func_id: u32

StructDef新增pub default_context: Option<TemplateContext>。旧Value::struct_def与struct_template签名保持，填None；新增pub fn struct_template_in_context(name: impl Into<Rc<str>>, fields: Vec<(Rc<str>,Expr)>, methods: Vec<(Rc<str>,Rc<Closure>)>, context: TemplateContext) -> Self，填Some。外部手写StructDef literal需补字段，属于明确Rust构造迁移；不得顺手改旧Value语义。

legacy无模块声明始终None，eval_struct_lit继续旧实例化caller Scope默认值行为，旧trace不加帧。graph模式声明为Some，记录定义home、按现make_closure捕获算法升级/继承可见局部cells并捕获self；不强Rc持Env。默认表达式在定义globals+captured/self Scope按字段声明序重新求值，每实例可变默认独立；显式字面覆盖字段在所有默认值完成且恢复caller上下文之后，仍按caller Scope/书写序求值。graph定义上下文是新模块语义，不宣称legacy从来如此。

为了默认Expr真实来源诊断，graph声明若存在默认字段，分配session全局唯一defaults_func_id，func_names显示名精确“{TemplateName}::<defaults>”；函数ID不复用不回滚。TemplateContext无默认字段时仍可分配该ID，但实例化不额外push空默认帧。调用用户方法仍原名，不把模板diagnostic frame冒充普通可调用Value。

默认字段非空时，在切home/push之前按与call_user相同RECURSION_LIMIT检查depth；已到上限则在caller构造span返回原RecursionError，不进入home。否则先更新caller span为构造点，depth加1，切定义上下文，push defaults_func_id/首默认Expr span及定义SourceId。每字段eval_expr更新该帧span，错误保存定义fault source。完成全部默认值或任何Ok非Value Flow时，先pop该诊断帧/source、depth退1、恢复caller active/globals再继续/传播Flow；任一Err恢复active/globals但故障帧/source/depth保留，交外层attempt或最终错误处理。

该边界不改变Flow::Return/Break/Continue传播规则，不把默认Expr当一个吞return的新LFZ函数；它只是来源/递归保护诊断帧。禁止graph默认值递归构造绕过depth保护，资源错误不被attempt捕获。相关上限验证用安全合成state或有界单元，不真实制造stack abort/OOM。不得为跨模块默认值复用caller帧而在错误文件画下划线。

当前仓库StructDef构造点为Value::struct_def/struct_template；runtime调用声明路径用struct_template_in_context，保留旧helper None。新增context字段及构造机械迁移列入作者报告，不删除旧默认值行为测试。

### 21.6 CLI、真实文件与owner

runtime唯一value.rs/evaluator.rs：图入口/结构验证/session、函数及模板context、来源记录与attempt恢复；不改B loader/modules/AST/source。tooling唯一cli.rs与必要main说明：run file/-e/stdin根counted/load/lex/parse一次，若模块build_graph→新eval→stack内drop；check只build/drop。无需新语法或额外CLI选项。

模块runtime错误CaseEval由run.error_source提供最深真实display_path/Loaded。push_error_fields.file及人类错误源码摘录使用该source；traceback_json每帧file及push_trace_frame按实际帧下标查run.frame_source。首10尾30折叠须传原实际index，不能切片后enumerate从0；JSON仍全量不折叠，原字段/键序保持。

旧origins=None路径仍用原path/source逐字节；graph路径缺sidecar/来源是宿主Origins/InvalidGraph，不得伪装legacy fallback。无Loaded不借其它文件源码。初始化错误只渲染实际进入的帧，前端GraphFailure仍原B前端定位协议。

### 21.7 验收与实现GO

B独立PASS后方GO源码；runtime单元/整库先，tooling整合唯一编译，再公开与独立验收。必须验证三文件/菱形单次初始化/两次执行隔离，caller同名全局不串，home闭包/方法/self/局部模板/默认值及显式覆盖来源，导出快照/共享容器/namespace写入，统一argv，初始化失败零entry副作用，attempt跨模块恢复后新错无旧帧/来源，模板默认值资源保护，坏宿主图零副作用拒绝，真实跨文件人类/JSON/折叠trace以及全部legacy T15/T14/JSON/apps。check保持不初始化，C不改B限额或T15协议。

新模块热路径与二进制变化必须实际性能测量后才评价效率，R304仍OPEN。规范冻结不等于实现或正式发布完成。

## 22. T17 执行预算、取消与协作截止时间（2026-10-30 冻结）

依据DECISIONS中team-lead 2026-10-30 08:00独立ADR。本节新增第13个错误类，优先于旧章节“12类”的封闭计数；旧12类触发条件/消息及C接口不变。规范冻结不代表实现验收；源码须待C独立PASS后GO。

### 22.1 错误契约（core唯一写error.rs）
新增以下公开类型，既有LzError其余variants/Span/TraceFrame不改：
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ExecutionLimitReason { StepBudget, Cancelled, Deadline }
    LzError::ExecutionLimit { reason: ExecutionLimitReason, span: Span }
    pub fn execution_limit(reason: ExecutionLimitReason, span: Span) -> Box<LzError>
helper沿既有冷错误路径标记cold/inline(never)。class_name固定ExecutionLimitError，span()恒Some(span)。message仅按reason返回：StepBudget“执行步数预算已耗尽”；Cancelled“执行已取消”；Deadline“执行时间上限已到”。Display仍“类名: 消息”，不追加预算数/token/耗时，无新的JSON键。
Rust下游穷尽match必须新增分支；注册表从12机械扩13，原12行为测试不可删减或泛化成非空断言。R<T>不变。ExecutionLimit不是GraphExecutionError，不伪装Recursion/Overflow，也不属于attempt白名单。T15 Result仍三/五/三字段，系统终止不会被转换成r.error。

### 22.2 宿主入口（runtime唯一写evaluator.rs）
公开于lfz::evaluator，依赖仅std：
    #[derive(Clone, Debug, Default)]
    pub struct EvalLimits {
        pub max_steps: Option<u64>,
        pub deadline: Option<std::time::Instant>,
        pub cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    }
    pub fn eval_module_traced_with_args_and_limits(
        program: &Program, args: &[String], limits: EvalLimits
    ) -> TracedRun
    pub fn eval_module_traced_on_thread_with_args_and_limits(
        program: &Program, args: &[String], limits: EvalLimits
    ) -> TracedRun
    pub fn eval_graph_traced_on_thread_with_args_and_limits(
        graph: &ModuleGraph, args: &[String], limits: EvalLimits
    ) -> Result<TracedRun, GraphExecutionError>
limits按值消费；Clone只共享token身份，deadline值复制；每次调用重新从max_steps建立私有remaining，不共享余量。Default三项None。旧全部公开求值入口签名及返回类型保持，委托Default路径；旧run/eval_module默认unlimited，不额外增加无args/非traced/graph跨线程wrapper。本节不改变unsafe Transfer的约束，EvalLimits不持LFZ Value/Rc/Env。

第一新入口沿现on_eval_stack创建大栈；另外两个仅供已在大栈调用者使用。graph仍先完整validate_execution_graph，坏graph外层Err且无LFZ执行；有效graph的资源终止位于Ok(TracedRun{result:Err(..),..})，origins按C校验。前端失败/旧eval模块声明拒绝发生在求值前，优先于limits；库传入已过deadline不覆盖SyntaxError/InvalidGraph。解析/加载/图构建不扣求值预算。

### 22.3 单调计步与poll
执行会话持私有预算状态，只有一次Interp；模块初始化、导出函数、闭包、方法、graph模板默认值、高阶回调全共享。每次实际进入exec_stmt和eval_expr各算一个求值步；不是其inner递归以外另计，也不按Rust调用次数计。import/export经过exec_stmt各1步；缓存模块不重复执行。函数入栈、绑定、namespace构造、默认值诊断帧本身不额外计步；其中实际stmt/expr仍计。现有优化不得在有预算路径跳过语义要求的节点入口。
每个节点在其自身任何求值副作用前，以当前节点span及当前真实source执行一次poll，固定优先级：cancel置位→deadline已到→remaining==0。三者均未触发且remaining为Some则减1，随后进入节点。预算N允许最多N个节点进入；减到0不立刻失败，下一节点才StepBudget。u64减法只在大于0后，unlimited不累加计数、不允许wrap。无公开used_steps统计。
poll失败须通过现有note_error_span/source路径在故障点记录当前span/source，active/globals恢复前保存；不能直接在包裹层?返回而漏记fault。成功poll无需改写既有可见trace位置，避免改变非资源IO错误定位；“当前节点定位”指poll用该节点参数并在失败时先记该节点，不能使用上一节点位置。不限额路径沿旧trace行为。
零语句Program不进入节点，即使max_steps=0、token已取消或deadline已过也成功nil；空模块同理。没有入口额外poll，不为构图/空程序伪造语句。单表达式程序1有Stmt+Int两步：N=0在Stmt失败，N=1在Int失败，N=2成功。let x=1换行x共4步，N=4成功；供确定性边界见证。
节点路径相同则步数相同；不把HashMap探测、Rust格式化、graph验证计入。资源配置不改变普通表达式的结果与副作用顺序；终止前副作用保留，失败节点本身未开始，但其已执行的父/兄弟节点副作用不回滚。

### 22.4 attempt、来源与生命周期
remaining/deadline/token永远不加入attempt快照，成功/可恢复失败均不补步、不重置token或延长deadline。多层attempt不能吞ExecutionLimit；终止原样穿透、保留故障frame/source/depth。沿C每次函数/默认值执行的Ok/Err active/globals清理继续执行；终止不重置其它模块状态或重试初始化。函数ID/用户容器/IO/RNG不回滚。
legacy资源错误origins仍None；graph错误真实source/folded traceback沿C，不新增Result/JSON顶层字段。Cancel token由host以store(true,Ordering::Release)置位，poll load(Ordering::Acquire)；Interp不写/reset共享token。不同调用默认互不关联；host明确共享已取消token则各运行在下一节点取消。host不得把运行中的LFZ Value/Env跨线程共享。
deadline为绝对Instant，比较now>=deadline；不用系统日期/时区。各次poll按固定优先级，实际到期与并发置位的观测时刻不确定，不承诺同时事件的全局顺序。
本批只保证stmt/expr边界poll，不新增native内部poll或将其算步；大range/JSON/排序/format及input/readFile等宿主调用可能在下一poll前持续运行。末次native调用完成后若再无节点，可成功返回，即使期间超过deadline或token置位。宿主有需要可在返回后自行判断时钟，但不能改写本次已完成的LFZ结果。不是CPU配额、内存/文件权限沙箱或硬实时超时。禁止强杀解释线程或catch_unwind伪装资源终止；T18的外部进程/HTTP生命周期另定义。

### 22.5 CLI选项（tooling唯一写cli.rs/必要main）
仅执行命令（裸FILE、run FILE、-e CODE、run -e CODE、-、run -）支持：
    --max-steps N
    --timeout-ms N
N必须非空ASCII十进制[0-9]+，允许0和前导0，值在u64范围；拒绝符号、空白、下划线、小数及溢出。只支持独立两token，不支持等号简写。每项最多一次；与--json及彼此可任意选项顺序，允许前/后于执行来源；必须保持现输入来源数量/文件名判断，不增加新文件转义语法。
scanner先保护-e后的紧邻一个原始CODE token，即使CODE恰为--max-steps/--timeout-ms/--/--json也不识别为选项。-e此前的限额选项先摘除再识别CODE位置。已完整执行来源之后的--终止全部选项扫描，之后每个token原样argv；不得因剥除限额值吞掉--。限额参数必须在--之前；缺值/下个token是--视为缺值。未知选项沿旧规则拒绝。重复--json沿旧规则，不受本节每项唯一限制。
以下等价设置，argv均为["--max-steps","9"]：
    lfz --max-steps 10 run --timeout-ms 100 -e "1" -- --max-steps 9
    lfz run -e "1" --timeout-ms 100 --max-steps 10 -- --max-steps 9
check/test/help/version不接受限额选项，不静默忽略（即使原help/version会忽略多余token也先拒限额）；无新限额选项时旧行为不改。check -e CODE内文本仍受保护。解析所有限额、重复及命令适用性在读文件/stdin前完成，无LFZ副作用。

精确新增usage消息（沿现stderr文本及用法尾行，exit2；--json时仍非JSON usage）：
- 缺值：“选项 '{flag}' 需要一个非负十进制 u64 参数”
- 非法/溢出：“选项 '{flag}' 的值必须是非负十进制 u64”
- 重复：“选项 '{flag}' 不能重复”
- 非执行命令：“执行限额选项只适用于运行命令”
- Instant.checked_add失败：“选项 '--timeout-ms' 超出当前平台可表示的时间范围”
flag为完整--max-steps或--timeout-ms。单项遇到即报缺值/非法/重复；多项按argv自左向右，值解析错误优先于最终非执行命令检查。timeout转换Duration::from_millis(N)，成功前端（含静态build_graph）之后、调用eval入口前从Instant::now().checked_add建立deadline；checked_add失败走usage拒绝，无求值副作用。CLI提供的N是求值poll时间窗口，不含root/依赖读、lex/parse、静态build_graph和输出渲染；graph入口内部的宿主结构校验及Interp建立开销包含窗口，结构校验失败仍优先于节点poll。宿主绝对deadline可由host自行提前建立。不新增公开graph验证API或为deadline重复验证整图，不重读源码。
timeout=0且有节点时首poll Deadline；空程序成功。步数+时间+token均启用时按22.3，CLI不提供token开关。Ctrl-C沿OS默认，无跨平台handler/第三方依赖/结构化Cancelled承诺；后续须单独平台ADR。
资源错误exit2，--json仍唯一原形错误对象，class ExecutionLimitError、message固定、line/col/traceback真实；--json下程序输出仍stderr。成功exit0。host GraphExecutionError沿C stderr协议，不伪装LFZ终止或业务成功。

新增flag族等号边界：仅在未受CODE/argv保护的选项区域，token以--max-steps=或--timeout-ms=开头（含等号后为空/任意文本）时，扫描到即usage拒绝，精确消息“选项 '{flag}' 不支持等号写法，请使用 '{flag} N'”。flag是不含等号的完整名称；此错误按token出现顺序，先于最终命令适用性检查；不把等号形式当一次合法赋值或进入重复检查。因而--help --max-steps=1和--version --timeout-ms=1均exit2，不能被旧help/version忽略额外token放行。其它旧help/version额外token规则保持；不扩大匹配到--max-stepsX等其它名称。CODE恰为该文本或完整来源后--的同名argv仍原样保护。

命令拼写裁定（2026-10-30）：本节“help/version”仅指既有合法--help/-h与--version/-V；“check/test”指既有check/test子命令。不新增裸help/version命令别名，也不把它们列入非执行命令。裸help或version继续按既有文件路径/扩展规则处理，附合法限额时亦如此；可读取的同名路径不得因本节变成帮助命令或被“不适用于运行命令”拒绝。不存在/不合扩展名的路径仍走旧IOError，不应把此结果误判为漏拒帮助命令。--help --max-steps 1与--version --timeout-ms 1须报限额不适用；--help --max-steps=1等号形式按前述扫描规则立即报等号usage。其它旧help额外token规则不改。
计步范围再次明确：for语句本身及迭代来源表达式等实际进入的节点仍计步；未执行的空循环体不虚构节点，每次迭代的Rust控制逻辑不额外计步。大量迭代但没有新增stmt/expr入口不能据此承诺小预算终止。CODE中的双负号按既有lexer/parser/求值规则处理，不因与CLI选项相似而新增语法错误；check只检查语法/模块，不动态查名。上述均为既有节点与CODE保护规则的澄清，不改计步范围或新增语法。

### 22.6 Owner与可复验门禁
- architect：三spec §22、T17-design.md、ADR与本人记录。
- core：唯一src/error.rs新增reason/variant/helper和三个reason的class/message/span/Display单元验证，机械更新13类注册；不写evaluator/CLI。
- runtime：唯一src/evaluator.rs新API/预算、节点poll、attempt/跨module/default/fault保护及runtime单元；保持C/value/AST无额外公开字段迁移。除固定接口必须的消费外不写error.rs。
- tooling：唯一CLI/main选项与limits贯通、帮助、usage/JSON进程验证；老test runner和case helper默认unlimited，不给check/test隐式配置。
- test-engineer公开黑盒/应用owner/人AI指南各按宪法边界；verifier独立，perf最后冻结字节无并发测量。

分批A先error+步数/token，B再deadline+CLI；可同冻结契约但源码在C独立PASS后串行。最小验收：上述2/4步精确阈值重复；0与u64::MAX不wrap；空while/for/递归/7HOF在小预算受控，不能用真实栈溢出abort作测例；普通attempt连续业务失败不补步、嵌套终止不吞；模块diamond仅执行一次、初始化总预算、依赖失败entry无后续副作用；graph函数/模板默认的终止定位真实文件及caller清理；已置token/执行中置位/独立与明确共享token；过去/未来deadline与同时事件优先；有限运行结束后不偷偷追加poll。
库并发取消测试用受控worker与宽容watchdog，超时不能join无限卡死主测试或强杀解释线程；先小步数提供有限兜底，但断言reason必须Cancelled。deadline测试预过期确定性+有界到期见证，不断言毫秒级硬响应。CLI四类入口/任意选项顺序/CODE与argv保护/重复非法溢出/check-test-help拒绝/唯一JSON完整回归；旧默认无预算全套差分，graph坏元数据仍执行前拒绝。原19项性能在实际最终hash一次规定窗口比较，unlimited分支不能称零成本；R304保持OPEN，不自动改变原线。
