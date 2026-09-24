# runtime-dev — 工作状态
> 最后更新: 2026-09-24 08:46 by runtime-dev

## 当前状态
**P3.11 两条缺陷：1 完成 / 1 阻塞。**
- ✅ **bug-20260924-09（traceback 巨量输出）修复完成** —— `src/cli.rs` 人类可读 traceback 折叠（ADR 裁定 3）。
- ⛔ **bug-20260924-06（`let` 重绑定未受限）阻塞** —— ADR 裁定需新增 `TypeMsg::ImmutableRebind`（`src/error.rs` 属 **core-dev**，任务书明令**不得改**）；已按任务书 ⚠️ **停工**并列出 core-dev 变更清单（见下「阻塞」）。

既有基线：**375 passed / 0 failed / 1 ignored**（原基线 373 passed，未破坏）；`cargo build` **0 warning**。P3.9b（54/54 内置）、P3.8（evaluator 语义定稿）、P3.7、P3.6b 均保持。

## 进行中
- （无）—— 待 core-dev 落地 `error.rs` 变体后接续 bug-06 求值器接线。

## 最近完成
### bug-20260924-09 —— 深递归 traceback 折叠（2026-09-24 08:46）
- **裁定 → 落点**：ADR 裁定 3 / `semantics.md` §8.2 / `interface-contract.md` §10.3 → **`src/cli.rs` `render_error`**（`TracedRun`/`LzError` **不变**）。
- **规则（逐字实现）**：帧总数 `T ≤ 40` → 逐帧原样（浅栈逐字节不变）；`T > 40` → **首 10 帧** → 一行 `  ... 省略 {T−40} 帧 ...`（前缀 **2 空格**）→ **尾 30 帧** → 末行。
- **实现**：新增 `pub const TRACEBACK_HEAD = 10` / `TRACEBACK_TAIL = 30` / `TRACEBACK_FOLD_THRESHOLD = 40`；新增私有 `push_frames`（折叠逻辑）与 `push_trace_frame`（单帧，按 `func_id` 反查显示名）；`render_error` 的运行期分支改调 `push_frames`。**`--json` 未实现（P4）**，故无需处理（裁定要求其 `traceback` 不折叠）。
- **回归用例**：`cli::tests::deep_recursion_traceback_is_folded`（`src/cli.rs:419`，bin 单测）——e2e `lex+parse+eval` 无限递归 → `RecursionError`；断言数据层 `frames ≥ 10000`（不折叠）、渲染层 `  File "` 帧行 **= 40**、含逐字符省略行、末行消息正确。
- **真实 CLI 证据**：`lfz run` 一深递归 `.lfz` → **exit 2**，stderr **123 行**（原 ~30000 行）；`  File "` 帧行 **40**；第 32 行 `  ... 省略 9961 帧 ...`（T=10001）；末行 `RecursionError: 递归深度超限（超过 10000 层）`。

### bug-20260924-06 —— `let` 重绑定：**已定位 + 用例就绪，接线阻塞**（2026-09-24 08:46）
- **裁定 → 落点**：ADR 裁定 1 / `semantics.md` §4.5.2+§8.1 / `interface-contract.md` §10.8 → 判 **`TypeError`**，新增子消息 **`TypeMsg::ImmutableRebind { name }`**，消息 `不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容`，`span` = 赋值目标变量名首字符；触发条件 = 裸 `IDENT` lvalue + op `{=,+=,-=,*=,/=,%=}` + 名字沿词法可见链解析到**显式 `let`**（含捕获的 `let`）。
- **为何停在接线**：`Select-String src\*.rs -Pattern ImmutableRebind` → **无匹配**（该变体当前不存在；仅 spec §10.8 有定义）。任务书要求：需新增该变体 → **停工**并列 core-dev 清单。**未改 `src/error.rs`**。
- **已就绪的回归用例**：
  - 正例 `evaluator::tests::var_rebind_is_still_allowed` —— `var b = 1; b = 2` 仍合法（**ok**，锁定「不触发」边界）。
  - 负例 `evaluator::tests::let_rebind_is_type_error` —— `let a = 1; a = 2` → `TypeError` + 逐字符消息；当前标 **`#[ignore]`**（`--ignored` 运行证实因缺陷 panic：`let 重绑定应报错`）。**接线后移除 `#[ignore]`**。

## 阻塞 / 需要支持
- ⛔ **bug-06 唯一阻塞：请 core-dev 在 `src/error.rs` 落地 `TypeMsg::ImmutableRebind`**（ADR「待执行代码变更清单」#1）：

