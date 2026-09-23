# runtime-dev — 工作状态
> 最后更新: 2026-09-24 01:30 by runtime-dev

## 当前状态
**P3.7 ✅ 完成**（`src/evaluator.rs` 树遍历求值器核心：表达式 / 语句 / 控制流 / 函数与闭包 / 调用）。
待 team-lead 核验 + release-manager 提交 `feat(p3): tree-walking evaluator core`。

- **公开 ABI**（tooling-dev / P3.8 消费）：
  - `pub fn eval_module(program: &ast::Program) -> R<Value>` —— 求值编译单元，返回最后一条语句值。
  - `pub fn run(program: &ast::Program) -> R<()>` —— 执行并丢弃最终值。
  - （AST 类型名为 `Program`，任务书所称 `Module` 即此。）
- **跨模块扩展**（本批新增，旧 ABI 不变）：
  - `value.rs`：`Closure` 增 `user: Option<Rc<UserFn>>`；`pub struct UserFn { params, body, captured: Rc<Vec<(Rc<str>, Cell)>>, func_id, self_val }`；`StructDef` 扩为 `{ name, fields: Vec<(Rc<str>, Expr)>, methods: Vec<(Rc<str>, Rc<Closure>)> }`；新增 `Value::struct_template(name, fields, methods)`。`Closure::named/anonymous`、`Value::struct_def` **签名不变**（占位载荷）。
  - `env.rs`：`Env` 增 `names: Vec<(String, abs_slot, bool)>` + `define_named` / `local_index` / `local_mutable` / `get_local` / `set_local` / `capture_local` / `named_indices`。既有 `define`/`get`/`set`/`capture` 行为**不变**。
- 既有基线：P3.6b `value.rs` 值语义（A6 + 全序）、P3.6a `value.rs`/`env.rs`、P3.9a `builtins.rs`（47 非高阶内置）+ 契约缺口闭合均完成。

## 进行中
- （无）

## 最近完成
- **P3.7 `src/evaluator.rs` 核心**（2026-09-24 01:30）
  - **求值**：全部表达式（字面量/标识符/`self`/数组/struct/字段/下标/调用/一元/二元/逻辑/插值（含格式说明符子集）/lambda）。
  - **执行**：`let`/`var`、赋值（`=`/`+=`/`-=`/`*=`/`/=`/`%=`；`a[i]=v`/`s.k=v` **原地修改** A1；嵌套路径）、`if`、`while`、`for…in`（快照）、`break`/`continue`/`return`、命名 `fn`（递归）、`struct` 模板 + 实例化、块作用域。
  - **名字解析 = 运行时查名**（ADR [2026-09-24 01:30]）：`Env` per-scope 名字表 + 沿词法链内→外查名后直读本作用域槽位；查名顺序 调用帧/块（不含顶层）→ 捕获 cell → globals。**顶层不参与 cell 捕获**（§4.5.3）。
  - **A2 捕获**：`Env::capture_local` 原地升级为共享 `Cell`；**循环每轮独立 cell**（`for`/`while` 每轮新子作用域）。
  - **调用**：用户函数（新帧 + 参数绑定 + 返回）、内置（`Span` 取**调用点**）、方法（`recv.m()` 绑定 `self`）、管道（parser 脱糖为 `Call`，运行时零开销）。
  - **错误**：`R<T>` 冒泡，位置取节点 / 调用点 `Span`；测试锁定**非 `Span::START`**。
  - 证据：
    - `Get-ChildItem src\evaluator.rs` → **`evaluator.rs  79980`**（原 182 B；合法 UTF-8）。
    - `cargo build --message-format=json 2>$null | Select-String '"level":"warning"'` → **0 warning**。
    - `cargo test` → **`195 passed; 0 failed`**（新增 `evaluator::tests::*` **26**）。
