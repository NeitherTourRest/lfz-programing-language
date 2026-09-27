# `lfz test` runner 契约（派生物 · 供 test-engineer 使用）

> 唯一写者: tooling-dev ｜ 状态: **v1.2（P4.2-fix：`--json` 下程序输出重定向到 stderr；含 v1.1 的 `--json` 与 P4.1 契约）**
> **权威来源**: [`docs/spec/interface-contract.md`](../spec/interface-contract.md) §11.2（**T-R1 … T-R4**，硬性契约）、
> §8.1（错误类 + 退出码 D-008）、§10.6（`--json` 字段）；[`docs/spec/semantics.md`](../spec/semantics.md) §8.2/§8.3/§8.4（输出格式与 `--json` 字段）、§4.5.10（`assert`/`check`/`fail`）。
> 本文件是上述规范的**实现侧派生说明**：把「发现规则 / 清单 schema / 判定与退出码裁定 / 输出格式」写死，供 test-engineer 按契约编写黑盒测试集（P5）。
> **若本文件与 `docs/spec/` 有任何出入，以 `docs/spec/` 为准并立即报告 tooling-dev。** 本文件不改动 `docs/spec/`。

---

## 1. 用法

```
lfz test              # 默认发现：cwd 下 tests/**/*.lfz（排除 tests/fixtures/**）
lfz test <路径...>    # 目录 → 递归发现该目录下 *.lfz；文件 → 直接作为用例运行
lfz test --json       # 同上，但 stdout 只输出一行 JSON（机器可读；见 §9）
lfz test --json <路径...>
```

- 无参数时**发现根** = 当前工作目录下的 `tests/`；若该目录不存在 → 退出码 `2` + stderr `lfz test: 未发现测试目录 'tests'`。
- 显式路径不存在 → 退出码 `2` + stderr `lfz test: 路径不存在：<path>`。
- 发现 0 个用例 → 退出码 `2` + stderr `lfz test: 未发现任何测试用例`（**不静默成功**）。

---

## 2. 发现规则（T-R4，规范性）

1. **默认根** = `tests`（相对 cwd）。`<路径>` 为目录时，该目录即发现根。
2. 递归收集扩展名匹配 `"lfz"`（`loader::is_lfz`，**ASCII 大小写不敏感**：`x.lfz` / `X.LFZ` / `a.b.Lfz` 均算）的**普通文件**。
3. **排除**：遍历中**跳过任何名为 `fixtures` 的目录**（规范原文为 `tests/fixtures/**`；本实现按目录名 `fixtures` 统一排除，使显式根也受同一规则保护）。T-R2 的负例夹具必须放在**非自动发现**处，默认可放 `tests/fixtures/`。
4. 显式**文件**参数**不做扩展名过滤**（可由清单一并声明非 `.lfz` 夹具）。
5. 结果按路径（`/` 规范化、区分大小写）**稳定排序**后输出 → 可复现。
6. **仅 `.lfz` 文件要求 `#42` 前导**（T-R1 / syntax.md §2.2.0）：由 `loader::load_file` 按扩展名分流，非 `.lfz` 夹具（如 `.txt`）不校验前导。

---

## 3. 用例清单 `cases.json`（T-R2）

发现根目录下若存在 `cases.json`，runner 读取它（`<根>/cases.json`，容忍 UTF-8 BOM）。schema：

```json
{
  "cases": [
    { "path": "fixtures/missing_preamble.lfz", "expect": { "error": "CosmosAnswerError" } },
    { "path": "fixtures/bad_syntax.txt",       "expect": { "error": "SyntaxError" } },
    { "path": "cases/extra.lfz",               "name": "额外用例" }
  ]
}
```

- `cases`（必需）：数组。
- `path`（必需，字符串）：**相对清单所在目录**（即发现根）的路径。可指向**自动发现不到的夹具**（如 `fixtures/` 下、或非 `.lfz` 文件）。指向已发现文件时**就地附加** `expect` / `name`，不重复运行。
- `expect.error`（可选，字符串）：期望该用例以**指定错误类**终止（缺 `#42` → `CosmosAnswerError` 等）。取值必须是 §8.1 的 **12 个类名之一**，否则 runner 报环境错误（退出码 `2`）。
- `name`（可选，字符串）：报告中显示的用例名（默认 = `path`）。

> 清单**只声明**（`expect` / `name`），不发明脚本内容；用例正文仍由 test-engineer 以 `.lfz`（或夹具）编写。

---

## 4. 单用例判定（T-R1 / T-R3 + D-008）

执行链与 `lfz run` 完全一致：`load_file → lex → parse → eval_module_traced`（复用 [`cli::eval_case`]）。

