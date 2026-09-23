# LFZ 解释器后端架构建议（Rust · 高性能）

> 作者: runtime-dev（运行时开发）
> 日期: 2026-09-23
> 状态: **设计草案（DESIGN-DRAFT，待用户评审）** — 非正式交付物，未冻结
> 依据: `DECISIONS.md` D-011（实现语言 = Rust）、D-007（v1 = 5 特色 + float）、`task-info.md`（性能项 10 分）、`KICKOFF.md` §一.1（跨语言性能硬数据）
> 范围: 仅覆盖后端（evaluator / vm / builtins / env）；前端 lexer/parser/AST 由 core-dev 负责
> 说明: 本文所有性能数字为**工程估算**，最终以 P6 实测为准；标 ★ 的是硬性架构约束

---

## 0. 结论先行（TL;DR）

| 问题 | 建议 | 一句话理由 |
|------|------|-----------|
| 执行模型 | **字节码栈式 VM 为最终引擎**；以**优化树遍历器**作 P3 先行 MVP + 差分测试参照 | 只有字节码 VM 能稳定追平/超越 CPython；树遍历器保证关键路径不被 VM 拖死 |
| 值表示 | `enum Value`（标量内联 Copy）+ `Rc<RefCell<...>>` 承载可变复合类型 | 免箱整数/浮点是相对 CPython 的**最大性能红利** |
| 变量 | 编译期 name resolution → local **slot** / global **index**；热路径零哈希 | 消灭 CPython 的 dict 查名开销 |
| 调用栈 | VM 内**迭代式**调用帧（堆上 `Vec<CallFrame>`） | 递归深度只受内存限制，无 Python 式 1000 层天花板 |
| 错误 | 全程 `Result<T, LfzError>`；**用户错误绝不 panic**；错误构造只在冷路径 | 热路径零开销，错误仍带 span/code/hint |
| 性能目标 | `--release` 下：数值/循环基准 **≥ 1.0× CPython**（目标 2–3×），动态字符串/结构体 **≥ 0.5×**；中位数 ≥ 1.0× | 把"性能优良"变成可执行门禁 |
| 路线 | **P3 双后端共享骨架 → P6 加 VM 层** | 正确性先行，性能后加，不返工 |

---

## 1. 执行模型选型

四种候选：(a) 朴素树遍历、(b) 优化树遍历（resolved slots）、(c) 编译到字节码 + 栈式 VM、(d) 闭包/JIT 式编译。

### 1.1 对比表（对 CPython 的估算）

| 模型 | 对 CPython 预期倍率¹ | 工作量² | AI 团队复杂度/风险 | 适配 v1 特性集³ | 判定 |
|------|--------------------|--------|-------------------|----------------|------|
| (a) 朴素树遍历（`HashMap` 环境） | 0.01–0.05×（慢 20–100×） | 最低 ~3–5d | 低 | 全适配 | 仅作 MVP |
| (b) 优化树遍历（resolved slots） | 0.4–1.5× | 中 ~5–8d | 低–中 | 全适配 | **备选 / 参照** |
| (c) 字节码栈式 VM | 数值·循环 1.5–5×；动态字符串/结构体 0.5–1.5× | 高 ~12–20d | 中–高 | 全适配 | **★ 主推** |
| (d) 闭包/JIT（Cranelift / tracing / copy-and-patch） | 潜在 5–50× | 极高（需第三方 crate / unsafe） | 极高 | 过度设计 | **否决** |

> ¹ 倍率 = CPython 耗时 / LFZ 耗时（>1 表示 LFZ 更快）。基于：跨语言基准（ceronman/loxido）显示**任何树遍历解释器都慢于 CPython**，仅 C/unsafe-Rust 字节码 VM 能稳定快于 Python；再叠加 LFZ 的实现红利（标量免箱、slot 寻址、无 GIL、无 per-op 引用计数）。
> ² 人日估算按"熟练实现者单线开发"折算；AI 团队实际差异主要体现为会话轮次与返工率，非墙钟人日。
> ³ v1 特性：管道 `|>`、合一 struct（dict+object+method）、富插值、结构化错误 + assert/check、确定性语义、float。四种模型都能表达，差异只在效率与复杂度。

### 1.2 逐项分析

