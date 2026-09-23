# LFZ 语言 v0.4 定稿候选（FINAL CANDIDATE · 可冻结）

> 作者: language-architect ｜ 日期: 2026-09-23 ｜ 状态: **v0.4 = 定稿候选，可冻结**（尚非冻结 spec）
> 依据: `.opencode/team/KICKOFF.md`、`.opencode/team/DECISIONS.md`（D-007 v1 范围、D-011 实现语言 = Rust 与 float 纳入、D-012 v0.2 语法修订、**本轮 D-014 v0.4 定案**）、`.opencode/team/DRAFT-LFZ-v0.2.md`、`.opencode/team/DRAFT-LFZ-v0.3.md`、**用户本轮拍板（前导按扩展名 / 错误模型 / 可移植性 共 5 条）**。
> 本文件以 v0.3 为基线：落实用户本轮 5 条拍板，**未触及处逐字沿用 v0.3**。
> 核心目标：**每一步解析/求值都有唯一确定的解释**（无歧义），且**任何语言行为都能被黑盒测试二值判定**。

> **「v0.4 = 定稿候选，可冻结」**：v0.3 §14 的 5 项开放问题**已全部闭合**（用户拍板），§14 现仅剩 **4 条"遗留默认（仍可覆盖）"**（`;;` 作用域 / `;;` 输出通道 / 多行续行 / 单 `;`）；它们**不阻塞冻结**。满足下列前提即转入 `docs/spec/` 三件套并冻结 v1。

### 冻结前提条件（满足后即写入 `docs/spec/{syntax,semantics,interface-contract}.md`）

1. **无遗留分歧**：§14 仅剩 4 条"遗留默认"，视为**可覆盖默认**；team-lead / 用户如无异议即视为接受，**不阻塞冻结**。
2. **可实现性评审通过**：core-dev（文法可解析性）与 runtime-dev（语义可求值性）评审通过或**明确接受**（依 D-008 契约先行、D-009 门禁）。
3. **ADR 已落库**：`.opencode/team/DECISIONS.md` 追加本轮定案 ADR（**D-014**，编号说明见 ADR 正文）。
4. **拆分映射（冻结后即写）**：
   - `docs/spec/syntax.md` ← §2–§7；
   - `docs/spec/semantics.md` ← §3.6–§3.7、§4、§8；
   - `docs/spec/interface-contract.md` ← §8.1 错误类 + §10 的 **loader 双入口 + `ext(path)` 扩展名判定 + `Span` + `CallFrame` + `LfzError` 类名枚举 + `DebugSym` + `line_base` 行号语义**。
5. **冻结动作**：上述完成后追加 ADR「**spec v1 冻结**」，此后任何变更先 ADR 后改文档。

---

## TL;DR（三句话）

1. **每个 `.lfz` 文件的第一行必须恰好是 `#42`**（其后紧跟一个行终止符）——这是「宇宙的答案」仪式。缺失即抛 `CosmosAnswerError: 你忘记了宇宙的答案`。**触发看扩展名**：仅 `.lfz`（大小写不敏感）文件要求；**非 `.lfz` 文件**（如 `.txt`，即被 `lfz run` 执行也不校验）与 **REPL / stdin / `lfz run -e`** 一律豁免。
2. **错误模型改为 Python 风格**：命名错误类（`SyntaxError` / `TypeError` / … / `CosmosAnswerError`）+ **中文消息** + `File "<path>", line N` + 源码行 + 插入符；跨函数显示 **traceback**；**取消 `E-xxx` 编号**（用户输出不再出现，`--json` 用类名）。
3. **可移植性**：行终止符接受 `\n` / `\r\n` / `\r`；文件开头 UTF-8 BOM 静默跳过；源码要求 UTF-8（非 UTF-8 → 清晰报错）；`#42` 内容本身**严格无变体**。

---

## 0. 相对 v0.2 的差异摘要（用户新指令的落地）

| # | 用户指令 | v0.2 | v0.3 采用 |
|---|---|---|---|
| A | **文件前导 `#42`** | 无 | **新增**：`.lfz` 文件第一行必须恰好 `#42`+行终止符；仅文件模式；无变体；失败抛 `CosmosAnswerError`（§2.2、§6-B、§8）；**v0.4 把"仅文件模式"收窄为"仅 `.lfz` 扩展名文件"，见下表 F** |
| B | **Python 风格错误模型** | `E-xxx` 编号 + 消息模板 | **全量改写**：命名错误类 + 中文消息 + 位置 + 插入符 + traceback；**取消编号**；新增 `CosmosAnswerError`（§8） |
| C | **可移植性** | UTF-8 无 BOM；`LF/CRLF/CR` | **放宽**：跳过 BOM；接受 `\n/\r\n/\r`；非 UTF-8 清晰报错；`#42` 仍严格（§2.1、§6-B7/B8） |
| D | 样例必须带头 | 无头 | **所有样例程序加 `#42` 首行**（§9） |
| E | AI 指南 / test-runner | 无要求 | **新增硬性要求**：AI 指南写死 `#42` 首行 + 错误类清单；runner 契约要求测试文件带头（§11） |

**连带修订**：
- 错误码 → 类名映射表（§13，仅追溯用，不进入用户输出）。
- EBNF：`program` 增加 `[ preamble ]`（§7）。
- Rust 实现新增 **loader 阶段**、**AST `Span`**、**调用栈（traceback）**（§10）；`DebugSym` 职责不变（服务 `;;`）。
- 保留：5 大特色（D-007）、Rust（D-011）、float 与 `int→float` 加宽、花括号块、换行终结语句、`;;` dump、`${}` 插值、A1–A26 全部歧义规则。

### v0.3 → v0.4 差异摘要（用户本轮拍板落地）

| # | 变更点 | v0.3 | v0.4 采用 |
|---|---|---|---|
| F | **前导检查触发条件** | "文件即需前导"（凡从文件路径加载即要求 `#42`，**不限扩展名**） | **仅按扩展名触发**：扩展名（ASCII 大小写不敏感）== `lfz` 才校验；**非 `.lfz` 文件豁免**（§2.2.0、B11、§7） |
| G | `CosmosAnswerError` 源码行显示 | 不显示第 1 行原文/插入符 | **保持**（**已定**；§8.2、B10） |
| H | 空程序体 | 允许（`#42` + 行终止符即合法） | **保持**（**已定**；B2、§7） |
| I | `hint` / `提示：` 行 | v1 不输出（§14-4 列为"可选"议题） | **明确写死 v1 不输出**；可选能力**列入 v1.1 backlog**（§8.1） |
| J | `#42` 后行终止符 | 必须紧跟行终止符 | **保持**（**已定**；3 字节 `#42` → `CosmosAnswerError`；B2） |
| K | §14 开放问题 | 5 项待拍板 | **清掉 5 项**；仅剩 v0.2/v0.3 **遗留默认 4 条**（§14） |

---

## 1. 语言身份（不变）

LFZ（读作 "elf-zee"）是**动态类型、表达式导向的通用脚本语言**，魂为一句话：

> **「万物皆值，值皆可流；一种结构，两种面孔。」**

- **万物皆值**：`if`、块、函数、数组、结构体全是值；块的值 = 最后一条表达式的值；函数体末尾表达式的值 = 返回值（`return` 仅用于提前退出）。
- **值皆可流**：任何值都能顺 `|>` 管道流向下一函数，`_` 占位符可把它精确塞进任意参数位。
- **一种结构，两种面孔**：复合类型仅 `array`（有序）与 `struct`（无序键值）；`struct` 同时具备对象面孔（`.字段`/方法）与字典面孔（`["键"]`），同源存储。
- **无魔法 / 确定性**：无 truthiness（条件必须 `bool`）、无隐式转换（唯一例外 `int→float` 无损加宽）、越界/缺字段/整数溢出/类型不符一律结构化报错。
- **仪式感（v0.3 新增；v0.4 收窄触发面）**：每个 **`.lfz` 文件**（按扩展名判定，大小写不敏感）以 `#42` 起手——「宇宙的答案」，缺失即 `CosmosAnswerError`；**非 `.lfz` 文件**与 REPL/stdin/`-e` 豁免。

**与现有语言的差异一句话**：F# 的管道（尾参注入）+ Elixir 的 `_` 插位 + Python 的动态易用与换行风格 + C 的花括号块 + Rust 的"无隐式/溢出即错"纪律 + 一种自成一体的"字典=对象"统一 struct + 独一无二的 `#42` 文件前导与 `CosmosAnswerError`。

---

## 2. 词法基础（v0.3 修订）

### 2.1 源文本、编码与行终止符（修订）

