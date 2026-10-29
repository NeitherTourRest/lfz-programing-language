# `lfz test` runner 契约（派生物 · 供 test-engineer 使用）

> 唯一写者: tooling-dev ｜ 状态: **v1.3（T11-V01：测试工作目录准备；P4.2-fix：`--json` 下程序输出重定向到 stderr；含 v1.1 的 `--json` 与 P4.1 契约）**
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

### 1.1 测试工作目录准备（T11-V01）

发现与清单校验成功且测试集非空后，runner 在执行首个用例前显式准备 **当前工作目录（cwd）下的 `Temp/`**：缺失时创建，已有目录时复用。默认发现、显式目录与显式文件参数均采用这一流程，文本与 `--json` 模式一致。

runner **不改变 cwd**，所以测试正文中的 `Temp/...` 相对进程启动时的 cwd 解析，与发现根或脚本所在目录无关。该准备仅提供测试集约定的工作目录；`readFile` / `writeFile` / `appendFile` 的语言语义不变，其他缺失父目录仍由内置函数报告 `IOError`。runner 不负责清理测试生成的文件。

若 `Temp` 已是普通文件、权限不足或发生其他创建失败，runner 不执行任何用例，退出码 `2`，stdout 为空，stderr 写 `lfz test: 无法准备测试工作目录 'Temp'：<系统错误>`（包括 `--json` 模式）。发现失败、清单非法或空测试集在此步骤之前返回，不创建 `Temp`。

回归证据：`tests/test_runner.rs::{fresh_cwd_io_suite_prepares_temp,temp_file_conflict_is_runner_environment_error,invalid_or_empty_suite_does_not_prepare_temp}`。第一条直接复用正式 `test_v11_io.lfz`，在无 `Temp` 的独立 cwd 验证默认一命令与显式目录 JSON 两种入口。

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
> `--json` 可出现在 `run` / `test` 的选项位置；T12 起，`-e` 紧随的 CODE 与执行入口之后 `--` 分隔的脚本参数均按字面保留，不参加选项解析。

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
| `--json` 标志解析 | `src/cli.rs::split_options`（保护 CODE 与 `--` 后脚本参数） |
| `run --json` 字段 | `src/cli.rs::{success_json, error_json, push_error_fields, traceback_json}` |
| `test --json` 汇总 | `src/test_runner.rs::run`（`json` 分支）+ `case_json` |
| JSON 编码 | `src/json.rs::encode`（紧凑单行、非 ASCII 原样） |
| 单测 | `json::tests::*`（示例 4 逐字符）、`cli::tests::{error_json_*, success_json_*, execute_run_json_*}`、`test_runner::tests::json_*` |
| e2e | `tests/cli.rs::json_*`（成功 + 缺 `#42` + 语法错 + 运行期错 + 断言失败 + IOError）、`tests/test_runner.rs::json_*` |
| 输出重定向（P4.2-fix） | `src/builtins.rs::{set_stdout_to_stderr, b_print, b_input}`；置位点 `src/cli.rs::run_file` / `src/test_runner.rs::run` |
| 输出重定向 e2e | `tests/cli.rs::json_run_print_success_redirects_to_stderr` / `tests/cli.rs::json_run_print_before_error_redirects_to_stderr` / `tests/test_runner.rs::json_print_case_stdout_is_single_json` |

## 10. T12 代码入口和脚本参数（2026-10-25）

规范来源：[syntax.md §16](../spec/syntax.md)、[interface-contract.md §16](../spec/interface-contract.md)。本节同步实现接口，不改变 test-runner 的发现、期望、汇总或退出码。

- `lfz run FILE [-- ARG...]`、`lfz FILE.lfz [-- ARG...]`：文件入口保持原前导和扩展名规则；只有 `--` 后进入 `argv()`，没有分隔符的额外项仍为 CLI 参数错误。
- `lfz [run] -e CODE [-- ARG...]`：执行单项源码，不要求 `#42`，错误伪路径为 `<command>`。CODE 可以为空，也可以恰好等于 `--json`、`--`、`--help`，都按源码执行。
- `lfz [run] - [-- ARG...]`：从 stdin 读取全部 UTF-8 源码至 EOF；错误伪路径 `<stdin>`。读取失败或非法 UTF-8 为 `IOError: 无法读取标准输入`，无 span；`--json` 中 line/col 为 null、traceback 为 []。
- 两种源码入口复用 `load_source`：BOM/CRLF 归一化、line_base=0，第一行就是 line 1；解析、求值和 AST 析构均处在既有大栈线程。
- stdin 源码先完整读取到 EOF，因此程序随后 `input()` 按原契约得到 EOF 错误；文件和 `-e` 可继续通过 stdin 提供程序输入。
- `--json` 在 CODE/脚本参数区外仍支持原前后位置、重复幂等；程序 print/dump/input 提示转 stderr，stdout 为唯一状态 JSON。不会自动打印最后表达式的值；业务 JSON 使用正常模式 `print(jsonStringify(value))`。
- `argv()` 不含解释器名/文件名/CODE/分隔符，保留空字符串、Unicode、空格和所有以 `-` 开头的值。CLI 经 `eval_module_traced_on_thread_with_args` 传递求值上下文；test-runner 的 `eval_case` 默认为空参数。
- help/version/test 保持此前解析行为；无参仍是参数错误，未新增隐式stdin或REPL。

