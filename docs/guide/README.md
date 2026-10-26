# LFZ 开发者指南

> 面向**人类开发者**的 LFZ 手册。目标：让你在 **5 分钟**内装好 LFZ、跑通第一个程序、看懂常见报错。
> 权威语法/语义以 `docs/spec/` 为准；本手册只做讲解与示例，不定义新语法。
> 配套：教程 [`tutorial.md`](tutorial.md) ｜ 参考 [`reference.md`](reference.md) ｜ 错误模型 [`errors.md`](errors.md) ｜ 测试 [`testing.md`](testing.md)
> 最后更新: 2026-10-25（T12：1.4.0 参数、代码入口与 JSON） by docs-writer

---

## 1. LFZ 是什么

LFZ（读作 "elf-zee"）是一门**动态类型、表达式导向**的通用脚本语言。一句话记住它：

> **万物皆值，值皆可流；一种结构，两种面孔。**

- **万物皆值**：`if`、块、函数、数组、结构体都是值；块的值 = 最后一条表达式的值；函数体末尾表达式的值就是返回值。
- **值皆可流**：任何值都能顺 `|>` 管道流向下一个函数（见教程 Step 7）。
- **一种结构，两种面孔**：复合类型只有 `array`（有序）与 `struct`（键值）；`struct` 既能当对象用（`.字段` / 方法），也能当字典用（`["键"]`）。
- **无魔法**：条件必须是 `bool`；除了 `int → float` 加宽外无隐式转换；越界 / 缺字段 / 溢出 / 类型不符一律结构化报错。
- **仪式感**：每个 `.lfz` 文件以第一行 `#42` 起手——"宇宙的答案"，缺失即报错。

当前已实现 **72 个内置**，包括双参 range、字符串查找/补齐/子串、ord/chr、sin/cos/log/exp、contains、UTF-8 文件 IO，以及 `argv`、`jsonParse`、`jsonStringify`。签名和边界见参考 §5；相对文件路径基于进程当前工作目录。

完整形式定义见 [`docs/spec/syntax.md`](../spec/syntax.md)；语义见 [`docs/spec/semantics.md`](../spec/semantics.md)。

---

## 2. 准备环境（约 2 分钟）

当前版本为 **1.4.0**。仓库内已有 `dist/lfz.exe` 时，可直接使用它，不必先构建；不要误用尚未更新的全局安装。自行构建时，LFZ 解释器用 **Rust** 实现，**无第三方依赖**，需要 Rust 工具链。

- 安装 Rust：见 <https://www.rust-lang.org/tools/install>（安装后重启终端，确保 `cargo` 在 PATH）。
- 验证：`cargo --version` 能输出版本号即可。

在仓库根目录构建：

```console
$ cargo build
```

成功即得到可运行的 `lfz`（`cargo build` 输出 `Finished` 且无 error）。

> Windows 提示：若 `cargo` 找不到，把 `%USERPROFILE%\.cargo\bin` 加进 `PATH`。

---

## 3. 第一个程序（约 1 分钟）

**Step 1**：新建一个文件 `hello.lfz`，内容**恰好**两行：

```lfz
#42
print("Hello, LFZ!")
```

**Step 2**：运行它。

```console
$ cargo run --quiet -- run hello.lfz
Hello, LFZ!
```

第一行 `#42` 是文件前导，**必须**是文件第 1 行的三个字符，其后紧跟换行（详见 [`reference.md` §1](reference.md)）。
`print` 把参数按显示形式拼接后写入 stdout（`docs/spec/interface-contract.md` §10.7）。

> 仓库里已经有一个同样的样例，可以直接运行：
> ```console
> $ cargo run --quiet -- run examples/hello.lfz
> Hello, LFZ!
> ```

---

## 4. 再走一步：一个稍大的程序

把下面内容写进 `greeting.lfz`：

```lfz
#42
let name = "LFZ"
var visits = 0
visits += 1
print("你好，${name}！")
print("这是第 ${visits} 次问候。")
```

运行：

```console
$ cargo run --quiet -- run greeting.lfz
你好，LFZ！
这是第 1 次问候。
```

- `let` 声明**不可重绑定**的变量；`var` 声明**可重绑定**的变量（`docs/spec/syntax.md` §7；`docs/spec/semantics.md` §4.5.2）。
- `+=` 是复合赋值（`docs/spec/syntax.md` §7 `assign_op`）。
- `"${...}"` 是**富字符串插值**（`docs/spec/syntax.md` §2.8）。

---

## 5. 常用命令一览

| 命令 | 作用 |
|---|---|
| `cargo build` | 构建解释器 |
| `cargo run --quiet -- run <file>` | 运行一个 LFZ 脚本（`.lfz` 文件首行须 `#42`） |
| `cargo run --quiet -- run --json <file>` | 运行并以**一行 JSON** 输出执行状态/错误（机器可读） |
| `cargo run --quiet -- test` | 运行测试集（默认发现 `tests/**/*.lfz`，排除 `tests/fixtures`） |
| `cargo run --quiet -- test --json` | 测试的机器可读输出 |
| `cargo run --quiet -- --help` | 查看 CLI 帮助 |

> `--json` 是执行选项，可放在源码参数和脚本参数区域以外的选项位置；开启后 **stdout 只写 JSON**，程序输出改到 stderr（`docs/tooling/runner-contract.md` §9.3）。详见 [`errors.md` §6](errors.md)。

---

## 6. 常见错误一眼看懂