| 情形 | 判定 | 依据 |
|---|---|---|
| 全链路无任何 `LfzError` | **PASS** | — |
| `check(cond)` 返回 `false`（非致命） | **PASS** | A4 / T-R3：`check` 不抛错、不改退出码（警告写 stderr） |
| 清单声明 `expect.error` 且实际类名**一致** | **PASS** | T-R2（负例） |
| 仅 `AssertionError`（`assert` 失败 / `fail`） | **FAIL** | T-R3：仅这两者抛 `AssertionError` |
| 清单声明 `expect` 但**未报错**或**类名不符** | **FAIL** | 期望未达成 |
| 其余任一错误类（`SyntaxError` / `NameError` / … / `CosmosAnswerError`） | **ERROR** | T-R1 + §8.1 + D-008 |

- **`error` 的典型来源**：`.lfz` 缺 / 非法 `#42` → `CosmosAnswerError`（T-R1）；用例正文自身写错（如目标代码 `1 / 0`）。
- 因此：**要测试「负例」（期望报错）必须走清单 `expect` + 非自动发现夹具**（T-R2），否则会被记成 `ERROR`（退出码 `2`）而非 `PASS`。

---

## 5. 退出码（D-008 + §11.2 裁定）

| 退出码 | 条件 |
|---|---|
| `0` | 全部用例 **PASS** |
| `1` | 无 `ERROR`，但存在 **FAIL**（测试失败） |
| `2` | 存在 **ERROR**；或 runner 环境错误（参数 / 缺目录 / 清单非法 / 未发现用例） |

**裁定（tooling-dev）**：当同一轮既有 `FAIL` 又有 `ERROR` 时取 **`2`（ERROR 优先）**——D-008 以「测试失败 = 1」与「所有错误类 = 2」并列，`ERROR` 表示解释器/用例出现非断言类异常，语义上更严重，且 T-R1 明确把缺 `#42` 定为「error（退出码 2）」。
> 列为「**契约待确认**」：`FAIL` 与 `ERROR` 并存时的优先级规范未逐字钉死；如 language-architect / team-lead 另有裁定，按裁定修订并同步本文件与 `DECISIONS.md`。

---

## 6. 输出格式

- **测试报告写 stdout**（report 是 `lfz test` 的产物）；**runner 自身错误写 stderr**。
- 每用例一行状态：

  ```
  PASS   <name>
  FAIL   <name>  (<path>:<line>:<col>)     # 有 span 时附位置
  ERROR  <name>  (<path>:<line>:<col>)
  ```

  非 `PASS` 紧随其后输出缩进 2 空格的 **§8.2 错误块**（`File "<path>", line N[, in <func>]` + 源码行 + 插入符 + `类名: 消息`；运行期错误带 `Traceback` 头，加载/解析期不带）——满足 R-008「失败可定位」与 T-R3「使用 §8.2 位置信息」。
- 末尾汇总（逐字符稳定）：

  ```
  汇总：共 <总数> 个用例，通过 <P>，失败 <F>，错误 <E>
  ```

- **流纪律（P4.2-fix：已解决）**：非 `--json` 模式下，解释器内建 `print` 写 stdout、`check` 警告写 stderr。**`--json` 模式下 CLI 将 `print` / `input` 提示重定向到 stderr**（`lfz::builtins::set_stdout_to_stderr`），故 stdout 恒为**唯一一个 JSON**，程序输出不再与 JSON 交错。测试正文仍建议以 `assert` 表达断言。

---

## 7. 给 test-engineer 的最小接入清单

1. 用例文件放 `tests/**/*.lfz`，**首行必须 `#42`**（否则会被判 `ERROR`）。
2. 负例放 `tests/fixtures/`，在 `tests/cases.json` 里用 `expect.error` 声明期望类。
3. 断言用 `assert(cond, msg)`；需要"软检查"用 `check(cond, msg)` 并自行读返回值（A4）。
4. 本地自测 `cargo run -- test`（或 `lfz test`）应能一键跑全量；`0` / `1` / `2` 见 §5。

---

## 8. 契约条目 → 实现落点对照（T-R1 … T-R4）

| 契约 | 落点 |
|---|---|
| T-R1 | `src/test_runner.rs::judge`（非 `AssertionError` → `ERROR`）+ `loader::load_file`（按扩展名要求 `#42`）；单测 `tr1_missing_preamble_is_error_exit_2` / e2e `missing_preamble_in_tests_is_error_exit_2` |
| T-R2 | `src/test_runner.rs::apply_manifest` + `collect_lfz` 的 `fixtures` 排除；单测 `tr2_manifest_negative_fixture_passes` / `tr2_manifest_expect_mismatch_is_fail` / `tr4_fixtures_dir_excluded_from_discovery`；e2e `manifest_negative_fixture_passes` |
| T-R3 | `src/test_runner.rs::judge`（仅 `AssertionError` → `FAIL`）+ `report` 复用 `cli::render_error`（§8.2 位置）；单测 `tr3_assert_failure_is_fail_exit_1_with_position` / `tr3_other_error_class_is_error_exit_2`；e2e `check_failure_is_non_fatal_pass` |
| T-R4 | `src/test_runner.rs::discover` / `collect_lfz`（§2 全文）；单测 `tr4_fixtures_dir_excluded_from_discovery`；e2e `default_discovery_reports_mixed_pass_and_fail` / `nested_and_fixtures_exclusion` |

