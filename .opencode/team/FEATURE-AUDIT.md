# LFZ 语言特性缺口审计 + v1.1 候选新增特性提案（FEATURE-AUDIT.md）

> 作者: language-architect ｜ 日期: 2026-09-27 ｜ 状态: **审计与提案（未改 `docs/spec/`）**
> 性质: 本文件是**提案**，不是事实源。**用户拍板后**才由 language-architect 写 ADR + 改 `docs/spec/`。
> 触发: `lfz-programming` skill 盲测（8 个零上下文 agent，只读 skill 各写一个复杂程序）**2/8 通过**。
> 判据（用户给定，**必须同时满足**）：**① 便于 agent 书写**（为"编程 Agent"服务）；**② 性能**（不得因易用性引入隐藏的 O(n²) 等代价）。

---

## 0. 结论先行（TL;DR）

1. **盲测 2/8 的根因主要在文档（skill），不在语言。** 7 条暴露项中 **5 条是纯 skill 缺口**（`range` 只有名字没签名、`let` 循环体语义未说明、`<` 左对齐未列出、`len(string)` 未说明、链式下标赋值/`else if`/多 `${}`/零参 `print()` 未展示），**只有 2 条涉及语言**（字符串不可下标未警告；格式说明符无动态宽度）。→ **P0 全部落在文档侧；语言侧最高只到 P1。**
2. **语言侧 v1.1 只做"低风险、高频、复杂度可证"的补强**：`range(lo,hi)`、字符串方法族、文件 IO、`ord`/`chr`、少量 math、`contains`，外加一条 spec 补钉（字符串 `[]` 明确报错）。
3. **明确拒绝一切引入新控制流/新类型/隐藏 O(n²) 的大特性**：`try`/`catch`、标签 `break`、`match`、生成器/惰性流、一等区间 `..`、Unicode 大小写折叠、格式动态宽度、可选链。理由逐条见 §4。
4. **性能红线（本审计核心发现）**：LFZ 字符串是 `Rc<String>` UTF-8 不可变（§4.5.11）。**任何"按标量下标取字符"若以 `chars().nth(i)` 实现 → 循环遍历即 O(n²)**；现有 `split("", s)` 才是 O(n) 正解。同理，**循环里 `s = s + c` 拼接是隐藏 O(n²)**，需 doc 引导 + runtime 侧优化（§3.C1 / §7）。
5. **一句话总建议**：**"先修 skill，再谈加特性"**——skill 用 7 处改动即可把盲测通过率从 2/8 拉起来（零语言风险）；语言侧仅在 v1.1 纳入 7 项低风险补强，其余一律 OUT。

---

## 1. 方法、输入与一手证据

### 1.1 输入（全部已读）

| # | 输入 | 用途 |
|---|---|---|
| 1 | `.opencode/team/BRAINSTORM.md` | v1 IN / v1.1 backlog 原始划分 |
| 2 | `.opencode/team/DECISIONS.md` D-007（v1 = 核心 + 5 特色，11 项延后）、D-011、D-016 | 冻结范围与纪律 |
| 3 | `docs/spec/{syntax,semantics,interface-contract}.md`（910/415/311 行） | 现状：类型/运算符/语句/内置 54/错误模型/§10.7 内置表 |
| 4 | 盲测现场：`hashmap_chain.lfz`(311) / `json_mini.lfz`(309) / `expr_eval.lfz`(209) / `knapsack_dp.report.md`（**逐条缺口清单**）/ `maze_bfs.report.md` / 测试者探针 `t2_sindex.lfz`、`t7_short.lfz` 等 | 缺口归因与复现 |
| 5 | `.opencode/skills/lfz-programming/SKILL.md`（554 行，**只读**） | 核对 7 条缺口的准确表述 |

### 1.2 解释器探针（一手证据，只读运行 `lfz.exe`，未改任何项目文件）

命令形态：`& lfz.exe run <probe>.lfz`（`--json` 取干净消息）。探针文件位于临时目录，不入库。

| 探针 | 关键输入 | 实测结果 | 结论 |
|---|---|---|---|
| `probe_str` | `s="abc"`；`len(s)`；`split("",s)`；`s[0]` | `len=3`；`cs=["a","b","c"]`；`cs[0]=a`；**`s[0]` → `TypeError: 运算符 '[]' 不支持 string 与 array / struct`（exit 2）** | **字符串不可下标**；`split("",s)` 惯用法可用；**该错误消息不在 spec 的 `TypeError` 细分表内 → spec 未定义此行为** |
| `probe_range` | `range(3)`；`range(1,4)` | `r1=[0,1,2]`；**`range(1,4)` → `TypeError: 函数 range 期待 1 个参数，得到 2`** | `range` 仅 **1 参** |
| `probe_letloop` | `while i<3 { let x = i*2 }` | 输出 `0/2/4`，exit 0 | **循环体内 `let` 每轮是新绑定，合法**（与 A2 一致） |
| `probe_misc` | `print()`；`else if`；`"${"ab":<5}"`；`t[1][0]=7` | 空行 / `other` / `[ab   ]` / `[[0, 0], [7, 0]]`，exit 0 | `<` 左对齐**可用**；`else if`、链式下标赋值、零参 `print()` **均可用** |
| `probe_width` | `"${"ab":>5}"`；`"${"ab":>w}"` | 前者 `[   ab]`；后者 **`ValueError: 格式说明符非法：'>w'`** | 固定宽度支持；**动态宽度不支持** |
| `probe_short` | `len([])>0 && xs[0]==1` | `false`，exit 0，无越界 | **`&&` 短路**（spec §4.5.1 正确） |
| `probe_ord` | `ord("a")` | **`NameError: 未定义的名字 'ord'`** | 无 `ord`/`chr` |

> 全部探针与盲测报告结论一致；**探针把"skill 说没写"与"语言真没有"两件事分开了**——这正是本审计的关键方法。

