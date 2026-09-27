# core-dev — 工作状态
> 最后更新: 2026-09-27 22:40 by core-dev

## 当前状态
**`AST_DEPTH_LIMIT`（AST 深度上限，收口第 4 类残余「深左偏 AST」）已实现 ✅ 完成** —— `cargo build --all-targets` **0 warning / 0 error**、`cargo test` **482 passed / 0 failed / 0 ignored**（lib 379 + main 48 + cli 28 + test_runner 12 + unit 15）、`cargo run -q -- test` **87/87 exit 0**；`1+1+…` N=9999 → exit 0、N=10000/10001 → exit 2 + `SyntaxError: 表达式嵌套过深（超过 10000 层）`、**旧崩溃点 N=100000 → exit 2（不再是 `-1073741571`）**；回归 `(`×1000 → exit 0、`(`×1001 → exit 2（`嵌套深度超限`）。待 team-lead 核验 + release-manager 提交。
- 依据：`DECISIONS.md` [2026-09-27 22:15] [language-architect] ADR「第 4 类残余裁定：`AST_DEPTH_LIMIT = 10000`」（逐条实现，未自行发明）+ `docs/spec/syntax.md` §3.9 / `semantics.md` §4.5.5 / §8.1 / `interface-contract.md` §10.8 / §10.9 R-S3。

## 进行中
- （无）

## 最近完成
- **第 4 类残余裁定实现：`AST_DEPTH_LIMIT = 10000`**（2026-09-27 22:40，完整启动，TDD-ish：先落单测确认红，再实现）。
  - `src/parser.rs`：新增 `pub const AST_DEPTH_LIMIT: u32 = 10000;` + 自由函数 `check_ast_depth(&Program)`（**显式栈迭代后序遍历**，帧在堆上，**绝不以 AST 深度递归**）+ 借用节点枚举 `DepthNode`（`child(i)->Option` 逐产生式列直接语法子节点）+ `parse()` 末尾调用。度量严格按 §3.9：叶 = 1，`depth(n)=1+max(子)`，程序深度 = 顶层语句最大值（空 = 0），括号分组透明，左结合链 = 链长；**只计 Expr/Stmt/Block/FnDecl/StructDecl/Member/Field/Lvalue/Body，分组不产生节点**。首个 `depth>10000` 的节点 → `syntax(SyntaxMsg::ExprTooDeep, node.span())`（后序首个越限节点，链式构造取链起点）。
  - `src/error.rs`：`SyntaxMsg::ExprTooDeep`（无字段，18→**19**）+ `message()` 逐字符 `表达式嵌套过深（超过 10000 层）`；`class_name()` 仍 `SyntaxError`，**12 类不变**；改 `mod tests`（`syntax_msg_covers_all_nineteen_rows` + 新增 `expr_too_deep_message_is_the_spec_text`）。
  - `tests/unit/ast_depth.rs`（新，7 项）+ `tests/unit/main.rs` 注册 `mod ast_depth;`。
  - **零求值器改动**：`src/evaluator.rs` **本轮未动**；未改 `src/ast.rs`/`src/cli.rs`/`src/builtins.rs`/`docs/spec/**`/`tests/lfz/**`。
  - 证据：字节 `parser.rs 232060`（+9949）、`error.rs 44725`（+1814）、`ast_depth.rs 4116`（新）、`tests/unit/main.rs 460`（+15）；`git diff --numstat`（工作区净）：`parser.rs +440/-17`、`error.rs +117/-4`（**含 22:10 及更早批次既有未提交改动**，见「关键经验」）；`cargo build --all-targets` 0 warning；`cargo test` 482/0/0；`cargo run -q -- test` 87/87。
- 更早：**解析嵌套深度上限 `PARSE_DEPTH_LIMIT = 1000` + 流水线栈契约（R-S1/R-S2）**（2026-09-27 22:10）✅；**P3.11 裁定 1 `TypeMsg::ImmutableRebind`**（09-24）✅；**P3.4a `ast.rs`** ✅、**P3.3b lexer** ✅、**P3.9a `ValueMsg` 2 变体** ✅、**P3.3a lexer** / **P3.1 基座** / **P3.2 加载器** ✅。

