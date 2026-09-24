# P3 独立验收报告（LFZ v1 解释器核心）

> 验收人: verifier（模拟助教视角，独立验收）｜ 日期: 2026-09-24
> 验收对象: LFZ 解释器 P3（loader / lexer / ast / parser / value / env / evaluator / builtins / 最小 CLI）
> 验收基线: **git HEAD `682d1fb`**（`feat(p3): parser v1 features (pipe desugaring, dump, interpolation, i64::MIN)`）
> 验收命令依据: 任务书 P3.11（6 条验收命令）+ `docs/spec/{syntax,semantics,interface-contract}.md`（冻结 v1）
> 纪律声明: **只验证、不修复**。本报告未改动任何交付物（`src/**`、`docs/spec/**`、他人 STATUS/JOURNAL 均只读）。新增仅 `docs/reports/P3-verification.md` 与夹具目录 `docs/reports/fixtures-p3/`。

---

## 0. 环境与验收过程说明（重要）

- 平台: Windows（win32）；工具链: `cargo`/`rustc`（`$env:Path += ";$env:USERPROFILE\.cargo\bin"`）。
- **验收期间仓库发生并发写入**：开工时 HEAD = `0ce5123`，且工作树有 **未提交** 的 `src/parser.rs`（+1085 行，即 P3.5：管道/`;;`/插值/`i64::MIN`）。验收过程中该改动被提交为 **`682d1fb`**，同时 `.opencode/team/PROJECT_STATE.md` / `TEAM_BOARD.md` 处于工作树已修改（未提交）状态。本报告全部命令在 **`682d1fb`** 上重跑确认（见 §1、§3），结论对 `682d1fb` 负责。
- **编码说明**：本机 PowerShell 控制台会按本地代码页重编码原生进程输出，捕获日志中的中文可能显示为乱码。已用**字节级校验**确认 CLI 的 stderr 为 **UTF-8**：`CosmosAnswerError` 消息字节尾 `E4BDA0 E5BF98 E8AEB0 E4BA86 E5AE87 E5AE99 E79A84 E7AD94 E6A188` = UTF-8「你忘记了宇宙的答案」。报告中的中文消息据此还原。
- 证据纪律：以下每条结论均由**实际执行**的命令/输出支撑；不采信任何转述（含 team-lead 汇报、代码注释中的"已实现"）。

---

## 1. 验收清单与结论总表

### 1.1 任务书 6 条验收命令

| 命令 | 期望 | 结论 | 证据 |
|---|---|---|---|
| 1) `cargo build`（clean） | 0 warning | **通过** | §2.1 |
| 2) `cargo test` | 全绿（报总数） | **通过** | §2.2（354+8+7 = **369**） |
| 3) `cargo run -- run examples/hello.lfz` | 正确输出、退出码 0 | **通过** | §2.3 |
| 4) 缺 `#42` 的 `.lfz`（UTF-8 无 BOM） | 退出码 2 + `CosmosAnswerError: 你忘记了宇宙的答案` | **通过** | §2.4 |
| 5) 语法错误程序 | 退出码 2 + `SyntaxError: <中文>` + 位置 `line N, col M` | **通过** | §2.5 |
| 6) 运行期错误（未定义名 / `div(1,0)`） | 退出码 2 + `类名: 中文` + `Traceback` 头 | **通过** | §2.6 |

### 1.2 自测夹具（`docs/reports/fixtures-p3/`）

| 夹具 | 覆盖 | 结论 | 证据 |
|---|---|---|---|
| `01_arith.lfz` | 算术/比较/逻辑/字符串运算/`i64::MIN`/精确比较 | **通过** | §2.7 |
| `02_control.lfz` | `if/else`、`while`、`for`(array+struct)、`break/continue` | **通过** | §2.7 |
| `03_functions.lfz` | 函数、递归、闭包 counter | **通过** | §2.7 |
| `04_containers.lfz` | 数组/结构体/字段/下标/动态字段/keys/values/entries/has/深相等 | **通过** | §2.7 |
| `05_pipe.lfz` | 管道 data-last、`_` 占位、链式管道、lambda | **通过** | §2.7 |
| `06_interp.lfz` | 富字符串插值（含 `format_spec`） | **失败** | §2.7 + §3 bug-02 |
| `07_dump.lfz` | `;;` 可见链/遮蔽 | **通过** | §2.7 |
| `08_builtins.lfz` | 内置（54 表中的代表项，data-last/新值语义） | **通过** | §2.7 |
| `spec_9_4_refs.lfz` | `docs/spec/` 样例 §9.4（原样） | **失败** | §2.8 + §3 bug-01 |

### 1.3 对照 `docs/spec/` 三件套的抽查一致性

| # | 检查项 | 结论 | 证据 |
|---|---|---|---|
| C-01 | `ext(path)` 判定（大小写不敏感、无扩展名、`bak`、Windows 盘符） | **通过** | §4.1 |
| C-02 | `#42` 严格性（无变体、必跟行终止符、BOM 仅跳一个） | **通过** | §4.2 |
| C-03 | `line_base` 行号语义（`.lfz` 程序体自 line 2 起） | **通过** | §4.3 |
| C-04 | 12 个错误类与中文消息 | **通过**（除 C-13 单列） | §4.4 |
| C-05 | `ScopeDebug` / `;;` 可见链、遮蔽去重、行格式与通道 | **通过** | §4.5 |
| C-06 | §10.7 内置表 **54/54**（data-last） | **通过** | §4.6 |
| C-07 | `del` 仅数据字段（方法字段 → `FieldError`） | **通过** | §4.7 |
| C-08 | `pop([])` → `Index{idx:-1,len:0}` | **通过** | §4.8 |
| C-09 | `insert` 不接受负索引 | **通过** | §4.8 |
| C-10 | `floor/ceil/round` 的 `NaN`/`±Inf`/超界 | **通过** | §4.9 |
| C-11 | 未闭合块注释 → `SyntaxError` | **通过** | §4.10 |
| C-12 | 管道 data-last 注入、优先级、`_` 绑定范围 | **通过** | §4.11 |
| C-13 | **插值 `format_spec`** | **失败** | §4.12 bug-02 |
| C-14 | `i64::MIN` 规则 | **通过** | §4.13 |
| C-15 | **多行块**（`{` 后换行 / 块首语句） | **失败** | §4.14 bug-01 |
| C-16 | **`if` 表达式形态**（`unary → if_expr`） | **失败** | §4.15 bug-03 |

---

## 2. 逐条详细证据（原文粘贴）

### 2.1 验收命令 1 —— `cargo build` 0 warning

```
PS> cargo clean
     Removed 1155 files, 279.0MiB total
PS> cargo build
   Compiling lfz v0.1.0 (D:\XUE\2026fall\Program Design\lfz-programing language design)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.43s
=== BUILD EXIT CODE: 0 ===
```
**结论：通过**。输出无任何 `warning:`/`error:` 行（clean 全量重编）。

### 2.2 验收命令 2 —— `cargo test` 全绿

```
PS> cargo test
running 354 tests
...
test result: ok. 354 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
=== TEST EXIT: 0 ===
```
**结论：通过**。合计 **369 passed / 0 failed**（354 库 + 8 bin + 7 集成 `tests/cli.rs`），无 warning。与 PROJECT_STATE 自报「369 passed / 0 failed / 0 warnings」一致。

### 2.3 验收命令 3 —— `examples/hello.lfz`

```
PS> cargo run --quiet -- run examples/hello.lfz
Hello, LFZ!
=== EXIT CODE: 0 ===
```
**结论：通过**。

