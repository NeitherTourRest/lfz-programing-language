# runtime-dev — 工作日志
> 只追加，最新条目在最上方。
## [2026-09-24 08:54] P3.11 bug-06 接线完成 — `let` 重绑定检查落地，摘掉 `#[ignore]`（lib 361 passed / 0 ignored）
- 来源: team-lead 任务书「完成 bug-20260924-06 的最后一步：接线 `let` 重绑定检查，并摘掉阻塞占位 `#[ignore]`」。依据 ADR `.opencode/team/DECISIONS.md` `[2026-09-24 00:30] …P3.11 验收 3 处规范裁定` 裁定 1 #2、`docs/spec/semantics.md` §4.5.2/§8.1、`docs/spec/interface-contract.md` §10.8。
- 完成:
  - **前提已满足**：core-dev 已在 `src/error.rs` 落地 `TypeMsg::ImmutableRebind { name: String }`（消息逐字符 `不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容`，`class_name()` 仍映射 `TypeError`）。**未改 `error.rs`**（红线）。
  - **`src/evaluator.rs` 接线（tri-state）**：新增模块级私有 `enum AssignOutcome { Assigned, Immutable, NotFound }`；`Interp::assign_name` 返回类型由 `bool` 改为 `AssignOutcome`，沿 **env 链 → 捕获 cell → globals** 内→外解析，命中 `Env::local_mutable(i) == Some(false)` / 捕获 `mutable == false` → `Immutable`（**不写入**）；`exec_assign` 无后缀分支 match 三态：`Assigned` → `Ok`，`NotFound` → 既有 `NameError`（`self` 保持 `NotFound`，行为不变），`Immutable` → `type_error(TypeMsg::ImmutableRebind { name }, target.span)`（`span` = 赋值目标首字符）。
  - **捕获 cell 携带可变性**（ADR 裁定 1 #2 要求）：`src/value.rs` 新增 `pub type CapturedVar = (Rc<str>, Cell, bool)`，`UserFn.captured` 由 `Rc<Vec<(Rc<str>, Cell)>>` 改为 `Rc<Vec<CapturedVar>>`；`evaluator.rs` `Scope.captured` 同步改型；`make_closure` 捕获时读 `Env::local_mutable(idx)`（`unwrap_or(true)`）写入标志并随继承传递 → 支持**闭包内**判定捕获的 `let` 重绑定。
  - **`let`/`var` 定义与 `a[i]=` / `s.k=` 容器路径不变**（A1；后两者不经 `assign_name`）。
  - **摘除 `#[ignore]`**：`let_rebind_is_type_error` 移除 `#[ignore]` 及其注解，并扩充为覆盖两条路径（① 顶层 `let`（globals）；② 闭包捕获的 `let`（captured cell））——保持为**单个**测试函数以维持验收计数 **361**。消息断言以 `error.rs` 落地实现为准（逐字符）。
- 产出:
  - `git diff --stat -- src/value.rs src/evaluator.rs` → `2 files changed, 111 insertions(+), 38 deletions(-)`（evaluator.rs 139 行变动、value.rs 10 行）。**仅动自有模块**；未改 `src/span.rs`/`src/error.rs`/`src/loader.rs`/`src/lexer.rs`/`src/ast.rs`/`src/parser.rs`、未改 `docs/spec/`、`Cargo.toml`。
  - `cargo build` → `Finished`（**0 warning**）；`cargo build --tests --message-format=json 2>$null` → `WARN_COUNT=0`。
  - `cargo test` → lib **`361 passed; 0 failed; 0 ignored`**（基线 360 passed / **1 ignored** → ignored **1 → 0**）+ bin **9 passed** + `tests/cli.rs` **7 passed**（全绿）。
  - `cargo test --lib let_rebind_is_type_error` → `1 passed; 0 ignored`；`cargo test --lib var_rebind_is_still_allowed` → `1 passed`。
  - **bug-06 真实 CLI 证据**（临时文件置于预批准 `%TEMP%\opencode`，均以 `#42` 前缀）：`let a = 1; a = 2` → **EXIT=2**，stderr `TypeError: 不能重新赋值 let 变量 'a'；let 只锁重绑定，不锁内容`；闭包捕获版 → **EXIT=2**，同消息（`'x'`）；`var b = 1; b = 2; print(b)` → **EXIT=0**、stdout `2`。
  - **bug-09 未回归**：深递归 `.lfz` → **EXIT=2**，stderr **123 行**、`  File "` 帧行 **40**、第 32 行 `  ... 省略 9961 帧 ...`、末行 `RecursionError: 递归深度超限（超过 10000 层）`。
