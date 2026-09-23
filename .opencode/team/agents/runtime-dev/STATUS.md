# runtime-dev — 工作状态
> 最后更新: 2026-09-24 by runtime-dev

## 当前状态
**P3.9a ✅ 完成**（内置函数第一批：47 个非高阶内置）+ **P3.9a 契约缺口闭合 ✅ 完成**（清单 3/4/5 落地）。
待 team-lead 核验 + release-manager 提交 `feat(p3): builtins (non-HOF)`。
- **7 个高阶内置**（`map` `filter` `reduce` `sortBy` `minBy` `maxBy` `each`）留 **P3.9b**（已 `// TODO(P3.9b)` 标注）。
- 架构师 6 项缺口裁定**全部闭合**：清单 3/4（改代码）+ 清单 5（确认一致，无代码变更）均落地并单测。

## 进行中
- （无）

## 最近完成
- **P3.9a 契约缺口闭合（清单 3/4/5 落地）**（2026-09-24）
  - `src/builtins.rs`（本轮 **+55 / −28 行**）。
  - 逐条：
    1. **`min`/`max` 空数组 → `ValueMsg::EmptyExtremum { func }`**（不再用 `Convert` 兜底，ADR 缺口 1）：`empty_collection_value_error(name, span)`（L1079）改用新变体；消息 `空数组没有极值（min）` / `（max）`。`minBy`/`maxBy` 属 HOF（P3.9b），代码里**无占位实现**，无需改。
    2. **`randInt(lo,hi)` 且 `lo >= hi` → `ValueMsg::BadRange { lo, hi }`**（ADR 缺口 2）：`b_rand_int`（L884）；消息 `区间非法：5 >= 5`。
    3. **`del(k,s)` 仅作用于数据字段**（ADR 缺口 5，§4.5.9）：`b_del`（L686）判定由 `raw_fields`「存在即删」改为「**数据字段集合**」；新增共用谓词 `is_data_field`（L634），`b_has`（L675）复用之；`k` 为方法字段 → `FieldError`（`结构体没有字段 'm'`）。**不变量 `del(k,s) 成功 ⟺ has(k,s)`** 由新测 `del_succeeds_iff_has_is_true` 锁定。
    4. **确认对齐（无代码变更）**：
       - `pop([])` → `Index { idx: -1, len: 0 }`（`b_pop` L487）——已符合；
       - `insert` 合法域恒 `i ∈ [0,len]`、`i<0 || i>len` → `Index { idx: 实参 i, len: len(xs) }`、**不支持负索引**（`b_insert` L509）——已符合；
       - `floor`/`ceil`/`round` 的 `NaN`/`±Inf`/超 `i64` 复用 `int(float)` 口径（`float_to_int` L305：`NaN` → `ValueMsg::Convert{src:"float",dst:"int",text:"nan"}`、`±Inf`/截断后超界 → `OverflowError`）——已符合（三者均经 `float_to_int`）。
       - 以上 3 条**仅补断言**（未改实现），逐字符锁定规范文本。
  - 证据
    - `git diff --stat -- src/builtins.rs` → `1 file changed, 55 insertions(+), 28 deletions(-)`。
    - `cargo build --tests --message-format=json 2>$null` → `warnings=0 errors=0`。
    - `cargo test` → **`135 passed; 0 failed`**（builtins 模块 **36** 个测试全绿）。
- **P3.9a 内置函数表（非高阶）**（2026-09-23）
  - `src/builtins.rs`（P3.9a 初版 72102 B / 1549 行，含单测）。
  - **稳定 ABI**：
    - `pub type BuiltinFn = fn(&[Value], Span) -> R<Value>;`
    - `pub struct Builtin { name, min_args, max_args, func }` + `Builtin::call(args, span)`（**集中**校验参数个数）。
    - `pub fn lookup(name) -> Option<Builtin>`、`pub fn call(name, args, span) -> R<Value>`（未知名 → `NameError`）、
      `pub fn is_builtin(name) -> bool`、`pub const BUILTIN_NAMES: &[&str]`（47，字母序）。
    - `static TABLE: [Builtin; 47]`（字母序，与 `BUILTIN_NAMES` 逐项一致，**由测试锁定**）。
  - **实现清单（47 → ✅）**：
    - 核心/数组（14）：`len` `range` `push` `pop` `removeAt` `insert` `swap` `slice` `min` `max` `sum` `sort` `take` `drop`
    - struct/字典（5）：`keys` `values` `entries` `has` `del`
    - 字符串（8）：`split` `join` `trim` `upper` `lower` `replace` `repeat` `startsWith`
    - 数学/随机/转换/IO/断言（20）：`abs` `floor` `ceil` `round` `sqrt` `pow` `div` `rand` `randInt` `seed` `str` `int` `float` `type` `print` `eprint` `input` `assert` `check` `fail`
  - **规范要点落地**：
    - **A1（返回新值）**：`push/pop/removeAt/insert/swap/slice/sort/take/drop/del` 一律先 `snapshot` 再产新容器，**原容器不变**（含 `Rc` 不同一性断言）。
    - **A5/B3（数据面 + 键序）**：`keys/values/entries/has/len` 走 `StructObj::data_fields_sorted()`（跳方法字段、字节序升序）；`has` 对方法字段返回 `false`。
    - **§10.7 数值形参加宽**：`floor/ceil/round/sqrt/pow` 的 `int` 实参经 `Value::as_f64()`（唯一入口）加宽；**`abs` 同型、绝不加宽**（`abs(-3)`→int 3、`abs(-3.0)`→float 3.0）。
    - **`int()` 边界（§4.5.7）**：`NaN → ValueError`；`±Inf → OverflowError`；有限浮点**向零截断**、截断后超 `i64` → `OverflowError`。
    - **`round` = 银行家舍入**（`floor` + 半值取偶）；**`div` 向下取整**、`b == 0 → ZeroDivisionError`、`i64::MIN / -1 → OverflowError`。
    - **`check` 非致命**：失败 → stderr 一行 `check 失败：{msg}` + 返回 `false`、**不产生 `LzError`**；**`assert`/`fail` 致命**（`Assert.msg` 由构造方组装：`断言失败：{msg}` / `{msg}`）。
    - **全序（§4.5.6 / §4.5.7）**：`min/max/sort` 支持数值（int/float 混用，**数学精确比较**——大整数不因加宽误判）与 string（字节序）；`-Inf < 有限 < +Inf < NaN`；类型不一致 → `TypeError`。`sort` 用 `slice::sort_by`（**稳定**）。
    - **随机**：无第三方依赖，自实现 `splitmix64`（`thread_local` 状态）；`rand` ∈ `[0.0,1.0)`、`randInt(lo,hi)` ∈ `[lo,hi)`（i128 宽度计算防溢出）、`seed(n)` 固定序列、默认时间种子。
    - **IO**：`print`/`eprint` 按显示形式空格连接；`input` 先写 prompt 并 flush、**EOF → IOError**、去 `\n`/`\r\n`；内核 `input_with<R: BufRead, W: Write>` **可单测**。
  - **参数校验**：个数不符 → `TypeError::ArgCount`（`n` 取越界侧边界）；类型不符 → `TypeError::BadOperands`；`assert`/`check` 条件非 bool → `TypeError::ConditionNotBool`。
  - **位置红线**：所有内置错误携带**调用点 `Span`**（ABI 携带，见 ADR）。
  - 证据：`cargo test` → 当时 `82 passed; 0 failed`（P3.6 基线 47 + P3.9a 新增 35）。