### 2.4 验收命令 4 —— 缺 `#42`（临时目录，UTF-8 无 BOM）

夹具 `nohdr.lfz` 内容 `print("hi")`（`[System.Text.UTF8Encoding]::new($false)` 写入，无 BOM）：
```
PS> lfz.exe run "...\nohdr.lfz"
File "...\nohdr.lfz", line 1
CosmosAnswerError: 你忘记了宇宙的答案
exit=2
```
字节级校验：消息尾部 `…E7AD94 E6A188 0A` = UTF-8「你忘记了宇宙的答案」。
**结论：通过**（退出码 2；无 `Traceback` 头；无源码行/插入符，符合 §8.2/B10）。

### 2.5 验收命令 5 —— 语法错误（`let x = 1 $ 2`）

```
PS> lfz.exe run "...\syntaxerr.lfz"
  File "...\syntaxerr.lfz", line 2
    let x = 1 $ 2
              ^
SyntaxError: 非法字符 '$'
exit=2
```
**结论：通过**。位置 = line 2（`#42` 为 line 1，`line_base=1`），插入符指向 `$`（第 11 列），无 `Traceback` 头。与 `semantics.md` §8.3 示例 1 完全一致。

### 2.6 验收命令 6 —— 运行期错误

```
PS> lfz.exe run "...\divzero.lfz"     # 内容: div(1, 0)
Traceback (most recent call last):
  File "...\divzero.lfz", line 2, in <module>
    div(1, 0)
    ^
ZeroDivisionError: 除以零
exit=2
```
```
PS> lfz.exe run "...\nameerr.lfz"     # 内容: print(undefinedName)
Traceback (most recent call last):
  File "...\nameerr.lfz", line 2, in <module>
    print(undefinedName)
    ^
NameError: 未定义的名字 'undefinedName'
exit=2
```
**结论：通过**（退出码 2 + `类名: 中文` + `Traceback` 头 + `in <module>`）。

### 2.7 自测夹具运行结果

**`01_arith.lfz`（exit 0）**：
```
3
3.5
1
2
-2
-4
14
20
1
false
true
true
ab
ababab
3.5
true
2.0
-9223372036854775808
false
```
（依次 = `1+2`、`7/2` 真除法、`7%3`、`-7%3`、`7%-3`、`div(-7,2)`、`2+3*4`、`(2+3)*4`、`-2+3`、`!true`、`1<2&&3>=3`、`1==1||false`、`"a"+"b"`、`"ab"*3`、`3.0+0.5`、`1==1.0`、`2.0`、`i64::MIN`、`9007199254740993 == 9007199254740992.0`）→ 全部符合 `semantics.md` §4.2/§4.5.6/§4.5.7。

**`02_control.lfz`（exit 0）**：`10 / 1 2 3 4 / a b / big / 1 2 / 1 3`
（`acc=0..4=10`；`for` array 打印 1..4；`for k in {a,b}` 键按字节序 `a b`；`if/else`→`big`；`break`→`1 2`；`continue`→`1 3`）✅

**`03_functions.lfz`（exit 0）**：`120 / 55 / 1 2 3`
（`fact(5)=120`；`fib(10)=55`；闭包 counter 按 cell 捕获 → `1 2 3`）✅

**`04_containers.lfz`（exit 0）**：
```
[10, 2, 3]                                  // a[0]=10 原地写
[10, 20, 3]                                 // let b=a 引用共享，b[1]=20 对 a 可见（A1）
3
3
a
1
{name: "a", score: 1, z: 9}                 // s.k / s["k"] / 动态字段 s.z
true
true
["name", "score", "z"]                      // keys 字节序
["a", 1, 9]
[["name", "a"], ["score", 1], ["z", 9]]
2
true                                        // a == [10,20,3] 深相等
```
✅（引用语义 A1、动态字段、键序 B3、深相等 A6）

**`05_pipe.lfz`（exit 0）**：`4 / 3 / 15 / 15 / 5 / [0, 2, 4, 6, 8] / [1, 2, 3] / [2, 4] / ABC`
（`3|>inc()`=4；`[1,2,3]|>len()`=3；`10|>add(5)`=15 data-last；`10|>add(_,5)`=15；链式=5；`map`、`sort`、`filter`、`upper` 管道）✅

**`06_interp.lfz`（exit 2）—— 失败**：
```
  File "...\06_interp.lfz", line 6
    print("n=${n:05d}")
                ^
SyntaxError: 这里期待 '}'，但得到 ':'
```
纯文本插值（`"hello ${name}"`）可解析；**凡带 `format_spec`（`:` 后）的插值一律解析失败**（详见 bug-02）。

**`07_dump.lfz`（exit 0）**：
```
outer ： 100
x ： 1
f ： <fn f>
x ： 2
outer ： 100
f ： <fn f>
outer ： 100
x ： 1
f ： <fn f>
```
（第 1 段 = 模块 `;;`，声明序 `outer, x, f`；第 2 段 = `if` 块内 `;;` → 内层 `x:2` 优先、外层 `x` 被遮蔽去重；第 3 段 = 退出块后再次 `;;`）✅ 与 `semantics.md` §3.6（内→外、遮蔽去重）一致。
分隔符字节校验：`61 62 20 EF BC 9A 20 31 0A` = `ab` + `U+0020` + `U+FF1A`(全角冒号) + `U+0020` + `1` + `\n` ✅

**`08_builtins.lfz`（exit 0）**：58 行输出，逐项符合 §10.7，摘录：
```
3 / 5 / [0, 1, 2, 3] / [] / [1, 2, 3, 4] / [1, 2] / [1, 2] / [1, 9, 2, 3] / [3, 2, 1] / [2, 3]
1 / 3 / 6 / 6.5 / [1, 2] / [3] / [1, 2, 3] / [3, 2, 1] / 6 / 3 / 3 / [2, 3, 4] / [2, 3]
42 / 1.0 / [1, "a"] / 42 / 2 / 1.5 / int / array
3 / 3.0 / 3 / 4 / 2 / 4 / 2.0 / nan / 1024.0 / 3
hi / ABC / abc / bbb / ababab / (空行) / true
["a", "b", "c"] / ["a", "b", "c"] / a-b-c / true / {j: 2} / ["a", "b"] / [2, 1] / [["a", 2], ["b", 1]]
7
8
```
要点：`len("héllo")=5`（Unicode 标量）；`range(-2)=[]`；容器更新内置返回**新值**（`push/pop/sort` 等不改原数组）；`sum([1,2.5,3])=6.5`；`round(2.5)=2`、`round(3.5)=4`（banker's）；`sqrt(-1)=nan`；`str(1.0)="1.0"`；`int(2.9)=2`（向零）；`keys/values/entries` 按字节序；`del/keys` 数据面。

### 2.8 `docs/spec/` 样例 §9.4 原样运行（失败）

夹具 `spec_9_4_refs.lfz` = `syntax.md` §9.4 样例逐字复制：
```
  File "...\spec_9_4_refs.lfz", line 2
    // refs.lfz — v0.5 语义演示
                           ^
SyntaxError: 表达式未结束：行尾不能终止表达式；请用括号跨行
exit=2
```
**结论：失败**（详见 bug-01：块首/程序首的换行未跳过）。即使去掉首行注释，紧跟的 `fn makeCounter() {` 亦在 `{` 后换行处报同一错误（见 §4.14）。

---

## 3. 缺陷单汇总

> 每条附最小复现（命令 + 期望 vs 实际）。严重度：🔴 阻塞（影响评分项/语言不可用）/ 🟡 非阻塞（功能可用但与契约不符）/ 🟢 建议。

