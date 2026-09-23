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
