# runtime-dev — 工作状态
> 最后更新: 2026-09-24 04:00 by runtime-dev

## 当前状态
**P3.9b ✅ 完成**（`src/builtins.rs`：7 个高阶内置 `map` / `filter` / `reduce` / `sortBy` / `minBy` / `maxBy` / `each`）。
至此 **§10.7 全表 54/54 齐备**（P3.9a 47 非高阶 + 本批 7 高阶）。
待 team-lead 核验；建议 release-manager 提交 `feat(p3): higher-order builtins (map/filter/reduce/sortBy/minBy/maxBy/each)`。

- **公开 ABI 新增（builtins.rs）**：
  - `pub type Invoke<'a> = &'a mut dyn FnMut(&Value, &[Value], Span) -> R<Value>;` —— 高阶内置的**调用能力注入**接口（由求值器提供）。
  - `pub type HofFn = fn(&[Value], Span, Invoke<'_>) -> R<Value>;`
  - `pub struct Hof { name, min_args, max_args }` + `impl Hof::call(self, args, span, invoke)`。
  - `pub fn lookup_hof(name) -> Option<Hof>`；`pub const HOF_NAMES: &[&str]`（7）。
  - `pub fn call_with(name, args, span, invoke) -> R<Value>` —— **求值器唯一内置入口**（高阶表优先，否则回落 `call`）。
  - `pub fn is_builtin(name)` 改为两表并集（54）；`BUILTIN_NAMES` 由 47 → **54**（字母序）。
  - `lookup` / `call`（非高阶 47）**签名与语义完全不变**（求值器改调 `call_with`，对 47 个行为不变）。
- **跨模块新增（evaluator.rs）**：
  - `Interp::call_value(&mut self, callee: &Value, args: &[Value], span) -> R<Value>`：HOF 回调注入点（非函数 → `NotCallable`；函数 → `call_user`）。
  - `eval_call` 内置派发改为 `call_with(... &mut invoke)`（`invoke` = 就地闭包 `|f,a,s| self.call_value(f,a,s)`）。
- 既有基线：P3.8 evaluator 语义定稿、P3.7 evaluator 核心、P3.6b value、P3.9a builtins（47 非高阶）。

## 进行中
- （无）

## 最近完成
- **P3.9b 7 个高阶内置**（2026-09-24 04:00）
  - 语义（§10.7 / §8.1 / §4.5.6）：data-last；容器更新返回**新值**（A1）；回调先校验为函数（否则 `NotCallable`，空容器亦先报错）；容器非 array → `TypeError`。
  - `map` 逐元素；`filter` 谓词**须 `bool`** 否则 `TypeError::ConditionNotBool`；`each` 仅副作用、返回 `nil`。
  - `reduce` **左折叠** `f(acc,x)`，空数组 → 返回 `init`（不调用 `f`）。
  - `sortBy` 按 `keyFn` 结果**升序稳定**（`-0.0`/`0.0` 等键可观测）；键须同类可全序，否则 `TypeError`。
  - `minBy`/`maxBy` 空 → `ValueError`「空数组没有极值（{func}）」；非空返回原元素（等键取首）。
  - 全部错误携带**调用点 span**。
  - 证据：
    - `Get-ChildItem src\builtins.rs,src\evaluator.rs` → builtins.rs **90390 B**、evaluator.rs **119972 B**（均合法 UTF-8）。
    - `cargo build` → `Finished`（**0 warning**）；`cargo build --tests --message-format=json 2>$null` → **warnings=0 errors=0**。
    - `cargo test` → **`256 passed; 0 failed; 0 ignored`**（基线 246 + 净新增 10）。
    - 新增测试（11，替换 1 条过时断言）：builtins —— `hof_map_returns_new_array_and_leaves_original` / `hof_filter_requires_bool_predicate` / `hof_reduce_folds_left_to_right` / `hof_sort_by_is_stable_and_uses_key` / `hof_min_by_max_by_and_empty_value_error` / `hof_each_visits_all_and_returns_nil` / `hof_callback_type_and_arg_count_checks` / `hof_names_are_registered`；evaluator —— `e2e_hof_map_routes_and_rejects_non_function_callback` / `e2e_hof_data_last_and_arg_count_checks` / `hof_drives_user_closures_through_evaluator`。
    - 单测**不依赖 evaluator**（传 stub 回调）；另含 2 条 `lex+parse+eval` + 1 条程序化 AST 闭包全链路。
  - 决策：追加 ADR `[2026-09-24 04:00] P3.9b 高阶内置：调用能力注入（Invoke ABI）+ 54/54 齐备`（含「为何 `&mut dyn FnMut` 而非 `&dyn Fn`」）。