---

## 9. `--json` 机器可读输出（P4.2）

> 权威字段来源：`semantics.md` **§8.3 示例 4** / **§8.4**（`run --json`）；`test --json` 的汇总形态为本契约扩展（spec 未定义，**与 `run --json` 同源复用错误字段**）。
> **硬性约束**：`--json` 模式下 **stdout 只写一行 JSON**；人类可读的报告 / 诊断一律转 **stderr**。
> `--json` 为全局开关，可出现在 `run` / `test` 子命令的任意位置。

### 9.1 `lfz run --json <file>`

- 成功 → `{"ok":true}`（退出码 `0`）。
- 失败 → 字段与顺序**逐字符对齐** §8.3 示例 4：

  ```json
  {"ok":false,"error":"<12 类名之一>","message":"<中文消息>","file":"<path>","line":N,"col":M,"traceback":[{"file":"<path>","line":N,"func":"<module>|<fn 名>|<fn>"}]}
  ```

  | 字段 | 类型 | 说明 |
  |---|---|---|
  | `ok` | bool | 恒 `false`（失败）；成功时恒 `true` |
  | `error` | string | `LzError::class_name()`，**12 类名之一**；**禁止** `E-xxx` |
  | `message` | string | `LzError::message()`（中文） |
  | `file` | string | 传入的 `<file>` **原样**（`\` 不归一化；JSON 内 `\` 转义为 `\\`） |
  | `line` / `col` | int \| null | `LzError::span()`（1-based）；`IOError` 无 span → `null` |
  | `traceback` | array | 运行期：完整帧表（自外→内，**不折叠**，§8.4）；加载/解析期：**合成单帧 `<module>`**；无 span → `[]` |

- 退出码仍 **D-008**（错误 → `2`）。

### 9.2 `lfz test --json [路径...]`

- stdout 为**唯一一行 JSON**：

  ```json
  {"ok":<bool>,"total":N,"passed":P,"failed":F,"errored":E,"cases":[ … ]}
  ```

  其中 `ok = (failed == 0 && errored == 0)`。
- 每用例对象：`{"name":…,"path":…,"verdict":"PASS"|"FAIL"|"ERROR"[,…错误字段]}`；
  非 `PASS` 追加 §8.4 错误字段（`error` / `message` / `file` / `line` / `col` / `traceback`），
  与 `run --json` **同源**（复用 `cli::push_error_fields`）。
- `path` / `name` 按 `/` 规范化（复用发现规则，跨平台可复现）。
- 退出码同 §5（`0` / `1` / `2`）。
- **环境错误**（参数 / 缺目录 / 清单非法 / 无用例）：stdout **为空**，诊断写 **stderr**，退出码 `2`。

### 9.3 程序输出的重定向（P4.2-fix：**已解决**）

`--json` 模式下，CLI 在运行前调用 `lfz::builtins::set_stdout_to_stderr(true)`，把解释器内建 `print` 与 `input` 的**提示输出**重定向到 **stderr**；`eprint`（本就 stderr）与 `check` 警告亦在 stderr，二者不受开关影响。因此 **stdout 恒为唯一一行 JSON**——`run --json` 与 `test --json` 皆然——被运行程序 / 用例正文的 `print` 输出一律出现在 stderr，不再与 JSON 交错。默认（非 `--json`）目标仍为 stdout，原行为不变。

> 实现：`src/builtins.rs` 进程级 `static STDOUT_TO_STDERR: AtomicBool` + `pub fn set_stdout_to_stderr(on: bool)`；`b_print` / `b_input` 按开关择流，`b_eprint` 不变。置位点：`cli::run_file` 与 `test_runner::run`（均以 `json` 形参）。

### 9.4 契约条目 → 实现落点

| 条目 | 落点 |
|---|---|
| `--json` 标志解析 | `src/cli.rs::execute`（任意位置摘除） |
| `run --json` 字段 | `src/cli.rs::{success_json, error_json, push_error_fields, traceback_json}` |
| `test --json` 汇总 | `src/test_runner.rs::run`（`json` 分支）+ `case_json` |
| JSON 编码 | `src/json.rs::encode`（紧凑单行、非 ASCII 原样） |
| 单测 | `json::tests::*`（示例 4 逐字符）、`cli::tests::{error_json_*, success_json_*, execute_run_json_*}`、`test_runner::tests::json_*` |
| e2e | `tests/cli.rs::json_*`（成功 + 缺 `#42` + 语法错 + 运行期错 + 断言失败 + IOError）、`tests/test_runner.rs::json_*` |
| 输出重定向（P4.2-fix） | `src/builtins.rs::{set_stdout_to_stderr, b_print, b_input}`；置位点 `src/cli.rs::run_file` / `src/test_runner.rs::run` |
| 输出重定向 e2e | `tests/cli.rs::json_run_print_success_redirects_to_stderr` / `tests/cli.rs::json_run_print_before_error_redirects_to_stderr` / `tests/test_runner.rs::json_print_case_stdout_is_single_json` |