**(a) 朴素树遍历** — `eval(node, env)` 递归 + 环境链 `HashMap<String, Value>`。
- 优点：最快拿到"能跑 hello world + 全特性通过单测"，P3 关键路径最短；语义最直观，适合当"规格活文档"。
- 缺点：每变量访问一次字符串哈希；每节点一次 Rust 递归；无指令局部性。**honest 预期：比 CPython 慢一个数量级**，无法支撑"性能优良"。
- 结论：只作 MVP，且环境结构**从第一天就按 slot 设计**，避免二次重写（见 §3）。

**(b) 优化树遍历（resolved slots）** — 加一个 **resolver pass**，把 `Variable{name}` 改写为 `Local{slot}` / `Global{idx}` / `Upvalue{idx}`；环境变 `Vec<Value>` 帧 + 父帧索引。
- 优点：干掉哈希后，整型运算 + 循环可接近 CPython（≈0.4–1.5×）；resolver 与字节码编译器**可复用**；改动相对小。
- 缺点：树遍历的固有开销仍在——每个 AST 节点一次**虚/枚举匹配 + 原生递归 + 指针追逐**，没有指令流局部性。难以稳定超过 CPython（CPython 3.11+ 有自适应特化解释器，3.13 还有实验性 JIT，基线本身在走高）。
- 结论：作为**过渡实现 + 差分测试参照**，并作为 VM 进度失控时的 fallback。

**(c) 编译到字节码 + 栈式 VM** — `compiler` 把（已 resolve 的）AST 编译成 `Chunk { code: Vec<Instr>, consts: Vec<Value>, spans: Vec<Span> }`；VM 用 `loop { match code[ip] }` 分发。
- 优点：**唯一有现实希望稳定 ≥ CPython 的路线**。指令紧凑、缓存友好；无原生递归；配合 slot 寻址 + 标量免箱 + 类型特化 opcode（`AddInt`/`AddFloat`/`AddStr`），数值/循环负载可显著领先 CPython（CPython 每个 int 都装箱 + 引用计数增减，LFZ 不会）。
- 缺点：活动部件多（ip、栈、帧、upvalue、常量池、反汇编器），测试面更大；对 AI 团队是**中–高风险**（但可控，见 §7 的差分测试与分步提交纪律）。
- 结论：**★ 作为最终执行引擎**。

**(d) 闭包/JIT** — Cranelift JIT / tracing JIT / copy-and-patch。
- 优点：理论性能天花板最高。
- 缺点：需要第三方 crate（违反"优先仅用 std"）；unsafe 与平台相关度高（Windows MSVC）；调试/测试成本极高；对课程作业是**严重的过度工程**，会给权重最高的评分项（解释器 20 + 测试 20 + 应用 30）引入灾难性风险。
- 结论：**v1 明确否决**；仅在 v1.1 有余量时作为"性能叙事彩蛋"评估。

### 1.3 明确建议

- **主推 (c) 字节码栈式 VM**，作为 LFZ 的**目标执行引擎**——这是"D-011 选了 Rust + 用户要性能优良"这一组合下唯一站得住的选择。
- **以 (b) 优化树遍历器作为 P3 先行 MVP 与永久差分测试参照**，两者**共享** `Value` / 环境布局 / `LfzError` / resolver / builtins ABI。
- **Fallback**：若 P6 排期被黑盒测试/应用挤占，则发布**(b) 优化树遍历器**并如实报告——它已含 resolver，通常接近 CPython，足以支撑一份诚实可信的性能报告。

### 1.4 Rust 专属注意事项（★ 全部为硬约束）

1. **所有性能工作必须用 `cargo build --release`**。Rust debug 构建可慢 10–100×；性能报告若用 debug 数字即作废。
2. **整型溢出必须报错**（D-007：64-bit 有符号、溢出报错）→ 运算用 `checked_add/checked_mul/...`，溢出转 `LfzError`。这比 wrapping 多一个分支，但**仍远廉于 CPython 的装箱 + 引用计数**。
3. **LFZ 无隐式转换**（确定性语义）→ 可用类型特化 opcode（int+int、float+float、str+str 分派），避免运行期反复判型。
4. 指令表示 v1 用 `enum Instr`（可读、LLVM 优化好）；若 profiling 显示分发是瓶颈，再评估定宽 8–16 字节指令 + 跳转表。
5. 稳定版 Rust **无保证尾调用优化**，不要依赖"尾递归 = 循环"。
6. 不引第三方 crate：interner、identity hasher、计时都用 std 自实现。