- 决策: 无新增 ADR（严格落地既有裁定 1；`CapturedVar` 为裁定 1 #2 明令「捕获 cell 须携带可变性」的实现细节）。
- 下一步: 待 team-lead 核验；建议 release-manager 提交 `fix(p3): enforce let rebind TypeError (bug-20260924-06)`。
- 阻塞: 无。
## [2026-09-24 08:46] P3.11 缺陷落地 — bug-09（traceback 折叠）完成；bug-06（`let` 重绑定）阻塞于 core-dev
- 来源: team-lead 任务书「落地架构师已裁定的两条非阻塞缺陷」；依据 ADR `.opencode/team/DECISIONS.md` `[2026-09-24 00:30] [language-architect] P3.11 验收 3 处规范裁定`（裁定 1 / 裁定 3）、`docs/spec/semantics.md` §4.5.2/§8.2/§8.4、`docs/spec/interface-contract.md` §10.3/§10.8。
- 完成:
  - **bug-20260924-09（完成）**：`src/cli.rs` `render_error` 的人类可读 traceback 增加**显示层折叠**（§8.2）：帧数 `T ≤ 40` 逐帧原样；`T > 40` → **首 10 帧** → 一行 `  ... 省略 {T−40} 帧 ...`（前缀 2 空格）→ **尾 30 帧**。新增规范常量 `TRACEBACK_HEAD=10` / `TRACEBACK_TAIL=30` / `TRACEBACK_FOLD_THRESHOLD=40`，新增 `push_frames` / `push_trace_frame`。**数据层不动**：`TracedRun.frames` / `LzError` 保持**完整不折叠**（§10.3 / §8.4）。
  - **bug-20260924-06（阻塞，按任务书 ⚠️ 停工）**：ADR 裁定需**新增 `TypeMsg::ImmutableRebind { name: String }`**（`src/error.rs`，属 core-dev）。`Select-String src\*.rs -Pattern ImmutableRebind` 确认**当前不存在**（仅 `docs/spec/interface-contract.md` §10.8 有定义）→ 无法命名该变体 → 无法抛出 → 负例无法通过。**未改 `src/error.rs`**（红线）；求值器侧接线待 core-dev 落地后执行（清单见 STATUS「待办 / 阻塞」）。
- 产出:
  - `git diff --stat -- src/cli.rs src/evaluator.rs` → `cli.rs +102/−6`、`evaluator.rs +29`；仅动自有模块。
  - `cargo build --tests --message-format=json` → **warnings=0 errors=0**。
  - `cargo test` → **360 lib**（`359 passed / 1 ignored / 0 failed`）+ **9 bin**（全过）+ **7 tests/cli**（全过）= **375 passed / 0 failed / 1 ignored**（基线 `373 passed`，未破坏）。
  - bug-09 回归 `cli::tests::deep_recursion_traceback_is_folded`（bin）→ **ok**；bug-06 正例 `evaluator::tests::var_rebind_is_still_allowed` → **ok**；bug-06 负例 `evaluator::tests::let_rebind_is_type_error` → **`#[ignore]`**（`--ignored` 运行证实当前因缺陷而 panic：`let 重绑定应报错`）。
  - 真实 CLI 证据：`lfz run` 深递归文件 → stderr **123 行**（原 ~30000 行），`  File "` 帧行 **40**，含逐字符 `  ... 省略 9961 帧 ...`（T=10001，N=9961），末行 `RecursionError: 递归深度超限（超过 10000 层）`，退出码 **2**。
