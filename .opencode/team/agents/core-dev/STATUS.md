# core-dev — 工作状态
> 最后更新: 2026-09-23 22:29 by core-dev

## 当前状态
**P3.4a（`src/ast.rs`：AST 全节点定义）✅ 完成** —— `cargo build` **0 warning**、`cargo test` **154 passed / 0 failed**（+19 新测试）。待 team-lead 核验 + release-manager 提交。
- 依据：`docs/spec/syntax.md` §3–§7（逐产式对照）+ `interface-contract.md` §10.2 / §10.3 / §10.5 / §10.6 / §10.8；`DECISIONS.md` runtime-dev P3.6 ADR（`Dump.scope` 用 `crate::env::ScopeId`）。
- `src/ast.rs`：**151 B → 38638 B**（纯类型定义 + `#[cfg(test)]` 19 测），**不写任何 parser/evaluator 逻辑**。
- **位置承载方式（决定）**：`Spanned<T>` 泛型包装 **+** 辅助节点内嵌 `span` 字段（混合式，理由见模块头注释）。
- core-dev 链下一步：**P3.4b / P3.5 parser**（消费本 AST + 完整 lexer 记号流）。

## 进行中
- （无）

## 最近完成
- **P3.4a `src/ast.rs`**（2026-09-23，先落盘再测试，遵守轻量启动纪律）。
  - **位置包装**：`pub struct Spanned<T> { pub node: T, pub span: Span }`（`new` / `span()`）；`pub type Expr = Spanned<ExprKind>`、`pub type Stmt = Spanned<StmtKind>`。
  - **程序/语句（§3 / §7）**：`Program{span,stmts}`；`StmtKind` 11 变体 —— `Decl{mutable,name,init}`（let/var）、`Assign{target:Lvalue,op:AssignOp,value}`、`FnDecl(FnDecl)`、`StructDecl(StructDecl)`、`If(Expr)`、`While{cond,body}`、`For{var,iter,body}`、`Return(Option<Expr>)`、`Break`、`Continue`、`Dump{scope:ScopeId}`、`Expr(Expr)`；`AssignOp` 6 变体（`=`/`+=`/`-=`/`*=`/`/=`/`%=`）。
  - **辅助节点（内嵌 `span`）**：`Lvalue{span,base:LvalueBase,path:Vec<LvalueSeg>}`（`base` = `Name`/`SelfValue`；`seg` = `Field`/`Index`）、`FnDecl{span,name,params,body:Body}`、`StructDecl{span,name,members}`、`StructMember`（`Method(FnDecl)`/`Field(FieldInit)`）、`FieldInit{span,name,value}`、`Block{span,stmts}`、`Body`（`Block(Block)`/`Expr(Expr)`）。
  - **表达式（§4 / §7）**：`ExprKind` 18 变体 —— 字面量 `Int(i64)`/`Float(f64)`/`Str(String)`/`Bool(bool)`/`Nil`、`Interp(InterpString)`、`Ident(String)`、`SelfRef`、`Array(Vec<Expr>)`、`StructLit(StructLit)`、`Field{object,name}`、`Index{object,index}`、`Call{callee,args}`、`Unary{op,operand}`、`Binary{op,left,right}`、`Logical{op,left,right}`、`If(IfExpr)`、`Lambda(Box<Lambda>)`。
  - **运算符**：`UnaryOp`（`-`/`!`）、`BinaryOp` 11（`+ - * / % < <= > >= == !=`）、`LogicalOp`（`&&`/`||`，短路独立成节点）。
  - **富字符串（§2.8 M6）**：`InterpString{parts:Vec<StrPart>}`；`StrPart::Text(String)` / `StrPart::Expr{expr,format_spec:Option<String>}`（**无 `:` 为 `None`**）。
  - **`if` 结构**：`IfExpr{cond,then_block:Block,else_branch:Option<ElseBranch>}`；`ElseBranch::If(Box<Expr>)` / `Block(Block)`（A4/A27）。
  - **`lambda` 结构**：`Lambda{params,body:Body}`（`fn(..){..}` 与 `(..)=> ..` 两种形式）；`ExprKind::Lambda` 用 `Box` 断开 `Body→Expr→ExprKind→Lambda→Body` 递归环（E0072）。
  - **两点规范说明**：① **无 `Pipe` 节点** —— 依 §4.3 / §10.6，管道解析期脱糖为 `Call`（`pipe_is_desugared_to_call_no_pipe_node` 测试锁定）；② **整数字面量存已解析 `i64`**（§10.8，含 `-9223372036854775808 → i64::MIN` 特判），evaluator 零转换消费。
  - **单测**（`#[cfg(test)]`，19 项）：`spanned_carries_node_and_span`、`literals_cover_all_five_kinds`、`plain_string_vs_interp_string`、`interp_format_spec_is_none_without_colon`、`postfix_call_index_field`、`unary_binary_logical_nodes`、`binary_op_covers_all_eleven`、`array_and_struct_literals`、`if_expression_with_else_if_and_else_block`、`if_without_else_has_none_branch`、`lambda_both_forms`、`all_statement_kinds_constructible`（19 语句，覆盖全部 StmtKind + 全 6 个 AssignOp + 多级 lvalue 链）、`dump_scope_uses_env_scope_id`、`pipe_is_desugared_to_call_no_pipe_node`、`program_aggregates_statements`、`block_and_body_variants`、`field_init_span_and_name_accessible`、`struct_member_both_variants_carry_spans`、`clone_and_partial_eq_derive_work`。
  - 证据：
    - `Get-ChildItem src\ast.rs | Select Name,Length` → `ast.rs 38638`（151 → 38638，**远大于 151**）。
    - `cargo build`（改 mtime 强制重编）→ `BUILD_EXIT=0`、`warning` 计数 **0**（`cargo build 2>&1 | Select-String warning | Measure-Object` → 0）。
    - `cargo test` → `test result: ok. 154 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`，exit 0（135 → 154，**+19**）。
  - 边界纪律：**未改** `src/span.rs`、`src/error.rs`、`src/loader.rs`、`src/lexer.rs`、`src/value.rs`、`src/env.rs`、`src/builtins.rs`、`docs/spec/`、`Cargo.toml`；**未** commit / tag / push。