---

## 2. 运行时值表示（`Value`）

### 2.1 推荐定义（★ 冻结候选）

```rust
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),                          // 内联，免箱 —— 相对 CPython 的核心红利
    Float(f64),                        // 内联，免箱
    Str(Rc<String>),                   // 不可变；clone = 计数 +1（O(1)）
    Array(Rc<RefCell<Vec<Value>>>),    // 引用语义
    Struct(Rc<RefCell<Struct>>),       // Struct { fields: Vec<(Sym, Value)> }
    Func(Rc<Closure>),                 // Closure { chunk: Rc<Chunk>, upvals: Vec<Rc<RefCell<Value>>> }
    Native(NativeId),                  // builtin 按 id 分发，无闭包分配
}

pub struct Span { pub start: u32, pub end: u32 }   // 8 字节，非 usize
pub type Sym = u32;                                // interned 标识符/字段名
```

- **`Value` 目标大小 = 16 字节**：判别符 + 8 字节负载（i64/f64/指针均可容纳）。
- 关键取舍：字符串用 **`Rc<String>`（细指针，8B）** 而非 `Rc<str>`（胖指针，16B）。胖指针让 `Value` 涨到 24B，栈/帧缓存密度下降；细指针多一次解引用但**每个栈槽更小**，对栈式 VM 更划算。取 `&rc_str[..]` 即得 `&str`，使用无碍。

### 2.2 备选方案与实际取舍

| 方案 | 优点 | 缺点 | 判定 |
|------|------|------|------|
| **`enum Value` + `Rc<RefCell>`（推荐）** | 安全、std-only、可读、标量内联 | 复合类型运行期借用检查有成本；`Rc` 有计数开销 | ★ v1 采用 |
| NaN-boxing（值压进 `u64`） | 8B 值、无判别分支、f64 免箱 | 需 `unsafe`、指针 48-bit 规范化、与 `Rc` 组合脆弱、调试困难、GC 集成难 | v1 否决 |
| 手写 arena / `Gc` crate | 天然处理环 | 需第三方或自研 GC（巨大复杂度） | v1 否决 |
| 全 `Rc<RefCell<Value>>`（连标量也装箱） | 实现最省事 | 每次算术都要分配 + 借用检查 → **性能自杀** | 否决 |

### 2.3 `Rc<RefCell>` 与性能的张力（实现纪律）

- **缩短借用窗口**：绝不在持有 `borrow_mut()` 期间递归求值。例：`a[i] = f()` 必须**先算下标与右值**，再 `borrow_mut` 写入；否则既慢又易 `RefCell` panic。
- **避免每步 `try_borrow` 语义**：借用冲突是**程序 bug**而非用户错误，用 `borrow_mut()`（panic 暴露）即可，不要为它造 `LfzError` 分支污染热路径。
- **只在被捕获时才装箱**：普通局部变量存裸 `Value`（`Vec<Value>` 帧）；仅当它被闭包捕获时，才升级为 `Rc<RefCell<Value>>` cell。这样"没有闭包的循环"零额外分配。
- **字符串不可变**：`Str` 拼接产生新 `Rc<String>`，不改变原值（需 spec 明确字符串不可变）。这也让 `Rc<str>`/`Rc<String>` 的 clone 廉价且无别名风险。

### 2.4 GC-less 的现实：环与泄漏（必须写进语义/文档）

Rust std **没有环收集器**；`Rc` 构成的环会泄漏到进程结束。v1 里能造环的例子：

- `var s = {}; s.self = s;` → struct 自引用；
- 局部递归闭包捕获自身所在环境 → `env → closure → env`。

**处置（推荐）：**
1. **全局函数不进入被捕获环境**：globals 由 VM 以顶层 `Vec<Value>` 持有，**不包在 `Rc` 里**；命名函数的递归通过 global index 解析 → 最常见的递归/闭包场景**不产生环**。
2. **接受"短生命周期脚本"下的有界泄漏**：CLI 进程跑完即退，泄漏无实际影响；在语义文档中**明示**"LFZ 不做环回收；自引用结构在长驻进程中会泄漏"（诚实、且符合课程规模）。
3. 若未来必须无泄漏：仅对复合类型加一个**弱引用注册表 + 标记清扫**，或对回边使用 `Weak`。v1 **明确延后**。