- 决策: 无新增 ADR（严格照 language-architect 裁定，无自主设计）。
- 阻塞: **bug-06 待 core-dev 在 `src/error.rs` 增 `TypeMsg::ImmutableRebind { name: String }`**。落地后 runtime-dev 接线 `exec_assign`（tri-state + captured 可变性）并移除 `#[ignore]`。
- 下一步: 待 core-dev #1 → 完成 bug-06 求值器接线 + 启用负例。
## [2026-09-24 04:00] P3.9b — `src/builtins.rs`：7 个高阶内置（§10.7 全表 54/54 齐备）
- 来源: team-lead 任务书「P3.9b — `src/builtins.rs`：7 个高阶内置」；依据 `docs/spec/interface-contract.md` §10.7（HOF 签名/语义 + 边界补钉）、§8.1（错误类↔消息）、§10.8（`R<T>`），`semantics.md` §4.5.6（全序）、§8.1。
- 完成:
  - **调用能力注入（关键决策）**：新增 `pub type Invoke<'a> = &'a mut dyn FnMut(&Value, &[Value], Span) -> R<Value>` 与 `HofFn`；7 个 HOF 只依赖此窄接口。**为何 `&mut dyn FnMut` 而非任务书推荐的 `&dyn Fn`**：`call_user` 需 `&mut self`（改递归深度 `depth` + 压/弹 `trace` 帧栈），共享借用无法表达可变能力。**为何不做求值器特判**：会割裂内置表并破坏「脱离 evaluator 可测」。
  - **求值器侧**：新增 `Interp::call_value`（非函数 → `NotCallable`；函数 → `call_user`）；`eval_call` 内置派发由 `builtins::call` 改调 `builtins::call_with(..., &mut invoke)`（`invoke` = 就地闭包）。
  - **表结构**：保留 `TABLE: [Builtin; 47]` 与 `lookup` / `call` **语义与签名不变**；新增 `HOF_TABLE: [Hof; 7]`、`lookup_hof`、`HOF_NAMES`、`call_with`；`is_builtin` = 两表并集；`BUILTIN_NAMES` 47 → **54**。
  - **7 个 HOF 语义**（全部 data-last、A1 返回新值、错误带调用点 span）：`map` 逐元素；`filter` 谓词须 `bool` 否则 `TypeError::ConditionNotBool`；`reduce` 左折叠 `f(acc,x)`、空→`init`；`sortBy` 按 key 升序**稳定**；`minBy`/`maxBy` 空→`ValueError`「空数组没有极值（{func}）」、非空返回原元素；`each` 仅副作用、返回 `nil`。回调**先校验**为函数（空容器也对非法 `f` 报错）。
- 产出:
  - `Get-ChildItem src\builtins.rs,src\evaluator.rs` → builtins.rs **90390 B**、evaluator.rs **119972 B**（合法 UTF-8）。
  - `cargo build` → `Finished`（**0 warning**）；`cargo build --tests --message-format=json 2>$null` → `warnings=0 errors=0`。
  - `cargo test` → **`256 passed; 0 failed; 0 ignored`**（基线 246 + 净新增 10）。
  - 新增测试（11；替换 1 条过时断言）：builtins `hof_map_returns_new_array_and_leaves_original` / `hof_filter_requires_bool_predicate` / `hof_reduce_folds_left_to_right` / `hof_sort_by_is_stable_and_uses_key` / `hof_min_by_max_by_and_empty_value_error` / `hof_each_visits_all_and_returns_nil` / `hof_callback_type_and_arg_count_checks` / `hof_names_are_registered`；evaluator `e2e_hof_map_routes_and_rejects_non_function_callback` / `e2e_hof_data_last_and_arg_count_checks` / `hof_drives_user_closures_through_evaluator`。单测均传 **stub 回调**（不依赖 evaluator）；另含 2 条 `lex+parse+eval` + 1 条程序化 AST 闭包全链路。
- 决策: 追加 ADR `[2026-09-24 04:00] [runtime-dev] P3.9b 高阶内置：调用能力注入（Invoke ABI）+ §10.7 全表 54/54 齐备`。**未**改 `docs/spec/`、`error.rs`、`span.rs`、`loader.rs`、`lexer.rs`、`ast.rs`、`parser.rs`、`Cargo.toml`。
- 缺口（上报，不自行发明）: (1) parser 尚无 lambda → HOF 成功路径暂以程序化 AST 覆盖，待 lambda 落地改回 e2e；(2) 请 architect 确认「回调先校验 / `filter` 非 bool 用 `ConditionNotBool`」； (3) v1 内置非一等值 → `map(len, xs)` 不可写（与 spec §9 样例一致）。
- 下一步: 待 team-lead 核验；parser lambda 落地后补成功路径 e2e。
- 阻塞: 无。