---

## 2. 现状盘点：LFZ v1 已实现特性清单

> 来源：`docs/spec/` 三件套（冻结 v1，D-016）；括号内为规范锚点。

### 2.1 类型 / 值模型

- 标量（**值语义**）：`int`（i64，越界即错）、`float`（f64，IEEE）、`bool`、`nil`、`string`（**不可变**，§4.5.11）。
- 引用类型（**引用语义**，A1/§4.5.2）：`array`、`struct` 实例；赋值/传参共享同一容器；`a[i]=v`/`s.k=v` 原地改，其余内置返回新值。
- `struct` = **合一结构**：字典面孔 `s["k"]` + 对象面孔 `s.k` + 方法（函数值字段），同源存储；模板 `struct Name{...}`；实例可动态加字段；无原型链（B7）。
- `function`：命名函数 / 闭包 / λ；**一等值**；相等仅按同一性。
- **无 truthiness、无隐式转换**（唯一例外 `int→float` 加宽，§4.5.7）；键序恒为 UTF-8 字节序升序（B3）。
- 环安全：`==` 身份优先 + 访问对集合（A6）；显示重复容器输出 `<cycle>`（§3.7）。

### 2.2 字面量

`int`（十/`0x`/`0b`/`0o`，允许 `_`）、`float`（含指数式，`1.` 非法）、`string`（**仅双引号**，`${}` 插值）、`true`/`false`/`nil`、`array [..]`、`struct`（匿名 `{..}` / 具名 `Name{..}`）、函数字面量 / λ。

### 2.3 声明与绑定

`let`（**不可重绑定**，内容可改）、`var`（可重绑定）；`fn` 声明；`struct` 声明（含方法）；`self`。动态字段、模板实例化平拷贝（每次求值默认值，B7）。

### 2.4 运算符（§4.1 优先级 高→低，全左结合）

`() [] .`（postfix）> 一元 `- !` > `* / %` > `+ -` > **`|>`** > `< <= > >=` > `== !=` > `&&` > `||`；赋值 `= += -= *= /= %=`。
类型化语义（§4.2）：`+`（int/float/string 连接）、`-`、`*`（数值 + `string * int` 重复）、`/`（**恒 float 真除法**）、`%`（Python 取模）、`div(a,b)`（内置，向下取整）；比较（数值数学精确、string 字节序）；`&&`/`||` 短路且两侧须 `bool`。

### 2.5 语句 / 控制流 / 块

表达式语句；`if / else / else if`（**if 是表达式**，可赋值）；`while`；`for x in <array|struct>`（array 元素快照 B2，struct 键快照 B3）；`return / break / continue`；`;;` 变量 dump（stdout，内→外，遮蔽去重，§3.6）；块 `{}`（**无裸块语句**，A9）；`=>` 后 `{` 恒为块（M3）。
作用域 = 词法作用域；闭包**按 cell 引用捕获**（A2）；**循环每轮 `let`/`var` 是新绑定**（§4.5.0 / §4.5.3）。

### 2.6 函数 / 闭包

`fn name(params){...}` / `fn name(params) => expr` / `fn(params){...}` / `(params) => expr|block`；块体末表达式即返回值；递归深度上限 10000 → `RecursionError`（B4/A7）。

### 2.7 内置函数（§10.7，共 54 个，data-last，容器更新返回新值）

- **核心/数组（21）**：`len range push pop removeAt insert swap slice min max sum minBy maxBy sort sortBy map filter reduce take drop each`
- **struct/字典（5）**：`keys values entries has del`
- **字符串（8）**：`split join trim upper lower replace repeat startsWith`
- **数学/转换/IO/断言（20）**：`abs floor ceil round sqrt pow div rand randInt seed str int float type print eprint input assert check fail`
- **`sqrt`/`pow` 之外的 math（`sin`/`log`/`exp` 等）v1 不提供**（§10.7 注）；`upper`/`lower` 仅 ASCII（Unicode 折叠 → v1.1）。

### 2.8 IO / 入口 / 文件

`print`/`eprint`（stdout/stderr）、`input(prompt?)`（EOF → `IOError`）；**无文件 IO、无 stdin 管道、无 REPL、无 `-e`**（CLI v1 仅 `lfz run <file>` / `lfz <file>` / `lfz test`）。
`.lfz` 文件首行须恰好 `#42`（按扩展名触发，大小写不敏感；BOM 豁免；B1–B12）；可移植三种行终止符。

### 2.9 错误模型（§8）

12 具体类 + 基类 `LfzError`：`CosmosAnswerError SyntaxError NameError TypeError IndexError FieldError ZeroDivisionError OverflowError ValueError IOError AssertionError RecursionError`；**中文消息 + 位置 + traceback**（运行期 10 类带 `Traceback` 头，加载/解析期 2 类不带）；`--json` 类名字段；退出码 `0` 成功 / `1` 测试用例失败 / `2` 错误；`check` 非致命（stderr + false + 不影响退出码），`assert`/`fail` 致命。

### 2.10 五大特色（D-007）

① 管道 `|>` + `_` 占位；② 合一 struct；③ 富字符串插值 + 格式说明符；④ 结构化错误 + `assert`/`check`；⑤ 确定性语义「无魔法」。

### 2.11 v1 明确未提供（= v1.1 backlog 或 spec 标注）

- D-007 延后的 11 项：生成器/惰性流、`match`、值式错误处理、**一等区间/切片**、值式循环 + 标签 `break`、`lfz check`、REPL、`lfz fmt/doc/init/package/watch`、**模块**、类/继承、类型注解。
- spec 标注：**Unicode 大小写折叠**（§10.7 `upper`/`lower`）、**`sin`/`log`/`exp`**（§10.7 注）、**`hint`/提示句**（§8.1，v1 明确不输出）。

---