### 【缺陷单】bug-20260924-01 —— 多行块解析失败 🔴
- **交付物**: `src/parser.rs`（`parse_stmt_seq` / `parse_block`）
- **摘要**: 块 `{` 之后（或语句序列开头）直接出现 `NEWLINE` 时，解析器把 `NEWLINE` 当作"期望语句/表达式"处理，抛 `IncompleteExpr`。**所有多行函数体/`if`/`while`/`for` 体、以及程序体首个空行或注释行都无法解析**，使惯用多行 LFZ 程序（含 spec 全部样例）不可运行。
- **最小复现（命令）**:
  ```
  #42
  fn f() {
    print(1)
  }
  f()
  ```
  → `lfz run block.lfz`
- **期望**: 输出 `1`，退出码 0。
- **实际**: 退出码 2；
  ```
    File "...\block.lfz", line 2
      fn f() {
              ^
  SyntaxError: 表达式未结束：行尾不能终止表达式；请用括号跨行
  ```
- **根因定位**: `parse_stmt_seq`（`src/parser.rs:250`）的 `match` 无 `TokenKind::Newline` 分支：块内/程序首遇到 `NEWLINE` 落入 `_ => self.parse_stmt()`，`parse_stmt` 走到 `parse_expr`，`primary` 在 `TokenKind::Newline` 处抛 `IncompleteExpr`（`src/parser.rs:1125`）。`parse_block`（`:603`）在 `expect(LBrace)` 后未 `skip_newlines()`，`parse_program` 亦然。
- **影响**: 阻断评分项 1（解释器）；应用（评分项 5，≥200 行）、黑盒测试集（评分项 2）、文档示例均无法用惯用多行语法编写。
- **为何 369 单测未发现**: 解析器单测**手工构造 token 流**，无一处把 `NEWLINE` 紧跟 `LBrace`（唯一"`{` 后换行"的用例是 struct 成员表 `IGN` 模式，换行被忽略）；evaluator 的端到端 `eval_src` 仅 8 处且均为单行源码。**建议 owner 补充"块首换行/程序首换行"回归用例。**
- **建议 owner**: core-dev

### 【缺陷单】bug-20260924-02 —— 插值 `format_spec` 解析失败 🔴
- **交付物**: `src/parser.rs`（`parse_string`，与 `src/lexer.rs` 记号口径不一致）
- **摘要**: lexer 在 `${expr:spec}` 处先发 `Colon` 再发 `FormatSpec(text)`（见 `lexer.rs:349-354` 及其单测 `format_spec_is_raw_until_brace`）；而 parser 的 `parse_string`（`parser.rs:1153`）只匹配 `TokenKind::FormatSpec`，**未消费中间的 `Colon`**，故 `expect(InterpEnd)` 失败。
- **最小复现（命令）**:
  ```
  #42
  print("${1:>3}")
  ```
  → `lfz run fmt.lfz`
- **期望**: 输出 `  1`（宽度 3 右对齐），退出码 0。
- **实际**: 退出码 2；
  ```
    File "...\fmt.lfz", line 2
      print("${1:>3}")
                ^
  SyntaxError: 这里期待 '}'，但得到 ':'
  ```
- **影响**: 富字符串插值（特色 3）的格式说明符完全不可用；spec §9.1/§9.2 样例（`:>3`、`:.2f`、`:05d`、`:x`）全部失败。阻断评分项 1。
- **为何未发现**: `parser.rs` 单测（`:4862`/`:4887`）手工构造 token 流时**直接塞 `FormatSpec` 而省略 `Colon`**，绕过 lexer；evaluator `eval_src` 未覆盖 `format_spec`；lexer 单测与 parser 单测各自通过，二者契约未联合校验。
- **建议 owner**: core-dev

### 【缺陷单】bug-20260924-03 —— `if` 表达式形态未实现 🔴
- **交付物**: `src/parser.rs`（`unary` 层缺 `if_expr`）
- **摘要**: `if` 仅支持**语句形态**（`StmtKind::If`）；`syntax.md` §7 `unary = … | if_expr | lambda` 要求的**表达式位置 `if`** 未实现（parser 头注释第 57-58 行自述为"后续批次"的已知简化）。`if` 不能作实参、不能作 `let` 初始化式。
- **最小复现（命令）**:
  ```
  #42
  let x = if true { 1 } else { 2 }
  print(x)
  ```
  → `lfz run ifexpr.lfz`
- **期望**: 输出 `1`，退出码 0。
- **实际**: 退出码 2；
  ```
    File "...\ifexpr.lfz", line 2
      let x = if true { 1 } else { 2 }
              ^
  SyntaxError: 这里期待 表达式，但得到 'if'
  ```
- **影响**: 违反"万物皆值"（§1）与 §7 文法；`syntax.md` §9.1 样例 `let star = if s.isTop() { "★" } else { "·" }` 无法运行。阻断评分项 1。
- **建议 owner**: core-dev

### 【缺陷单】bug-20260924-04 —— 运行期错误的 traceback 位置过期（指向模块首语句）🟡
- **交付物**: `src/evaluator.rs`（帧 `span` 更新策略）
- **摘要**: 当错误发生在**非 Call 表达式**（如字段/下标访问、作实参求值时），模块帧 `span` 未被更新为"当前求值的最小节点"，而是停留在**模块首个节点**，导致 `File … line N` 与源码行/插入符指向错误位置（§8.2 要求指向引发错误的最小 AST 节点）。
- **最小复现（命令）**:
  ```
  #42
  let s = { a: 1 }
  s.missing
  ```
  → `lfz run h_field1.lfz`
- **期望**: 位置 = line 3（`s.missing`），`FieldError: 结构体没有字段 'missing'`。
- **实际**: 位置 = **line 2**（`let s = { a: 1 }`）；
  ```
  Traceback (most recent call last):
    File "...\h_field1.lfz", line 2, in <module>
      let s = { a: 1 }
      ^
  FieldError: 结构体没有字段 'missing'
  ```
  （同类：`let a=[1,2]` / `print(a[9])` → 报 line 2；`print("before")` / `let s=…` / `print(s.missing)` → 亦报 line 2）
- **影响**: 结构化错误的**位置信息不可信**（评分项 1 的"Python 风格报错"体验），但不影响错误类/消息与退出码。
- **建议 owner**: runtime-dev

### 【缺陷单】bug-20260924-05 —— 管道右侧非函数的消息不符 🟡
- **交付物**: `src/evaluator.rs` / `src/builtins.rs`（调用错误消息）
- **摘要**: 管道脱糖为 `Call` 后，RHS 非函数时报通用"调用非函数"消息，而非 `semantics.md` §8.1 `TypeError` 细分表中"管道右侧非函数 → `管道右侧必须是函数，得到 {t}`"。
- **最小复现（命令）**:
  ```
  #42
  print(3 |> 5)
  ```
- **期望**: `TypeError: 管道右侧必须是函数，得到 int`。
- **实际**: `TypeError: 不可调用：int 不是函数`（类正确、消息不符）。
- **影响**: 与契约消息模板不一致（黑盒测试若逐字符比对将失败）。非阻塞。
- **建议 owner**: runtime-dev

### 【缺陷单】bug-20260924-06 —— `let` 重绑定未被限制 🟡
- **交付物**: `src/evaluator.rs` / `src/env.rs`
- **摘要**: `semantics.md` §4.5.2 规定"`let` 只锁重绑定，不锁内容"（`a = []` 对 `let` 非法）。实际 `let a = 1` 后 `a = 2` 被允许。
- **最小复现（命令）**:
  ```
  #42
  let a = 1
  a = 2
  print(a)
  ```