## 阻塞 / 需要支持
- **无硬阻塞**。原 6 处契约缺口已由 language-architect 裁定（ADR [2026-09-24 00:20]）并由本轮代码落地（清单 3/4/5）**全部闭合**。
- 备注：本轮 `cargo test` 期间 core-dev 正并发编辑 `src/lexer.rs`（P3.3b 字符串/插值），曾出现 2 次**瞬时** lexer 测试失败；core-dev 改动稳定后全绿。**与本轮 `builtins.rs` 改动无关**。

## 下一步计划
- 等 team-lead 派发 **P3.7 `evaluator.rs` 核心**（表达式/语句/控制流/函数与闭包/管道调用/`;;`）——它是 P3.9b 的前置。
- **P3.9b**：在求值器就绪后实现 7 个高阶内置（`map` `filter` `reduce` `sortBy` `minBy` `maxBy` `each`）；`sortBy` 稳定、按 `keyFn` 全序；`filter` 谓词须 `bool` 否则 `TypeError`；`minBy`/`maxBy` 空数组复用 `ValueMsg::EmptyExtremum`。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **warning/error 计数**：`$out = cargo build --tests --message-format=json 2>$null; (($out|Select-String '"level":"warning"')).Count`（`2>$null` 丢进度，JSON 诊断走 stdout；**勿** `2>&1`，会被 PowerShell 包成 `NativeCommandError`）。
- **并发编辑**：多名 agent 同时改 `src/**` 时，`cargo test` 可能捕捉到他人**半成品**状态（本轮 lexer 曾瞬时失败）。判据：只看**自己模块**的测试（`cargo test --lib builtins::`）+ 全绿时的全量结果；他模块失败先确认归属再上报，勿越界修。
- **数据面单一谓词**：`keys/values/entries/has/len/del` 必须共用**同一**「数据字段」谓词（本轮抽为 `is_data_field`）——`Value::Func(_)` 即方法字段。`del` 的「成功 ⟺ `has`」不变量是这类改造的验收锚点。
- **`Ref::clone()` 陷阱**：`rc.borrow().clone()` 对 `RefCell<T>` 会把 `Ref` 本身 `Clone`（得 `Ref`，非 `T`），并在尾表达式位置触发 `E0597`。要克隆**内层值**：先 `let snap = rc.borrow().to_vec();`（或 `(*rc.borrow()).clone()`）再返回。
- **泛型参数名勿用 `R`**：本 crate 的 `error::R<T>` 会被遮蔽 → `error: type arguments are not allowed on type parameter R`。`input_with<Rd: BufRead>` 即为此改名。
- **`&Value` 的 `.clone()`**：`best.clone()`（`best: &Value`）解析为 `<&Value as Clone>` 得 `&Value`；要 `Value` 须写 `Value::clone(best)`。
- **§8.1 消息选型**：条件类实参（`assert`/`check`）的非 bool 应走 `TypeMsg::ConditionNotBool`（「条件必须是 bool，得到 {t}」），**不是**通用 `BadOperands`；通用实参类型不符才用 `BadOperands`。
- **`-0.0` 可作稳定排序的可观测探针**：`-0.0` 与 `0.0` 全序相等（`partial_cmp == Equal`）但显示可区分（`-0.0` / `0.0`），据此在**纯标量**上单测 `sort` 稳定性。
- **空 struct 显示为 `{}`**（`Display` 跳过函数值字段后为空）：可用于 `del` 掉唯一数据字段后的断言。