> 结论：**v1 无环 GC；靠环境布局让常见场景不成环；文档如实声明。** 用简单换速度，不引入追踪式 GC。

### 2.5 标识符 interning 与 struct 字段

- **Interner**：parser 阶段把所有标识符/字段名映射为 `Sym(u32)`（`Vec<String>` + `HashMap<String, u32>`，仅解析期哈希，运行期零字符串哈希）。
- **struct 字段存储**：v1 用 **`Vec<(Sym, Value)>` + 线性扫描**（字段通常 ≤ 8–16 个，缓存友好、天然保持插入顺序，利于打印与"两种面孔"的确定性）。若 P6 profiling 证明存在大结构体热点，再切到 **identity-hash 的 `HashMap<Sym, Value>`**（自实现把 `u32` 直通的 `Hasher`，std-only）。
- **方法**：方法就是值为函数的字段；`s.m(...)` 运行期按 `Sym` 查字段并绑定 `self`。方法名是动态字段 → 运行期查找，但键是 `u32` 而非字符串，开销可接受；方法调用点后续可加**内联缓存（inline cache）**。

---

## 3. 变量 / 作用域

核心原则：**把名字解析从运行期挪到编译期**，热循环里不留任何哈希。

### 3.1 名字解析（resolver pass，两种后端共享）

| 引用种类 | 解析结果 | 运行期取值 | 代价 |
|---------|---------|-----------|------|
| 局部变量 | `Local { slot: u32 }` | `stack[frame.base + slot]` | 数组索引（O(1)，无哈希）|
| 全局变量 | `Global { idx: u32 }` | `globals[idx]` | 数组索引（O(1)，无哈希）|
| 被闭包捕获的局部 | `Upvalue { idx: u32 }` | `upvals[idx].borrow()` | 一次 cell 解引用 |
| struct 字段 | `Sym`（u32） | `fields` 线性扫描 / 哈希 | 字段少时近似 O(1) |

- LFZ 无 `eval`、无按名字动态取变量、无模块 → **全局也能在编译期定 index**，不必用 `HashMap`。
- 未定义变量的检测：全局表用 `Value::Uninit` 哨兵或 `defined: Vec<bool>` 位图，读取未定义 → 结构化错误 `E_UNDEF`。

### 3.2 帧布局与作用域

- **Local 帧 = 栈上连续切片** `[base .. base + nlocals]`；参数先入栈再"成为"前 n 个局部变量（clox 同款）。
- **块作用域**：进块分配新 slot 区间，出块回收（`base` 指针回退）；变量遮蔽 = 不同 slot，天然正确。
- **全局**：VM 持有的顶层 `Vec<Value>`，**不参与闭包捕获**（避免 §2.4 的环）。
- **闭包捕获语义**（★ 须 spec 冻结）：推荐**按 cell 引用捕获**——被捕获局部升级为 `Rc<RefCell<Value>>`，闭包与外部共享该 cell，外部后续修改对闭包可见。这是最常见且实现最简单的语义；**必须与 language-architect 确认**，因为它是"照直觉写错"的高发区。

### 3.3 如何彻底移除热路径哈希

1. 局部/全局 → 编译期 slot/index。**这是最大的一刀。**
2. 字段名/标识符 → interner 的 `Sym(u32)`；struct 字段 ≤16 时线性扫描。
3. 不用 `HashMap<String, Value>` 做任何环境。
4. 后续可选：方法调用点 inline cache；字符串常量池去重。

> 与 CPython 对比：CPython 每 `LOAD_GLOBAL`/`LOAD_NAME` 要走 dict + 自适应缓存；LFZ 全局是裸数组索引，局部是栈偏移。

---

## 4. 函数调用 / 栈

### 4.1 VM 结构（迭代式，无原生递归）

```rust
struct Vm {
    stack:   Vec<Value>,             // 值栈
    frames:  Vec<CallFrame>,         // 调用帧栈（堆上）
    globals: Vec<Value>,
    chunks:  Vec<Rc<Chunk>>,
}
struct CallFrame {
    chunk: Rc<Chunk>,
    ip: usize,
    base: usize,                     // 本帧局部变量在 stack 中的起点
}
```

