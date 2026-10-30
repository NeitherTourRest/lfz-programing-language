# LFZ 测试：如何写与跑

> LFZ 内建测试运行器（`lfz test`）。本页讲清：**怎么跑、怎么写、负例怎么放、结果怎么读**。
> 权威契约：[`docs/spec/interface-contract.md`](../spec/interface-contract.md) §11.2（**T-R1 … T-R4**）；实现侧派生：[`docs/tooling/runner-contract.md`](../tooling/runner-contract.md)。
> 配套：[`README.md`](README.md) ｜ [`tutorial.md`](tutorial.md) ｜ [`reference.md`](reference.md) ｜ [`errors.md`](errors.md)
> 最后更新: 2026-10-27（T14：非执行语法检查；1.4.0候选） by docs-writer

---

## 1. 一条命令跑测试

在仓库根目录：

```console
$ cargo run --quiet -- test
```

- 默认发现根 = 当前目录下的 `tests/`；递归收集 `tests/**/*.lfz`，**跳过 `tests/fixtures/`**，并读取 `tests/cases.json` 声明负例。
- 每个用例一行 `PASS/FAIL/ERROR <name>`；末尾汇总行逐字符稳定：

```text
汇总：共 166 个用例，通过 166，失败 0，错误 0
```

其它用法：

```console
$ cargo run --quiet -- test <目录或文件>     # 指定发现根 / 单个用例
$ cargo run --quiet -- test --json           # stdout 只写一行 JSON
```

> `--json test` 亦可。执行脚本时，`-e` 后的源码和 `--` 后的脚本参数不参与选项解析。

---

## 2. runner 契约（T-R1 … T-R4）

出处：`interface-contract.md` §11.2。工具链实现见 `runner-contract.md`。

| 条目 | 内容 |
|---|---|
| **T-R1** | 被 runner 作为 LFZ 程序加载的 **`.lfz` 测试文件**必须带 `#42` 首行；缺失 → 该用例记为 **error**（退出码 2）。**非 `.lfz` 夹具按扩展名豁免**（不校验前导）。 |
| **T-R2** | "缺前导 / 非法前导"等**负例**不得放入自动发现目录；须放**非自动发现**的夹具目录（约定 `tests/fixtures/`），在清单 `tests/cases.json` 用 `"expect": {"error": "<类名>"}` 声明。 |
| **T-R3** | 失败输出用 `semantics.md` §8.2 的位置信息；**仅 `assert`/`fail`** 抛 `AssertionError`，以此记 FAIL；`check` 失败**非致命**（stderr 警告 + 返回 `false`）。 |
| **T-R4** | 发现规则：默认根 `tests/`；递归 `*.lfz`（大小写不敏感）；**排除 `fixtures/`**；结果稳定排序。 |

---

## 3. 写一个正向用例（`.lfz`）

正向用例用 LFZ 语言写，断言用内建 `assert` / `check`；文件名建议 `test_<特性>.lfz`；**首行必须 `#42`**。

```lfz
#42
// tests/lfz/test_arithmetic.lfz（示意）
assert(1 + 1 == 2, "加法")
assert(7 / 2 == 3.5, "除法返回 float")
assert(div(7, 2) == 3, "div 向下取整")
assert(-7 % 3 == 2, "Python 取模")
check(1 == 2, "软检查不致命")          // 失败只写 stderr，用例仍 PASS
```

- **`assert(cond, msg?)`**：失败**致命**，用例判 `FAIL`（退出码 1）。
- **`check(cond, msg?)`**：失败**非致命**，写一行 stderr 警告并返回 `false`，用例仍 `PASS`（`SE §4.5.10`）。

> 断言消息建议写成 `<特性>_<场景>_<期望>`（如 `sortBy_is_stable`），失败时一眼能定位。

---

## 4. 写一个负例（期望报错）

**负例 = 期望程序以某个错误类终止**。它必须：

1. 放在**非自动发现**的目录，约定 `tests/fixtures/`（`fixtures` 目录会被发现器跳过，`runner-contract §2.3`）。
2. 在 `tests/cases.json` 里用 `expect.error` 声明期望的**错误类名**（必须是 [`errors.md` §1](errors.md) 的 12 个类名之一）。

