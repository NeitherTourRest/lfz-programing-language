# runtime-dev — 工作状态
> 最后更新: 2026-09-24 00:40 by runtime-dev

## 当前状态
**P3.6b ✅ 完成**（`src/value.rs` 值语义辅助：A6 环安全深相等 + §4.5.6 全序，**唯一共享实现**；`builtins.rs` 复用、删除内联比较器）。
待 team-lead 核验 + release-manager 提交 `refactor(p3): shared value semantics (A6 ==, total order)`。
- 公开 ABI（P3.7/P3.9b 消费）：
  - `pub fn Value::deep_eq(&self, other: &Value) -> bool`
  - `pub fn Value::total_cmp(&self, other: &Value) -> Option<std::cmp::Ordering>`
  - `pub fn Value::order_kind(&self) -> Option<OrderKind>`（+ `pub enum OrderKind { Num, Str }`）
  - `pub(crate) const TWO_POW_63: f64`
- 既有基线：P3.6a `value.rs`/`env.rs`、P3.9a `builtins.rs`（47 非高阶内置）+ 契约缺口闭合均完成。

## 进行中
- （无）

## 最近完成
- **P3.6b `src/value.rs` 值语义辅助（A6 + 全序）唯一共享实现**（2026-09-24 00:40）
  - `src/value.rs`（本轮 **445+/1-**）：新增 `deep_eq`（A6）、`total_cmp`（§4.5.6）、`order_kind`(+`OrderKind`)、`pub(crate) TWO_POW_63`；私有 `eq_rec`/`num_eq_int_float`/`cmp_int_float`/`num_order`。
  - `src/builtins.rs`（本轮 **17+/99-**）：**删除**内联比较器 `Kind`/`order_kind`/`total_cmp_ok`/`num_order`/`cmp_int_float` + 本地 `TWO_POW_63`；`sort`/`min`/`max` 改调 `Value::total_cmp`，`validate_orderable` 改调 `Value::order_kind`。
  - **A6 语义（§4.5.9）**：身份优先（`Rc::ptr_eq` 短路，含自引用容器）→ 标量按 §4.2/§4.5.6/§4.5.7（`NaN != NaN`、`+0.0 == -0.0`、int/float 混合按数学精确值、string 按内容、function 仅同一性）；容器维护「**已访问有序对集合**」`Vec<(usize,usize)>`（`Rc::as_ptr` 身份），**重访一对 ⇒ 视为相等**（不报错、不死循环）；struct 比较**忽略函数值字段**（A5）、键集按 UTF-8 字节序、**键序无关**。
  - **全序（§4.5.6/§4.5.7）**：`-Inf < 有限 < +Inf < NaN`；int/float 混合**不先加宽**、数学精确；**仅同类别**（数值组 / 字符串组）可比较，否则 `None`。
  - 证据：
    - `git diff --stat -- src/value.rs src/builtins.rs` → `2 files changed, 462 insertions(+), 100 deletions(-)`（`value.rs 445+/1-`、`builtins.rs 17+/99-`）。
    - `cargo build --tests --message-format=json 2>$null` → `warnings=0 errors=0`。
    - `cargo test` → **`169 passed; 0 failed`**（新增 value 单测 **15**）。
    - 新增测试名：`deep_eq_self_referential_arrays_are_equal_and_terminate`、`deep_eq_pure_cycle_arrays`、`deep_eq_self_referential_structs`、`deep_eq_mutually_aliased_cycles`、`deep_eq_struct_ignores_method_fields`、`deep_eq_struct_key_order_independent`、`deep_eq_scalars_and_exact_int_float`、`deep_eq_function_identity_only`、`deep_eq_different_container_types_are_false`、`deep_eq_nested_containers`、`total_cmp_numeric_group`、`total_cmp_infinities_and_nan_last`、`total_cmp_strings_by_utf8_bytes`、`total_cmp_incomparable_categories_return_none`、`order_kind_classification`。
