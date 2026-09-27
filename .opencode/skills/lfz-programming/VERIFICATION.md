# `lfz-programming` SKILL — 实测验证记录（ai-dx-engineer）

> 证据文化：无证据不算完成。本文件记录「只凭 `SKILL.md` 能否写出可运行 LFZ 程序」的**实测**闭环。
> 基线：`docs/spec/` 冻结 v1、解释器 `cargo run` 可用（P3/P4 已完成）。命令均在**仓库根目录**、PowerShell 下执行（先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`）。
> 日期：2026-09-27（§7 第 4 次实测于 P7c 增量交付补充）。

---

## 1. 交付要求 → SKILL 落点对照（本包交付证据）

本包同时满足**作业要求（`task-info.md` 第 4 条）**与 **`docs/spec/interface-contract.md` §11.1 五条硬性要求**。逐条如下。

### 1.1 作业要求第 4 条（`task-info.md`）

> 原文：「设计 **ZS（=本项目 LFZ）语言**的开发指南（**例如 Skills 等文档**）**供编程 Agent 使用**，使得编程 Agent 能够**使用 LFZ 语言顺利编写代码**。」

| 要求拆解 | SKILL / 本包落点 | 证据 |
|---|---|---|
| (a) 产出「开发指南」且以 **Skill 等形式** | `.opencode/skills/lfz-programming/` 是一个标准 opencode Skill（`SKILL.md` 带 frontmatter `name`/`description`），并附 `README.md`（安装）、`prompt-template.md`（模板）、`VERIFICATION.md`（证据） | `SKILL.md` L1–L4 含 `name: lfz-programming` + `description`；包内 4 文件 |
| (b) **面向编程 Agent** | `SKILL.md` 开篇声明「读者：未接触过 LFZ 的编程 Agent」；**硬纪律**「不要凭记忆写 LFZ」；写作遵循 AI 友好原则（陷阱前置、正反例对照、完整可运行示例） | `SKILL.md` 顶部读者声明 + 🚫 硬纪律；§4 21 条陷阱"错误→结果→正确"三段式 |
| (c) 让 Agent 能**顺利编写代码**（可运行） | §0.5「Agent 标准工作流（5 步）」把写码变成**可执行流程**（第 4 步**强制实跑、必须 exit 0**）；§2/§4.5 帮 Agent 自诊断报错；§7 分级示例含真实输出；§8 自检清单；本文件 §2/§7 提供**实跑证据** | §0.5 五步；§4.5「错误类→原因→修法」27 行对照；§7 L1–L4；VERIFICATION §2/§7 |
| (d) 是**指南**而非新语法定义 | 全篇标注「事实源 `docs/spec/`；本文件是速查/转述，不定义新语法；冲突以 spec 为准」 | `SKILL.md` 顶部 + §10；`README.md` §4 |

### 1.2 `interface-contract.md` §11.1 五条硬性要求

| §11.1 要求 | SKILL 中的位置 | 落实方式 |
|---|---|---|
| ① `#42` 首行规则（置顶） | **§1「铁律」**（正文第一节） | 恰 3 字符 `#42` + 行终止符；违反 → `CosmosAnswerError: 你忘记了宇宙的答案`；**仅按扩展名 `.lfz`（大小写不敏感）**触发；非 `.lfz`/REPL/stdin/`-e` 豁免；列出 `# 42`/`#42 `/`#43`/`##42`/前导空白/仅 3 字节/空文件等全部变体反例；BOM 豁免 |
| ② 错误类清单（逐条：触发条件 + 中文消息示例） | **§2「错误类清单」** | 12 个具体类 + 基类 `LfzError`，逐行给出「触发条件 / 中文消息示例 / 阶段」，并附 10 条实测消息样本 |
| ③ 所有代码示例必须含 `#42` 首行（片段须注明） | **§3 / §4 / §7** | 所有完整程序示例首行均为 `#42`；§3/§4 骨架片段在块前统一注明「若作为 `.lfz` 文件，首行须为 `#42`」；§9 提示模板要求输出含 `#42` |
| ④ 禁止出现旧 `E-xxx` 错误码 | **§2 表头声明 + §8 自检清单 + 全文** | 明确「只有类名 + 中文消息，不存在任何编号错误码」；SKILL / README / 本文件全文均不含任何 `E-` 编号错误码 |
| ⑤ `check` 非致命 / `assert` 致命（A4） | **§5「check 非致命 / assert 致命」** | 表格写死三者行为 + 实测 `check(1==2)` → stdout `continue` / stderr `check 失败：soft` / 退出码 `0` |

