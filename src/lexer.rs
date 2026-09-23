//! 词法分析器（lexer）：LFZ 源码 → `Token` 序列。
//!
//! 契约依据：`docs/spec/syntax.md` §2（词法基础：2.3 最大匹配 / 2.4 注释 /
//! 2.5 空白与换行 / 2.6 标识符与关键字 / 2.7 数值字面量 / 2.8 字符串与插值），
//! 以及 `docs/spec/interface-contract.md` §10.2（Span）/ §10.6（模块分工）。
//!
//! **本批范围（P3.3a）**：CODE 模式下的核心记号 —— 空白 / 注释 / 换行 /
//! 标识符 / 关键字 / 保留字 / 数值字面量 / 运算符与分隔符。
//! **本批不含**：字符串与插值（`"` / `${` / `FORMAT_SPEC`，模式栈 CODE/STR/INTERP）
//! —— 留待 P3.3b；本批遇 `"` 暂时按 `IllegalChar('"')` 兜底。
//!
//! 位置语义（§10.2）：`line` = **本地行号 + `line_base`**；`col` 为 1-based、
//! 按 **Unicode 标量**计数（非字节偏移）。

use crate::error::{syntax, SyntaxMsg, R};
use crate::span::Span;

/// 记号种类。一次定义到位，含下一批（P3.3b）才产生的字符串/插值变体。
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TokenKind {
    // ---- 字面量 ----
    /// 整数字面量；保存**原文**（B12），不做范围/进制归一。
    Int(String),
    /// 浮点字面量；保存**原文**。
    Float(String),
    /// 标识符（仅 ASCII）。
    Ident(String),

    // ---- 字符串 / 插值（P3.3b 产出；本批仅定义，不产生）----
    /// 字符串开始 `"`。
    StrBegin,
    /// 字符串结束 `"`。
    StrEnd,
    /// 字符串普通文本片段。
    Text(String),
    /// 插值开始 `${`。
    InterpBegin,
    /// 插值结束 `}`（花括号深度归零处）。
    InterpEnd,
    /// 格式说明符原始文本（`${expr:spec}` 中 `spec` 部分）。
    FormatSpec(String),

    // ---- 关键字（16，`syntax.md` §2.6）----
    /// `let`
    KwLet,
    /// `var`
    KwVar,
    /// `fn`
    KwFn,
    /// `return`
    KwReturn,
    /// `if`
    KwIf,
    /// `else`
    KwElse,
    /// `while`
    KwWhile,
    /// `for`
    KwFor,
    /// `in`
    KwIn,
    /// `break`
    KwBreak,
    /// `continue`
    KwContinue,
    /// `struct`
    KwStruct,
    /// `true`
    KwTrue,
    /// `false`
    KwFalse,
    /// `nil`
    KwNil,
    /// `self`
    KwSelf,

    /// 未来保留字（当前使用即 `SyntaxError`，`syntax.md` §2.6）。
    Reserved(&'static str),

    // ---- 记号（`syntax.md` §2.3 最大匹配表）----
    /// `;;`（dump 语句，占一个逻辑行，A10–A12）。
    Dump,
    /// `;`（单独出现非法，A11；由 parser 判定）。
    Semi,
    /// `=>`
    Arrow,
    /// `==`
    EqEq,
    /// `!=`
    NotEq,
    /// `<=`
    Le,
    /// `>=`
    Ge,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `&&`
    AndAnd,
    /// `||`
    OrOr,
    /// `|>`（管道）。
    Pipe,
    /// `+=`
    PlusAssign,
    /// `-=`
    MinusAssign,
    /// `*=`
    StarAssign,
    /// `/=`
    SlashAssign,
    /// `%=`
    PercentAssign,
    /// `=`
    Assign,
    /// `!`
    Bang,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// `.`
    Dot,
    /// `_`（管道占位符；单独一个 `_`，§2.6 / A24）。
    Placeholder,
    /// 换行 `\n`（独立记号，§2.5）。
    Newline,
    /// 输入结束。
    Eof,
}

/// 一个记号：种类 + 位置。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token {
    /// 记号种类。
    pub kind: TokenKind,
    /// 记号起始位置（行 + 列，§10.2）。
    pub span: Span,
}