清单 schema（出处 `runner-contract.md` §3）：

```json
{
  "cases": [
    { "path": "fixtures/no_header.lfz", "name": "缺 #42 前导（负例）", "expect": { "error": "CosmosAnswerError" } },
    { "path": "fixtures/div_zero.lfz",  "expect": { "error": "ZeroDivisionError" } }
  ]
}
```

- `path`：**相对清单所在目录**（即发现根）的路径，可指向自动发现不到的夹具。
- `expect.error`：期望的错误类名；一致 → `PASS`，否则 `FAIL`。
- `name`：报告里显示的名字（默认 = `path`）。

> **为什么负例不能直接放 `tests/lfz/`？** 自动发现的 `.lfz` 若报非断言类错误会被记为 `ERROR`（退出码 2），污染正常用例集。必须用 `expect` + 非自动发现夹具，才判为 `PASS`（T-R2）。

---

## 5. 判定与退出码

| 情形 | 判定 | 出处 |
|---|---|---|
| 全链路无任何 `LfzError` | `PASS` | T-R3 |
| `check` 返回 `false`（非致命） | `PASS` | A4 / T-R3 |
| 清单声明 `expect.error` 且实际类名一致 | `PASS` | T-R2 |
| 仅 `AssertionError`（`assert` 失败 / `fail`） | `FAIL` | T-R3 |
| 清单声明 `expect` 但未报错或类名不符 | `FAIL` | T-R3 |
| 其它任一错误类（`SyntaxError` / `NameError` / … / `CosmosAnswerError`） | `ERROR` | T-R1 + §8.1 |

退出码（`runner-contract §5`、D-008）：

| 退出码 | 条件 |
|---|---|
| `0` | 全部 `PASS` |
| `1` | 无 `ERROR`，但存在 `FAIL` |
| `2` | 存在 `ERROR`；或 runner 环境错误（参数 / 缺目录 / 清单非法 / 未发现用例） |

> 同一轮既有 `FAIL` 又有 `ERROR` 时取 **`2`（ERROR 优先）**（`runner-contract §5` 裁定）。

> **退出码 `1` 是 `lfz test` 专属的"用例失败"**，别和 `lfz run` 混淆（常见困惑）：
> - `lfz run x.lfz` 下，`assert` 失败 / `fail()` → 抛 `AssertionError`（一个**错误类**）→ 退出码 **`2`**。
> - `lfz test` 下，同一个 `assert` 失败的**用例**记为 `FAIL` → 退出码 **`1`**。
> 也就是说：`assert` 本身永远是错误；只有**测试运行器**把 `AssertionError` 归类为"用例失败"并给 `1`。详见 [`errors.md` §3.4](errors.md)。

---

## 6. 完整最小示例（实测）

目录结构：

```text
mini/
├── cases.json
├── pass_basic.lfz          # 正向
├── fail_assert.lfz         # 故意失败，演示 FAIL
└── fixtures/
    └── no_header.lfz       # 缺 #42 的负例
```

`pass_basic.lfz`：
```lfz
#42
// mini/pass_basic.lfz — 一个正向用例
assert(1 + 1 == 2, "加法")
assert([1, 2, 3] |> sum() == 6, "管道求和")
let doubled = [1, 2, 3] |> map((x) => x * 2)
assert(doubled == [2, 4, 6], "map 翻倍")
```

`fail_assert.lfz`：
```lfz
#42
// mini/fail_assert.lfz — 故意让 assert 失败的用例（演示 FAIL 与退出码 1）
assert(1 == 2, "这个断言故意失败")
```

`fixtures/no_header.lfz`（**故意不写 `#42`**）：
```lfz
print("no header here")
```

`cases.json`：
```json
{
  "cases": [
    { "path": "fixtures/no_header.lfz", "name": "缺 #42 前导（负例）", "expect": { "error": "CosmosAnswerError" } }
  ]
}
```

运行（把发现根指向 `mini`）：