### 1.3 结论

两条来源的要求**均达标**（无缺口）。P7c 增量为强化 (c) 与"最后一公里"：新增 `SKILL.md` §0.5 工作流、§4.5 错误→修法表、顶部硬纪律，新增 `README.md`（三种安装方式），并把 §7 扩到 L4（新实跑程序）。

### 1.4 T11-③ 增量 → 要求 (c) 追溯（D1–D7 + O(n) 构建）

盲测 2/8 暴露的 7 条缺口（`FEATURE-AUDIT.md` §6.2）全部落在要求 **(c)「让 Agent 能顺利编写代码」**上；本次逐条落点与实测证据如下（证据全文见本文件 §8）：

| 缺口 | SKILL 落点 | 实测证据 |
|---|---|---|
| D1 字符串不可下标 + `split("", s)` 惯用法 | §4-22 / §4.5 / §6 `split` / L5 | §8.1 |
| D2 `range` 完整签名（`range(n)`，1 参） | §6 `range` | §8.1 |
| D3 循环体 `let` 每轮新绑定 | §4-23 / §3 / L5 | §8.1 |
| D4 退出码语境（`run`→2 / `test`→1） | §0 / §5 / §8 | §8.1 |
| D5 对齐 `<`/`>`/`^`+fill；无动态宽度 | §3 / §4.5 | §8.1 |
| D6 `len(string)` 合法 + O(n) 提醒 | §6 `len` / §4.6 | §8.1 |
| D7 隐性语法正面示例（链式下标赋值 / `else if` / 多 `${}` / 零参 `print()` / 短路） | §3 / §4 / §7 L5 | §8.1 |
| 新增：O(n) 字符串 / 数组构建惯用法 | §4.6 / §4-24 / §8 | §8.2 |

---

## 2. 实测程序（代码 + 真实输出 + 退出码）

> 源文件：`docs/guide/ai/examples/{01_hello,02_basics,03_students,04_wordcount}.lfz`。

### 2.1 `01_hello.lfz`

```lfz
#42
print("Hello, LFZ!")
```

命令与输出：
```
> cargo run --quiet -- run docs\guide\ai\examples\01_hello.lfz
Hello, LFZ!
exit=0
```

### 2.2 `02_basics.lfz`（类型 / 算术陷阱 / if 表达式 / 插值 / `;;`）

```lfz
#42
let name = "LFZ"
var count = 3
count = count + 1

print("hello, ${name}")
print("7 / 2 = ${7 / 2}")
print("div(7, 2) = ${div(7, 2)}")
print("-7 % 3 = ${-7 % 3}")
print("round(2.5) = ${round(2.5)}")
print("round(3.5) = ${round(3.5)}")

let grade = if count > 3 { "high" } else { "low" }
print("grade = ${grade}")

;;
```

命令与输出：
```
> cargo run --quiet -- run docs\guide\ai\examples\02_basics.lfz
hello, LFZ
7 / 2 = 3.5
div(7, 2) = 3
-7 % 3 = 2
round(2.5) = 2
round(3.5) = 4
grade = high
name ： LFZ
count ： 4
grade ： high
exit=0
```

### 2.3 `03_students.lfz`（**管道 + 插值 + 结构体**，稍复杂综合）

```lfz
#42
struct Student {
    name: "",
    score: 0,
    fn isTop() => self.score >= 90,
    fn line() => "${self.name}: ${self.score:>3}",
}

fn average(xs) {
    var total = 0.0
    for s in xs { total += float(s.score) }
    total / float(len(xs))
}

let roster = [
    Student { name: "Alice", score: 93 },
    Student { name: "Bob", score: 67 },
    Student { name: "Cara", score: 88 },
]

let ranked = roster |> sortBy((s) => -s.score)
print("ranking (desc):")
for s in ranked { print("  " + s.line()) }

let names = roster |> map((s) => s.name) |> join(", ")
print("members: ${names}")
print("average: ${average(roster):.2f}")
let top = roster |> maxBy((s) => s.score)
print("top: ${top.name} (${top.score})")
```

