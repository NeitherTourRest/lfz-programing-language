# test-engineer — 工作日志
> 只追加，最新条目在最上方。
## [2026-09-27 12:00] P5.2 黑盒测试第二批（控制流/函数/闭包/结构体/管道/富插值/`;;`）
- 来源: team-lead 任务书 **P5.2**（轻量启动）
- 完成: 新增 **10** 个正向用例文件（**170** 条 `assert`）+ **12** 条负例夹具；`cases.json` 扩充至 **33** 负例 + 1 正向豁免；更新 `coverage-matrix.md`（P5.2 矩阵 + §3 不可断言项诚实标注 + §4 缺陷单 + §5 负例清单 + §7 证据 + §8 骨架状态）。
- 产出:
  - `tests/lfz/`：`test_if_else.lfz`(11) / `test_loops.lfz`(16) / `test_functions.lfz`(26) / `test_closures.lfz`(14) / `test_structs.lfz`(27) / `test_pipe.lfz`(15) / `test_interpolation.lfz`(16) / `test_dump.lfz`(7) / `test_reference_semantics.lfz`(29) / `test_cycle_safety.lfz`(9)
  - `tests/fixtures/`：`if_condition_not_bool` / `for_over_string` / `del_method_field` / `del_missing_data_field` / `pipe_rhs_not_function` / `pipe_multiple_placeholders` / `pipe_underscore_in_lambda` / `interp_bad_format` / `interp_format_type_mismatch` / `continue_outside_loop` / `return_outside_fn` / `deep_recursion`（.lfz，均带 `#42`）
  - 证据: `cargo run --quiet -- test` → `汇总：共 51 个用例，通过 51，失败 0，错误 0`，exit **0**；`cargo clean -p lfz; cargo build` → **0 warning / 0 error**；`cargo test` → **431 passed / 0 failed**。
- 决策: `;;` 文本与 `<cycle>` 文本**不做端到端断言**（runner 不捕获程序 stdout），仅保底「不报错」并在矩阵诚实标注；错误断言只断「错误类」。
- 缺陷: 提交 **bug-20260927-01** —— `p["方法"]()` 经方括号取值再调用**未绑定 `self`**（`.字段()` 正常），违反 semantics §3.7「`s.k ≡ s["k"]` …（`self` 绑定）」；最小复现已交 team-lead，`src/**` 未改。
- 下一步: P5.4（内置函数 54 个 data-last、`IndexError`/`IOError` 负例、traceback 折叠与 `--json` 字段）。
- 阻塞: 无（bug-20260927-01 不阻塞本批全绿；修复后补端到端用例并回归）
## [2026-09-27 00:00] P5.1 黑盒测试第一批（基础语言特性）
- 来源: team-lead 任务书 **P5.1**
- 完成: 建立 `tests/lfz/`（正向，自动发现）+ `tests/fixtures/`（负例/豁免夹具）+ `tests/cases.json` + `tests/coverage-matrix.md`；编写 7 个正向用例文件（114 条 assert）+ 21 条负例 + 1 条非 `.lfz` 正向豁免。
- 产出:
  - `tests/lfz/test_literals.lfz`(40 assert) / `test_let_var.lfz`(12) / `test_arithmetic.lfz`(24) / `test_precedence.lfz`(11) / `test_division_modulo.lfz`(14) / `test_int_min.lfz`(8) / `test_preamble.lfz`(5)
  - `tests/fixtures/*`（21 负例 + `plain_ok.txt`）
  - `tests/cases.json`（22 条 manifest：21 负例 `expect.error` + 1 正向豁免）
  - `tests/coverage-matrix.md`（本批矩阵 + §5 后续特性骨架）
  - 证据: `cargo run --quiet -- test` → `汇总：共 29 个用例，通过 29，失败 0，错误 0`，exit **0**；`cargo clean -p lfz; cargo build` → **0 warning / 0 error**；`cargo test` → **431 passed / 0 failed**。
- 决策: 负例全部放 `tests/fixtures/` 经 `cases.json` 声明（遵 runner-contract §2/T-R2）；错误断言只断「错误类」不断文案。
- 下一步: P5.2（插值/格式说明符、数组、struct）。
- 阻塞: 无
## [2026-09-22] 角色初始化
- 来源: 团队初始化（系统构建）
- 完成: 角色定义与活文档建立
- 产出: `.opencode/agents/test-engineer.md`
- 下一步: 等待 team-lead 调度