- **期望**: 重绑定 `let` 应报错（**注**：spec §8.1 的 12 类中未定义"重绑定 `let`"的专属错误类，属契约缺口，需 language-architect 澄清应属哪一类）。
- **实际**: 退出码 0，输出 `2`（无任何错误）。
- **影响**: `let`/`var` 语义区分失效。非阻塞（但需 spec 澄清）。
- **建议 owner**: runtime-dev（+ language-architect 澄清错误类）

### 【缺陷单】bug-20260924-07 —— 语句首 `{` 未按 A9 作匿名 struct 字面量 🟡
- **交付物**: `src/parser.rs`（`parse_stmt_seq` 的 `LBrace` 分支；parser 头注释第 54-56 行自述为已知简化）
- **摘要**: A9 规定"语句首 `{` 恒为匿名 struct 字面量"。实际被当作**裸块语句内联**，且 AST 无 `Block` 语句变体。
- **最小复现（命令）**:
  ```
  #42
  { "k": 1 }
  ```
- **期望**: 按 A9 为匿名 struct 字面量表达式（或被明确拒绝为非语句）。
- **实际**: 退出码 2，`SyntaxError: 语句之间必须有换行`（把 `"k": 1` 当块内语句）。
- **影响**: 与 A9/§3.3 不符。非阻塞（惯用代码少见）。
- **建议 owner**: core-dev

### 【缺陷单】bug-20260924-08 —— `.self` 字段访问被拒（spec 样例自相冲突）🟢
- **交付物**: `docs/spec/syntax.md`（§9.4 样例）与 `src/parser.rs`（`field = "." IDENT`）
- **摘要**: `syntax.md` §9.4 样例使用 `r.self = r`；但 EBNF `field = "." , IDENT` 且 `self` 是保留关键字（§2.6），故 `.self` 被拒绝。
- **最小复现（命令）**:
  ```
  #42
  let r = {}
  r.self = r
  print(r)
  ```
- **期望**（按 §9.4 样例）: `{self: <cycle>}` / `true`。
- **实际**: 退出码 2，`SyntaxError: 这里期待 字段名，但得到 'self'`。
  - 改用非关键字字段 `r.me = r` 则正确：输出 `{me: <cycle>}` 与 `true`（A6 环安全实现无误）。
- **影响**: 规则与样例冲突，须 language-architect 裁决（改样例或允许 `.self`）。非阻塞。
- **建议 owner**: language-architect（spec）/ core-dev（若裁决允许）

### 【缺陷单】bug-20260924-09 —— `RecursionError` 的 traceback 输出量巨大（~10000 帧）🟢
- **交付物**: `src/cli.rs` / `src/evaluator.rs`（traceback 序列化）
- **摘要**: 深递归触发 `RecursionError` 时，按 §10.3"把帧栈最外层→最内层全部序列化"输出**上万帧**（每帧 3 行），单次运行产生数十万行 stderr。
- **最小复现（命令）**: `#42` + `fn r(n) { r(n + 1) }` + `print(r(0))`
- **实际**: 最终行 `RecursionError: 递归深度超限（超过 10000 层）`，退出码 2；但此前打印约 10000 帧 × 3 行。
- **影响**: 契约未规定截断，故**非违规**；但实用性差（演示/验收易被海量输出淹没）。建议加"省略 N 帧"折叠（需先 ADR/改 spec）。
- **建议 owner**: runtime-dev + language-architect

**缺陷统计**：🔴 阻塞 **3**（bug-01/02/03）｜🟡 非阻塞 **4**（bug-04/05/06/07）｜🟢 建议 **2**（bug-08/09）。
**过程性问题**（非缺陷单，报告 team-lead）：验收期间工作树含未提交的 `PROJECT_STATE.md`/`TEAM_BOARD.md`；`src/parser.rs`（P3.5）在验收中途才提交（`682d1fb`）——发布纪律由 release-manager 处置。

---

## 4. 对照 spec 的检查明细（逐项）

### 4.1 `ext(path)` 判定 —— 通过
| 路径 | 期望 | 实际 |
|---|---|---|
| `upper.LFZ`（无 `#42`） | 视为 `.lfz`（大小写不敏感）→ 报错 | 退出码 2 `CosmosAnswerError` ✅ |
| `ok.LFZ`（含 `#42`） | 正常运行 | 退出码 0，输出 `1` ✅ |
| `plain.txt`（无 `#42`） | 非 `.lfz` → 豁免 | 退出码 0，输出 `1` ✅ |
| `bak.lfz.bak`（无 `#42`） | ext=`bak` → 豁免 | 退出码 0，输出 `1` ✅ |
| `noext`（无 `#42`） | 无扩展名 → 豁免 | 退出码 0，输出 `1` ✅ |

### 4.2 `#42` 严格性 —— 通过
`# 42`、`#42 `（尾随空格）、`#43`、`#42abc`、`##42`、`\n#42`（前置空行）、3 字节 `#42`（无行终止符）→ 全部退出码 2 + `CosmosAnswerError`；`#42\n`（空程序体）→ 退出码 0、无输出 ✅（符合 §2.2/B1–B5）。
BOM：`<BOM>#42\n…` → 运行；`<BOM>\n#42…` → `CosmosAnswerError` ✅（§B4）。
行终止符：`#42\r\n…`、`#42\r…` → 均运行（归一化）✅（§B7）。
非 UTF-8：`#42\n\xFF\xFE\n` → `SyntaxError: 文件不是合法的 UTF-8 编码（首个非法字节位于字节偏移 4）`，**先编码后前导** ✅（§B8）。

### 4.3 `line_base` 行号 —— 通过
`.lfz` 程序体首行（本地 1）报错显示 `line 2`（例：`syntaxerr.lfz`、`divzero.lfz`）；`CosmosAnswerError` 固定 `line 1`。✅（§2.2.2/§10.1）

### 4.4 12 个错误类与中文消息 —— 通过
逐类触发（退出码均 2）：
| 类 | 触发用例 | 实际消息 |
|---|---|---|
| `CosmosAnswerError` | 缺 `#42` | `你忘记了宇宙的答案` ✅ |
| `SyntaxError` | `let x = 1 $ 2` | `非法字符 '$'` ✅ |
| `NameError` | `print(undefined)` | `未定义的名字 'undefined'` ✅ |
| `TypeError` | `1 + "a"` | `运算符 '+' 不支持 int 与 string` ✅ |
| `IndexError` | `[1,2][-5]` | `下标 -5 越界（长度 2）` ✅ |
| `FieldError` | `s.missing` | `结构体没有字段 'missing'` ✅ |
| `ZeroDivisionError` | `5 % 0` | `对零取模` ✅ |
| `OverflowError` | `i64::MAX + 1` | `整数溢出：结果超出 i64 范围` ✅ |
| `ValueError` | `int("abc")` | `无法把 string 转换为 int（'abc'）` ✅ |
| `IOError` | `input()` 遇 EOF | `输入结束（EOF）` ✅ |
| `AssertionError` | `assert(1==2)` / `fail("x")` | `断言失败` / `x` ✅ |
| `RecursionError` | 深递归 | `递归深度超限（超过 10000 层）` ✅ |
（另：`if 1 {…}` → `条件必须是 bool，得到 int`；`filter((x)=>x,[1])` → 同；`int("abc")`/`float("xyz")` → `无法把 {src} 转换为 {dst}（'{text}'）`）
**唯一例外**：管道右侧非函数消息见 bug-05（类正确、消息不符）。