## 3. 候选卡片（按优先级排序）

> 卡片格式：**① 现状 ② agent 易用性论证（不给会怎样——引盲测证据） ③ 性能影响（标注隐藏 O(n)/O(n²)） ④ 实现成本与风险（模块 / A1–A7 / §4.5 / 445 单测 / 85 黑盒） ⑤ 建议（IN/OUT + P0/P1/P2）**。

---

### C1 · 字符串按字符访问 `s[i]`

- **① 现状**：**无**。spec 未定义字符串下标；实测 `s[0]` → `TypeError: 运算符 '[]' 不支持 string 与 array / struct`（该消息**不在** spec 的 `TypeError` 细分表中，属未定义行为）；间接替代品 `split(sep, s)`（`sep=""` 时按字符切分，§10.7）**已存在**。
- **② 易用性**：写**解析器/哈希表/词法器**必踩。盲测 `hashmap_chain.lfz` 直接写 `ALPHABET[mid]`（L29）、`key[i]`（L49）→ 失败；其注释自陈"**LFZ 没有 `ord()`**，改用字符串比较实现"。`json_mini.lfz`/`expr_eval.lfz` 则**改用 `split("", src)` + `charAt` sentinel 绕开**。→ gap 在"**skill 从未警告字符串不可下标**"（反而用 `[1,2][5]` 举 IndexError 例子，且该例子本身是数组，语义误导）。
- **③ 性能（核心分析）**：**若新增 `s[i]`，几乎必然是隐藏 O(n²) 陷阱**。理由：LFZ `string = Rc<String>`（UTF-8 变长），按 **Unicode 标量**取第 i 个字符的通用实现是 `chars().nth(i)` = **O(i)**；一个 `for i in range(len(s)) { s[i] }` 的遍历 → **O(n²)**。要让它 O(1) 只能把 string 改成 `Vec<char>`/码点数组 ⇒ **放弃廉价 `Rc` clone、ASCII 内存 ×4、颠覆全运行时值模型**（代价远超收益）。**反观 `split("", s)`：一次 O(n)** 产出 n 个单字符 string，之后索引数组 O(1) ⇒ 遍历总 O(n)。
- **④ 成本/风险**：新增 `s[i]` 会触及 evaluator 索引语义 + 值模型 + 全套字符串/索引测试；且**必然**与"不可变 + `Rc<String>`"打架。**风险高、收益负**。
- **⑤ 建议**：**OUT `s[i]`**（永不纳入，除非将来重构值模型）；**但必须做两件事**：
  - **spec 补钉（P1）**：在 §4.2 明确 `[]` **仅适用于 `array`/`struct`**；字符串下标 → `TypeError`（采用现 impl 消息 `运算符 '[]' 不支持 string 与 array / struct`，或归一为 `运算符 '[]' 不支持 string 与 {rt}`），消除"未定义行为"。
  - **skill 补钉（P0）**：给出**推荐惯用法**（复杂度逐条标注，见 §5.a）。

---

### C2 · `range(lo, hi)`（第二重载）

- **① 现状**：`range(n) -> [0..n)`，`n<0→[]`（§10.7）；**仅 1 参**（探针 `range(1,4)` → `TypeError 期待 1 个参数`）。BRAINSTORM #8「一等区间 + 切片」仍在 v1.1 backlog（D-007 延后）。
- **② 易用性**：盲测两份报告**同时**把"`range` 只列名、无签名"列为**第一痛点**：`knapsack_dp.report.md`「这是盲测中最大的"想用却用不了"的功能」；`maze_bfs.report.md`「**完全不用 `range`**，所有下标循环用 `while` 手写」。缺 `range(lo,hi)` 时，`for i in range(1, n)` 这类**极常见**写法要退化为 `var i=1 / while i<n / ... / i+=1`，代码膨胀、易错。
- **③ 性能**：**中性/可证**。`range(n)` O(n) 时间 + O(n) 内存（**物化 array**，非惰性迭代器）；`range(lo,hi)` = O(hi−lo) 同量级。**不引入新的复杂度阶**。⚠️ 已知代价：大 n 会物化大数组（既有，非本特性引入）——doc 应提醒"超大循环优先 `while`"。
- **④ 成本/风险**：仅改 §10.7 `range` 行 + `builtins.rs` 分支 + 少量单测；**不触及文法**（无新记号）、**不影响 A1–A7**（仍返回新数组）、**不改 §4.5 确定性**。既有 445 单测中若有"`range` 二元报错"断言需改 1 条；85 黑盒不受影响（增量为可选新用例）。
- **⑤ 建议**：**IN，P1**；**3 参 `range(lo,hi,step)` 一并 OUT**（见 §4）。最小规范草案见 §5.b。

---

### C3 · 字符串方法族补强（`indexOf` / `endsWith` / `padEnd` / `padStart` / `substring`）

- **① 现状**：字符串仅 8 个内置（`split join trim upper lower replace repeat startsWith`）。**无 `indexOf`/`find`、无 `endsWith`、无 `pad*`、无子串提取**；`slice` 仅接受 array。
- **② 易用性**：解析器需要"找分隔符位置/取子串"；做**对齐表格**需要"补空格到固定宽度"——盲测 `knapsack_dp.report.md` 缺口 #4 明确：「**格式说明符只给了 4 种、无左对齐**……做"对齐 ASCII 网格"时列宽只能固定……`padr` **本可以是一个 `<` 格式符**」，且其 `padr` 是手写补空格。`indexOf` 缺失会让"定位子串"退化为手写 `split`+`len` 循环。
- **③ 性能**：均为 **O(n)** 级别：`padEnd/padStart` O(输出长度)、`substring(from,to,s)` O(子串长)、`endsWith` O(前缀长)；`indexOf(sub,s)` 朴素实现 **最坏 O(n·m)**（n=主串、m=子串；**须在 doc 标注**，不得宣称 O(n)）；若用高效算法可 O(n+m)。**不引入隐藏 O(n²)**（只要不把它们放进字符串拼接循环——见 §7）。
- **④ 成本/风险**：纯内置新增，不改文法、不动 A1–A7、不影响既有测试（可加性）；string 不可变 ⇒ 全部返回新串，语义干净。与 §4.2「无隐式转换」无冲突（参数类型不符 → `TypeError`）。
- **⑤ 建议**：**IN，P1**；**`padEnd`/`padStart` 直接替代"格式动态宽度"**（见 C9/§5.d）。`charAt` **OUT**（`split("",s)` 已覆盖）。