命令与输出：
```
> cargo run --quiet -- run docs\guide\ai\examples\03_students.lfz
ranking (desc):
  Alice:  93
  Cara:  88
  Bob:  67
members: Alice, Bob, Cara
average: 82.67
top: Alice (93)
exit=0
```

### 2.4 最小任务基线（§9 参考解）

```lfz
#42
fn sum_even(xs) {
    xs |> filter((x) => x % 2 == 0) |> reduce((acc, x) => acc + x, 0)
}
print(sum_even([1, 2, 3, 4, 5]))
```

`run --json` → `{"ok":true}`，exit `0`；非 JSON 输出 `6`。

---

## 3. 错误类实测（类名 + 中文消息 + 退出码；`run --json`）

| 触发程序 | 类名 | 中文消息 | 退出码 |
|---|---|---|---|
| `.lfz` 缺 `#42` | `CosmosAnswerError` | `你忘记了宇宙的答案` | 2 |
| `#42 `（尾随空格）/ 仅 3 字节 `#42` / 空 `.lfz` | `CosmosAnswerError` | `你忘记了宇宙的答案` | 2 |
| 单 `;` | `SyntaxError` | `单独的 ';' 非法；打印变量请用 ';;'` | 2 |
| `let a = 1` 后 `a = 2` | `TypeError` | `不能重新赋值 let 变量 'a'；let 只锁重绑定，不锁内容` | 2 |
| `1 + "a"` | `TypeError` | `运算符 '+' 不支持 int 与 string` | 2 |
| `if 1 { ... }` | `TypeError` | `条件必须是 bool，得到 int` | 2 |
| `1 / 0` | `ZeroDivisionError` | `除以零` | 2 |
| `[1,2][5]` | `IndexError` | `下标 5 越界（长度 2）` | 2 |
| `{ k: 1 }.missing` | `FieldError` | `结构体没有字段 'missing'` | 2 |
| `9223372036854775807 + 1` | `OverflowError` | `整数溢出：结果超出 i64 范围` | 2 |
| `int("abc")` | `ValueError` | `无法把 string 转换为 int（'abc'）` | 2 |
| `int(float("nan"))` | `ValueError` | `无法把 float 转换为 int（'nan'）` | 2 |
| `int(float("inf"))` | `OverflowError` | `整数溢出：结果超出 i64 范围` | 2 |
| `min([])` | `ValueError` | `空数组没有极值（min）` | 2 |
| `print(undefined_thing)` | `NameError` | `未定义的名字 'undefined_thing'` | 2 |
| `assert(1 == 2, "bad")` | `AssertionError` | `断言失败：bad` | 2 |
| `fn f(n) { f(n+1) }; f(0)` | `RecursionError` | `递归深度超限（超过 10000 层）` | 2 |
| `check(1 == 2, "soft")`（非致命） | — | stderr：`check 失败：soft`（stdout 继续输出） | **0** |
| 非 `.lfz` 文件（`.txt`）无 `#42` | — | 正常运行（扩展名豁免） | 0 |

**头部规则边界实测**：`Foo.LFZ`（大写扩展名）带头 → 正常运行；`NoHdr.LFZ` 不带头 → `CosmosAnswerError`；`<BOM>#42` → 正常运行；程序中间 `let x = 1 # 2` → `SyntaxError: '#' 只能出现在文件首行的前导位；(字符串 / 注释 / 格式说明符内的 '#' 除外)`。

**其他实测事实**：`-9223372036854775808` 合法（i64::MIN）；`/` `div` `%` `round` 行为如上；管道 `xs |> map(f) |> join(sep)` 与 `10 |> push(_, [1,2])` 均通过；`push` 返回新值（原 `xs` 不变）；`keys({b:1,a:2})` → `["a", "b"]`；`sort([3,1,2])` 不改原数组且返回 `[1,2,3]`；自引用 struct 显示 `{me: <cycle>}`。

---

## 4. 实测驱动的修正清单（体现「用结果说话」）

下列每一条都是**我按初稿写程序时真的踩到 / 验证出**的问题，已回填 SKILL：