### 10.1 实现与验证

- 参数拆分：`src/cli.rs::{split_options,execution_ready,parse_command}`。
- 统一加载/求值：`eval_file_with_args`、`eval_loaded_on_stack`、`run_source`；无参数旧文件和runner入口保持可用。
- 原 `tests/cli.rs` 28 条与 bin crate 48 条全部原样通过。
- 新 `tests/cli_t12.rs` 12 条进程集成测试覆盖：两种拼写、空程序/空参数/Unicode、-- 字面参数、源码选项形字符串、机器状态通道、UTF-8失败、错误行号、input和EOF、文件前导与扩展名兼容。
- 复现：项目内配置 RUSTUP_HOME/CARGO_HOME/TEMP/TMP 后执行 `cargo test --test cli --test cli_t12` 及 `cargo test --bin lfz`。
- 首轮日志：[集成测试](./evidence/T12/cli-tests.txt)、[bin单测](./evidence/T12/bin-tests.txt)（原始临时副本 Temp/T12/tooling/）；全量与独立验收由team-lead另行汇总。构建存在环境路径/增量hard-link警告，不声称零warning。
## 11. T14 非执行前端检查（冻结契约§17）

`lfz check FILE`、`lfz check -e CODE`、`lfz check -` 分别检查文件、直接代码及读取至EOF的stdin源码。只调用既有loader、lexer、parser（含AST深度检查），加载、解析、Program析构全位于既有大栈包装内。成功不会建立Interp或调用求值入口、用户内置、输出重定向；宿主仍需读取被检查的源码。通过语法检查不代表名称、类型、函数参数、文件路径或运行行为正确，也不是通用安全沙箱。

- FILE沿显式run加载规则：任意扩展名均可；仅.lfz（ASCII大小写不敏感）要求并消费精确#42前导。CODE/stdin不要求前导，行基数0，伪路径<command>/<stdin>。
- CODE是-e紧随的一个原始argv字符串，保留空串、Unicode以及恰为--json/--的内容；CODE外--json可前后重复。CODE外任何--（含空尾）和脚本参数均拒绝；check --help拒绝，帮助用lfz --help。
- 普通模式成功双流空、exit0；前端失败沿既有诊断写stderr、stdout空、exit2。没有exit1或LFZ程序输出。
- 参数合法的check --json成功唯一一行：`{"ok":true,"mode":"check","executed":false}`。失败前三键相同（ok:false），随后按原顺序error/message/file/line/col/traceback；复用现JSON编码与错误字段构造。前端错误的位置记录不是实际执行栈。
- usage错误继续沿既有CLI约定：即使有--json仍stdout空、stderr文本、exit2；不伪造LFZ错误类。因此唯一JSON保证仅适用于参数合法的check。
- stdin读取失败或非UTF8仍为无span IOError“无法读取标准输入”；文件UTF8/前导/行终止符和错误位置沿原loader，不另写文法。
- 实现边界：src/cli.rs的CheckInput/parse_check_input、check_input、check_loaded_on_stack。旧run/test/argv入口和JSON对象保持既有行为；LFZ源码里的check(condition)仍是原断言内置。

新增进程回归在tests/cli_t14.rs：三来源、CODE/选项保护、usage、加载及准确偏移、前端错误、非静态承诺、无程序输出/输入/循环执行、文件哨兵、解析/AST受控边界、旧JSON与argv。所有测试临时件均在项目Temp/T14，子进程有超时保护。作者证据位于docs/tooling/evidence/T14；独立非执行验收由verifier另行提供。

## 13. T16-B静态模块图检查

冻结契约见三spec §20（含20.9冲突位置和20.10已观测字节预算）。check三来源使用load_file_counted/load_source_counted，raw_bytes来自同一次读取/源码字节，不metadata猜测或重读。根只lex/parse一次，无模块声明保持原前端结果且不启用图预算；有声明时将根快照移交modules::build_graph。文件base为canonical父目录，CODE/stdin为检查开始捕获cwd。整个图构造和Program/图析构均在on_eval_stack内，不初始化、不求值。

失败由GraphFailure.failure.source_id定位SourceMap，用该来源display_path和可选Loaded构造原CaseEval诊断。依赖错误不得画入口文本；没有Loaded的来源不伪造文本。check机器协议仍ok/mode/executed前三键及原错误字段顺序，模块IO无span时line/col null和空traceback。B阶段run/旧eval的模块执行预检由runtime负责，在所有副作用前拒绝；B不宣称支持模块初始化。

新增tests/cli_t16b.rs验证非执行菱形图、CODE/stdin相对基准、依赖真实文件与语法文本、缺失/非法UTF8、静态错误及循环、旧export标识符、legacy大根不套图限额及run零副作用拒绝。测试进程10秒有界超时，所有fixture在项目Temp/T16-B-tooling；不改既有测试。