## [2026-09-24 02:30] P3.8 — `src/evaluator.rs` 语义定稿（§4.5 确定性八项 / A4 / A5 / A6 / RecursionError / `;;` / traceback）
- 来源: team-lead 任务书「P3.8 — `src/evaluator.rs` 语义定稿」，依据 `docs/spec/semantics.md` §4.5（全节）/§3.6/§3.7/§8，`interface-contract.md` §10.3/§10.4/§10.5/§10.8。
- 完成:
  - **§4.5 确定性八项**：B1 求值序（二元左→右、赋值 key→rhs、数组/struct 字面量书写序、短路）；B2 `for` 迭代快照；B3 struct 键 UTF-8 字节序升序；B4 迭代/递归栈上限；B6 `float` IEEE（`NaN` 比较、`±Inf`、除零不产 Inf）；B7 struct 实例化平拷贝；B13 `int→float` 加宽（唯一入口 `Value::as_f64`）+ 精确比较。
  - **RecursionError（B4）**：`Interp.depth` 帧深计数，`> RECURSION_LIMIT(10000)` → `LzError::Recursion { depth, limit, span }`；**深结构**路径经新 `Value::deep_eq_bounded(other, limit) -> Result<bool,u32>`（`==`/`!=` 接入，超限转 `RecursionError`）。
  - **求值线程（关键工程决策）**：树遍历器 10000 层递归在 8 MiB / 64 MiB 栈均**栈溢出**，故 `eval_module_traced` 在 **256 MiB 大栈线程**上求值；结果经 `Transfer<T>`（`unsafe impl Send`，`join` happens-before 保证不并发）跨线程移交。
  - **A4**：`check` 非致命（`builtins` 已实现；单测确认返回 `false` 且**继续执行**、不产 `LzError`）；`assert` / `fail` 致命（`AssertionError`，消息构造方组装：`断言失败：{msg}` / `{msg}`）。
  - **A5**：单测锁定 struct 数据面（`keys`/`values`/`entries`/`display`/`==`/`has`/`len`）**不含**函数值字段。
  - **A6**：`==` 环安全（重访即相等）+ 身份优先；`!=` 取反；端到端构造自引用环验证不报错、不死循环。
  - **`;;`（Dump）**：新增纯函数 `pub fn render_dump(&[(Rc<str>, Value)]) -> String`（行格式 `<name> ： <value>`，U+FF1A 全角冒号）；`visible_entries` 沿可见链**内→外**、同层**声明序（slot 升序）**、**遮蔽去重**；Dump 处理器仅收集 + 写 **stdout**。
  - **traceback**：新增 `pub fn eval_module_traced(&Program) -> TracedRun` 与 `TracedRun::frame_name`；按 §10.3 N1「进入 Call 先更新当前帧 span 为调用点，再压新帧」，帧栈自最外层→最内层，位置用 `Span`（line/col）。
  - **跨模块扩展**（只改 runtime-dev 自有模块）：`value.rs` 增 `deep_eq_bounded`（`deep_eq` 签名/行为不变）；`env.rs` 增 `Env::bindings()`（声明序）。
- 产出:
  - `Get-ChildItem src\evaluator.rs,src\env.rs,src\value.rs` → evaluator.rs **114981 B**、env.rs **22303 B**、value.rs **45986 B**（均合法 UTF-8）。
  - `cargo build --tests --message-format=json 2>$null` → **warnings = 0**。
  - `cargo test` → **`226 passed; 0 failed; 0 ignored`**（新增 `evaluator::tests::*` **15**，全绿）。
  - 新增测试（15）：b1_evaluation_order_is_left_to_right / b2_for_iterates_over_snapshot / b3_struct_keys_are_byte_order_sorted / b4_deep_recursion_yields_recursion_error / b6_float_ieee_rules / b7_struct_instantiation_is_flat_copy / b13_int_float_widening_is_the_only_implicit_conversion / a4_check_nonfatal_assert_and_fail_fatal / a5_struct_data_plane_excludes_method_fields / a6_equality_is_cycle_safe_and_identity_first / dump_render_is_pure_and_formats_lines / dump_visible_entries_inner_to_outer_shadow_dedup / dump_statement_runs_and_returns_nil / traceback_frames_follow_call_site_rule / end_to_end_lex_parse_eval_smoke。