## 阻塞 / 需要支持
- **无硬阻塞**。
- **契约 / 环境缺口（已上报，未自行发明）**：
  1. **parser 尚不支持 lambda**（core-dev 并行实现中）：`map` + 闭包的 **lex+parse+eval 成功路径**暂不可达。本批以 2 条 `lex+parse+eval` 用例（路由 / data-last / 参数个数 / 回调类型）+ 1 条**程序化 AST** 闭包用例覆盖；**待 lambda 落地后可改回 lex+parse+eval**（HOF 实现无需改动）。
  2. **HOF 回调类型先校验口径**（空容器也对非法 `f` 报 `TypeError`）与 **`filter` 非 bool → `ConditionNotBool`**：请 language-architect 确认（合同 §10.7 概括为「参数类型不符 → TypeError」，具体消息未逐字指定）。
  3. **v1 内置函数不是一等值**（`eval_expr` 对内置名 `Ident` 报 `NameError`）→ `map(len, xs)` 不可写；HOF 回调须为**用户 lambda / 命名函数**（与 spec §9 样例一致）。若语言要求内置可作回调，需 architect 裁定 + core-dev 扩展（非本模块）。
  4. 承 P3.8 的 4 项缺口（`LzError` 无 traceback 字段 / `let` 重绑定错误类 / `Display` 深结构上限 / `StructDef` 模板 `==`）**仍待 architect**。

## 下一步计划
- 待 team-lead 核验 P3.9b；待 parser lambda 落地后补 `map`/`sortBy` 的 **lex+parse+eval 成功路径**用例。
- 若有新的 §10 契约补钉（如 `each` 返回值 / 空容器口径）按 architect 裁定调整。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **⚠️ 改 `src/*.rs` 只用 `edit` / `write` 工具**：PowerShell `Get-Content`/`Set-Content` 会按 ANSI/GBK 读写 UTF-8，双重编码 + 吞换行，结构损坏（承 P3.8 教训）。本批已严格遵守。
- **warning/error 计数**：`$out = cargo build --tests --message-format=json 2>$null; (($out | Select-String '"level":"warning"')).Count`（勿 `2>&1`）。
- **高阶内置 ABI（本批定型）**：`Invoke = &mut dyn FnMut(&Value, &[Value], Span) -> R<Value>`；HOF 是 `HofFn`；派发走 `call_with`。**不要**把 HOF 塞进 `TABLE`（其 `BuiltinFn` 无 `&mut` 调用能力）。
- **调用能力必须 `&mut`**：`call_user` 会改 `depth` / `trace`，故用 `&mut dyn FnMut`（`&dyn Fn` 不可行）；求值器侧 `call_value` 是唯一注入点，非函数错误口径 = `NotCallable`。
- **`RefCell` 陷阱**（承 P3.7）：`if let Some(i) = env.borrow().local_index(name) { env.borrow_mut()... }` 会 panic；先 `let idx = ...;`。
- **并发编辑**：`cargo test` 可能捕捉到 core-dev 半成品（`parser.rs` / `lexer.rs`）；判据：只看自己模块测试 + 全绿时全量。本批 `parser.rs` 无 lambda → e2e 成功路径受限（见上）。