- 文件后缀 `.lfz`；源码编码 = **UTF-8**。
- **BOM（`U+FEFF`）**：文件**开头**若有 UTF-8 BOM（字节 `EF BB BF`）→ **静默跳过**，不作为源码内容、不报错。仅跳过一个 BOM。
- **行终止符**接受三种：`\n`（LF，`U+000A`）、`\r\n`（CRLF，`U+000D U+000A`）、`\r`（CR，`U+000D`）。加载器**统一归一化为 `\n`**（§2.2）。
- **非 UTF-8** → `SyntaxError`（消息见 §8）。**判定顺序：先编码校验，后前导校验**（见 §2.2）。
- **缩进与空白纯视觉**：`空格` / `Tab` 只作 token 分隔，**不决定块结构**（块由 `{}` 决定）。
- **字符串内不允许裸（未转义）行终止符** → `SyntaxError`（§2.8）。
- **文件扩展名决定"是否要求前导"**（v0.4 新增）：仅当路径扩展名（ASCII 大小写不敏感）== `lfz` 时要求 `#42` 前导；非 `.lfz` 文件豁免。判定规则见 §2.2.0。

### 2.2 文件前导 `#42`（v0.3 核心新增；v0.4 改为按扩展名触发）

#### 2.2.0 文件扩展名判定（v0.4 新增，规范性）

**前导 `#42` 的强制，仅由「路径的扩展名」决定**（不再"凡文件皆需前导"）。

```
ext(path):
  1. 取路径的最后一个路径分量：按 '/' 与 '\' 两者切分，取最后一段。
     （盘符 'C:' 中的 ':' 不参与切分，故 Windows 路径安全。）
  2. 若该分量不含 '.' -> 无扩展名。
  3. 否则取【最后一个 '.'】之后的子串为扩展名（可为空串）。
  4. 扩展名与 "lfz" 比较时按 ASCII 大小写不敏感。
```

- **是 `.lfz` 文件（要求前导）**：`foo.lfz`、`Foo.LFZ`、`a.b.Lfz`、`x.lfZ`。
- **非 `.lfz` 文件（豁免前导）**：`foo.txt`、`foo`（无扩展名）、`foo.`（扩展名为空串）、`.gitignore`（扩展名 `gitignore`）、`foo.lfz.bak`（扩展名 `bak`）、`foo.lfz.txt`（扩展名 `txt`）。
- **大小写不敏感**（Windows 友好）：`.lfz` / `.LFZ` / `.Lfz` / `.lfZ` 均视为 `.lfz`，仅按 **ASCII** 折叠（不涉及 Unicode 大小写）。
- **判定只看路径字符串**：不访问文件系统、不解析符号链接或 Windows 8.3 短名；同名不同扩展名严格区分；`lfz run` 传入什么路径，就按该路径字符串判定。
- **无路径即无扩展名**：REPL / stdin / `-e` 无路径 → 不是 `.lfz` 文件 → 豁免（§6-B9）。

#### 2.2.1 前导定义（规范性）

1. 前导是一行**恰好等于** `#42` 的内容（三个字符：`U+0023` `U+0034` `U+0032`），**其后必须紧跟一个行终止符**（`\n` | `\r\n` | `\r`）。**仅三字节 `#42`、无行终止符 → `CosmosAnswerError`**（**v0.4 已定**；§6-B2）。
2. **`#` 是「文件开头专用符号」，与注释无关**：注释仍用 `//` 与 `/* */`。`#` 出现在前导位之外的任何位置（字符串 / 注释 / 格式说明符之外）→ 非法字符 `SyntaxError`。
3. **触发条件 = 扩展名为 `.lfz`（ASCII 大小写不敏感）的文件**（§2.2.0）。分三档：
   - **(a) `.lfz` 文件**：**要求**前导；缺失/违规 → `CosmosAnswerError`。
   - **(b) 非 `.lfz` 文件**（如 `.txt`）：**不要求**前导；即使经 `lfz run` 执行也**不校验**（仍是文件模式：UTF-8 校验、跳 BOM、归一化行终止符、错误用真实路径）。
   - **(c) 非文件入口**（REPL / stdin / `-e`）：**豁免**前导。
4. **不允许任何变体**：`# 42`、`#42abc`、`#43`、`#42 `（尾随空格）、`#42\t`、`##42`、前导空行/空格、BOM 之后的空行——全部违规。
5. 不满足 → 抛 **`CosmosAnswerError`**，用户可见消息**恒为**「你忘记了宇宙的答案」，**不显示编号、不显示 hint、不显示第 1 行原文/插入符**（**v0.4 均定**；§8）。

#### 2.2.2 加载器算法（唯一确定，逐字节）

```
load_file(path):
  1. is_lfz = ( ext(path) 按 ASCII 大小写不敏感 == "lfz" )。     // §2.2.0
  2. 读入全部字节。
  3. 按 UTF-8 解码整文件；解码失败 -> SyntaxError（编码错误），终止。
     （先编码、后前导：非 UTF-8 文件报编码错，而非 CosmosAnswerError；所有文件皆然。）
  4. 若文本以 U+FEFF 开头 -> 去掉一个该字符（仅一次）。
  5. 将文本中 \r\n 与单独 \r 归一化为 \n。
  6. 若 !is_lfz：
       - 不校验、不消费前导；line_base = 0；交接 lexer。
  7. 若 is_lfz：
       a. 若文本为空 -> CosmosAnswerError（line 1）。
       b. 若前 3 个字符不严格等于 "#42" -> CosmosAnswerError（line 1）。
       c. 若第 4 个字符不是行终止符（已归一化为 \n；若是 EOF 或其它字符，
          含空格/Tab/字母/第 2 个 '#'）-> CosmosAnswerError（line 1）。
       d. 消费 "#42" 三字符与其后的 \n（即整个前导行）；
          **不产出任何 token（含不产出 NEWLINE）**。
       e. line_base = 1（前导占原第 1 行）；交接 lexer。
load_source(text):          # REPL / stdin / -e 使用
  跳过前导校验；仍做解码(3)/跳 BOM(4)/归一化(5) 中适用步骤；
  line_base = 0。
```

**行号语义（v0.4 明确，规范性）**：lexer 报告的 `line` = **本地行号 + `line_base`**，本地行号自 1 起（body 首行 = 本地 1）。
- **`.lfz` 文件**：body 首行 = 1 + 1 = **2**（前导 = line 1）。
- **非 `.lfz` 文件 / `load_source`**：body 首行 = 1 + 0 = **1**。

**规则冲突消解（规范性）**：
- `#` 只在**加载器的文件前导位**（`is_lfz` 的步骤 7d）被消费；lexer 的 CODE 模式**从不接受** `#`。
- 因此 `#` 合法出现的场景**仅**：(a) `.lfz` 文件前导位；(b) 字符串文本内（STR 模式）；(c) 注释内；(d) 格式说明符文本内（§2.8）。其余一律 `SyntaxError`。

### 2.3 多字符记号的最大匹配表（沿用 v0.2 §2.2）

词法采用**最长匹配（maximal munch）**。完整记号表（长→短，**必须整体识别**）：

```
;;   =>   ==   !=   <=   >=   &&   ||   |>
+=   -=   *=   /=   %=
//   /*   */
(   )   [   ]   {   }   ,   :   .   _
=   <   >   !   +   -   *   /   %   ;
```

- **`#` 不是记号**：只在 **`.lfz` 文件**的前导位由加载器消费，CODE 模式下出现即非法字符（§2.2）。
- **禁止合成**（不存在这些记号，必须拆开）：`>>` `<<` `++` `--` `->` `..` `??` `::` `|` `&`。
- `//` **恒为行注释**，绝不构成整除运算符（整除用内置 `div(a,b)`，§6-B 外的 A18）。

### 2.4 注释（沿用 v0.2 §2.3）

- 行注释 `// … 到行尾`（**不含**行尾换行；其后换行仍是 `NEWLINE` 记号）。注释内 `#` 是普通字符。
- 块注释 `/* … */`，**不可嵌套**；一个块注释经词法后**等价于一个空格**。
  - **规范性**：块注释**不产生** `NEWLINE`，**不充当语句分隔符**。跨行块注释把两行粘成一条逻辑行 → 若导致两条语句相邻则 `SyntaxError`（A22）。

### 2.5 空白与换行（沿用 v0.2 §2.4）

- 空白 = 空格 / Tab / `\r`（归一化前）/ 块注释。
- **`NEWLINE` 是独立记号**（值 `\n`，加载器已归一化）。其"是否有效"由**解析器的换行模式栈**决定（§3.2），**不是**词法阶段按括号深度统一抑制。

### 2.6 标识符与关键字（沿用 v0.2 §2.5）

```
IDENT = (letter | "_") , { letter | digit | "_" } ;
letter = "a"…"z" | "A"…"Z" ;
```

- **仅 ASCII**。
- **保留 token**：单独一个 `_` 不是标识符，而是占位符 token `PLACEHOLDER`（只用于管道右侧，A24）。`_x`、`x_` 是普通标识符。
- **关键字（v1 保留，不可作标识符）**：
  `let` `var` `fn` `return` `if` `else` `while` `for` `in` `break` `continue` `struct` `true` `false` `nil` `self`
- **未来保留（当前使用即 `SyntaxError`）**：
  `match` `class` `import` `from` `try` `catch` `rescue` `yield` `async` `await` `and` `or` `not` `is`

### 2.7 数值字面量（沿用 v0.2 §2.6）

