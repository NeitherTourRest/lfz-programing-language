# test-engineer — 工作日志
> 只追加，最新条目在最上方。
## [2026-09-27 19:45] P5.6 等价路径双路补测（关闭接缝盲区；verifier P9 rev.2 §9.4 建议）
- 来源: team-lead 任务书 **P5.6**（关闭「等价路径接缝盲区」；轻量启动）
- 完成:
  - **新增 `tests/lfz/test_equivalence_paths.lfz`（41 `assert`）**：`s.k≡s["k"]` 读 / 写 / 调用 / 改 `self`（同一实例）；复合赋值脱糖；管道 data-last 脱糖（sum/take/slice/map/filter/reduce/len/裸函数/占位符）；块注释 ≡ 空格；`a--b ≡ a-(-b)`；逻辑分组 + 短路；`/` vs `div` 非等价并置。
  - **补强 `tests/lfz/test_structs.lfz`（27→36 `assert`）**：`p["norm2"]()==25`、`p["describe"]()`、`cc["bump"]()` 写回**同一实例**、两实例分别用点/方括号驱动最终值相同、数据字段写两路可见；改写过时注释（"仅在 `.字段()` 调用点绑定"）。
  - **负例 +2**：`fixtures/{field_dot_missing,field_bracket_missing}.lfz` → `FieldError`（缺失键读取两取法等价报错）；`tests/cases.json` 56→58。
  - **矩阵新增 §1.3「等价路径双路覆盖小节」**（E1–E11 + 诚实列出「未找到等价表述 / 无法断言」5 条）；`REPORT.md` / 矩阵数字全量同步；bug-20260927-01 标记**已修复**。
  - **先行探针**：9 个临时 probe 验证 `a[i]+=`、`s.k+=`、`5--3`、内联 `/*c*/`、`p["k"]()`、管道等价全部可用（用完即删）。
- 产出:
  - `tests/lfz/test_equivalence_paths.lfz`（新）、`tests/lfz/test_structs.lfz`（改）、`tests/fixtures/field_{dot,bracket}_missing.lfz`（新）、`tests/cases.json`（改）、`tests/coverage-matrix.md`（改）、`tests/REPORT.md`（改）。
  - 证据: `cargo run --quiet -- test` → **85 PASS / 0 FAIL / 0 ERROR，exit 0**（82→85）；`cargo clean -p lfz; cargo build` → **0 warning / 0 error**；`cargo test` → **432 passed / 0 failed / 0 ignored**（lib 362）；`git status --porcelain` 仅 `tests/**`（+他人 verifier 文件）。
- 决策: ADR [2026-09-27 19:45] **等价双路纪律**（凡 spec 声明等价，两路都须有用例且结果一致）。
- 下一步: P5.5（可移植性 / 入口一致性）；bug-02（`test --json` 多行）仍待 tooling 裁定。
- 阻塞: 无（`src/**` 未改，无新缺陷单）。

## [2026-09-27 18:20] P5.4 黑盒测试集定稿批（12 类错误核对 + 查漏补缺 + 覆盖矩阵定稿 + REPORT）
- 来源: team-lead 任务书 **P5.4**（轻量启动；只读 STATUS.md + coverage-matrix.md）
- 完成:
  - **12 类错误矩阵核对**（对照 `interface-contract.md` §8.1）：逐类统计 `cases.json` 的 `expect.error`，确认 **11/12** 类有黑盒触发用例（`CosmosAnswerError`5 / `SyntaxError`11 / `NameError`1 / `TypeError`11 / `IndexError`6 / `FieldError`2 / `ZeroDivisionError`3 / `OverflowError`5 / `ValueError`10 / `AssertionError`1 / `RecursionError`1）；**`IOError` 黑盒跳过**（runner 不提供 stdin，`input()` 有阻塞/EOF 不可复现风险），按任务授权注明**由 Rust 单测覆盖**（`src/builtins.rs::tests::input_reads_line_crlf_and_eof` EOF→`IOError`、`src/error.rs::tests::class_name_all_twelve_match_spec_exactly`）；基类 `LfzError` 不直接抛出（spec §8.1）。
  - **查漏补缺 §10.7 内置**：54 个逐项核对 → **53 黑盒 + 1 跳过（`input`）**；`print`/`eprint`/`check` 输出文本不可捕获 → 仅断言「返回值/不报错」，在矩阵/报告注明替代覆盖者。
  - **`tests/coverage-matrix.md` 重构定稿**：§0 总览（用例构成/判定退出码/覆盖结论）、§1 特性覆盖表（词法/表达式/语句/函数/闭包/结构体/管道/插值/`;;`/A1/A6）+ 特性域状态总览、§2 内置 54 逐个、§3 **错误类 12 逐个**、§4 **不可断言项清单（8 条，逐条「为什么测不到 + 由谁覆盖」）**、§5 负例 56、§6 正向 25、§7 缺陷单、§8 运行证据、§9 后续批次。
  - **新建 `tests/REPORT.md`**：测试方法（黑盒 + runner 契约 T-R1…T-R4）、运行方式（`cargo run --quiet -- test` + 退出码 0/1/2 语义）、用例总数与构成（82 = 正向 25 文件/547 assert + 负例 56 + 豁免 1）、覆盖矩阵摘要（特性/内置/错误类）、不可断言项与理由、如何复现（可复制命令）、回归记录。