- 决策: 追加 ADR [2026-09-24 02:30]（求值大栈线程 + `TracedRun` traceback ABI + `deep_eq_bounded` + 规范缺口）。**未**改 `docs/spec/`、`error.rs`、`ast.rs`、`parser.rs`、`span.rs`、`lexer.rs`、`loader.rs`、`Cargo.toml`。
- 缺口（上报，不自行发明）: (1) `LzError`（§10.4）无帧栈字段而 §10.3 要求写入 → 新增 `eval_module_traced` 带出帧栈，请 architect 确认；(2) `let` 重绑定错误类未定义（仍不强制）；(3) `Display`/`fmt` 无法返回 `RecursionError`（深结构仅 `==` 路径收口）；(4) `StructDef` 模板 `==` 仍同一性（承 P3.6b）。
- 事故与恢复: 一次误用 PowerShell `Get-Content`+`Set-Content` 改 `src/evaluator.rs`，导致 UTF-8 中文双重编码 + 换行吞并（源码结构性损坏）；经 `git checkout HEAD -- src/evaluator.rs`（HEAD 恰为 P3.7 基线 79980 B）恢复后，**全程改用 `edit`/`write` 工具**重做本批编辑。已将教训写入 STATUS「关键经验」。
- 下一步: 待 team-lead 核验 P3.8；P3.9b（7 个高阶内置）。
- 阻塞: 无。

## [2026-09-24 01:30] P3.7 — `src/evaluator.rs` 核心（树遍历求值器：表达式 / 语句 / 控制流 / 函数与闭包 / 调用）
- 来源: team-lead 任务书「P3.7 — `src/evaluator.rs` 核心」，依据 `docs/spec/semantics.md` §3.6/§3.7/§4/§4.2/§4.5.1–§4.5.11，`interface-contract.md` §10.2/§10.3/§10.5/§10.6/§10.7/§10.8。
- 完成:
  - **公开入口**：`pub fn eval_module(&Program) -> R<Value>`（返回最后一条语句值）、`pub fn run(&Program) -> R<()>`。（AST 类型名为 `Program`，非 `Module`。）
  - **表达式**：字面量（Int/Float/Str/Bool/Nil）、标识符、`self`、数组、struct 字面量/实例化、字段、下标、调用、一元（`-`/`!`）、二元（算术/比较/相等）、短路逻辑、`if`、插值（含格式说明符子集 `[fill]align/sign/0/width/.precision/type`）、lambda。
  - **语句**：`let`/`var`、赋值（`=`/`+=`/`-=`/`*=`/`/=`/`%=`；`a[i]=v`/`s.k=v` **原地修改** A1；嵌套路径写入）、`if`、`while`、`for…in`（array 元素快照 / struct 数据字段键字节序快照，§4.5.4）、`break`/`continue`/`return`、`fn` 声明（递归）、`struct` 声明（默认值 + 方法）、块作用域。
  - **A2 捕获**：闭包创建时对非顶层具名局部 `Env::capture_local` 原地升级为共享 `Cell`，闭包与定义作用域共享；`for`/`while` 每轮新子作用域 → **每轮独立 cell**。
  - **调用**：用户函数（新帧挂在 `globals` 下 + 参数绑定 + 返回）、内置（ABI `fn(&[Value], Span) -> R<Value>`，**`Span` 取调用点**）、方法（`recv.m()` 绑定 `self`）。
  - **错误**：`R<T>` 冒泡，位置取节点 / 调用点 `Span`（新增测试锁定非 `Span::START`）。
  - **名字解析 = 运行时查名**（见 ADR）：`env.rs` 增 per-scope 名字表（`define_named`/`local_index`/`get_local`/`set_local`/`capture_local`/`named_indices`），查名顺序 调用帧/块 → 捕获 cell → globals。
  - **跨模块扩展**：`value.rs` `Closure` 增 `user: Option<Rc<UserFn>>`、新增 `UserFn`、`StructDef` 扩为 `{name, fields, methods}` + `Value::struct_template`；`Closure::named/anonymous`、`Value::struct_def` 签名不变。