```
INT   = dec_int | "0x" hex_digits | "0b" bin_digits | "0o" oct_digits ;
FLOAT = digit {digit|"_"} "." digit {digit|"_"} [exponent]
      | digit {digit|"_"} exponent ;            (* 指数式无小数点，如 1e10 *)
```

- `1_000`、`0xFF`、`0b1010`、`0o755`、`3.14`、`2.5e-3`、`1e10` 合法。
- **边界（消歧）**：小数点后**必须**有数字 → `1.` 是 `INT(1)` 紧跟 `.`（字段访问，A23）；`.5` 非法（须写 `0.5`，`SyntaxError`）。不存在 `..` 记号。

### 2.8 字符串与插值（沿用 v0.2 §2.7，补 3 条）

**词法产出结构化记号**（不是单个 `STRING` token），供 parser 组合：

```
STRING_BEGIN   TEXT(片段)   INTERP_BEGIN   <普通代码记号流>   INTERP_END   STRING_END
```

**lexer 模式栈** `M ∈ { CODE, STR, INTERP }`：

1. `CODE` 下遇 `"` → 发 `STRING_BEGIN`，push `STR`。
2. `STR` 下：
   - 普通字符 → 累积为 `TEXT`（遇 `"` 结束发 `STRING_END`）；转义 `\n \t \r \\ \" \e \$` 在 `TEXT` 中解码（`\e` = ESC 0x1B）。**`#` 是普通字符**（`"#42"` → 文本 `#42`）。
   - 遇 **`${`** → 发 `INTERP_BEGIN`，置**花括号深度 `depth = 1`**，push `INTERP`，转入 `CODE`。
   - 遇未转义 `"` → 发 `STRING_END`，pop。
   - 遇**裸行终止符**（`\n`）→ `SyntaxError`（字符串未闭合，见 §8）。(**补充规则 1**)
   - 遇 EOF → `SyntaxError`（字符串未闭合）。
   - **`{` / `}` 在 `STR` 模式下是普通字面字符**（无特殊含义，A14）。
3. `INTERP` 模式（即 `CODE` 模式下）：
   - `{` → `depth += 1`；`}` → `depth -= 1`；当 `depth` 归 0 → 发 `INTERP_END`，pop 回 `STR`。
   - 遇 `"` → 递归进入 `STR`（嵌套字符串）；从 `STR` 返回后仍处于 `INTERP`，`depth` 不变。
   - 在 `depth == 1`（相对 `INTERP` 的顶层）且遇到第一个 `:` → 记为 **format-spec 分隔点**，`:` 之后到匹配 `}` 之间是 `format_spec` 文本（其中不含花括号）。
   - 遇到 `INTERP` 的 EOF → `SyntaxError`（字符串未闭合）。
4. 后缀格式说明符：
   ```
   format_spec = [ [fill] align ] [sign] [width] [ "." precision ] [type] ;
   align = "<" | ">" | "^" ;  type = "d" | "x" | "X" | "o" | "b" | "f" | "e" | "s" ;
   ```
   - **format_spec 是原始文本**（非 CODE 模式）：其中的 `#` 等字符按字面处理，**不触发非法字符**。(**补充规则 2**)
   - 非法说明符 → `ValueError`；说明符与值类型不符 → `TypeError`（§8）。(**补充规则 3**)

**无冲突理由（规范性）**：块/字面量的 `{}` 出现在 `CODE` 模式；插值仅由 `$` 前缀的 `${` 触发且发生在 `STR`→`INTERP` 模式内。两个字符集在**模式上互斥**，故 `${…}` 与块花括号**不可能冲突**。`$` 后非 `{` → 字面 `$`。

---

## 3. 语句、块、换行（沿用 v0.2 §3；仅把错误码改为类名）

### 3.1 语句终结规则（否定式）

- **没有**语句分隔符。**一条语句占一条逻辑行**（same-line 两条语句**不可能**）。
- **唯一**的换行续行手段是**括号**：`( … )`、`[ … ]`，以及**字面量/声明体** `{ … }`。括号内换行被忽略。
- **不允许**行尾运算符续行，**不允许**反斜杠续行，**不允许**行首 `.`/`|>` 续行（A2）。

### 3.2 解析器换行模式栈（**必须由 parser 管理**）

> 为什么不能只按 lexer 括号深度？因为块可嵌在 `()`/`[]` 里，例如 `f(fn(x){ let y=x \n y })`——若在 `(` 内统一抑制换行，块内语句就会粘连。故换行是否有效**只能由 parser 依上下文决定**。

解析器维护 `NLMode ∈ { SIG, IGN }` 栈（初始 = `SIG`）：

| 进入构造 | push | 该上下文内 `NEWLINE` |
|---|---|---|
| 程序顶层 / **块 `{`**（if/while/for/fn/lambda/else 的函数体） | `SIG` | **有效**：终结语句 |
| 实参表 `( … )`、数组字面量 `[ … ]`、**struct 字面量/声明的成员表 `{ … }`** | `IGN` | **忽略**：当空白跳过 |

- `IGN` 模式下：解析器**跳过**所有 `NEWLINE` 记号。
- `SIG` 模式下：`NEWLINE` 终结当前语句。
- 嵌套时**各构造独立 push/pop**：例如 `f(fn (x) { … })` 中，`(` 压 `IGN`，块 `{` 再压 `SIG`，块内换行有效；块闭合 pop 回 `IGN`。
- **块 `{` 与 字面量 `{` 的判定**：完全由**文法位置**决定（§3.3），不靠词法。

**语句终结检查**：每条语句解析完后，下一个有效记号**必须**是 `NEWLINE`（消费连续多个）、`}` 或 `EOF`；否则 `SyntaxError`（同一逻辑行出现两条语句）。

### 3.3 `{` 的两种角色：块 vs struct 字面量

按**文法位置**二分，二者并集覆盖全部可能，无重叠：

1. **块位置（block-required）**：紧跟下列头部之后（中间允许换行，§3.5）：
   `if <expr>`、`else`、`while <expr>`、`for <id> in <expr>`、`fn <name>(<params>)`、`fn (<params>)`、`(<params>) =>`、`struct <Name>`。
   → 此处 `{` **是块**。
2. **表达式位置**：其余一切期待表达式之处遇到 `{`（可选前置类型名 `IDENT`）→ **struct 字面量**。
   - 含**语句起始处**的 `{`：LFZ **没有"裸块语句"**，故语句首 `{` 恒为**匿名 struct 字面量**（A9）。
   - **例外**：`if`/`while` 条件位与 `for` 可迭代位**禁止裸 struct 字面量**（§3.4 / A6）。

### 3.4 控制流条件位的"禁裸花括号"规则（采 Rust 规则）

> 问题：`IDENT {` 是"模板实例化"（`Student { x:1 }`），故 `if c { }` 会被误读为条件 `c{…}`。

**规则（规范性）**：解析 `if`/`while` 的条件表达式、`for … in` 的可迭代表达式时，进入 **NO_BRACE_LITERAL 模式**：

- 在该表达式的**最外层**（不在任何 `()`/`[]` 内）遇到 `{` → **终止表达式**，该 `{` 交给块。
- 进入 `(`/`[` 时**临时清除**该限制；离开时**恢复**（只在"最外层"生效）。
- 因此条件里要用 struct 字面量，**必须加括号**（A6）。

### 3.5 头部与块之间的换行

- **块前换行**：当文法**期待一个块**时，解析器先**跳过**任意 `NEWLINE`，再期待 `{`。
- **`else` 前换行**：解析完 `if` 的 consequence 块后，解析器**保存位置 → 跳过 `NEWLINE` 们 → 看下一个有效记号是否为 `else`**；是则消费并解析 `else` 分支，否则**回退**。
- 花括号使块边界明确，故悬挂 `else` **不存在**：`else` 一律绑定**最近的、尚未闭合的** `if`（A4）。

### 3.6 `;;` —— 变量 dump 语句

**语法**：`;;`（**两个相邻 `;` 字符，中间不可有空白/注释**）= 一个 `DUMP` 记号 = 一条**独立语句**，**独占其逻辑行**。

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

### 3.7 值的显示形式（供 `print` / `;;` / `str()` 复用）

| 类型 | 顶层 | 嵌套在 array/struct 内 |
|---|---|---|
| `nil` | `nil` | `nil` |
| `bool` | `true`/`false` | 同 |
| `int` | 十进制 | 同 |
| `float` | 最短往返；整值浮点显示 `.0`（`1.0`） | 同 |
| `string` | **原样，不加引号** | **加 `"`，内部按转义输出** |
| `array` | `[e1, e2, …]`（逗号+空格） | 同 |
| `struct` 实例 | `{k1: v1, k2: v2}`，**键按字节序升序** | 同 |
| `function` | `<fn 名字>`（匿名 `<fn>`） | 同 |
| `struct` 模板 | `<struct 名字>` | 同 |

---

## 4. 表达式、运算符与优先级（沿用 v0.2 §4，未变）

### 4.1 优先级表（高 → 低，全部左结合）