### 4.5 `;;` 可见链与遮蔽 —— 通过
见 §2.7 `07_dump.lfz`：内→外、同层声明序、**遮蔽去重**（内层 `x` 只打印一次）、通道为 **stdout**、空链输出零行（`#42\n;;` → 无输出、退出码 0）。分隔符 = `U+0020 U+FF1A U+0020`（字节验证）✅（§3.6/§10.5）

### 4.6 §10.7 内置表 54/54（data-last）—— 通过
- `src/builtins.rs::BUILTIN_NAMES` 恰含 **54** 名（`TABLE` 47 非高阶 + `HOF_TABLE` 7 高阶），与 spec §10.7 清单逐名一致；库内单测 `lookup_table_names_consistency`（断言 `len==54` 且三表一致）在 369 全绿中通过。
- 运行期抽查（§2.7 `08_builtins.lfz`）：`len/range/push/pop/removeAt/insert/swap/slice/min/max/sum/take/drop/sort/sortBy/map/filter/reduce/minBy/maxBy/each/keys/values/entries/has/del/split/join/trim/upper/lower/replace/repeat/startsWith/abs/floor/ceil/round/sqrt/pow/div/str/int/float/type/print/eprint/assert/check/fail/randInt` 等行为均符合签名与返回类型；`seed` 确定性验证：`seed(12345)` 两次 `rand()` 相等 → `true`。
- data-last 与"返回新值不改原容器"：`[3,1,2] |> sort()` 返回 `[1,2,3]` 且原 `xs` 仍 `[3,1,2]`（§2.7 `f_funnew`）✅

### 4.7 `del` 仅数据字段 —— 通过
`struct S { x: 0, fn m() => self.x }`：`has("m", s)=false`、`has("x", s)=true`、`len(s)=1`、`keys(s)=["x"]`、`print(s)={x: 5}`（方法不入数据面）；`del("m", s)` → `FieldError: 结构体没有字段 'm'` ✅（§4.5.9/§10.7）

### 4.8 `pop([])` 与 `insert` 负索引 —— 通过
- `pop([])` → `IndexError: 下标 -1 越界（长度 0）`（= `Index{idx:-1,len:0}`）✅
- `insert(-1, 9, [1,2,3])` → `IndexError: 下标 -1 越界（长度 3）`；`insert(5, 9, [1,2,3])` → `下标 5 越界（长度 3）`（不接受负索引）✅（§10.7 补钉）

### 4.9 `floor/ceil/round` 的 `NaN`/`±Inf`/超界 —— 通过
- `floor(float("nan"))` → `ValueError: 无法把 float 转换为 int（'nan'）` ✅
- `ceil(float("inf"))` → `OverflowError: 整数溢出：结果超出 i64 范围` ✅
- `round(1e30)` → `OverflowError: 整数溢出：结果超出 i64 范围` ✅（§4.5.7 与 §10.7 补钉）

### 4.10 未闭合块注释 —— 通过
`#42\n/* not closed\n` → `SyntaxError: 块注释在此处未闭合（缺少 '*/'）`，插入符指向 `/*` 的 `/` ✅（§2.4/§10.6）

### 4.11 管道 data-last / 优先级 / `_` 绑定 —— 通过
- `1 + 2 |> f`（f=×10）→ `30`（`|>` 比 `+` 松）✅；`[1,2,3] |> sum() > 4` → `true`（`|>` 比比较紧）✅
- `[1,2,3] |> sum() + 1` → `SyntaxError`（`|>` RHS 仅收 `pipe_rhs`，A2/§4.4）✅
- `10 |> add(5)`（无 `_`）= `add(5,10)`=15；`10 |> add(_, 5)`（一个 `_` 原位）=15 ✅
- `[1,2] |> map((x) => x + _)` → `SyntaxError: 占位符 '_' 只能出现在管道右侧的调用实参中`（λ 体内非法，M4）✅

### 4.12 插值 `format_spec` —— **失败**
见 §2.7 `06_interp.lfz` 与 bug-02：`"${1:>3}"` 等一律 `SyntaxError: 这里期待 '}'，但得到 ':'`。纯文本插值可用。

### 4.13 `i64::MIN` 规则 —— 通过
`-9223372036854775808` → 输出 `-9223372036854775808`（合法）；`9223372036854775808` → `SyntaxError: 整数字面量超出 i64 范围`；`print(1.)` 边界见 §4.14？（`.5` → `SyntaxError: 这里期待 表达式，但得到 '.'`，A23）✅（§10.8 B12）

### 4.14 多行块 —— **失败**
`fn f() {\n  print(1)\n}\nf()`、`if true {\n  print(7)\n}`、以及无注释版 §9.4，均在 `{` 后换行处报 `SyntaxError: 表达式未结束：行尾不能终止表达式；请用括号跨行`；块为**首行空行/注释**亦同（`#42\n\nprint(1)`、`#42\n// c\nprint(1)`、`#42\n/* b */\nprint(1)`）。单行块 `fn f() { print(1) }` 正常。见 bug-01。

### 4.15 `if` 表达式 —— **失败**（bug-03）
`let x = if true { 1 } else { 2 }` → `SyntaxError: 这里期待 表达式，但得到 'if'`；语句形态 `if true { print(1) }` 正常。

### 4.16 其他通过项（补充抽查）
- 结构体方法 / `self`：`struct P { x: 0, fn dbl() => self.x * 2 }` + `P{x:5}.dbl()` → `10` ✅
- 引用语义 A1：`let b=a; b[1]=20` 对 `a` 可见 ✅；内置返回新值 ✅
- 环安全 A6：`r.me=r; print(r)` → `{me: <cycle>}`；`r==r` → `true` ✅
- 数组/实参跨行（括号内 IGN）：`[\n1,\n2,\n]`、`print(\n1+2\n)` 均正常 ✅
- 数值字面量：`0xFF=255`、`0b1010=10`、`0o755=493`、`1_000=1000`、`1e10=10000000000.0`、`2.5e-3=0.0025` ✅
- 注释：`//` 恒为行注释（`print(7 / 2) // c` → `3.5`）✅

---

## 5. 回归记录

| 缺陷 | 首次发现 | 修复状态 | 复验 |
|---|---|---|---|
| bug-20260924-01 | 2026-09-24（本报告） | 未修复 | 待复验 |
| bug-20260924-02 | 2026-09-24（本报告） | 未修复 | 待复验 |
| bug-20260924-03 | 2026-09-24（本报告） | 未修复 | 待复验 |
| bug-20260924-04 | 2026-09-24（本报告） | 未修复 | 待复验 |
| bug-20260924-05 | 2026-09-24（本报告） | 未修复 | 待复验 |
| bug-20260924-06 | 2026-09-24（本报告） | 未修复 | 待复验 |
| bug-20260924-07 | 2026-09-24（本报告） | 未修复 | 待复验 |
| bug-20260924-08 | 2026-09-24（本报告） | 未修复 | 待复验 |
| bug-20260924-09 | 2026-09-24（本报告） | 未修复 | 待复验 |

> 本报告是 P3 的首次独立验收，无历史缺陷。缺陷修复后由 verifier 复现原用例并在本表更新为"已修复/仍失败"，最终报告此表即"质量问题闭环"证据。

---

## 6. 总体结论

**问题计数**：🔴 阻塞 **3** ｜ 🟡 非阻塞 **4** ｜ 🟢 建议 **2**（合计 9）。