- 每次 LFZ 函数调用 = push 一个 `CallFrame`，`base` 指向参数起点；**VM 主循环本身是 `loop`，绝不为 LFZ 调用递归 Rust**。
- **递归深度只受内存限制**，不存在 Python 的 1000 层硬顶。可选地实现一个可配置 `max_frames`（默认很大，如 10^5）以把"失控递归"变成结构化错误而非 OOM。

### 4.2 树遍历器（P3 MVP）的栈问题

- 树遍历器每个 LFZ 调用要**原生递归** → 深递归会撞 Rust 线程栈，直接 abort。
- ★ 缓解：解释器主逻辑跑在**大栈线程**（`std::thread::Builder::new().stack_size(64<<20)`），并加**深度计数器**：超限 → 抛 `LfzError { code: E_STACK_OVERFLOW, span, hint }`，绝不 abort。这样 MVP 也能深递归且错误可控。
- VM 上线后此限制自然消失（§4.1）。

### 4.3 调用约定与 builtins ABI

- **builtins 统一 ABI**：`fn(&mut Vm, args: &[Value]) -> Result<Value, LfzError>`，从栈上弹出实参、压回返回值。**builtins 只面向该 ABI 编写，不碰 evaluator 内部** → 两种后端**共用同一套 builtins**，零重写。
- `print` / `input` / `int` / `float` / `str` / `assert` / `check` / 以及 app 需要的 `random(seed)` / `sleep` / ANSI 原样输出，全部注册为 `NativeId`。
- 不发明契约外 builtin；清单以 `interface-contract.md` + `semantics.md` 为准。
- **无 tail-call 优化**（稳定 Rust 不保证）→ 深尾递归仍占帧；文档说明即可。

---

## 5. 错误处理

### 5.1 总原则

- **用户可见的一切失败走 `Result<T, LfzError>`；绝不 `panic`。** `panic` 只留给"内部不变量被破坏"的程序 bug（并尽量用 `unreachable!`/`debug_assert!` 表达）。
- 典型 `LfzError` 类别（须与 spec 的错误码表对齐）：`E_TYPE`、`E_DIV0`、`E_OVERFLOW`、`E_OOB`、`E_UNDEF`、`E_NOT_CALLABLE`、`E_ARITY`、`E_ASSERT`、`E_STACK_OVERFLOW`、`E_IO`。

### 5.2 形态与热路径零开销

```rust
pub struct LfzError {
    pub code: ErrCode,        // enum，可 JSON 序列化为稳定字符串码
    pub span: Span,           // 8B，来自 AST/指令
    pub message: String,      // 人读信息（错误路径才构造）
    pub hint: Option<String>, // 修复建议（结构化错误特色）
}
pub type R<T> = Result<T, LfzError>;
```

- **热路径开销 = 0**：成功路径只返回 `Ok(v)`；`LfzError` 的字符串/提示**只在出错时构造**（冷路径）。
- **`?` 传播**：VM 每个 op 用 `?`；`Result<Value, LfzError>` 的返回体量要留意——`Value` 16B + tag，`Result` 可能涨到 24–32B。为压在寄存器友好区间，可让 `Err` 侧为 `Box<LfzError>`（`E` 变一个机器字），错误路径多一次分配无所谓。**实测二选一**。
- **span 效率**：指令**不内联完整 Span**，只存 `u32` 的 span/line 索引，真正的 `Vec<Span>` 放在 `Chunk` 里；异常时按 ip 反查 → 指令保持紧凑。
- **结构化错误四要素**（特色 #4）：`类别/码/行列/hint`；CLI 的 `--json` 直接序列化这四项。

### 5.3 `assert` / `check`

- `assert(cond, msg?)` 失败 → `E_ASSERT` + span + hint（调用点 span）。
- `check(...)` 语义以 spec 为准（可能返回 bool 或同 assert），实现上复用同一错误构造器。
- 二者都在 builtins ABI 内实现，不特殊化 VM。

### 5.4 测试/健壮性辅助

- 可选 **step budget**：VM 主循环计步，超阈值抛 `LfzError`（防止黑盒测试里的死循环挂死 runner）。默认关闭或设很高，由 runner/`--max-steps` 打开。
- 错误必须带位置（红线）；span 来自 core-dev 的 AST 节点，联调时确认位置字段可用。

---

## 6. 可测量性能目标（把"性能优良"变成门禁）

### 6.1 基准集（LFZ 与 CPython 跑**同一算法**）