- 产出:
  - `Get-ChildItem src\evaluator.rs` → **`evaluator.rs  79980`**（原 182 B；合法 UTF-8）。
  - `cargo build --message-format=json 2>$null | Select-String '"level":"warning"'` → **0 warning**。
  - `cargo test` → **`195 passed; 0 failed; 0 ignored`**（新增 `evaluator::tests::*` **26** 个，全绿）。
  - 新增测试名（26）：arithmetic_precedence_and_string_concat / int_overflow_and_division_by_zero / modulo_python_semantics / comparisons_and_nan_ieee / equality_is_deep_and_cycle_safe / logical_short_circuits / if_expression_yields_branch_value / while_loop_accumulates / for_loop_with_break_and_continue / for_over_struct_iterates_keys_sorted / factorial_recursion / closure_counter_shares_cell / loop_iterations_get_independent_cells / user_fn_arity_mismatch_is_type_error / array_index_read_and_negative / a1_array_write_is_visible_through_alias / a1_struct_field_write_through_alias_and_dynamic_add / nested_index_write_through_path / struct_template_method_binds_self / struct_template_defaults_are_reevaluated_per_instance / builtins_len_push_str_range / builtin_call_span_is_call_site_and_argcount_checks / interpolation_plain_and_format_spec / function_body_block_value_is_last_expr / return_propagates_out_of_loops_and_branches / runtime_errors_carry_node_span。
- 决策: 追加 ADR [2026-09-24 01:30]（运行时查名策略 + A2 捕获 + `Env`/`value.rs` 扩展 + 4 项规范缺口）。**未**改 `docs/spec/`、`error.rs`、`ast.rs`、`parser.rs`、`span.rs`、`lexer.rs`、`loader.rs`、`Cargo.toml`。
- 缺口（上报，不自行发明）: (1) `let` 重绑定的错误类 §8.1 未定义 → 本批**存储** `mutable` 但**暂不强制**；(2) `;;`（`Dump`）输出、`ScopeDebug`/`def_scope` 可见链 → P3.8；(3) `RecursionError`（帧深上限）与 `TraceFrame`/traceback 组装 → P3.8（当前无限递归无保护）；(4) §4.5.5 深结构 10000 层上限仍未施加（承 P3.6b 遗留）。
- 下一步: P3.8（`Dump`/`;;`、`check` 专项、`RecursionError`、traceback、`let` 不可变性）；P3.9b（HOF 需将 `call_user` 提升为可复用 ABI）。
- 阻塞: 无。
## [2026-09-24 00:40] P3.6b — `src/value.rs` 值语义辅助（A6 环安全深相等 + §4.5.6 全序）唯一共享实现
- 来源: team-lead 任务书「P3.6b — 值语义辅助落 `src/value.rs`」，依据 `docs/spec/semantics.md` §4.5.6（全序）/§4.5.7（精确比较）/§4.5.9（A5/A6）/§3.7，`interface-contract.md` §10.7。
- 完成:
  - `src/value.rs` 新增**唯一**公开 ABI（P3.7/P3.9b 消费）:
    - `pub fn Value::deep_eq(&self, other: &Value) -> bool` —— A6：`Rc::ptr_eq` 身份优先 → 标量（§4.2/§4.5.6/§4.5.7）/ 容器结构比较；容器维护「已访问有序对集合」，重访 ⇒ 相等（环安全）；struct 忽略函数值字段（A5）、键集字节序、键序无关。
    - `pub fn Value::total_cmp(&self, other: &Value) -> Option<Ordering>` —— §4.5.6 全序：`-Inf<有限<+Inf<NaN`；int/float 混合按 §4.5.7 精确；仅同类别可比，否则 `None`。
    - `pub fn Value::order_kind(&self) -> Option<OrderKind>` + `pub enum OrderKind { Num, Str }`。
    - `pub(crate) const TWO_POW_63: f64`（与 `builtins` 共用）。
  - 私有内核：`eq_rec`（有序对集合 `Vec<(usize,usize)>`，`Rc::as_ptr` 作身份）、`num_eq_int_float`、`cmp_int_float`、`num_order`。
  - `src/builtins.rs` 改造：**删除**内联比较器 `Kind` / `order_kind` / `total_cmp_ok` / `num_order` / `cmp_int_float` + 本地 `TWO_POW_63`；`sort`/`min`/`max` 改调 `Value::total_cmp`，`validate_orderable` 改调 `Value::order_kind`（消除两份口径漂移）。
- 产出:
  - `git diff --stat -- src/value.rs src/builtins.rs` → `2 files changed, 462 insertions(+), 100 deletions(-)`（`value.rs 445+/1-`、`builtins.rs 17+/99-`）。
  - `cargo build --tests --message-format=json 2>$null` → `warnings=0 errors=0`。
  - `cargo test` → **`169 passed; 0 failed`**（新增 value 单测 **15**：`deep_eq_*` 10 + `total_cmp_*` 4 + `order_kind_classification` 1）。
  - 既有 `builtins::tests::sort_is_stable` / `sort_stable_total_order_and_type_error` / `min_max_total_order_and_empty` 仍全绿（行为无冲突）。
