# runtime-dev — 工作状态
> 最后更新: 2026-09-27 by runtime-dev

## 当前状态
**bug-20260927-01 已修复（`s["k"]()` 未绑定 `self`），待 team-lead / verifier 核验。**
- ✅ **修复**：`src/evaluator.rs` `Interp::eval_call` 新增 `ExprKind::Index` 分支——当被调表达式形如 `recv["k"]`、`recv` 为 struct 且键为 string 时，与 `recv.k(...)` 同一路径调用并绑定 `self = recv`（`semantics.md` §4.5 L51 `s.k ≡ s["k"]` + §3.7）。数组下标 / 非 string 键仍走原通用路径（行为不变）。
- ✅ **TDD 回归单测**：`evaluator::tests::struct_method_via_bracket_index_call_binds_self`（先红后绿）。

基线与证据（本轮实测）：`cargo build` → **0 warning**；`cargo test` → 库 **362 passed / 0 failed / 0 ignored**（+42 +16 +12，合计 **432**，基线 431 +1）；`cargo run --quiet -- test` → **82/82 exit 0**。`git diff --stat` = **仅 `src/evaluator.rs`，99 insertions(+)**。P3 已闭环缺陷（含 bug-06/bug-09）保持不回归。

## 进行中
- （无）—— 待核验本批修复。

## 最近完成
### bug-20260927-01 —— `s["k"]()` 方法调用未绑定 `self`（2026-09-27）
- **规范依据**：`docs/spec/semantics.md` **§4.5 L51**「`s.k ≡ s["k"]` 仍能取到该函数值并调用（`self` 绑定）」+ §3.7；缺陷单 `docs/reports/P9-verification.md` §5（涉及 R-107 / 评分项 1）。
- **复现（修复前）**：`struct Point { fn norm2() => self.x*self.x+self.y*self.y }` → `p.norm2()` 正常 `25`；`p["norm2"]()` → stderr `NameError: 未定义的名字 'self'`（`--json`: `{"ok":false,"error":"NameError",...}`），exit=2。
- **修复（最小、只补该语义）**：`src/evaluator.rs` `eval_call`，在 `Field` 方法分支后加 `Index` 分支：
  - 求 `object` → `recv`、`index` → `iv`（各一次，保持左→右求值）。
  - `matches!(recv, Value::Struct(_)) && iv.as_str().is_some()` → `index_read` 取字段后 `call_func(func, argv, span, Some(recv), false)`（与 `recv.k(...)` 同路径）。
  - 否则（数组下标 / 非 string 键）→ `index_read` + `call_func(..., None, piped)`（原通用口径，含 `piped` 判定）。
- **未触碰**：`docs/spec/**`（规范已写明）、`tests/**`、`docs/guide/**`、README；`error.rs`/`ast.rs`/`parser.rs`/`span.rs`/`lexer.rs`/`loader.rs`、`Cargo.toml`。
- **回归证据（修复后）**：stdout `25\n25`、stderr **0 bytes**、`--json` = `{"ok":true}`、exit=0；边界实跑 `fs[0]()` → `ok:true`、`p["x"]()` → `TypeError: 不可调用：int 不是函数`、`fs["0"]()` → 既有 `TypeError` 不变。

### bug-20260924-06 / -09 —— 保持（前批闭环，本轮无回归）
- bug-06（`let` 重绑定 `TypeError`）：`exec_assign`/`assign_name` 三态接线保持；bug-09（深递归 traceback 折叠）：`cli.rs` 保持。

## 阻塞 / 需要支持
- **无硬阻塞**。本轮修复无需改 `docs/spec/`（规范已写明等价性，实现偏离）。
- 承前缺口（上报 architect，未自行发明）：(1) `Display` 深结构上限无法返回 `RecursionError`；(2) `StructDef` 模板 `==` 仍同一性；(3) HOF 回调类型口径；(4) v1 内置非一等值。

## 下一步计划
- 待 team-lead 核验本批；建议 release-manager 提交 `fix(runtime): bind self for s["k"]() method call (bug-20260927-01)`。
- 建议 verifier 复验并作回归；黑盒端到端用例（`["method"]()`）归 test-engineer（本轮未改 `tests/**`）。
- 承前：待 parser lambda 落地补 HOF 成功路径 e2e；按 architect 新补钉调整 §10 契约。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **⚠️ 改 `src/*.rs` 只用 `edit` / `write` 工具**（PowerShell `Get-Content`/`Set-Content` 会按 ANSI/GBK 读写 UTF-8，双重编码 + 吞换行，结构损坏）。
- **warning/error 计数**：`cargo build --tests --message-format=json 2>$null | Select-String '"level":"warning"'`（勿 `2>&1`）。
- **看 CLI 中文 stderr 的坑**：PowerShell `2>` 会把 native stderr 当 `NativeCommandError` 记录 + 控制台按 GBK 重编码。**正确姿势**：`cmd /c "... 2> file"` 落盘，再用 **Read 工具**（UTF-8）或 `[System.Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes(...))` 读。**`--json` 的 `error` 类名是 ASCII，最适合做跨编码证据**。
- **`.lfz` 文件必须有 `#42\n` 前缀**（loader 强制，首行**恰为** `#42`）；缺前缀 → `CosmosAnswerError`（`line 1`）。
- **cmd 里 `echo EXIT=%ERRORLEVEL%` 不可信**（解析期展开）。取退出码用 PowerShell：`& cargo ... *> $null; "EXIT=$LASTEXITCODE"`。
- **方法绑定是「调用点语法」而非「值属性」**：`self` 绑定发生在 `eval_call` 对 `.字段` / `["字符串键"]` 两种 callee 形态的特殊分支；`let m = p.k; m()` 不绑定（与 `p["k"]` 值语义一致）。新增方法取法务必两条路径同步。
- **`Value::as_str()`** 是判「下标键为 string」的正规入口（`index_read` 同款）；struct 判定用 `matches!(v, Value::Struct(_))`。
- **`RefCell` 陷阱**（承 P3.7）：先 `let idx = ...;` 再 `borrow_mut()`，勿在同一表达式两次借用。
- **模块边界红线**：`error.rs`/`span.rs`/`loader.rs`/`lexer.rs`/`ast.rs`/`parser.rs` 属他人；缺接口 → **停工 + 列清单**，禁自行发明或改他人模块。
