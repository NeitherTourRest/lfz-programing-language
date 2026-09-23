//! 语法分析器（递归下降）—— 第一批：最小可用子集。
//!
//! 契约：`docs/spec/interface-contract.md` §10.6；语法：`docs/spec/syntax.md` §3.1 / §3.2。
//!
//! # 本批范围（其余构造在后续批次实现，当前一律报 `UnexpectedToken`）
//!
//! - 换行模式栈（§3.2）：`(` / `[` 内 `NEWLINE` 忽略（`IGN`），语句层生效（`SIG`）。
//! - 语句：`let` / `var` 声明、表达式语句、块 `{ ... }`、程序（`Program`）本身。
//! - 表达式：字面量（Int / Float / 纯字符串 / true / false / nil）、标识符、
//!   括号分组 `( expr )`、一元 `-` / `!`。
//! - 错误：`UnexpectedToken` / `IncompleteExpr` / `LoneSemicolon`，
//!   语句终结检查（§3.2 同逻辑行两条语句）报 `TwoStatements`。
//!
//! # 本批两处已知简化
//!
//! 1. 裸块语句（语句起始处的 `{ ... }`）：AST 的 `StmtKind` 无 Block 变体，
//!    故将块内语句**内联**进外层语句序列（`{ let x = 1 }` ≡ `let x = 1`）。
//! 2. `;;`→`Dump`、管道、插值、赋值、二元运算等均为后续批次；遇之即 `UnexpectedToken`。

use crate::ast::*;
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

/// 递归下降解析器：`tokens` + 游标 + 换行模式栈。
pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    /// 换行模式栈（§3.2），栈顶 = 当前模式；初始 `[Sig]`。
    nl_stack: Vec<NlMode>,
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

    /// 解析一条语句（`let` / `var` 声明、表达式语句；`;` → `LoneSemicolon`）。
    fn parse_stmt(&mut self) -> R<Stmt> {
        let span = self.peek_span();
        match self.peek().clone() {
            TokenKind::KwLet | TokenKind::KwVar => self.parse_decl(span),
            TokenKind::Semi => Err(syntax(SyntaxMsg::LoneSemicolon, span)),
            _ => {
                let expr = self.parse_expr()?;
                Ok(Spanned::new(StmtKind::Expr(expr), span))
            }
        }
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
            _ => return Err(self.unexpected("变量名")),
        };
        self.expect(&TokenKind::Assign)?;
        let init = self.parse_expr()?;
        Ok(Spanned::new(StmtKind::Decl { mutable, name, init }, span))
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
    // 表达式（一元 + 基本表达式）
    // ------------------------------------------------------------------

    /// `expr = unary`（本批无二元运算，直接落到一元层）。
    fn parse_expr(&mut self) -> R<Expr> {
        self.parse_unary()
    }

    /// 一元 `-` / `!`（§4.1 级别 2）；否则降级到基本表达式。
    fn parse_unary(&mut self) -> R<Expr> {
        self.skip_ign_newlines();
        let span = self.peek_span();
        let op = match self.peek() {
            TokenKind::Minus => Some(UnaryOp::Neg),
            TokenKind::Bang => Some(UnaryOp::Not),
            _ => None,
        };
        match op {
            Some(op) => {
                self.bump();
                let operand = self.parse_unary()?;
                Ok(Spanned::new(ExprKind::Unary { op, operand: Box::new(operand) }, span))
            }
            None => self.parse_primary(),
        }
    }

    /// 基本表达式：字面量 / 标识符 / 括号分组。
    fn parse_primary(&mut self) -> R<Expr> {
        self.skip_ign_newlines();
        let span = self.peek_span();
        match self.peek().clone() {
            TokenKind::Int(text) => {
                self.bump();
                let n = parse_int(&text).ok_or_else(|| {
                    syntax(
                        SyntaxMsg::UnexpectedToken {
                            expected: "i64 范围内的整数".to_string(),
                            got: format!("'{text}'"),
                        },
                        span,
                    )
                })?;
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
            TokenKind::Ident(name) => {
                self.bump();
                Ok(Spanned::new(ExprKind::Ident(name), span))
            }
            TokenKind::LParen => self.parse_group(span),
            TokenKind::Newline => Err(syntax(SyntaxMsg::IncompleteExpr, span)),
            _ => Err(self.unexpected("表达式")),
        }
    }

    /// 纯字符串字面量：`StrBegin , { Text } , StrEnd`（本批不支持插值）。
    fn parse_string(&mut self, span: Span) -> R<Expr> {
        self.bump(); // StrBegin
        let mut text = String::new();
        loop {
            match self.peek().clone() {
                TokenKind::Text(part) => {
                    text.push_str(&part);
                    self.bump();
                }
                TokenKind::StrEnd => {
                    self.bump();
                    break;
                }
                TokenKind::InterpBegin => {
                    return Err(syntax(
                        SyntaxMsg::UnexpectedToken {
                            expected: "\"".to_string(),
                            got: "${".to_string(),
                        },
                        self.peek_span(),
                    ));
                }
                _ => return Err(self.unexpected("\"")),
            }
        }
        Ok(Spanned::new(ExprKind::Str(text), span))
    }

    /// 括号分组 `( expr )`：`(` 内压入 `IGN` 模式（换行当空白）。
    fn parse_group(&mut self, span: Span) -> R<Expr> {
        self.bump(); // LParen
        self.nl_stack.push(NlMode::Ign);
        let inner = self.parse_expr()?;
        self.skip_ign_newlines();
        self.expect(&TokenKind::RParen)?;
        self.nl_stack.pop();
        Ok(Spanned::new(inner.node, span))
    }
}

// ===========================================================================
// 辅助函数
// ===========================================================================

/// 解析整数字面量原文（`_` 分隔与 `0x` / `0b` / `0o` 进制，§2.7）。
///
/// 失败返回 `None`（B12 的 `i64::MIN` 特判属后续批次）。
fn parse_int(text: &str) -> Option<i64> {
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
    i64::from_str_radix(digits, radix).ok()
}

/// 解析浮点字面量原文（允许 `_` 分隔，§2.7）。
fn parse_float(text: &str) -> Option<f64> {
    text.replace('_', "").parse::<f64>().ok()
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
    fn unexpected_keyword_if() {
        let err = parse(&[
            tok(TokenKind::KwIf, 1, 1),
            tok(TokenKind::Eof, 1, 3),
        ])
        .unwrap_err();
        assert_syntax(
            &err,
            SyntaxMsg::UnexpectedToken {
                expected: "表达式".to_string(),
                got: "'if'".to_string(),
            },
            1,
            1,
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
}