**3 条最重要发现**：
1. **🔴 bug-01 多行块不可解析**——`{` 后换行/块首语句即 `IncompleteExpr`，使惯用多行函数体/控制流体全部失败（spec §9 全部样例、应用、黑盒测试集均无法编写）。这是解释器"端到端可用"论断的最强反例。
2. **🔴 bug-02 插值 `format_spec` 不可解析**——lexer 发 `Colon`+`FormatSpec`、parser 只认 `FormatSpec`，特色 3（富插值）的格式说明符完全失效。
3. **🔴 bug-03 `if` 不能作表达式**——违反"万物皆值"与 §7 `unary → if_expr`，§9.1 样例无法运行。

**加分/达标项**：`cargo build` 0 warning、`cargo test` 369 全绿、5 条错误/运行命令（3–6）全部符合预期；loader（ext/`#42`/BOM/行终止符/UTF-8）、`line_base`、12 错误类与中文消息、`;;` 可见链与遮蔽、§10.7 **54/54**、`del` 数据面、`pop([])`/`insert` 边界、`floor/ceil/round` 边界、未闭合块注释、管道 data-last 与优先级、`i64::MIN` 等**均通过**——实现质量在可解析的语法子集内相当高。

**但**：3 项 🔴 阻塞直接命中评分项 1（解释器）且波及评分项 2/5，与"P3 实现全部完成、可打 `v0.2.0`"不符。

> **【验收结论】FAIL（阻塞项：bug-20260924-01 多行块解析、bug-20260924-02 插值 format_spec、bug-20260924-03 `if` 表达式；均须 core-dev 修复并补回归用例，修复后由 verifier 复验；**不得**打 `v0.2.0` 标签。）**

*（报告完。夹具与原始输出见 `docs/reports/fixtures-p3/`。）*

---
---

# 复验（rev.2）

> 复验人: verifier（模拟助教视角，独立验收）｜ 复验日期: **2026-09-24**
> 复验提交: **`1e8fd5f`**（`fix(p3): blocking defects + runtime error spans`）＋ **`05e42d9`**（`docs(spec): rule on 3 P3.11 gaps`，规范裁定，无代码）
> 复验时仓库 **HEAD = `05e42d9`**（工作树 clean；`05e42d9` 仅改 `docs/spec/**` 与 `.opencode/team/**`，生产代码状态 = `1e8fd5f`）
> 依据: 上一轮 §6【验收结论】FAIL 的 3×🔴 + 4×🟡 + 2×🟢；本轮仅复验**已声明修复项**并区分「解释器缺陷 / 规范缺陷 / 夹具缺陷」。
> 纪律声明: **只验证、不修复**。本轮**未改** `src/**`、**未改** `docs/spec/**`、未改他人交付物；新增仅 `docs/reports/P3-verification.md` 本节与夹具目录 `docs/reports/fixtures-p3-rev2/`。
> 环境: Windows（win32）；`$env:Path += ";$env:USERPROFILE\.cargo\bin"`；所有命令行均为 `cargo run --quiet -- run <file>`（真实 CLI 全链路，非单测）。

## 7.1 复验基线与两条总命令

**`cargo build`（clean 全量重编）** —— 通过：
```
PS> cargo clean
     Removed 1157 files, 279.1MiB total
PS> cargo build
   Compiling lfz v0.1.0 (D:\XUE\2026fall\Program Design\lfz-programing language design)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.94s
=== BUILD EXIT: 0 ===
warning:/error: 命中数 = 0
```
**结论：通过**（0 warning / 0 error）。

**`cargo test`** —— 通过：
```
running 358 tests
test result: ok. 358 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
=== TEST EXIT: 0 ===
```
**结论：通过**。合计 **373 passed / 0 failed**（358 lib + 8 bin + 7 `tests/cli.rs`）= rev.1 的 369 **+4**，与「新增 4 个经 lexer 的真实源码回归用例」一致（见 §7.6）。

## 7.2 逐项复验（原 🔴 三项 + 原 🟡 两项）

### 7.2.1 bug-20260924-01（🔴 多行块）—— **已修复**
修复声明：`parse_stmt_seq` 现于序列开头 `skip_newlines()`。已核 `git show 1e8fd5f -- src/parser.rs`：`parse_stmt_seq` 循环体首行新增 `self.skip_newlines();`，并附注释（`bug-01 修复`）。

**最小复现**（`docs/reports/fixtures-p3-rev2/bug01_multiline.lfz`）：
```
#42
fn f() {
  print(1)
}
f()
```
```
PS> cargo run --quiet -- run docs\reports\fixtures-p3-rev2\bug01_multiline.lfz
1
=== EXIT: 0 ===
```
**期望 `1` / 退出码 0 → 实测一致**。

**补强抽查**（独立，非仅最小复现）：
| 用例 | 文件 | 期望 | 实测 |
|---|---|---|---|
| 程序体首行注释 | `bug01_leading_comment.lfz` | `42` / 0 | `42` / 0 ✅ |
| 程序体首行空行 + 块首注释 | `bug01_leading_blank_block.lfz` | `7` / 0 | `7` / 0 ✅ |

**结论：已修复**（`{` 后换行、块首/程序首空行或注释行均可解析）。

### 7.2.2 bug-20260924-02（🔴 插值 `format_spec`）—— **已修复**
修复声明：`parse_string` 先消费 `Colon` 再取 `FormatSpec`。已核 diff：新增 `if *self.peek() == TokenKind::Colon { self.bump(); ... }` 分支。

**最小复现**（`bug02_format_spec.lfz`：`print("${1:>3}")`）：
```
PS> cargo run --quiet -- run docs\reports\fixtures-p3-rev2\bug02_format_spec.lfz
  1
=== EXIT: 0 ===
```
**期望 `  1`（宽 3 右对齐）/ 退出码 0 → 实测一致**。

**更广格式说明符**：夹具 `06_interp.lfz` 现整体通过（rev.1 为失败），实测输出：
```
hello LFZ
n=00042
hex=2a HEX=2A oct=52 bin=101010
pi=3.142
right=    42 left=42     center=  42  
sign=+42
s=LFZ
esc ${n}
```
（`:05d`、`:x`/`:X`/`:o`/`:b`、`:.3f`、`:>6`/`:<6`/`:^6`、`:+d`、`:s`、`\${n}` 转义均正确。）
**结论：已修复**。

### 7.2.3 bug-20260924-03（🔴 `if` 作表达式）—— **已修复**
修复声明：`parse_unary` 新增 `TokenKind::KwIf => self.parse_if_expr(span)`。已核 diff：`unary` 层确新增该分支。

**最小复现**（`bug03_if_expr.lfz`）：
```
#42
let x = if true { 1 } else { 2 }
print(x)
```
```
PS> cargo run --quiet -- run docs\reports\fixtures-p3-rev2\bug03_if_expr.lfz
1
=== EXIT: 0 ===
```
**期望 `1` / 退出码 0 → 实测一致**。修复侧另有 `print(if false { 1 } else { 2 })`（`if` 作实参）经 lexer 回归用例（见 §7.6）。
**结论：已修复**。

### 7.2.4 bug-20260924-04（🟡 traceback 位置过期）—— **已修复**
修复声明：新增 `note_error_span`，在 `eval_expr`/`exec_stmt` 出错时把当前帧 `span` 覆盖为错误 `span`。已核 diff：`eval_expr`/`exec_stmt` 改为委托 `*_inner` 并在 `Err` 分支 `note_error_span`。