- 决策: 追加 ADR [2026-09-24 00:40]（共享 ABI + `StructDef` 相等保守口径 + §4.5.5 深度上限遗留项），供 P3.7/P3.9b/architect 对齐。**未**改 `docs/spec/`、`error.rs`、`env.rs`、`Cargo.toml`。
- 缺口（上报，不自行发明）: (1) `StructDef`（模板）的 `==` 规范未定义——暂取**同一性**；(2) §4.5.5 深结构 10000 层上限（`RecursionError`）在 `deep_eq`/`Display` 中**未**实现（递归实现，无显式迭代栈/`Result`），建议随 P3.7 收口；(3) `< <= > >=` 的 IEEE `NaN→false` 语义**不**由 `total_cmp` 承担，P3.7 须自行处理。
- 下一步: P3.7 求值器消费 `deep_eq`/`total_cmp`；P3.9b HOF 复用 `total_cmp`。
- 阻塞: 无。

## [2026-09-24] P3.9a 契约缺口闭合 — `src/builtins.rs`（清单 3/4/5 落地）
- 来源: team-lead 任务书「改造 `src/builtins.rs`（P3.9a 缺口的代码落地，清单 3/4/5）」，依据 ADR [2026-09-24 00:20] language-architect「P3.9a 契约缺口闭合（6 项）」。
- 完成（逐条）:
  1. `min`/`max` 空数组 → `ValueMsg::EmptyExtremum { func }`（不再 `Convert` 兜底）：`empty_collection_value_error`（L1079）改用新变体；消息 `空数组没有极值（min）` / `（max）`。
  2. `randInt(lo>=hi)` → `ValueMsg::BadRange { lo, hi }`：`b_rand_int`（L884）；消息 `区间非法：5 >= 5`。
  3. `del(k,s)` 仅作用于数据字段：`b_del`（L686）判定由 `raw_fields`「存在即删」改为「数据字段集合」；新增共用谓词 `is_data_field`（L634），`b_has`（L675）复用之；方法字段 → `FieldError`（`结构体没有字段 'm'`）。不变量 `del(k,s) 成功 ⟺ has(k,s)` 由新测 `del_succeeds_iff_has_is_true` 锁定。
  4. 确认对齐（**无代码变更**）: `pop([])` → `Index{-1,0}`（`b_pop` L487）；`insert` 不支持负索引、越界 → `Index{i,len}`（`b_insert` L509）；`floor`/`ceil`/`round` 复用 `int(float)` 口径（`float_to_int` L305：NaN→`Convert{float,int,nan}`、±Inf/超界→`Overflow`）——三者均已满足，仅补断言。
  5. `minBy`/`maxBy`：HOF，属 P3.9b；代码里**无占位实现**（仅文档 TODO），无需改。
- 产出:
  - `git diff --stat -- src/builtins.rs` → `1 file changed, 55 insertions(+), 28 deletions(-)`。
  - `cargo build --tests --message-format=json 2>$null` → `warnings=0 errors=0`。
  - `cargo test` → `135 passed; 0 failed`（builtins 模块 **36** 个测试全绿：新增 `del_succeeds_iff_has_is_true` + 改动 `min_max_total_order_and_empty` / `rand_seed_deterministic_and_rand_int` / `del_returns_new_struct_and_missing_field` / `insert_bounds_and_new_array` / `floor_ceil_round_widen_and_banker`）。
- 决策: 无新 ADR（纯代码落地，遵循既有 ADR 裁定；未新增/改 spec、未改 `error.rs`）。
- 下一步: P3.7 求值器 → P3.9b（7 个高阶内置；`minBy`/`maxBy` 空数组复用 `EmptyExtremum`）。
- 阻塞: 无。（注：本轮 `cargo test` 期间 core-dev 的 `src/lexer.rs` 正在并发编辑，曾出现 2 次瞬时 lexer 测试失败；core-dev 改动稳定后全绿，与本轮 `builtins.rs` 无关。）

