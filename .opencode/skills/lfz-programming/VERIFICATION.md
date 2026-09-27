# `lfz-programming` SKILL — 实测验证记录（ai-dx-engineer）

> 证据文化：无证据不算完成。本文件记录「只凭 `SKILL.md` 能否写出可运行 LFZ 程序」的**实测**闭环。
> 基线：`docs/spec/` 冻结 v1、解释器 `cargo run` 可用（P3/P4 已完成）。命令均在**仓库根目录**、PowerShell 下执行（先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`）。
> 日期：2026-09-27。

---

## 1. `interface-contract.md` §11.1 五条硬性要求 → SKILL 落点对照

| §11.1 要求 | SKILL 中的位置 | 落实方式 |
|---|---|---|
| ① `#42` 首行规则（置顶） | **§1「铁律」**（正文第一节，仅排在触发说明之后） | 恰 3 字符 `#42` + 行终止符；违反 → `CosmosAnswerError: 你忘记了宇宙的答案`；**仅按扩展名 `.lfz`（大小写不敏感）**触发；非 `.lfz`/REPL/stdin/`-e` 豁免；列出 `# 42`/`#42 `/`#43`/`##42`/前导空白/仅 3 字节/空文件等全部变体反例；BOM 豁免 |
| ② 错误类清单（逐条：触发条件 + 中文消息示例） | **§2「错误类清单」** | 12 个具体类 + 基类 `LfzError`，逐行给出「触发条件 / 中文消息示例 / 阶段」，并附 10 条实测消息样本 |
| ③ 所有代码示例必须含 `#42` 首行（片段须注明） | **§3 / §4 / §7** | 所有完整程序示例首行均为 `#42`；§3 骨架片段在块前统一注明「若作为 `.lfz` 文件，首行须为 `#42`」；§9 提示模板要求输出含 `#42` |
| ④ 禁止出现旧的编号错误码（形如「前缀码-编号」） | **§2 表头声明 + §8 自检清单 + 全文** | 明确「只有类名 + 中文消息，不存在任何编号错误码」；SKILL 与 README 全文均不含任何编号错误码 |
| ⑤ `check` 非致命 / `assert` 致命（A4） | **§5「check 非致命 / assert 致命」** | 表格写死三者行为 + 实测 `check(1==2)` → stdout `continue` / stderr `check 失败：soft` / 退出码 `0` |

---

## 2. 实测程序（代码 + 真实输出 + 退出码）

> 源文件：`docs/guide/ai/examples/{01_hello,02_basics,03_students}.lfz`。

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
echo $LASTEXITCODE    # → 0
```