| 级别 | 运算符 | 说明 | 结合性 |
|---|---|---|---|
| 1（最紧） | `()` `[]` `.` | 调用 / 索引 / 字段（postfix） | 左 |
| 2 | `-` `!`（一元前缀） | 取负 / 逻辑非 | 右（前缀） |
| 3 | `*` `/` `%` | 乘 / 除 / 取模 | 左 |
| 4 | `+` `-` | 加 / 减 / 字符串连接 | 左 |
| 5 | **`\|>`** | **管道** | 左 |
| 6 | `<` `<=` `>` `>=` | 比较 | 左 |
| 7 | `==` `!=` | 相等 | 左 |
| 8 | `&&` | 短路与 | 左 |
| 9（最松） | `\|\|` | 短路或 | 左 |

直观规则：**`|>` 比 `+ - * / %` 松，比任何比较/逻辑都紧**（Elixir 位置）。

### 4.2 类型化运算符（无隐式转换，特色 5）

- `+`：`int+int`、`float+float`、`string+string`（连接）。
- `- * / %`：数值。
- `*`：额外支持 `string * int`（重复，`"█" * 5`）。
- 比较：数值之间、`string` 之间（UTF-8 字节序）；`bool` 仅 `== !=`；`nil` 仅 `== !=`。
- `==`/`!=`：标量按值；`array`/`struct` **深结构相等**；`function` **同一性**。
- `&& ||`：两侧与结果都必须 `bool`，短路求值。
- **`int`/`float` 混合**：唯一隐式转换——`int` 在算术/比较中**无损加宽**为 `float`（D-011）。

### 4.3 管道语义（特色 1，parser 阶段脱糖，运行时零开销）

**语法**：`pipe_expr = add_expr { "|>" pipe_rhs }`，`pipe_rhs = postfix | lambda`。

**脱糖规则**（解析期）：

1. `L |> F(a1,…,an)`：若参数中**恰有一个 `_`** → 把 `L` 替换该位；**无 `_`** → `L` **追加为末参**（data-last）；**≥2 个 `_`** → `SyntaxError`。
2. `L |> R`（R 非调用：函数名/方法/lambda）→ `R(L)`。
3. `R` 求值后不是函数 → 运行时 `TypeError`。
4. `_` 出现在右侧之外 → `SyntaxError`（含 `let _ = …` 绑定）。
5. `L |> R1 |> R2` 左结合 = `R2(R1(L))`；`_` 可出现在右侧调用实参的**任意嵌套深度**。

### 4.4 管道优先级的修正（真 bug 已修，采表）

v0.2 已修正 v1 的"表 vs EBNF"矛盾。v0.3 保持：

```
cmp_expr  = pipe_expr { ("<"|"<="|">"|">=") pipe_expr } ;
pipe_expr = add_expr  { "|>" pipe_rhs } ;
add_expr  = mul_expr  { ("+"|"-") mul_expr } ;
mul_expr  = unary     { ("*"|"/"|"%") unary } ;
unary     = ("-"|"!") unary | postfix | if_expr | lambda ;
```

- `1 + 2 |> f` → `(1+2) |> f` → `f(3)` ✅
- `xs |> sum() > 10` → `(xs |> sum()) > 10` ✅
- `xs |> sum() + 1` → **语法错误**（`|>` 右侧仅收 `pipe_rhs`）→ 须写 `(xs |> sum()) + 1`。**有意为之、已写死**（A2）。

---

## 5. 歧义审计与消解（A1–A26，沿用 v0.2；错误码统一替换为类名）

> 每条格式：**潜在歧义** → **消解规则** → **正例** / **反例**。所有语法类违规均为 `SyntaxError`，运行类见 §8。

- **A1 换行终结语句 vs 括号内换行**：NL 模式栈（§3.2）。`SIG` 内 `NEWLINE` 终结语句；`IGN` 内当空白。正例 `f(a,\n b)`；反例 `let x = 1 let y = 2` → `SyntaxError`。
- **A2 行尾运算符不续行**（Python 式，**括号是唯一续行手段**）。正例 `let x = (1 +\n 2)`；反例 `let x = 1 +\n2` → `SyntaxError`。
- **A3 `{` 前换行**：块期待位先跳过换行再见 `{`；表达式位换行照常终结语句。正例 `if c\n{ … }`；反例 `let p = Foo\n{ x: 1 }` → 两句（陷阱，不报错）。
- **A4 `else` 与 `if` 换行 / 悬挂 else**：跳过换行看 `else`；绑定最近未闭合 `if`。反例 第二个 `else` 无主 → `SyntaxError`。
- **A5 `{}` 双重角色**：纯文法位置判定；无裸块语句。正例 `while c { … }` / `let d = { "k": 1 }`。
- **A6 `if c { … }` 被吞块**：条件/可迭代位 NO_BRACE_LITERAL（§3.4）。反例 `if Point { x:1 }.x > 0 { }` → `SyntaxError`。
- **A7 struct 成员分隔符**：**逗号必填**，尾逗号可选，声明体与字面体一致，其 `{}` 内换行忽略。反例 缺逗号 → `SyntaxError`。
- **A8 数组元素 / 实参跨行**：同 A7（逗号必填，换行忽略，尾逗号可选）。
- **A9 语句首 `{`**：恒为匿名 struct 字面量。反例 `{ let x = 1 }` → `SyntaxError`。
- **A10 `;;` 触发与位置**：`;;` 是相邻两 `;` 组成的单一记号、独立语句、独占逻辑行（允许行首/行尾空白与尾随 `//` 注释）。反例 `x = 1 ;;` → `SyntaxError`。
- **A11 单个 `;`**：词法合法（`SEMI`）但**文法从不接受** → `SyntaxError`（提示改用 `;;`）。取消"空语句"。
- **A12 `;;;` / `; ;`**：最长匹配。`;;;` = `DUMP`+`SEMI` → `SyntaxError`；`; ;` = `SEMI SEMI` → `SyntaxError`；`;;;;` = 两个 `DUMP` 同行 → `SyntaxError`。
- **A13 `;;` 输出通道/顺序/作用域**：§3.6 已写死（stdout、内→外、slot 升序、遮蔽去重、`<name> ： <value>`）。
- **A14 插值 `${}` 与块花括号**：字符串内 `{`/`}` 是普通字面字符；插值必须由 `${` 触发；模式上互斥，无冲突。
- **A15 插值转义 / 嵌套 / `:` 切分**：`\${` 输出字面 `${`；`$` 后非 `{` 即字面 `$`；仅切分 `depth==1` 处第一个 `:`。反例 `"${x"` → `SyntaxError`。
- **A16 一元负号 vs 减法**：`-` 单记号，前缀=一元 / 中缀=减法；无 `--`。`a--b` = `a - (-b)`。
- **A17 管道 `|>` 与换行**：仅同一逻辑行或括号内跨行。反例 第二行以 `|>` 开头 → `SyntaxError`。
- **A18 `|>` vs `||` vs `|`；`=>` vs `==` vs `=`；`//` vs 除**：最大匹配表（§2.3）。单独 `|`/`&` 非法 → `SyntaxError`；`//` 恒为注释。
- **A19 `return`/`break`/`continue` 后换行**：`return` 行尾 = 返回 `nil`；返回值须同行。`break`/`continue` 天然无歧义。
- **A20 lambda `(params) => …` vs 括号表达式**：`( … )` 后前瞻 `=>` → lambda；含逗号只可能是参数表。反例 `(a + b) => c` → `SyntaxError`。
- **A21 赋值 vs 表达式语句（lvalue）**：先按表达式解析；后跟 `assign_op` 则左侧须为 lvalue，否则 `SyntaxError`。反例 `f() = 1`、`a + b = 1`。
- **A22 块注释跨行是否作分隔符**：**不算**（等价空格）；粘出两语句 → `SyntaxError`。
- **A23 数值字面量边界**：小数点后必须有数字；不许前导小数点。反例 `.5` → `SyntaxError`。
- **A24 `_` 占位符 vs 标识符**：单独 `_` 仅可作 `|>` 右侧调用实参；绑定/主位使用 → `SyntaxError`。
- **A25 `>>` 不合成**：拆成两个 `>`；v1 无泛型/移位，无冲突。
- **A26 `if`/lambda 后直接后缀调用**：`if_expr`/`lambda` 不作 postfix 基，须加括号。反例 `if c {1} else {2}(x)` → `SyntaxError`。

**审计条目数：26（A1–A26）。**

---

## 6. 新增规则的歧义 / 边界审计（B1–B12）★ v0.3 新增；v0.4 修订（B1/B2/B9/B10/B11/B12，按扩展名触发）

> 针对用户本轮新指令（文件前导 `#42` / 错误模型 / 可移植性）逐项给出**规则 + 正例 + 反例**。默认入口模式记为 `FILE`；若涉及其它入口模式则显式标注。

### B1. 空文件（0 字节）
- **规则**：**`.lfz` 文件**要求前导 → 空 `.lfz` 文件 → `CosmosAnswerError`（line 1）；**非 `.lfz` 空文件**不要求前导 → **合法空程序**（执行无输出）。
- 正例：空的 `x.txt`（非 `.lfz`）→ 合法空程序。
- 反例：`build artifacts` 产生的 0 字节 `x.lfz` → `CosmosAnswerError: 你忘记了宇宙的答案`。

