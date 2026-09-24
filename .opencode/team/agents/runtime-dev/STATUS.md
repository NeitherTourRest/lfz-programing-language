# runtime-dev — 工作状态
> 最后更新: 2026-09-24 08:54 by runtime-dev

## 当前状态
**P3.11 两条缺陷：全部闭环（bug-06 + bug-09）。**
- ✅ **bug-20260924-06（`let` 重绑定未受限）完成** —— `src/evaluator.rs` `exec_assign` / `assign_name` 接线 `TypeMsg::ImmutableRebind`（ADR 裁定 1），并摘除阻塞占位 `#[ignore]`。
- ✅ **bug-20260924-09（traceback 巨量输出）保持修复** —— `src/cli.rs` 折叠，本轮**未回归**（深递归 stderr 123 行 / 帧行 40 / 含省略行）。

既有基线：**lib 361 passed / 0 failed / 0 ignored**（上一轮 360 passed / **1 ignored** → ignored 数 **1 → 0**）；**bin 9 passed**；**`tests/cli.rs` 7 passed**；`cargo build` **0 warning**。P3.9b（54/54 内置）、P3.8（evaluator 语义定稿）、P3.7、P3.6b 均保持。

## 进行中
- （无）—— P3.11 两条缺陷已全部闭环，待 team-lead 核验。

## 最近完成
### bug-20260924-06 —— `let` 重绑定检查接线（2026-09-24 08:54）
- **前提**：core-dev 已在 `src/error.rs` 落地 `TypeMsg::ImmutableRebind { name: String }`（消息 `不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容`，`class_name()` 仍 `TypeError`）。**未改 `error.rs`**。
- **落点（ADR 裁定 1 #2 / `semantics.md` §4.5.2 / 契约 §10.8）**：触发条件 = 裸 `IDENT` lvalue + op `{=,+=,-=,*=,/=,%=}` + 名字沿词法可见链解析到**不可变的显式 `let`**（含**捕获的 `let`**）→ `TypeError`，`span = target.span`。
- **实现**：
  1. `src/evaluator.rs` 新增模块级私有 `enum AssignOutcome { Assigned, Immutable, NotFound }`；`Interp::assign_name` 返回类型 `bool → AssignOutcome`；沿 **env 链 → 捕获 cell → globals** 内→外解析，命中 `Env::local_mutable(i) == Some(false)`（或捕获 `mutable == false`）→ `Immutable`（**不写入**）。
  2. `exec_assign`（无后缀分支）match 三态：`Assigned` → `Ok`；`NotFound` → 既有 `NameError`（`self` 仍 `NotFound`，行为不变）；`Immutable` → `type_error(TypeMsg::ImmutableRebind { name }, target.span)`。
  3. **捕获 cell 携带可变性**：`src/value.rs` 新增 `pub type CapturedVar = (Rc<str>, Cell, bool)`，`UserFn.captured` 改型；`Scope.captured` 同步；`make_closure` 捕获时读 `Env::local_mutable(idx)`（`unwrap_or(true)`）写入并随继承传递 → 支持闭包内判定。
  4. `let`/`var` 定义与 `a[i]=` / `s.k=` 容器路径**不变**（A1）。
- **回归用例**：`evaluator::tests::let_rebind_is_type_error`（摘除 `#[ignore]`，已运行通过）——① 顶层 `let a = 1; a = 2` → `TypeError` + 逐字符消息；② 闭包捕获 `let x`（`outer` 帧）内 `x = 2` → 同消息。正例 `evaluator::tests::var_rebind_is_still_allowed`（`var b = 1; b = 2` → `2`）保持通过；`closure_counter_shares_cell`（捕获 `var n` 重绑定）保持通过（锁定「`var` 捕获仍可变」）。
- **真实 CLI 证据**：`let a = 1; a = 2` → **EXIT=2** + `TypeError: 不能重新赋值 let 变量 'a'；…`；闭包捕获版 → **EXIT=2**；`var b` 版 → **EXIT=0**、stdout `2`。