- **P3.6b `src/value.rs` 值语义辅助（A6 + 全序）唯一共享实现**（2026-09-24 00:40）：`deep_eq`/`total_cmp`/`order_kind`；`builtins.rs` 复用、删内联比较器。
- **P3.9a 内置函数表（非高阶）**（2026-09-23）：`builtins.rs` 47 个非高阶内置 + 稳定 ABI；7 个 HOF 留 P3.9b。
- **P3.6a 值模型 + 作用域**（2026-09-23）：`value.rs` + `env.rs`。

## 阻塞 / 需要支持
- **无硬阻塞**。
- **规范缺口（已上报，未自行发明；本实现取保守口径）**：
  1. **`let` 重绑定的错误类未定义**：§4.5.2 称 `a = []` 对 `let`「非法」，但 §8.1 未给错误类/消息，且 `error.rs` 对本角色**只读**。本批**存储** `mutable` 标志但**暂不强制**（`let`/`var` 当前行为相同）；待 language-architect 指定错误类后于 P3.8 落地。
  2. **`;;`（`Dump`）输出**：依赖 `ScopeDebug`/`ScopeId` 可见链（`def_scope` 现为空链占位）。**留 P3.8**。
  3. **`RecursionError`（§4.5.5 帧深 10000）与 `TraceFrame`/traceback 组装**：`func_id` 仅分配不复用；**留 P3.8**。当前**无限递归无保护**（会栈溢出）。
  4. **§4.5.5 深结构 10000 层上限**：`deep_eq`/`Display`/求值仍为递归实现，未施加上限（承 P3.6b 遗留，建议随 P3.8 收口）。
  5. **`StructDef`（模板）的 `==`**：§4.5.9 未列该类，取**同一性**（承 P3.6b）。

## 下一步计划
- **P3.8**：`Dump`/`;;` 输出（`ScopeDebug` 可见链 + `def_scope` 落地）、`check` 专项接线、`RecursionError`（帧深上限）、traceback 组装（`TraceFrame` + `func_id` 反查名字）、`let` 不可变性（待错误类）。
- **P3.9b**：7 个高阶内置（`map`/`filter`/`reduce`/`sortBy`/`minBy`/`maxBy`/`each`）——需将本批 `Interp::call_user` 的「调用用户函数」能力提升为**可复用 ABI**。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **warning/error 计数**：`$out = cargo build --tests --message-format=json 2>$null; (($out|Select-String '"level":"warning"')).Count`（`2>$null` 丢进度；**勿** `2>&1`）。
- **⚠️ 绝不用 PowerShell `Get-Content -Raw` + `Set-Content` 改 `src/*.rs`**：PS 5.1 默认 ANSI 编码，会把 UTF-8 中文注释**改坏成非法 UTF-8**（rustc 报 invalid byte）。改文件**只用 `edit`/`write` 工具**（UTF-8 安全）；批量替换若必须用 shell，先确认无损再用 `edit` 重写。
- **`RefCell` 陷阱**：`if let Some(i) = env.borrow().local_index(name) { env.borrow_mut()... }` 会 panic（scrutinee 的不可变借用存活到 `if let` 块尾）。先 `let idx = env.borrow().local_index(name);` 再 `if let Some(i) = idx { env.borrow_mut()... }`。
- **A2 递归自引用**：命名 `fn` 必须「**先 `nil` 预绑定其名 → 建闭包（捕获自身名 cell）→ 写回闭包**」，否则函数体内查不到自己。
- **运行时常量 `Span::START`**：错误构造函数别图省事写死 `Span::START`——那会违反「运行时错误必须带位置」红线；helper 须显式接收节点 `span`。
- **AST 类型名是 `Program` 不是 `Module`**；`Expr`/`Stmt` = `Spanned<ExprKind/StmtKind>`，可抛错节点用 `.span` 取位置。
- **并发编辑**：`cargo test` 可能捕捉到 core-dev 半成品（如 `parser.rs`/`lexer.rs`）状态；判据：只看自己模块测试 + 全绿时全量。