### B2. 只有 `#42`、无行终止符（EOF 紧随）——**v0.4 已定**
- **规则**：`.lfz` 文件中 `#42` 后**必须**紧跟行终止符；EOF → `CosmosAnswerError`（line 1）。
- 正例：`#42\n`（合法，**程序体为空亦合法——v0.4 已定**）；`#42\r\n`、`#42\r` 均合法。
- 反例：文件内容恰为 3 字节 `#42`（无换行）→ `CosmosAnswerError`。

### B3. `#42` 前有空行 / 空格 / Tab
- **规则**：前导必须在**第 1 行第 1 列**；任何前置空白 → `CosmosAnswerError`。
- 正例：`#42\nlet x = 1\n`。
- 反例：`\n#42\n…`、` #42\n…`、`\t#42\n…` → 均 `CosmosAnswerError`。

### B4. BOM 位置
- **规则**：仅跳过**文件最开头**的一个 BOM（`U+FEFF`）；之后即要求 `#42`。
- 正例：`<BOM>#42\n…`（合法）；`#42\n…`（无 BOM 亦合法）。
- 反例：`<BOM>\n#42\n…`（BOM 后有换行）→ `CosmosAnswerError`；`<BOM><BOM>#42\n`（双 BOM）→ `CosmosAnswerError`；`\n<BOM>#42`（BOM 非开头）→ `CosmosAnswerError`。

### B5. 前导内容的变体（严格）
- **规则**：第一行内容必须**恰好** 3 字符 `#42`，无任何变体。
- 正例：`#42` + 行终止符。
- 反例（全部 `CosmosAnswerError`）：`# 42`（有空格）、`#42abc`、`#43`、`#42 `（尾随空格）、`#42\t`、`##42`、`#４２`（全角）、`#42;`、`#42//x`。

### B6. `#` 出现在程序中间
- **规则**：CODE 模式下 `#` 非法，除非处于前导位。**字符串内 `#` 合法**；注释内 `#` 合法；格式说明符内 `#` 合法（§2.2、§2.8）。
- 正例：`print("#42")`、`print("# a # b")`、`// #42 这是注释`、`"${x:#>5}"`。
- 反例：`let x = 1 # 2` → `SyntaxError`（非法字符 '#'）；第 2 行 `#42` → `SyntaxError`；`f(#)` → `SyntaxError`。

### B7. CRLF / LF / CR 三种行终止符
- **规则**：三者均接受，加载器归一化为 `\n`。
- 正例：`#42\r\nlet x = 1\r\nprint(x)\r\n`、`#42\n…`、`#42\r…`。
- 反例：`#42<U+2028>…`（行分隔符非三种之一）→ `CosmosAnswerError`；混合 `\r\n` 与裸 `\r` 仍合法（各自归一化）。

### B8. 非 UTF-8 编码
- **规则**：**先编码校验、后前导校验**；非 UTF-8 → `SyntaxError`（非 `CosmosAnswerError`）。
- 正例：UTF-8（带/不带 BOM）皆可。
- 反例：文件含非法字节 `0xFF 0xFE` → `SyntaxError: 文件不是合法的 UTF-8 编码（首个非法字节位于 0x…）`。

### B9. REPL / stdin / `lfz run -e` / 非 `.lfz` 文件 豁免
- **规则**：只有**扩展名为 `.lfz` 的文件**要求前导（§2.2.0）；REPL / stdin / `-e`（无路径）与**非 `.lfz` 文件**均**豁免**。
- 正例：`lfz run -e 'print(1 + 2)'`、`echo 'print(1)' | lfz run -`、REPL 输入 `let x = 1`、`lfz run note.txt`（非 `.lfz` 文件）→ 均**不**要求 `#42`。
- 反例：无（豁免是定义）。
- **附则**：`-e`/stdin/REPL 的出错位置伪路径分别为 `<command>` / `<stdin>` / `<repl>`（§8.3）；非 `.lfz` 文件出错仍用其**真实路径**。

### B10. `CosmosAnswerError` 的位置信息
- **规则**：无前导时，位置**固定为第 1 行**；**不显示源码行、不显示插入符、不显示编号/hint**（**v0.4 已定**）。
- 正例（输出）：
  ```
  File "forgot.lfz", line 1
  CosmosAnswerError: 你忘记了宇宙的答案
  ```
- 反例：无（输出格式唯一）。
- 说明：不加 `Traceback` 头（加载期错误，§8.2）。

### B11. `lfz init/fmt/test/check` 是否统一查头（**v0.4 改为按扩展名**）
- **统一原则（规范性）**：**"`.lfz` 文件即需前导"（file with ext `.lfz` ⇒ header）**。凡以**扩展名为 `.lfz`（ASCII 大小写不敏感）的文件**身份进入加载器的输入，**一律**校验前导；**非 `.lfz` 文件**与 **REPL/stdin/`-e`** 一律豁免。因此**所有子命令自动一致**。
- 分命令约定（工具链表，v0.4 更新）：
  | 命令 | v1 状态 | 是否查头 | 失败行为 |
  |---|---|---|---|
  | `lfz run <file.lfz>` | ✅ 实现 | **是**（扩展名 `.lfz`） | `CosmosAnswerError`，退出码 2 |
  | `lfz run <file.txt>`（非 `.lfz`） | ✅ 实现 | **否（扩展名豁免）** | 不因缺前导报错；按普通源加载执行 |
  | `lfz test` | ✅ 实现 | **是**（每个 `.lfz` 测试文件；非 `.lfz` 夹具豁免） | 记该用例为 error，退出码 2 |
  | `lfz check <file>` | ⏸ 延后 | 同 `run`（按扩展名） | 同 `run` |
  | `lfz fmt <file>` | ⏸ 延后 | 同 `run`（按扩展名；fmt **不**自动补前导） | `CosmosAnswerError`，退出码 2 |
  | `lfz init` | ⏸ 延后 | 生成的项目 `.lfz` 模板**自动含 `#42` 首行**（新手不踩坑） | — |
  | REPL / stdin / `-e` | ✅/⏸ | **否（豁免）** | — |
- 正例：`lfz run ok.lfz`（有头）→ 正常；`lfz run plain.txt`（无头非 `.lfz`）→ 正常；`lfz test`（全部测试为 `.lfz` 且带头）→ 正常。
- 反例：`lfz run noheader.lfz` → `CosmosAnswerError`，退出码 2；`lfz run plain.txt` **不会**因缺前导报错。

### B12. 前导的字节精确性与"唯一入口"
- **规则**：`.lfz` 文件的 `#42` 校验在**解码后、词法前**完成；`#` 不进入记号流。lexer 永不产生 `#` 记号。
- 正例：`#42` 首行被消费且**不产出 `NEWLINE` 记号**（程序体首行的行号仍从 2 起，由 `line_base = 1` 保证）。
- 反例：任何试图把 `#42` 当作"注释/记号"的解析（如 `//#42`、`x = #42`）→ `CosmosAnswerError` 或 `SyntaxError`（依法）。

**审计条目数：12（B1–B12）。**

---

## 7. EBNF（ISO EBNF，v0.3 修订版）