| # | 现象（实测） | 初稿问题 | 修正（已落地 SKILL） |
|---|---|---|---|
| 1 | `print("x = ${\"a\" + \"b\"}")` → `SyntaxError: 非法字符 '\'` | 初稿未提醒 `${}` 内是 CODE 模式 | 新增陷阱 §4-8：`${}` 内写普通 `"`，不要写 `\"` |
| 2 | `10 \|> push([1, 2])` → `TypeError`（`10` 被追加为**末参** `push([1,2],10)`） | 初稿易误以为 `10` 自动进首位 | 陷阱 §4-9 + §3 示例明确 `10 \|> push(_, [1, 2])` |
| 3 | `let a = 1; a = 2` → `SyntaxError`（先撞 `;`），**不是** `TypeError` | 错误类样例表里误写为 `TypeError` | 改为「对 let 变量再次赋值 → TypeError」，避免 `;` 干扰 |
| 4 | `run -e '...'` → 「只接受一个 `<file>` 参数」；`run -`（stdin）→ `IOError: 无法读取：-` | 初稿照 spec 提 `-e`/stdin 豁免，易误导 AI 去用 | §0 写死：**CLI v1 只有 `run <file>` / `test`**，无 `-e`/stdin/REPL；程序一律落成 `.lfz` 文件 |
| 5 | `run --json` 时程序 `print` 输出走 **stderr**，stdout 只有一行 JSON | 初稿未点明，AI 可能误判输出位置 | §0 注明 `--json` 的流行为 |
| 6 | `min([])`、`int(float("nan"))`、`int(float("inf"))` 三类易混边界 | 初稿只在表里泛写 | §4-13 / §4-18 + §2 实测样本逐条钉死 |

> 结论：**经上述修正后，本 SKILL 中的每个 LFZ 示例与每条消息样本均来自真实运行**；最小任务基线一次跑通（exit 0）。

---

## 5. 内置函数 54 个数目核对

按 `interface-contract.md` §10.7 分组计数并逐名列出（SKILL §6）：核心/数组 21 + struct 5 + 字符串 8 + 数值/转换/IO/断言 20 = **54**。数目与 spec 一致。

---

## 6. 复现方式

```powershell
$env:Path += ";$env:USERPROFILE\.cargo\bin"
cd "D:\XUE\2026fall\Program Design\lfz-programing language design"
cargo run --quiet -- run docs\guide\ai\examples\01_hello.lfz
cargo run --quiet -- run docs\guide\ai\examples\02_basics.lfz
cargo run --quiet -- run docs\guide\ai\examples\03_students.lfz
cargo run --quiet -- run docs\guide\ai\examples\04_wordcount.lfz
echo $LASTEXITCODE    # → 0
```

---

## 7. 第 4 次实测（P7c 增量）—「词频统计」：struct + 方法（带参）+ `while` + 管道 + 插值

> 目的：验证**只凭本 skill 的工作流**，能否写出一个**此前没写过**、且把 §3/§6 多个特性组合起来的小工具。程序见 `docs/guide/ai/examples/04_wordcount.lfz`（已同步进 SKILL §7 的 L4）。

### 7.1 用到的 skill 条目（自查）

| skill 条目 | 用途 |
|---|---|
| §0.5 第 1 步 | 首行写 `#42` |
| §0.5 第 2 步 | 到 §6 核对 `has` / `len` / `entries` / `sortBy` / `take` / `split` 的**签名与 data-last 顺序** |
| §3「结构体（模板+方法）」 | `struct Counter { ... fn bump(w) {...} }`；`Counter {}` 用默认字段 |
| §3「while / for」 | `while i < len(ws)` 驱动计数；`for e in c.top(3)` 遍历 |
| §3「管道 data-last」 | `self.counts \|> entries() \|> sortBy((e) => -e[1]) \|> take(n)` |
| §3「字符串插值 / 格式说明符」 | `"total=${self.total}"`、`"${e[0]}: ${e[1]:>2}"` |
| §0.5 第 4 步 | 写完**立刻** `cargo run --quiet -- run`，看到 exit `0` |
| §0.5 第 5 步 / §8 | 交付前过自检清单 |

### 7.2 踩的坑与解决（实测暴露）

| # | 现象 | 根因（对应 §4/§4.5） | 解决 |
|---|---|---|---|
| 1 | 首版把格式说明符写在占位符**外面**（`"${e[0]}:>${e[1]}"`），输出里出现字面量 `:>` | §3 明确格式说明符必须写在 `{}` 内、紧跟 `:` 之后；这是**从别语言带过来的直觉** | 改成 `${e[1]:>2}`（宽度/对齐都放进占位符内） |
| 2 | 计数时担心"读一个还不存在的键"会报错 | §4.5：读缺失键 → `FieldError`（§4-21） | 先 `has(w, self.counts)` 判断分支；缺失时写入 `1` |
| 3 | 不确定空 struct 字面量与"动态加字段"是否合法 | §3 `{}` 为 struct 字面量；`semantics.md` 明确实例可动态加字段 | 用 `Counter {}` + `self.counts[w] = ...` 动态加键，实测通过 |

