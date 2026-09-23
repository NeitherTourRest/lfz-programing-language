# runtime-dev — 工作状态
> 最后更新: 2026-09-24 02:30 by runtime-dev

## 当前状态
**P3.8 ✅ 完成**（`src/evaluator.rs` 语义定稿：§4.5 确定性八项 / A4 / A5 / A6 / `RecursionError` / `;;` / traceback）。
待 team-lead 核验 + release-manager 提交 `feat(p3): evaluator determinism, recursion guard, dump, traceback`。

- **公开 ABI**（tooling-dev 消费；旧 ABI 不变，本批新增 2 项）：
  - `pub fn eval_module(&Program) -> R<Value>`（**不变**）—— 返回最后一条语句值。
  - `pub fn run(&Program) -> R<()>`（**不变**）。
  - **新增** `pub fn eval_module_traced(&Program) -> TracedRun` —— 带 traceback 帧栈求值（§10.3）。
    - `TracedRun { pub result: R<Value>, pub frames: Vec<TraceFrame>, .. }`；`frames` 自**最外层 → 最内层**；`traced.frame_name(&frame) -> Rc<str>` 解析 `<module>` / 函数名 / `<fn>`（§8.4）。
    - **建议 CLI 组装 traceback 改用此入口**（`LzError` 无帧栈字段，见 ADR / 缺口 1）。
  - **新增** `pub fn render_dump(entries: &[(Rc<str>, Value)]) -> String` —— `;;` 渲染**纯函数**（§3.6；行格式 `<name> ： <value>`）。
  - **新增** `pub const RECURSION_LIMIT: u32 = 10_000;`。
- **跨模块新增**（本批）：
  - `value.rs`：`pub fn Value::deep_eq_bounded(&self, other: &Value, limit: u32) -> Result<bool, u32>`（A6 带上限；超限 `Err(实际深度)`）；`deep_eq` 签名/行为**不变**（内部改走 bounded，`limit = u32::MAX`）。
  - `env.rs`：`pub fn Env::bindings(&self) -> Vec<(String, usize)>`（**声明序 = slot 升序**，供 `;;`）。
- 既有基线：P3.7 `evaluator.rs` 核心、P3.6b `value.rs` 值语义、P3.6a value/env、P3.9a builtins（47 非高阶内置）。

## 进行中
- （无）

## 最近完成
- **P3.8 `src/evaluator.rs` 语义定稿**（2026-09-24 02:30）
  - §4.5 八项：B1（求值序，含赋值 key→rhs）、B2（`for` 快照）、B3（键 UTF-8 字节序）、B4（`RecursionError`：调用帧 10000 + 深结构 10000）、B6（IEEE / 除零不产 Inf）、B7（平拷贝）、B13（`as_f64` 唯一加宽 + 精确比较）。
  - A4：`check` 非致命（builtins 已实现：stderr 一行 + `false` + 继续）；`assert` / `fail` 致命（消息由构造方组装）。
  - A5/A6：struct 数据面排除方法字段；`==` / `!=` 复用 `deep_eq_bounded`（环安全 + 身份优先 + 精确比较）。
  - `;;`（Dump）：沿可见链**内→外**、同层 **slot 升序**、**遮蔽去重**；纯函数 `render_dump`；写 **stdout**。
  - traceback：`TracedRun` 按 §10.3 N1 组装帧栈（进入 Call **先更新当前帧 span 为调用点，再压新帧**）。
  - **求值线程**：`eval_module_traced` 在 **256 MiB 大栈线程**上求值（树遍历器 10000 层递归远超主线程默认栈；实测 8 MiB / 64 MiB 均栈溢出），结果经 `Transfer`（`unsafe impl Send`，`join` happens-before 保证无并发）跨线程移交。详见 ADR。
  - 证据：
    - `Get-ChildItem src\evaluator.rs,src\env.rs,src\value.rs` → evaluator.rs **114981 B**、env.rs **22303 B**、value.rs **45986 B**（均合法 UTF-8）。
    - `cargo build --tests --message-format=json 2>$null` → **warnings = 0**。
    - `cargo test` → **`226 passed; 0 failed; 0 ignored`**（新增 `evaluator::tests::*` **15**）。
    - 新增测试（15）：b1_evaluation_order_is_left_to_right / b2_for_iterates_over_snapshot / b3_struct_keys_are_byte_order_sorted / b4_deep_recursion_yields_recursion_error / b6_float_ieee_rules / b7_struct_instantiation_is_flat_copy / b13_int_float_widening_is_the_only_implicit_conversion / a4_check_nonfatal_assert_and_fail_fatal / a5_struct_data_plane_excludes_method_fields / a6_equality_is_cycle_safe_and_identity_first / dump_render_is_pure_and_formats_lines / dump_visible_entries_inner_to_outer_shadow_dedup / dump_statement_runs_and_returns_nil / traceback_frames_follow_call_site_rule / end_to_end_lex_parse_eval_smoke。