```
(* ============================================================
   LFZ v0.3 grammar — ISO EBNF
   记法: = 定义 ; , 连接 ; | 选择 ; { } 0..n ; [ ] 可选 ; " " 终结符
   约定:
   - 加载器（loader）在词法前完成: UTF-8 校验 -> 跳 BOM -> 归一化行终止符
     -> 若扩展名为 .lfz 则校验并消费 preamble (否则跳过)。违反 -> CosmosAnswerError。
   - NEWLINE 是否有效由「解析器换行模式栈」决定（§3.2），非词法统一处理。
   - 块仅在 block-required 位；其余 { 为 struct 字面量（§3.3）。
   - if/while 条件位、for 可迭代位禁用裸 struct 字面量（§3.4）。
   - 所有二元运算符左结合；优先级见 §4.1。
   - '#' 不是记号: 仅出现在 preamble 位；CODE 模式下出现即非法字符。
   ============================================================ *)

program        = [ preamble ] , { NEWLINE | statement } , EOF ;
(* 规范性 mode 区分 (v0.4 三档, 按扩展名):
   - .lfz 文件 (ext 按 ASCII 大小写不敏感 == "lfz"): preamble 必须出现,
     否则 CosmosAnswerError。
   - 非 .lfz 文件: preamble 必须缺失(省略); 仍是文件模式
     (UTF-8 校验 / 跳 BOM / 行终止符归一化 / 错误用真实路径)。
   - 非文件入口 (REPL / stdin / -e): preamble 必须缺失(省略)。
   加载器消费 preamble 行, 不产出任何记号 (含不产出 NEWLINE);
   并返回 line_base (=1 当且仅当消费了 preamble 行);
   词法行号 = 本地行号 + line_base。 *)

preamble       = "#42" , NEWLINE ;      (* 内容严格, 无变体; 见 §6-B5 *)

block          = "{" , { NEWLINE | statement } , "}" ;
(* 规范性: 相邻两 statement 之间必须 >=1 个 NEWLINE; statement 后可直接为 "}" 或 EOF。
   违反 -> SyntaxError。 *)

statement      = decl_stmt | assign_stmt | fn_decl | struct_decl
               | while_stmt | for_stmt
               | return_stmt | break_stmt | continue_stmt
               | dump_stmt | expr_stmt ;

decl_stmt      = ( "let" | "var" ) , IDENT , "=" , expression ;
assign_stmt    = lvalue , assign_op , expression ;
assign_op      = "=" | "+=" | "-=" | "*=" | "/=" | "%=" ;
lvalue         = IDENT , { "." , IDENT | "[" , expression , "]" } ;

fn_decl        = "fn" , IDENT , "(" , [ params ] , ")" , body ;
body           = block | "=>" , expression ;
struct_decl    = "struct" , IDENT , "{" , [ member_list ] , "}" ;
member_list    = member , { "," , member } , [ "," ] ;
member         = fn_decl | IDENT , ":" , expression ;

while_stmt     = "while" , expression , block ;
for_stmt       = "for" , IDENT , "in" , expression , block ;
return_stmt    = "return" , [ expression ] ;          (* 换行即返回空 *)
break_stmt     = "break" ;
continue_stmt  = "continue" ;
dump_stmt      = ";;" ;                                (* 独占逻辑行 *)
expr_stmt      = expression ;

params         = IDENT , { "," , IDENT } ;

(* -------- 表达式: 高 precedence -> 低 -------- *)
expression     = or_expr ;
or_expr        = and_expr , { "||" , and_expr } ;
and_expr       = eq_expr  , { "&&" , eq_expr } ;
eq_expr        = cmp_expr , { ( "==" | "!=" ) , cmp_expr } ;
cmp_expr       = pipe_expr , { ( "<" | "<=" | ">" | ">=" ) , pipe_expr } ;
pipe_expr      = add_expr , { "|>" , pipe_rhs } ;
add_expr       = mul_expr , { ( "+" | "-" ) , mul_expr } ;
mul_expr       = unary    , { ( "*" | "/" | "%" ) , unary } ;
unary          = ( "-" | "!" ) , unary | postfix | if_expr | lambda ;
postfix        = primary , { call | index | field } ;
call           = "(" , [ args ] , ")" ;
index          = "[" , expression , "]" ;
field          = "." , IDENT ;
args           = expression , { "," , expression } , [ "," ] ;
pipe_rhs       = postfix | lambda ;

primary        = INT | FLOAT | STRING
               | "true" | "false" | "nil" | "self"
               | IDENT | "_"
               | array_lit | struct_lit
               | "(" , expression , ")" ;

array_lit      = "[" , [ args ] , "]" ;
struct_lit     = [ IDENT ] , "{" , [ field_list ] , "}" ;
field_list     = field_init , { "," , field_init } , [ "," ] ;
field_init     = ( IDENT | STRING ) , ":" , expression ;

if_expr        = "if" , expression , block ,
                 [ "else" , ( if_expr | block ) ] ;
lambda         = "fn" , "(" , [ params ] , ")" , body
               | "(" , [ params ] , ")" , "=>" , ( expression | block ) ;

(* -------- 词法 -------- *)
IDENT          = ( letter | "_" ) , { letter | digit | "_" } ;   (* 单独 "_" 除外 *)
letter         = "a" | ... | "z" | "A" | ... | "Z" ;
digit          = "0" | ... | "9" ;
INT            = dec_int | "0x" , hex_digits | "0b" , bin_digits | "0o" , oct_digits ;
dec_int        = digit , { digit | "_" } ;
FLOAT          = digit , { digit | "_" } , "." , digit , { digit | "_" } , [ exponent ]
               | digit , { digit | "_" } , exponent ;
exponent       = ( "e" | "E" ) , [ "+" | "-" ] , digit , { digit } ;
NEWLINE        = "\n" ;                 (* 加载器已把 \r\n / \r 归一化为 \n *)
COMMENT        = "//" , { any_except_newline } | "/*" , { any } , "*/" ;

(* 字符串: 结构化记号（lexer 产出, parser 组合） *)
STRING         = STRING_BEGIN , { TEXT | interpolation } , STRING_END ;
interpolation  = INTERP_BEGIN , expression , [ ":" , format_spec ] , INTERP_END ;
format_spec    = [ [ fill ] , align ] , [ sign ] , [ width ] ,
                 [ "." , precision ] , [ type ] ;
align          = "<" | ">" | "^" ;
type           = "d" | "x" | "X" | "o" | "b" | "f" | "e" | "s" ;
```

**maximal-munch 记号表**：见 §2.3。**错误类表**：见 §8。

---

## 8. 错误模型（v0.3 全量改写 · Python 风格）

### 8.1 错误类清单（规范，共 11 类 + 1 基类）

> **取消 `E-xxx` 编号**：用户可见输出**不再**出现编号；`--json` 的机器可读字段使用**类名**。基类 `LfzError` 不被直接抛出。

| 错误类（class） | 触发条件（充分必要） | 中文消息模板 | 阶段 |
|---|---|---|---|
| `LfzError`（基类） | 不直接抛出 | — | — |
| `CosmosAnswerError` | **`.lfz`** 文件缺少合法 `#42` 前导（§2.2.2 `is_lfz` 分支 a–c 任一失败） | **`你忘记了宇宙的答案`**（固定，无参数） | 加载 |
| `SyntaxError` | 词法/语法错误：非法字符、字符串未闭合、意外记号、表达式未结束、单 `;`、同行两语句、赋值目标非法、`#` 位置非法、`_` 位置非法、管道多 `_`、break/continue 在循环外、return 在函数外、非 UTF-8 编码 | 见下表（细分消息） | 加载/解析 |
| `NameError` | 引用未定义的名字 | `未定义的名字 '{name}'` | 运行 |
| `TypeError` | 运算符/条件/调用/参数/格式说明符的**类型**不符；调用非函数；管道右侧非函数；参数个数不符 | 见下表 | 运行 |
| `IndexError` | 数组下标越界（含负索引规范化后越界） | `下标 {i} 越界（长度 {n}）` | 运行 |
| `FieldError` | struct 不存在该字段（`.字段` 或 `["键"]` 读取缺失键） | `结构体没有字段 '{name}'` | 运行 |
| `ZeroDivisionError` | 整数/浮点 `/` 或 `%` 的除数为零 | `除以零` / `对零取模` | 运行 |
| `OverflowError` | `int` 运算结果超出 i64 范围 | `整数溢出：结果超出 i64 范围` | 运行 |
| `ValueError` | 显式转换失败（`int("abc")` 等）；格式说明符语法非法 | `无法把 {src} 转换为 {dst}（'{text}'）` / `格式说明符非法：'{spec}'` | 运行 |
| `IOError` | `input()` 遇 EOF；不可读文件等 | `输入结束（EOF）` / `无法读取：{path}` | 运行 |
| `AssertionError` | `assert(cond, msg)` / `check(cond, msg)` 失败 | `断言失败：{msg}` / `check 失败：{msg}` | 运行 |

**`SyntaxError` 细分消息（中文）**：

| 子场景 | 消息模板 |
|---|---|
| 非法字符 | `非法字符 '{c}'` |
| `#` 位置非法 | `'#' 只能出现在文件首行的前导位；(字符串 / 注释 / 格式说明符内的 '#' 除外)` |
| 字符串未闭合（EOF 或裸换行） | `字符串字面量在此处未闭合` |
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

**`TypeError` 细分消息（中文）**：

| 子场景 | 消息模板 |
|---|---|
| 运算符类型不符 | `运算符 '{op}' 不支持 {lt} 与 {rt}` |
| 条件非 bool | `条件必须是 bool，得到 {t}` |
| 调用非函数 | `不可调用：{t} 不是函数` |
| 管道右侧非函数 | `管道右侧必须是函数，得到 {t}` |
| 参数个数不符 | `函数 {name} 期待 {n} 个参数，得到 {m}` |
| 格式说明符与值类型不符 | `格式说明符 '{spec}' 不适用于 {t}` |

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
- **运行期错误**（其余 9 类）：**有** `Traceback (most recent call last):` 头 → 逐帧（**最外层在前、最内层在后**）→ 末行 `<类名>: <消息>`。

**退出码（沿用 D-008）**：`0` 成功；`1` 测试失败；**所有错误类 → `2`**。`--json` 时错误仍退出 `2`。

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

---

## 9. 样例程序（v0.3 语法，**全部含 `#42` 首行**，含 `;;`）

### 9.1 样例 A —— 学生成绩统计（完整程序，含 `;;`）