---

### C4 · 文件 IO（`readFile` / `writeFile` / `appendFile`）

- **① 现状**：**无**。仅 `print`/`eprint`/`input`（stdout/stderr）。`IOError` 错误类已存在（`无法读取：{path}`）。
- **② 易用性**：**"读数据→处理→写结果"是真实程序最常见形态**（文本/CSV/配置/日志）。当前 agent 只能硬编码数据，或依赖 `input()`（CLI v1 无 stdin 管道）。盲测 5 题恰好都是**纯计算**题故未暴露；但一旦任务是文件处理（评分项 5 的"真实应用"很可能需要），**当前无法完成**。→ 判为"agent 写真实程序**必然需要**"。
- **③ 性能**：O(文件大小)；一次性读写，无隐藏复杂度。
- **④ 成本/风险**：低——Rust `std::fs` 封装为 3 个内置即可；错误走既有 `IOError`。**风险**：① 引入**外部环境依赖**（沙箱/路径/编码），与 `input()` 同类的"环境敏感"；不影响 A1–A7 与 §4.5 求值确定性（外部 IO 本就不在确定性语义讨论内）；② 黑盒测试需临时目录与清理，须与 test-engineer 约定夹具纪律。
- **⑤ 建议**：**IN，P1**。最小规范：`readFile(path) -> string`（失败 → `IOError: 无法读取：{path}`；UTF-8，非法编码 → `SyntaxError`？——**草案建议**：非 UTF-8 → `ValueError` 或 `IOError`，见 §5 待定）；`writeFile(path, contents) -> nil`；`appendFile(path, contents) -> nil`。**不做**目录遍历/二进制模式（OUT）。

---

### C5 · `ord(c)` / `chr(n)`

- **① 现状**：**无**（探针 `ord("a")` → `NameError`）。`hashmap_chain.lfz` 自陈"LFZ 没有 `ord()`"，被迫用**对 64 字符字母表做二分查找**来求字符序号。
- **② 易用性**：中高——**哈希/编码/字符分类/进制转换**高频需要"字符 ↔ 码点"。盲测 `hashmap_chain` 明确因此扭曲了实现（用字母表二分代替码点，且**仍因 `s[i]` 缺失而失败**）。配合 C1 的 `split("",s)`，`map((c)=>ord(c), split("",s))` 即可得码点序列。
- **③ 性能**：**O(1)**，纯函数，无隐藏代价。`ord` 要求实参恰为 1 个 Unicode 标量（否则 `ValueError`）；`chr` 接受有效码点（否则 `ValueError`/`OverflowError`）。
- **④ 成本/风险**：极低——2 个内置；不改文法、不动 A1–A7、不影响既有测试。类型边界（代理对/超范围）须在 spec 钉死，避免"未定义"。
- **⑤ 建议**：**IN，P2**（成本极低、价值中等，故非 P1）。

---

### C6 · math 补强（`sin` / `cos` / `log` / `exp`）

- **① 现状**：**v1 不提供**（§10.7 注：`sqrt`/`pow` 之外不提供）；BRAINSTORM #6（分形/仿真类应用）需要。
- **② 易用性**：中——数学/几何/仿真/统计类程序常见；BRAINSTORM 应用候选（Mandelbrot、Sierpinski、图灵机）直接依赖。当前任务（排序可视化）不需要，但"agent 写真实程序"通用性上值得。
- **③ 性能**：**O(1)**（f64 硬件指令），无隐藏代价；`log(负数)` → `NaN`、`exp(大数)` → `+Inf`，与既有 `sqrt`/`pow` 的 IEEE 口径一致（§4.5.6）。
- **④ 成本/风险**：极低；**沿用 §10.7「数值内置形参加宽」**（`int` 实参先加宽为 `float`，同 `sqrt`/`pow`）；不改文法、不动 A1–A7。须在 spec 补一行即可，避免"加宽未声明"。
- **⑤ 建议**：**IN，P2**（最小集 `sin cos log exp`；`tan`/`log2`/`log10`/`atan2` 等 OUT，按需再加）。

---

### C7 · `contains(v, xs)`（数组成员测试）

- **① 现状**：**无**。struct 有 `has(k,s)`，但 **array 无成员测试**，字符串也无 `contains`；`any`/`indexOf` 均无。
- **② 易用性**：中——"在不在集合里"极常见（图遍历 visited 判定、去重、字符集判断）。当前只能手写 `while` 或 `filter(...) |> len(...) > 0`（后者会物化中间数组、写起来别扭）。盲测 `maze_bfs` 手写 `visited` 数组查找。
- **③ 性能**：**O(n) 线性扫描**（必须在 doc 明说，避免 agent 以为 O(1) 而放进循环 ⇒ O(n²)）。若加的是 `in` **运算符**则会与现有 `in`（`for … in`）关键字冲突/需新优先级层，成本上升（见 §4）。
- **④ 成本/风险**：作为**内置 `contains`**：极低（1 个内置，data-last：`contains(v, xs)->bool`）；不改文法、不动 A1–A7。作为 **`in` 运算符**：需改 §4.1 优先级表 + EBNF + A21 lvalue 判定，**成本中等**、回归面大。
- **⑤ 建议**：**IN `contains(v, xs)`，P2**；**`in` 运算符 OUT**（语法成本远超收益，§4）。