| # | 基准 | 施压点 | 期望 LFZ 优势来源 |
|---|------|--------|------------------|
| B1 | `fib(30)` 递归 | 函数调用 + int 运算 | slot 帧 + 免箱 int |
| B2 | 紧循环求和 `1..10^7` | 循环 + int 算术 + 比较 | 字节码分发 + 免箱 int |
| B3 | 数组填充 + 求和 `10^6` | 数组索引 + 修改 | 无 GIL、`Vec` 连续存储 |
| B4 | 字符串拼接 `10^4–10^5` | 字符串分配 | 未知，可能持平/略慢 |
| B5 | struct 字段更新 `10^6` | 字段读写 | interned `Sym` |
| B6 | 排序（如快排/归并，N=10^5） | 综合 + 递归 | **与应用选题（排序可视化）同源** |

### 6.2 门禁阈值（建议，可调）

| 类别 | 门槛（LFZ 耗时 / CPython 耗时） | 备注 |
|------|-------------------------------|------|
| 数值/调用/循环（B1·B2·B3·B6） | **≤ 1.0×**（即不慢于 CPython）为 MVP 门禁；目标 0.3–0.5×（快 2–3×） | VM 应达成 |
| 动态字符串/结构体（B4·B5） | **≤ 2.0×**（允许稍慢） | 诚实区间 |
| 总体 | 中位数 **≤ 1.0×**；且 **≥ 60% 的基准 ≤ 0.8×** | 综合判据 |
| 启动时间 | LFZ **显著快于** CPython（免解释器初始化） | 稳拿的叙事亮点 |

### 6.3 如何验证（可复现、可 CI）

1. **构建**：`cargo build --release`（硬性；并记录命令）。
2. **公平性规则**：同机、同算法、无热循环内 IO；每项 warmup 3 次 + 采样 ≥ 7 次，报 **best-of-N + 中位数**；记录 CPython 版本、LFZ 版本、CPU。
3. **门禁文件**：`benchmarks/thresholds.toml`（或 JSON）逐项写最低倍率；一个 `benchmarks/run.py`（或 PowerShell）驱动两侧、输出表格，**任一门禁不达标则退出码非 0**。
4. **正确性先行**：所有基准必须**先在两后端上结果一致**（§7 差分测试），性能才有意义——**绝不用语义妥协换数字**。
5. **归属**：harness 与数字由 perf-engineer 产出；runtime-dev 负责让门禁通过或如实报告瓶颈（不美化、不挑数据）。

> 交付物 4（性能报告）的评分考的是**方法学公平 + 瓶颈诚实**，因此上表把"可复现 + 可判定"做成硬门禁，正好服务该评分项。

---

## 7. 分期实现路线（P3 先行、P6 加层，不返工）

### 7.1 P3（MVP，关键路径）—— 优化树遍历器

1. ★ **先冻结共享骨架**（并入 `interface-contract.md`）：`Value`、环境/帧布局、`LfzError`、builtins ABI、resolver 输出形态。**冻结前 core-dev/runtime-dev 只写测试不写核心。**
2. 实现 **resolver pass**（AST → slot 注解），MVP 的树遍历器**直接吃 slot**（非朴素 HashMap）——一次投入，两处复用。
3. 实现树遍历 evaluator，覆盖**全部 v1 特性**（int/float/string/bool/nil/array/struct/function/闭包、if/while/for、管道脱糖结果、富插值脱糖结果、结构化错误、assert/check）。
4. 全 runtime 单测绿 + hello world 可跑；交付 **P4 CLI / test-engineer / app-dev 可依赖的稳定运行时**。
5. ★ 同时建立 **差分测试骨架**（`LFZ_ENGINE` 开关，为 P6 预留）。

### 7.2 P6（性能层）—— 字节码 VM

1. `compiler`：resolved-AST → `Chunk`（`Instr` 枚举 + 常量池 + span 表）。
2. `vm`：值栈 + 调用帧 + `loop { match code[ip] }`；**复用** `Value` / 环境布局 / `LfzError` / builtins。
3. 反汇编器 + `--dump-bytecode`（调试与答辩材料）。
4. ★ **差分测试**：全部单测/黑盒测试在 `--engine=tree` 与 `--engine=vm` 下结果必须逐字节一致；不一致即语义 bug。
5. CLI 默认切 VM；树遍历器保留为 `--engine=tree`（调试/参照）。
6. 优化轮次（按 profiling 顺序，不预优化）：类型特化 opcode → 方法调用 inline cache → 超指令（superinstruction）→ 指令定宽化（若分发是瓶颈）。