## 阻塞 / 需要支持
- **无阻塞**。
- **需团队知悉**：① 本轮实现严格按 architect ADR，**未自行发明规则**；② 口径**分离**——`PARSE_DEPTH_LIMIT=1000`（护 parser 栈）与 `AST_DEPTH_LIMIT=10000`（护求值 / 析构栈）**两个独立常量、不可合并**；③ 第 4 类残余（深左偏 AST）**已收口**，**无需** AST 迭代析构（R-S3 已使析构深度 ≤ 10000）。

## 下一步计划
- 等 verifier 复验本任务（N=9999/10000/10001/100000 夹具 + `(`×1000/1001 回归 + 87 黑盒 + `tests/cli.rs` + `tests/unit/**`）。
- 等 team-lead 派发下一项 v1.1 迭代任务（T11-05 语言补强，若轮到我）。

## 关键经验（写给未来的自己）
- **本机 cargo/rustc 不在默认 PATH**：须先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **⚠️ 绝不用 PowerShell `Set-Content` / `Out-File` 改源码**：PS 5.1 无 BOM 时按 **ANSI** 读写，会把 UTF-8 中文**整文件破坏**。源码改动**只用 `edit`/`write` 工具**。（生成 Temp/ 下的**纯 ASCII 夹具**用 `Set-Content -Encoding ascii` 安全，本轮即如此。）
- **共享工作区有他人未提交改动**：`src/error.rs`/`src/evaluator.rs`/`src/cli.rs`/`src/builtins.rs` 含 21:19–21:32 修复批次与更早的既有改动；`src/parser.rs` 含 22:10 本角色上一任务的既有改动（`PARSE_DEPTH_LIMIT`）。**`git diff --stat` 会把它们一并算入 —— 汇报要注明归属**（本轮 parser 我净增 ≈ +229 行 / +9949 B）。
- **AST 深度检查务必「非递归」**：契约 §10.9 R-S3 明文禁止以 AST 深度递归（检查自身会栈溢出）。正解 = **显式栈迭代后序遍历**（帧存堆），本轮 `check_ast_depth` 即此；`DepthNode::child(i)->Option` 逐产生式枚举直接语法子节点，避免 `Vec` 每帧分配。
- **`&Box<T>` 在枚举构造位可自动 deref 到 `&T`**（deref coercion），故 `DepthNode::Expr(object)`（`object: &Box<Expr>`）可直接编译；`ifx.cond` 用 `&*ifx.cond` 更直白。
- **`(…)` 分组不产生 AST 节点**（`parse_group` 返回内部表达式）→ AST 深度天然透明；这是「解析嵌套大而 AST 深度小」的根源，也是两口径不可合并的证据。
- **`cargo test` 5 个 harness**：lib(379) + main(48) + cli(28) + test_runner(12) + **unit(15)** = **482**。team 文档里的「445/451/456/474」为历史值，**以实测为准**；本波基线 = **474**（新增前），收工 = **482**。
- **`tests/<dir>/main.rs` 会被 Cargo 自动识别为集成测试目标**（无需改 `Cargo.toml`）：`tests/unit/main.rs` → 目标 `unit`，同目录 `*.rs` 作 `mod`（本轮加 `mod ast_depth;`）。
- **深链测试必须在 `on_eval_stack` 大栈上跑**：主测试线程栈（约 2–8 MiB）在深 AST 求值 / 析构时会溢出；返回值只取**标量/错误**，避免深 AST 逃逸到小栈线程**析构**。
- **解析位置 `line` 与 `line_base`**：`parse_src`/`tests/unit` 用 `line_base=1`（模拟 `.lfz`：`#42` 占文件第 1 行），故程序体第 1 行报为**第 2 行** —— 断言 span 时要加 1。
- **PowerShell 抓 `cargo` 输出**：stderr 会被包成 `NativeCommandError`（`error` 关键字可能误命中，非真错）；**以 `Select-String 'warning:'/'error[:[]'` + `$LASTEXITCODE` 为准**，或 `cargo clean -p lfz` 后重编译数 warning。
- **console 中文乱码是显示层（GBK），不代表文件坏**：源文件/JSON/stderr 是 UTF-8；以逐字符单测 / `--json` 字节为准。
- **exit `-1073741571`** = 进程栈溢出 abort（0xC00000FD）；本轮把该路径改为**解析期**受控 `SyntaxError`（exit 2）。