/// 词法分析入口。
///
/// `line_base` 为加载器给定的行号基线（§10.1）：记号绝对行号 = 本地行号 + `line_base`。
/// 返回的记号流**以 `Eof` 结尾**。失败返回带位置的 `SyntaxError`。
pub fn lex(text: &str, line_base: u32) -> R<Vec<Token>> {
    Lexer::new(text, line_base).run()
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    /// 本地行号（1-based）；绝对行号 = 本地行号 + `line_base`。
    line: u32,
    /// 列号（1-based，按 Unicode 标量计数）。
    col: u32,
    line_base: u32,
    out: Vec<Token>,
}

impl Lexer {
    fn new(text: &str, line_base: u32) -> Self {
        Self {
            chars: text.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            line_base,
            out: Vec::new(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, k: usize) -> Option<char> {
        self.chars.get(self.pos + k).copied()
    }

    fn peek_matches<F: Fn(char) -> bool>(&self, k: usize, f: F) -> bool {
        self.peek_at(k).map(f).unwrap_or(false)
    }

    /// 当前位置的绝对 `Span`。
    fn span(&self) -> Span {
        Span::new(self.line + self.line_base, self.col)
    }

    /// 取当前字符并前进一个 Unicode 标量，维护行列号。
    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn consume_while<F: Fn(char) -> bool>(&mut self, f: F) {
        while let Some(c) = self.peek() {
            if f(c) {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn slice(&self, start: usize) -> String {
        self.chars[start..self.pos].iter().collect()
    }

    fn push(&mut self, kind: TokenKind, span: Span) {
        self.out.push(Token { kind, span });
    }

    fn run(mut self) -> R<Vec<Token>> {
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\t' | '\r' => {
                    self.bump();
                }
                '\n' => {
                    let sp = self.span();
                    self.bump();
                    self.push(TokenKind::Newline, sp);
                }
                '/' if self.peek_at(1) == Some('/') => self.line_comment(),
                '/' if self.peek_at(1) == Some('*') => self.block_comment(),
                c if c.is_ascii_digit() => self.number(),
                c if c.is_ascii_alphabetic() || c == '_' => self.ident_or_keyword(),
                _ => self.punct_or_error()?,
            }
        }
        let sp = self.span();
        self.push(TokenKind::Eof, sp);
        Ok(self.out)
    }

    /// `// … 到行尾`（不含行尾换行）。
    fn line_comment(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.bump();
        }
    }

    /// `/* … */`，不可嵌套；等价于一个空格，不产生 `Newline`（§2.4）。
    fn block_comment(&mut self) {
        self.bump(); // '/'
        self.bump(); // '*'
        while let Some(c) = self.peek() {
            if c == '*' && self.peek_at(1) == Some('/') {
                self.bump();
                self.bump();
                return;
            }
            self.bump();
        }
        // EOF 处仍未闭合：`SyntaxMsg` 无对应变体（契约缺口，见汇报）。
        // 本批按「消费至 EOF、等价一个空白」处理，不发明新错误。
    }

    /// 数值字面量（§2.7），原文保留。
    fn number(&mut self) {
        let sp = self.span();
        let start = self.pos;

        // 进制前缀：`0x` / `0b` / `0o`（须后随至少一个该进制字符，否则回退为十进制）。
        if self.peek() == Some('0') {
            if let Some(pred) = match self.peek_at(1) {
                Some('x') => Some(is_hex as fn(char) -> bool),
                Some('b') => Some(is_bin as fn(char) -> bool),
                Some('o') => Some(is_oct as fn(char) -> bool),
                _ => None,
            } {
                if self.peek_matches(2, pred) {
                    self.bump(); // '0'
                    self.bump(); // 进制字母
                    self.consume_while(pred);
                    let s = self.slice(start);
                    self.push(TokenKind::Int(s), sp);
                    return;
                }
            }
        }

        // 十进制整数部分。
        self.consume_while(|c| c.is_ascii_digit() || c == '_');
        let mut is_float = false;

        // 小数部分：`.` 后**必须**有数字（否则 `.` 是字段访问，A23）。
        if self.peek() == Some('.') && self.peek_matches(1, |c| c.is_ascii_digit()) {
            is_float = true;
            self.bump(); // '.'
            self.consume_while(|c| c.is_ascii_digit() || c == '_');
        }

        // 指数部分：`[eE] [+-]? digit`；不完整指数（如 `1e`）回退（`e` 归标识符）。
        if matches!(self.peek(), Some('e') | Some('E')) {
            let signed = matches!(self.peek_at(1), Some('+') | Some('-'));
            let digit_at = if signed { 2 } else { 1 };
            if self.peek_matches(digit_at, |c| c.is_ascii_digit()) {
                is_float = true;
                self.bump(); // 'e' / 'E'
                if signed {
                    self.bump(); // '+' / '-'
                }
                self.consume_while(|c| c.is_ascii_digit() || c == '_');
            }
        }

        let s = self.slice(start);
        let kind = if is_float {
            TokenKind::Float(s)
        } else {
            TokenKind::Int(s)
        };
        self.push(kind, sp);
    }

    /// 标识符（仅 ASCII）/ 关键字 / 保留字 / 单独 `_` 占位符（§2.6）。
    fn ident_or_keyword(&mut self) {
        let sp = self.span();
        let start = self.pos;
        self.consume_while(|c| c.is_ascii_alphanumeric() || c == '_');
        let s = self.slice(start);
        let kind = match s.as_str() {
            "_" => TokenKind::Placeholder,
            "let" => TokenKind::KwLet,
            "var" => TokenKind::KwVar,
            "fn" => TokenKind::KwFn,
            "return" => TokenKind::KwReturn,
            "if" => TokenKind::KwIf,
            "else" => TokenKind::KwElse,
            "while" => TokenKind::KwWhile,
            "for" => TokenKind::KwFor,
            "in" => TokenKind::KwIn,
            "break" => TokenKind::KwBreak,
            "continue" => TokenKind::KwContinue,
            "struct" => TokenKind::KwStruct,
            "true" => TokenKind::KwTrue,
            "false" => TokenKind::KwFalse,
            "nil" => TokenKind::KwNil,
            "self" => TokenKind::KwSelf,
            "match" => TokenKind::Reserved("match"),
            "class" => TokenKind::Reserved("class"),
            "import" => TokenKind::Reserved("import"),
            "from" => TokenKind::Reserved("from"),
            "try" => TokenKind::Reserved("try"),
            "catch" => TokenKind::Reserved("catch"),
            "rescue" => TokenKind::Reserved("rescue"),
            "yield" => TokenKind::Reserved("yield"),
            "async" => TokenKind::Reserved("async"),
            "await" => TokenKind::Reserved("await"),
            "and" => TokenKind::Reserved("and"),
            "or" => TokenKind::Reserved("or"),
            "not" => TokenKind::Reserved("not"),
            "is" => TokenKind::Reserved("is"),
            _ => TokenKind::Ident(s),
        };
        self.push(kind, sp);
    }

    /// 运算符 / 分隔符（最长匹配，§2.3）；未识别字符报错。
    fn punct_or_error(&mut self) -> R<()> {
        let sp = self.span();
        let c = self.peek().unwrap_or('\0');
        let c1 = self.peek_at(1);

        // 先尝试两字符记号（最长匹配）。
        let two = match (c, c1) {
            (';', Some(';')) => Some(TokenKind::Dump),
            ('=', Some('>')) => Some(TokenKind::Arrow),
            ('=', Some('=')) => Some(TokenKind::EqEq),
            ('!', Some('=')) => Some(TokenKind::NotEq),
            ('<', Some('=')) => Some(TokenKind::Le),
            ('>', Some('=')) => Some(TokenKind::Ge),
            ('&', Some('&')) => Some(TokenKind::AndAnd),
            ('|', Some('|')) => Some(TokenKind::OrOr),
            ('|', Some('>')) => Some(TokenKind::Pipe),
            ('+', Some('=')) => Some(TokenKind::PlusAssign),
            ('-', Some('=')) => Some(TokenKind::MinusAssign),
            ('*', Some('=')) => Some(TokenKind::StarAssign),
            ('/', Some('=')) => Some(TokenKind::SlashAssign),
            ('%', Some('=')) => Some(TokenKind::PercentAssign),
            _ => None,
        };
        if let Some(kind) = two {
            self.bump();
            self.bump();
            self.push(kind, sp);
            return Ok(());
        }

        // 单字符记号。
        let one = match c {
            '=' => Some(TokenKind::Assign),
            '<' => Some(TokenKind::Lt),
            '>' => Some(TokenKind::Gt),
            '!' => Some(TokenKind::Bang),
            '+' => Some(TokenKind::Plus),
            '-' => Some(TokenKind::Minus),
            '*' => Some(TokenKind::Star),
            '/' => Some(TokenKind::Slash),
            '%' => Some(TokenKind::Percent),
            '(' => Some(TokenKind::LParen),
            ')' => Some(TokenKind::RParen),
            '[' => Some(TokenKind::LBracket),
            ']' => Some(TokenKind::RBracket),
            '{' => Some(TokenKind::LBrace),
            '}' => Some(TokenKind::RBrace),
            ',' => Some(TokenKind::Comma),
            ':' => Some(TokenKind::Colon),
            '.' => Some(TokenKind::Dot),
            ';' => Some(TokenKind::Semi),
            _ => None,
        };
        if let Some(kind) = one {
            self.bump();
            self.push(kind, sp);
            return Ok(());
        }

        // 错误：`#` 在 CODE 模式非法（§2.2 / B6）；其余为非法字符（§2.3 禁止合成）。
        if c == '#' {
            return Err(syntax(SyntaxMsg::HashPosition, sp));
        }
        Err(syntax(SyntaxMsg::IllegalChar { c }, sp))
    }
}

fn is_hex(c: char) -> bool {
    c.is_ascii_hexdigit() || c == '_'
}

fn is_bin(c: char) -> bool {
    c == '0' || c == '1' || c == '_'
}

fn is_oct(c: char) -> bool {
    matches!(c, '0'..='7') || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::LzError;

    /// 全量记号（含结尾 `Eof`）。
    fn kinds(src: &str) -> Vec<TokenKind> {
        lex(src, 0).unwrap().into_iter().map(|t| t.kind).collect()
    }

    /// 全量记号，去掉结尾 `Eof`。
    fn kinds_no_eof(src: &str) -> Vec<TokenKind> {
        let mut k = kinds(src);
        assert_eq!(k.pop(), Some(TokenKind::Eof), "记号流必须以 Eof 结尾");
        k
    }

    /// 期望词法失败，返回 (消息, 位置)。
    fn err_of(src: &str) -> (SyntaxMsg, Span) {
        let e = lex(src, 0).unwrap_err();
        match *e {
            LzError::Syntax { msg, span } => (msg, span),
            other => panic!("期望 SyntaxError，得到 {other:?}"),
        }
    }

    // ---- 标识符 / 关键字 / 保留字（§2.6）----

    #[test]
    fn sixteen_keywords() {
        let cases = [
            ("let", TokenKind::KwLet),
            ("var", TokenKind::KwVar),
            ("fn", TokenKind::KwFn),
            ("return", TokenKind::KwReturn),
            ("if", TokenKind::KwIf),
            ("else", TokenKind::KwElse),
            ("while", TokenKind::KwWhile),
            ("for", TokenKind::KwFor),
            ("in", TokenKind::KwIn),
            ("break", TokenKind::KwBreak),
            ("continue", TokenKind::KwContinue),
            ("struct", TokenKind::KwStruct),
            ("true", TokenKind::KwTrue),
            ("false", TokenKind::KwFalse),
            ("nil", TokenKind::KwNil),
            ("self", TokenKind::KwSelf),
        ];
        for (src, want) in cases {
            assert_eq!(kinds_no_eof(src), vec![want], "src={src}");
        }
    }

    #[test]
    fn reserved_words() {
        for w in [
            "match", "class", "import", "from", "try", "catch", "rescue", "yield", "async",
            "await", "and", "or", "not", "is",
        ] {
            assert_eq!(kinds_no_eof(w), vec![TokenKind::Reserved(w)], "w={w}");
        }
    }

    #[test]
    fn identifiers_only_ascii() {
        assert_eq!(
            kinds_no_eof("foo _x x_ a1"),
            vec![
                TokenKind::Ident("foo".into()),
                TokenKind::Ident("_x".into()),
                TokenKind::Ident("x_".into()),
                TokenKind::Ident("a1".into()),
            ]
        );
    }

    #[test]
    fn lone_underscore_is_placeholder() {
        assert_eq!(kinds_no_eof("_"), vec![TokenKind::Placeholder]);
        assert_eq!(
            kinds_no_eof("f(_, 1)"),
            vec![
                TokenKind::Ident("f".into()),
                TokenKind::LParen,
                TokenKind::Placeholder,
                TokenKind::Comma,
                TokenKind::Int("1".into()),
                TokenKind::RParen,
            ]
        );
    }

    #[test]
    fn non_ascii_letter_is_illegal() {
        assert_eq!(err_of("中").0, SyntaxMsg::IllegalChar { c: '中' });
        assert_eq!(err_of("let 变量 = 1").0, SyntaxMsg::IllegalChar { c: '变' });
    }

    // ---- 运算符 / 分隔符（§2.3）----

    #[test]
    fn single_char_operators_and_delimiters() {
        let cases = [
            ("=", TokenKind::Assign),
            ("<", TokenKind::Lt),
            (">", TokenKind::Gt),
            ("!", TokenKind::Bang),
            ("+", TokenKind::Plus),
            ("-", TokenKind::Minus),
            ("*", TokenKind::Star),
            ("/", TokenKind::Slash),
            ("%", TokenKind::Percent),
            ("(", TokenKind::LParen),
            (")", TokenKind::RParen),
            ("[", TokenKind::LBracket),
            ("]", TokenKind::RBracket),
            ("{", TokenKind::LBrace),
            ("}", TokenKind::RBrace),
            (",", TokenKind::Comma),
            (":", TokenKind::Colon),
            (".", TokenKind::Dot),
            (";", TokenKind::Semi),
        ];
        for (src, want) in cases {
            assert_eq!(kinds_no_eof(src), vec![want], "src={src}");
        }
    }

    #[test]
    fn multi_char_operators() {
        let cases = [
            (";;", TokenKind::Dump),
            ("=>", TokenKind::Arrow),
            ("==", TokenKind::EqEq),
            ("!=", TokenKind::NotEq),
            ("<=", TokenKind::Le),
            (">=", TokenKind::Ge),
            ("&&", TokenKind::AndAnd),
            ("||", TokenKind::OrOr),
            ("|>", TokenKind::Pipe),
            ("+=", TokenKind::PlusAssign),
            ("-=", TokenKind::MinusAssign),
            ("*=", TokenKind::StarAssign),
            ("/=", TokenKind::SlashAssign),
            ("%=", TokenKind::PercentAssign),
        ];
        for (src, want) in cases {
            assert_eq!(kinds_no_eof(src), vec![want], "src={src}");
        }
    }

    #[test]
    fn maximal_munch_pipe_vs_or() {
        assert_eq!(kinds_no_eof("|>"), vec![TokenKind::Pipe]);
        assert_eq!(err_of("|").0, SyntaxMsg::IllegalChar { c: '|' });
        assert_eq!(kinds_no_eof("||"), vec![TokenKind::OrOr]);
    }

    #[test]
    fn maximal_munch_ampersand() {
        assert_eq!(kinds_no_eof("&&"), vec![TokenKind::AndAnd]);
        assert_eq!(err_of("&").0, SyntaxMsg::IllegalChar { c: '&' });
    }

    #[test]
    fn maximal_munch_shift_and_dotdot_split() {
        // 禁止合成：`>>` 拆成两个 `>`；`..` 拆成两个 `.`
        assert_eq!(kinds_no_eof(">>"), vec![TokenKind::Gt, TokenKind::Gt]);
        assert_eq!(kinds_no_eof(".."), vec![TokenKind::Dot, TokenKind::Dot]);
    }

    #[test]
    fn maximal_munch_assign_family() {
        assert_eq!(kinds_no_eof("=>"), vec![TokenKind::Arrow]);
        assert_eq!(kinds_no_eof("="), vec![TokenKind::Assign]);
        assert_eq!(kinds_no_eof("=="), vec![TokenKind::EqEq]);
        assert_eq!(kinds_no_eof("+="), vec![TokenKind::PlusAssign]);
        assert_eq!(kinds_no_eof("+"), vec![TokenKind::Plus]);
        assert_eq!(kinds_no_eof("!="), vec![TokenKind::NotEq]);
        assert_eq!(kinds_no_eof("!"), vec![TokenKind::Bang]);
        assert_eq!(kinds_no_eof("<="), vec![TokenKind::Le]);
        assert_eq!(kinds_no_eof("<"), vec![TokenKind::Lt]);
        assert_eq!(kinds_no_eof(">="), vec![TokenKind::Ge]);
        assert_eq!(kinds_no_eof(">"), vec![TokenKind::Gt]);
    }

    #[test]
    fn maximal_munch_semi_vs_dump() {
        assert_eq!(kinds_no_eof(";;"), vec![TokenKind::Dump]);
        assert_eq!(kinds_no_eof(";"), vec![TokenKind::Semi]);
        // `;;;` = DUMP + SEMI（A12）
        assert_eq!(kinds_no_eof(";;;"), vec![TokenKind::Dump, TokenKind::Semi]);
    }

    // ---- 注释（§2.4）----

    #[test]
    fn line_comment_ends_before_newline() {
        assert_eq!(
            kinds("// hi\nx"),
            vec![
                TokenKind::Newline,
                TokenKind::Ident("x".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn line_comment_at_eof_emits_no_token() {
        assert_eq!(kinds_no_eof("x // tail"), vec![TokenKind::Ident("x".into())]);
    }

    #[test]
    fn block_comment_acts_as_space_without_newline() {
        // 跨行块注释不产生 Newline（§2.4）
        assert_eq!(
            kinds_no_eof("/* a\nb */x"),
            vec![TokenKind::Ident("x".into())]
        );
        assert_eq!(
            kinds_no_eof("a/* */b"),
            vec![TokenKind::Ident("a".into()), TokenKind::Ident("b".into())]
        );
    }

    #[test]
    fn hash_inside_comment_is_plain() {
        assert_eq!(kinds_no_eof("// #42 注释"), vec![]);
        assert_eq!(kinds_no_eof("/* #42 # */"), vec![]);
    }

    // ---- 换行 / 空白 ----

    #[test]
    fn spaces_tabs_cr_are_skipped() {
        assert_eq!(
            kinds_no_eof("a \t\r b"),
            vec![TokenKind::Ident("a".into()), TokenKind::Ident("b".into())]
        );
    }

    #[test]
    fn newline_is_a_token() {
        assert_eq!(
            kinds_no_eof("a\nb"),
            vec![
                TokenKind::Ident("a".into()),
                TokenKind::Newline,
                TokenKind::Ident("b".into()),
            ]
        );
    }

    // ---- 数值字面量（§2.7）----

    #[test]
    fn integer_literals_preserve_source() {
        let cases = [
            ("1_000", "1_000"),
            ("0xFF", "0xFF"),
            ("0b1010", "0b1010"),
            ("0o755", "0o755"),
            ("42", "42"),
        ];
        for (src, want) in cases {
            assert_eq!(
                kinds_no_eof(src),
                vec![TokenKind::Int(want.into())],
                "src={src}"
            );
        }
    }

    #[test]
    fn float_literals_preserve_source() {
        let cases = [
            ("3.14", "3.14"),
            ("2.5e-3", "2.5e-3"),
            ("1e10", "1e10"),
            ("1E10", "1E10"),
        ];
        for (src, want) in cases {
            assert_eq!(
                kinds_no_eof(src),
                vec![TokenKind::Float(want.into())],
                "src={src}"
            );
        }
    }

    #[test]
    fn incomplete_exponent_falls_back() {
        // `1e` → Int("1") + Ident("e")
        assert_eq!(
            kinds_no_eof("1e"),
            vec![TokenKind::Int("1".into()), TokenKind::Ident("e".into())]
        );
        assert_eq!(
            kinds_no_eof("1e+"),
            vec![
                TokenKind::Int("1".into()),
                TokenKind::Ident("e".into()),
                TokenKind::Plus,
            ]
        );
    }

    #[test]
    fn trailing_dot_is_int_then_dot() {
        // `1.` → INT(1) + `.`（A23）
        assert_eq!(
            kinds_no_eof("1."),
            vec![TokenKind::Int("1".into()), TokenKind::Dot]
        );
        // `x.5`：`.` 是字段访问，`5` 是整数
        assert_eq!(
            kinds_no_eof("x.5"),
            vec![
                TokenKind::Ident("x".into()),
                TokenKind::Dot,
                TokenKind::Int("5".into()),
            ]
        );
    }

    #[test]
    fn radix_without_digits_falls_back() {
        assert_eq!(
            kinds_no_eof("0x"),
            vec![TokenKind::Int("0".into()), TokenKind::Ident("x".into())]
        );
    }

    #[test]
    fn huge_integer_text_is_preserved_no_range_check() {
        // 原文保留、不做 i64 范围检查（B12）：`-9223372036854775808` 词法不报错
        let t = lex("-9223372036854775808", 0).unwrap();
        assert_eq!(t[0].kind, TokenKind::Minus);
        assert_eq!(t[1].kind, TokenKind::Int("9223372036854775808".into()));
    }

    // ---- `#` 与非法字符 ----

    #[test]
    fn hash_anywhere_in_code_is_error() {
        for src in ["#", "#42", "x # y", "let x = 1 # 2", "f(#)"] {
            assert_eq!(err_of(src).0, SyntaxMsg::HashPosition, "src={src}");
        }
    }

    #[test]
    fn illegal_chars() {
        assert_eq!(err_of("@").0, SyntaxMsg::IllegalChar { c: '@' });
        assert_eq!(err_of("?").0, SyntaxMsg::IllegalChar { c: '?' });
        assert_eq!(err_of("a ? b").0, SyntaxMsg::IllegalChar { c: '?' });
        // 本批字符串未实现：`"` 暂按非法字符兜底（P3.3b 替换）
        assert_eq!(err_of("\"").0, SyntaxMsg::IllegalChar { c: '"' });
    }

    #[test]
    fn error_carries_position_of_offending_char() {
        assert_eq!(err_of("x @").1, Span::new(1, 3));
        assert_eq!(err_of("ab\ncd @").1, Span::new(2, 4));
    }

    // ---- line_base（§10.1 / §10.2）----

    #[test]
    fn line_base_offsets_absolute_line() {
        assert_eq!(lex("x", 0).unwrap()[0].span.line, 1);
        assert_eq!(lex("x", 1).unwrap()[0].span.line, 2);

        let t = lex("x\ny", 1).unwrap();
        assert_eq!(t[0].span.line, 2); // x
        assert_eq!(t[1].span.line, 2); // Newline
        assert_eq!(t[2].span.line, 3); // y
    }

    // ---- 列号按 Unicode 标量计数（§10.2）----

    #[test]
    fn col_counts_unicode_scalars_not_bytes() {
        // 块注释内含 1 个多字节标量 😀（4 字节）：其后 `@` 应在第 6 列
        // （若按字节偏移计数则为第 9 列，从而区分两种口径）
        assert_eq!(err_of("/*😀*/@").1, Span::new(1, 6));
        // 需求给的正例：注释内多字节字符不影响后续位置
        assert_eq!(err_of("//中文😀\n@").1, Span::new(2, 1));
    }

    #[test]
    fn multibyte_inside_comment_does_not_shift_following_marks() {
        // 注释 `//中文😀` 共 5 个标量 → 其后 `\n` 位于第 6 列
        let t = lex("//中文😀\nx", 0).unwrap();
        assert_eq!(t[0].kind, TokenKind::Newline);
        assert_eq!(t[0].span, Span::new(1, 6));
        assert_eq!(t[1].kind, TokenKind::Ident("x".into()));
        assert_eq!(t[1].span, Span::new(2, 1));
    }

    // ---- 记号流收尾 ----

    #[test]
    fn empty_input_yields_only_eof() {
        assert_eq!(
            lex("", 0).unwrap(),
            vec![Token {
                kind: TokenKind::Eof,
                span: Span::new(1, 1)
            }]
        );
    }
}