出错不会静默：LFZ 会给**类名 + 中文消息 + 位置**。下表是最容易踩的坑（完整清单见 [`errors.md`](errors.md)）。

| 现象 | 你会看到 | 正确写法 |
|---|---|---|
| 文件忘了 `#42` | `CosmosAnswerError: 你忘记了宇宙的答案` | 第 1 行写 `#42` |
| 写了单个 `;` | `SyntaxError: 单独的 ';' 非法；打印变量请用 ';;'` | 用换行分隔语句；打印变量用 `;;` |
| 一行写两条语句 | `SyntaxError: 语句之间必须有换行` | 每条语句独占一行 |
| 条件不是 `bool` | `TypeError: 条件必须是 bool，得到 int` | 比较后得到 `bool`，如 `if n > 0 { ... }` |
| `let` 变量被重新赋值 | `TypeError: 不能重新赋值 let 变量 'x'；let 只锁重绑定，不锁内容` | 该变量声明处改用 `var` |
| 除以零 | `ZeroDivisionError: 除以零` | 先判除数；整数向下取整用 `div(a, b)` |
| 下标越界 | `IndexError: 下标 10 越界（长度 3）` | 检查下标范围 `[0, len-1]` |
| 函数外写 `return` | `SyntaxError: 'return' 只能出现在函数体内` | 把 `return` 放进 `fn` 体 |
| 整数溢出 | `OverflowError: 整数溢出：结果超出 i64 范围` | 用 `float` 或控制数值范围 |
| **字符串取下标** | `TypeError: 运算符 '[]' 不支持 string 与 array / struct` | 字符串**不能** `s[0]`；用 `split("", s)` 拆成字符数组 |
| 容器构造过大 | `OverflowError: 容量溢出：所需容量超出可分配上限` | 别构造超大数组/字符串；超大循环改用 `while` |
| 嵌套太深 | `SyntaxError: 嵌套深度超限（超过 1000 层）` | 拆小表达式、减少嵌套 |
| 格式宽度写了变量 | `ValueError: 格式说明符非法：'>w'` | 宽度只支持字面数字；按变量宽度用 `padEnd` / `padStart` |

关于除法：`/` **永远**返回 `float`（`7 / 2 == 3.5`）；需要整数向下取整用内置 `div(a, b)`（`div(7, 2) == 3`）；`%` 是 Python 式取模（`-7 % 3 == 2`）（`docs/spec/semantics.md` §4.2）。

> **两个最常问的点**：
> 1. **字符串不能下标**：`s[0]` 非法。按字符处理先 `split("", s)`（一次 O(n)），之后用数组下标（O(1)）。逐字符**别**用 `s = s + c`（循环里是 O(n²)），改用 map 一次构造数组或预分配后下标写入，再 join；循环 push 也是 O(n²)。详见 [`reference.md` §3.5](reference.md)、教程 Step 13。
> 2. **`assert` 的退出码看语境**：`lfz run` 下 `assert` 失败 → 错误类 → 退出 **2**；`lfz test` 下用例失败 → 退出 **1**。详见 [`errors.md` §3.4](errors.md)。

---

## 7. 接下来读什么

| 想做的事 | 读 |
|---|---|
| 跟着例子从零写一个完整小程序 | [`tutorial.md`](tutorial.md) |
| 查语法、运算符优先级、72 个内置函数 | [`reference.md`](reference.md) |
| 搞懂 12 类错误、`Traceback`、退出码、`--json` | [`errors.md`](errors.md) |
| 写测试、跑测试、覆盖矩阵 | [`testing.md`](testing.md) |
| 权威语法 / 语义 / 接口契约 | [`../spec/`](../spec/) |

## 8. 用参数和 JSON 写可复用脚本（1.4.0）

从仓库根目录执行。`-e` 直接运行一项源码，不需要 `#42`，也不会自动打印最后表达式；要显示结果就调用 `print`：

```powershell
./dist/lfz.exe -e 'print(jsonStringify(argv()))' -- '中文' '--json' ''
```

stdout：`["中文","--json",""]`。`--` 之后每项原样交给脚本，数字和 JSON 也先是字符串；由脚本按需调用 `int` 或 `jsonParse`。文件入口同样支持 `lfz run file.lfz -- ARG...`。没有 `--` 的额外参数仍报错。

标准输入也能提供整段源码：

```powershell
'print(1 + 2)' | ./dist/lfz.exe -
```

stdout 为 `3`。stdin 源码先读到 EOF，再执行；此模式不能同时从同一 stdin 向 `input()` 提供数据。文件/`-e` 模式仍可使用 `input()`。REPL 尚未实现；无参数运行不会等待输入。

普通模式的 `print(jsonStringify(value))` 用于输出业务 JSON；CLI 的 `--json` 输出执行状态，成功时 stdout 为 `{"ok":true}`，程序的 `print` 转到 stderr。把 `--json` 放在脚本参数 `--` 之后，只会传给 `argv()`。

真实应用见 [jsonflow](../../app/README.md)：

```powershell
New-Item -ItemType Directory -Force Temp | Out-Null
./dist/lfz.exe run app/jsonflow.lfz -- app/fixtures/t12/config.json app/fixtures/t12/input.json Temp/guide-result.json
```

它读取配置和记录数组，筛选并投影字段，将结果写入指定文件。换配置和路径即可复用。另一个应用 [sortviz](../../app/README.md) 支持 `--small`、`--big`、`--seed` 和 `--no-frames`，不再需要改源码常量。