---

### C8 · `let` 在循环体内的语义

- **① 现状**：**已定义且已实现**。spec §4.5.0「循环中每次迭代的块级 `let`/`var` 是**新绑定**（各自新 cell）」、§4.5.3 重申。探针 `probe_letloop` 实测每轮新绑定、合法。
- **② 易用性**：agent 的困惑**完全来自 skill 未写**（`knapsack_dp.report.md` 缺口 #5：「`let` 在循环体内的作用域未定义……**所有循环局部量一律用 `var` 提升到循环外**」；`maze_bfs.report.md` 缺口 #2 同）。语言无缺陷。
- **③ 性能**：N/A（语义已定，零成本）。
- **④ 成本/风险**：无（不改 spec）。
- **⑤ 建议**：**无需语言改动**；**skill 补钉（P0）**：明确"循环体每轮执行时 `let`/`var` 均产生**当轮新绑定**，闭包捕获当轮 cell；可放心在循环内用 `let`"。裁定见 §5.c。

---

### C9 · 格式说明符 `<` 左对齐 与 动态宽度

- **① 现状**：`<` **已支持且已在 spec**（§2.8 `align = "<" | ">" | "^"`），探针实测 `"${"ab":<5}"` → `[ab   ]`；**动态宽度不支持**（`:>w` → `ValueError 格式说明符非法`）。skill §3 只列了 4 种（`:>6` `:.2f` `:05d` `:x`），**漏了 `<`/`^` 与 fill 字符**。
- **② 易用性**：`knapsack_dp.report.md` 缺口 #4 明确抱怨"无 `<` 左对齐"——**但这是 skill 缺口而非语言缺口**（`<` 一直可用）；动态宽度的需求，用新内置 `padEnd`（C3）或手写 `repeat` 补空格即可满足（`knapsack` 正是手写 `padr` 绕过的）。
- **③ 性能**：固定宽度对齐 O(1) 附加；`padEnd` O(n)；动态宽度若实现需**递归解析 format_spec**（见下）。
- **④ 成本/风险**：动态宽度会**破坏 M6「format_spec 是原始文本、不解析 `{}`」**的契约（lexer 需在 `FORMAT_SPEC` 内再嵌套求值），使 lexer/AST 复杂化、错误面扩大；收益（对齐表格）已被 `<` + `padEnd` 覆盖。
- **⑤ 建议**：**`<`/`>`/`^`/fill：无需语言改动，skill 补钉（P0）**；**动态宽度：OUT**（用 C3 的 `padEnd`/`padStart` 替代）。裁定见 §5.d。

---

### C10 · `len(string)`

- **① 现状**：**已合法且已定义**。§10.7 `len` 行明写"string 的 **Unicode 标量数**"；探针 `len("abc")=3`；盲测 `knapsack_dp` 实测通过。
- **② 易用性**：困惑纯来自 skill 只用 `len(xs)` 语境、未点明 string 合法。
- **③ 性能**：O(n)（统计标量需扫描；**不是** O(1)，doc 须提醒"别在循环里反复对同一长串求 `len`"）。⚠️ 这是**既有**的潜在 O(n²)（若 agent 在循环内 `len(s)` 且 s 很长）——doc 应引导"用 `split("",s)` 后对数组 `len`（O(1)）"。
- **④ 成本/风险**：无。
- **⑤ 建议**：**无需语言改动；skill 补钉（P0）**。裁定见 §5.e。

---

### C11 · 退出码的语境歧义（skill 自相矛盾）

- **① 现状**：**spec 无矛盾**：§8.2「`0` 成功；`1` **测试失败**；所有错误类 → `2`」。skill §0 说"`1` 测试失败（`assert`/`fail`）"，skill §5 又说"`assert` 失败……退出码 `2`"——**两处语境不同、措辞却像冲突**。
- **② 易用性**：`knapsack_dp.report.md` 缺口 #3：「两条互相打架（assert 失败到底是 1 还是 2）……我无法仲裁」。agent 因此**只能保证 assert 全过**。
- **③ 性能**：N/A。
- **④ 成本/风险**：无。
- **⑤ 建议**：**无需语言改动；skill 补钉（P0）**：明确"`lfz run` 下 `assert`/`fail` → `AssertionError` → 退出 **2**；`lfz test` 下断言的用例计失败 → 退出 **1**"。裁定见 §5（并入 §3 文档表）。

---

### C12 · 「可用但未展示」的构造（链式下标赋值 / `else if` / 多 `${}` / 零参 `print()` / `&&` 短路）

- **① 现状**：**全部已实现**。探针：`t[1][0]=7` 合法；`else if` 合法；多插值合法；`print()` 合法；`&&` 短路。
- **② 易用性**：盲测 agent **主动规避**这些"没展示"的写法（`knapsack_dp.report.md` 缺口 #6/#7/#8/#9；`maze_bfs.report.md` 缺口 #3/#4）。成功者自述"**所有可能猜错的地方都在写码前主动规避了**"——即 **skill 的"只用语档内语法"纪律防住了错误，也削掉了能力**。
- **③ 性能**：N/A。
- **④ 成本/风险**：无。
- **⑤ 建议**：**无需语言改动；skill 补钉（P0）**：在语法速查/陷阱中**正面示例**这些构造，并明写 `&&`/`||` **短路**（spec §4.5.1 已定义）。

---

## 4. 明确 OUT 清单（克制也是结论）

> 每条给"① 现状/出处 · ② agent 是否需要 · ③ 性能 · ④ 成本/风险 · ⑤ OUT 理由"。

