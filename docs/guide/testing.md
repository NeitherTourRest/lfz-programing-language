# LFZ 测试：如何写与跑

> LFZ 内建测试运行器（`lfz test`）。本页讲清：**怎么跑、怎么写、负例怎么放、结果怎么读**。
> 权威契约：[`docs/spec/interface-contract.md`](../spec/interface-contract.md) §11.2（**T-R1 … T-R4**）；实现侧派生：[`docs/tooling/runner-contract.md`](../tooling/runner-contract.md)。
> 配套：[`README.md`](README.md) ｜ [`tutorial.md`](tutorial.md) ｜ [`reference.md`](reference.md) ｜ [`errors.md`](errors.md)
> 最后更新: 2026-09-27（T11-③ D4 退出码语境；用例数刷新为 90） by docs-writer

---

## 1. 一条命令跑测试

在仓库根目录：

```console
$ cargo run --quiet -- test
```

- 默认发现根 = 当前目录下的 `tests/`；递归收集 `tests/**/*.lfz`，**跳过 `tests/fixtures/`**，并读取 `tests/cases.json` 声明负例。
- 每个用例一行 `PASS/FAIL/ERROR <name>`；末尾汇总行逐字符稳定：

```text
汇总：共 90 个用例，通过 90，失败 0，错误 0
```

其它用法：

```console
$ cargo run --quiet -- test <目录或文件>     # 指定发现根 / 单个用例
$ cargo run --quiet -- test --json           # stdout 只写一行 JSON
```

> `--json` 是全局开关，可放在任意位置（`--json test` 亦可）。

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

当前基线（见 [`tests/REPORT.md`](../tests/REPORT.md)）：`cargo run --quiet -- test` → **90 PASS / 0 FAIL / 0 ERROR**，退出码 `0`。

### 7.1 运行全量并查看结果

```console
$ cargo run --quiet -- test
...
汇总：共 90 个用例，通过 90，失败 0，错误 0
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
