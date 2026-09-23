# runtime-dev — 工作状态
> 最后更新: 2026-09-23 by runtime-dev

## 当前状态
**P3.9a（`src/builtins.rs` 内置函数第一批：全部非高阶内置）✅ 完成，待 team-lead 核验 + release-manager 提交 `feat(p3): builtins (non-HOF)`。**
- 对照 §10.7 全表：**47 个非高阶内置全部实现**；**7 个高阶内置**（`map` `filter` `reduce` `sortBy` `minBy` `maxBy` `each`）留 **P3.9b**（已 `// TODO(P3.9b)` 标注）。

## 进行中
- （无）

## 最近完成
- **P3.9a 内置函数表（非高阶）**（2026-09-23）
  - `src/builtins.rs`（**72102 B / 1549 行**，含单测）。
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
  - **位置红线**：所有内置错误携带**调用点 `Span`**（ABI 携带，见下「阻塞/需支持」1 与 ADR）。
  - 证据
    - `cargo build --tests --message-format=json 2>$null` → `warnings=0 errors=0`。
    - `cargo test` → **`82 passed; 0 failed`**（P3.6 基线 47 + P3.9a 新增 **35**）。

## 阻塞 / 需要支持
- **无硬阻塞**。以下为**契约缺口 / 需 language-architect 补钉**（**非阻塞**，本批已按最贴近规范的方式落地并单测）：
  1. **ABI 携带 `Span`（决策，已记 ADR）**：任务书「建议」`fn(&[Value]) -> R<Value>`；但「运行时错误必须带位置」为红线，故本批 ABI 定为 **`fn(&[Value], Span) -> R<Value>`**（`Span` = 调用点）。P3.7 求值器按此调用。**不改变契约语义**，仅钉死 Rust 侧签名。
  2. **`min`/`max` 空数组 → `ValueError`，但无对应消息模板**：`error.rs::ValueMsg` 为只读冻结枚举（仅 `Convert` / `BadFormatSpec`），§8.1 `ValueError` 两模板均不适用「空集合取极值」。现以 `Convert{src:"array", dst:<内置名>, text:"空数组"}` 承载。**待补钉**（建议新增 `ValueMsg` 变体或在 §8.1 给出消息）。
  3. **`randInt(lo,hi)` 且 `lo >= hi` → `ValueError`，同缺口 2**：现以 `Convert{src:"int", dst:"int", text:"{lo} >= {hi}"}` 承载。**待补钉**。
  4. **`pop([])` 的 `IndexError` 下标值未规定**：现取 `idx = -1, len = 0`（`下标 -1 越界（长度 0）`）。**待补钉**。
  5. **`floor`/`ceil`/`round` 的 `NaN`/`±Inf`/超 `i64` 结果**：§10.7 只给「返回 `int`」未给边界；现复用 `int(float)` 口径（`NaN → ValueError`、`±Inf`/超界 → `OverflowError`）。**待补钉**。
  6. **`del(k, s)` 对方法字段**：A5 未列 `del`；现按 `raw_fields` 判定「存在即删」（方法字段可被删）。**待确认**。

## 下一步计划
- 等 team-lead 派发 **P3.7 `evaluator.rs` 核心**（表达式/语句/控制流/函数与闭包/管道调用/`;;`）——它是 P3.9b 的前置。
- **P3.9b**：在求值器就绪后实现 7 个高阶内置（`map` `filter` `reduce` `sortBy` `minBy` `maxBy` `each`）；`sortBy` 稳定、按 `keyFn` 全序；`filter` 谓词须 `bool` 否则 `TypeError`。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **warning/error 计数**：`$out = cargo build --tests --message-format=json 2>$null; (($out|Select-String '"level":"warning"')).Count`（`2>$null` 丢进度，JSON 诊断走 stdout；**勿** `2>&1`，会被 PowerShell 包成 `NativeCommandError`）。
- **`Ref::clone()` 陷阱**：`rc.borrow().clone()` 对 `RefCell<T>` 会把 `Ref` 本身 `Clone`（得 `Ref`，非 `T`），并在尾表达式位置触发 `E0597`。要克隆**内层值**：先 `let snap = rc.borrow().to_vec();`（或 `(*rc.borrow()).clone()`）再返回。
- **泛型参数名勿用 `R`**：本 crate 的 `error::R<T>` 会被遮蔽 → `error: type arguments are not allowed on type parameter R`。`input_with<Rd: BufRead>` 即为此改名。
- **`&Value` 的 `.clone()`**：`best.clone()`（`best: &Value`）解析为 `<&Value as Clone>` 得 `&Value`；要 `Value` 须写 `Value::clone(best)`。
- **§8.1 消息选型**：条件类实参（`assert`/`check`）的非 bool 应走 `TypeMsg::ConditionNotBool`（「条件必须是 bool，得到 {t}」），**不是**通用 `BadOperands`；通用实参类型不符才用 `BadOperands`。
- **`-0.0` 可作稳定排序的可观测探针**：`-0.0` 与 `0.0` 全序相等（`partial_cmp == Equal`）但显示可区分（`-0.0` / `0.0`），据此在**纯标量**上单测 `sort` 稳定性。