```
#42
// ============================================================
// grades.lfz — 学生成绩统计（v0.3 语法）
// 覆盖: struct/方法、array、函数、while/for/if、
//       字符串插值、管道 |>、assert、基本 IO、;; 变量 dump
// ============================================================

struct Student {
    name: "",
    score: 0,
    fn isTop() => self.score >= 90,
    fn line() => "${self.name}: ${self.score:>3}",
}

fn sortDesc(xs) {
    var pool = xs
    var out = []
    while len(pool) > 0 {
        var best = 0
        for i in range(len(pool)) {
            if pool[i].score > pool[best].score { best = i }
        }
        out = push(pool[best], out)
        pool = removeAt(best, pool)
    }
    out
}

fn average(xs) {
    assert(len(xs) > 0, "average() 需要非空数组")
    var total = 0.0
    for s in xs { total += float(s.score) }
    total / float(len(xs))
}

let roster = [
    Student { name: "Alice", score: 93 },
    Student { name: "Bob",   score: 67 },
    Student { name: "Cara",  score: 88 },
]

;;                                    // ← 变量 dump（独占一行）

print("原始名单：")
for s in roster { print("  " + s.line()) }

let ranked = roster |> sortDesc()

print("")
print("排名：")
var rank = 1
for s in ranked {
    let star = if s.isTop() { "★" } else { "·" }
    print("  ${rank}. ${star} ${s.line()}")
    rank += 1
}

print("")
print("平均分 = ${average(roster):.2f}")
let top = ranked |> maxBy((s) => s.score)
print("最高分 = ${top.name} (${top.score})")
```

`;;` 处（stdout，内→外，同层按声明序）预期：

```
Student ： <struct Student>
sortDesc ： <fn sortDesc>
average ： <fn average>
roster ： [{name: "Alice", score: 93}, {name: "Bob", score: 67}, {name: "Cara", score: 88}]
```

### 9.2 样例 B —— 词频统计（完整程序，含 `;;`、多行管道）

```
#42
// ============================================================
// words.lfz — 词频统计（v0.3 语法）
// 覆盖: 匿名 struct（字典面孔）、struct 方法、动态字段、
//       多行管道链（括号续行）、格式说明符、check、;; dump
// ============================================================

struct Word {
    text: "",
    count: 0,
    fn bump() { self.count += 1 },
    fn row() => "${self.text:>10} │ ${self.count:>3}",
}

fn countWords(text) {
    var table = {}
    for w in text |> split(" ") {
        if has(w, table) {
            table[w].bump()
        } else {
            table[w] = Word { text: w, count: 1 }
        }
    }
    table
}

let text = "the quick the fox the dog a quick fox"
let freq = countWords(text)

;;                                    // ← 打印当前可见变量

check(len(freq) == 5, "期望 5 个不同的词，实得 ${len(freq)}")

print("词频 Top 3")
let top3 = (freq                       // ← 多行管道必须括号化
    |> values()
    |> sortBy((w) => -w.count)
    |> take(3))
for w in top3 {
    print("  " + w.row())
}

// 格式说明符一览
let pi = 3.14159265
let n = 42
print("pi=${pi:.3f}  n=${n:05d}  hex=${n:x}  right=${n:>6}")
```

`;;` 处（stdout）预期：

```
Word ： <struct Word>
countWords ： <fn countWords>
text ： the quick the fox the dog a quick fox
freq ： {a: {count: 1, text: "a"}, dog: {count: 1, text: "dog"}, fox: {count: 2, text: "fox"}, quick: {count: 2, text: "quick"}, the: {count: 3, text: "the"}}
```

程序主体预期输出：

```
词频 Top 3
         the │   3
       quick │   2
         fox │   2
pi=3.142  n=00042  hex=2a  right=    42
```

### 9.3 样例 C —— 排序可视化核心片段（服务 P8 应用，片段；作为文件时首行亦为 `#42`）

```
#42
// bars.lfz — ASCII 排序可视化核心片段（v0.3 语法）
print("\e[2J\e[H")                     // ANSI 清屏+归位（\e = ESC 0x1B）
seed(12345)                            // 固定随机种子 → 演示可复现
var xs = range(40) |> map((i) => randInt(5, 100))

fn draw(a, hi) {
    var out = "\e[H"
    for i in range(len(a)) {
        let bar = "█" * a[i]           // string * int = 重复
        out += if i == hi { "${bar}  << " } else { bar + "\n" }
    }
    print(out)
}

for i in range(len(xs)) {
    for j in range(len(xs) - i - 1) {
        if xs[j] > xs[j + 1] { xs = xs |> swap(j, j + 1) }
        draw(xs, j)
    }
}
;;
print("排序完成")
```

---

## 10. 对 Rust 实现与 DebugSym 的影响（D-011）

### 10.1 新增模块：loader（加载器）

- 新模块 `loader`（建议 `src/loader.rs`），职责（§2.2）：读文件 → UTF-8 校验（失败 → `SyntaxError`）→ 跳 BOM → 归一化行终止符 → **按扩展名判定 `ext(path) == "lfz"`（ASCII 大小写不敏感）** → 仅当是 `.lfz` 时校验/消费 `#42` 前导（失败 → `CosmosAnswerError`）。
- 对外暴露两类入口（**v0.4：`load_file` 内部按扩展名分流**）：
  - `load_file(path) -> Result<Loaded, LfzError>`：扩展名 `.lfz` → **要求前导**；否则 → 与 `load_source` 同等（不要求前导）。`Loaded = { text: String, line_base: u32 }`（`line_base` = 1 当且仅当消费了前导行）。
  - `load_source(text) -> Result<Loaded, LfzError>`：**不要求前导**（REPL / stdin / `-e`），`line_base = 0`。
- 前导**不产出 token**；行号 = 本地行号 + `line_base`（`.lfz` 程序体自 line 2 起；非 `.lfz` / non-file 自 line 1 起）。

### 10.2 AST 必须携带源码位置（新增硬要求）

- Python 风格报错要求**行列信息**，故 AST 节点**必须**携带 `span`：
  ```rust
  struct Span { line: u32 /*1-based; = 本地行号 + loader.line_base*/, col: u32 /*1-based, Unicode 标量计数*/ }
  ```
- 每个可抛错节点（表达式、语句、记号对应的字面量）**至少**记录其**起始** `Span`。
- 建议在 parser 里维护 `last_span`（当前/最近记号起点），节点构造时填入。

### 10.3 调用栈（traceback）

- 运行时需维护**调用帧栈**：
  ```rust
  struct CallFrame { func: Rc<str>, /* "<module>" 顶层 */ span: Span }
  ```
- 进入函数调用压帧、返回弹帧；错误发生时把帧栈自**最外层到最内层**序列化进 `LfzError`。
- `Rc<Closure>` 需保留**函数名**（`<fn 名字>` / 匿名 `<fn>`）。

### 10.4 错误枚举

```rust
enum LfzError {
    CosmosAnswer,                         // 你忘记了宇宙的答案
    Syntax   { msg: SyntaxMsg, span: Span },
    Name     { name: String, span: Span },
    Type     { msg: TypeMsg, span: Span },
    Index    { idx: i64, len: usize, span: Span },
    Field    { name: String, span: Span },
    DivZero  { modulo: bool, span: Span },
    Overflow { span: Span },
    Value    { msg: ValueMsg, span: Span },
    Io       { msg: String, span: Option<Span> },
    Assert   { msg: String, span: Span },
}
```

- `fn class_name(&self) -> &'static str`：返回**类名**（`"SyntaxError"` / `"ZeroDivisionError"` / …），`--json` 用此字段；**不再**有 `E-xxx` 字符串码。
- `fn message(&self) -> String`：返回**中文消息**（§8.1）。
- `fn span(&self) -> Option<Span>`：位置。
- **显示层**（CLI）据 `class_name()` 是否为 `"CosmosAnswerError"`/`"SyntaxError"`（加载/解析期）决定是否打印 `Traceback` 头（§8.2）。

### 10.5 DebugSym（职责不变，仍服务 `;;`）

- 名字解析阶段为每个作用域分配**槽位**并保留 `DebugSym { slots: Vec<String> }`（slot→名字，按声明序）；每个函数/闭包携带其 `DebugSym`。
- `DUMP` 指令在运行时遍历环境链，用各层 `DebugSym` 反查名字。
- **与 traceback 的区别**：`DebugSym` 供**局部变量名**；`CallFrame.func` 供**函数名**。两者互补，均须写入 `interface-contract.md`。

### 10.6 其它

- lexer：模式栈 `CODE/STR/INTERP`（§2.8）；最大匹配表（§2.3）；CODE 模式下 `#` → `SyntaxError`。
- parser：换行模式栈 `SIG/IGN`（§3.2）+ NO_BRACE_LITERAL 限制位（§3.4）；`;;` 生成 `Dump` 节点；管道脱糖为 `Call`；每节点填 `Span`。
- CLI：仅做**错误格式化**与**退出码映射**（0/1/2）；`--json` 输出 §8.3 示例 4 的字段。

---

## 11. 对下游角色的硬性要求（规范，供 ai-dx / test-engineer / docs / app 执行）★ v0.3 新增；v0.4 修订（11.1/11.2/11.5，按扩展名）

### 11.1 AI 开发指南 / skill（ai-dx-engineer）

AI 指南与 `lfz-programming` skill（计划路径 `.opencode/skills/lfz-programming/`）**必须写死**以下内容：

1. **`#42` 首行规则（置顶）**：任何 **`.lfz` 文件**的**第一行必须恰好是 `#42`** 三个字符，其后紧跟一个行终止符；否则程序以
   `CosmosAnswerError: 你忘记了宇宙的答案` 终止。**触发看扩展名**：仅 `.lfz`（大小写不敏感）；非 `.lfz` 文件与 REPL / stdin / `lfz run -e` **都不需要** `#42`。（若给出片段示例，须注明"作为 `.lfz` 文件时首行须为 `#42`"。）
