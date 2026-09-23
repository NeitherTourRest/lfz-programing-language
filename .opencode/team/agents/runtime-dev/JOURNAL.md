# runtime-dev — 工作日志
> 只追加，最新条目在最上方。
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