## 阻塞 / 需要支持
- **无硬阻塞**。
- **规范 / 契约缺口（已上报，未自行发明；本实现取保守 / 适配口径）**：
  1. **`LzError`（§10.4）无 traceback 字段**：§10.3 要求「出错时把帧栈序列化进 `LzError`」，但 `error.rs`（只读）的 12 变体无此字段。→ 新增 `eval_module_traced` + `TracedRun.frames` 带出帧栈；**请 language-architect 确认此 ABI**（若坚持写入 `LzError`，需 core-dev 改 `error.rs`）。
  2. **`let` 重绑定的错误类未定义**（§4.5.2 称「非法」、§8.1 无错误类）：仍**存储** `mutable` 标志但**不强制**（承 P3.7）。
  3. **`Display` / `fmt` 的深结构 10000 层上限无法产 `RecursionError`**（`fmt::Result` 不能返回 `LzError`）；`==` 路径已收口（`deep_eq_bounded`），`Display` / `str` 仍为无上限递归（仅 > 10000 层嵌套时理论风险）。
  4. **`struct` 模板（`StructDef`）的 `==`** 仍取**同一性**（承 P3.6b）。
- **给 tooling-dev 的建议**：CLI 错误格式化请用 `eval_module_traced` 取帧栈；位置用 `Span`（`line/col`，非字节偏移）。

## 下一步计划
- **待 team-lead 核验 P3.8**；若需 `LzError` 承载帧栈，等 architect 裁定 + core-dev 改 `error.rs`。
- **P3.9b**：7 个高阶内置（`map` / `filter` / `reduce` / `sortBy` / `minBy` / `maxBy` / `each`）——需将 `Interp::call_user` 提升为可复用 ABI。
- `let` 不可变性：待 architect 指定错误类。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **⚠️⚠️ 血的教训（本批踩中）**：**绝不用 PowerShell `Get-Content` / `Set-Content` / `-replace` 改 `src/*.rs`**。PS 5.1 默认按 ANSI/GBK 读 UTF-8，会把中文**双重编码**（mojibake）并**吞并换行**，导致源码结构性损坏（`///` 与后续 `pub` 挤到同一物理行 → 整个 item 被注释）。改文件**只用 `edit` / `write` 工具**。本次损坏的 `src/evaluator.rs` 经 `git checkout HEAD -- src/evaluator.rs` 恢复（HEAD 恰为 P3.7 基线 79980 B）后重做本批编辑。
- **warning/error 计数**：`$out = cargo build --tests --message-format=json 2>$null; (($out | Select-String '"level":"warning"')).Count`（`2>$null` 丢进度；**勿** `2>&1`）。
- **深递归栈**：debug 构建下 10000 层递归在 8 MiB / 64 MiB 栈均溢出，**256 MiB** 才够 → `eval_module_traced` 起专用大栈线程。
- **`RefCell` 陷阱**：`if let Some(i) = env.borrow().local_index(name) { env.borrow_mut()... }` 会 panic（scrutinee 借用存活到块尾）。先 `let idx = ...;`。
- **A2 递归自引用**：命名 `fn` 必须「先 `nil` 预绑定 → 建闭包（捕获自身名 cell）→ 写回闭包」。
- **运行时常量 `Span::START`**：错误构造函数别写死 `Span::START`——违反「运行时错误必须带位置」红线；helper 须显式接收节点 `span`。
- **AST 类型名是 `Program`**；`Expr` / `Stmt` = `Spanned<ExprKind / StmtKind>`；`Dump { scope: ScopeId }`。
- **并发编辑**：`cargo test` 可能捕捉到 core-dev 半成品（`parser.rs` / `lexer.rs`）；判据：只看自己模块测试 + 全绿时全量。