```console
$ cargo run --quiet -- test mini
FAIL  mini/fail_assert.lfz  (mini/fail_assert.lfz:3:1)
  Traceback (most recent call last):
    File "mini/fail_assert.lfz", line 3, in <module>
      assert(1 == 2, "这个断言故意失败")
      ^
  AssertionError: 断言失败：这个断言故意失败
PASS  缺 #42 前导（负例）
PASS  mini/pass_basic.lfz

汇总：共 3 个用例，通过 2，失败 1，错误 0
```
退出码 `1`。

要点：
- `pass_basic.lfz` → `PASS`（断言全通过）。
- `fail_assert.lfz` → `FAIL`（`assert` 抛 `AssertionError`，附位置信息与错误块）。
- `fixtures/no_header.lfz` → `PASS`（负例：实际抛 `CosmosAnswerError`，与清单声明一致）。

`--json`（仅展示形态，stdout 为唯一一行）：
```json
{"ok":false,"total":3,"passed":2,"failed":1,"errored":0,"cases":[{"name":"...fail_assert.lfz","path":"...fail_assert.lfz","verdict":"FAIL","error":"AssertionError","message":"断言失败：这个断言故意失败","file":"...fail_assert.lfz","line":3,"col":1,"traceback":[...]},{"name":"缺 #42 前导（负例）","path":"...fixtures/no_header.lfz","verdict":"PASS"},{"name":"...pass_basic.lfz","path":"...pass_basic.lfz","verdict":"PASS"}]}
```

---

## 7. 本仓库的测试集

| 位置 | 内容 |
|---|---|
| `tests/lfz/**/*.lfz` | 正向用例（自动发现；首行恒 `#42`；用 `assert`/`check` 断言） |
| `tests/fixtures/**` | 负例夹具（非 `.lfz` 亦可；被自动发现跳过） |
| `tests/cases.json` | 负例清单（`expect.error` / `name`） |
| `tests/coverage-matrix.md` | **覆盖矩阵**：逐特性 × 正常/边界/错误 |
| `tests/REPORT.md` | **测试报告**：方法、结论、覆盖统计、已知缺陷、复现命令 |

2026-10-25 T12指南初测历史快照为517项Rust测试与166项黑盒，另有91项进程检查（其中包含一次全量黑盒，不与166相加）；最终独立结论见团队T12报告。下列数量指黑盒测试。`lfz test` 在有效且非空的测试集执行前自动准备进程 cwd 下的 Temp/，保留一命令测试约定；准备失败按环境错误退出 2，stdout 为空。直接 `lfz run` 不代建 IO 父目录，单独运行文件 IO 示例时自行准备父目录。读写采用 UTF-8；还应测非法 UTF-8、非 string 参数、缺失父目录，以及 cwd 不同于脚本目录的情形。

当前基线（见 [`tests/REPORT.md`](../../tests/REPORT.md)）：`cargo run --quiet -- test` → **166 PASS / 0 FAIL / 0 ERROR**，退出码 `0`。

### 7.1 运行全量并查看结果

```console
$ cargo run --quiet -- test
...
汇总：共 166 个用例，通过 166，失败 0，错误 0
$ echo $LASTEXITCODE
0
```

---

## 8. 常见问题

| 现象 | 原因 / 解决 |
|---|---|
| 用例被记为 `ERROR`（不是你预期的负例） | 该 `.lfz` 缺 `#42` 或自身写错。负例必须放 `fixtures/` + `cases.json expect`（T-R2）。 |
| 报 `lfz test: 未发现测试目录 'tests'` | 不在仓库根目录，或 `tests/` 不存在。 |
| 报 `lfz test: 未发现任何测试用例` | 发现根下没有 `.lfz`（且清单为空）。 |
| `assert` 失败但想继续 | 改用 `check` 并自行读取返回值（A4，非致命）。 |
| 想调试单条用例 | `cargo run --quiet -- run <用例文件>` 直接运行它。 |

> 测试只依赖 `docs/spec/` 的公开行为，**不读解释器实现、不改解释器源码**；发现缺陷出缺陷单（见 `tests/REPORT.md` §7）。

## 9. 测试参数、JSON 与可复现输入

