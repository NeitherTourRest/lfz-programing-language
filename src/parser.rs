//! 语法分析器（递归下降）—— 第六批：管道脱糖 / `;;` dump / 富字符串插值 / i64 边界
//! （前五批：语句、表达式、控制流、声明与函数字面量）。
//!
//! 契约：`docs/spec/interface-contract.md` §10.6；语法：`docs/spec/syntax.md` §3.1 / §3.2 / §3.4 / §3.5 / §4.1 / §7。
//!
//! # 已实现（本批 + 前批；其余构造在后续批次实现，当前一律报 `UnexpectedToken`）
//!
//! - 换行模式栈（§3.2）：`(` / `[` / 字面量与声明成员表 `{` 内 `NEWLINE` 忽略（`IGN`），
//!   语句层生效（`SIG`）。
//! - 语句：`let` / `var` 声明、**赋值语句**（`assign_stmt`：`= += -= *= /= %=`，
//!   目标为 `IDENT` / `self` + `. 字段` / `[ 下标 ]` 链，A21）、表达式语句、
//!   块 `{ ... }`、程序（`Program`）本身。
//! - 控制流：`if` / `else` / `else if` 链（条件走 §3.4 NO_BRACE_LITERAL、
//!   §3.5 块前 / else 前换行、A4 悬挂 else 绑定最近 if）、`while`（§7 `while_stmt`）、
//!   `for IDENT in expr`（§7 `for_stmt`）、`break` / `continue`（循环外 →
//!   `BreakContinueOutsideLoop`）、`return [expr]`（A19 行尾 = 返回 nil；函数外 →
//!   `ReturnOutsideFunction`）。
//! - **声明与函数字面量（本批）**：命名函数声明 `fn name(params) body`（§7 `fn_decl`；
//!   头部与块间允许换行，§3.3 / §3.5）、结构体模板 `struct Name { … }`（§7
//!   `struct_decl`；成员表 `{ … }` 内换行忽略；成员 = 字段 `IDENT : expr` 或方法
//!   `fn …`，逗号必填、尾逗号可选）、函数字面量（§7 `lambda`）两种形式
//!   `fn (params) body` 与 `(params) => (block | expression)`（A20 前瞻 `=>` 判定，
//!   未命中回退为括号分组；M3 `=> {` 恒为块；A26 lambda 不作后缀基）、
//!   表达式位置的 `self`（§7 `primary`）。函数体 / lambda 体解析期间 `fn_depth` +1，
//!   `return` 由此合法。
//! - 表达式：字面量（Int / Float / 纯字符串 / true / false / nil / self）、标识符、
//!   括号分组 `( expr )`、一元 `-` / `!`（§4.1 级别 2）、
//!   后缀链（调用 `f(a, …)` / 索引 `xs[i]` / 字段 `s.k`，可任意链式，§4.1 级别 1）、
//!   数组字面量 `[a, b, c]`、struct 字面量 `{ k: v }` 与 `Name { k: v }`（§7）、
//!   以及 §4.1 优先级表的二元层（全部左结合）：`* / %`（级别 3）→ `+ -`（级别 4）→
//!   比较 `< <= > >=`（级别 6）→ 相等 `== !=`（级别 7）→ 短路逻辑 `&&`（级别 8）→
//!   `||`（级别 9）。
//! - 错误：`UnexpectedToken` / `IncompleteExpr` / `LoneSemicolon` / `InvalidAssignTarget` /
//!   `BreakContinueOutsideLoop` / `ReturnOutsideFunction`；语句终结检查
//!   （§3.2 同逻辑行两条语句）报 `TwoStatements`。
//!
//! # 本批实现（第六批）
//!
//! - **管道脱糖**（§4.1 级别 5 / §4.3）：`cmp_expr` 层插入 `pipe_expr = add_expr { "|>" pipe_rhs }`。
//!   `L |> F(a1,…,an)` 脱糖为 `Call`（data-last：无 `_` 时 `L` 追加为末参；恰一个 `_` 原位替换；
//!   ≥2 个 `_` → `PipeMultiplePlaceholder`）；右侧非调用（函数名 / 方法 / lambda）→ `R(L)`。
//!   `_` 仅合法于管道 RHS 的调用实参位（任意嵌套深度），其余位置（含 lambda 体、`let _ = …`）
//!   → `PlaceholderPosition`（§4.3 规则 4/6，M4）。
//! - **`;;` → `Dump` 节点**（§10.5 / §10.6）：`StmtKind::Dump { scope }`，`scope` 填哨兵
//!   `ScopeId(0)`；evaluator 执行时**忽略**该字段、以运行时当前 scope 为准（见汇报的协调点）。
//! - **富字符串插值**（§2.8）：`StrBegin / Text / InterpBegin / …表达式… / [FormatSpec] /
//!   InterpEnd / … / StrEnd` → `ExprKind::Interp(InterpString)`，`format_spec: Option<String>`
//!   （无 `:` 为 `None`，`:` 后可空）。纯文本无插值仍产出 `ExprKind::Str`。
//! - **`i64::MIN` 字面量**（§10.8 B12）：`-` 紧邻 INT 且其正值为 `2^63` → `Int(i64::MIN)`
//!   （不产生 `Unary`）；十进制超出 `i64` / radix 形式超出 `u64` → `IntegerOutOfRange`。
//!
//! # 本批已知简化
//!
//! 1. 语句起始处的 `{ ... }` 仍按**裸块语句**解析并内联进外层语句序列
//!    （AST 的 `StmtKind` 无 Block 变体）；§3.3 / A9「语句首 `{` 恒为匿名
//!    struct 字面量」留待语句形态补齐批次一并处理。
//! 2. `if` 仅支持**语句形态**（`StmtKind::If` 包裹 `if_expr`）；表达式位置的
//!    `if_expr`（§7 `unary` 层）属后续批次。
//! 3. 循环 / 函数体深度由 `loop_depth` / `fn_depth` 计数（本批起 `fn` 声明、
//!    lambda 体真正 +1）。
//! 4. （已由第六批实现：`;;`→`Dump`、管道 `|>` 脱糖、富字符串插值、
//!    `i64::MIN` / `IntegerOutOfRange` 特判，见上「本批实现」。）
//! 5. `self` 与形参重名不作语法级限制（§7 无规则）；`self` 归属方法体的语义校验
//!    在求值期。

use crate::ast::*;
use crate::env::ScopeId;
use crate::error::{syntax, LzError, R, SyntaxMsg};
use crate::lexer::{Token, TokenKind};
use crate::span::Span;

/// 换行模式（`syntax.md` §3.2）：决定 `NEWLINE` 是否有效。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum NlMode {
    /// `NEWLINE` 有效：终结语句（程序顶层 / 块内）。
    Sig,
    /// `NEWLINE` 忽略：当空白跳过（`( … )` / `[ … ]` 内）。
    Ign,
}

/// 管道上下文（§4.3）：决定 `_` 占位符的合法性。
#[derive(Clone, Debug)]
enum PipeCtx {
    /// 正在解析某个 `|>` 的 RHS：记录已见 `_` 与当前调用实参位嵌套深度。
    Rhs {
        /// 已见的 `_` 位置（至多一个，§4.3 规则 1）。
        placeholder: Option<Span>,
        /// 调用实参位嵌套深度（`_` 仅在此 >0 时合法，§4.3 规则 4）。
        call_args: u32,
    },
    /// 函数 / lambda 体：`_` 不得穿 λ 体（§4.3 规则 6，M4）。
    Barrier,
}

impl PipeCtx {
    /// 取出本帧已记录的 `_` 位置（`Barrier` 帧恒为 `None`）。
    fn placeholder(&self) -> Option<Span> {
        match self {
            PipeCtx::Rhs { placeholder, .. } => *placeholder,
            PipeCtx::Barrier => None,
        }
    }
}

/// `_` 占位符在当前位置的合法性判定结果。
enum PlaceholderState {
    /// 管道 RHS 调用实参位且尚无占位符：合法。
    Legal,
    /// 管道 RHS 已有一个占位符：第二个 `_` → `PipeMultiplePlaceholder`。
    Multiple,
    /// 其它位置（或 lambda 体内）：非法 → `PlaceholderPosition`。
    Illegal,
}

/// 递归下降解析器：`tokens` + 游标 + 换行模式栈 + 控制流上下文。
pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    /// 换行模式栈（§3.2），栈顶 = 当前模式；初始 `[Sig]`。
    nl_stack: Vec<NlMode>,
    /// 循环嵌套深度（`while` / `for` 体 +1）：`break` / `continue` 合法性判定。
    loop_depth: u32,
    /// 函数嵌套深度（`fn` 声明体 / lambda 体 +1，构造属下一批）：`return` 合法性判定。
    fn_depth: u32,
    /// 正在解析控制流头（if / while 条件、for 可迭代位）的表达式（§3.4）。
    cond_restrict: bool,
    /// 控制流头表达式中 `(` / `[` 嵌套深度：`>0` 时 NO_BRACE_LITERAL 临时解除（§3.4）。
    group_depth: u32,
    /// 管道上下文栈（§4.3）：`|>` RHS 压 `Rhs`，函数体压 `Barrier`；`_` 合法性据此判定。
    pipe_stack: Vec<PipeCtx>,
}