| ID | 候选 | 出处 | OUT 理由（含性能） |
|---|---|---|---|
| **O1** | **`try`/`catch` 或值式错误处理（`rescue`/`?`/`expect`）** | BRAINSTORM #13 / D-007 延后 | agent 已能用**值式错误记录**建模（`json_mini`/`expr_eval` 的 `{ok,msg,v}` 记录，结构上成功）；引入新控制流要穿透求值器 + 与冻结错误模型/traceback 交互，**成本高、回归面大**。**OUT**（v1.1 不做）。 |
| **O2** | **标签 `break`/`continue`** | BRAINSTORM #15 / D-007 延后 | 嵌套循环逃生可用"标志位 + 辅助函数 `return`"等价实现；标签需 lexer/AST/parser/evaluator 全链改动 + 悬空标签校验，**成本高**。**OUT**。 |
| **O3** | **`match` 模式匹配** | BRAINSTORM #14 / D-007 延后 | 裁剪版也要新文法 + 绑定作用域 + 穷尽性语义；D-007 已判"成本最高"。**OUT**。 |
| **O4** | **生成器 / 惰性流** | BRAINSTORM #16 / D-007 延后 | 依赖运行时长出惰性迭代协议，颠覆"array 是唯一序列"的现有模型；`range` 物化属既有设计。**OUT**。 |
| **O5** | **一等区间 / 切片 `..`/`..<`/`..step..`** | BRAINSTORM #8 / D-007 延后 | `slice(from,to,xs)` 已覆盖数组切片；`range(lo,hi)`（C2）覆盖整数区间；新增 `..` 记号 + 区间类型要动 §2.3 最长匹配表 + §4.1 优先级 + EBNF，收益被既有内置覆盖。**OUT**（`..` 记号同时会与 A23"不存在 `..`"冲突）。 |
| **O6** | **`in` 成员运算符** | 架构师候选 | 用内置 `contains`（C7）零文法成本达成；`in` 运算符要新优先级层 + 与 `for … in` 关键字共存 + A21 lvalue 判定。**OUT**。 |
| **O7** | **Unicode 大小写折叠** | spec §10.7（`upper`/`lower` ASCII）| 全量折叠是**语义泥潭**：长度可变映射（`ß→SS`）、locale、多字符映射，会让 `len(upper(s)) != len(s)` 出乎 agent 意料；ASCII 覆盖绝大多数程序文本。**OUT**（保持 ASCII；doc 标注限制）。 |
| **O8** | **`hint`/`提示：` 行** | spec §8.1（v1 明确不输出）/ D-014 | 逐字符稳定是黑盒测试（85 用例）与 `--json` 的前提；结构化"类名+消息+位置+traceback"已足够定位。**OUT（永久）**。 |
| **O9** | **格式说明符动态宽度** | 盲测缺口 #5 | 破坏 M6「format_spec 原始文本」契约、需嵌套求值；用 `padEnd`/`padStart`（C3）替代。**OUT**。 |
| **O10** | **模块（`import`/`from`）** | BRAINSTORM / D-007 延后 | 单文件程序已满足课程与 agent 场景；模块要解析器/加载器/名字空间全链改动。**OUT**。 |
| **O11** | **类 / 继承** | BRAINSTORM / D-007 延后 | 合一 struct + 方法 + 闭包已覆盖"轻量对象"需求；继承引入原型/虚表语义。**OUT**。 |
| **O12** | **类型注解 / 静态类型** | BRAINSTORM / D-007 延后 | 与动态类型身份冲突；`lfz check` 亦在 backlog。**OUT**。 |
| **O13** | **独立"字典/映射"类型** | 架构师候选 | **已被合一 struct 承担**（`s["k"]` 字典面孔）；再加字典类型会造成"两种复合类型"的重复与 `==`/显示歧义。**OUT**。 |
| **O14** | **可选链 `?.` / `??`** | 架构师候选 | JSON 式遍历可用 `has(k,s)` 防御（盲测已如此）；新记为符号会与 `?` 未定义记号冲突。**OUT**。 |
| **O15** | **`range(lo,hi,step)`（3 参）** | 架构师候选 | 需求低频（逆序可用 `while` 或 `sortBy`）；`step==0` 边界与负步长语义增加规范面。**OUT**（C2 只收 2 参）。 |
| **O16** | **工具链项：REPL / `lfz fmt` / `lfz doc` / `lfz init` / `lfz package` / `--watch` / `lfz check` / `--explain`** | BRAINSTORM #10/#12/#17–21 | **非语言特性**，属 P4 工具链 backlog；不影响 agent"写对程序"的语言能力。**OUT of this audit**（如需，另派 tooling-dev）。 |
| **O17** | **`readFile` 之外的二进制/目录 IO、`exit(code)`、`enumerate`、`zip` 等** | 架构师候选 | 低价值/低频；保持内置面收敛。**OUT**（按需再加）。 |
| **O18** | **`charAt(i,s)` 内置** | 架构师候选 | 与 `split("",s)` + 索引**功能重复**，且会重新引入"按标量下标 O(n)"的误用。**OUT**。 |

---

## 5. 6 个焦点问题裁定

### a) 字符串按字符访问：加 `s[i]`？还是用 `split("", s)`？

**结论：不加 `s[i]`（OUT）；保留并**明确文档化** `split("", s)` 惯用法。**

- **一句话理由**：LFZ string 是 UTF-8 `Rc<String>`，`s[i]` 天然是 `chars().nth(i)` 的 **O(i)**，循环遍历即 **O(n²)**；而 `split("", s)` 一次 **O(n)** 且之后 O(1)/次。
- **推荐惯用法（写进 doc，复杂度逐条标注）**：

  ```lfz
  #42
  // 惯用法 1：一次性拆成字符数组，之后 O(1) 索引，总 O(n)
  let cs = split("", s)
  var i = 0
  while i < len(cs) {
      // 用 cs[i]（一个单字符 string）
      i += 1
  }

  // 惯用法 2：只顺序遍历，无需下标 —— for 直接吃字符数组，O(n)
  for c in split("", s) {
      // c 是单字符 string
  }
  ```

  复杂度：`split("", s)` = **O(n)**（n = Unicode 标量数）；随后每次 `cs[i]` 或迭代 = **O(1)**；整段遍历 **O(n)**。对照：**若实现 `s[i]` 为 `chars().nth(i)`，惯用法 1 会退化成 O(n²)**。