除正常往返，还应覆盖：空参数/Unicode/字面`--json`、CODE恰为选项文本、stdin非法UTF-8与EOF、精确i64/有限float、解码后重复键、Unicode代理对、128/129层、环与共享非环、方法字段过滤及输出通道。JSON文本偏移按UTF-8字节验证，源码span单独验证。当前可复现进程检查：

```powershell
D:/app/ANACONDA/python.exe -B tests/t12_cli_check.py --lfz dist/lfz.exe --output Temp/guide-cli-check.json --full-suite
```

参数格式以脚本 `--help` 为准；输出报告路径必须尚不存在。完整覆盖与原始证据见 [T12-blackbox.md](../reports/T12-blackbox.md)。应用独立oracle见 [应用说明](../../app/README.md)，验证错误输入不会覆盖已有结果文件、重复参数运行字节一致。

指南的新示例逐项运行证据见 [T12-guide-evidence.md](../reports/T12-guide-evidence.md)。不要由短进程含启动耗时直接推断渐近复杂度；T12性能历史见 [T12-performance.md](../reports/T12-performance.md)；T13内部优化候选因性能回归已拒收，全部失败数据见 [T13-performance.md](../reports/T13-performance.md)。效率门槛R304仍开放，不由功能测试通过推断提速。

历史 `LFZ-AGENT-BENCH-v2` 首轮8/8是当时冻结输入的成绩，不能作为新版本的自动成绩。旧九项输入已保全在 `docs/reports/agent-benchmark-v2/v2-20261024-r01/raw-inputs-submissions.zip`；当前skill或binary变化后，旧manifest在开发目录校验失败是预期。复现旧轮时在项目Temp还原原路径和原字节；新轮另建版本化输入及manifest，不覆盖旧manifest、用例或成绩。

## 10. 把语法检查放在执行前

```powershell
./dist/lfz.exe check examples/hello.lfz
if ($LASTEXITCODE -eq 0) { ./dist/lfz.exe test }
```

check只复用前端，不能替代执行测试。测试框架应分别记录「语法通过」「运行成功」「断言通过」；退出0及`executed:false`只属于第一项。语法有效的除零、未知名、类型错误、文件读写、input和无限循环不应在check期间求值。

非执行回归可准备一个已有哨兵文件和一个不存在的目标，让源码包含writeFile/appendFile；检查成功以及源码后续有语法错时，已有文件都应保持字节不变，新目标都不应创建。另查程序print/eprint/dump/input不产生输出/提示，循环只解析后及时返回。文件与stdin读取仍是宿主IO，不能称完整沙箱。

合法check的JSON成功有且仅有`ok/mode/executed`三个键，错误沿原错误字段；CLI参数错误（如CODE外`--`、缺源或`check --help`）仍stdout空/stderr文本/退出2，即使传了`--json`。本轮只实跑新增指南示例与关键旧例，独立于旧35块全量记录；证据见 [T14-guide-evidence.md](../reports/T14-guide-evidence.md)。

## T15：验证失败之后仍能继续

新用例应分别验证成功nil、坏JSON后继续、嵌套恢复、连续失败后成功、后续未捕获错误的trace不污染、共享容器和文件写入不回滚。边界arity/noncallable、容量错误传播应与业务错误恢复分开测试。不要通过真实OOM/abort测试不可捕获性；资源variant可用运行时单元构造见证。

新增指南聚焦实跑记录见[T15-guide-evidence.md](../reports/T15-guide-evidence.md)，只代表该报告绑定候选的实际例子，不替代历史全部指南块或独立验收。先check再run，并分别记录语法、运行和断言结果；R304仍OPEN。

## T16-C 本地模块

模块回归要分别验证check无初始化、菱形单次初始化、运行间缓存隔离、home globals、标量快照与容器共享、argv、模板默认定义上下文及显式覆盖caller来源。真实多文件见证与RustAPI边界见[模块教程](modules.md)及T16-C指南证据；旧单文件输出也须兼容。

## T17 执行限额

限额测试应核单表达式2步、let后读4步、空程序0成功、共享模块预算与attempt不补步；CODE/argv保护及等号usage须精确。取消只在host token可验证，不把CLI Ctrl-C当结构化取消；native/IO不做毫秒级硬中断断言。实际最终字节证据见T17-guide报告，不复用修前620/F487结果。