> 说明：本次先在**临时目录**用两个探测程序（`probe1.lfz` / `probe2.lfz`）确认 `{}` / 动态键 / 方法改 `self` / `while` / 管道后，才写正式程序——这正是 §0.5「先查 §6、立刻跑」的流程体现。

### 7.3 程序（`docs/guide/ai/examples/04_wordcount.lfz`）

```lfz
#42
struct Counter {
    counts: {},
    total: 0,
    fn bump(w) {
        if has(w, self.counts) {
            self.counts[w] = self.counts[w] + 1
        } else {
            self.counts[w] = 1
        }
        self.total += 1
    },
    fn top(n) => self.counts |> entries() |> sortBy((e) => -e[1]) |> take(n),
    fn report() => "total=${self.total}, distinct=${len(self.counts)}",
}

fn count_all(ws) {
    let c = Counter {}
    var i = 0
    while i < len(ws) {
        c.bump(ws[i])
        i += 1
    }
    c
}

let text = "the quick brown fox jumps over the lazy dog the fox jumps the"
let words = text |> split(" ")

let c = count_all(words)
print(c.report())
print("top 3 words (desc):")
for e in c.top(3) {
    print("  ${e[0]}: ${e[1]:>2}")
}
```

### 7.4 实跑证据

```
> cargo run --quiet -- run docs\guide\ai\examples\04_wordcount.lfz
total=13, distinct=8
top 3 words (desc):
  the:  4
  fox:  2
  jumps:  2
exit=0

> cargo run --quiet -- run --json docs\guide\ai\examples\04_wordcount.lfz
（stdout）{"ok":true}
（stderr）total=13, distinct=8 / top 3 words (desc): / the: 4 / fox: 2 / jumps: 2
json-exit=0
```

> 校验：文本 13 个词、8 个不同词；`the`×4、`fox`×2、`jumps`×2。`entries` 按键升序 + `sortBy` 稳定 → 并列按键序。**结论：一次跑通，退出码 `0`。**

### 7.5 结论

新增程序与 `§2` 的 3 个示例**不重复**（首次覆盖"struct 带参方法 + `while` 驱动 + 动态键 struct 计数"的组合），**一次跑通、exit 0**；暴露的 1 个真实坑（格式说明符位置）已回填到本节，并印证 §0.5 工作流有效。

---

## 8. T11-③ 修订实测（D1–D7 + O(n) 构建，2026-09-27）