- **P3.3b lexer 第二批**（2026-09-24）✅、**P3.9a `ValueMsg` 新增 2 变体**（2026-09-24）✅、**P3.3a lexer 第一批**（2026-09-23）✅、**P3.1 基座** / **P3.2 加载器** ✅。

## 阻塞 / 需要支持
- **无阻塞**。
- **跨模块观察（非本模块问题，已自愈）**：本次联调期间 `cargo test` 曾一度因 `src/builtins.rs`（runtime-dev 领地）编译错（`order_kind` / `num_order` / `Kind` 未定义）失败；经确认为 runtime-dev **并行编辑进行中**的瞬时状态，最终 `cargo build` / `cargo test` 双绿（0 warning / 154 pass）。**我未改动 builtins.rs**。
- 历史契约缺口（未闭合块注释）已由 architect ADR 裁定并落地。3 处「契约待确认」（`NotUtf8.span` / `#42` 消费后 `text` 边界 / `ext` 返回形态）仍按最小合理读法实现，不阻塞。

## 下一步计划
- 等待 team-lead 派发 **P3.4b / P3.5 parser**：按 `syntax.md` §7 EBNF 递归下降，消费 lexer 记号流（`StrBegin…StrEnd`、`InterpBegin…FormatSpec?…InterpEnd`）与本 AST 节点；实现 NL 模式栈 `SIG/IGN`、`NO_BRACE_LITERAL`、管道脱糖、`_` 绑定（M4）、`i64::MIN` 特判（§10.8）、`Dump` 节点填 `scope`。
- runtime-dev 消费本 AST 写 P3.7 evaluator（AST 为双方唯一交接物；接口若有歧义 → 升级 language-architect）。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：须先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **递归 AST 必须 `Box`**：`Body → Expr → ExprKind::Lambda → Lambda.body(Body)` 形成无间接递归环 → `E0072`。`ExprKind::Lambda(Box<Lambda>)` 一处 `Box` 断环即可（`Block` 经 `Vec<Stmt>` 已天然有间接）。
- **`match e.node { ExprKind::Lambda(l) => ... }` 会部分移动** `Box`，之后再用 `e.span()` 触发 `E0382`；用 `ref l` 借出。
- **大枚举的体积**：`ExprKind` 内联 `If(IfExpr)`（含 `Block`）使其偏大；当前 ~百字节级，可接受；若 runtime-dev 反馈 `Value`/栈压力再考虑对 `IfExpr`/`Lambda` 加 `Box`（破坏面仅本文件 + parser 构造点）。
- **混合位置承载**：递归大枚举用 `Spanned<T>`、辅助结构内嵌 `span` —— 兼顾「统一入口」与「读起来不双层包装」。
- **PowerShell 抓 `cargo` 输出**：`cargo ... 2>&1 | Tee-Object -Variable out`；cargo 往 stderr 写正常进度会被记成 `NativeCommandError`，**以 `$LASTEXITCODE` 为准**；统计 warning：`$out | Select-String 'warning' | Measure-Object`。
- **共享工作区会撞他人半成品**：`cargo test` 可能因他人（runtime-dev）**正在编辑**的模块瞬时编译失败；先看 mtime、隔一会儿重跑，勿越界修他人文件。