## [2026-09-23] P3.9a — `src/builtins.rs`（内置函数第一批：全部非高阶内置）
- 来源: team-lead 任务书「P3.9a — `src/builtins.rs`（内置函数第一批）」，契约 §10.7 全表 + §8.1/§10.8，semantics §3.7/§4.2/§4.5.6/§4.5.7/§4.5.10/§8.1。
- 完成:
  - 稳定 ABI：`BuiltinFn = fn(&[Value], Span) -> R<Value>`、`Builtin{name,min_args,max_args,func}` + `Builtin::call`（集中校参数个数）、`lookup`/`call`/`is_builtin`/`BUILTIN_NAMES`、`static TABLE: [Builtin;47]`（表与名单一致性由测试锁定）。
  - **47 个非高阶内置**全部实现（核心/数组 14、struct 5、字符串 8、数学/随机/转换/IO/断言 20）；**7 个高阶内置**（map/filter/reduce/sortBy/minBy/maxBy/each）留 P3.9b（`// TODO(P3.9b)`）。
  - 规范要点：A1「返回新值、原容器不变」（含 Rc 不同一断言）；A5/B3 数据面 + 字节序升序；§10.7 加宽补钉（`floor/ceil/round/sqrt/pow` 经 `as_f64`，`abs` 同型不加宽）；`int()` 极窄边界（NaN→ValueError / ±Inf→Overflow / 截断超界→Overflow）；银行家舍入 `round`；`div` 向下取整 + `i64::MIN/-1` 溢出；§4.5.6 全序（NaN 最后，int/float 数学精确比较）+ 稳定 `sort`；自实现 `splitmix64` 随机（无第三方依赖）；`check` 非致命 / `assert`·`fail` 致命；错误均携带调用点 `Span`。
- 产出:
  - `src/builtins.rs`（**72102 B / 1549 行**，含单测）。
  - `cargo build --tests --message-format=json 2>$null` → `warnings=0 errors=0`。
  - `cargo test` → **`82 passed; 0 failed`**（P3.6 基线 47 + 本批新增 **35** 个单测）。
- 决策: ABI 由「建议」的 `fn(&[Value])->R<Value>` **扩展为携带调用点 `Span`**（满足「运行时错误必须带位置」红线）；已追加 ADR（跨 P3.7 求值器接口）。
- 缺口（非阻塞，已上报）: `min`/`max` 空数组与 `randInt(lo>=hi)` 的 `ValueError` 无消息模板；`pop([])` 的 `IndexError` 下标未规定；`floor/ceil/round` 对 NaN/Inf/超界的返回未规定；`del` 对方法字段的判定未规定。
- 下一步: 等 P3.7 求值器就绪 → P3.9b（7 个高阶内置）。
- 阻塞: 无。

## [2026-09-23] P3.6 — `src/value.rs` + `src/env.rs`（值模型 + 作用域/cell/ScopeDebug）
- 来源: team-lead 任务书「P3.6 — value.rs + env.rs」，契约 §10.5 + semantics §3.6/§3.7/§4.5.2(A1)/§4.5.3(A2)/§4.5.7。
- 完成:
  - `value.rs`：`enum Value`（八类 + `StructDef` 模板）、`type_name()`、构造便捷函数、`as_f64()` 唯一加宽入口、严格访问器、§3.7 `Display`（含环安全 `<cycle>`、struct 数据面排序/跳方法字段）、`StructObj`。
  - `env.rs`：`Env` 作用域链 + 绝对槽位 + A2 `capture` 原地升级 cell（counter 语义、循环迭代各自独立 cell）；`ScopeDebug`/`ScopeId`/`FuncNameId`/`NameInterner`/`ScopeDebugTable`（内→外 + slot 升序 + 遮蔽去重的 `visible_slots()`）/`ScopeChain`。
- 产出:
  - `src/value.rs`（22760 B）、`src/env.rs`（17621 B）。
  - `cargo build --tests --message-format=json 2>$null` → `warnings=0 errors=0`。
  - `cargo test` → `47 passed; 0 failed`（P3.6 新增 20 个单测；`Value` 16 B 已锁定）。
- 决策: §10.5 的具体类型落地（`ScopeId`/`FuncNameId`/`ScopeChain` 置于 `env.rs`）；`Value` 增 `StructDef` 变体承载 struct 模板。已追加 ADR 供 core-dev 同步（不改 spec）。
- 下一步: 等 team-lead 派 P3.7 `evaluator.rs` 核心。
- 阻塞: 无。

## [2026-09-22] 角色初始化
- 来源: 团队初始化（系统构建）
- 完成: 角色定义与活文档建立
- 产出: `.opencode/agents/runtime-dev.md`
- 下一步: 等待 team-lead 调度