> 目的：把 skill 盲测 **2/8** 暴露的 7 条缺口（`FEATURE-AUDIT.md` §6.2 D1–D7）逐条落地——**每条先用 `lfz` 实跑验证，再写入 `SKILL.md`**。
> 环境：`dist\lfz.exe`（v1.0.1，由当前源码重建）；控制台 `chcp 65001` 以正确显示中文；探针在 `Temp\lfz-dx\`（**临时件，收工已清**，源码见 §8.4）。

### 8.1 逐条实测证据

| # | 修订 | 探针命令（`dist\lfz.exe run …`） | 实测输出 | 退出码 | 写入 SKILL |
|---|---|---|---|---|---|
| **D1** | 字符串不可下标 | `d1_str_index.lfz`（`print("abc"[0])`） | `TypeError: 运算符 '[]' 不支持 string 与 array / struct` | **2** | §4-22 / §4.5 / §6 `split` |
| D1 | `split("", s)` 惯用法 | `d1_split_idiom.lfz` | `["a", "b", "c"]` / `a` / `c` / `日` `本` `語` | **0** | §4-22 / §6 / L5 |
| **D2** | `range(n)` 签名 | `d2_range.lfz`（`range(3)`/`range(0)`/`range(-1)`） | `[0, 1, 2]` / `[]` / `[]` | **0** | §6 `range` |
| D2 | 只接受 1 参 | `d2_range2.lfz`（`range(1, 4)`） | `TypeError: 函数 range 期待 1 个参数，得到 2` | **2** | §6 `range`（**不写** `range(lo,hi)`） |
| **D3** | 循环体 `let` 每轮新绑定 | `d3_letloop.lfz`（循环内 `let x` + 闭包） | `0` / `10` / `20`（共用 cell 会是 20/20/20） | **0** | §4-23 / §3 / L5 |
| **D4** | `run` 下 assert 失败 → 2 | `d4_assert_run.lfz` | `AssertionError: 断言失败：boom` | **2** | §0 / §5 / §8 |
| D4 | `test` 下用例失败 → 1 | `lfz test <含失败用例的目录>` | `汇总：共 1 个用例，通过 0，失败 1，错误 0` | **1** | §0 / §5 / §8 |
| **D5** | 对齐 `<`/`>`/`^` + fill | `d5_align.lfz` | `[ab   ]` `[   ab]` `[ ab  ]` `[00042]` `[**ab***]` `[00005]` `[3.14]` `[ff]` | **0** | §3 / §4.5 |
| D5 | 动态宽度不支持 | `d5_width_dynamic.lfz`（`${"ab":>w}`） | `ValueError: 格式说明符非法：'>w'` | **2** | §3 / §4.5 |
| **D6** | `len(string)` 合法、按标量 | `d6_len_string.lfz` | `3` `0` `3` `1` `3` `5`（`"日本語"`→3、`"😀"`→1） | **0** | §6 `len` / §4.6 |
| **D7** | 隐性语法合集 | `d7_misc.lfz` | `[[0, 0], [7, 0]]` / `B` / `A` / `x-y-xy` /（空行）/ `after-blank` / `false` / `true` | **0** | §3 / §4 / L5 |
| 示例 | L5（内联示例） | `ex_l5.lfz` | 见 `SKILL.md` §7 L5（输出逐行一致） | **0** | §7 L5 |

**D7 覆盖的 5 个构造**（均**一次跑通**）：链式下标赋值 `t[1][0] = 7`、`else if` 链、一条串多 `${}`、零参 `print()`（空行）、`&&`/`||` 短路（`xs == []` 时 `len(xs) > 0 && xs[0] == 1` 不越界）。

### 8.2 新增：O(n) 字符串 / 数组构建（性能红线，**含一处与审计建议相反的实测发现**）

`FEATURE-AUDIT.md` §7.1 指出「循环内 `s = s + c` 拼接是 O(n²)」，并**建议**改为「先 `push` 到数组再 `join`（O(n)）」。逐条实跑复核如下（循环 N 次，毫秒，≈值）：

| 写法 | N=40000 | N=80000 | N=160000 | N=320000 | 实测复杂度 |
|---|---|---|---|---|---|
| `var s=""` + `s = s + "x"` | 61 | 139 | 425 | 1582 | **O(n²)**（审计说法**成立**） |
| `var a=[]` + `a = push("x", a)` + `join` | 251 | 674 | 4076 | 超时 | **O(n²)**（审计**建议不成立**） |
| `range(n) \|> map((i)=>"x") \|> join("")` | 25 | 35 | 56 | 104 | **O(n)** ✅ |
| 预分配 `arr[i]="x"` + `join` | 33 | — | 85 | 151 | **O(n)** ✅ |
| `split("", s)` + 逐字符遍历 | 30 | — | 71 | 127 | **O(n)** ✅ |

**结论（与 `FEATURE-AUDIT.md` §7.1 的偏差，已报 team-lead）**：
1. `s = s + c` 确为 O(n²) —— 源码：`Str + Str` 实现为 `format!("{a}{b}")`（复制整个前缀），`src/evaluator.rs:1496`。
2. **但「`push` 到数组再 `join`」同样是 O(n²)**：`push` 遵守 A1 返回**新数组**，内部 `xs.to_vec()` 整体克隆（`src/builtins.rs:385`），循环累积常数更大、**更慢**。→ **不能照搬为 O(n) 建议**。
3. **真正 O(n) 的惯用法**：①`range(n) |> map(f) |> join("")`（下标驱动，单次分配）；②**预分配 + 下标写** `arr[i] = v`（唯一原地写语法，条件累积）。已写入 `SKILL.md §4.6`。

### 8.3 复现命令

```powershell
chcp 65001
$env:Path += ";$env:USERPROFILE\.cargo\bin"
$exe = "dist\lfz.exe"
& $exe run Temp\lfz-dx\probes\d1_str_index.lfz      # exit 2 (TypeError)
& $exe run Temp\lfz-dx\probes\d1_split_idiom.lfz    # exit 0
& $exe run Temp\lfz-dx\probes\d2_range.lfz          # exit 0
& $exe run Temp\lfz-dx\probes\d2_range2.lfz         # exit 2
& $exe run Temp\lfz-dx\probes\d3_letloop.lfz        # exit 0
& $exe run Temp\lfz-dx\probes\d4_assert_run.lfz     # exit 2
& $exe test Temp\lfz-dx\testcases                   # exit 1
& $exe run Temp\lfz-dx\probes\d5_align.lfz          # exit 0
& $exe run Temp\lfz-dx\probes\d5_width_dynamic.lfz  # exit 2
& $exe run Temp\lfz-dx\probes\d6_len_string.lfz     # exit 0
& $exe run Temp\lfz-dx\probes\d7_misc.lfz           # exit 0
& $exe run Temp\lfz-dx\probes\ex_l5.lfz             # exit 0 (L5)
& $exe run Temp\lfz-dx\probes\snip1.lfz             # exit 0 -> 0,1,4,9,16
& $exe run Temp\lfz-dx\probes\snip2.lfz             # exit 0 -> 0-10-20
```

### 8.4 探针源码（临时件已清，此处保留以便复现）

```lfz
// d1_str_index.lfz（expect exit 2）        // d1_split_idiom.lfz（expect exit 0）
#42                                         #42
let s = "abc"                               let s = "abc"
print(s[0])                                 let cs = split("", s)
                                            print(cs)
                                            print(cs[0])            // a
                                            print(cs[2])            // c
                                            for c in split("", "日本語") { print(c) }