- **P3.9a 内置函数表（非高阶）**（2026-09-23）：`builtins.rs` 47 个非高阶内置 + 稳定 ABI（`BuiltinFn`/`Builtin`/`lookup`/`call`/`BUILTIN_NAMES`/`TABLE`）；7 个 HOF 留 P3.9b（`// TODO(P3.9b)`）。
- **P3.9a 契约缺口闭合（清单 3/4/5 落地）**（2026-09-24）：`min`/`max` 空 → `ValueMsg::EmptyExtremum`；`randInt(lo>=hi)` → `BadRange`；`del(k,s)` 仅数据字段（`del 成功 ⟺ has`）；3 条确认对齐仅补断言。
- **P3.6a 值模型 + 作用域**（2026-09-23）：`value.rs`（`Value` 九变体含 `StructDef`、`as_f64` 唯一加宽、环安全 `Display`、`StructObj`）+ `env.rs`（A2 cell 捕获 / `ScopeDebug` 链）。

## 阻塞 / 需要支持
- **无硬阻塞**。
- **规范未明确项（已上报，未自行发明；本实现取保守口径）**：
  1. **`StructDef`（struct 模板）的 `==` 语义**：§4.5.9 未列该类（非标量、非容器）。本实现取**同一性**（同 `Rc` → `true`，否则 `false`），**待 language-architect 补齐**。
  2. **§4.5.5 深结构 10000 层上限**：`deep_eq`（与既有 `Display` 同）为**递归**，未施加 10000 层深度上限 / `RecursionError`（需显式迭代栈 + `Result`）。建议随 P3.7 收口。
  3. **`< <= > >=` 的 IEEE `NaN → false`**：**不**由 `total_cmp` 承担（`total_cmp` 仅服务排序 / 极值，`NaN` 排最后）；P3.7 求值器须自行按 §4.5.6 处理。

## 下一步计划
- 等 team-lead 派发 **P3.7 `evaluator.rs` 核心**（表达式/语句/控制流/函数与闭包/管道调用/`;;`）——它是 P3.9b 的前置。
  - 消费 `Value::deep_eq`（`==`/`!=`）与 `Value::total_cmp`（`< <= > >=`，`None` → `TypeError`；`NaN` 另行处理）。
- **P3.9b**：7 个高阶内置（`map`/`filter`/`reduce`/`sortBy`/`minBy`/`maxBy`/`each`）；`sortBy` 稳定、按 `keyFn` 全序（复用 `Value::total_cmp`）；`minBy`/`maxBy` 空数组复用 `ValueMsg::EmptyExtremum`。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **warning/error 计数**：`$out = cargo build --tests --message-format=json 2>$null; (($out|Select-String '"level":"warning"')).Count`（`2>$null` 丢进度，JSON 诊断走 stdout；**勿** `2>&1`，会被 PowerShell 包成 `NativeCommandError`）。
- **并发编辑**：多名 agent 同时改 `src/**` 时，`cargo test` 可能捕捉到他人**半成品**状态。判据：只看**自己模块**的测试 + 全绿时的全量结果；他模块失败先确认归属再上报，勿越界修。
- **A6 环安全用「有序对集合」而非「路径集合」**：与 `Display` 的「当前路径集合」（进入 push / 离开 pop）不同，`==` 的已访问对集合**全程不 pop**（同构图的余归纳等价）——这是 `[a] == [b]` 类纯环能判相等、且不误报不等的原因。
- **`RefCell` 嵌套深比较安全**：`eq_rec` 只做**不可变** `borrow()`（从不 `borrow_mut`），故同一 cell 在嵌套路径上被多次不可变借用不会 panic；且「先查 seen 再 borrow」使自引用对短路。
- **比较逻辑单一化**：`builtins` 的**错误话语**（`TypeError` 期望 `number`/`string`）留在 `builtins::validate_orderable`（消费 `Value::order_kind`），**比较算法**全部下沉 `value.rs`——分工即「口径在 value、话语在 builtins」。
- **共享常量 `TWO_POW_63`**：`int(float)` 越界判定（`builtins::float_to_int`）与 int/float 精确比较（`value::cmp_int_float`）共用 `value::TWO_POW_63`，避免 2^63 边界漂移。
- **文件名过长（`lfz-programing language design`）**：PowerShell `Select-String -Path "src\*.rs","app\**\*.rs"` 中不存在的通配路径会整体报错；用 `grep` 工具或逐路径更稳。