- **若 IN 的最小规范草案（仅在将来重估时用）**：在 §4.2 增加「`[]` 下标仅适用于 `array`/`struct`；对 `string` 取下标 → `TypeError`（消息 `运算符 '[]' 不支持 string 与 array / struct`）」。**本审计建议只补这条"明确报错"，不新增 `s[i]` 功能。**

### b) `range` 的完整签名

**结论：保留 `range(n)`，**新增** `range(lo, hi)`（半开区间 `[lo, hi)`）；**不**加 3 参 step 形式。**

- **一句话理由**：半开区间覆盖"从 1 / 从任意起点"的绝大多数索引循环，且与 `range(n)=[0,n)` 及 `slice` 的半开惯例一致；3 参需求低频。
- **可写进 §10.7 的最小规范草案**：

  ```
  | range | range(n) -> array          | array[int] | [0, 1, …, n-1]；n < 0 → 空数组 |
  | range | range(lo, hi) -> array     | array[int] | [lo, lo+1, …, hi-1]；hi <= lo → 空数组 |
  ```

  语义：两形均产出**普通 `array[int]`**（沿用 §4.5.4 array 快照规则；`for i in range(lo,hi)` 与 `for i in range(n)` 行为一致）。参数非 `int` → `TypeError`；结果仍受 i64 边界约束（不产生溢出，因为只含界内整数）。**不含惰性迭代**。

### c) `let` 在循环体内的语义

**结论：每次迭代产生**新绑定**（当轮独立 cell），与 A2 一致 —— **现有 spec 已如此规定，无需改动**；仅需 skill 明说。**

- **一句话理由**：§4.5.0 与 §4.5.3 已写死"循环中每次迭代的块级 `let`/`var` 是新绑定（各自新 cell）"，探针亦证实；这是"文档缺口"而非"设计缺口"。
- **最小规范草案**：无需（引用 §4.5.0 / §4.5.3 即可）。

### d) 格式说明符是否补 `<`（左对齐）与动态宽度

**结论：`<` **无需补**（spec 早已支持 `align = "<" | ">" | "^"`，仅 skill 漏列）；动态宽度 **OUT**。**

- **一句话理由**：`<` 是 skill 缺口；动态宽度会破坏 M6「format_spec 原始文本」契约，且被 `padEnd`/`padStart`（C3）取代。
- **最小规范草案**：**无 spec 改动**；skill 补一行：`align ∈ { < 左, > 右, ^ 居中 }`，可带 fill 字符（如 `:0>5`）；**明写"宽度必须是字面数字，动态宽度不支持；需要动态宽度用 `padEnd(n, fill, s)`"**。

### e) `len(string)` 是否明确合法（按什么计数）

**结论：**已合法且已定义**：`len(string)` = **Unicode 标量数**（§10.7）。**

- **一句话理由**：§10.7 `len` 行原文即"string 的 Unicode 标量数"，探针 `len("abc")=3` 验证；纯属 skill 未点明。
- **最小规范草案**：无需（可考虑在 skill 中举例，并提醒 `len(s)` 是 **O(n)**，长串循环内应改用 `split("",s)` 后对数组求 `len`（O(1)））。

### f) backlog 里的 Unicode 大小写折叠 / `sin`·`log`·`exp` / `hint` 特性

| 项 | 结论 | 一句话理由 |
|---|---|---|
| **Unicode 大小写折叠** | **OUT** | 全量折叠语义复杂（长度可变、locale），ASCII 已覆盖绝大多数程序文本；收益不抵风险。 |
| **`sin`/`log`/`exp`（+`cos`）** | **IN（P2）** | O(1)、纯函数、沿用既有 int→float 加宽；解锁数学/仿真类真实程序，成本几乎为零。 |
| **`hint`/`提示句`** | **OUT（永久）** | 破坏黑盒测试逐字符稳定；结构化错误（类名+消息+位置+traceback）已足够。 |

---

## 6. 建议纳入下一版（v1.1）的特性表

### 6.1 语言特性（IN）

| # | 特性 | 优先级 | 一句话理由 | 复杂度/性能 |
|---|---|---|---|---|
| 1 | **spec 补钉：`string` 取下标 → `TypeError`** | P1 | 消除"未定义行为"与 impl/spec 消息分歧 | —（纯规范） |
| 2 | **`range(lo, hi)`（半开区间）** | P1 | 覆盖最常见索引循环，终结"手写 while"（盲测第一痛点） | O(hi−lo) |
| 3 | **字符串方法族：`indexOf` / `endsWith` / `padEnd` / `padStart` / `substring`** | P1 | 解析/对齐高频缺失；`padEnd` 直接替代动态宽度 | O(n)；`indexOf` 最坏 O(n·m)（须标注） |
| 4 | **文件 IO：`readFile` / `writeFile` / `appendFile`** | P1 | "读→处理→写"是真实程序基本形态，当前无法完成 | O(文件大小) |
| 5 | **`ord(c)` / `chr(n)`** | P2 | 字符↔码点，哈希/编码/分类常用 | O(1) |
| 6 | **math：`sin` / `cos` / `log` / `exp`** | P2 | 数学/仿真类程序；沿用既有加宽口径 | O(1) |
| 7 | **`contains(v, xs)`** | P2 | 数组成员测试缺失（仅 struct 有 `has`） | O(n)（须标注，勿入循环） |