```

```lfz
// d2_range.lfz（exit 0）                    // d2_range2.lfz（exit 2）
#42                                         #42
print(range(3))                             print(range(1, 4))
print(range(0))
print(range(-1))
```

```lfz
// d3_letloop.lfz（exit 0 -> 0/10/20）        // d4_assert_run.lfz（exit 2）
#42                                         #42
var fns = []                                print("before")
var i = 0                                   assert(1 == 2, "boom")
while i < 3 {                               print("after")
    let x = i * 10
    let f = fn() => x
    fns = push(f, fns)
    i += 1
}
for f in fns { print(f()) }
```

```lfz
// d5_align.lfz（exit 0）                    // d5_width_dynamic.lfz（exit 2）
#42                                         #42
let n = 5                                   let w = 5
print("[${"ab":<5}]")    // [ab   ]         print("[${"ab":>w}]")
print("[${"ab":>5}]")    // [   ab]
print("[${"ab":^5}]")    // [ ab  ]
print("[${42:0>5}]")     // [00042]
print("[${"ab":*^7}]")   // [**ab***]
print("[${n:05d}]")      // [00005]
print("[${3.14159:.2f}]") // [3.14]
print("[${255:x}]")      // [ff]
```

```lfz
// d6_len_string.lfz（exit 0）               // d7_misc.lfz（exit 0）
#42                                         #42
print(len("abc"))        // 3               let t = [[0, 0], [0, 0]]
print(len(""))           // 0               t[1][0] = 7
print(len("日本語"))      // 3               print(t)                 // [[0, 0], [7, 0]]
print(len("😀"))          // 1               fn grade(n) {
print(len("a😀b"))        // 3                   if n >= 90 { "A" }
print(len("hello"))      // 5                   else if n >= 80 { "B" }
                                                else if n >= 70 { "C" }
                                                else { "F" }
                                            }
                                            print(grade(85))         // B
                                            print(grade(95))         // A
                                            let a = "x"
                                            let b = "y"
                                            print("${a}-${b}-${a}${b}")  // x-y-xy
                                            print()                  // 空行
                                            print("after-blank")
                                            let xs = []
                                            print(len(xs) > 0 && xs[0] == 1)     // false
                                            print(len(xs) == 0 || xs[0] == 1)    // true
```

> `d4` 的 `test` 侧探针：`Temp\lfz-dx\testcases\fail_case.lfz` = `#42` + `let x = 1` + `assert(x == 2, "…")`；`lfz test` 该目录 → `失败 1`、exit `1`。
> 说明：上述探针均为**临时件**，已按团队约定从 `Temp/` 清理；本文保留命令与输出作为可复现证据。