### bug-20260924-09 —— 深递归 traceback 折叠（2026-09-24 08:46，本轮回归确认）
- **落点**：`semantics.md` §8.2 / 契约 §10.3 → `src/cli.rs` `render_error`（`TracedRun`/`LzError` 不变）。
- **规则**：帧总数 `T ≤ 40` → 逐帧原样；`T > 40` → 首 10 帧 → 一行 `  ... 省略 {T−40} 帧 ...`（前缀 2 空格）→ 尾 30 帧 → 末行。常量 `TRACEBACK_HEAD=10` / `TRACEBACK_TAIL=30` / 阈值 40。
- **本轮回归证据**：`lfz run` 深递归 `.lfz` → **EXIT=2**，stderr **123 行**（数据层完整），`  File "` 帧行 **40**，第 32 行 `  ... 省略 9961 帧 ...`（T=10001），末行 `RecursionError: 递归深度超限（超过 10000 层）`。**未改动 `cli.rs`，未回归**。

## 阻塞 / 需要支持
- **无硬阻塞**（bug-06 / bug-09 均已闭环）。
- 承前缺口（上报 architect，未自行发明）：(1) `Display` 深结构上限无法返回 `RecursionError`；(2) `StructDef` 模板 `==` 仍同一性；(3) HOF 回调类型口径；(4) v1 内置非一等值。

## 下一步计划
- 待 team-lead 核验本批；建议 release-manager 提交 `fix(p3): enforce let rebind TypeError (bug-20260924-06)`。
- 承前：待 parser lambda 落地补 HOF 成功路径 e2e；按 architect 新补钉调整 §10 契约。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **⚠️ 改 `src/*.rs` 只用 `edit` / `write` 工具**（PowerShell `Get-Content`/`Set-Content` 会按 ANSI/GBK 读写 UTF-8，双重编码 + 吞换行，结构损坏）。
- **warning/error 计数**：`cargo build --tests --message-format=json 2>$null | Select-String '"level":"warning"'`（勿 `2>&1`）。
- **看 CLI 中文 stderr 的坑**：PowerShell `2>` 会把 native stderr 当 `NativeCommandError` 记录 + 控制台按 GBK 重编码。**正确姿势**：`cmd /c "... 2> file"` 落盘，再用 **Read 工具**（UTF-8）或 `[System.Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes(...))` 读。
- **`.lfz` 文件必须有 `#42\n` 前缀**（loader 强制，首行**恰为** `#42`）；缺前缀 → `CosmosAnswerError`（`line 1`）。写 CLI 复现脚本时勿忘。
- **`Select-String` 会吞行首空白**，判断源码缩进请用 `(git show HEAD:file)` + `Where-Object { $_ -match ... }` 或 Read 工具；**`edit` 工具对行首空白可能宽松匹配**，替换整段 `use` 时留意别误删缩进。
- **`let` 可变性三态**（本批定型）：`assign_name` 返回 `AssignOutcome`；env 链 / 捕获 cell / globals 三处都要查 `mutable`；env 链用 `Env::local_mutable(idx)`，捕获处用 `CapturedVar` 第三元 `bool`。**`self` 保持 `NotFound`**（→ `NameError`），不改契约。
- **`RefCell` 陷阱**（承 P3.7）：`if let Some(i) = env.borrow().local_index(name) { env.borrow_mut()... }` 会 panic；先 `let idx = ...;`。本批 `local_mutable` 与 `set_local` 必须分两次借用。
- **traceback 折叠分层**：**序列化层**（`TracedRun`/`LzError`）恒完整；**显示层**（`cli::render_error`）折叠。改折叠只碰 `render_error`，勿动 `evaluator`。
- **模块边界红线**：`error.rs`/`span.rs`/`loader.rs`/`lexer.rs`/`ast.rs`/`parser.rs` 属他人；缺接口 → **停工 + 列清单**，禁自行发明或改他人模块。