**最小复现**（`bug04_span.lfz`：第 2 行 `let s = { a: 1 }`、第 3 行 `s.missing`）：
```
PS> cargo run --quiet -- run docs\reports\fixtures-p3-rev2\bug04_span.lfz
Traceback (most recent call last):
  File "docs\reports\fixtures-p3-rev2\bug04_span.lfz", line 3, in <module>
    s.missing
    ^
FieldError: 结构体没有字段 'missing'
=== EXIT: 2 ===
```
**期望 位置 = line 3（`s.missing`）→ 实测 line 3，插入符指向 `s`（`s.missing` 起始）**。消息字节级校验：`FieldError: ` 后 `E7 BB 93 E6 9E 84 E4 BD 93 E6 B2 A1 E6 9C 89 E5 AD 97 E6 AE B5` = 结构体没有字段 ✅。

**回归抽查**（确认覆盖策略未把别处位置改坏）：
| 用例 | 期望位置 | 实测 |
|---|---|---|
| `div(1, 0)`（line 2） | line 2 | line 2 ✅ |
| `print(s.missing)`（line 3，**作实参**） | line 3 | line 3 ✅ |
| `print(undefinedName)`（line 2） | line 2 | line 2 ✅ |

**结论：已修复**（且未观察到位置回归）。

### 7.2.5 bug-20260924-05（🟡 管道右侧非函数消息不符）—— **已修复**
修复声明：`call_func` 依据 `is_pipe_desugared` 改报 `PipeRhsNotFunction`。已核 diff：新增 `is_pipe_desugared` / `non_function_call_msg`，`call_func` 增 `piped` 形参。

**最小复现**（`bug05_pipe_rhs.lfz`：`print(3 |> 5)`）：
```
PS> cargo run --quiet -- run docs\reports\fixtures-p3-rev2\bug05_pipe_rhs.lfz
Traceback (most recent call last):
  File "docs\reports\fixtures-p3-rev2\bug05_pipe_rhs.lfz", line 2, in <module>
    print(3 |> 5)
          ^
TypeError: 管道右侧必须是函数，得到 int
=== EXIT: 2 ===
```
**期望 `TypeError: 管道右侧必须是函数，得到 int` → 实测一致**。消息字节级校验：`TypeError: ` 后 `E7 AE A1 E9 81 93 E5 8F B3 E4 BE A7 E5 BF 85 E9 A1 BB E6 98 AF E5 87 BD E6 95 B0 EF BC 8C E5 BE 97 E5 88 B0` = `管道右侧必须是函数，得到`，尾 ` int` ✅。（插入符指向管道脱糖 `Call.span` = 左操作数 `3`，属该项设计，不在原缺陷范围。）
**结论：已修复**。

## 7.3 残留项独立确认（未修复，逐条复核"声称"）

> 对「bug-07 未修 / bug-06·08·09 规范裁定未落地代码」——**不采信声明**，逐条实测。

| 项 | 声称 | 实测命令与结果 | 与声称一致? |
|---|---|---|---|
| bug-06 `let` 重绑定 | 规范已裁定（`05e42d9`），**代码未落地** | `bug06_let_rebind.lfz`：`let a = 1` / `a = 2` / `print(a)` → **EXIT=0，输出 `2`**（无 `TypeError`）| ✅ 未落地 |
| bug-07 语句首 `{` | **有意推迟**，仍按裸块内联 | `bug07_stmt_brace.lfz`：`{ "k": 1 }` → **EXIT=2**，`SyntaxError: 语句之间必须有换行`（插入符指向 `"k"`）| ✅ 未修 |
| bug-08 `.self` | 规范改样例（`r.self`→`r.me`），**实现本就正确** | `05e42d9` diff 确认 §9.4 样例与预期输出已改 `me`；`interface-contract.md` §10.6 记 `.self → SyntaxError` 为**正确行为**；实测 `spec_9_4_refs_fixed.lfz` 用 `r.me` → `{me: <cycle>}` / `true` ✅ | ✅ 已由规范侧闭合 |
| bug-09 `RecursionError` 巨量帧 | 规范已裁定折叠（K=10/M=30/阈值 40，落点 `cli.rs`，tooling-dev），**代码未落地** | `bug09_recursion.lfz`：深递归 → **EXIT=2**，stderr **共 30005 行**（约 10000 帧 ×3），末行 `RecursionError: 递归深度超限（超过 10000 层）`；**未见** `  ... 省略 N 帧...` 折叠行 | ✅ 未落地 |

> 结论：四项残留状态与修复侧声明**完全一致**；其中 bug-06/09 已从「实现疑点」转为「**规范已定、待落地的实现 backlog**」，bug-08 属**规范侧闭合（实现无误）**，bug-07 为**有意推迟**。四项均为**非阻塞**（不使解释器不可用、不影响 P3 核心评分项）。

## 7.4 夹具复跑（`docs/reports/fixtures-p3/`，全部 9 份）

| 夹具 | rev.1 | rev.2（本次） | 说明 |
|---|---|---|---|
| `01_arith.lfz` | 通过 | **通过**（exit 0，19 行输出不变） | — |
| `02_control.lfz` | 通过 | **通过**（exit 0） | — |
| `03_functions.lfz` | 通过 | **通过**（exit 0：`120 / 55 / 1 2 3`） | — |
| `04_containers.lfz` | 通过 | **通过**（exit 0，14 行输出不变） | — |
| `05_pipe.lfz` | 通过 | **通过**（exit 0：`4 / 3 / 15 / 15 / 5 / [0,2,4,6,8] / [1,2,3] / [2,4] / ABC`） | — |
| `06_interp.lfz` | **失败**（bug-02） | **通过**（exit 0，10 行富插值输出全对） | bug-02 修复直接收益 |
| `07_dump.lfz` | 通过 | **通过**（exit 0，`outer ： 100 …` 掩蔽去重不变） | 分隔符 `U+0020 U+FF1A U+0020` 保持 |
| `08_builtins.lfz` | 通过 | **通过**（exit 0，58 行输出不变） | — |
| `spec_9_4_refs.lfz` | **失败**（归因 bug-01） | **失败**（**原因改变**，见下） | **夹具/规范冲突，非解释器缺陷** |

**`spec_9_4_refs.lfz` 的失败性质（重要区分）**：
```
PS> cargo run --quiet -- run docs\reports\fixtures-p3\spec_9_4_refs.lfz
  File "...\spec_9_4_refs.lfz", line 5
      fn inc() { n += 1; n }          // 闭包按 cell 捕获 n（A2）
                       ^
SyntaxError: 语句之间必须有换行
=== EXIT: 2 ===
```
- rev.1 时该夹具在 **line 2**（首行注释）即撞上 bug-01（`IncompleteExpr`）；**bug-01 修复后**它已能解析到 **line 5**，却因 §9.4 样例**自身使用单个 `;`** 而在 line 5 报 `SyntaxError`。
- 该 `;` 触发的是 `syntax.md` **A11**（第 373 行）：「单个 `;` 词法合法（`SEMI`）但**文法从不接受** → `SyntaxError`」。**解释器行为与 A11 一致（正确）**。
- **判定：这是夹具（逐字复制自规范 §9.4）与规范自身的冲突 → 夹具/规范问题，不是解释器缺陷。**（rev.1 曾把它并入 bug-01 结论，此处予以修正与区分。）
- **反证**：我另建 `spec_9_4_refs_fixed.lfz`（仅两处自洽化：① 把 `n += 1; n` 改为换行分割；② 依 §9.4 v1 补钉把 `r.self` 改为 `r.me`），实测**退出码 0**，输出与 §9.4「预期输出」**逐行一致**：
  ```
  1 2 3
  99 99
  [3, 1, 2]  [1, 2, 3]
  {me: <cycle>}
  true
  ```
  → 证明**解释器本身正确**，失败根因在样例文本。

## 7.5 【规范侧发现】spec-20260924-01 —— `syntax.md` §9.4 样例自相矛盾（使用单个 `;`）🟡

