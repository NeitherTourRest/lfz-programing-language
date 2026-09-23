# runtime-dev — 工作状态
> 最后更新: 2026-09-23 by runtime-dev

## 当前状态
**P3.6（`src/value.rs` + `src/env.rs`：运行时值模型 + 作用域/cell/`ScopeDebug`）✅ 完成，待 team-lead 核验 + release-manager 提交 `feat(p3): value and env`。**

## 进行中
- （无）

## 最近完成
- **P3.6 值模型与作用域**（2026-09-23）
  - `src/value.rs`（22760 B / 472 行）
    - `pub enum Value`：`Nil` / `Bool` / `Int(i64)` / `Float(f64)` / `Str(Rc<String>)` /
      `Array(Rc<RefCell<Vec<Value>>>)` / `Struct(Rc<RefCell<StructObj>>)` / `StructDef(Rc<StructDef>)` /
      `Func(Rc<Closure>)`。覆盖 `type()` 八类；`StructDef` 为 §3.7/§4.5.0 的 struct **模板**（`type` 亦报 `"struct"`、显示 `<struct 名字>`）。
    - **A1 引用语义**：`array` / `struct` 用 `Rc<RefCell<…>>`；`Clone` 共享同一分配，原地修改对所有引用可见。
    - `type_name()`（八类，§10.7 `type` 所需）、构造便捷函数、`as_f64()`（**唯一** int→float 加宽入口，§4.5.7）、严格访问器（`as_int`/`as_float`/`as_bool`/`as_str`/`as_array`/`as_struct`/`as_closure`）。
    - `Display` 按 §3.7：标量 / `float` 最短往返含 `.0` 与 `nan`/`inf`；**顶层字符串裸输出、嵌套加引号+转义**；array `[e1, e2]`；struct 键按**字节序升序**且**跳过方法字段**（A5/B3）；`<fn 名字>`/`<fn>`/`<struct 名字>`；**环安全**（路径集合 → `<cycle>`）。
    - `StructObj`：插入序存储 + `data_fields_sorted()`（过滤函数值字段、字节序升序）/ `data_len()` / `get`/`set`（动态加字段，§4.5.8）。
  - `src/env.rs`（17621 B / 370 行）
    - **运行时 `Env`**：作用域链 + 绝对槽位（`first_slot + 偏移`）+ `get`/`set`/`capture` 沿链内→外。
    - **A2**：`Slot::Direct` → 被捕获时**原地升级**为 `Cell = Rc<RefCell<Value>>`；再次 `capture` 返回同一 cell；循环体每次迭代 `child_after` 新作用域 → **各自独立 cell**。
    - **调试侧**：`ScopeDebug { first_slot, names: Vec<FuncNameId>, parent: Option<ScopeId> }`（照 §10.5）、`ScopeId`、`FuncNameId`、`NameInterner`（interned 名字表）、`ScopeDebugTable`（arena + `visible_slots()` 内→外 / slot 升序 / **遮蔽去重**枚举器）、`ScopeChain`（`Rc` 共享调试表 + 定义处 `ScopeId`，由 `Closure.def_scope` 携带）。
    - **零热路径开销**：`Env` 读写路径不触碰调试信息；`ScopeDebug` 链仅 `;;` 时读取。
  - 证据
    - `cargo build --tests --message-format=json` → JSON `warnings=0`、`errors=0`。
    - `cargo test` → lib harness `47 passed; 0 failed`（另 2 harness 各 0）。P3.6 新增 **20** 个单测：`value::tests::{value_is_two_words, type_name_covers_all_eight_classes, a1_array_reference_is_shared_between_bindings, a1_struct_field_write_is_visible_through_aliases, display_scalar_forms, display_string_top_raw_nested_quoted, display_array_elements_comma_space, display_struct_sorts_keys_and_skips_methods, display_cycle_is_safe, display_function_and_struct_template, as_f64_is_the_only_widening_entry, accessors_are_strict_and_do_not_convert, struct_data_plane_helpers}` + `env::tests::{a2_captured_local_upgrades_to_shared_cell, a2_loop_iterations_get_independent_cells, env_get_set_walks_scope_chain, scope_debug_inner_shadows_outer, scope_debug_empty_chain_yields_nothing, name_interner_is_stable_and_resolvable, closure_carries_def_scope_chain}`。
    - `Value` 大小经 `value_is_two_words` 锁定 = **16 B**（2 机器字，架构目标）。

## 阻塞 / 需要支持
- （无硬阻塞）两处**契约落地**（非新增语义，仅把 §10.5 未给出的具体类型定死；已写 ADR 供 core-dev 同步）：
  1. §10.5 只给出 `ScopeDebug` 字段，未定义 `ScopeId` / `FuncNameId`。**落地**：`ScopeId(pub u32)`、`FuncNameId = u32`、`NameInterner`、`ScopeDebugTable`、`ScopeChain` 均在 `src/env.rs`。**core-dev 的 `ast::Dump { scope }` 请用 `crate::env::ScopeId`**（需要时）。
  2. `Value` 增 `StructDef` 变体承载 struct **模板**（§3.7 明确要求其显示 `<struct 名字>`、§4.5.0 明确 `type` 报 `"struct"`）——非新增语义，属值模型必备。

## 下一步计划
- 等待 team-lead 派发 **P3.7 `evaluator.rs` 核心**（表达式/语句/控制流/函数与闭包/管道调用），依赖 P3.6 ✅ 与 core-dev AST（P3.4/P3.5）。
- P3.7 将在 `Closure` 上扩展形参 / 函数体 AST / 捕获 cell 列表；`Env` 在其上构建调用帧。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- PowerShell 里 `cargo ... 2>&1` 会把 cargo 的 stderr 进度行包成 `NativeCommandError`（误判为 error）。**看 warning 的正确姿势**：`cargo build --tests --message-format=json 2>$null`，再统计 `"level":"warning"`；`2>$null` 丢弃进度、JSON 诊断走 stdout。
- `RefCell` 升级 cell 的借用陷阱：先 `match &self.slots[idx] { Shared → return clone; Direct(v) => v.clone() }` 取出**克隆值**再 `self.slots[idx] = Shared(…)`（NLL 下克隆结束不可变借用后才可写）。
- `Rc<str>: Borrow<str>` 让 `HashMap<Rc<str>, _>::get(&str)` 直接可用；`Rc` 的 `Hash`/`Eq` 按内容。
- `Value` 目前 16 B：`Str` 用 `Rc<String>`（细指针）而非 `Rc<str>`（胖指针）——否则涨到 24 B。
- `cargo test` 会把 core-dev 的 `loader.rs`（当前工作区为未提交 WIP）一并编译；联调时以其 STATUS 为准，勿改其文件。