> **合计 7 项，全部为"内置新增/签名扩展 + 1 条 spec 补钉"，零新文法、零新控制流、零新类型；对 A1–A7 与 §4.5 确定性均无破坏。**

### 6.2 必做文档修订（P0，非语言；**不属 v1.1 特性，但优先级最高**）

> **这 7 条才是把盲测通过率从 2/8 拉起来的关键**；它们是 `ai-dx-engineer` / `docs-writer` 的产物（只读引用 spec），本审计仅提出清单与口径。

| # | 修订点 | 落点 | 盲测证据 |
|---|---|---|---|
| D1 | **字符串不可下标** + `split("",s)` 推荐惯用法（含 O(n) 复杂度） | skill §3/§4/§6 | hashmap_chain 失败；`t2_sindex` |
| D2 | **`range` 完整签名**（`range(n)`，及 v1.1 的 `range(lo,hi)`） | skill §6 | 两份 report 同时列为第一痛点 |
| D3 | **循环体内 `let` 每轮新绑定**（引用 §4.5.0/A2） | skill §4/§7 | knapsack #5 / maze #2 |
| D4 | **退出码语境**：`run` 下 assert → 2；`test` 下用例失败 → 1 | skill §0/§5 | knapsack #3 |
| D5 | **对齐**：`<`/`>`/`^` + fill 可用；动态宽度不支持（用 `padEnd`） | skill §3 | knapsack #4 |
| D6 | **`len(string)` 合法** + 按 Unicode 标量计数 + O(n) 提醒 | skill §6 | knapsack #2 |
| D7 | **正面示例**：链式下标赋值 `t[i][j]=v`、`else if`、多 `${}`、零参 `print()`、`&&`/`||` 短路 | skill §3/§4 | knapsack #6–9 / maze #3–4 |

---

## 7. 与冻结约束的兼容性核对

| 约束 | 本提案影响 | 结论 |
|---|---|---|
| **A1 引用语义 / 内置返回新值** | 新增内置（字符串方法、文件 IO、ord/chr、math、contains）一律返回值、不改原容器 | ✅ 兼容 |
| **A2 按 cell 捕获 / 循环每轮新绑定** | 无改动；C8 正是重申 | ✅ 兼容 |
| **A3 除法/取模、A4 check、A5 数据面、A6 环安全、A7 RecursionError** | 均不触及 | ✅ 兼容 |
| **§4.5 求值顺序 / 确定性「无魔法」** | 新内置为**纯函数**（ord/chr/math/contains/字符串方法）；文件 IO 与既有 `input()` 同属"外部 IO"，不改变求值顺序/类型/无隐式转换纪律 | ✅ 兼容（IO 是环境依赖，已在错误模型内） |
| **M6 format_spec 原始文本** | 拒绝动态宽度即保护 M6 | ✅ 兼容 |
| **§2.3 无 `..`/`??` 记号** | 拒绝区间 O5 / 可选链 O14 即保护 | ✅ 兼容 |
| **既有 445 单测** | 新增内置为**可加性**；唯一需改：若存在"`range` 二元报 `ArgCount`"断言 → 改 1 条；`s[i]` 消息补钉后若单测断言的是另一文案 → 对齐 | 🟡 1 处待核 |
| **既有 85 黑盒用例** | 新增内置不影响既有用例；文件 IO 用例须用临时目录 + 清理（交 test-engineer） | ✅ 兼容 |

### 7.1 既有隐藏 O(n²)（**本审计的架构级发现**，与"新增特性"分开）

1. **字符串 `s[i]`**：**不新增即可**（C1）。若历史上有人实现过 → 立即回退。
2. **字符串拼接循环**：`var s = "" / while … { s = s + c }` 因 string 不可变，每次 `+` 复制整个前缀 ⇒ **O(n²)**。盲测 `json_mini`/`expr_eval` 正是这么写 token 文本的。**建议**：
   - **doc（P0）**：引导"逐字符构建 → 先 `push` 到数组，最后 `join("", arr)`"（**O(n)**）。
   - **runtime（非本 spec 范围，交 runtime-dev 评估）**：当 `Rc<String>` 强计数为 1 且容量足够时，`Str + Str` / `+=` 就地追加（**均摊 O(1)**），语义不变（仍返回"新值"的观测行为）。**不改语义、不改 A1**。
3. **`len(s)` 在循环内反复调用**（C10）：O(n)/次 ⇒ 长串循环 O(n²)；doc 引导先 `split`。

> 这三条共同说明：**"易用性"与"性能"的冲突点集中在 string 的按字符访问与拼接上**；本提案用"`split` + `join` 惯用法 + O(n) 正解"化解，而**不是**新增一个会诱发 O(n²) 的 `s[i]`。

---

## 8. 下一步

1. **本审计交 team-lead/用户**；**用户拍板**决定 v1.1 范围（第 6.1 节的 7 项 IN、第 4 节的 18 项 OUT 是否认可）。
2. 用户确认后，由 language-architect：**先写 ADR（`.opencode/team/DECISIONS.md`，只追加）→ 再改 `docs/spec/`**（§4.2 字符串下标补钉、§10.7 内置表扩展、§4.5.7 math 加宽口径、§8.1 文件 IO 错误口径）。
3. **D1–D7 文档修订**（P0）与语言改动**解耦**、可**立即**交 `ai-dx-engineer`（skill）/ `docs-writer`（手册）执行——零语言风险，先拿收益。
4. 通知下游：`core-dev`（若涉及 builtins 注册）、`runtime-dev`（字符串拼接就地优化评估）、`test-engineer`（新内置用例 + 文件 IO 夹具）、`docs-writer`/`ai-dx-engineer`（D1–D7）。

---

*（本文件是审计与提案，非事实源；未修改 `docs/spec/`、`src/**`、`docs/guide/**`、`.opencode/skills/**`、`tests/**`。事实源仍为 `docs/spec/` 冻结 v1 三件套。）*