- **交付物**: `docs/spec/syntax.md`（**规范**，非代码）
- **摘要**: `syntax.md` §9.4 样例第 **787 行** 写作 `fn inc() { n += 1; n }`，用**单个 `;`** 分隔两条语句；但同一规范 **A11（第 373 行）** 明定「单个 `;` 词法合法（`SEMI`）但**文法从不接受** → `SyntaxError`（提示改用 `;;`）」。**样例与规则自相矛盾**。
- **最小复现步骤**:
  1. `docs/reports/fixtures-p3/spec_9_4_refs.lfz`（= §9.4 逐字复制）
  2. `cargo run --quiet -- run docs\reports\fixtures-p3\spec_9_4_refs.lfz`
- **期望（按 A11）**: 该样例**不应**包含单个 `;`；应能以换行分隔正常解析并输出 §9.4「预期输出」。
- **实际**: 退出码 2；
  ```
    File "...\spec_9_4_refs.lfz", line 5
      fn inc() { n += 1; n }          // 闭包按 cell 捕获 n（A2）
                       ^
  SyntaxError: 语句之间必须有换行
  ```
- **影响**: ① 规范**唯一的引用语义/闭包/环安全综合样例**按原文**无法运行**——文档示例与规则冲突会被评分项 4（语法说明 20 分）与人工复核直接扣分；② 直接导致我的夹具 `spec_9_4_refs.lfz` 失败（夹具逐字复制样例）；③ 会误导 test-engineer（若把 §9.4 当黑盒用例，将得到与规范矛盾的期望）。
- **严重度**: 🟡 非阻塞（**规范文档缺陷**，非解释器缺陷；解释器按 A11 行为正确）。
- **建议 owner**: **language-architect**（改 §9.4 样例：`n += 1; n` → 两行 `n += 1` / `n`；或明确 A11 例外——后者须改冻结点 A11，不推荐；建议改样例）。
- **关联遗留**: rev.1 夹具 `spec_9_4_refs.lfz` 的失败**归因于本发现 + 旧 bug-01**；bug-01 已修，本发现取代其为该夹具失败的当前根因（且属规范侧，非实现侧）。

## 7.6 修复质量旁证（不替代本报告结论，仅交叉印证）

- `git show 1e8fd5f` 显示：`src/parser.rs` +95/−? 、`src/evaluator.rs` +87/−?；四处声明的代码改动均**真实存在**（非仅注释）。
- **新增回归用例为"经 lexer 的真实源码"**：diff 中新增 `fn parse_src(body) { ... crate::lexer::lex(body, 1)?; crate::parser::parse(&toks) }`，四个用例 `reg_multi_line_blocks_parse` / `reg_leading_blank_and_comment_lines_parse` / `reg_interp_format_spec_through_lexer` / `reg_if_as_expression_through_lexer` 均**先 lex 再 parse**（修复了 rev.1 指出的"手工构造 token 流绕过 lexer"盲区）。
- 两处旧 parser 单测（`interp_with_format_spec_some` / `interp_empty_format_spec_is_some_empty`）已补 `Colon`——与修复声明一致。
- 计数吻合：lib 测试 354→**358**（+4），总 369→**373**。
- **无回归**：rev.1 通过的 5 条验收命令相关项（`cargo build` 0 warning、`hello.lfz`→`Hello, LFZ!` exit 0、无 `#42`→`CosmosAnswerError: 你忘记了宇宙的答案` line 1 exit 2、`div(1,0)`/`undefinedName` 位置与消息）本次复跑**全部保持**。

## 7.7 回归记录（更新版）

| 缺陷 | 首次发现 | rev.1 | **rev.2（本次复验）** | 复验证据 |
|---|---|---|---|---|
| bug-20260924-01 多行块 🔴 | 2026-09-24 | 未修复 | **已修复** | §7.2.1（exit 0）|
| bug-20260924-02 插值 format_spec 🔴 | 2026-09-24 | 未修复 | **已修复** | §7.2.2（exit 0）|
| bug-20260924-03 `if` 表达式 🔴 | 2026-09-24 | 未修复 | **已修复** | §7.2.3（exit 0）|
| bug-20260924-04 traceback 位置 🟡 | 2026-09-24 | 未修复 | **已修复** | §7.2.4（line 3）|
| bug-20260924-05 管道右侧消息 🟡 | 2026-09-24 | 未修复 | **已修复** | §7.2.5（消息逐字节一致）|
| bug-20260924-06 `let` 重绑定 🟡 | 2026-09-24 | 未修复 | **仍失败**（规范已裁定，代码未落地）| §7.3（exit 0/输出 `2`）|
| bug-20260924-07 语句首 `{` 🟡 | 2026-09-24 | 未修复 | **仍失败**（有意推迟，待 A9 确认）| §7.3（SyntaxError）|
| bug-20260924-08 `.self` 🟢 | 2026-09-24 | 未修复 | **已闭合（规范侧）**（改样例 `r.me`；实现无误）| §7.3（`{me: <cycle>}`）|
| bug-20260924-09 RecursionError 巨量帧 🟢 | 2026-09-24 | 未修复 | **仍失败**（规范已裁定折叠，代码未落地）| §7.3（30005 行）|
| **spec-20260924-01 §9.4 样例含 `;`**（新） | 2026-09-24 | — | **新发现（规范侧）** | §7.5 |

**本轮新发现数**：**1**（spec-20260924-01，规范侧 🟡，owner: language-architect）。
**原 3×🔴 是否全修**：**是，3/3 全修**（且原 🟡 bug-04/05 亦全修）。
**是否有残留/回归**：**无阻塞残留、无回归**；非阻塞残留 3 项（bug-06/07/09）+ 新增规范侧 1 项（spec-01）。

## 7.8 复验总结论

- ✅ **原 3×🔴 全部修复**（多行块 / 插值 `format_spec` / `if` 表达式），并经真实 CLI 端到端复现通过。
- ✅ **原 🟡 bug-04 / bug-05 亦已修复**（traceback 位置、管道右侧消息），位置与消息经字节级校验通过。
- ✅ `cargo build` 0 warning；`cargo test` **373 passed / 0 failed**（+4 真实源码回归用例）；修复未引入回归。
- ⚠️ **非阻塞残留**：bug-06（`let` 重绑定，规范已定 `TypeError`，待 core-dev/runtime-dev 落地）、bug-07（语句首 `{`，有意推迟）、bug-09（traceback 折叠，规范已定，待 tooling-dev 在 `cli.rs` 落地）；bug-08 已由规范侧闭合。
- ⚠️ **新发现（规范侧）**：`syntax.md` §9.4 样例使用单个 `;`，与 A11 自相矛盾（owner: language-architect）；它同时解释了我的夹具 `spec_9_4_refs.lfz` 为何失败——**属夹具/规范问题，非解释器缺陷**。

> **【复验结论】CONCERNS（列非阻塞）——可推进 `v0.2.0`。**
> 3×🔴 与 2×🟡 修复项**全部复验通过**，已无阻塞项，解释器核心端到端可用；但仍有**非阻塞遗留**（bug-06/bug-07/bug-09 的实现落地）与**一项规范侧缺陷**（spec-20260924-01，`docs/spec/syntax.md` §9.4 样例自相矛盾）需在后续阶段（P4/P7 前）闭合。**建议：`v0.2.0` 可打；上述非阻塞项与规范侧发现转交对应 owner（language-architect / core-dev / runtime-dev / tooling-dev），由 verifier 后续抽验闭环。**

*（复验（rev.2）完。本轮夹具与捕获输出见 `docs/reports/fixtures-p3-rev2/`。）*