2. **错误类清单（逐条列出）**：`CosmosAnswerError` / `SyntaxError` / `NameError` / `TypeError` / `IndexError` / `FieldError` / `ZeroDivisionError` / `OverflowError` / `ValueError` / `IOError` / `AssertionError`（+ 基类 `LfzError`），每条附**触发条件**与**中文消息示例**。
3. **所有代码示例必须含 `#42` 首行**（含片段示例；若为片段则注明"文件首行须为 `#42`"）。
4. **禁止出现旧 `E-xxx` 错误码**（已取消）；错误一律以类名 + 中文消息呈现。

### 11.2 test-runner 契约（tooling-dev 定稿 → test-engineer 使用）

- **T-R1**：所有被 runner 作为 LFZ 程序加载的 **`.lfz` 测试文件**必须是带 `#42` 首行的合法文件；缺失 → 该用例记为 **error**（退出码 2）。**非 `.lfz` 夹具按扩展名豁免**（不校验前导）。
- **T-R2**：对"缺前导 / 非法前导"这类**负例**，**不得**放入自动发现目录（否则会污染正常用例集）；须用**非自动发现的夹具**（建议 `tests/fixtures/`）并在清单（如 `tests/cases.json`）中声明 `"expect": {"error": "CosmosAnswerError"}`。
- **T-R3**：runner 的失败输出使用 §8.2 的位置信息；断言失败（`assert`/`check`）以 `AssertionError` 记 failure/error，退出码按 D-008（测试失败 = 1）。
- **T-R4**：runner 发现测试文件的规则须明确写入契约（建议 glob 如 `tests/**/*.lfz`，并排除 `tests/fixtures/**`）；**仅 `.lfz` 文件要求前导**（与 §2.2.0 一致）。

### 11.3 人类文档（docs-writer）

- 手册必须包含 **`#42` 仪式**（为什么、唯一合法形式、REPL/stdin/`-e` 例外）与 **Python 风格错误模型**（类清单 + 3 个输出示例 + `--json`）。
- 手册中所有 LFZ 代码块**必须含 `#42` 首行**。

### 11.4 应用（app-dev）

- `app/` 下全部 `.lfz` 源文件**必须含 `#42` 首行**；片段复用（§9.3）作为文件时亦然。

### 11.5 spec 三件套（language-architect 本人，后续）

- 冻结时把本草案拆入 `docs/spec/{syntax,semantics,interface-contract}.md`（**计划路径，当前不存在**）：
  - `syntax.md`：§2–§7。
  - `semantics.md`：§3.6–§3.7、§4、§8。
  - `interface-contract.md`：§8.1 错误类 + §10 的 `Span` / `CallFrame` / `LfzError` / `DebugSym` / loader 接口 + **`ext(path)` 扩展名判定** + **`Loaded{text,line_base}` / `line_base` 行号语义**。

---

## 12. 与 D-007 / D-011 / 用户指令的一致性核对

| 约束 | v0.3 状态 |
|---|---|
| D-007 5 特色（管道 / 合一 struct / 富插值 / 结构化错误 + assert·check / 确定性语义） | ✅ 全保留（§4.3、§3.7、§2.8、§8、§4.2）；**特色 4 升级为类名 + 中文消息 + traceback + `--json`**（编号取消，符合用户新指令） |
| D-007 必做核心（int/float/string/bool/nil/array/struct/function、let/var、运算符、if/while/for、函数、IO、注释） | ✅ 全覆盖（§2–§8） |
| D-007 "换行终结" | ✅ 保持（§3） |
| D-011 实现语言 = Rust | ✅ §10 已按 Rust 调整（loader / `Result` / `enum LfzError` / `Rc<RefCell>` / `Span` / `CallFrame`） |
| D-011 float 纳入 + int→float 无损加宽 | ✅ §4.2 |
| 用户指令：花括号块 / 换行语句 / `;;` dump / `${}` 插值 | ✅ §3、§2.8 |
| **用户指令 A：文件前导 `#42`** | ✅ §2.2、§6-B1~B12、§7、§9、§11 |
| **用户指令 B：Python 风格错误模型 + `CosmosAnswerError`** | ✅ §8（全量改写） |
| **用户指令 C：可移植性（三种行终止符 / BOM / UTF-8）** | ✅ §2.1、§6-B4/B7/B8 |
| 用户指令 5：无歧义 | ✅ §5（26 条）+ §6（12 条） |
| **用户拍板 1：前导仅 `.lfz` 扩展名生效（大小写不敏感）** | ✅ §2.2.0、B11、§7、§10.1、§11 |
| **用户拍板 2：`CosmosAnswerError` 不显示第 1 行原文** | ✅ §8.2、B10（**已定**） |
| **用户拍板 3：程序体可空（`#42` + 行终止符）** | ✅ B2、§7（**已定**） |
| **用户拍板 4：不输出 `hint`/`提示：` 行** | ✅ §8.1（明确写死）；v1.1 backlog |
| **用户拍板 5：`#42` 后必须紧跟行终止符** | ✅ §2.2.1、B2（**已定**） |

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
| （v1）assert | `AssertionError` | |
| —（新增） | `CosmosAnswerError` | 缺 `#42` 前导；消息固定「你忘记了宇宙的答案」 |

> `E-SYN-004` 在 v0.2 即保留未用；v0.3 随编号制一并取消。

---

## 14. 仍待决项（v0.4 收敛后）

> **v0.3 §14 的 5 项开放问题已全部闭合（用户拍板）**，见 §0「v0.3 → v0.4 差异摘要」与 DECISIONS **D-014**。以下仅剩**沿用 v0.2/v0.3 默认的遗留项**：均为**当前默认、仍可覆盖**，**不阻塞 v1 冻结（§0 冻结前提）**。

**遗留默认 4 条（当前默认，仍可覆盖）**：

1. **`;;` 作用域范围**：当前 = **完整可见链**（最内层→外层，含块作用域 / 函数局部 / 闭包捕获 / 模块顶层；遮蔽去重）；备用 = 仅当前函数。§3.6。
2. **`;;` 输出通道**：当前 = **stdout**（与 `print` 同通道，按执行顺序交织）；备用 = stderr。§3.6。
3. **多行续行糖**：当前 = **仅括号**（`()` / `[]` / 字面体·声明体 `{}` 内换行忽略）；备用 = 允许行尾运算符续行 / 行首 `.`·`|>` 续行。§3.1、A2、A17。
4. **单变量打印形式**：当前 = **单 `;` 一律 `SyntaxError`**（无 `;name` 形式）；备用 = `;name` 打印单个变量。A11、A12。

**新增未决项：无**（v0.4 未引入新的未决项：扩展名判定、空程序体、hint、`CosmosAnswerError` 显示、前导行终止符 5 项均已闭合）。

> 其余全部按本文件默认（含 5 特色、Rust、float、int→float 加宽、`|>` 尾参注入 data-last、i64 溢出即错、负索引、深相等）。

---

## 15. 对 core-dev 审计的对账（保留 v0.2 §9 结论；错误码统一替换为类名）

> v0.2 §9 的 10 条对账结论在 v0.3 **全部维持**，仅把错误码替换为类名；v0.4 维持不变。摘要：

- **#1 管道优先级矛盾（真 bug）**：采纳修正层序（§4.4）；`xs |> sum() + 1` 为 `SyntaxError`。
- **#2 `if c { … }` 吞块**：采纳 Rust 的 NO_BRACE_LITERAL（§3.4、A6）。
- **#3 `;` / `;;` 记号化**：最长匹配；单 `;` 等 → `SyntaxError`（A10–A12）。
- **#4 `{}` 双角色 + 换行由 parser 管理**：NL 模式栈 `SIG/IGN`（§3.2）。
- **#5 换行终结（ASI）**：`return` 行尾 = 返回 `nil`；**括号是唯一续行手段**（部分不同意 core-dev 的续行组合，见 §14 遗留默认 3）。
- **#6 悬挂 else + `else` 前换行**：保存位置→跳过换行→看 `else`，否则回退（§3.5、A4）。
- **#7 一元负号 + 最大匹配表**：单记号 `-`；禁止合成 `>> << ++ -- -> .. ?? ::`（§2.3、A16、A25）。
- **#8 字符串插值模式栈 + 结构化记号**：`STRING_BEGIN/TEXT/INTERP_BEGIN/…/INTERP_END/STRING_END`（§2.8）。
- **#9 `>>` 不合成**：拆成两个 `>`（A25）。
- **#10 分隔符一致性**：struct 声明体与字面体、数组、实参一律**逗号必填**（A7、A8）。

---

*（本文件为 v0.4 定稿候选：用户本轮 5 项拍板已逐条落实，§14 仅剩 4 条遗留默认（仍可覆盖）。满足 §0 冻结前提后即拆分为 `docs/spec/` 三件套并冻结 v1，冻结走 ADR。）*