/// 解析记号流为一个程序（`Program` = 顶层语句序列；即任务书所称 module）。
///
/// 记号流须以 `Eof` 结尾（`lexer::lex` 保证）。
#[must_use]
pub fn parse(tokens: &[Token]) -> R<Program> {
    Parser::new(tokens).parse_program()
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        debug_assert!(
            matches!(tokens.last(), Some(t) if t.kind == TokenKind::Eof),
            "记号流必须以 Eof 结尾"
        );
        Self {
            tokens,
            pos: 0,
            nl_stack: vec![NlMode::Sig],
            loop_depth: 0,
            fn_depth: 0,
            cond_restrict: false,
            group_depth: 0,
            pipe_stack: Vec::new(),
        }
    }

    // ------------------------------------------------------------------
    // 基本原语
    // ------------------------------------------------------------------

    /// 当前 token 的种类（不消费）。
    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    /// 当前 token 的 `Span`。
    fn peek_span(&self) -> Span {
        self.tokens[self.pos].span
    }

    /// 下一个 token 的种类（不消费；当前已是末位 `Eof` 时为 `None`）。
    fn peek_next(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos + 1).map(|t| &t.kind)
    }

    /// 消费当前 token（不越过末尾 `Eof`）。
    fn bump(&mut self) {
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
    }

    /// 当前换行模式。
    fn mode(&self) -> NlMode {
        *self.nl_stack.last().expect("模式栈非空")
    }

    /// `IGN` 模式下跳过 `NEWLINE`（括号内换行当空白）。
    fn skip_ign_newlines(&mut self) {
        if self.mode() == NlMode::Ign {
            while *self.peek() == TokenKind::Newline {
                self.pos += 1;
            }
        }
    }

    /// 无条件跳过 `NEWLINE`（语句序列的语句分隔）。
    fn skip_newlines(&mut self) {
        while *self.peek() == TokenKind::Newline {
            self.pos += 1;
        }
    }

    /// 期待 `kind`；命中则消费并返回其 `Span`，否则报 `UnexpectedToken`。
    fn expect(&mut self, kind: &TokenKind) -> R<Span> {
        let span = self.peek_span();
        if self.peek() == kind {
            self.bump();
            Ok(span)
        } else {
            Err(syntax(
                SyntaxMsg::UnexpectedToken {
                    expected: token_desc(kind),
                    got: token_desc(self.peek()),
                },
                span,
            ))
        }
    }

    /// 在当前 token 处报 `UnexpectedToken`（期待 `expected`）。
    fn unexpected(&self, expected: &str) -> Box<LzError> {
        syntax(
            SyntaxMsg::UnexpectedToken {
                expected: expected.to_string(),
                got: token_desc(self.peek()),
            },
            self.peek_span(),
        )
    }

    // ------------------------------------------------------------------
    // 程序 / 语句序列 / 语句
    // ------------------------------------------------------------------

    /// `program = { NEWLINE | statement } , EOF`（§7）。
    fn parse_program(&mut self) -> R<Program> {
        let span = self.peek_span();
        let stmts = self.parse_stmt_seq()?;
        if *self.peek() != TokenKind::Eof {
            // 顶层不应残留 `}`。
            return Err(self.unexpected("语句"));
        }
        Ok(Program { span, stmts })
    }

    /// 语句序列（程序体 / 块体），直到 `}` 或 `Eof`（两者均不消费）。
    fn parse_stmt_seq(&mut self) -> R<Vec<Stmt>> {
        let mut stmts = Vec::new();
        loop {
            match self.peek().clone() {
                TokenKind::Eof | TokenKind::RBrace => break,
                TokenKind::LBrace => {
                    // 裸块语句（本批简化）：块内语句内联进外层语句序列。
                    let block = self.parse_block()?;
                    stmts.extend(block.stmts);
                }
                _ => stmts.push(self.parse_stmt()?),
            }
            // 语句终结检查（§3.2）：下个有效记号必须是 NEWLINE / `}` / EOF。
            if *self.peek() == TokenKind::Newline {
                self.skip_newlines();
            } else if !matches!(self.peek(), TokenKind::Eof | TokenKind::RBrace) {
                return Err(syntax(SyntaxMsg::TwoStatements, self.peek_span()));
            }
        }
        Ok(stmts)
    }

    /// 解析一条语句（`let` / `var` 声明、控制流、赋值、表达式语句；`;` → `LoneSemicolon`）。
    fn parse_stmt(&mut self) -> R<Stmt> {
        let span = self.peek_span();
        match self.peek().clone() {
            TokenKind::KwLet | TokenKind::KwVar => self.parse_decl(span),
            TokenKind::KwIf => self.parse_if_stmt(span),
            TokenKind::KwWhile => self.parse_while(span),
            TokenKind::KwFor => self.parse_for(span),
            TokenKind::KwReturn => self.parse_return(span),
            TokenKind::KwBreak | TokenKind::KwContinue => self.parse_break_continue(span),
            // `fn` + IDENT → 命名函数声明；`fn` + `(` → 函数字面量表达式语句（走表达式路径）。
            TokenKind::KwFn if matches!(self.peek_next(), Some(TokenKind::Ident(_))) => {
                self.parse_fn_decl(span)
            }
            TokenKind::KwStruct => self.parse_struct_decl(span),
            // `;;` → `Dump` 节点（§10.5 / §10.6）：scope 填哨兵 `ScopeId(0)`，
            // evaluator 执行时忽略该字段、以运行时当前作用域为准。
            TokenKind::Dump => {
                self.bump();
                Ok(Spanned::new(StmtKind::Dump { scope: ScopeId(0) }, span))
            }
            TokenKind::Semi => Err(syntax(SyntaxMsg::LoneSemicolon, span)),
            _ => {
                // `assign_stmt = lvalue , assign_op , expression`（§7；A21 先试 lvalue 头部）。
                if let Some((target, op)) = self.try_parse_assign_head() {
                    let value = self.parse_expr()?;
                    return Ok(Spanned::new(StmtKind::Assign { target, op, value }, span));
                }
                let expr = self.parse_expr()?;
                // A21：表达式后跟 assign_op 而左侧不是 lvalue → `InvalidAssignTarget`。
                if assign_op_of(self.peek()).is_some() {
                    return Err(syntax(SyntaxMsg::InvalidAssignTarget, expr.span));
                }
                Ok(Spanned::new(StmtKind::Expr(expr), span))
            }
        }
    }

    /// 试探赋值语句的「lvalue + assign_op」头部（§7 `assign_stmt` / `lvalue`）。
    ///
    /// 基座须为 `IDENT` / `self`，路径只允许 `. IDENT` / `[ expression ]`；
    /// lvalue 解析完且**紧跟 `assign_op`** 才提交（`Some`）。否则回滚游标与换行
    /// 模式栈并返回 `None`，由调用方按普通表达式语句继续 —— 回滚保证
    /// `Point { … }`、`f(x)`、`a.b.c` 等以 IDENT 开头的表达式语句不受影响。
    ///
    /// 路径节（`LvalueSeg`）的 `span` 取 `.` / `[` 的精确位置（ast 契约）。
    /// 内层 `[ expression ]` 的解析错误不在此上报：回滚后由完整表达式解析
    /// 在同位置报出同错（两遍解析的记号流一致）。
    fn try_parse_assign_head(&mut self) -> Option<(Lvalue, AssignOp)> {
        let start_pos = self.pos;
        let start_stack = self.nl_stack.len();
        let span = self.peek_span();
        let base = match self.peek().clone() {
            TokenKind::Ident(name) => {
                self.bump();
                LvalueBase::Name(name)
            }
            TokenKind::KwSelf => {
                self.bump();
                LvalueBase::SelfValue
            }
            _ => return None,
        };
        let mut path = Vec::new();
        loop {
            self.skip_ign_newlines();
            match self.peek().clone() {
                TokenKind::Dot => {
                    let seg_span = self.peek_span();
                    self.bump();
                    match self.peek().clone() {
                        TokenKind::Ident(name) => {
                            self.bump();
                            path.push(LvalueSeg {
                                span: seg_span,
                                kind: LvalueSegKind::Field(name),
                            });
                        }
                        _ => {
                            self.reset(start_pos, start_stack);
                            return None;
                        }
                    }
                }
                TokenKind::LBracket => {
                    let seg_span = self.peek_span();
                    self.bump();
                    self.nl_stack.push(NlMode::Ign);
                    let inner = self.parse_expr();
                    self.nl_stack.pop();
                    match inner {
                        Ok(inner) => {
                            self.skip_ign_newlines();
                            if *self.peek() == TokenKind::RBracket {
                                self.bump();
                                path.push(LvalueSeg {
                                    span: seg_span,
                                    kind: LvalueSegKind::Index(inner),
                                });
                            } else {
                                self.reset(start_pos, start_stack);
                                return None;
                            }
                        }
                        Err(_) => {
                            self.reset(start_pos, start_stack);
                            return None;
                        }
                    }
                }
                _ => break,
            }
        }
        self.skip_ign_newlines();
        match assign_op_of(self.peek()) {
            Some(op) => {
                self.bump();
                Some((Lvalue { span, base, path }, op))
            }
            None => {
                self.reset(start_pos, start_stack);
                None
            }
        }
    }

    /// 回滚游标与换行模式栈到给定快照（赋值试探失败时用）。
    fn reset(&mut self, pos: usize, stack_len: usize) {
        self.pos = pos;
        self.nl_stack.truncate(stack_len);
    }

    /// `let x = expr` / `var x = expr`（`decl_stmt`）。
    fn parse_decl(&mut self, span: Span) -> R<Stmt> {
        let mutable = matches!(self.peek(), TokenKind::KwVar);
        self.bump(); // let / var
        let name = match self.peek().clone() {
            TokenKind::Ident(name) => {
                self.bump();
                name
            }
            // §4.3 规则 4：`_` 不得作绑定名（`let _ = …` 非法）。
            TokenKind::Placeholder => {
                return Err(syntax(SyntaxMsg::PlaceholderPosition, self.peek_span()));
            }
            _ => return Err(self.unexpected("变量名")),
        };
        self.expect(&TokenKind::Assign)?;
        let init = self.parse_expr()?;
        Ok(Spanned::new(StmtKind::Decl { mutable, name, init }, span))
    }

    // ------------------------------------------------------------------
    // 声明：fn / struct（§7：fn_decl / struct_decl / member / params / body）
    // ------------------------------------------------------------------

    /// `fn name ( params ) body`（§7 `fn_decl`）：函数体内 `fn_depth` +1（`return` 合法）。
    fn parse_fn_decl(&mut self, span: Span) -> R<Stmt> {
        let (name, params, body) = self.parse_fn_components()?;
        Ok(Spanned::new(StmtKind::FnDecl(FnDecl { span, name, params, body }), span))
    }

    /// `fn` 头 + 参数表 + 函数体（`fn_decl` 与 struct 方法成员共用）。
    fn parse_fn_components(&mut self) -> R<(String, Vec<String>, Body)> {
        self.bump(); // fn
        let name = match self.peek().clone() {
            TokenKind::Ident(name) => {
                self.bump();
                name
            }
            _ => return Err(self.unexpected("函数名")),
        };
        let params = self.parse_params_parens()?;
        let body = self.parse_fn_body()?;
        Ok((name, params, body))
    }

    /// 参数表 `( params )`：`(` 内压 `IGN`（换行忽略，§3.2）。
    fn parse_params_parens(&mut self) -> R<Vec<String>> {
        self.expect(&TokenKind::LParen)?;
        self.nl_stack.push(NlMode::Ign);
        let params = self.parse_params()?;
        self.skip_ign_newlines();
        self.expect(&TokenKind::RParen)?;
        self.nl_stack.pop();
        Ok(params)
    }

    /// `params = IDENT , { "," , IDENT }`（§7）：空表合法；逗号必填、无尾逗号。
    fn parse_params(&mut self) -> R<Vec<String>> {
        let mut params = Vec::new();
        self.skip_ign_newlines();
        if *self.peek() == TokenKind::RParen {
            return Ok(params);
        }
        loop {
            self.skip_ign_newlines();
            match self.peek().clone() {
                TokenKind::Ident(name) => {
                    self.bump();
                    params.push(name);
                }
                // A24 / §4.3 规则 4：`_` 不得作形参名。
                TokenKind::Placeholder => {
                    return Err(syntax(SyntaxMsg::PlaceholderPosition, self.peek_span()));
                }
                _ => return Err(self.unexpected("形参名")),
            }
            self.skip_ign_newlines();
            if *self.peek() == TokenKind::Comma {
                self.bump();
                continue;
            }
            break;
        }
        Ok(params)
    }

    /// 函数体 `body = block | "=>" , ( block | expression )`（§7；§3.5 块前换行；M3）。
    ///
    /// 解析期间 `fn_depth` +1（体内 `return` 合法）并压入 `Barrier`（§4.3 规则 6：
    /// `_` 不得穿 λ 体）；错误路径亦先回退深度再上抛。
    fn parse_fn_body(&mut self) -> R<Body> {
        self.fn_depth += 1;
        self.pipe_stack.push(PipeCtx::Barrier);
        let result = self.parse_body_forms();
        self.pipe_stack.pop();
        self.fn_depth -= 1;
        result
    }

    /// `parse_fn_body` 的形态判定：`{` 块 / `=> {` 块（M3）/ `=> expr`。
    fn parse_body_forms(&mut self) -> R<Body> {
        self.skip_newlines(); // §3.5：块前换行
        if *self.peek() == TokenKind::LBrace {
            let block = self.parse_block()?;
            return Ok(Body::Block(block));
        }
        self.expect(&TokenKind::Arrow)?;
        self.parse_arrow_body()
    }

    /// `=>` 之后的体：`( block | expression )`；`=> {` 恒为块（M3）。
    fn parse_arrow_body(&mut self) -> R<Body> {
        self.skip_newlines();
        if *self.peek() == TokenKind::LBrace {
            let block = self.parse_block()?;
            Ok(Body::Block(block))
        } else {
            let expr = self.parse_expr()?;
            Ok(Body::Expr(expr))
        }
    }

    /// 箭头 lambda 的体（`=>` 已由 `try_parse_arrow_lambda` 消费）：
    /// 仅补 `fn_depth` 管理与 `Barrier`（§4.3 规则 6）后解析 `( block | expression )`。
    fn parse_arrow_fn_body(&mut self) -> R<Body> {
        self.fn_depth += 1;
        self.pipe_stack.push(PipeCtx::Barrier);
        let result = self.parse_arrow_body();
        self.pipe_stack.pop();
        self.fn_depth -= 1;
        result
    }

    /// `struct Name { members }`（§7 `struct_decl`）：成员表 `{ … }` 内换行忽略
    /// （§3.2）；头部与 `{` 之间允许换行（§3.3 / §3.5）。
    fn parse_struct_decl(&mut self, span: Span) -> R<Stmt> {
        self.bump(); // struct
        let name = match self.peek().clone() {
            TokenKind::Ident(name) => {
                self.bump();
                name
            }
            _ => return Err(self.unexpected("结构体名")),
        };
        self.skip_newlines();
        self.expect(&TokenKind::LBrace)?;
        self.nl_stack.push(NlMode::Ign);
        let members = self.parse_member_list()?;
        self.skip_ign_newlines();
        self.expect(&TokenKind::RBrace)?;
        self.nl_stack.pop();
        Ok(Spanned::new(StmtKind::StructDecl(StructDecl { span, name, members }), span))
    }

    /// `member_list = member , { "," , member } , [ "," ]`（§7；A7 尾逗号可选）。
    fn parse_member_list(&mut self) -> R<Vec<StructMember>> {
        let mut members = Vec::new();
        self.skip_ign_newlines();
        if *self.peek() == TokenKind::RBrace {
            return Ok(members);
        }
        loop {
            self.skip_ign_newlines();
            members.push(self.parse_member()?);
            self.skip_ign_newlines();
            if *self.peek() == TokenKind::Comma {
                self.bump();
                self.skip_ign_newlines();
                if *self.peek() == TokenKind::RBrace {
                    break; // 尾逗号
                }
                continue;
            }
            break;
        }
        Ok(members)
    }

    /// `member = fn_decl | IDENT , ":" , expression`（§7）。
    fn parse_member(&mut self) -> R<StructMember> {
        let span = self.peek_span();
        if *self.peek() == TokenKind::KwFn {
            let (name, params, body) = self.parse_fn_components()?;
            Ok(StructMember::Method(FnDecl { span, name, params, body }))
        } else {
            let name = match self.peek().clone() {
                TokenKind::Ident(name) => {
                    self.bump();
                    name
                }
                _ => return Err(self.unexpected("字段名或方法")),
            };
            self.expect(&TokenKind::Colon)?;
            let value = self.parse_expr()?;
            Ok(StructMember::Field(FieldInit { span, name, value }))
        }
    }

    /// 块 `{ ... }`：压入 `SIG` 模式，解析到匹配的 `}`。
    fn parse_block(&mut self) -> R<Block> {
        let span = self.peek_span(); // `{`
        self.expect(&TokenKind::LBrace)?;
        self.nl_stack.push(NlMode::Sig);
        let stmts = self.parse_stmt_seq()?;
        self.expect(&TokenKind::RBrace)?;
        self.nl_stack.pop();
        Ok(Block { span, stmts })
    }

    // ------------------------------------------------------------------
    // 控制流语句（§7：if / while / for / return / break / continue）
    // ------------------------------------------------------------------

    /// `if <expr> <block> [else ...]` 的**语句形态**：`StmtKind::If` 包裹 `if_expr`。
    fn parse_if_stmt(&mut self, span: Span) -> R<Stmt> {
        let expr = self.parse_if_expr(span)?;
        Ok(Spanned::new(StmtKind::If(expr), span))
    }

    /// `if_expr = "if" , expression , block , [ "else" , ( if_expr | block ) ]`（§7）。
    ///
    /// 条件走 NO_BRACE_LITERAL（§3.4）；块前换行跳过（§3.5）；`else` 前换行
    /// 先试探、无 `else` 则回退（§3.5；A4 悬挂 else 绑定最近的 if）。
    fn parse_if_expr(&mut self, span: Span) -> R<Expr> {
        self.bump(); // if
        let cond = self.parse_cond()?;
        self.skip_newlines();
        let then_block = self.parse_block()?;
        let else_branch = self.try_parse_else()?;
        Ok(Spanned::new(
            ExprKind::If(IfExpr {
                cond: Box::new(cond),
                then_block,
                else_branch,
            }),
            span,
        ))
    }

    /// `else` 分支（§3.5）：跳过换行看 `else`；命中则消费并解析
    /// `else if`（递归）或 `else { ... }`；未命中回退游标与模式栈。
    fn try_parse_else(&mut self) -> R<Option<ElseBranch>> {
        let save_pos = self.pos;
        let save_stack = self.nl_stack.len();
        self.skip_newlines();
        if *self.peek() != TokenKind::KwElse {
            self.reset(save_pos, save_stack);
            return Ok(None);
        }
        self.bump(); // else
        self.skip_newlines();
        if *self.peek() == TokenKind::KwIf {
            let span = self.peek_span();
            let inner = self.parse_if_expr(span)?;
            Ok(Some(ElseBranch::If(Box::new(inner))))
        } else {
            let block = self.parse_block()?;
            Ok(Some(ElseBranch::Block(block)))
        }
    }

    /// `while <expr> <block>`（§7 `while_stmt`）：条件走 NO_BRACE_LITERAL，体内 `loop_depth` +1。
    fn parse_while(&mut self, span: Span) -> R<Stmt> {
        self.bump(); // while
        let cond = self.parse_cond()?;
        self.skip_newlines();
        self.loop_depth += 1;
        let body = self.parse_block()?;
        self.loop_depth -= 1;
        Ok(Spanned::new(StmtKind::While { cond, body }, span))
    }

    /// `for <id> in <expr> <block>`（§7 `for_stmt`）：可迭代表达式走 NO_BRACE_LITERAL。
    fn parse_for(&mut self, span: Span) -> R<Stmt> {
        self.bump(); // for
        let var = match self.peek().clone() {
            TokenKind::Ident(name) => {
                self.bump();
                name
            }
            _ => return Err(self.unexpected("迭代变量名")),
        };
        self.expect(&TokenKind::KwIn)?;
        let iter = self.parse_cond()?;
        self.skip_newlines();
        self.loop_depth += 1;
        let body = self.parse_block()?;
        self.loop_depth -= 1;
        Ok(Spanned::new(StmtKind::For { var, iter, body }, span))
    }

    /// `return [expr]`（§7；A19：行尾 / `}` / EOF 即返回 nil，返回值须同行）。
    fn parse_return(&mut self, span: Span) -> R<Stmt> {
        if self.fn_depth == 0 {
            return Err(syntax(SyntaxMsg::ReturnOutsideFunction, span));
        }
        self.bump(); // return
        let value = match self.peek() {
            TokenKind::Newline | TokenKind::RBrace | TokenKind::Eof => None,
            _ => Some(self.parse_expr()?),
        };
        Ok(Spanned::new(StmtKind::Return(value), span))
    }

    /// `break` / `continue`（§7；循环外 → `BreakContinueOutsideLoop`）。
    fn parse_break_continue(&mut self, span: Span) -> R<Stmt> {
        let kw = match self.peek() {
            TokenKind::KwBreak => "break",
            TokenKind::KwContinue => "continue",
            _ => unreachable!("调用方保证当前是 break/continue"),
        };
        if self.loop_depth == 0 {
            return Err(syntax(
                SyntaxMsg::BreakContinueOutsideLoop { kw: kw.to_string() },
                span,
            ));
        }
        self.bump();
        let node = if kw == "break" { StmtKind::Break } else { StmtKind::Continue };
        Ok(Spanned::new(node, span))
    }

    /// 控制流头的表达式（if / while 条件、for 可迭代位）：
    /// 置 NO_BRACE_LITERAL（§3.4）后解析，结束恢复原状。
    fn parse_cond(&mut self) -> R<Expr> {
        let saved = self.cond_restrict;
        self.cond_restrict = true;
        let result = self.parse_expr();
        self.cond_restrict = saved;
        result
    }

    /// §3.4：NO_BRACE_LITERAL 生效且处于最外层（不在任何 `(` / `[` 内）时，
    /// 裸 `{` 不再作为 struct 字面量，而是终止条件表达式、交给块。
    fn no_brace_literal(&self) -> bool {
        self.cond_restrict && self.group_depth == 0
    }

    // ------------------------------------------------------------------
    // 表达式（§4.1 优先级分层：or → and → eq → cmp → add → mul → unary → postfix）
    // ------------------------------------------------------------------

    /// `expression = or_expr`（§7；`if_expr` / `lambda` 属后续批次）。
    fn parse_expr(&mut self) -> R<Expr> {
        self.parse_or()
    }

    /// `or_expr = and_expr , { "||" , and_expr }`（§4.1 级别 9，左结合）。
    fn parse_or(&mut self) -> R<Expr> {
        self.parse_logical_layer(Self::parse_and, or_op)
    }

    /// `and_expr = eq_expr , { "&&" , eq_expr }`（§4.1 级别 8，左结合）。
    fn parse_and(&mut self) -> R<Expr> {
        self.parse_logical_layer(Self::parse_eq, and_op)
    }

    /// `eq_expr = cmp_expr , { ( "==" | "!=" ) , cmp_expr }`（§4.1 级别 7，左结合）。
    fn parse_eq(&mut self) -> R<Expr> {
        self.parse_binary_layer(Self::parse_cmp, eq_op)
    }

    /// `cmp_expr = pipe_expr , { ( "<" | "<=" | ">" | ">=" ) , pipe_expr }`（§4.1 级别 6，左结合）。
    ///
    /// §4.4：`cmp_expr` 直接收 `pipe_expr`（`|>` 级别 5 比比较/逻辑紧、比 `+ - * / %` 松）。
    fn parse_cmp(&mut self) -> R<Expr> {
        self.parse_binary_layer(Self::parse_pipe, cmp_op)
    }

    /// `pipe_expr = add_expr , { "|>" , pipe_rhs }`（§4.1 级别 5，左结合；§4.3）。
    ///
    /// 每个 `|>` 的 RHS 解析期间压入 `PipeCtx::Rhs`（`_` 由此合法），解析完立即弹出
    /// （错误路径也弹出）；脱糖由 [`desugar_pipe`] 完成。
    fn parse_pipe(&mut self) -> R<Expr> {
        let mut left = self.parse_add()?;
        loop {
            self.skip_ign_newlines();
            if *self.peek() != TokenKind::Pipe {
                break;
            }
            let span = left.span;
            self.bump();
            self.pipe_stack.push(PipeCtx::Rhs {
                placeholder: None,
                call_args: 0,
            });
            let rhs_result = self.parse_pipe_rhs();
            let frame = self.pipe_stack.pop().expect("管道帧必然存在");
            let rhs = rhs_result?;
            left = desugar_pipe(left, rhs, frame.placeholder(), span);
        }
        Ok(left)
    }

    /// `pipe_rhs = postfix | lambda`（§4.3）。
    fn parse_pipe_rhs(&mut self) -> R<Expr> {
        self.skip_ign_newlines();
        let span = self.peek_span();
        match self.peek() {
            TokenKind::KwFn => self.parse_fn_lambda(span),
            TokenKind::LParen => match self.try_parse_arrow_lambda(span)? {
                Some(lambda) => Ok(lambda),
                None => self.parse_postfix(),
            },
            _ => self.parse_postfix(),
        }
    }

    /// `_` 占位符在当前位置的合法性（§4.3 规则 1 / 4 / 6）。
    fn placeholder_state(&self) -> PlaceholderState {
        match self.pipe_stack.last() {
            Some(PipeCtx::Rhs {
                placeholder: Some(_),
                ..
            }) => PlaceholderState::Multiple,
            Some(PipeCtx::Rhs {
                placeholder: None,
                call_args,
            }) if *call_args > 0 => PlaceholderState::Legal,
            _ => PlaceholderState::Illegal,
        }
    }

    /// 进入一层调用实参表：栈顶管道帧的实参位深度 +1。
    fn pipe_call_args_enter(&mut self) {
        if let Some(PipeCtx::Rhs { call_args, .. }) = self.pipe_stack.last_mut() {
            *call_args += 1;
        }
    }

    /// 离开一层调用实参表：栈顶管道帧的实参位深度 -1。
    fn pipe_call_args_leave(&mut self) {
        if let Some(PipeCtx::Rhs { call_args, .. }) = self.pipe_stack.last_mut() {
            *call_args -= 1;
        }
    }

    /// `add_expr = mul_expr , { ( "+" | "-" ) , mul_expr }`（§4.1 级别 4，左结合）。
    fn parse_add(&mut self) -> R<Expr> {
        self.parse_binary_layer(Self::parse_mul, add_op)
    }

    /// `mul_expr = unary , { ( "*" | "/" | "%" ) , unary }`（§4.1 级别 3，左结合）。
    fn parse_mul(&mut self) -> R<Expr> {
        self.parse_binary_layer(Self::parse_unary, mul_op)
    }

    /// 左结合二元层骨架：`layer = next , { op next }`（算术 / 比较 / 相等）。
    ///
    /// 每个 `Binary` 节点的 `span` 取最左操作数的起始位置（与后缀链同规）。
    /// 循环前 `skip_ign_newlines`：`IGN` 内换行当空白；`SIG` 下不吞换行
    /// （A2 行尾运算符不续行，缺失右操作数报 `IncompleteExpr`）。
    fn parse_binary_layer(
        &mut self,
        next: fn(&mut Self) -> R<Expr>,
        op_of: fn(&TokenKind) -> Option<BinaryOp>,
    ) -> R<Expr> {
        let mut left = next(self)?;
        loop {
            self.skip_ign_newlines();
            let op = match op_of(self.peek()) {
                Some(op) => op,
                None => break,
            };
            let span = left.span;
            self.bump();
            let right = next(self)?;
            left = Spanned::new(
                ExprKind::Binary { op, left: Box::new(left), right: Box::new(right) },
                span,
            );
        }
        Ok(left)
    }

    /// 左结合短路逻辑层骨架（`&&` / `||`）：与二元层同构，产出 `Logical` 节点。
    fn parse_logical_layer(
        &mut self,
        next: fn(&mut Self) -> R<Expr>,
        op_of: fn(&TokenKind) -> Option<LogicalOp>,
    ) -> R<Expr> {
        let mut left = next(self)?;
        loop {
            self.skip_ign_newlines();
            let op = match op_of(self.peek()) {
                Some(op) => op,
                None => break,
            };
            let span = left.span;
            self.bump();
            let right = next(self)?;
            left = Spanned::new(
                ExprKind::Logical { op, left: Box::new(left), right: Box::new(right) },
                span,
            );
        }
        Ok(left)
    }

    /// 一元 `-` / `!`（§4.1 级别 2）；否则降级到后缀表达式。
    ///
    /// `fn (…)` / `(…) => …` 在此层识别为函数字面量（§7 `unary` 层含 `lambda`；
    /// A26：lambda 不作后缀基，故不进入 `parse_postfix` 链）。
    fn parse_unary(&mut self) -> R<Expr> {
        self.skip_ign_newlines();
        let span = self.peek_span();
        match self.peek() {
            TokenKind::Minus => {
                self.bump();
                // B12（§10.8）：`-` 紧邻 INT 且其正值为 2^63 → `Int(i64::MIN)`
                // （不产生 `Unary` 节点）；否则按普通一元负号继续。
                if let TokenKind::Int(text) = self.peek().clone() {
                    if int_magnitude(&text) == Some(1u128 << 63) {
                        self.bump();
                        return Ok(Spanned::new(ExprKind::Int(i64::MIN), span));
                    }
                }
                let operand = self.parse_unary()?;
                Ok(Spanned::new(
                    ExprKind::Unary { op: UnaryOp::Neg, operand: Box::new(operand) },
                    span,
                ))
            }
            TokenKind::Bang => {
                self.bump();
                let operand = self.parse_unary()?;
                Ok(Spanned::new(
                    ExprKind::Unary { op: UnaryOp::Not, operand: Box::new(operand) },
                    span,
                ))
            }
            TokenKind::KwFn => self.parse_fn_lambda(span),
            TokenKind::LParen => match self.try_parse_arrow_lambda(span)? {
                Some(lambda) => Ok(lambda),
                None => self.parse_postfix(),
            },
            _ => self.parse_postfix(),
        }
    }

    /// 函数字面量 `fn ( params ) body`（§7 `lambda` 形式一；表达式位置）。
    fn parse_fn_lambda(&mut self, span: Span) -> R<Expr> {
        self.bump(); // fn
        let params = self.parse_params_parens()?;
        let body = self.parse_fn_body()?;
        Ok(Spanned::new(ExprKind::Lambda(Box::new(Lambda { params, body })), span))
    }

    /// 试探 `( params ) =>` lambda（§7 `lambda` 形式二；A20）：`(` 后若能按参数表
    /// 解析且 `)` 后紧跟 `=>` 则提交；否则**回滚**游标与换行模式栈，由调用方按
    /// 括号分组继续（`(a + b) => c` 因此回退为分组、随后在 `=>` 处报
    /// `TwoStatements`，符合 A20 反例）。
    fn try_parse_arrow_lambda(&mut self, span: Span) -> R<Option<Expr>> {
        let save_pos = self.pos;
        let save_stack = self.nl_stack.len();
        self.bump(); // (
        self.nl_stack.push(NlMode::Ign);
        let params = match self.parse_params() {
            Ok(p) => p,
            Err(_) => {
                self.reset(save_pos, save_stack);
                return Ok(None);
            }
        };
        self.skip_ign_newlines();
        if self.expect(&TokenKind::RParen).is_err() {
            self.reset(save_pos, save_stack);
            return Ok(None);
        }
        self.nl_stack.pop();
        self.skip_ign_newlines(); // 外层模式：`SIG` 下不吞换行（`=>` 须与 `)` 同逻辑行）
        if *self.peek() != TokenKind::Arrow {
            self.reset(save_pos, save_stack);
            return Ok(None);
        }
        self.bump(); // =>
        let body = self.parse_arrow_fn_body()?;
        Ok(Some(Spanned::new(
            ExprKind::Lambda(Box::new(Lambda { params, body })),
            span,
        )))
    }

    /// 后缀表达式 `postfix = primary , { call | index | field }`（§4.1 级别 1，左结合）。
    ///
    /// 链式示例：`a.b[0](x)` → `Call { Index { Field(a, b), 0 }, [x] }`。
    /// 每个后缀节点的 `span` 一律取基座（primary）的起始位置。
    fn parse_postfix(&mut self) -> R<Expr> {
        self.skip_ign_newlines();
        let span = self.peek_span();
        let mut expr = self.parse_primary()?;
        loop {
            self.skip_ign_newlines();
            match self.peek().clone() {
                // 调用 `callee ( args )`。
                TokenKind::LParen => {
                    self.bump();
                    self.nl_stack.push(NlMode::Ign);
                    self.group_depth += 1; // §3.4：实参表内临时解除 NO_BRACE_LITERAL
                    self.pipe_call_args_enter(); // §4.3：`_` 在调用实参位才合法
                    let args = self.parse_args(TokenKind::RParen)?;
                    self.pipe_call_args_leave();
                    self.expect(&TokenKind::RParen)?;
                    self.group_depth -= 1;
                    self.nl_stack.pop();
                    expr = Spanned::new(ExprKind::Call { callee: Box::new(expr), args }, span);
                }
                // 索引 `object [ expr ]`。
                TokenKind::LBracket => {
                    self.bump();
                    self.nl_stack.push(NlMode::Ign);
                    self.group_depth += 1; // §3.4：下标内临时解除 NO_BRACE_LITERAL
                    let inner = self.parse_expr()?;
                    self.skip_ign_newlines();
                    self.expect(&TokenKind::RBracket)?;
                    self.group_depth -= 1;
                    self.nl_stack.pop();
                    expr = Spanned::new(
                        ExprKind::Index { object: Box::new(expr), index: Box::new(inner) },
                        span,
                    );
                }
                // 字段 `object . IDENT`。
                TokenKind::Dot => {
                    self.bump();
                    let name = match self.peek().clone() {
                        TokenKind::Ident(name) => {
                            self.bump();
                            name
                        }
                        _ => return Err(self.unexpected("字段名")),
                    };
                    expr = Spanned::new(ExprKind::Field { object: Box::new(expr), name }, span);
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    /// 基本表达式：字面量 / 标识符 / 括号分组 / 数组与 struct 字面量。
    fn parse_primary(&mut self) -> R<Expr> {
        self.skip_ign_newlines();
        let span = self.peek_span();
        match self.peek().clone() {
            TokenKind::Int(text) => {
                self.bump();
                // §10.8 B12：十进制超出 i64 / radix 形式超出 u64 → `IntegerOutOfRange`。
                let n = parse_int(&text)
                    .ok_or_else(|| syntax(SyntaxMsg::IntegerOutOfRange, span))?;
                Ok(Spanned::new(ExprKind::Int(n), span))
            }
            TokenKind::Float(text) => {
                self.bump();
                let f = parse_float(&text).ok_or_else(|| {
                    syntax(
                        SyntaxMsg::UnexpectedToken {
                            expected: "浮点数".to_string(),
                            got: format!("'{text}'"),
                        },
                        span,
                    )
                })?;
                Ok(Spanned::new(ExprKind::Float(f), span))
            }
            TokenKind::StrBegin => self.parse_string(span),
            TokenKind::KwTrue => {
                self.bump();
                Ok(Spanned::new(ExprKind::Bool(true), span))
            }
            TokenKind::KwFalse => {
                self.bump();
                Ok(Spanned::new(ExprKind::Bool(false), span))
            }
            TokenKind::KwNil => {
                self.bump();
                Ok(Spanned::new(ExprKind::Nil, span))
            }
            TokenKind::KwSelf => {
                self.bump();
                Ok(Spanned::new(ExprKind::SelfRef, span))
            }
            TokenKind::Ident(name) => {
                self.bump();
                // `IDENT {`：具名 struct 字面量（§3.3 规则 2、§7 `struct_lit`）。
                // 仅当 `{` 紧随其后（`IGN` 内换行忽略，`SIG` 下换行即拆句，A3）；
                // 但 NO_BRACE_LITERAL（§3.4）最外层禁用：`{` 交给块。
                self.skip_ign_newlines();
                if *self.peek() == TokenKind::LBrace && !self.no_brace_literal() {
                    self.parse_struct_lit(Some(name), span)
                } else {
                    Ok(Spanned::new(ExprKind::Ident(name), span))
                }
            }
            TokenKind::LBracket => self.parse_array(span),
            // `_` 占位符（§4.3）：仅管道 RHS 调用实参位合法，见 `placeholder_state`。
            TokenKind::Placeholder => match self.placeholder_state() {
                PlaceholderState::Legal => {
                    self.bump();
                    if let Some(PipeCtx::Rhs { placeholder, .. }) = self.pipe_stack.last_mut() {
                        *placeholder = Some(span);
                    }
                    // 哨兵：`Ident("_")` 不可能由词法产生（裸 `_` 是 `Placeholder` 记号），
                    // RHS 解析完成后由 `replace_placeholder` 原位替换为管道左侧。
                    Ok(Spanned::new(ExprKind::Ident("_".to_string()), span))
                }
                PlaceholderState::Multiple => {
                    Err(syntax(SyntaxMsg::PipeMultiplePlaceholder, span))
                }
                PlaceholderState::Illegal => Err(syntax(SyntaxMsg::PlaceholderPosition, span)),
            },
            TokenKind::LBrace => {
                if self.no_brace_literal() {
                    // §3.4：条件 / 可迭代位最外层的裸 `{` 终止表达式；此处表达式
                    // 尚未开始（或无法继续）→ 不完整。
                    Err(syntax(SyntaxMsg::IncompleteExpr, span))
                } else {
                    self.parse_struct_lit(None, span)
                }
            }
            TokenKind::LParen => self.parse_group(span),
            TokenKind::Newline => Err(syntax(SyntaxMsg::IncompleteExpr, span)),
            _ => Err(self.unexpected("表达式")),
        }
    }

    /// 字符串字面量（§2.8）：`StrBegin , { Text | 插值段 } , StrEnd`。
    ///
    /// 无插值 → `ExprKind::Str`（纯文本）；含插值 → `ExprKind::Interp(InterpString)`，
    /// 分段按书写顺序；插值段 `InterpBegin , 表达式 , [FormatSpec] , InterpEnd`，
    /// `format_spec` 无 `:` 为 `None`（M6），`:` 后可空串。
    fn parse_string(&mut self, span: Span) -> R<Expr> {
        self.bump(); // StrBegin
        let mut parts: Vec<StrPart> = Vec::new();
        let mut cur_text = String::new();
        let mut saw_interp = false;
        loop {
            match self.peek().clone() {
                TokenKind::Text(part) => {
                    cur_text.push_str(&part);
                    self.bump();
                }
                TokenKind::InterpBegin => {
                    self.bump();
                    saw_interp = true;
                    if !cur_text.is_empty() {
                        parts.push(StrPart::Text(std::mem::take(&mut cur_text)));
                    }
                    let expr = self.parse_expr()?;
                    let format_spec = match self.peek().clone() {
                        TokenKind::FormatSpec(spec) => {
                            self.bump();
                            Some(spec)
                        }
                        _ => None,
                    };
                    self.expect(&TokenKind::InterpEnd)?;
                    parts.push(StrPart::Expr { expr, format_spec });
                }
                TokenKind::StrEnd => {
                    self.bump();
                    break;
                }
                _ => return Err(self.unexpected("\"")),
            }
        }
        if saw_interp {
            if !cur_text.is_empty() {
                parts.push(StrPart::Text(std::mem::take(&mut cur_text)));
            }
            Ok(Spanned::new(ExprKind::Interp(InterpString { parts }), span))
        } else {
            Ok(Spanned::new(ExprKind::Str(cur_text), span))
        }
    }

    /// 括号分组 `( expr )`：`(` 内压入 `IGN` 模式（换行当空白）。
    fn parse_group(&mut self, span: Span) -> R<Expr> {
        self.bump(); // LParen
        self.nl_stack.push(NlMode::Ign);
        self.group_depth += 1; // §3.4：括号内临时解除 NO_BRACE_LITERAL
        let inner = self.parse_expr()?;
        self.skip_ign_newlines();
        self.expect(&TokenKind::RParen)?;
        self.group_depth -= 1;
        self.nl_stack.pop();
        Ok(Spanned::new(inner.node, span))
    }

    /// `args = expression , { "," , expression } , [ "," ]`（§7；A8）。
    ///
    /// 用于调用实参表与数组元素表；`close` 为终结分隔符（`)` / `]`）。
    /// 逗号必填、尾逗号可选；表内换行已由调用方压入的 `IGN` 模式忽略。
    fn parse_args(&mut self, close: TokenKind) -> R<Vec<Expr>> {
        let mut args = Vec::new();
        self.skip_ign_newlines();
        if self.peek() == &close {
            return Ok(args);
        }
        loop {
            self.skip_ign_newlines();
            args.push(self.parse_expr()?);
            self.skip_ign_newlines();
            if *self.peek() == TokenKind::Comma {
                self.bump();
                self.skip_ign_newlines();
                if self.peek() == &close {
                    break; // 尾逗号
                }
                continue;
            }
            break;
        }
        Ok(args)
    }

    /// 数组字面量 `array_lit = "[" , [ args ] , "]"`（§7；`[` 内压 `IGN`）。
    fn parse_array(&mut self, span: Span) -> R<Expr> {
        self.bump(); // LBracket
        self.nl_stack.push(NlMode::Ign);
        self.group_depth += 1; // §3.4：方括号内临时解除 NO_BRACE_LITERAL
        let elems = self.parse_args(TokenKind::RBracket)?;
        self.expect(&TokenKind::RBracket)?;
        self.group_depth -= 1;
        self.nl_stack.pop();
        Ok(Spanned::new(ExprKind::Array(elems), span))
    }

    /// struct 字面量 `struct_lit = [ IDENT ] , "{" , [ field_list ] , "}"`（§7）。
    ///
    /// 成员表 `{ … }` 内压 `IGN`（§3.2 表）。`type_name` 为具名形式的前缀名。
    /// 期望当前 token 为 `{`。
    fn parse_struct_lit(&mut self, type_name: Option<String>, span: Span) -> R<Expr> {
        self.expect(&TokenKind::LBrace)?;
        self.nl_stack.push(NlMode::Ign);
        let fields = self.parse_field_list()?;
        self.expect(&TokenKind::RBrace)?;
        self.nl_stack.pop();
        Ok(Spanned::new(ExprKind::StructLit(StructLit { type_name, fields }), span))
    }

    /// `field_list = field_init , { "," , field_init } , [ "," ]`（§7；A7）。
    fn parse_field_list(&mut self) -> R<Vec<FieldInit>> {
        let mut fields = Vec::new();
        self.skip_ign_newlines();
        if *self.peek() == TokenKind::RBrace {
            return Ok(fields);
        }
        loop {
            self.skip_ign_newlines();
            fields.push(self.parse_field_init()?);
            self.skip_ign_newlines();
            if *self.peek() == TokenKind::Comma {
                self.bump();
                self.skip_ign_newlines();
                if *self.peek() == TokenKind::RBrace {
                    break; // 尾逗号
                }
                continue;
            }
            break;
        }
        Ok(fields)
    }

    /// `field_init = ( IDENT | STRING ) , ":" , expression`（§7）。
    fn parse_field_init(&mut self) -> R<FieldInit> {
        let span = self.peek_span();
        let name = match self.peek().clone() {
            TokenKind::Ident(name) => {
                self.bump();
                name
            }
            TokenKind::StrBegin => {
                let e = self.parse_string(span)?;
                match e.node {
                    // §7 `field_init = ( IDENT | STRING )`：字段名只收纯字符串。
                    ExprKind::Str(s) => s,
                    ExprKind::Interp(_) => {
                        return Err(syntax(
                            SyntaxMsg::UnexpectedToken {
                                expected: "字段名".to_string(),
                                got: "插值字符串".to_string(),
                            },
                            span,
                        ));
                    }
                    _ => unreachable!("字符串解析必得 Str 或 Interp"),
                }
            }
            _ => return Err(self.unexpected("字段名")),
        };
        self.expect(&TokenKind::Colon)?;
        let value = self.parse_expr()?;
        Ok(FieldInit { span, name, value })
    }
}

// ===========================================================================
// 辅助函数
// ===========================================================================

/// 解析整数字面量原文（`_` 分隔与 `0x` / `0b` / `0o` 进制，§2.7；§10.8 B12）。
///
/// 十进制按 `i64` 解析；十六 / 二 / 八进制按 `u64` 解析后**按位重解释**为 `i64`
/// （radix 形式仅在超出 `u64` 范围时报错）。失败返回 `None`。
fn parse_int(text: &str) -> Option<i64> {
    let t = text.replace('_', "");
    if let Some(r) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        u64::from_str_radix(r, 16).ok().map(|v| v as i64)
    } else if let Some(r) = t.strip_prefix("0b").or_else(|| t.strip_prefix("0B")) {
        u64::from_str_radix(r, 2).ok().map(|v| v as i64)
    } else if let Some(r) = t.strip_prefix("0o").or_else(|| t.strip_prefix("0O")) {
        u64::from_str_radix(r, 8).ok().map(|v| v as i64)
    } else {
        t.parse::<i64>().ok()
    }
}

/// 整数字面量原文的无符号数值（`u128`），供 `-` 后 `2^63` 特判（§10.8 B12）。
///
/// 覆盖十进制与 `0x` / `0b` / `0o` 进制（`_` 分隔先去除）。
fn int_magnitude(text: &str) -> Option<u128> {
    let t = text.replace('_', "");
    let (radix, digits) = if let Some(r) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        (16, r)
    } else if let Some(r) = t.strip_prefix("0b").or_else(|| t.strip_prefix("0B")) {
        (2, r)
    } else if let Some(r) = t.strip_prefix("0o").or_else(|| t.strip_prefix("0O")) {
        (8, r)
    } else {
        (10, t.as_str())
    };
    u128::from_str_radix(digits, radix).ok()
}

/// §4.3 管道脱糖：`L |> F(args)` 生成 `Call`。
///
/// - 恰有一个 `_` → 原位替换为 `L`（`replace_placeholder`）；
/// - 无 `_` 且右侧是调用 → `L` **追加为末参**（data-last）；
/// - 无 `_` 且右侧非调用（函数名 / 方法 / lambda）→ `R(L)`。
///
/// 生成节点的 `span` 取管道左侧起始位置（与二元层「最左操作数」同规）。
fn desugar_pipe(left: Expr, rhs: Expr, placeholder: Option<Span>, span: Span) -> Expr {
    if placeholder.is_some() {
        let replaced = replace_placeholder(rhs, &left);
        Spanned::new(replaced.node, span)
    } else {
        let node = match rhs.node {
            ExprKind::Call { callee, mut args } => {
                args.push(left);
                ExprKind::Call { callee, args }
            }
            _ => ExprKind::Call {
                callee: Box::new(rhs),
                args: vec![left],
            },
        };
        Spanned::new(node, span)
    }
}

/// 把管道 RHS 中的占位哨兵（`Ident("_")`，仅由管道实参位产生）替换为 `with`。
///
/// `_` 只可能出现在调用实参位（解析期保证），故沿实参可嵌套的节点（`Call` /
/// 数组 / struct 字面量 / 插值字符串 / 后缀 / 一元 / 二元 / 逻辑）深入即可；
/// lambda 体内的 `_` 解析期即被 `Barrier` 拒绝（§4.3 规则 6），故不进入 `Lambda`。
fn replace_placeholder(expr: Expr, with: &Expr) -> Expr {
    let span = expr.span;
    let node = match expr.node {
        ExprKind::Ident(ref name) if name == "_" => return with.clone(),
        ExprKind::Array(xs) => ExprKind::Array(
            xs.into_iter().map(|e| replace_placeholder(e, with)).collect(),
        ),
        ExprKind::Interp(InterpString { parts }) => ExprKind::Interp(InterpString {
            parts: parts
                .into_iter()
                .map(|p| match p {
                    StrPart::Text(t) => StrPart::Text(t),
                    StrPart::Expr { expr, format_spec } => StrPart::Expr {
                        expr: replace_placeholder(expr, with),
                        format_spec,
                    },
                })
                .collect(),
        }),
        ExprKind::StructLit(StructLit { type_name, fields }) => {
            ExprKind::StructLit(StructLit {
                type_name,
                fields: fields
                    .into_iter()
                    .map(|f| FieldInit {
                        span: f.span,
                        name: f.name,
                        value: replace_placeholder(f.value, with),
                    })
                    .collect(),
            })
        }
        ExprKind::Field { object, name } => ExprKind::Field {
            object: Box::new(replace_placeholder(*object, with)),
            name,
        },
        ExprKind::Index { object, index } => ExprKind::Index {
            object: Box::new(replace_placeholder(*object, with)),
            index: Box::new(replace_placeholder(*index, with)),
        },
        ExprKind::Call { callee, args } => ExprKind::Call {
            callee: Box::new(replace_placeholder(*callee, with)),
            args: args
                .into_iter()
                .map(|a| replace_placeholder(a, with))
                .collect(),
        },
        ExprKind::Unary { op, operand } => ExprKind::Unary {
            op,
            operand: Box::new(replace_placeholder(*operand, with)),
        },
        ExprKind::Binary { op, left, right } => ExprKind::Binary {
            op,
            left: Box::new(replace_placeholder(*left, with)),
            right: Box::new(replace_placeholder(*right, with)),
        },
        ExprKind::Logical { op, left, right } => ExprKind::Logical {
            op,
            left: Box::new(replace_placeholder(*left, with)),
            right: Box::new(replace_placeholder(*right, with)),
        },
        // 字面量 / SelfRef / If / Lambda：不可能含占位符。
        other => other,
    };
    Spanned::new(node, span)
}

/// 解析浮点字面量原文（允许 `_` 分隔，§2.7）。
fn parse_float(text: &str) -> Option<f64> {
    text.replace('_', "").parse::<f64>().ok()
}

/// §4.1 级别 3：`*` `/` `%` → `BinaryOp`。
fn mul_op(kind: &TokenKind) -> Option<BinaryOp> {
    match kind {
        TokenKind::Star => Some(BinaryOp::Mul),
        TokenKind::Slash => Some(BinaryOp::Div),
        TokenKind::Percent => Some(BinaryOp::Rem),
        _ => None,
    }
}

/// §4.1 级别 4：`+` `-` → `BinaryOp`。
fn add_op(kind: &TokenKind) -> Option<BinaryOp> {
    match kind {
        TokenKind::Plus => Some(BinaryOp::Add),
        TokenKind::Minus => Some(BinaryOp::Sub),
        _ => None,
    }
}

/// §4.1 级别 6：`<` `<=` `>` `>=` → `BinaryOp`。
fn cmp_op(kind: &TokenKind) -> Option<BinaryOp> {
    match kind {
        TokenKind::Lt => Some(BinaryOp::Lt),
        TokenKind::Le => Some(BinaryOp::Le),
        TokenKind::Gt => Some(BinaryOp::Gt),
        TokenKind::Ge => Some(BinaryOp::Ge),
        _ => None,
    }
}

/// §4.1 级别 7：`==` `!=` → `BinaryOp`。
fn eq_op(kind: &TokenKind) -> Option<BinaryOp> {
    match kind {
        TokenKind::EqEq => Some(BinaryOp::Eq),
        TokenKind::NotEq => Some(BinaryOp::Ne),
        _ => None,
    }
}

/// §4.1 级别 8：`&&` → `LogicalOp`。
fn and_op(kind: &TokenKind) -> Option<LogicalOp> {
    match kind {
        TokenKind::AndAnd => Some(LogicalOp::And),
        _ => None,
    }
}

/// §4.1 级别 9：`||` → `LogicalOp`。
fn or_op(kind: &TokenKind) -> Option<LogicalOp> {
    match kind {
        TokenKind::OrOr => Some(LogicalOp::Or),
        _ => None,
    }
}

/// `assign_op = "=" | "+=" | "-=" | "*=" | "/=" | "%="`（§7）。
fn assign_op_of(kind: &TokenKind) -> Option<AssignOp> {
    match kind {
        TokenKind::Assign => Some(AssignOp::Assign),
        TokenKind::PlusAssign => Some(AssignOp::AddAssign),
        TokenKind::MinusAssign => Some(AssignOp::SubAssign),
        TokenKind::StarAssign => Some(AssignOp::MulAssign),
        TokenKind::SlashAssign => Some(AssignOp::DivAssign),
        TokenKind::PercentAssign => Some(AssignOp::RemAssign),
        _ => None,
    }
}

/// 记号的可读描述（用于错误消息的「得到 …」）。
fn token_desc(kind: &TokenKind) -> String {
    let raw = match kind {
        TokenKind::Int(t)
        | TokenKind::Float(t)
        | TokenKind::Ident(t)
        | TokenKind::Text(t)
        | TokenKind::FormatSpec(t) => return format!("'{t}'"),
        TokenKind::Reserved(w) => return format!("'{w}'"),
        TokenKind::Newline => "换行",
        TokenKind::Eof => "文件末尾",
        TokenKind::StrBegin | TokenKind::StrEnd => "\"",
        TokenKind::InterpBegin => "${",
        TokenKind::InterpEnd => "}",
        TokenKind::KwLet => "let",
        TokenKind::KwVar => "var",
        TokenKind::KwFn => "fn",
        TokenKind::KwReturn => "return",
        TokenKind::KwIf => "if",
        TokenKind::KwElse => "else",
        TokenKind::KwWhile => "while",
        TokenKind::KwFor => "for",
        TokenKind::KwIn => "in",
        TokenKind::KwBreak => "break",
        TokenKind::KwContinue => "continue",
        TokenKind::KwStruct => "struct",
        TokenKind::KwTrue => "true",
        TokenKind::KwFalse => "false",
        TokenKind::KwNil => "nil",
        TokenKind::KwSelf => "self",
        TokenKind::Dump => ";;",
        TokenKind::Semi => ";",
        TokenKind::Arrow => "=>",
        TokenKind::EqEq => "==",
        TokenKind::NotEq => "!=",
        TokenKind::Le => "<=",
        TokenKind::Ge => ">=",
        TokenKind::Lt => "<",
        TokenKind::Gt => ">",
        TokenKind::AndAnd => "&&",
        TokenKind::OrOr => "||",
        TokenKind::Pipe => "|>",
        TokenKind::PlusAssign => "+=",
        TokenKind::MinusAssign => "-=",
        TokenKind::StarAssign => "*=",
        TokenKind::SlashAssign => "/=",
        TokenKind::PercentAssign => "%=",
        TokenKind::Assign => "=",
        TokenKind::Bang => "!",
        TokenKind::Plus => "+",
        TokenKind::Minus => "-",
        TokenKind::Star => "*",
        TokenKind::Slash => "/",
        TokenKind::Percent => "%",
        TokenKind::LParen => "(",
        TokenKind::RParen => ")",
        TokenKind::LBracket => "[",
        TokenKind::RBracket => "]",
        TokenKind::LBrace => "{",
        TokenKind::RBrace => "}",
        TokenKind::Comma => ",",
        TokenKind::Colon => ":",
        TokenKind::Dot => ".",
        TokenKind::Placeholder => "_",
    };
    format!("'{raw}'")
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::ScopeId;
    use crate::error::LzError;

    fn tok(kind: TokenKind, line: u32, col: u32) -> Token {
        Token {
            kind,
            span: Span::new(line, col),
        }
    }

    /// 断言错误为 `SyntaxError`，且子消息与位置符合预期。
    fn assert_syntax(err: &Box<LzError>, want: SyntaxMsg, line: u32, col: u32) {
        match err.as_ref() {
            LzError::Syntax { msg, span } => {
                assert_eq!(msg, &want, "子消息不符");
                assert_eq!(*span, Span::new(line, col), "位置不符");
            }
            other => panic!("应为 SyntaxError，得到 {}", other.class_name()),
        }
    }

    #[test]
    fn empty_program_is_valid() {
        let p = parse(&[tok(TokenKind::Eof, 1, 1)]).unwrap();
        assert!(p.stmts.is_empty());
        assert_eq!(p.span, Span::new(1, 1));
    }

    #[test]
    fn let_declaration_binds_name_and_init() {
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("x".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::Int("1".into()), 1, 9),
            tok(TokenKind::Eof, 1, 10),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 1);
        assert_eq!(p.stmts[0].span, Span::new(1, 1));
        match &p.stmts[0].node {
            StmtKind::Decl { mutable, name, init } => {
                assert!(!mutable);
                assert_eq!(name, "x");
                assert_eq!(init.node, ExprKind::Int(1));
                assert_eq!(init.span, Span::new(1, 9));
            }
            other => panic!("应为 Decl，得到 {other:?}"),
        }
    }

    #[test]
    fn var_declaration_is_mutable() {
        let p = parse(&[
            tok(TokenKind::KwVar, 1, 1),
            tok(TokenKind::Ident("y".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::Int("2".into()), 1, 9),
            tok(TokenKind::Eof, 1, 10),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::Decl { mutable, name, .. } => {
                assert!(*mutable);
                assert_eq!(name, "y");
            }
            other => panic!("应为 Decl，得到 {other:?}"),
        }
    }

    #[test]
    fn block_statement_inlines_contents() {
        let p = parse(&[
            tok(TokenKind::LBrace, 1, 1),
            tok(TokenKind::KwLet, 1, 3),
            tok(TokenKind::Ident("x".into()), 1, 7),
            tok(TokenKind::Assign, 1, 9),
            tok(TokenKind::Int("1".into()), 1, 11),
            tok(TokenKind::RBrace, 1, 13),
            tok(TokenKind::Eof, 1, 14),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 1);
        assert!(matches!(p.stmts[0].node, StmtKind::Decl { .. }));
    }

    #[test]
    fn expression_statement_int() {
        let p = parse(&[
            tok(TokenKind::Int("1".into()), 1, 1),
            tok(TokenKind::Eof, 1, 2),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 1);
        match &p.stmts[0].node {
            StmtKind::Expr(e) => {
                assert_eq!(e.node, ExprKind::Int(1));
                assert_eq!(e.span, Span::new(1, 1));
            }
            other => panic!("应为 Expr，得到 {other:?}"),
        }
    }

    #[test]
    fn unary_neg() {
        let p = parse(&[
            tok(TokenKind::Minus, 1, 1),
            tok(TokenKind::Int("1".into()), 1, 2),
            tok(TokenKind::Eof, 1, 3),
        ])
        .unwrap();
        let e = match &p.stmts[0].node {
            StmtKind::Expr(e) => e,
            other => panic!("应为 Expr，得到 {other:?}"),
        };
        assert_eq!(e.span, Span::new(1, 1));
        match &e.node {
            ExprKind::Unary { op, operand } => {
                assert_eq!(*op, UnaryOp::Neg);
                assert_eq!(operand.node, ExprKind::Int(1));
            }
            other => panic!("应为 Unary，得到 {other:?}"),
        }
    }

    #[test]
    fn unary_not() {
        let p = parse(&[
            tok(TokenKind::Bang, 1, 1),
            tok(TokenKind::KwTrue, 1, 2),
            tok(TokenKind::Eof, 1, 6),
        ])
        .unwrap();
        let e = match &p.stmts[0].node {
            StmtKind::Expr(e) => e,
            other => panic!("应为 Expr，得到 {other:?}"),
        };
        match &e.node {
            ExprKind::Unary { op, operand } => {
                assert_eq!(*op, UnaryOp::Not);
                assert_eq!(operand.node, ExprKind::Bool(true));
            }
            other => panic!("应为 Unary，得到 {other:?}"),
        }
    }

    #[test]
    fn paren_grouping_keeps_outer_span() {
        let p = parse(&[
            tok(TokenKind::LParen, 1, 1),
            tok(TokenKind::Int("1".into()), 1, 2),
            tok(TokenKind::RParen, 1, 3),
            tok(TokenKind::Eof, 1, 4),
        ])
        .unwrap();
        let e = match &p.stmts[0].node {
            StmtKind::Expr(e) => e,
            other => panic!("应为 Expr，得到 {other:?}"),
        };
        assert_eq!(e.node, ExprKind::Int(1));
        assert_eq!(e.span, Span::new(1, 1)); // 外层 `(` 的位置
    }

    #[test]
    fn nil_literal() {
        let p = parse(&[
            tok(TokenKind::KwNil, 1, 1),
            tok(TokenKind::Eof, 1, 4),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::Expr(e) => assert_eq!(e.node, ExprKind::Nil),
            other => panic!("应为 Expr，得到 {other:?}"),
        }
    }

    #[test]
    fn plain_string_literal() {
        let p = parse(&[
            tok(TokenKind::StrBegin, 1, 1),
            tok(TokenKind::Text("hi".into()), 1, 2),
            tok(TokenKind::StrEnd, 1, 4),
            tok(TokenKind::Eof, 1, 5),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::Expr(e) => assert_eq!(e.node, ExprKind::Str("hi".into())),
            other => panic!("应为 Expr，得到 {other:?}"),
        }
    }

    #[test]
    fn missing_decl_init_is_incomplete_expr() {
        let err = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("x".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::Newline, 1, 9),
            tok(TokenKind::Eof, 2, 1),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::IncompleteExpr, 1, 9);
    }

    #[test]
    fn lone_semicolon_is_error() {
        let err = parse(&[
            tok(TokenKind::Semi, 1, 1),
            tok(TokenKind::Eof, 1, 2),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::LoneSemicolon, 1, 1);
    }

    #[test]
    fn if_without_condition_is_error() {
        // 本批起 `if` 是合法语句头；`if` 后跟 EOF 即条件表达式缺失。
        let err = parse(&[
            tok(TokenKind::KwIf, 1, 1),
            tok(TokenKind::Eof, 1, 3),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "表达式".to_string(),
                got: "'文件末尾'".to_string(),
            },
            1,
            3,
        );
    }

    #[test]
    fn newlines_separate_statements() {
        let p = parse(&[
            tok(TokenKind::Int("1".into()), 1, 1),
            tok(TokenKind::Newline, 1, 2),
            tok(TokenKind::Int("2".into()), 2, 1),
            tok(TokenKind::Eof, 2, 2),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 2);
        assert_eq!(p.stmts[0].span, Span::new(1, 1));
        assert_eq!(p.stmts[1].span, Span::new(2, 1));
    }

    #[test]
    fn newline_inside_parens_is_ignored() {
        let p = parse(&[
            tok(TokenKind::LParen, 1, 1),
            tok(TokenKind::Int("1".into()), 1, 2),
            tok(TokenKind::Newline, 1, 3),
            tok(TokenKind::RParen, 2, 1),
            tok(TokenKind::Eof, 2, 2),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 1);
        match &p.stmts[0].node {
            StmtKind::Expr(e) => assert_eq!(e.node, ExprKind::Int(1)),
            other => panic!("应为 Expr，得到 {other:?}"),
        }
    }

    #[test]
    fn two_statements_on_same_line_is_error() {
        let err = parse(&[
            tok(TokenKind::Int("1".into()), 1, 1),
            tok(TokenKind::Int("2".into()), 1, 3),
            tok(TokenKind::Eof, 1, 4),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::TwoStatements, 1, 3);
    }

    // ---- 后缀表达式（§4.1 级别 1：调用 / 索引 / 字段）----

    /// 取程序首条语句的表达式节点。
    fn expr_of(p: &Program) -> &Expr {
        match &p.stmts[0].node {
            StmtKind::Expr(e) => e,
            other => panic!("应为 Expr，得到 {other:?}"),
        }
    }

    /// 取程序首条语句（`let` 声明）的初始化表达式节点。
    fn decl_init(p: &Program) -> &Expr {
        match &p.stmts[0].node {
            StmtKind::Decl { init, .. } => init,
            other => panic!("应为 Decl，得到 {other:?}"),
        }
    }

    #[test]
    fn call_with_zero_args() {
        let p = parse(&[
            tok(TokenKind::Ident("f".into()), 1, 1),
            tok(TokenKind::LParen, 1, 2),
            tok(TokenKind::RParen, 1, 3),
            tok(TokenKind::Eof, 1, 4),
        ])
        .unwrap();
        let e = expr_of(&p);
        assert_eq!(e.span, Span::new(1, 1));
        match &e.node {
            ExprKind::Call { callee, args } => {
                assert_eq!(callee.node, ExprKind::Ident("f".into()));
                assert!(args.is_empty());
            }
            other => panic!("应为 Call，得到 {other:?}"),
        }
    }

    #[test]
    fn call_with_many_args() {
        let p = parse(&[
            tok(TokenKind::Ident("f".into()), 1, 1),
            tok(TokenKind::LParen, 1, 2),
            tok(TokenKind::Int("1".into()), 1, 3),
            tok(TokenKind::Comma, 1, 4),
            tok(TokenKind::Int("2".into()), 1, 6),
            tok(TokenKind::Comma, 1, 7),
            tok(TokenKind::Int("3".into()), 1, 9),
            tok(TokenKind::RParen, 1, 10),
            tok(TokenKind::Eof, 1, 11),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Call { args, .. } => {
                assert_eq!(args.len(), 3);
                assert_eq!(args[2].node, ExprKind::Int(3));
            }
            other => panic!("应为 Call，得到 {other:?}"),
        }
    }

    #[test]
    fn call_trailing_comma_is_allowed() {
        let p = parse(&[
            tok(TokenKind::Ident("f".into()), 1, 1),
            tok(TokenKind::LParen, 1, 2),
            tok(TokenKind::Int("1".into()), 1, 3),
            tok(TokenKind::Comma, 1, 4),
            tok(TokenKind::RParen, 1, 5),
            tok(TokenKind::Eof, 1, 6),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Call { args, .. } => assert_eq!(args.len(), 1),
            other => panic!("应为 Call，得到 {other:?}"),
        }
    }

    #[test]
    fn index_expression() {
        let p = parse(&[
            tok(TokenKind::Ident("xs".into()), 1, 1),
            tok(TokenKind::LBracket, 1, 3),
            tok(TokenKind::Int("0".into()), 1, 4),
            tok(TokenKind::RBracket, 1, 5),
            tok(TokenKind::Eof, 1, 6),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Index { object, index } => {
                assert_eq!(object.node, ExprKind::Ident("xs".into()));
                assert_eq!(index.node, ExprKind::Int(0));
            }
            other => panic!("应为 Index，得到 {other:?}"),
        }
    }

    #[test]
    fn field_access() {
        let p = parse(&[
            tok(TokenKind::Ident("s".into()), 1, 1),
            tok(TokenKind::Dot, 1, 2),
            tok(TokenKind::Ident("k".into()), 1, 3),
            tok(TokenKind::Eof, 1, 4),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Field { object, name } => {
                assert_eq!(object.node, ExprKind::Ident("s".into()));
                assert_eq!(name, "k");
            }
            other => panic!("应为 Field，得到 {other:?}"),
        }
    }

    #[test]
    fn chained_postfix_a_b_0_x() {
        let p = parse(&[
            tok(TokenKind::Ident("a".into()), 1, 1),
            tok(TokenKind::Dot, 1, 2),
            tok(TokenKind::Ident("b".into()), 1, 3),
            tok(TokenKind::LBracket, 1, 4),
            tok(TokenKind::Int("0".into()), 1, 5),
            tok(TokenKind::RBracket, 1, 6),
            tok(TokenKind::LParen, 1, 7),
            tok(TokenKind::Ident("x".into()), 1, 8),
            tok(TokenKind::RParen, 1, 9),
            tok(TokenKind::Eof, 1, 10),
        ])
        .unwrap();
        let e = expr_of(&p);
        assert_eq!(e.span, Span::new(1, 1)); // 全链取基座起始位置
        match &e.node {
            ExprKind::Call { callee, args } => {
                assert_eq!(args.len(), 1);
                assert_eq!(args[0].node, ExprKind::Ident("x".into()));
                match &callee.node {
                    ExprKind::Index { object, index } => {
                        assert_eq!(index.node, ExprKind::Int(0));
                        match &object.node {
                            ExprKind::Field { object, name } => {
                                assert_eq!(object.node, ExprKind::Ident("a".into()));
                                assert_eq!(name, "b");
                            }
                            other => panic!("应为 Field，得到 {other:?}"),
                        }
                    }
                    other => panic!("应为 Index，得到 {other:?}"),
                }
            }
            other => panic!("应为 Call，得到 {other:?}"),
        }
    }

    // ---- 数组字面量（§7 `array_lit`）----

    #[test]
    fn array_literal_empty() {
        let p = parse(&[
            tok(TokenKind::LBracket, 1, 1),
            tok(TokenKind::RBracket, 1, 2),
            tok(TokenKind::Eof, 1, 3),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Array(elems) => assert!(elems.is_empty()),
            other => panic!("应为 Array，得到 {other:?}"),
        }
    }

    #[test]
    fn array_literal_non_empty_with_trailing_comma() {
        let p = parse(&[
            tok(TokenKind::LBracket, 1, 1),
            tok(TokenKind::Int("1".into()), 1, 2),
            tok(TokenKind::Comma, 1, 3),
            tok(TokenKind::Int("2".into()), 1, 5),
            tok(TokenKind::Comma, 1, 6),
            tok(TokenKind::RBracket, 1, 7),
            tok(TokenKind::Eof, 1, 8),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Array(elems) => {
                assert_eq!(elems.len(), 2);
                assert_eq!(elems[0].node, ExprKind::Int(1));
                assert_eq!(elems[1].node, ExprKind::Int(2));
            }
            other => panic!("应为 Array，得到 {other:?}"),
        }
    }

    #[test]
    fn array_elements_may_be_postfix_or_nested_array() {
        let p = parse(&[
            tok(TokenKind::LBracket, 1, 1),
            tok(TokenKind::Ident("f".into()), 1, 2),
            tok(TokenKind::LParen, 1, 3),
            tok(TokenKind::Int("1".into()), 1, 4),
            tok(TokenKind::RParen, 1, 5),
            tok(TokenKind::Comma, 1, 6),
            tok(TokenKind::LBracket, 1, 8),
            tok(TokenKind::RBracket, 1, 9),
            tok(TokenKind::RBracket, 1, 10),
            tok(TokenKind::Eof, 1, 11),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Array(elems) => {
                assert_eq!(elems.len(), 2);
                assert!(matches!(elems[0].node, ExprKind::Call { .. }));
                assert!(matches!(elems[1].node, ExprKind::Array(ref xs) if xs.is_empty()));
            }
            other => panic!("应为 Array，得到 {other:?}"),
        }
    }

    // ---- struct 字面量（§7 `struct_lit`；§3.3 / NO_BRACE_LITERAL）----

    #[test]
    fn struct_literal_empty_in_expr_position() {
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("x".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::LBrace, 1, 9),
            tok(TokenKind::RBrace, 1, 10),
            tok(TokenKind::Eof, 1, 11),
        ])
        .unwrap();
        match &decl_init(&p).node {
            ExprKind::StructLit(StructLit { type_name, fields }) => {
                assert_eq!(*type_name, None);
                assert!(fields.is_empty());
            }
            other => panic!("应为 StructLit，得到 {other:?}"),
        }
    }

    #[test]
    fn struct_literal_non_empty_with_fields() {
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("x".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::LBrace, 1, 9),
            tok(TokenKind::Ident("k".into()), 1, 10),
            tok(TokenKind::Colon, 1, 11),
            tok(TokenKind::Int("1".into()), 1, 13),
            tok(TokenKind::Comma, 1, 14),
            tok(TokenKind::Ident("j".into()), 1, 16),
            tok(TokenKind::Colon, 1, 17),
            tok(TokenKind::Int("2".into()), 1, 19),
            tok(TokenKind::RBrace, 1, 20),
            tok(TokenKind::Eof, 1, 21),
        ])
        .unwrap();
        match &decl_init(&p).node {
            ExprKind::StructLit(StructLit { type_name, fields }) => {
                assert_eq!(*type_name, None);
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "k");
                assert_eq!(fields[0].value.node, ExprKind::Int(1));
                assert_eq!(fields[0].span, Span::new(1, 10));
                assert_eq!(fields[1].name, "j");
                assert_eq!(fields[1].value.node, ExprKind::Int(2));
            }
            other => panic!("应为 StructLit，得到 {other:?}"),
        }
    }

    #[test]
    fn struct_literal_string_key_and_trailing_comma() {
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("x".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::LBrace, 1, 9),
            tok(TokenKind::StrBegin, 1, 10),
            tok(TokenKind::Text("k".into()), 1, 11),
            tok(TokenKind::StrEnd, 1, 12),
            tok(TokenKind::Colon, 1, 13),
            tok(TokenKind::Int("1".into()), 1, 15),
            tok(TokenKind::Comma, 1, 16),
            tok(TokenKind::RBrace, 1, 17),
            tok(TokenKind::Eof, 1, 18),
        ])
        .unwrap();
        match &decl_init(&p).node {
            ExprKind::StructLit(StructLit { fields, .. }) => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].name, "k"); // 字符串键解码为字段名
            }
            other => panic!("应为 StructLit，得到 {other:?}"),
        }
    }

    #[test]
    fn named_struct_literal_after_ident() {
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("p".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::Ident("Point".into()), 1, 9),
            tok(TokenKind::LBrace, 1, 15),
            tok(TokenKind::Ident("x".into()), 1, 16),
            tok(TokenKind::Colon, 1, 17),
            tok(TokenKind::Int("1".into()), 1, 19),
            tok(TokenKind::RBrace, 1, 20),
            tok(TokenKind::Eof, 1, 21),
        ])
        .unwrap();
        match &decl_init(&p).node {
            ExprKind::StructLit(StructLit { type_name, fields }) => {
                assert_eq!(type_name.as_deref(), Some("Point"));
                assert_eq!(fields.len(), 1);
            }
            other => panic!("应为 StructLit，得到 {other:?}"),
        }
    }

    #[test]
    fn struct_literal_as_call_argument() {
        // `f({k: 1})`：表达式位置的 `{` 是 struct 字面量（§3.3 规则 2）。
        let p = parse(&[
            tok(TokenKind::Ident("f".into()), 1, 1),
            tok(TokenKind::LParen, 1, 2),
            tok(TokenKind::LBrace, 1, 3),
            tok(TokenKind::Ident("k".into()), 1, 4),
            tok(TokenKind::Colon, 1, 5),
            tok(TokenKind::Int("1".into()), 1, 7),
            tok(TokenKind::RBrace, 1, 8),
            tok(TokenKind::RParen, 1, 9),
            tok(TokenKind::Eof, 1, 10),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Call { args, .. } => {
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0].node, ExprKind::StructLit(_)));
            }
            other => panic!("应为 Call，得到 {other:?}"),
        }
    }

    #[test]
    fn no_brace_literal_statement_start_is_block() {
        // NO_BRACE_LITERAL：语句起始处的 `{` 仍是块（本批内联进外层语句序列）。
        let p = parse(&[
            tok(TokenKind::LBrace, 1, 1),
            tok(TokenKind::KwLet, 1, 3),
            tok(TokenKind::Ident("a".into()), 1, 7),
            tok(TokenKind::Assign, 1, 9),
            tok(TokenKind::Int("1".into()), 1, 11),
            tok(TokenKind::Newline, 1, 13),
            tok(TokenKind::KwLet, 2, 3),
            tok(TokenKind::Ident("b".into()), 2, 7),
            tok(TokenKind::Assign, 2, 9),
            tok(TokenKind::Int("2".into()), 2, 11),
            tok(TokenKind::RBrace, 2, 13),
            tok(TokenKind::Eof, 2, 14),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 2);
        assert!(matches!(p.stmts[0].node, StmtKind::Decl { .. }));
        assert!(matches!(p.stmts[1].node, StmtKind::Decl { .. }));
    }

    #[test]
    fn named_struct_literal_as_statement() {
        let p = parse(&[
            tok(TokenKind::Ident("Point".into()), 1, 1),
            tok(TokenKind::LBrace, 1, 7),
            tok(TokenKind::Ident("x".into()), 1, 8),
            tok(TokenKind::Colon, 1, 9),
            tok(TokenKind::Int("1".into()), 1, 11),
            tok(TokenKind::RBrace, 1, 12),
            tok(TokenKind::Eof, 1, 13),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::StructLit(StructLit { type_name, .. }) => {
                assert_eq!(type_name.as_deref(), Some("Point"));
            }
            other => panic!("应为 StructLit，得到 {other:?}"),
        }
    }

    // ---- 错误情形（缺 `]` / `)` / `:` / 字段名）----

    #[test]
    fn array_missing_rbracket_is_error() {
        let err = parse(&[
            tok(TokenKind::LBracket, 1, 1),
            tok(TokenKind::Int("1".into()), 1, 2),
            tok(TokenKind::Comma, 1, 3),
            tok(TokenKind::Int("2".into()), 1, 5),
            tok(TokenKind::Eof, 1, 6),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "']'".to_string(),
                got: "'文件末尾'".to_string(),
            },
            1,
            6,
        );
    }

    #[test]
    fn call_missing_rparen_is_error() {
        let err = parse(&[
            tok(TokenKind::Ident("f".into()), 1, 1),
            tok(TokenKind::LParen, 1, 2),
            tok(TokenKind::Int("1".into()), 1, 3),
            tok(TokenKind::Eof, 1, 4),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "')'".to_string(),
                got: "'文件末尾'".to_string(),
            },
            1,
            4,
        );
    }

    #[test]
    fn struct_field_missing_colon_is_error() {
        let err = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("x".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::LBrace, 1, 9),
            tok(TokenKind::Ident("k".into()), 1, 10),
            tok(TokenKind::Int("1".into()), 1, 12),
            tok(TokenKind::RBrace, 1, 13),
            tok(TokenKind::Eof, 1, 14),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "':'".to_string(),
                got: "'1'".to_string(),
            },
            1,
            12,
        );
    }

    #[test]
    fn field_missing_name_is_error() {
        let err = parse(&[
            tok(TokenKind::Ident("a".into()), 1, 1),
            tok(TokenKind::Dot, 1, 2),
            tok(TokenKind::Eof, 1, 3),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "字段名".to_string(),
                got: "'文件末尾'".to_string(),
            },
            1,
            3,
        );
    }

    // ---- 二元运算符（§4.1 优先级表）----

    #[test]
    fn precedence_mul_binds_tighter_than_add() {
        // `1 + 2 * 3` → Add(1, Mul(2, 3))。
        let p = parse(&[
            tok(TokenKind::Int("1".into()), 1, 1),
            tok(TokenKind::Plus, 1, 3),
            tok(TokenKind::Int("2".into()), 1, 5),
            tok(TokenKind::Star, 1, 7),
            tok(TokenKind::Int("3".into()), 1, 9),
            tok(TokenKind::Eof, 1, 10),
        ])
        .unwrap();
        let e = expr_of(&p);
        assert_eq!(e.span, Span::new(1, 1));
        match &e.node {
            ExprKind::Binary { op, left, right } => {
                assert_eq!(*op, BinaryOp::Add);
                assert_eq!(left.node, ExprKind::Int(1));
                match &right.node {
                    ExprKind::Binary { op, left, right } => {
                        assert_eq!(*op, BinaryOp::Mul);
                        assert_eq!(left.node, ExprKind::Int(2));
                        assert_eq!(right.node, ExprKind::Int(3));
                    }
                    other => panic!("应为 Mul，得到 {other:?}"),
                }
            }
            other => panic!("应为 Add，得到 {other:?}"),
        }
    }

    #[test]
    fn associativity_subtraction_is_left() {
        // `1 - 2 - 3` → Sub(Sub(1, 2), 3)。
        let p = parse(&[
            tok(TokenKind::Int("1".into()), 1, 1),
            tok(TokenKind::Minus, 1, 3),
            tok(TokenKind::Int("2".into()), 1, 5),
            tok(TokenKind::Minus, 1, 7),
            tok(TokenKind::Int("3".into()), 1, 9),
            tok(TokenKind::Eof, 1, 10),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Binary { op, left, right } => {
                assert_eq!(*op, BinaryOp::Sub);
                assert_eq!(right.node, ExprKind::Int(3));
                match &left.node {
                    ExprKind::Binary { op, left, right } => {
                        assert_eq!(*op, BinaryOp::Sub);
                        assert_eq!(left.node, ExprKind::Int(1));
                        assert_eq!(right.node, ExprKind::Int(2));
                    }
                    other => panic!("应为 Sub，得到 {other:?}"),
                }
            }
            other => panic!("应为 Sub，得到 {other:?}"),
        }
    }

    #[test]
    fn associativity_and_is_left() {
        // `a && b && c` → And(And(a, b), c)。
        let p = parse(&[
            tok(TokenKind::Ident("a".into()), 1, 1),
            tok(TokenKind::AndAnd, 1, 3),
            tok(TokenKind::Ident("b".into()), 1, 6),
            tok(TokenKind::AndAnd, 1, 8),
            tok(TokenKind::Ident("c".into()), 1, 11),
            tok(TokenKind::Eof, 1, 12),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Logical { op, left, right } => {
                assert_eq!(*op, LogicalOp::And);
                assert_eq!(right.node, ExprKind::Ident("c".into()));
                match &left.node {
                    ExprKind::Logical { op, left, right } => {
                        assert_eq!(*op, LogicalOp::And);
                        assert_eq!(left.node, ExprKind::Ident("a".into()));
                        assert_eq!(right.node, ExprKind::Ident("b".into()));
                    }
                    other => panic!("应为 And，得到 {other:?}"),
                }
            }
            other => panic!("应为 And，得到 {other:?}"),
        }
    }

    #[test]
    fn comparison_is_looser_than_arithmetic() {
        // `1 + 2 < 3 * 4` → Lt(Add(1, 2), Mul(3, 4))。
        let p = parse(&[
            tok(TokenKind::Int("1".into()), 1, 1),
            tok(TokenKind::Plus, 1, 3),
            tok(TokenKind::Int("2".into()), 1, 5),
            tok(TokenKind::Lt, 1, 7),
            tok(TokenKind::Int("3".into()), 1, 9),
            tok(TokenKind::Star, 1, 11),
            tok(TokenKind::Int("4".into()), 1, 13),
            tok(TokenKind::Eof, 1, 14),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Binary { op, left, right } => {
                assert_eq!(*op, BinaryOp::Lt);
                assert!(matches!(left.node, ExprKind::Binary { op: BinaryOp::Add, .. }));
                assert!(matches!(right.node, ExprKind::Binary { op: BinaryOp::Mul, .. }));
            }
            other => panic!("应为 Lt，得到 {other:?}"),
        }
    }

    #[test]
    fn comparison_chain_is_left_assoc() {
        // `1 < 2 < 3` → Lt(Lt(1, 2), 3)。
        let p = parse(&[
            tok(TokenKind::Int("1".into()), 1, 1),
            tok(TokenKind::Lt, 1, 3),
            tok(TokenKind::Int("2".into()), 1, 5),
            tok(TokenKind::Lt, 1, 7),
            tok(TokenKind::Int("3".into()), 1, 9),
            tok(TokenKind::Eof, 1, 10),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Binary { op, left, right } => {
                assert_eq!(*op, BinaryOp::Lt);
                assert_eq!(right.node, ExprKind::Int(3));
                assert!(matches!(left.node, ExprKind::Binary { op: BinaryOp::Lt, .. }));
            }
            other => panic!("应为 Lt，得到 {other:?}"),
        }
    }

    #[test]
    fn equality_is_looser_than_comparison() {
        // `a == b < c` → Eq(a, Lt(b, c))。
        let p = parse(&[
            tok(TokenKind::Ident("a".into()), 1, 1),
            tok(TokenKind::EqEq, 1, 3),
            tok(TokenKind::Ident("b".into()), 1, 6),
            tok(TokenKind::Lt, 1, 8),
            tok(TokenKind::Ident("c".into()), 1, 10),
            tok(TokenKind::Eof, 1, 11),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Binary { op, left, right } => {
                assert_eq!(*op, BinaryOp::Eq);
                assert_eq!(left.node, ExprKind::Ident("a".into()));
                assert!(matches!(right.node, ExprKind::Binary { op: BinaryOp::Lt, .. }));
            }
            other => panic!("应为 Eq，得到 {other:?}"),
        }
    }

    #[test]
    fn logical_short_circuit_shape_and_precedence() {
        // `a || b && c` → Or(a, And(b, c))：`&&` 比 `||` 紧。
        let p = parse(&[
            tok(TokenKind::Ident("a".into()), 1, 1),
            tok(TokenKind::OrOr, 1, 3),
            tok(TokenKind::Ident("b".into()), 1, 6),
            tok(TokenKind::AndAnd, 1, 8),
            tok(TokenKind::Ident("c".into()), 1, 11),
            tok(TokenKind::Eof, 1, 12),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Logical { op, left, right } => {
                assert_eq!(*op, LogicalOp::Or);
                assert_eq!(left.node, ExprKind::Ident("a".into()));
                assert!(matches!(right.node, ExprKind::Logical { op: LogicalOp::And, .. }));
            }
            other => panic!("应为 Or，得到 {other:?}"),
        }
        // `a && b || c` → Or(And(a, b), c)：同级左结合。
        let p = parse(&[
            tok(TokenKind::Ident("a".into()), 1, 1),
            tok(TokenKind::AndAnd, 1, 3),
            tok(TokenKind::Ident("b".into()), 1, 6),
            tok(TokenKind::OrOr, 1, 8),
            tok(TokenKind::Ident("c".into()), 1, 11),
            tok(TokenKind::Eof, 1, 12),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Logical { op, left, right } => {
                assert_eq!(*op, LogicalOp::Or);
                assert_eq!(right.node, ExprKind::Ident("c".into()));
                assert!(matches!(left.node, ExprKind::Logical { op: LogicalOp::And, .. }));
            }
            other => panic!("应为 Or，得到 {other:?}"),
        }
    }

    #[test]
    fn unary_binds_tighter_than_binary() {
        // `-a + b` → Add(Neg(a), b)。
        let p = parse(&[
            tok(TokenKind::Minus, 1, 1),
            tok(TokenKind::Ident("a".into()), 1, 2),
            tok(TokenKind::Plus, 1, 4),
            tok(TokenKind::Ident("b".into()), 1, 6),
            tok(TokenKind::Eof, 1, 7),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Binary { op, left, right } => {
                assert_eq!(*op, BinaryOp::Add);
                assert!(matches!(left.node, ExprKind::Unary { op: UnaryOp::Neg, .. }));
                assert_eq!(right.node, ExprKind::Ident("b".into()));
            }
            other => panic!("应为 Add，得到 {other:?}"),
        }
        // `1 - -2` → Sub(1, Neg(2))：二元 `-` 的右侧仍收一元层。
        let p = parse(&[
            tok(TokenKind::Int("1".into()), 1, 1),
            tok(TokenKind::Minus, 1, 3),
            tok(TokenKind::Minus, 1, 5),
            tok(TokenKind::Int("2".into()), 1, 6),
            tok(TokenKind::Eof, 1, 7),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Binary { op, left, right } => {
                assert_eq!(*op, BinaryOp::Sub);
                assert_eq!(left.node, ExprKind::Int(1));
                assert!(matches!(right.node, ExprKind::Unary { op: UnaryOp::Neg, .. }));
            }
            other => panic!("应为 Sub，得到 {other:?}"),
        }
        // `!a && b` → And(Not(a), b)：逻辑层的操作数仍为一元层。
        let p = parse(&[
            tok(TokenKind::Bang, 1, 1),
            tok(TokenKind::Ident("a".into()), 1, 2),
            tok(TokenKind::AndAnd, 1, 4),
            tok(TokenKind::Ident("b".into()), 1, 7),
            tok(TokenKind::Eof, 1, 8),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Logical { op, left, right } => {
                assert_eq!(*op, LogicalOp::And);
                assert!(matches!(left.node, ExprKind::Unary { op: UnaryOp::Not, .. }));
                assert_eq!(right.node, ExprKind::Ident("b".into()));
            }
            other => panic!("应为 And，得到 {other:?}"),
        }
    }

    #[test]
    fn all_binary_operators_map_to_binaryop() {
        let cases: &[(TokenKind, BinaryOp)] = &[
            (TokenKind::Plus, BinaryOp::Add),
            (TokenKind::Minus, BinaryOp::Sub),
            (TokenKind::Star, BinaryOp::Mul),
            (TokenKind::Slash, BinaryOp::Div),
            (TokenKind::Percent, BinaryOp::Rem),
            (TokenKind::Lt, BinaryOp::Lt),
            (TokenKind::Le, BinaryOp::Le),
            (TokenKind::Gt, BinaryOp::Gt),
            (TokenKind::Ge, BinaryOp::Ge),
            (TokenKind::EqEq, BinaryOp::Eq),
            (TokenKind::NotEq, BinaryOp::Ne),
        ];
        for (kind, want) in cases {
            let p = parse(&[
                tok(TokenKind::Int("1".into()), 1, 1),
                tok(kind.clone(), 1, 3),
                tok(TokenKind::Int("2".into()), 1, 5),
                tok(TokenKind::Eof, 1, 6),
            ])
            .unwrap();
            match &expr_of(&p).node {
                ExprKind::Binary { op, left, right } => {
                    assert_eq!(op, want);
                    assert_eq!(left.node, ExprKind::Int(1));
                    assert_eq!(right.node, ExprKind::Int(2));
                }
                other => panic!("应为 Binary，得到 {other:?}"),
            }
        }
    }

    // ---- 赋值语句（§7 `assign_stmt` / A21）----

    /// 取程序首条语句的赋值三元组 `(target, op, value)`。
    fn assign_of(p: &Program) -> (&Lvalue, AssignOp, &Expr) {
        match &p.stmts[0].node {
            StmtKind::Assign { target, op, value } => (target, *op, value),
            other => panic!("应为 Assign，得到 {other:?}"),
        }
    }

    #[test]
    fn assign_simple_identifier() {
        // `x = 1`。
        let p = parse(&[
            tok(TokenKind::Ident("x".into()), 1, 1),
            tok(TokenKind::Assign, 1, 3),
            tok(TokenKind::Int("1".into()), 1, 5),
            tok(TokenKind::Eof, 1, 6),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 1);
        assert_eq!(p.stmts[0].span, Span::new(1, 1));
        let (target, op, value) = assign_of(&p);
        assert_eq!(op, AssignOp::Assign);
        assert_eq!(target.span, Span::new(1, 1));
        assert_eq!(target.base, LvalueBase::Name("x".into()));
        assert!(target.path.is_empty());
        assert_eq!(value.node, ExprKind::Int(1));
    }

    #[test]
    fn assign_each_compound_operator() {
        // `x += 1` / `y -= 2` / `z *= 3` / `w /= 4` / `v %= 5`。
        let p = parse(&[
            tok(TokenKind::Ident("x".into()), 1, 1),
            tok(TokenKind::PlusAssign, 1, 3),
            tok(TokenKind::Int("1".into()), 1, 6),
            tok(TokenKind::Newline, 1, 7),
            tok(TokenKind::Ident("y".into()), 2, 1),
            tok(TokenKind::MinusAssign, 2, 3),
            tok(TokenKind::Int("2".into()), 2, 6),
            tok(TokenKind::Newline, 2, 7),
            tok(TokenKind::Ident("z".into()), 3, 1),
            tok(TokenKind::StarAssign, 3, 3),
            tok(TokenKind::Int("3".into()), 3, 6),
            tok(TokenKind::Newline, 3, 7),
            tok(TokenKind::Ident("w".into()), 4, 1),
            tok(TokenKind::SlashAssign, 4, 3),
            tok(TokenKind::Int("4".into()), 4, 6),
            tok(TokenKind::Newline, 4, 7),
            tok(TokenKind::Ident("v".into()), 5, 1),
            tok(TokenKind::PercentAssign, 5, 3),
            tok(TokenKind::Int("5".into()), 5, 6),
            tok(TokenKind::Eof, 5, 7),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 5);
        let wants = [
            (AssignOp::AddAssign, "x", 1),
            (AssignOp::SubAssign, "y", 2),
            (AssignOp::MulAssign, "z", 3),
            (AssignOp::DivAssign, "w", 4),
            (AssignOp::RemAssign, "v", 5),
        ];
        for (i, (want_op, want_name, want_val)) in wants.iter().enumerate() {
            match &p.stmts[i].node {
                StmtKind::Assign { target, op, value } => {
                    assert_eq!(op, want_op);
                    assert_eq!(target.base, LvalueBase::Name((*want_name).into()));
                    assert!(target.path.is_empty());
                    assert_eq!(value.node, ExprKind::Int(*want_val));
                }
                other => panic!("第 {i} 条应为 Assign，得到 {other:?}"),
            }
        }
    }

    #[test]
    fn assign_to_index_and_field() {
        // `a[0] = 1` 与 `s.k = 2`（两条语句）。
        let p = parse(&[
            tok(TokenKind::Ident("a".into()), 1, 1),
            tok(TokenKind::LBracket, 1, 2),
            tok(TokenKind::Int("0".into()), 1, 3),
            tok(TokenKind::RBracket, 1, 4),
            tok(TokenKind::Assign, 1, 6),
            tok(TokenKind::Int("1".into()), 1, 8),
            tok(TokenKind::Newline, 1, 9),
            tok(TokenKind::Ident("s".into()), 2, 1),
            tok(TokenKind::Dot, 2, 2),
            tok(TokenKind::Ident("k".into()), 2, 3),
            tok(TokenKind::Assign, 2, 5),
            tok(TokenKind::Int("2".into()), 2, 7),
            tok(TokenKind::Eof, 2, 8),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 2);
        // 第一句：a[0] = 1。
        match &p.stmts[0].node {
            StmtKind::Assign { target, op, value } => {
                assert_eq!(*op, AssignOp::Assign);
                assert_eq!(target.base, LvalueBase::Name("a".into()));
                assert_eq!(target.path.len(), 1);
                assert_eq!(target.path[0].span, Span::new(1, 2)); // `[` 的位置
                match &target.path[0].kind {
                    LvalueSegKind::Index(e) => assert_eq!(e.node, ExprKind::Int(0)),
                    other => panic!("应为 Index，得到 {other:?}"),
                }
                assert_eq!(value.node, ExprKind::Int(1));
            }
            other => panic!("应为 Assign，得到 {other:?}"),
        }
        // 第二句：s.k = 2。
        match &p.stmts[1].node {
            StmtKind::Assign { target, op, value } => {
                assert_eq!(*op, AssignOp::Assign);
                assert_eq!(target.base, LvalueBase::Name("s".into()));
                assert_eq!(target.path.len(), 1);
                assert_eq!(target.path[0].span, Span::new(2, 2)); // `.` 的位置
                match &target.path[0].kind {
                    LvalueSegKind::Field(f) => assert_eq!(f, "k"),
                    other => panic!("应为 Field，得到 {other:?}"),
                }
                assert_eq!(value.node, ExprKind::Int(2));
            }
            other => panic!("应为 Assign，得到 {other:?}"),
        }
    }

    #[test]
    fn assign_nested_chain_with_compound_op() {
        // `a.b[0] += 1`：目标 = Name(a) + Field(b) + Index(0)。
        let p = parse(&[
            tok(TokenKind::Ident("a".into()), 1, 1),
            tok(TokenKind::Dot, 1, 2),
            tok(TokenKind::Ident("b".into()), 1, 3),
            tok(TokenKind::LBracket, 1, 4),
            tok(TokenKind::Int("0".into()), 1, 5),
            tok(TokenKind::RBracket, 1, 6),
            tok(TokenKind::PlusAssign, 1, 8),
            tok(TokenKind::Int("1".into()), 1, 11),
            tok(TokenKind::Eof, 1, 12),
        ])
        .unwrap();
        let (target, op, value) = assign_of(&p);
        assert_eq!(op, AssignOp::AddAssign);
        assert_eq!(target.span, Span::new(1, 1));
        assert_eq!(target.base, LvalueBase::Name("a".into()));
        assert_eq!(target.path.len(), 2);
        assert_eq!(target.path[0].span, Span::new(1, 2)); // `.`
        assert_eq!(target.path[1].span, Span::new(1, 4)); // `[`
        match &target.path[0].kind {
            LvalueSegKind::Field(f) => assert_eq!(f, "b"),
            other => panic!("应为 Field，得到 {other:?}"),
        }
        match &target.path[1].kind {
            LvalueSegKind::Index(e) => assert_eq!(e.node, ExprKind::Int(0)),
            other => panic!("应为 Index，得到 {other:?}"),
        }
        assert_eq!(value.node, ExprKind::Int(1));
    }

    #[test]
    fn assign_to_self_field() {
        // `self.count = 1`：基座 SelfValue（A21）。
        let p = parse(&[
            tok(TokenKind::KwSelf, 1, 1),
            tok(TokenKind::Dot, 1, 5),
            tok(TokenKind::Ident("count".into()), 1, 6),
            tok(TokenKind::Assign, 1, 12),
            tok(TokenKind::Int("1".into()), 1, 14),
            tok(TokenKind::Eof, 1, 15),
        ])
        .unwrap();
        let (target, op, value) = assign_of(&p);
        assert_eq!(op, AssignOp::Assign);
        assert_eq!(target.base, LvalueBase::SelfValue);
        assert_eq!(target.path.len(), 1);
        match &target.path[0].kind {
            LvalueSegKind::Field(f) => assert_eq!(f, "count"),
            other => panic!("应为 Field，得到 {other:?}"),
        }
        assert_eq!(value.node, ExprKind::Int(1));
    }

    #[test]
    fn assign_value_may_be_binary_expr() {
        // `x = 1 + 2 * 3`：右值走完整表达式层。
        let p = parse(&[
            tok(TokenKind::Ident("x".into()), 1, 1),
            tok(TokenKind::Assign, 1, 3),
            tok(TokenKind::Int("1".into()), 1, 5),
            tok(TokenKind::Plus, 1, 7),
            tok(TokenKind::Int("2".into()), 1, 9),
            tok(TokenKind::Star, 1, 11),
            tok(TokenKind::Int("3".into()), 1, 13),
            tok(TokenKind::Eof, 1, 14),
        ])
        .unwrap();
        let (_, op, value) = assign_of(&p);
        assert_eq!(op, AssignOp::Assign);
        match &value.node {
            ExprKind::Binary { op, right, .. } => {
                assert_eq!(*op, BinaryOp::Add);
                assert!(matches!(right.node, ExprKind::Binary { op: BinaryOp::Mul, .. }));
            }
            other => panic!("应为 Add，得到 {other:?}"),
        }
    }

    #[test]
    fn invalid_assign_target_call() {
        // `f(x) = 1`：调用不能作为赋值目标（A21）。
        let err = parse(&[
            tok(TokenKind::Ident("f".into()), 1, 1),
            tok(TokenKind::LParen, 1, 2),
            tok(TokenKind::Ident("x".into()), 1, 3),
            tok(TokenKind::RParen, 1, 4),
            tok(TokenKind::Assign, 1, 6),
            tok(TokenKind::Int("1".into()), 1, 8),
            tok(TokenKind::Eof, 1, 9),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::InvalidAssignTarget, 1, 1);
    }

    #[test]
    fn invalid_assign_target_binary() {
        // `1 + 2 = 3`：二元表达式不能作为赋值目标。
        let err = parse(&[
            tok(TokenKind::Int("1".into()), 1, 1),
            tok(TokenKind::Plus, 1, 3),
            tok(TokenKind::Int("2".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::Int("3".into()), 1, 9),
            tok(TokenKind::Eof, 1, 10),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::InvalidAssignTarget, 1, 1);
    }

    #[test]
    fn invalid_assign_target_literal() {
        // `1 = 2`：字面量不能作为赋值目标。
        let err = parse(&[
            tok(TokenKind::Int("1".into()), 1, 1),
            tok(TokenKind::Assign, 1, 3),
            tok(TokenKind::Int("2".into()), 1, 5),
            tok(TokenKind::Eof, 1, 6),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::InvalidAssignTarget, 1, 1);
    }

    // ---- 回归：二元运算进入既有语法位（声明初值 / 实参 / 下标 / 字段值）----

    #[test]
    fn binary_in_decl_init() {
        // `let x = a + b`。
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("x".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::Ident("a".into()), 1, 9),
            tok(TokenKind::Plus, 1, 11),
            tok(TokenKind::Ident("b".into()), 1, 13),
            tok(TokenKind::Eof, 1, 14),
        ])
        .unwrap();
        match &decl_init(&p).node {
            ExprKind::Binary { op, left, right } => {
                assert_eq!(*op, BinaryOp::Add);
                assert_eq!(left.node, ExprKind::Ident("a".into()));
                assert_eq!(right.node, ExprKind::Ident("b".into()));
            }
            other => panic!("应为 Add，得到 {other:?}"),
        }
    }

    #[test]
    fn binary_in_call_args_index_and_field_value() {
        // `f(a + b, i * 2)`：实参中的二元运算。
        let p = parse(&[
            tok(TokenKind::Ident("f".into()), 1, 1),
            tok(TokenKind::LParen, 1, 2),
            tok(TokenKind::Ident("a".into()), 1, 3),
            tok(TokenKind::Plus, 1, 5),
            tok(TokenKind::Ident("b".into()), 1, 7),
            tok(TokenKind::Comma, 1, 8),
            tok(TokenKind::Ident("i".into()), 1, 10),
            tok(TokenKind::Star, 1, 12),
            tok(TokenKind::Int("2".into()), 1, 14),
            tok(TokenKind::RParen, 1, 15),
            tok(TokenKind::Eof, 1, 16),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Call { args, .. } => {
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0].node, ExprKind::Binary { op: BinaryOp::Add, .. }));
                assert!(matches!(args[1].node, ExprKind::Binary { op: BinaryOp::Mul, .. }));
            }
            other => panic!("应为 Call，得到 {other:?}"),
        }
        // `a[i + 1]`：下标中的二元运算。
        let p = parse(&[
            tok(TokenKind::Ident("a".into()), 1, 1),
            tok(TokenKind::LBracket, 1, 2),
            tok(TokenKind::Ident("i".into()), 1, 3),
            tok(TokenKind::Plus, 1, 5),
            tok(TokenKind::Int("1".into()), 1, 7),
            tok(TokenKind::RBracket, 1, 8),
            tok(TokenKind::Eof, 1, 9),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::Index { index, .. } => {
                assert!(matches!(index.node, ExprKind::Binary { op: BinaryOp::Add, .. }));
            }
            other => panic!("应为 Index，得到 {other:?}"),
        }
        // `let x = { k: 1 + 2 }`：struct 字面量字段值中的二元运算。
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("x".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::LBrace, 1, 9),
            tok(TokenKind::Ident("k".into()), 1, 10),
            tok(TokenKind::Colon, 1, 11),
            tok(TokenKind::Int("1".into()), 1, 13),
            tok(TokenKind::Plus, 1, 15),
            tok(TokenKind::Int("2".into()), 1, 17),
            tok(TokenKind::RBrace, 1, 18),
            tok(TokenKind::Eof, 1, 19),
        ])
        .unwrap();
        match &decl_init(&p).node {
            ExprKind::StructLit(StructLit { fields, .. }) => {
                assert_eq!(fields.len(), 1);
                assert!(matches!(fields[0].value.node, ExprKind::Binary { op: BinaryOp::Add, .. }));
            }
            other => panic!("应为 StructLit，得到 {other:?}"),
        }
    }

    #[test]
    fn ident_before_brace_still_struct_lit_after_assign_attempt() {
        // 赋值试探回滚后，`Point { x: 1 }` 语句仍须解析为具名 struct 字面量。
        let p = parse(&[
            tok(TokenKind::Ident("Point".into()), 1, 1),
            tok(TokenKind::LBrace, 1, 7),
            tok(TokenKind::Ident("x".into()), 1, 8),
            tok(TokenKind::Colon, 1, 9),
            tok(TokenKind::Int("1".into()), 1, 11),
            tok(TokenKind::RBrace, 1, 12),
            tok(TokenKind::Eof, 1, 13),
        ])
        .unwrap();
        match &expr_of(&p).node {
            ExprKind::StructLit(StructLit { type_name, .. }) => {
                assert_eq!(type_name.as_deref(), Some("Point"));
            }
            other => panic!("应为 StructLit，得到 {other:?}"),
        }
    }

    // ---- 控制流语句（§7：if / while / for / return / break / continue）----

    #[test]
    fn if_without_else() {
        // `if c { x }`：then 块；无 else 分支。
        let p = parse(&[
            tok(TokenKind::KwIf, 1, 1),
            tok(TokenKind::Ident("c".into()), 1, 4),
            tok(TokenKind::LBrace, 1, 6),
            tok(TokenKind::Ident("x".into()), 1, 8),
            tok(TokenKind::RBrace, 1, 10),
            tok(TokenKind::Eof, 1, 11),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 1);
        assert_eq!(p.stmts[0].span, Span::new(1, 1));
        match &p.stmts[0].node {
            StmtKind::If(e) => {
                assert_eq!(e.span, Span::new(1, 1));
                match &e.node {
                    ExprKind::If(IfExpr { cond, then_block, else_branch }) => {
                        assert_eq!(cond.node, ExprKind::Ident("c".into()));
                        assert_eq!(then_block.span, Span::new(1, 6));
                        assert_eq!(then_block.stmts.len(), 1);
                        assert!(matches!(then_block.stmts[0].node, StmtKind::Expr(_)));
                        assert!(else_branch.is_none());
                    }
                    other => panic!("应为 If 表达式，得到 {other:?}"),
                }
            }
            other => panic!("应为 If 语句，得到 {other:?}"),
        }
    }

    #[test]
    fn if_else_block_branch() {
        // `if a { 1 } else { 2 }`。
        let p = parse(&[
            tok(TokenKind::KwIf, 1, 1),
            tok(TokenKind::Ident("a".into()), 1, 4),
            tok(TokenKind::LBrace, 1, 6),
            tok(TokenKind::Int("1".into()), 1, 8),
            tok(TokenKind::RBrace, 1, 10),
            tok(TokenKind::KwElse, 1, 12),
            tok(TokenKind::LBrace, 1, 17),
            tok(TokenKind::Int("2".into()), 1, 19),
            tok(TokenKind::RBrace, 1, 21),
            tok(TokenKind::Eof, 1, 22),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 1);
        match &p.stmts[0].node {
            StmtKind::If(e) => match &e.node {
                ExprKind::If(IfExpr { else_branch, .. }) => match else_branch {
                    Some(ElseBranch::Block(b)) => {
                        assert_eq!(b.span, Span::new(1, 17));
                        assert_eq!(b.stmts.len(), 1);
                    }
                    other => panic!("应为 else 块，得到 {other:?}"),
                },
                other => panic!("应为 If 表达式，得到 {other:?}"),
            },
            other => panic!("应为 If 语句，得到 {other:?}"),
        }
    }

    #[test]
    fn if_else_if_chain() {
        // `if a {1} else if b {2} else {3}`（A4：else if 链绑定最近 if）。
        let p = parse(&[
            tok(TokenKind::KwIf, 1, 1),
            tok(TokenKind::Ident("a".into()), 1, 4),
            tok(TokenKind::LBrace, 1, 6),
            tok(TokenKind::Int("1".into()), 1, 7),
            tok(TokenKind::RBrace, 1, 8),
            tok(TokenKind::KwElse, 1, 10),
            tok(TokenKind::KwIf, 1, 15),
            tok(TokenKind::Ident("b".into()), 1, 18),
            tok(TokenKind::LBrace, 1, 20),
            tok(TokenKind::Int("2".into()), 1, 21),
            tok(TokenKind::RBrace, 1, 22),
            tok(TokenKind::KwElse, 1, 24),
            tok(TokenKind::LBrace, 1, 29),
            tok(TokenKind::Int("3".into()), 1, 30),
            tok(TokenKind::RBrace, 1, 31),
            tok(TokenKind::Eof, 1, 32),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::If(e) => match &e.node {
                ExprKind::If(IfExpr { else_branch, .. }) => {
                    let inner = match else_branch {
                        Some(ElseBranch::If(inner)) => inner,
                        other => panic!("应为 else if，得到 {other:?}"),
                    };
                    assert_eq!(inner.span, Span::new(1, 15)); // 内层 `if` 关键字位置
                    match &inner.node {
                        ExprKind::If(IfExpr { cond, else_branch, .. }) => {
                            assert_eq!(cond.node, ExprKind::Ident("b".into()));
                            assert!(matches!(else_branch, Some(ElseBranch::Block(_))));
                        }
                        other => panic!("应为 If 表达式，得到 {other:?}"),
                    }
                }
                other => panic!("应为 If 表达式，得到 {other:?}"),
            },
            other => panic!("应为 If 语句，得到 {other:?}"),
        }
    }

    #[test]
    fn if_newline_before_block_and_else() {
        // `if c\n{ 1 }\nelse\n{ 2 }`（§3.5：块前换行与 else 前换行均跳过）。
        let p = parse(&[
            tok(TokenKind::KwIf, 1, 1),
            tok(TokenKind::Ident("c".into()), 1, 4),
            tok(TokenKind::Newline, 1, 5),
            tok(TokenKind::LBrace, 2, 1),
            tok(TokenKind::Int("1".into()), 2, 3),
            tok(TokenKind::RBrace, 2, 5),
            tok(TokenKind::Newline, 2, 6),
            tok(TokenKind::KwElse, 3, 1),
            tok(TokenKind::Newline, 3, 5),
            tok(TokenKind::LBrace, 4, 1),
            tok(TokenKind::Int("2".into()), 4, 3),
            tok(TokenKind::RBrace, 4, 5),
            tok(TokenKind::Eof, 4, 6),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 1);
        match &p.stmts[0].node {
            StmtKind::If(e) => match &e.node {
                ExprKind::If(IfExpr { then_block, else_branch, .. }) => {
                    assert_eq!(then_block.span, Span::new(2, 1));
                    assert!(matches!(
                        else_branch,
                        Some(ElseBranch::Block(ref b)) if b.span == Span::new(4, 1)
                    ));
                }
                other => panic!("应为 If 表达式，得到 {other:?}"),
            },
            other => panic!("应为 If 语句，得到 {other:?}"),
        }
    }

    #[test]
    fn while_loop() {
        // `while i < 10 { i += 1 }`。
        let p = parse(&[
            tok(TokenKind::KwWhile, 1, 1),
            tok(TokenKind::Ident("i".into()), 1, 7),
            tok(TokenKind::Lt, 1, 9),
            tok(TokenKind::Int("10".into()), 1, 11),
            tok(TokenKind::LBrace, 1, 14),
            tok(TokenKind::Ident("i".into()), 1, 16),
            tok(TokenKind::PlusAssign, 1, 18),
            tok(TokenKind::Int("1".into()), 1, 21),
            tok(TokenKind::RBrace, 1, 23),
            tok(TokenKind::Eof, 1, 24),
        ])
        .unwrap();
        assert_eq!(p.stmts[0].span, Span::new(1, 1));
        match &p.stmts[0].node {
            StmtKind::While { cond, body } => {
                assert_eq!(cond.span, Span::new(1, 7));
                match &cond.node {
                    ExprKind::Binary { op, left, right } => {
                        assert_eq!(*op, BinaryOp::Lt);
                        assert_eq!(left.node, ExprKind::Ident("i".into()));
                        assert_eq!(right.node, ExprKind::Int(10));
                    }
                    other => panic!("应为 Lt，得到 {other:?}"),
                }
                assert_eq!(body.span, Span::new(1, 14));
                assert_eq!(body.stmts.len(), 1);
                assert!(matches!(body.stmts[0].node, StmtKind::Assign { .. }));
            }
            other => panic!("应为 While，得到 {other:?}"),
        }
    }

    #[test]
    fn for_in_loop() {
        // `for s in xs { print(s) }`。
        let p = parse(&[
            tok(TokenKind::KwFor, 1, 1),
            tok(TokenKind::Ident("s".into()), 1, 5),
            tok(TokenKind::KwIn, 1, 7),
            tok(TokenKind::Ident("xs".into()), 1, 10),
            tok(TokenKind::LBrace, 1, 13),
            tok(TokenKind::Ident("print".into()), 1, 15),
            tok(TokenKind::LParen, 1, 20),
            tok(TokenKind::Ident("s".into()), 1, 21),
            tok(TokenKind::RParen, 1, 22),
            tok(TokenKind::RBrace, 1, 24),
            tok(TokenKind::Eof, 1, 25),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 1);
        match &p.stmts[0].node {
            StmtKind::For { var, iter, body } => {
                assert_eq!(var, "s");
                assert_eq!(iter.node, ExprKind::Ident("xs".into()));
                assert_eq!(body.span, Span::new(1, 13));
                assert_eq!(body.stmts.len(), 1);
                assert!(matches!(body.stmts[0].node, StmtKind::Expr(_)));
            }
            other => panic!("应为 For，得到 {other:?}"),
        }
    }

    #[test]
    fn break_and_continue_inside_loop() {
        // `while true { break }` 与 `for i in xs { continue }`。
        let p = parse(&[
            tok(TokenKind::KwWhile, 1, 1),
            tok(TokenKind::KwTrue, 1, 7),
            tok(TokenKind::LBrace, 1, 12),
            tok(TokenKind::KwBreak, 1, 14),
            tok(TokenKind::RBrace, 1, 19),
            tok(TokenKind::Newline, 1, 20),
            tok(TokenKind::KwFor, 2, 1),
            tok(TokenKind::Ident("i".into()), 2, 5),
            tok(TokenKind::KwIn, 2, 7),
            tok(TokenKind::Ident("xs".into()), 2, 10),
            tok(TokenKind::LBrace, 2, 13),
            tok(TokenKind::KwContinue, 2, 15),
            tok(TokenKind::RBrace, 2, 23),
            tok(TokenKind::Eof, 2, 24),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 2);
        match &p.stmts[0].node {
            StmtKind::While { body, .. } => {
                assert_eq!(body.stmts.len(), 1);
                assert!(matches!(body.stmts[0].node, StmtKind::Break));
            }
            other => panic!("应为 While，得到 {other:?}"),
        }
        match &p.stmts[1].node {
            StmtKind::For { body, .. } => {
                assert_eq!(body.stmts.len(), 1);
                assert!(matches!(body.stmts[0].node, StmtKind::Continue));
            }
            other => panic!("应为 For，得到 {other:?}"),
        }
    }

    #[test]
    fn break_inside_if_inside_loop_is_ok() {
        // `while c { if d { break } }`：break 合法性只看循环深度，if 不影响。
        let p = parse(&[
            tok(TokenKind::KwWhile, 1, 1),
            tok(TokenKind::Ident("c".into()), 1, 7),
            tok(TokenKind::LBrace, 1, 9),
            tok(TokenKind::KwIf, 1, 11),
            tok(TokenKind::Ident("d".into()), 1, 14),
            tok(TokenKind::LBrace, 1, 16),
            tok(TokenKind::KwBreak, 1, 18),
            tok(TokenKind::RBrace, 1, 23),
            tok(TokenKind::RBrace, 1, 25),
            tok(TokenKind::Eof, 1, 26),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::While { body, .. } => {
                assert_eq!(body.stmts.len(), 1);
                match &body.stmts[0].node {
                    StmtKind::If(e) => match &e.node {
                        ExprKind::If(IfExpr { then_block, .. }) => {
                            assert!(matches!(then_block.stmts[0].node, StmtKind::Break));
                        }
                        other => panic!("应为 If 表达式，得到 {other:?}"),
                    },
                    other => panic!("应为 If 语句，得到 {other:?}"),
                }
            }
            other => panic!("应为 While，得到 {other:?}"),
        }
    }

    #[test]
    fn return_with_and_without_value_in_function() {
        // `fn` 声明属下一批：直置 `fn_depth = 1` 模拟函数体，验证
        // `return x`（带值）与行尾 `return`（A19：返回 nil）两种形态。
        let toks = vec![
            tok(TokenKind::KwReturn, 1, 1),
            tok(TokenKind::Ident("x".into()), 1, 8),
            tok(TokenKind::Newline, 1, 9),
            tok(TokenKind::KwReturn, 2, 1),
            tok(TokenKind::Newline, 2, 7),
            tok(TokenKind::Eof, 3, 1),
        ];
        let mut parser = Parser::new(&toks);
        parser.fn_depth = 1;
        let p = parser.parse_program().unwrap();
        assert_eq!(p.stmts.len(), 2);
        match &p.stmts[0].node {
            StmtKind::Return(Some(e)) => assert_eq!(e.node, ExprKind::Ident("x".into())),
            other => panic!("应为 Return(Some)，得到 {other:?}"),
        }
        assert_eq!(p.stmts[0].span, Span::new(1, 1));
        match &p.stmts[1].node {
            StmtKind::Return(None) => {}
            other => panic!("应为 Return(None)，得到 {other:?}"),
        }
    }

    #[test]
    fn break_outside_loop_is_error() {
        let err = parse(&[
            tok(TokenKind::KwBreak, 1, 1),
            tok(TokenKind::Eof, 1, 6),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::BreakContinueOutsideLoop { kw: "break".to_string() },
            1,
            1,
        );
    }

    #[test]
    fn continue_outside_loop_is_error() {
        let err = parse(&[
            tok(TokenKind::KwContinue, 1, 1),
            tok(TokenKind::Eof, 1, 9),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::BreakContinueOutsideLoop { kw: "continue".to_string() },
            1,
            1,
        );
    }

    #[test]
    fn return_outside_function_is_error() {
        let err = parse(&[
            tok(TokenKind::KwReturn, 1, 1),
            tok(TokenKind::Eof, 1, 7),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::ReturnOutsideFunction, 1, 1);
    }

    // ---- NO_BRACE_LITERAL（§3.4：条件 / 可迭代位最外层禁用裸 struct 字面量）----

    #[test]
    fn bare_struct_literal_condition_is_incomplete_expr() {
        // `if { k: 1 }`：最外层裸 `{` 终止条件 → 条件缺失 → IncompleteExpr。
        let err = parse(&[
            tok(TokenKind::KwIf, 1, 1),
            tok(TokenKind::LBrace, 1, 4),
            tok(TokenKind::Ident("k".into()), 1, 6),
            tok(TokenKind::Colon, 1, 7),
            tok(TokenKind::Int("1".into()), 1, 9),
            tok(TokenKind::RBrace, 1, 10),
            tok(TokenKind::Eof, 1, 11),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::IncompleteExpr, 1, 4);
    }

    #[test]
    fn struct_literal_in_parenthesized_condition_is_ok() {
        // `if ({ k: 1 } == nil) { }`：括号解除 NO_BRACE_LITERAL（§3.4 / A6）。
        let p = parse(&[
            tok(TokenKind::KwIf, 1, 1),
            tok(TokenKind::LParen, 1, 4),
            tok(TokenKind::LBrace, 1, 5),
            tok(TokenKind::Ident("k".into()), 1, 7),
            tok(TokenKind::Colon, 1, 8),
            tok(TokenKind::Int("1".into()), 1, 10),
            tok(TokenKind::RBrace, 1, 11),
            tok(TokenKind::EqEq, 1, 13),
            tok(TokenKind::KwNil, 1, 16),
            tok(TokenKind::RParen, 1, 19),
            tok(TokenKind::LBrace, 1, 21),
            tok(TokenKind::RBrace, 1, 22),
            tok(TokenKind::Eof, 1, 23),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::If(e) => match &e.node {
                ExprKind::If(IfExpr { cond, .. }) => match &cond.node {
                    ExprKind::Binary { op, left, .. } => {
                        assert_eq!(*op, BinaryOp::Eq);
                        assert!(matches!(left.node, ExprKind::StructLit(_)));
                    }
                    other => panic!("应为 Eq，得到 {other:?}"),
                },
                other => panic!("应为 If 表达式，得到 {other:?}"),
            },
            other => panic!("应为 If 语句，得到 {other:?}"),
        }
    }

    #[test]
    fn named_struct_brace_in_condition_goes_to_block() {
        // `if Point { x: 1 }`：`Point` 后 `{` 交块（§3.4），块内 `x: 1` 两句
        // 无换行 → TwoStatements（A6 反例，SyntaxError）。
        let err = parse(&[
            tok(TokenKind::KwIf, 1, 1),
            tok(TokenKind::Ident("Point".into()), 1, 4),
            tok(TokenKind::LBrace, 1, 10),
            tok(TokenKind::Ident("x".into()), 1, 12),
            tok(TokenKind::Colon, 1, 13),
            tok(TokenKind::Int("1".into()), 1, 15),
            tok(TokenKind::RBrace, 1, 16),
            tok(TokenKind::Eof, 1, 17),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::TwoStatements, 1, 13);
    }

    // ---- 回归：控制流与既有构造（声明 / 赋值 / 二元 / 调用 / 后缀链）混用 ----

    #[test]
    fn control_flow_regression_with_existing_constructs() {
        // let n = 0
        // while n < 3 {
        //     n += 1
        //     if n == 2 { break }
        // }
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("n".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::Int("0".into()), 1, 9),
            tok(TokenKind::Newline, 1, 10),
            tok(TokenKind::KwWhile, 2, 1),
            tok(TokenKind::Ident("n".into()), 2, 7),
            tok(TokenKind::Lt, 2, 9),
            tok(TokenKind::Int("3".into()), 2, 11),
            tok(TokenKind::LBrace, 2, 13),
            tok(TokenKind::Ident("n".into()), 2, 15),
            tok(TokenKind::PlusAssign, 2, 17),
            tok(TokenKind::Int("1".into()), 2, 20),
            tok(TokenKind::Newline, 2, 21),
            tok(TokenKind::KwIf, 3, 3),
            tok(TokenKind::Ident("n".into()), 3, 6),
            tok(TokenKind::EqEq, 3, 8),
            tok(TokenKind::Int("2".into()), 3, 11),
            tok(TokenKind::LBrace, 3, 13),
            tok(TokenKind::KwBreak, 3, 15),
            tok(TokenKind::RBrace, 3, 20),
            tok(TokenKind::RBrace, 3, 22),
            tok(TokenKind::Eof, 3, 23),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 2);
        assert!(matches!(p.stmts[0].node, StmtKind::Decl { .. }));
        match &p.stmts[1].node {
            StmtKind::While { cond, body } => {
                assert!(matches!(cond.node, ExprKind::Binary { op: BinaryOp::Lt, .. }));
                assert_eq!(body.stmts.len(), 2);
                assert!(matches!(body.stmts[0].node, StmtKind::Assign { .. }));
                match &body.stmts[1].node {
                    StmtKind::If(e) => match &e.node {
                        ExprKind::If(IfExpr { then_block, else_branch, .. }) => {
                            assert!(matches!(then_block.stmts[0].node, StmtKind::Break));
                            assert!(else_branch.is_none());
                        }
                        other => panic!("应为 If 表达式，得到 {other:?}"),
                    },
                    other => panic!("应为 If 语句，得到 {other:?}"),
                }
            }
            other => panic!("应为 While，得到 {other:?}"),
        }
    }

    // ---- fn 声明（§7 fn_decl）----

    #[test]
    fn fn_decl_zero_params_block_body() {
        // fn f() { 1 }
        let p = parse(&[
            tok(TokenKind::KwFn, 1, 1),
            tok(TokenKind::Ident("f".into()), 1, 4),
            tok(TokenKind::LParen, 1, 5),
            tok(TokenKind::RParen, 1, 6),
            tok(TokenKind::LBrace, 1, 8),
            tok(TokenKind::Int("1".into()), 1, 10),
            tok(TokenKind::RBrace, 1, 11),
            tok(TokenKind::Eof, 1, 12),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 1);
        assert_eq!(p.stmts[0].span, Span::new(1, 1));
        match &p.stmts[0].node {
            StmtKind::FnDecl(FnDecl { span, name, params, body }) => {
                assert_eq!(*span, Span::new(1, 1));
                assert_eq!(name, "f");
                assert!(params.is_empty());
                match body {
                    Body::Block(block) => {
                        assert_eq!(block.span, Span::new(1, 8));
                        assert_eq!(block.stmts.len(), 1);
                        assert_eq!(block.stmts[0].node, StmtKind::Expr(Spanned::new(ExprKind::Int(1), Span::new(1, 10))));
                    }
                    other => panic!("应为块体，得到 {other:?}"),
                }
            }
            other => panic!("应为 FnDecl，得到 {other:?}"),
        }
    }

    #[test]
    fn fn_decl_one_param_arrow_expr() {
        // fn double(x) => x * 2
        let p = parse(&[
            tok(TokenKind::KwFn, 1, 1),
            tok(TokenKind::Ident("double".into()), 1, 4),
            tok(TokenKind::LParen, 1, 10),
            tok(TokenKind::Ident("x".into()), 1, 11),
            tok(TokenKind::RParen, 1, 12),
            tok(TokenKind::Arrow, 1, 14),
            tok(TokenKind::Ident("x".into()), 1, 17),
            tok(TokenKind::Star, 1, 19),
            tok(TokenKind::Int("2".into()), 1, 21),
            tok(TokenKind::Eof, 1, 22),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::FnDecl(FnDecl { name, params, body, .. }) => {
                assert_eq!(name, "double");
                assert_eq!(params, &vec!["x".to_string()]);
                match body {
                    Body::Expr(e) => match &e.node {
                        ExprKind::Binary { op, left, right } => {
                            assert_eq!(*op, BinaryOp::Mul);
                            assert!(matches!(left.node, ExprKind::Ident(ref n) if n == "x"));
                            assert_eq!(right.node, ExprKind::Int(2));
                        }
                        other => panic!("应为 Mul，得到 {other:?}"),
                    },
                    other => panic!("应为箭头表达式体，得到 {other:?}"),
                }
            }
            other => panic!("应为 FnDecl，得到 {other:?}"),
        }
    }

    #[test]
    fn fn_decl_many_params_arrow_block_is_block_by_m3() {
        // fn add3(a, b, c) => { a + b + c } —— M3：`=> {` 恒为块体。
        let p = parse(&[
            tok(TokenKind::KwFn, 1, 1),
            tok(TokenKind::Ident("add3".into()), 1, 4),
            tok(TokenKind::LParen, 1, 8),
            tok(TokenKind::Ident("a".into()), 1, 9),
            tok(TokenKind::Comma, 1, 10),
            tok(TokenKind::Ident("b".into()), 1, 12),
            tok(TokenKind::Comma, 1, 13),
            tok(TokenKind::Ident("c".into()), 1, 15),
            tok(TokenKind::RParen, 1, 16),
            tok(TokenKind::Arrow, 1, 18),
            tok(TokenKind::LBrace, 1, 21),
            tok(TokenKind::Ident("a".into()), 1, 23),
            tok(TokenKind::Plus, 1, 25),
            tok(TokenKind::Ident("b".into()), 1, 27),
            tok(TokenKind::Plus, 1, 29),
            tok(TokenKind::Ident("c".into()), 1, 31),
            tok(TokenKind::RBrace, 1, 32),
            tok(TokenKind::Eof, 1, 33),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::FnDecl(FnDecl { name, params, body, .. }) => {
                assert_eq!(name, "add3");
                assert_eq!(params, &vec!["a".to_string(), "b".to_string(), "c".to_string()]);
                match body {
                    Body::Block(block) => {
                        assert_eq!(block.span, Span::new(1, 21));
                        assert_eq!(block.stmts.len(), 1);
                        match &block.stmts[0].node {
                            StmtKind::Expr(e) => {
                                assert!(matches!(e.node, ExprKind::Binary { op: BinaryOp::Add, .. }));
                            }
                            other => panic!("应为 Expr，得到 {other:?}"),
                        }
                    }
                    other => panic!("应为块体（M3），得到 {other:?}"),
                }
            }
            other => panic!("应为 FnDecl，得到 {other:?}"),
        }
    }

    #[test]
    fn fn_decl_newline_before_block_and_nested_reused_param() {
        // fn outer(x)
        // {
        //     fn inner(x) => x + 1
        //     inner(x)
        // }
        // §3.5 块前换行；§7 无形参重名限制 → 嵌套 fn 可复用外层形参名。
        let p = parse(&[
            tok(TokenKind::KwFn, 1, 1),
            tok(TokenKind::Ident("outer".into()), 1, 4),
            tok(TokenKind::LParen, 1, 9),
            tok(TokenKind::Ident("x".into()), 1, 10),
            tok(TokenKind::RParen, 1, 11),
            tok(TokenKind::Newline, 1, 12),
            tok(TokenKind::LBrace, 2, 1),
            tok(TokenKind::KwFn, 3, 5),
            tok(TokenKind::Ident("inner".into()), 3, 8),
            tok(TokenKind::LParen, 3, 13),
            tok(TokenKind::Ident("x".into()), 3, 14),
            tok(TokenKind::RParen, 3, 15),
            tok(TokenKind::Arrow, 3, 17),
            tok(TokenKind::Ident("x".into()), 3, 20),
            tok(TokenKind::Plus, 3, 22),
            tok(TokenKind::Int("1".into()), 3, 24),
            tok(TokenKind::Newline, 3, 25),
            tok(TokenKind::Ident("inner".into()), 4, 5),
            tok(TokenKind::LParen, 4, 10),
            tok(TokenKind::Ident("x".into()), 4, 11),
            tok(TokenKind::RParen, 4, 12),
            tok(TokenKind::Newline, 4, 13),
            tok(TokenKind::RBrace, 5, 1),
            tok(TokenKind::Eof, 5, 2),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::FnDecl(FnDecl { name, params, body, .. }) => {
                assert_eq!(name, "outer");
                assert_eq!(params, &vec!["x".to_string()]);
                match body {
                    Body::Block(block) => {
                        assert_eq!(block.stmts.len(), 2);
                        match &block.stmts[0].node {
                            StmtKind::FnDecl(inner) => {
                                assert_eq!(inner.name, "inner");
                                assert_eq!(inner.params, vec!["x".to_string()]);
                                assert!(matches!(&inner.body, Body::Expr(_)));
                            }
                            other => panic!("应为嵌套 FnDecl，得到 {other:?}"),
                        }
                        assert!(matches!(block.stmts[1].node, StmtKind::Expr(_)));
                    }
                    other => panic!("应为块体，得到 {other:?}"),
                }
            }
            other => panic!("应为 FnDecl，得到 {other:?}"),
        }
    }

    #[test]
    fn return_inside_fn_decl_is_legal() {
        // fn f() { return 1 } —— 本批起 fn 体真正 fn_depth +1，源码级 return 不再报错。
        let p = parse(&[
            tok(TokenKind::KwFn, 1, 1),
            tok(TokenKind::Ident("f".into()), 1, 4),
            tok(TokenKind::LParen, 1, 5),
            tok(TokenKind::RParen, 1, 6),
            tok(TokenKind::LBrace, 1, 8),
            tok(TokenKind::KwReturn, 1, 10),
            tok(TokenKind::Int("1".into()), 1, 17),
            tok(TokenKind::RBrace, 1, 18),
            tok(TokenKind::Eof, 1, 19),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::FnDecl(FnDecl { body, .. }) => match body {
                Body::Block(block) => {
                    assert_eq!(block.stmts.len(), 1);
                    match &block.stmts[0].node {
                        StmtKind::Return(Some(e)) => assert_eq!(e.node, ExprKind::Int(1)),
                        other => panic!("应为 Return(Some)，得到 {other:?}"),
                    }
                }
                other => panic!("应为块体，得到 {other:?}"),
            },
            other => panic!("应为 FnDecl，得到 {other:?}"),
        }
    }

    // ---- struct 声明（§7 struct_decl）----

    #[test]
    fn struct_decl_empty() {
        // struct Empty {}
        let p = parse(&[
            tok(TokenKind::KwStruct, 1, 1),
            tok(TokenKind::Ident("Empty".into()), 1, 8),
            tok(TokenKind::LBrace, 1, 14),
            tok(TokenKind::RBrace, 1, 15),
            tok(TokenKind::Eof, 1, 16),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::StructDecl(StructDecl { span, name, members }) => {
                assert_eq!(*span, Span::new(1, 1));
                assert_eq!(name, "Empty");
                assert!(members.is_empty());
            }
            other => panic!("应为 StructDecl，得到 {other:?}"),
        }
    }

    #[test]
    fn struct_decl_fields() {
        // struct Point { x: 0, y: 0 }
        let p = parse(&[
            tok(TokenKind::KwStruct, 1, 1),
            tok(TokenKind::Ident("Point".into()), 1, 8),
            tok(TokenKind::LBrace, 1, 14),
            tok(TokenKind::Ident("x".into()), 1, 16),
            tok(TokenKind::Colon, 1, 17),
            tok(TokenKind::Int("0".into()), 1, 19),
            tok(TokenKind::Comma, 1, 20),
            tok(TokenKind::Ident("y".into()), 1, 22),
            tok(TokenKind::Colon, 1, 23),
            tok(TokenKind::Int("0".into()), 1, 25),
            tok(TokenKind::RBrace, 1, 26),
            tok(TokenKind::Eof, 1, 27),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::StructDecl(StructDecl { name, members, .. }) => {
                assert_eq!(name, "Point");
                assert_eq!(members.len(), 2);
                match &members[0] {
                    StructMember::Field(fi) => {
                        assert_eq!(fi.span, Span::new(1, 16));
                        assert_eq!(fi.name, "x");
                        assert_eq!(fi.value.node, ExprKind::Int(0));
                    }
                    other => panic!("应为字段成员，得到 {other:?}"),
                }
                match &members[1] {
                    StructMember::Field(fi) => {
                        assert_eq!(fi.name, "y");
                        assert_eq!(fi.value.span, Span::new(1, 25));
                    }
                    other => panic!("应为字段成员，得到 {other:?}"),
                }
            }
            other => panic!("应为 StructDecl，得到 {other:?}"),
        }
    }

    #[test]
    fn struct_decl_members_across_newlines_with_trailing_comma() {
        // struct P {
        //     x: 0,
        //     y: 0,
        // }
        // 成员表 `{ … }` 内换行忽略（§3.2）、尾逗号可选（A7）。
        let p = parse(&[
            tok(TokenKind::KwStruct, 1, 1),
            tok(TokenKind::Ident("P".into()), 1, 8),
            tok(TokenKind::LBrace, 1, 10),
            tok(TokenKind::Newline, 1, 11),
            tok(TokenKind::Ident("x".into()), 2, 5),
            tok(TokenKind::Colon, 2, 6),
            tok(TokenKind::Int("0".into()), 2, 8),
            tok(TokenKind::Comma, 2, 9),
            tok(TokenKind::Newline, 2, 10),
            tok(TokenKind::Ident("y".into()), 3, 5),
            tok(TokenKind::Colon, 3, 6),
            tok(TokenKind::Int("0".into()), 3, 8),
            tok(TokenKind::Comma, 3, 9),
            tok(TokenKind::Newline, 3, 10),
            tok(TokenKind::RBrace, 4, 1),
            tok(TokenKind::Eof, 4, 2),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::StructDecl(StructDecl { members, .. }) => {
                assert_eq!(members.len(), 2);
                assert!(matches!(members[0], StructMember::Field(_)));
                assert!(matches!(members[1], StructMember::Field(_)));
            }
            other => panic!("应为 StructDecl，得到 {other:?}"),
        }
    }

    #[test]
    fn struct_decl_with_methods_self_access() {
        // struct Student {
        //     name: "",
        //     fn isTop() => self.score >= 90,
        //     fn bump() { self.count += 1 },
        // }
        let p = parse(&[
            tok(TokenKind::KwStruct, 1, 1),
            tok(TokenKind::Ident("Student".into()), 1, 8),
            tok(TokenKind::LBrace, 1, 16),
            tok(TokenKind::Ident("name".into()), 2, 5),
            tok(TokenKind::Colon, 2, 9),
            tok(TokenKind::StrBegin, 2, 11),
            tok(TokenKind::StrEnd, 2, 13),
            tok(TokenKind::Comma, 2, 14),
            tok(TokenKind::KwFn, 3, 5),
            tok(TokenKind::Ident("isTop".into()), 3, 8),
            tok(TokenKind::LParen, 3, 13),
            tok(TokenKind::RParen, 3, 14),
            tok(TokenKind::Arrow, 3, 16),
            tok(TokenKind::KwSelf, 3, 19),
            tok(TokenKind::Dot, 3, 23),
            tok(TokenKind::Ident("score".into()), 3, 24),
            tok(TokenKind::Ge, 3, 30),
            tok(TokenKind::Int("90".into()), 3, 33),
            tok(TokenKind::Comma, 3, 35),
            tok(TokenKind::KwFn, 4, 5),
            tok(TokenKind::Ident("bump".into()), 4, 8),
            tok(TokenKind::LParen, 4, 12),
            tok(TokenKind::RParen, 4, 13),
            tok(TokenKind::LBrace, 4, 15),
            tok(TokenKind::KwSelf, 4, 17),
            tok(TokenKind::Dot, 4, 21),
            tok(TokenKind::Ident("count".into()), 4, 22),
            tok(TokenKind::PlusAssign, 4, 28),
            tok(TokenKind::Int("1".into()), 4, 31),
            tok(TokenKind::RBrace, 4, 32),
            tok(TokenKind::Comma, 4, 33),
            tok(TokenKind::RBrace, 5, 1),
            tok(TokenKind::Eof, 5, 2),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::StructDecl(StructDecl { members, .. }) => {
                assert_eq!(members.len(), 3);
                match &members[0] {
                    StructMember::Field(fi) => {
                        assert_eq!(fi.name, "name");
                        assert_eq!(fi.value.node, ExprKind::Str(String::new()));
                    }
                    other => panic!("应为字段成员，得到 {other:?}"),
                }
                match &members[1] {
                    StructMember::Method(m) => {
                        assert_eq!(m.span, Span::new(3, 5));
                        assert_eq!(m.name, "isTop");
                        assert!(m.params.is_empty());
                        match &m.body {
                            Body::Expr(e) => match &e.node {
                                ExprKind::Binary { op, left, .. } => {
                                    assert_eq!(*op, BinaryOp::Ge);
                                    assert!(matches!(
                                        left.node,
                                        ExprKind::Field { ref object, ref name } if name == "score" && matches!(object.node, ExprKind::SelfRef)
                                    ));
                                }
                                other => panic!("应为 Ge，得到 {other:?}"),
                            },
                            other => panic!("应为箭头表达式体，得到 {other:?}"),
                        }
                    }
                    other => panic!("应为方法成员，得到 {other:?}"),
                }
                match &members[2] {
                    StructMember::Method(m) => {
                        assert_eq!(m.name, "bump");
                        match &m.body {
                            Body::Block(block) => {
                                assert_eq!(block.stmts.len(), 1);
                                match &block.stmts[0].node {
                                    StmtKind::Assign { target, op, value } => {
                                        assert_eq!(*op, AssignOp::AddAssign);
                                        assert!(matches!(target.base, LvalueBase::SelfValue));
                                        assert_eq!(target.path.len(), 1);
                                        assert!(matches!(
                                            target.path[0].kind,
                                            LvalueSegKind::Field(ref n) if n == "count"
                                        ));
                                        assert_eq!(value.node, ExprKind::Int(1));
                                    }
                                    other => panic!("应为 Assign，得到 {other:?}"),
                                }
                            }
                            other => panic!("应为块体，得到 {other:?}"),
                        }
                    }
                    other => panic!("应为方法成员，得到 {other:?}"),
                }
            }
            other => panic!("应为 StructDecl，得到 {other:?}"),
        }
    }

    // ---- 函数字面量（§7 lambda）----

    #[test]
    fn fn_lambda_as_expression_statement() {
        // fn (x) { x } —— 语句位置的匿名函数字面量（§3.3 块判定）。
        let p = parse(&[
            tok(TokenKind::KwFn, 1, 1),
            tok(TokenKind::LParen, 1, 4),
            tok(TokenKind::Ident("x".into()), 1, 5),
            tok(TokenKind::RParen, 1, 6),
            tok(TokenKind::LBrace, 1, 8),
            tok(TokenKind::Ident("x".into()), 1, 10),
            tok(TokenKind::RBrace, 1, 11),
            tok(TokenKind::Eof, 1, 12),
        ])
        .unwrap();
        assert_eq!(p.stmts[0].span, Span::new(1, 1));
        match &p.stmts[0].node {
            StmtKind::Expr(e) => match &e.node {
                ExprKind::Lambda(l) => {
                    assert_eq!(l.params, vec!["x".to_string()]);
                    match &l.body {
                        Body::Block(block) => {
                            assert_eq!(block.span, Span::new(1, 8));
                            assert_eq!(block.stmts.len(), 1);
                        }
                        other => panic!("应为块体，得到 {other:?}"),
                    }
                }
                other => panic!("应为 Lambda，得到 {other:?}"),
            },
            other => panic!("应为 Expr，得到 {other:?}"),
        }
    }

    #[test]
    fn fn_lambda_in_decl_init() {
        // let f = fn (a, b) => a + b
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("f".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::KwFn, 1, 9),
            tok(TokenKind::LParen, 1, 12),
            tok(TokenKind::Ident("a".into()), 1, 13),
            tok(TokenKind::Comma, 1, 14),
            tok(TokenKind::Ident("b".into()), 1, 16),
            tok(TokenKind::RParen, 1, 17),
            tok(TokenKind::Arrow, 1, 19),
            tok(TokenKind::Ident("a".into()), 1, 22),
            tok(TokenKind::Plus, 1, 24),
            tok(TokenKind::Ident("b".into()), 1, 26),
            tok(TokenKind::Eof, 1, 27),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::Decl { init, .. } => match &init.node {
                ExprKind::Lambda(l) => {
                    assert_eq!(l.params, vec!["a".to_string(), "b".to_string()]);
                    assert!(matches!(l.body, Body::Expr(_)));
                }
                other => panic!("应为 Lambda，得到 {other:?}"),
            },
            other => panic!("应为 Decl，得到 {other:?}"),
        }
    }

    #[test]
    fn arrow_lambda_zero_params() {
        // let f = () => 42
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("f".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::LParen, 1, 9),
            tok(TokenKind::RParen, 1, 10),
            tok(TokenKind::Arrow, 1, 12),
            tok(TokenKind::Int("42".into()), 1, 15),
            tok(TokenKind::Eof, 1, 17),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::Decl { init, .. } => match &init.node {
                ExprKind::Lambda(l) => {
                    assert!(l.params.is_empty());
                    match &l.body {
                        Body::Expr(e) => assert_eq!(e.node, ExprKind::Int(42)),
                        other => panic!("应为箭头表达式体，得到 {other:?}"),
                    }
                }
                other => panic!("应为 Lambda，得到 {other:?}"),
            },
            other => panic!("应为 Decl，得到 {other:?}"),
        }
    }

    #[test]
    fn arrow_lambda_as_call_argument() {
        // f((x) => x * 2) —— 实参位置的箭头 lambda。
        let p = parse(&[
            tok(TokenKind::Ident("f".into()), 1, 1),
            tok(TokenKind::LParen, 1, 2),
            tok(TokenKind::LParen, 1, 3),
            tok(TokenKind::Ident("x".into()), 1, 4),
            tok(TokenKind::RParen, 1, 5),
            tok(TokenKind::Arrow, 1, 7),
            tok(TokenKind::Ident("x".into()), 1, 10),
            tok(TokenKind::Star, 1, 12),
            tok(TokenKind::Int("2".into()), 1, 14),
            tok(TokenKind::RParen, 1, 15),
            tok(TokenKind::Eof, 1, 16),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::Expr(e) => match &e.node {
                ExprKind::Call { callee, args } => {
                    assert!(matches!(callee.node, ExprKind::Ident(ref n) if n == "f"));
                    assert_eq!(args.len(), 1);
                    match &args[0].node {
                        ExprKind::Lambda(l) => {
                            assert_eq!(l.params, vec!["x".to_string()]);
                            assert!(matches!(l.body, Body::Expr(_)));
                        }
                        other => panic!("应为 Lambda，得到 {other:?}"),
                    }
                }
                other => panic!("应为 Call，得到 {other:?}"),
            },
            other => panic!("应为 Expr，得到 {other:?}"),
        }
    }

    #[test]
    fn nested_arrow_lambda_currying() {
        // let f = (x) => (y) => x + y —— 柯里化：体内再嵌 lambda。
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("f".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::LParen, 1, 9),
            tok(TokenKind::Ident("x".into()), 1, 10),
            tok(TokenKind::RParen, 1, 11),
            tok(TokenKind::Arrow, 1, 13),
            tok(TokenKind::LParen, 1, 16),
            tok(TokenKind::Ident("y".into()), 1, 17),
            tok(TokenKind::RParen, 1, 18),
            tok(TokenKind::Arrow, 1, 20),
            tok(TokenKind::Ident("x".into()), 1, 23),
            tok(TokenKind::Plus, 1, 25),
            tok(TokenKind::Ident("y".into()), 1, 27),
            tok(TokenKind::Eof, 1, 28),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::Decl { init, .. } => match &init.node {
                ExprKind::Lambda(outer) => {
                    assert_eq!(outer.params, vec!["x".to_string()]);
                    match &outer.body {
                        Body::Expr(inner_e) => match &inner_e.node {
                            ExprKind::Lambda(inner) => {
                                assert_eq!(inner.params, vec!["y".to_string()]);
                                assert!(matches!(inner.body, Body::Expr(_)));
                            }
                            other => panic!("应为内层 Lambda，得到 {other:?}"),
                        },
                        other => panic!("应为箭头表达式体，得到 {other:?}"),
                    }
                }
                other => panic!("应为 Lambda，得到 {other:?}"),
            },
            other => panic!("应为 Decl，得到 {other:?}"),
        }
    }

    #[test]
    fn paren_group_still_group_not_lambda() {
        // let x = (1 + 2) —— 无 `=>` 的括号仍是分组（A20 前瞻失败回退）。
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("x".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::LParen, 1, 9),
            tok(TokenKind::Int("1".into()), 1, 10),
            tok(TokenKind::Plus, 1, 12),
            tok(TokenKind::Int("2".into()), 1, 14),
            tok(TokenKind::RParen, 1, 15),
            tok(TokenKind::Eof, 1, 16),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::Decl { init, .. } => {
                assert_eq!(init.span, Span::new(1, 9)); // 分组保留外层 span
                assert!(matches!(init.node, ExprKind::Binary { op: BinaryOp::Add, .. }));
            }
            other => panic!("应为 Decl，得到 {other:?}"),
        }
    }

    #[test]
    fn self_reference_expression() {
        // self.score —— 方法体内的字段访问（§7 primary 含 self）。
        let p = parse(&[
            tok(TokenKind::KwSelf, 1, 1),
            tok(TokenKind::Dot, 1, 5),
            tok(TokenKind::Ident("score".into()), 1, 6),
            tok(TokenKind::Eof, 1, 11),
        ])
        .unwrap();
        match &p.stmts[0].node {
            StmtKind::Expr(e) => match &e.node {
                ExprKind::Field { object, name } => {
                    assert!(matches!(object.node, ExprKind::SelfRef));
                    assert_eq!(name, "score");
                }
                other => panic!("应为 Field，得到 {other:?}"),
            },
            other => panic!("应为 Expr，得到 {other:?}"),
        }
    }

    // ---- 回归：声明 / lambda 与既有构造混用 ----

    #[test]
    fn decl_and_lambda_regression_with_existing_constructs() {
        // let xs = [1, 2]
        // let f = (x) => x + 1
        // f(10)
        // struct P { x: 0 }
        // fn g() => 7
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("xs".into()), 1, 5),
            tok(TokenKind::Assign, 1, 8),
            tok(TokenKind::LBracket, 1, 10),
            tok(TokenKind::Int("1".into()), 1, 11),
            tok(TokenKind::Comma, 1, 12),
            tok(TokenKind::Int("2".into()), 1, 14),
            tok(TokenKind::RBracket, 1, 15),
            tok(TokenKind::Newline, 1, 16),
            tok(TokenKind::KwLet, 2, 1),
            tok(TokenKind::Ident("f".into()), 2, 5),
            tok(TokenKind::Assign, 2, 7),
            tok(TokenKind::LParen, 2, 9),
            tok(TokenKind::Ident("x".into()), 2, 10),
            tok(TokenKind::RParen, 2, 11),
            tok(TokenKind::Arrow, 2, 13),
            tok(TokenKind::Ident("x".into()), 2, 16),
            tok(TokenKind::Plus, 2, 18),
            tok(TokenKind::Int("1".into()), 2, 20),
            tok(TokenKind::Newline, 2, 21),
            tok(TokenKind::Ident("f".into()), 3, 1),
            tok(TokenKind::LParen, 3, 2),
            tok(TokenKind::Int("10".into()), 3, 3),
            tok(TokenKind::RParen, 3, 5),
            tok(TokenKind::Newline, 3, 6),
            tok(TokenKind::KwStruct, 4, 1),
            tok(TokenKind::Ident("P".into()), 4, 8),
            tok(TokenKind::LBrace, 4, 10),
            tok(TokenKind::Ident("x".into()), 4, 12),
            tok(TokenKind::Colon, 4, 13),
            tok(TokenKind::Int("0".into()), 4, 15),
            tok(TokenKind::RBrace, 4, 16),
            tok(TokenKind::Newline, 4, 17),
            tok(TokenKind::KwFn, 5, 1),
            tok(TokenKind::Ident("g".into()), 5, 4),
            tok(TokenKind::LParen, 5, 5),
            tok(TokenKind::RParen, 5, 6),
            tok(TokenKind::Arrow, 5, 8),
            tok(TokenKind::Int("7".into()), 5, 11),
            tok(TokenKind::Eof, 5, 12),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 5);
        assert!(matches!(p.stmts[0].node, StmtKind::Decl { .. }));
        match &p.stmts[1].node {
            StmtKind::Decl { init, .. } => assert!(matches!(init.node, ExprKind::Lambda(_))),
            other => panic!("应为 Decl(Lambda)，得到 {other:?}"),
        }
        assert!(matches!(p.stmts[2].node, StmtKind::Expr(_)));
        assert!(matches!(p.stmts[3].node, StmtKind::StructDecl(_)));
        match &p.stmts[4].node {
            StmtKind::FnDecl(FnDecl { name, body, .. }) => {
                assert_eq!(name, "g");
                match body {
                    Body::Expr(e) => assert_eq!(e.node, ExprKind::Int(7)),
                    other => panic!("应为箭头表达式体，得到 {other:?}"),
                }
            }
            other => panic!("应为 FnDecl，得到 {other:?}"),
        }
    }

    // ---- 错误：fn / struct / lambda 的语法违规（含 span）----

    #[test]
    fn fn_decl_missing_rparen_is_error() {
        // fn f(x { 1 } —— 缺 `)`。
        let err = parse(&[
            tok(TokenKind::KwFn, 1, 1),
            tok(TokenKind::Ident("f".into()), 1, 4),
            tok(TokenKind::LParen, 1, 5),
            tok(TokenKind::Ident("x".into()), 1, 6),
            tok(TokenKind::LBrace, 1, 8),
            tok(TokenKind::Int("1".into()), 1, 10),
            tok(TokenKind::RBrace, 1, 11),
            tok(TokenKind::Eof, 1, 12),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "')'".to_string(),
                got: "'{'".to_string(),
            },
            1,
            8,
        );
    }

    #[test]
    fn fn_decl_missing_arrow_or_block_is_error() {
        // fn f(x) 1 —— 体位置既无 `{` 也无 `=>`。
        let err = parse(&[
            tok(TokenKind::KwFn, 1, 1),
            tok(TokenKind::Ident("f".into()), 1, 4),
            tok(TokenKind::LParen, 1, 5),
            tok(TokenKind::Ident("x".into()), 1, 6),
            tok(TokenKind::RParen, 1, 7),
            tok(TokenKind::Int("1".into()), 1, 9),
            tok(TokenKind::Eof, 1, 10),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "'=>'".to_string(),
                got: "'1'".to_string(),
            },
            1,
            9,
        );
    }

    #[test]
    fn fn_decl_unterminated_block_is_error() {
        // fn f() { 1 —— 缺 `}`。
        let err = parse(&[
            tok(TokenKind::KwFn, 1, 1),
            tok(TokenKind::Ident("f".into()), 1, 4),
            tok(TokenKind::LParen, 1, 5),
            tok(TokenKind::RParen, 1, 6),
            tok(TokenKind::LBrace, 1, 8),
            tok(TokenKind::Int("1".into()), 1, 10),
            tok(TokenKind::Eof, 1, 11),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "'}'".to_string(),
                got: "'文件末尾'".to_string(),
            },
            1,
            11,
        );
    }

    #[test]
    fn fn_decl_missing_param_name_is_error() {
        // fn f(, x) { 1 } —— 形参表以逗号开头。
        let err = parse(&[
            tok(TokenKind::KwFn, 1, 1),
            tok(TokenKind::Ident("f".into()), 1, 4),
            tok(TokenKind::LParen, 1, 5),
            tok(TokenKind::Comma, 1, 6),
            tok(TokenKind::Ident("x".into()), 1, 8),
            tok(TokenKind::RParen, 1, 9),
            tok(TokenKind::LBrace, 1, 11),
            tok(TokenKind::Int("1".into()), 1, 13),
            tok(TokenKind::RBrace, 1, 14),
            tok(TokenKind::Eof, 1, 15),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "形参名".to_string(),
                got: "','".to_string(),
            },
            1,
            6,
        );
    }

    #[test]
    fn struct_decl_missing_name_is_error() {
        // struct { x: 1 }
        let err = parse(&[
            tok(TokenKind::KwStruct, 1, 1),
            tok(TokenKind::LBrace, 1, 8),
            tok(TokenKind::Ident("x".into()), 1, 10),
            tok(TokenKind::Colon, 1, 11),
            tok(TokenKind::Int("1".into()), 1, 13),
            tok(TokenKind::RBrace, 1, 14),
            tok(TokenKind::Eof, 1, 15),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "结构体名".to_string(),
                got: "'{'".to_string(),
            },
            1,
            8,
        );
    }

    #[test]
    fn struct_decl_missing_rbrace_is_error() {
        // struct P { x: 1 —— 缺 `}`。
        let err = parse(&[
            tok(TokenKind::KwStruct, 1, 1),
            tok(TokenKind::Ident("P".into()), 1, 8),
            tok(TokenKind::LBrace, 1, 10),
            tok(TokenKind::Ident("x".into()), 1, 12),
            tok(TokenKind::Colon, 1, 13),
            tok(TokenKind::Int("1".into()), 1, 15),
            tok(TokenKind::Eof, 1, 16),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "'}'".to_string(),
                got: "'文件末尾'".to_string(),
            },
            1,
            16,
        );
    }

    #[test]
    fn struct_member_missing_colon_is_error() {
        // struct P { x 1 } —— 字段缺 `:`。
        let err = parse(&[
            tok(TokenKind::KwStruct, 1, 1),
            tok(TokenKind::Ident("P".into()), 1, 8),
            tok(TokenKind::LBrace, 1, 10),
            tok(TokenKind::Ident("x".into()), 1, 12),
            tok(TokenKind::Int("1".into()), 1, 14),
            tok(TokenKind::RBrace, 1, 15),
            tok(TokenKind::Eof, 1, 16),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "':'".to_string(),
                got: "'1'".to_string(),
            },
            1,
            14,
        );
    }

    #[test]
    fn struct_method_missing_body_is_error() {
        // struct P { fn f() } —— 方法缺体（无 `{` 无 `=>`）。
        let err = parse(&[
            tok(TokenKind::KwStruct, 1, 1),
            tok(TokenKind::Ident("P".into()), 1, 8),
            tok(TokenKind::LBrace, 1, 10),
            tok(TokenKind::KwFn, 1, 12),
            tok(TokenKind::Ident("f".into()), 1, 15),
            tok(TokenKind::LParen, 1, 16),
            tok(TokenKind::RParen, 1, 17),
            tok(TokenKind::RBrace, 1, 19),
            tok(TokenKind::Eof, 1, 20),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "'=>'".to_string(),
                got: "'}'".to_string(),
            },
            1,
            19,
        );
    }

    #[test]
    fn arrow_after_group_expr_is_two_statements() {
        // (a + b) => c —— A20 反例：分组后 `=>` 处 TwoStatements。
        let err = parse(&[
            tok(TokenKind::LParen, 1, 1),
            tok(TokenKind::Ident("a".into()), 1, 2),
            tok(TokenKind::Plus, 1, 4),
            tok(TokenKind::Ident("b".into()), 1, 6),
            tok(TokenKind::RParen, 1, 7),
            tok(TokenKind::Arrow, 1, 9),
            tok(TokenKind::Ident("c".into()), 1, 12),
            tok(TokenKind::Eof, 1, 13),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::TwoStatements, 1, 9);
    }

    // ======================================================================
    // 第六批：`;;` → Dump 节点
    // ======================================================================

    #[test]
    fn dump_statement_builds_dump_node_with_sentinel_scope() {
        let p = parse(&[
            tok(TokenKind::Dump, 1, 1),
            tok(TokenKind::Eof, 1, 3),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 1);
        assert_eq!(p.stmts[0].span, Span::new(1, 1));
        match &p.stmts[0].node {
            StmtKind::Dump { scope } => assert_eq!(*scope, ScopeId(0)),
            other => panic!("应为 Dump，得到 {other:?}"),
        }
    }

    #[test]
    fn dump_inside_block_is_legal() {
        let p = parse(&[
            tok(TokenKind::LBrace, 1, 1),
            tok(TokenKind::Dump, 1, 3),
            tok(TokenKind::Newline, 1, 5),
            tok(TokenKind::RBrace, 1, 6),
            tok(TokenKind::Eof, 1, 7),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 1);
        assert!(matches!(p.stmts[0].node, StmtKind::Dump { .. }));
    }

    #[test]
    fn dump_must_occupy_own_line() {
        // `x = 1 ;;` —— A10 反例：`;;` 不独占逻辑行 → 同逻辑行两条语句。
        let err = parse(&[
            tok(TokenKind::Ident("x".into()), 1, 1),
            tok(TokenKind::Assign, 1, 3),
            tok(TokenKind::Int("1".into()), 1, 5),
            tok(TokenKind::Dump, 1, 7),
            tok(TokenKind::Eof, 1, 9),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::TwoStatements, 1, 7);
    }

    // ======================================================================
    // 第六批：管道 `|>` 脱糖（§4.3）
    // ======================================================================

    #[test]
    fn pipe_without_placeholder_appends_as_last_arg() {
        // `xs |> f(a, b)` → Call { f, [a, b, xs] }（data-last）。
        let p = parse(&[
            tok(TokenKind::Ident("xs".into()), 1, 1),
            tok(TokenKind::Pipe, 1, 4),
            tok(TokenKind::Ident("f".into()), 1, 7),
            tok(TokenKind::LParen, 1, 8),
            tok(TokenKind::Ident("a".into()), 1, 9),
            tok(TokenKind::Comma, 1, 10),
            tok(TokenKind::Ident("b".into()), 1, 12),
            tok(TokenKind::RParen, 1, 13),
            tok(TokenKind::Eof, 1, 14),
        ])
        .unwrap();
        let e = expr_of(&p);
        assert_eq!(e.span, Span::new(1, 1));
        match &e.node {
            ExprKind::Call { callee, args } => {
                assert!(matches!(&callee.node, ExprKind::Ident(n) if n == "f"));
                assert_eq!(args.len(), 3);
                assert!(matches!(&args[0].node, ExprKind::Ident(n) if n == "a"));
                assert!(matches!(&args[1].node, ExprKind::Ident(n) if n == "b"));
                assert!(matches!(&args[2].node, ExprKind::Ident(n) if n == "xs"));
                assert_eq!(args[2].span, Span::new(1, 1));
            }
            other => panic!("应为脱糖后的 Call，得到 {other:?}"),
        }
    }

    #[test]
    fn pipe_with_placeholder_injects_at_position() {
        // `xs |> f(a, _, b)` → Call { f, [a, xs, b] }。
        let p = parse(&[
            tok(TokenKind::Ident("xs".into()), 1, 1),
            tok(TokenKind::Pipe, 1, 4),
            tok(TokenKind::Ident("f".into()), 1, 7),
            tok(TokenKind::LParen, 1, 8),
            tok(TokenKind::Ident("a".into()), 1, 9),
            tok(TokenKind::Comma, 1, 10),
            tok(TokenKind::Placeholder, 1, 12),
            tok(TokenKind::Comma, 1, 13),
            tok(TokenKind::Ident("b".into()), 1, 15),
            tok(TokenKind::RParen, 1, 16),
            tok(TokenKind::Eof, 1, 17),
        ])
        .unwrap();
        let e = expr_of(&p);
        match &e.node {
            ExprKind::Call { callee, args } => {
                assert!(matches!(&callee.node, ExprKind::Ident(n) if n == "f"));
                assert_eq!(args.len(), 3);
                assert!(matches!(&args[0].node, ExprKind::Ident(n) if n == "a"));
                assert!(matches!(&args[1].node, ExprKind::Ident(n) if n == "xs"));
                assert!(matches!(&args[2].node, ExprKind::Ident(n) if n == "b"));
            }
            other => panic!("应为脱糖后的 Call，得到 {other:?}"),
        }
    }

    #[test]
    fn pipe_placeholder_in_nested_call_args() {
        // `xs |> f(g(_))` → Call { f, [Call { g, [xs] }] }（规则 5：任意嵌套深度）。
        let p = parse(&[
            tok(TokenKind::Ident("xs".into()), 1, 1),
            tok(TokenKind::Pipe, 1, 4),
            tok(TokenKind::Ident("f".into()), 1, 7),
            tok(TokenKind::LParen, 1, 8),
            tok(TokenKind::Ident("g".into()), 1, 9),
            tok(TokenKind::LParen, 1, 10),
            tok(TokenKind::Placeholder, 1, 11),
            tok(TokenKind::RParen, 1, 12),
            tok(TokenKind::RParen, 1, 13),
            tok(TokenKind::Eof, 1, 14),
        ])
        .unwrap();
        let e = expr_of(&p);
        match &e.node {
            ExprKind::Call { callee, args } => {
                assert!(matches!(&callee.node, ExprKind::Ident(n) if n == "f"));
                assert_eq!(args.len(), 1);
                match &args[0].node {
                    ExprKind::Call { callee, args } => {
                        assert!(matches!(&callee.node, ExprKind::Ident(n) if n == "g"));
                        assert!(matches!(&args[0].node, ExprKind::Ident(n) if n == "xs"));
                    }
                    other => panic!("应为嵌套 Call，得到 {other:?}"),
                }
            }
            other => panic!("应为脱糖后的 Call，得到 {other:?}"),
        }
    }

    #[test]
    fn pipe_chaining_is_left_associative() {
        // `xs |> f() |> g()` → Call { g, [Call { f, [xs] }] }。
        let p = parse(&[
            tok(TokenKind::Ident("xs".into()), 1, 1),
            tok(TokenKind::Pipe, 1, 4),
            tok(TokenKind::Ident("f".into()), 1, 7),
            tok(TokenKind::LParen, 1, 8),
            tok(TokenKind::RParen, 1, 9),
            tok(TokenKind::Pipe, 1, 11),
            tok(TokenKind::Ident("g".into()), 1, 14),
            tok(TokenKind::LParen, 1, 15),
            tok(TokenKind::RParen, 1, 16),
            tok(TokenKind::Eof, 1, 17),
        ])
        .unwrap();
        let e = expr_of(&p);
        assert_eq!(e.span, Span::new(1, 1));
        match &e.node {
            ExprKind::Call { callee, args } => {
                assert!(matches!(&callee.node, ExprKind::Ident(n) if n == "g"));
                match &args[0].node {
                    ExprKind::Call { callee, args } => {
                        assert!(matches!(&callee.node, ExprKind::Ident(n) if n == "f"));
                        assert!(matches!(&args[0].node, ExprKind::Ident(n) if n == "xs"));
                    }
                    other => panic!("应为内层 Call，得到 {other:?}"),
                }
            }
            other => panic!("应为脱糖后的 Call，得到 {other:?}"),
        }
    }

    #[test]
    fn pipe_rhs_bare_ident_wraps_call() {
        // `xs |> sum` → Call { sum, [xs] }（规则 2）。
        let p = parse(&[
            tok(TokenKind::Ident("xs".into()), 1, 1),
            tok(TokenKind::Pipe, 1, 4),
            tok(TokenKind::Ident("sum".into()), 1, 7),
            tok(TokenKind::Eof, 1, 10),
        ])
        .unwrap();
        let e = expr_of(&p);
        match &e.node {
            ExprKind::Call { callee, args } => {
                assert!(matches!(&callee.node, ExprKind::Ident(n) if n == "sum"));
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0].node, ExprKind::Ident(n) if n == "xs"));
            }
            other => panic!("应为脱糖后的 Call，得到 {other:?}"),
        }
    }

    #[test]
    fn pipe_rhs_lambda_wraps_call() {
        // `xs |> (x) => x` → Call { Lambda, [xs] }（规则 2）。
        let p = parse(&[
            tok(TokenKind::Ident("xs".into()), 1, 1),
            tok(TokenKind::Pipe, 1, 4),
            tok(TokenKind::LParen, 1, 7),
            tok(TokenKind::Ident("x".into()), 1, 8),
            tok(TokenKind::RParen, 1, 9),
            tok(TokenKind::Arrow, 1, 11),
            tok(TokenKind::Ident("x".into()), 1, 14),
            tok(TokenKind::Eof, 1, 15),
        ])
        .unwrap();
        let e = expr_of(&p);
        match &e.node {
            ExprKind::Call { callee, args } => {
                assert!(matches!(callee.node, ExprKind::Lambda(_)));
                assert!(matches!(&args[0].node, ExprKind::Ident(n) if n == "xs"));
            }
            other => panic!("应为脱糖后的 Call，得到 {other:?}"),
        }
    }

    #[test]
    fn pipe_precedence_add_binds_tighter() {
        // `1 + 2 |> f` → Call { f, [Binary(+, 1, 2)] }（§4.4：`|>` 比 `+` 松）。
        let p = parse(&[
            tok(TokenKind::Int("1".into()), 1, 1),
            tok(TokenKind::Plus, 1, 3),
            tok(TokenKind::Int("2".into()), 1, 5),
            tok(TokenKind::Pipe, 1, 7),
            tok(TokenKind::Ident("f".into()), 1, 10),
            tok(TokenKind::Eof, 1, 11),
        ])
        .unwrap();
        let e = expr_of(&p);
        match &e.node {
            ExprKind::Call { callee, args } => {
                assert!(matches!(&callee.node, ExprKind::Ident(n) if n == "f"));
                assert!(matches!(
                    &args[0].node,
                    ExprKind::Binary { op: BinaryOp::Add, .. }
                ));
            }
            other => panic!("应为脱糖后的 Call，得到 {other:?}"),
        }
    }

    #[test]
    fn pipe_rhs_must_be_postfix_or_lambda() {
        // `xs |> sum() + 1` → A2：`|>` 右侧仅收 pipe_rhs，`+` 处同逻辑行第二条语句。
        let err = parse(&[
            tok(TokenKind::Ident("xs".into()), 1, 1),
            tok(TokenKind::Pipe, 1, 4),
            tok(TokenKind::Ident("sum".into()), 1, 7),
            tok(TokenKind::LParen, 1, 10),
            tok(TokenKind::RParen, 1, 11),
            tok(TokenKind::Plus, 1, 13),
            tok(TokenKind::Int("1".into()), 1, 15),
            tok(TokenKind::Eof, 1, 16),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::TwoStatements, 1, 13);
    }

    #[test]
    fn pipe_multiple_placeholder_is_error() {
        // `xs |> f(_, _)` → 第二个 `_` 处 PipeMultiplePlaceholder。
        let err = parse(&[
            tok(TokenKind::Ident("xs".into()), 1, 1),
            tok(TokenKind::Pipe, 1, 4),
            tok(TokenKind::Ident("f".into()), 1, 7),
            tok(TokenKind::LParen, 1, 8),
            tok(TokenKind::Placeholder, 1, 9),
            tok(TokenKind::Comma, 1, 10),
            tok(TokenKind::Placeholder, 1, 12),
            tok(TokenKind::RParen, 1, 13),
            tok(TokenKind::Eof, 1, 14),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::PipeMultiplePlaceholder, 1, 12);
    }

    #[test]
    fn placeholder_as_whole_rhs_is_error() {
        // `xs |> _` → `_` 不在调用实参位 → PlaceholderPosition。
        let err = parse(&[
            tok(TokenKind::Ident("xs".into()), 1, 1),
            tok(TokenKind::Pipe, 1, 4),
            tok(TokenKind::Placeholder, 1, 7),
            tok(TokenKind::Eof, 1, 8),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::PlaceholderPosition, 1, 7);
    }

    #[test]
    fn placeholder_outside_pipe_is_error() {
        // 裸 `_` 表达式语句 → PlaceholderPosition（规则 4）。
        let err = parse(&[
            tok(TokenKind::Placeholder, 1, 1),
            tok(TokenKind::Eof, 1, 2),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::PlaceholderPosition, 1, 1);
    }

    #[test]
    fn placeholder_in_let_binding_is_error() {
        // `let _ = 1` → PlaceholderPosition（规则 4：`let _ = …` 非法）。
        let err = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Placeholder, 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::Int("1".into()), 1, 9),
            tok(TokenKind::Eof, 1, 10),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::PlaceholderPosition, 1, 5);
    }

    #[test]
    fn placeholder_in_lambda_body_is_error() {
        // `xs |> map((x) => x + _)` → 规则 6（M4）：`_` 不得穿 λ 体。
        let err = parse(&[
            tok(TokenKind::Ident("xs".into()), 1, 1),
            tok(TokenKind::Pipe, 1, 4),
            tok(TokenKind::Ident("map".into()), 1, 7),
            tok(TokenKind::LParen, 1, 10),
            tok(TokenKind::LParen, 1, 11),
            tok(TokenKind::Ident("x".into()), 1, 12),
            tok(TokenKind::RParen, 1, 13),
            tok(TokenKind::Arrow, 1, 15),
            tok(TokenKind::Ident("x".into()), 1, 18),
            tok(TokenKind::Plus, 1, 20),
            tok(TokenKind::Placeholder, 1, 22),
            tok(TokenKind::RParen, 1, 23),
            tok(TokenKind::Eof, 1, 24),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::PlaceholderPosition, 1, 22);
    }

    #[test]
    fn nested_pipe_inside_lambda_body_binds_to_inner_pipe() {
        // `xs |> map((x) => x |> f(_))` → 内层 `_` 绑定内层 `|>`（规则 6）。
        let p = parse(&[
            tok(TokenKind::Ident("xs".into()), 1, 1),
            tok(TokenKind::Pipe, 1, 4),
            tok(TokenKind::Ident("map".into()), 1, 7),
            tok(TokenKind::LParen, 1, 10),
            tok(TokenKind::LParen, 1, 11),
            tok(TokenKind::Ident("x".into()), 1, 12),
            tok(TokenKind::RParen, 1, 13),
            tok(TokenKind::Arrow, 1, 15),
            tok(TokenKind::Ident("x".into()), 1, 18),
            tok(TokenKind::Pipe, 1, 20),
            tok(TokenKind::Ident("f".into()), 1, 23),
            tok(TokenKind::LParen, 1, 24),
            tok(TokenKind::Placeholder, 1, 25),
            tok(TokenKind::RParen, 1, 26),
            tok(TokenKind::RParen, 1, 27),
            tok(TokenKind::Eof, 1, 28),
        ])
        .unwrap();
        let e = expr_of(&p);
        match &e.node {
            ExprKind::Call { callee, args } => {
                assert!(matches!(&callee.node, ExprKind::Ident(n) if n == "map"));
                // 无顶层 `_` → `xs` 追加为末参；lambda 体内 `x |> f(_)` → `f(x)`。
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0].node, ExprKind::Lambda(_)));
                assert!(matches!(&args[1].node, ExprKind::Ident(n) if n == "xs"));
                match &args[0].node {
                    ExprKind::Lambda(l) => match &l.body {
                        Body::Expr(inner) => match &inner.node {
                            ExprKind::Call { callee, args } => {
                                assert!(matches!(&callee.node, ExprKind::Ident(n) if n == "f"));
                                assert!(matches!(&args[0].node, ExprKind::Ident(n) if n == "x"));
                            }
                            other => panic!("应为内层脱糖 Call，得到 {other:?}"),
                        },
                        _ => panic!("lambda 体应为表达式"),
                    },
                    other => panic!("应为 Lambda，得到 {other:?}"),
                }
            }
            other => panic!("应为脱糖后的 Call，得到 {other:?}"),
        }
    }

    // ======================================================================
    // 第六批：富字符串插值（§2.8）
    // ======================================================================

    #[test]
    fn interp_text_only_is_plain_str() {
        // `"hi"` 无插值 → 仍为纯 `Str`（回归）。
        let p = parse(&[
            tok(TokenKind::StrBegin, 1, 1),
            tok(TokenKind::Text("hi".into()), 1, 2),
            tok(TokenKind::StrEnd, 1, 4),
            tok(TokenKind::Eof, 1, 5),
        ])
        .unwrap();
        let e = expr_of(&p);
        assert_eq!(e.node, ExprKind::Str("hi".to_string()));
    }

    #[test]
    fn interp_single_expr_without_format_spec() {
        // `"${x}"` → Interp { parts: [Expr { Ident(x), format_spec: None }] }。
        let p = parse(&[
            tok(TokenKind::StrBegin, 1, 1),
            tok(TokenKind::InterpBegin, 1, 2),
            tok(TokenKind::Ident("x".into()), 1, 4),
            tok(TokenKind::InterpEnd, 1, 5),
            tok(TokenKind::StrEnd, 1, 6),
            tok(TokenKind::Eof, 1, 7),
        ])
        .unwrap();
        let e = expr_of(&p);
        assert_eq!(e.span, Span::new(1, 1));
        match &e.node {
            ExprKind::Interp(InterpString { parts }) => {
                assert_eq!(parts.len(), 1);
                match &parts[0] {
                    StrPart::Expr { expr, format_spec } => {
                        assert!(matches!(&expr.node, ExprKind::Ident(n) if n == "x"));
                        assert_eq!(expr.span, Span::new(1, 4));
                        assert_eq!(*format_spec, None);
                    }
                    other => panic!("应为表达式段，得到 {other:?}"),
                }
            }
            other => panic!("应为 Interp，得到 {other:?}"),
        }
    }

    #[test]
    fn interp_multi_segment_text_expr_alternation() {
        // `"a${x}b${y}c"` → [Text(a), Expr(x), Text(b), Expr(y), Text(c)]。
        let p = parse(&[
            tok(TokenKind::StrBegin, 1, 1),
            tok(TokenKind::Text("a".into()), 1, 2),
            tok(TokenKind::InterpBegin, 1, 3),
            tok(TokenKind::Ident("x".into()), 1, 5),
            tok(TokenKind::InterpEnd, 1, 6),
            tok(TokenKind::Text("b".into()), 1, 7),
            tok(TokenKind::InterpBegin, 1, 8),
            tok(TokenKind::Ident("y".into()), 1, 10),
            tok(TokenKind::InterpEnd, 1, 11),
            tok(TokenKind::Text("c".into()), 1, 12),
            tok(TokenKind::StrEnd, 1, 13),
            tok(TokenKind::Eof, 1, 14),
        ])
        .unwrap();
        let e = expr_of(&p);
        match &e.node {
            ExprKind::Interp(InterpString { parts }) => {
                assert_eq!(parts.len(), 5);
                assert!(matches!(&parts[0], StrPart::Text(t) if t == "a"));
                assert!(matches!(&parts[1], StrPart::Expr { format_spec: None, .. }));
                assert!(matches!(&parts[2], StrPart::Text(t) if t == "b"));
                assert!(matches!(&parts[3], StrPart::Expr { format_spec: None, .. }));
                assert!(matches!(&parts[4], StrPart::Text(t) if t == "c"));
            }
            other => panic!("应为 Interp，得到 {other:?}"),
        }
    }

    #[test]
    fn interp_with_format_spec_some() {
        // `"${x:>3}"` → format_spec = Some(">3")（M6）。
        let p = parse(&[
            tok(TokenKind::StrBegin, 1, 1),
            tok(TokenKind::InterpBegin, 1, 2),
            tok(TokenKind::Ident("x".into()), 1, 4),
            tok(TokenKind::FormatSpec(">3".into()), 1, 5),
            tok(TokenKind::InterpEnd, 1, 8),
            tok(TokenKind::StrEnd, 1, 9),
            tok(TokenKind::Eof, 1, 10),
        ])
        .unwrap();
        let e = expr_of(&p);
        match &e.node {
            ExprKind::Interp(InterpString { parts }) => match &parts[0] {
                StrPart::Expr { format_spec, .. } => {
                    assert_eq!(format_spec.as_deref(), Some(">3"));
                }
                other => panic!("应为表达式段，得到 {other:?}"),
            },
            other => panic!("应为 Interp，得到 {other:?}"),
        }
    }

    #[test]
    fn interp_empty_format_spec_is_some_empty() {
        // `"${x:}"` → format_spec = Some("")（M6：`:` 后可空）。
        let p = parse(&[
            tok(TokenKind::StrBegin, 1, 1),
            tok(TokenKind::InterpBegin, 1, 2),
            tok(TokenKind::Ident("x".into()), 1, 4),
            tok(TokenKind::FormatSpec(String::new()), 1, 5),
            tok(TokenKind::InterpEnd, 1, 6),
            tok(TokenKind::StrEnd, 1, 7),
            tok(TokenKind::Eof, 1, 8),
        ])
        .unwrap();
        let e = expr_of(&p);
        match &e.node {
            ExprKind::Interp(InterpString { parts }) => match &parts[0] {
                StrPart::Expr { format_spec, .. } => assert_eq!(format_spec.as_deref(), Some("")),
                other => panic!("应为表达式段，得到 {other:?}"),
            },
            other => panic!("应为 Interp，得到 {other:?}"),
        }
    }

    #[test]
    fn interp_nested_string_inside_expr() {
        // `"${"inner"}"` → Expr { Str("inner"), None }（嵌套字符串，§2.8 规则 3）。
        let p = parse(&[
            tok(TokenKind::StrBegin, 1, 1),
            tok(TokenKind::InterpBegin, 1, 2),
            tok(TokenKind::StrBegin, 1, 4),
            tok(TokenKind::Text("inner".into()), 1, 5),
            tok(TokenKind::StrEnd, 1, 10),
            tok(TokenKind::InterpEnd, 1, 11),
            tok(TokenKind::StrEnd, 1, 12),
            tok(TokenKind::Eof, 1, 13),
        ])
        .unwrap();
        let e = expr_of(&p);
        match &e.node {
            ExprKind::Interp(InterpString { parts }) => match &parts[0] {
                StrPart::Expr { expr, format_spec } => {
                    assert_eq!(expr.node, ExprKind::Str("inner".to_string()));
                    assert_eq!(*format_spec, None);
                }
                other => panic!("应为表达式段，得到 {other:?}"),
            },
            other => panic!("应为 Interp，得到 {other:?}"),
        }
    }

    #[test]
    fn interp_expr_may_contain_operator() {
        // `"${a + 1}"` → Expr 段为 Binary(+, a, 1)。
        let p = parse(&[
            tok(TokenKind::StrBegin, 1, 1),
            tok(TokenKind::InterpBegin, 1, 2),
            tok(TokenKind::Ident("a".into()), 1, 4),
            tok(TokenKind::Plus, 1, 6),
            tok(TokenKind::Int("1".into()), 1, 8),
            tok(TokenKind::InterpEnd, 1, 9),
            tok(TokenKind::StrEnd, 1, 10),
            tok(TokenKind::Eof, 1, 11),
        ])
        .unwrap();
        let e = expr_of(&p);
        match &e.node {
            ExprKind::Interp(InterpString { parts }) => match &parts[0] {
                StrPart::Expr { expr, .. } => {
                    assert!(matches!(
                        &expr.node,
                        ExprKind::Binary { op: BinaryOp::Add, .. }
                    ));
                }
                other => panic!("应为表达式段，得到 {other:?}"),
            },
            other => panic!("应为 Interp，得到 {other:?}"),
        }
    }

    #[test]
    fn interp_in_assign_and_decl_init() {
        // `let s = "n=${n}"` 声明初值含插值 → Interp 段表达式为 Ident("n")。
        let p = parse(&[
            tok(TokenKind::KwLet, 1, 1),
            tok(TokenKind::Ident("s".into()), 1, 5),
            tok(TokenKind::Assign, 1, 7),
            tok(TokenKind::StrBegin, 1, 9),
            tok(TokenKind::Text("n=".into()), 1, 10),
            tok(TokenKind::InterpBegin, 1, 12),
            tok(TokenKind::Ident("n".into()), 1, 14),
            tok(TokenKind::InterpEnd, 1, 15),
            tok(TokenKind::StrEnd, 1, 16),
            tok(TokenKind::Eof, 1, 17),
        ])
        .unwrap();
        let e = decl_init(&p);
        match &e.node {
            ExprKind::Interp(InterpString { parts }) => {
                assert_eq!(parts.len(), 2);
                assert!(matches!(&parts[0], StrPart::Text(t) if t == "n="));
            }
            other => panic!("应为 Interp，得到 {other:?}"),
        }
    }

    // ======================================================================
    // 第六批：i64::MIN 字面量 / IntegerOutOfRange（§10.8 B12）
    // ======================================================================

    #[test]
    fn int_literal_i64_min_with_unary_minus_is_legal() {
        // `-9223372036854775808` → `Int(i64::MIN)`，**不产生** Unary 节点。
        let p = parse(&[
            tok(TokenKind::Minus, 1, 1),
            tok(TokenKind::Int("9223372036854775808".into()), 1, 2),
            tok(TokenKind::Eof, 1, 22),
        ])
        .unwrap();
        let e = expr_of(&p);
        assert_eq!(e.node, ExprKind::Int(i64::MIN));
        assert_eq!(e.span, Span::new(1, 1));
    }

    #[test]
    fn int_literal_i64_min_hex_with_unary_minus_is_legal() {
        // `-0x8000000000000000`（正值为 2^63）→ `Int(i64::MIN)`。
        let p = parse(&[
            tok(TokenKind::Minus, 1, 1),
            tok(TokenKind::Int("0x8000000000000000".into()), 1, 2),
            tok(TokenKind::Eof, 1, 20),
        ])
        .unwrap();
        let e = expr_of(&p);
        assert_eq!(e.node, ExprKind::Int(i64::MIN));
    }

    #[test]
    fn int_literal_i64_max_is_legal() {
        let p = parse(&[
            tok(TokenKind::Int("9223372036854775807".into()), 1, 1),
            tok(TokenKind::Eof, 1, 21),
        ])
        .unwrap();
        let e = expr_of(&p);
        assert_eq!(e.node, ExprKind::Int(i64::MAX));
    }

    #[test]
    fn int_literal_positive_overflow_is_error() {
        // `9223372036854775808`（无负号）→ IntegerOutOfRange。
        let err = parse(&[
            tok(TokenKind::Int("9223372036854775808".into()), 1, 1),
            tok(TokenKind::Eof, 1, 21),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::IntegerOutOfRange, 1, 1);
    }

    #[test]
    fn int_literal_negative_overflow_is_error() {
        // `-9223372036854775809`（正值为 2^63 + 1，非 2^63 特判）→ IntegerOutOfRange。
        let err = parse(&[
            tok(TokenKind::Minus, 1, 1),
            tok(TokenKind::Int("9223372036854775809".into()), 1, 2),
            tok(TokenKind::Eof, 1, 22),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::IntegerOutOfRange, 1, 2);
    }

    #[test]
    fn int_literal_i64_min_behind_paren_is_error() {
        // `-(9223372036854775808)` → `-` 不紧邻 INT → 括号内超界 → IntegerOutOfRange。
        let err = parse(&[
            tok(TokenKind::Minus, 1, 1),
            tok(TokenKind::LParen, 1, 2),
            tok(TokenKind::Int("9223372036854775808".into()), 1, 3),
            tok(TokenKind::RParen, 1, 22),
            tok(TokenKind::Eof, 1, 23),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::IntegerOutOfRange, 1, 3);
    }

    #[test]
    fn int_radix_literal_parses_as_u64() {
        // `0xFFFFFFFFFFFFFFFF`（u64::MAX，radix 形式按 u64 解析后按位重解释）→ Int(-1)。
        let p = parse(&[
            tok(TokenKind::Int("0xFFFFFFFFFFFFFFFF".into()), 1, 1),
            tok(TokenKind::Eof, 1, 19),
        ])
        .unwrap();
        let e = expr_of(&p);
        assert_eq!(e.node, ExprKind::Int(-1));
    }

    #[test]
    fn int_radix_literal_out_of_u64_is_error() {
        // `0x1FFFFFFFFFFFFFFFF`（2^64）→ radix 形式超出 u64 → IntegerOutOfRange。
        let err = parse(&[
            tok(TokenKind::Int("0x1FFFFFFFFFFFFFFFF".into()), 1, 1),
            tok(TokenKind::Eof, 1, 20),
        ])
        .unwrap_err();
        assert_syntax(&err, SyntaxMsg::IntegerOutOfRange, 1, 1);
    }

    #[test]
    fn int_plain_neg_small_still_unary() {
        // 回归：`-5` 仍为 Unary(Neg, Int(5))（2^63 特判不误伤普通负号）。
        let p = parse(&[
            tok(TokenKind::Minus, 1, 1),
            tok(TokenKind::Int("5".into()), 1, 2),
            tok(TokenKind::Eof, 1, 3),
        ])
        .unwrap();
        let e = expr_of(&p);
        assert!(matches!(
            &e.node,
            ExprKind::Unary { op: UnaryOp::Neg, operand }
                if matches!(operand.node, ExprKind::Int(5))
        ));
    }

    #[test]
    fn int_pipe_and_dump_regression_with_existing_constructs() {
        // 综合回归：管道脱糖 + 边界整数 + `;;` 共存。
        let p = parse(&[
            tok(TokenKind::Ident("xs".into()), 1, 1),
            tok(TokenKind::Pipe, 1, 4),
            tok(TokenKind::Ident("sum".into()), 1, 7),
            tok(TokenKind::Newline, 1, 10),
            tok(TokenKind::Minus, 2, 1),
            tok(TokenKind::Int("9223372036854775808".into()), 2, 2),
            tok(TokenKind::Newline, 2, 22),
            tok(TokenKind::Dump, 3, 1),
            tok(TokenKind::Eof, 3, 3),
        ])
        .unwrap();
        assert_eq!(p.stmts.len(), 3);
        assert!(matches!(p.stmts[0].node, StmtKind::Expr(_)));
        match &p.stmts[1].node {
            StmtKind::Expr(e) => assert_eq!(e.node, ExprKind::Int(i64::MIN)),
            other => panic!("应为 Int(i64::MIN) 表达式语句，得到 {other:?}"),
        }
        assert!(matches!(p.stmts[2].node, StmtKind::Dump { .. }));
    }
}