### 7.3 "不返工"的五条纪律（★）

1. **接口先冻结**：`Value`/`Error`/Env/ABI/resolver 在 P3 编码前定稿（走 ADR）。
2. **builtins 面向 ABI 写**，不依赖 evaluator 内部 → 两后端共用。
3. **resolver 共享**（树遍历器与编译器都消费其输出）。
4. **差分测试从 P3 起就有骨架**，P6 直接点亮。
5. **指令集设计为树遍历操作的一一映射超集**，避免语义分叉。

### 7.4 工作量诚实评估

- 双引擎 ≈ 单引擎的 **1.6–1.8×** 工作量；但树遍历器很小（相对 lexer/parser/CLI/黑盒测试），却把**串行关键路径变成"前端/测试/应用可并行的安全路径"**，并白得一个**规格级测试神谕**。
- 在 20（解释器）+ 20（测试）+ 30（应用）权重面前，早交付"已知正确"的 MVP 的价值**高于**省下的那点重复劳动。
- **激进变体**：若团队信心足、且接受风险，可**跳过树遍历器直上 VM**——但**不推荐**给 AI 团队的关键路径（VM 的构造/调试与 Rust 借用检查交互，返工面更大）。
- **保底**：P6 若排期告急，就发布 (b)（已含 resolver 的优化树遍历器）并如实写报告。

### 7.5 必须在 P2 由 language-architect 冻结的语义问题（阻塞项）

以下"按直觉写必翻车"，需 spec 明确后 runtime-dev 才动核心：

1. **复合类型语义**：array/struct 是**引用语义**还是值/拷贝语义？（推荐引用语义）
2. **闭包捕获**：按 **cell 引用**捕获还是按值快照？（推荐按 cell 引用）
3. **int 溢出**：报错（D-007 已示）——确认 `checked_*` 全覆盖。
4. **int/float 混合**：显式提升规则的具体形态（是否/何时自动提升）。
5. **字符串可变性**：不可变（推荐，利于 `Rc` 廉价 clone 与去别名）。
6. **求值顺序**：参数/字段/二元运算的从左到右顺序，写死到 spec（确定性语义）。
7. **条件必须 `bool`**（无 truthiness）——确认无例外（如 `while 1`）。
8. **struct 字段顺序**是否参与相等/打印（影响是否保留插入序，见 §2.5）。

---

## 8. 给用户的最终推荐（一段话）

**建议采用"字节码栈式 VM 为最终引擎、优化树遍历器为先行 MVP 与差分参照"的双后端架构：P3 先用带编译期变量槽（resolved slots）的树遍历器把全部 v1 特性跑通、单测全绿、尽早解阻塞 CLI/黑盒测试/应用，P6 再在同一套 `Value`/环境/错误/builtins/resolver 骨架上加一个栈式字节码 VM 并让 CLI 默认走 VM，同时用"两后端结果逐字节一致"的差分测试保证语义不分叉。** 这样做的理由是：跨语言基准已证明任何树遍历解释器都慢于 CPython，唯独字节码 VM 能稳定追平甚至超越它；而 Rust 的免箱 i64/f64、编译期 slot 寻址、无 GIL、无 per-op 引用计数，正是 CPython 结构上做不到的红利，能把 LFZ 在数值/循环负载上做到 2–3× CPython、动态字符串/结构体负载控制在 2× 以内，并把"性能优良"落成 `--release` + `benchmarks/thresholds.toml` 的可执行门禁而非口号。** 代价是双引擎约 1.6–1.8× 工作量与 Rust 较高的返工率**，故以"接口先冻结、builtins 走统一 ABI、resolver 共享、编译驱动 + 小步提交 + 频繁 `cargo test` + 差分测试"来控制风险；若 P6 排期告急，则发布已含 resolver 的优化树遍历器并如实报告，仍是站得住的交付。** 综上，请在评审时确认三点：① 采纳"VM 为目标、树遍历器为先行"的双后端路线；② 在 P2 冻结 §7.5 的 8 项语义（尤其复合类型引用语义与闭包捕获语义）；③ 将 §6 的性能门禁写入 P6 验收标准。**