- 产出:
  - `tests/coverage-matrix.md`（定稿，重构）
  - `tests/REPORT.md`（新建）
  - `.opencode/team/DECISIONS.md` 追加 ADR「黑盒覆盖边界：`IOError`/`input` 交由 Rust 单测」
  - 证据: `cargo run --quiet -- test` → `汇总：共 82 个用例，通过 82，失败 0，错误 0`，exit **0**；`cargo clean -p lfz; cargo build` → **0 warning / 0 error**；`cargo test` → **431 passed / 0 failed / 0 ignored**（361+42+16+12）。
- 决策: 新增条目见 `DECISIONS.md` [2026-09-27 18:20]；黑盒只断「错误类」不断消息/span/输出文本；`IOError`/`input` 由 Rust 单测覆盖（任务授权）。
- 下一步: P5.5（可移植性 BOM/CRLF/非 UTF-8；入口一致性 REPL/stdin/`-e`）；bug-20260927-01 修复后补 `p["方法"]()` 端到端用例并回归。
- 阻塞: 无（bug-20260927-01 仍待修，不影响定稿全绿；`src/**` 未改）
## [2026-09-27 15:00] P5.3 黑盒测试第三批（内置函数全表 54 个逐个覆盖）
- 来源: team-lead 任务书 **P5.3**（轻量启动）
- 完成: 新增 **8** 个正向用例文件（**263** 条 `assert`）+ **23** 条负例夹具；`cases.json` 扩充至 **56** 负例 + 1 正向豁免；`coverage-matrix.md` 新增 **§2b 内置 54 个逐个对照表**，并更新 §3 诚实标注 / §5.3 负例清单 / §6.3 正向清单 / §7 证据 / §8 骨架状态。
- 产出:
  - `tests/lfz/`：`test_builtins_array.lfz`(70) / `test_builtins_higher_order.lfz`(28) / `test_builtins_struct.lfz`(26) / `test_builtins_string.lfz`(33) / `test_builtins_math.lfz`(43) / `test_builtins_convert.lfz`(39) / `test_builtins_random.lfz`(10) / `test_builtins_io.lfz`(14)
  - `tests/fixtures/`（23 条，均带 `#42`）：`pop_empty`(IndexError) / `removeAt_out_of_range` / `removeAt_negative_out_of_range` / `insert_negative_index` / `insert_index_too_large` / `swap_out_of_range` / `min_empty` / `max_empty` / `minBy_empty` / `maxBy_empty`(ValueError) / `filter_predicate_not_bool` / `join_non_string_element` / `sort_mixed_types` / `div_non_int`(TypeError) / `int_nan` / `int_bad_string` / `float_bad_string` / `randInt_bad_range` / `floor_nan`(ValueError) / `int_inf` / `sum_overflow` / `ceil_inf`(OverflowError) / `fail_raises`(AssertionError)
  - 证据: `cargo run --quiet -- test` → `汇总：共 82 个用例，通过 82，失败 0，错误 0`，exit **0**；`cargo clean -p lfz; cargo build` → **0 warning / 0 error**；`cargo test` → **431 passed / 0 failed / 0 ignored**。
- 决策:
  - **54 内置中 53 个**由 LFZ 黑盒覆盖；唯 `input` **跳过自动发现**（runner 不提供 stdin，可能 EOF/阻塞，不稳），按任务授权注明**由 Rust 单测 `src/builtins.rs::tests::input_reads_line_crlf_and_eof` 覆盖**。
  - **内置名非一等值**：`type(type)` → `NameError`；spec §10.7 未声明内置名可作实参，故不作断言（矩阵 §3 诚实标注）。
  - `print`/`eprint`/`check` 输出文本不可捕获 → 只断言返回值。
  - 负例实际错误类与 §8.1 期望**逐项吻合**（23/23），无新缺陷单。
- 下一步: P5.4（值显示/相等语义全量；错误模型 traceback 折叠 + `--json` 字段；`IOError` 非 input 路径）；P5.5（可移植性 / 入口一致性）。
- 阻塞: 无（bug-20260927-01 仍待修，不影响本批全绿）
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
