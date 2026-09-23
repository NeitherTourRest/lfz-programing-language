//! 语法分析器（递归下降）—— 第二批：后缀表达式 + 数组 / struct 字面量。
//!
//! 契约：`docs/spec/interface-contract.md` §10.6；语法：`docs/spec/syntax.md` §3.1 / §3.2 / §4.1 / §7。
//!
//! # 已实现（本批 + 前批；其余构造在后续批次实现，当前一律报 `UnexpectedToken`）
//!
//! - 换行模式栈（§3.2）：`(` / `[` / 字面量成员表 `{` 内 `NEWLINE` 忽略（`IGN`），
//!   语句层生效（`SIG`）。
//! - 语句：`let` / `var` 声明、表达式语句、块 `{ ... }`、程序（`Program`）本身。
//! - 表达式：字面量（Int / Float / 纯字符串 / true / false / nil）、标识符、
//!   括号分组 `( expr )`、一元 `-` / `!`、
//!   后缀链（调用 `f(a, …)` / 索引 `xs[i]` / 字段 `s.k`，可任意链式，§4.1 级别 1）、
//!   数组字面量 `[a, b, c]`、struct 字面量 `{ k: v }` 与 `Name { k: v }`（§7）。
//! - 错误：`UnexpectedToken` / `IncompleteExpr` / `LoneSemicolon`，
//!   语句终结检查（§3.2 同逻辑行两条语句）报 `TwoStatements`。
//!
//! # 本批已知简化
//!
//! 1. 语句起始处的 `{ ... }` 仍按**裸块语句**解析并内联进外层语句序列
//!    （AST 的 `StmtKind` 无 Block 变体）；§3.3 / A9「语句首 `{` 恒为匿名
//!    struct 字面量」留待语句形态补齐批次一并处理。表达式位置的 `{`
//!   一律为 struct 字面量（NO_BRACE_LITERAL：块起始位不受影响）。
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

    /// 一元 `-` / `!`（§4.1 级别 2）；否则降级到后缀表达式。
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
            None => self.parse_postfix(),
        }
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
                    let args = self.parse_args(TokenKind::RParen)?;
                    self.expect(&TokenKind::RParen)?;
                    self.nl_stack.pop();
                    expr = Spanned::new(ExprKind::Call { callee: Box::new(expr), args }, span);
                }
                // 索引 `object [ expr ]`。
                TokenKind::LBracket => {
                    self.bump();
                    self.nl_stack.push(NlMode::Ign);
                    let inner = self.parse_expr()?;
                    self.skip_ign_newlines();
                    self.expect(&TokenKind::RBracket)?;
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
                // `IDENT {`：具名 struct 字面量（§3.3 规则 2、§7 `struct_lit`）。
                // 仅当 `{` 紧随其后（`IGN` 内换行忽略，`SIG` 下换行即拆句，A3）。
                self.skip_ign_newlines();
                if *self.peek() == TokenKind::LBrace {
                    self.parse_struct_lit(Some(name), span)
                } else {
                    Ok(Spanned::new(ExprKind::Ident(name), span))
                }
            }
            TokenKind::LBracket => self.parse_array(span),
            TokenKind::LBrace => self.parse_struct_lit(None, span),
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
        let elems = self.parse_args(TokenKind::RBracket)?;
        self.expect(&TokenKind::RBracket)?;
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
                    ExprKind::Str(s) => s,
                    _ => unreachable!("纯字符串解析必得 Str"),
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
}