| 文件 | 负责人 | 变更 | 依据 |
|---|---|---|---|
| `src/error.rs` | **core-dev** | ① `enum TypeMsg`（`:112`）增变体 `ImmutableRebind { name: String }`；② `impl TypeMsg::message()`（`:130`）增分支 `TypeMsg::ImmutableRebind { name } => format!("不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容")`；③ 顶部注释（`:110`）「**6 条**」→「**7 条**」；④ `TypeMsg` 单测增断言（消息逐字符） | 裁定 1 / 契约 §10.8 |

- **衔接计划（core-dev #1 落地后由 runtime-dev 执行，未落地前不写半成品/死代码）**：
  1. `Interp::assign_name`（`src/evaluator.rs:372`，现返回 `bool`）改为三态（`Assigned` / `Immutable` / `NotFound`）：沿 env 链 / `scope.captured` / globals 命中时读 `Env::local_mutable`（**该 API 已存在**，`src/env.rs:129`）。
  2. `exec_assign`（`:710`）无后缀分支（`:718`）match 三态：`Assigned` → `Ok`；`NotFound` → 现 `NameError`；`Immutable` → `Err(type_error(TypeMsg::ImmutableRebind { name: n.clone() }, target.span))`。
  3. **捕获 cell 须携带可变性**：`Closure`/`UserFn.captured`（`src/value.rs:686`）由 `Rc<Vec<(Rc<str>, Cell)>>` 扩为携带 `mutable` 标志；`make_closure`（`:461`）在 `named_indices` 捕获时读 `Env::local_mutable`，以支持**闭包内**判定 `let` 重绑定。
  4. `let`/`var` 定义与 `a[i]=` / `s.k=` 容器路径**不变**（A1）。
  5. 移除 `let_rebind_is_type_error` 的 `#[ignore]`；`cargo test` 应回到全绿 0 ignored。
- **无硬阻塞（bug-09 已闭环）**；承前缺口（`Display` 深结构上限、`StructDef` 模板 `==`、HOF 回调类型口径、内置非一等值）仍待 architect（见下）。

## 下一步计划
- 待 core-dev #1 → 执行上述「衔接计划」，完成 bug-06 并启用负例。
- 待 team-lead 核验本批；建议 release-manager 提交 `fix(p3): fold deep-recursion traceback (bug-20260924-09)`。
- 承前：待 parser lambda 落地补 HOF 成功路径 e2e；按 architect 新补钉调整 §10 契约。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **⚠️ 改 `src/*.rs` 只用 `edit` / `write` 工具**（PowerShell `Get-Content`/`Set-Content` 会按 ANSI/GBK 读写 UTF-8，双重编码 + 吞换行，结构损坏）。
- **warning/error 计数**：`cargo build --tests --message-format=json 2>$null | ... Select-String '"level":"warning"'`（勿 `2>&1`）。
- **看 CLI 中文 stderr 的坑**：PowerShell `2>` 会把 native stderr 当 `NativeCommandError` 记录 + 控制台按 GBK 重编码，`Get-Content`/`Select-String` 会乱码/误判。**正确姿势**：`cmd /c "... 2> file"` 落盘，再用 **Read 工具**（UTF-8）或 `[System.Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes(...))` 读。
- **traceback 折叠分层**（本批定型）：**序列化层**（`TracedRun`/`LzError`）恒完整；**显示层**（`cli::render_error`）折叠。常量 `TRACEBACK_HEAD=10` / `TRACEBACK_TAIL=30` / 阈值 40。改折叠只碰 `render_error`，勿动 `evaluator`。
- **`let` 可变性查得到**：`Env::local_mutable(idx)`（`:129`）已存在（按作用域内 slot 下标）；`Env::names` 为 `(名字, 绝对槽位, 是否可变)`。`assign_name` 沿链命中后即可判可变性——**但**闭包捕获路径的 `CapturedVar` 目前**未携带**可变性，接线时须补。
- **`RefCell` 陷阱**（承 P3.7）：`if let Some(i) = env.borrow().local_index(name) { env.borrow_mut()... }` 会 panic；先 `let idx = ...;`。
- **并发编辑**：`cargo test` 可能捕捉到他人半成品；判据：只看自己模块测试 + 全绿时全量。本轮 core-dev `error.rs` **无** `ImmutableRebind` → bug-06 受限（见上）。
- **模块边界红线**：`error.rs`/`span.rs`/`loader.rs`/`lexer.rs`/`ast.rs`/`parser.rs` 属他人；缺接口 → **停工 + 列清单**，禁自行发明或改他人模块。
