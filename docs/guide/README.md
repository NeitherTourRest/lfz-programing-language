# LFZ 开发者指南

> 面向**人类开发者**的 LFZ 手册。目标：让你在 **5 分钟**内装好 LFZ、跑通第一个程序、看懂常见报错。
> 权威语法/语义以 `docs/spec/` 为准；本手册只做讲解与示例，不定义新语法。
> 配套：教程 [`tutorial.md`](tutorial.md) ｜ 参考 [`reference.md`](reference.md) ｜ 错误模型 [`errors.md`](errors.md) ｜ 测试 [`testing.md`](testing.md)
> 最后更新: 2026-09-27 by docs-writer

---

## 1. LFZ 是什么

LFZ（读作 "elf-zee"）是一门**动态类型、表达式导向**的通用脚本语言。一句话记住它：

> **万物皆值，值皆可流；一种结构，两种面孔。**

- **万物皆值**：`if`、块、函数、数组、结构体都是值；块的值 = 最后一条表达式的值；函数体末尾表达式的值就是返回值。
- **值皆可流**：任何值都能顺 `|>` 管道流向下一个函数（见教程 Step 7）。
- **一种结构，两种面孔**：复合类型只有 `array`（有序）与 `struct`（键值）；`struct` 既能当对象用（`.字段` / 方法），也能当字典用（`["键"]`）。
- **无魔法**：条件必须是 `bool`；除了 `int → float` 加宽外无隐式转换；越界 / 缺字段 / 溢出 / 类型不符一律结构化报错。
- **仪式感**：每个 `.lfz` 文件以第一行 `#42` 起手——"宇宙的答案"，缺失即报错。

完整形式定义见 [`docs/spec/syntax.md`](../spec/syntax.md)；语义见 [`docs/spec/semantics.md`](../spec/semantics.md)。

---

## 2. 准备环境（约 2 分钟）

LFZ 解释器用 **Rust** 实现，**无第三方依赖**，只需要 Rust 工具链。

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
| `cargo run --quiet -- run --json <file>` | 运行并以**一行 JSON** 输出结果（机器可读） |
| `cargo run --quiet -- test` | 运行测试集（默认发现 `tests/**/*.lfz`，排除 `tests/fixtures`） |
| `cargo run --quiet -- test --json` | 测试的机器可读输出 |
| `cargo run --quiet -- --help` | 查看 CLI 帮助 |

> `--json` 是全局开关，可放在子命令任意位置；开启后 **stdout 只写 JSON**，程序输出改到 stderr（`docs/tooling/runner-contract.md` §9.3）。详见 [`errors.md` §6](errors.md)。

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

关于除法：`/` **永远**返回 `float`（`7 / 2 == 3.5`）；需要整数向下取整用内置 `div(a, b)`（`div(7, 2) == 3`）；`%` 是 Python 式取模（`-7 % 3 == 2`）（`docs/spec/semantics.md` §4.2）。

---

## 7. 接下来读什么

| 想做的事 | 读 |
|---|---|
| 跟着例子从零写一个完整小程序 | [`tutorial.md`](tutorial.md) |
| 查语法、运算符优先级、54 个内置函数 | [`reference.md`](reference.md) |
| 搞懂 12 类错误、`Traceback`、退出码、`--json` | [`errors.md`](errors.md) |
| 写测试、跑测试、覆盖矩阵 | [`testing.md`](testing.md) |
| 权威语法 / 语义 / 接口契约 | [`../spec/`](../spec/) |
