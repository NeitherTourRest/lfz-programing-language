//! 词法分析器（lexer）：LFZ 源码 → `Token` 序列。
//!
//! 契约依据：`docs/spec/syntax.md` §2（词法基础：2.3 最大匹配 / 2.4 注释 /
//! 2.5 空白与换行 / 2.6 标识符与关键字 / 2.7 数值字面量 / 2.8 字符串与插值），
//! 以及 `docs/spec/interface-contract.md` §10.2（Span）/ §10.6（模块分工）。
//!
//! **本批范围**：
//! - P3.3a：CODE 模式下的核心记号 —— 空白 / 注释 / 换行 / 标识符 / 关键字 /
//!   保留字 / 数值字面量 / 运算符与分隔符。
//! - P3.3b：字符串与插值（`syntax.md` §2.8）—— 模式栈 `CODE` / `STR` / `INTERP`
//!   （外加格式说明符原始模式），产出 `StrBegin` / `Text` / `InterpBegin` /
//!   `FormatSpec` / `InterpEnd` / `StrEnd`。
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

    // ---- 字符串 / 插值（P3.3b 产出；`syntax.md` §2.8）----
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

/// 模式栈条目的词法模式（`syntax.md` §2.8）。
///
/// 栈为空 = **CODE**（顶层）；`Str` = 字符串文本；`Interp(d)` = 插值表达式
/// （`d` 为花括号深度，进入时置 1）；`FormatSpec` = `:` 之后的格式说明符原始模式。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// 字符串文本模式（STR）。
    Str,
    /// 插值表达式模式（INTERP），携带花括号深度。
    Interp(u32),
    /// 格式说明符原始模式（自 `:` 后到匹配 `}`，不解析记号）。
    FormatSpec,
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    /// 本地行号（1-based）；绝对行号 = 本地行号 + `line_base`。
    line: u32,
    /// 列号（1-based，按 Unicode 标量计数）。
    col: u32,
    line_base: u32,
    /// 模式栈；空 = CODE（顶层）。
    modes: Vec<Mode>,
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
            modes: Vec::new(),
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
        loop {
            if self.peek().is_none() {
                break;
            }
            match self.modes.last() {
                None => self.code_token()?,
                Some(Mode::Str) => self.str_text(false)?,
                Some(Mode::Interp(_)) => self.code_token()?,
                Some(Mode::FormatSpec) => self.str_text(true)?,
            }
        }
        // EOF 时仍停留在字符串 / 插值 / 格式说明符内 → 字符串未闭合（§2.8）。
        if !self.modes.is_empty() {
            let sp = self.span();
            return Err(syntax(SyntaxMsg::UnterminatedString, sp));
        }
        let sp = self.span();
        self.push(TokenKind::Eof, sp);
        Ok(self.out)
    }

    /// 当前是否处于插值模式（栈顶为 `INTERP`）。
    fn interp_depth(&self) -> Option<u32> {
        match self.modes.last() {
            Some(Mode::Interp(d)) => Some(*d),
            _ => None,
        }
    }

    /// CODE 模式（顶层或插值内）的一个记号。
    ///
    /// 插值内（`INTERP`）与顶层 `CODE` 共用本函数，差异由模式栈细化：
    /// `\n` → 插值报错；`{` / `}` → 深度记账；depth==1 的 `:` → 格式说明符。
    fn code_token(&mut self) -> R<()> {
        let c = self.peek().unwrap_or('\0');
        match c {
            ' ' | '\t' | '\r' => {
                self.bump();
                Ok(())
            }
            '\n' => {
                if self.interp_depth().is_some() {
                    return Err(syntax(SyntaxMsg::InterpolationNewline, self.span()));
                }
                let sp = self.span();
                self.bump();
                self.push(TokenKind::Newline, sp);
                Ok(())
            }
            // 字符串开始（顶层 CODE 与 INTERP 内一致；INTERP 内即嵌套字符串）。
            '"' => {
                let sp = self.span();
                self.bump();
                self.push(TokenKind::StrBegin, sp);
                self.modes.push(Mode::Str);
                Ok(())
            }
            // 插值内的花括号深度记账。
            '{' if self.interp_depth().is_some() => {
                let sp = self.span();
                self.bump();
                if let Some(Mode::Interp(d)) = self.modes.last_mut() {
                    *d += 1;
                }
                self.push(TokenKind::LBrace, sp);
                Ok(())
            }
            '}' if self.interp_depth().is_some() => {
                let sp = self.span();
                self.bump();
                let depth = match self.modes.last_mut() {
                    Some(Mode::Interp(d)) => {
                        *d -= 1;
                        *d
                    }
                    _ => 0,
                };
                if depth == 0 {
                    self.push(TokenKind::InterpEnd, sp);
                    self.modes.pop();
                } else {
                    self.push(TokenKind::RBrace, sp);
                }
                Ok(())
            }
            // depth==1（插值顶层）的首个 `:` → 转格式说明符原始模式。
            ':' if self.interp_depth() == Some(1) => {
                let sp = self.span();
                self.bump();
                self.push(TokenKind::Colon, sp);
                self.modes.push(Mode::FormatSpec);
                Ok(())
            }
            '/' if self.peek_at(1) == Some('/') => {
                self.line_comment();
                Ok(())
            }
            '/' if self.peek_at(1) == Some('*') => self.block_comment(),
            c if c.is_ascii_digit() => {
                self.number();
                Ok(())
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                self.ident_or_keyword();
                Ok(())
            }
            _ => self.punct_or_error(),
        }
    }

    /// `STR` 模式（`format_spec == false`）或格式说明符原始模式（`true`）的一个片段。
    ///
    /// - `STR`：累积文本 → `Text`；`${` → `InterpBegin` 并压 `INTERP`；`"` → `StrEnd`；
    ///   转义解码；裸 `\n` → `UnterminatedString`。
    /// - `FormatSpec`：自 `:` 之后**原样**读到 `}`（不解析 `#` / `{}` / 转义），
    ///   发 `FormatSpec`（可为空串）后发 `InterpEnd` 并弹出 `FormatSpec` 与 `INTERP`。
    fn str_text(&mut self, format_spec: bool) -> R<()> {
        let sp = self.span();
        let mut text = String::new();
        loop {
            let c = match self.peek() {
                Some(c) => c,
                // EOF：由 `run` 统一报「字符串未闭合」。
                None => return Ok(()),
            };
            if format_spec {
                match c {
                    '}' => {
                        self.push(TokenKind::FormatSpec(std::mem::take(&mut text)), sp);
                        let sp2 = self.span();
                        self.bump();
                        self.push(TokenKind::InterpEnd, sp2);
                        self.modes.pop(); // FormatSpec
                        self.modes.pop(); // Interp
                        return Ok(());
                    }
                    '\n' => return Err(syntax(SyntaxMsg::InterpolationNewline, self.span())),
                    _ => {
                        self.bump();
                        text.push(c);
                    }
                }
                continue;
            }
            match c {
                '"' => {
                    self.flush_text(&text, sp);
                    let sp2 = self.span();
                    self.bump();
                    self.push(TokenKind::StrEnd, sp2);
                    self.modes.pop();
                    return Ok(());
                }
                '\n' => return Err(syntax(SyntaxMsg::UnterminatedString, self.span())),
                '$' if self.peek_at(1) == Some('{') => {
                    self.flush_text(&text, sp);
                    let sp2 = self.span();
                    self.bump(); // '$'
                    self.bump(); // '{'
                    self.push(TokenKind::InterpBegin, sp2);
                    self.modes.push(Mode::Interp(1));
                    return Ok(());
                }
                '\\' => {
                    let esc_sp = self.span();
                    self.bump(); // '\\'
                    let e = match self.peek() {
                        Some(e) => e,
                        None => return Err(syntax(SyntaxMsg::UnterminatedString, self.span())),
                    };
                    let decoded = match e {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        'e' => '\u{1B}',
                        '$' => '$',
                        other => {
                            return Err(syntax(SyntaxMsg::UnknownEscape { c: other }, esc_sp));
                        }
                    };
                    self.bump();
                    text.push(decoded);
                }
                // 其余字符（含 `{` `}` `#` 与未成 `${` 的 `$`）按字面入文本。
                _ => {
                    self.bump();
                    text.push(c);
                }
            }
        }
    }

    /// 仅当文本非空时发出 `TEXT` 片段。
    fn flush_text(&mut self, text: &str, sp: Span) {
        if !text.is_empty() {
            self.push(TokenKind::Text(text.to_string()), sp);
        }
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

    /// `/* … */`，不可嵌套；**已闭合**时等价于一个空格，不产生 `Newline`（§2.4）。
    /// 未闭合（`/*` 至 EOF 仍无 `*/`）→ `UnterminatedBlockComment`，`span` 指向 `/*` 的 `/`。
    fn block_comment(&mut self) -> R<()> {
        let sp = self.span();
        self.bump(); // '/'
        self.bump(); // '*'
        while let Some(c) = self.peek() {
            if c == '*' && self.peek_at(1) == Some('/') {
                self.bump();
                self.bump();
                return Ok(());
            }
            self.bump();
        }
        Err(syntax(SyntaxMsg::UnterminatedBlockComment, sp))
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
        // P3.3b 后 `"` 不再是非法字符（进入 STR 模式）
        assert_eq!(err_of("\"").0, SyntaxMsg::UnterminatedString);
    }

    // ---- 字符串与插值（§2.8，P3.3b）----

    #[test]
    fn empty_string_is_begin_end() {
        assert_eq!(
            kinds_no_eof("\"\""),
            vec![TokenKind::StrBegin, TokenKind::StrEnd]
        );
    }

    #[test]
    fn plain_string_is_one_text_fragment() {
        assert_eq!(
            kinds_no_eof("\"hello world\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::Text("hello world".into()),
                TokenKind::StrEnd,
            ]
        );
    }

    #[test]
    fn every_escape_decodes() {
        let cases = [
            ("\\n", '\n'),
            ("\\t", '\t'),
            ("\\r", '\r'),
            ("\\\\", '\\'),
            ("\\\"", '"'),
            ("\\e", '\u{1B}'),
            ("\\$", '$'),
        ];
        for (src, ch) in cases {
            let src = format!("\"a{src}b\"");
            assert_eq!(
                kinds_no_eof(&src),
                vec![
                    TokenKind::StrBegin,
                    TokenKind::Text(format!("a{ch}b")),
                    TokenKind::StrEnd,
                ],
                "src={src}"
            );
        }
    }

    #[test]
    fn escaped_dollar_is_literal_not_interp() {
        assert_eq!(
            kinds_no_eof("\"\\${x}\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::Text("${x}".into()),
                TokenKind::StrEnd,
            ]
        );
    }

    #[test]
    fn unknown_escape_is_error_at_backslash() {
        assert_eq!(err_of("\"\\q\"").0, SyntaxMsg::UnknownEscape { c: 'q' });
        assert_eq!(err_of("\"\\q\"").1, Span::new(1, 2));
        assert_eq!(err_of("\"\\{\"").0, SyntaxMsg::UnknownEscape { c: '{' });
        assert_eq!(err_of("\"\\'\"").0, SyntaxMsg::UnknownEscape { c: '\'' });
        assert_eq!(err_of("\"\\0\"").0, SyntaxMsg::UnknownEscape { c: '0' });
    }

    #[test]
    fn unterminated_string_at_eof_and_newline() {
        assert_eq!(err_of("\"abc").0, SyntaxMsg::UnterminatedString);
        assert_eq!(err_of("\"abc").1, Span::new(1, 5));
        // 裸换行：位置指向该 `\n`
        assert_eq!(err_of("\"a\nb\"").0, SyntaxMsg::UnterminatedString);
        assert_eq!(err_of("\"a\nb\"").1, Span::new(1, 3));
    }

    #[test]
    fn interpolation_single_and_multi_segment() {
        assert_eq!(
            kinds_no_eof("\"a${x}b\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::Text("a".into()),
                TokenKind::InterpBegin,
                TokenKind::Ident("x".into()),
                TokenKind::InterpEnd,
                TokenKind::Text("b".into()),
                TokenKind::StrEnd,
            ]
        );
        // 无文本片段的插值
        assert_eq!(
            kinds_no_eof("\"${x}\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::InterpBegin,
                TokenKind::Ident("x".into()),
                TokenKind::InterpEnd,
                TokenKind::StrEnd,
            ]
        );
    }

    #[test]
    fn nested_interpolation_and_nested_string() {
        // 插值内花括号 → 深度记账（LBrace / RBrace）
        assert_eq!(
            kinds_no_eof("\"${{a}}\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::InterpBegin,
                TokenKind::LBrace,
                TokenKind::Ident("a".into()),
                TokenKind::RBrace,
                TokenKind::InterpEnd,
                TokenKind::StrEnd,
            ]
        );
        // 嵌套字符串：`"a${ "b" }c"`
        assert_eq!(
            kinds_no_eof("\"a${ \"b\" }c\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::Text("a".into()),
                TokenKind::InterpBegin,
                TokenKind::StrBegin,
                TokenKind::Text("b".into()),
                TokenKind::StrEnd,
                TokenKind::InterpEnd,
                TokenKind::Text("c".into()),
                TokenKind::StrEnd,
            ]
        );
        // 嵌套字符串内再插值
        assert_eq!(
            kinds_no_eof("\"a${ \"b${y}\" }c\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::Text("a".into()),
                TokenKind::InterpBegin,
                TokenKind::StrBegin,
                TokenKind::Text("b".into()),
                TokenKind::InterpBegin,
                TokenKind::Ident("y".into()),
                TokenKind::InterpEnd,
                TokenKind::StrEnd,
                TokenKind::InterpEnd,
                TokenKind::Text("c".into()),
                TokenKind::StrEnd,
            ]
        );
    }

    #[test]
    fn format_spec_is_raw_until_brace() {
        assert_eq!(
            kinds_no_eof("\"${x:>3}\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::InterpBegin,
                TokenKind::Ident("x".into()),
                TokenKind::Colon,
                TokenKind::FormatSpec(">3".into()),
                TokenKind::InterpEnd,
                TokenKind::StrEnd,
            ]
        );
        // 空格式说明符允许
        assert_eq!(
            kinds_no_eof("\"${x:}\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::InterpBegin,
                TokenKind::Ident("x".into()),
                TokenKind::Colon,
                TokenKind::FormatSpec(String::new()),
                TokenKind::InterpEnd,
                TokenKind::StrEnd,
            ]
        );
        // 原始模式：`#` / `{` 均按字面（到**首个** `}` 为止）
        assert_eq!(
            kinds_no_eof("\"${x:#{a}\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::InterpBegin,
                TokenKind::Ident("x".into()),
                TokenKind::Colon,
                TokenKind::FormatSpec("#{a".into()),
                TokenKind::InterpEnd,
                TokenKind::StrEnd,
            ]
        );
        // depth>1 的 `:` 仍是普通 Colon（不触发格式说明符）
        assert_eq!(
            kinds_no_eof("\"${{x:y}}\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::InterpBegin,
                TokenKind::LBrace,
                TokenKind::Ident("x".into()),
                TokenKind::Colon,
                TokenKind::Ident("y".into()),
                TokenKind::RBrace,
                TokenKind::InterpEnd,
                TokenKind::StrEnd,
            ]
        );
    }

    #[test]
    fn interpolation_bare_newline_is_error() {
        assert_eq!(
            err_of("\"${x\n}\"").0,
            SyntaxMsg::InterpolationNewline
        );
        assert_eq!(err_of("\"${x\n}\"").1, Span::new(1, 5));
        // 插值字符串内裸换行仍是字符串未闭合
        assert_eq!(err_of("\"${ \"a\n\" }\"").0, SyntaxMsg::UnterminatedString);
    }

    #[test]
    fn hash_is_literal_in_str_but_error_in_interp() {
        // STR 内 `#` 字面
        assert_eq!(
            kinds_no_eof("\"#42\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::Text("#42".into()),
                TokenKind::StrEnd,
            ]
        );
        // INTERP 内 `#` 非法
        assert_eq!(err_of("\"${x#y}\"").0, SyntaxMsg::HashPosition);
        // 格式说明符内 `#` 字面（上面的 format_spec 测试已覆盖，此处补一条独立）
        assert_eq!(
            kinds_no_eof("\"${x:#}\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::InterpBegin,
                TokenKind::Ident("x".into()),
                TokenKind::Colon,
                TokenKind::FormatSpec("#".into()),
                TokenKind::InterpEnd,
                TokenKind::StrEnd,
            ]
        );
    }

    #[test]
    fn dollar_without_brace_is_literal() {
        assert_eq!(
            kinds_no_eof("\"a$b\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::Text("a$b".into()),
                TokenKind::StrEnd,
            ]
        );
    }

    #[test]
    fn braces_are_literal_in_str() {
        assert_eq!(
            kinds_no_eof("\"{a} # b\""),
            vec![
                TokenKind::StrBegin,
                TokenKind::Text("{a} # b".into()),
                TokenKind::StrEnd,
            ]
        );
    }

    #[test]
    fn unmatched_interp_brace_at_eof_is_unterminated() {
        assert_eq!(err_of("\"${x").0, SyntaxMsg::UnterminatedString);
        // EOF 落在格式说明符内同样报字符串未闭合
        assert_eq!(err_of("\"${x:").0, SyntaxMsg::UnterminatedString);
    }

    // ---- 未闭合块注释（v1 补钉 ADR）----

    #[test]
    fn unterminated_block_comment_is_error_at_slash() {
        let (msg, span) = err_of("/* abc");
        assert_eq!(msg, SyntaxMsg::UnterminatedBlockComment);
        assert_eq!(span, Span::new(1, 1));
        // 代码之后再开未闭合块注释：位置指向 `/*` 的 `/`
        assert_eq!(err_of("x\n  /* y").1, Span::new(2, 3));
    }

    #[test]
    fn closed_block_comment_still_acts_as_space() {
        // 对照：已闭合块注释行为不变（不报错）
        assert_eq!(
            kinds_no_eof("/* a */x"),
            vec![TokenKind::Ident("x".into())]
        );
    }

    // ---- 字符串跨行与 line_base ----

    #[test]
    fn line_base_holds_across_string_interp() {
        // 字符串在 body 第 2 行（line_base=1）+ 插值内标识符位置
        let src = "\"abc\"\n\"${xy}\"";
        let t = lex(src, 1).unwrap();
        assert_eq!(t[0].kind, TokenKind::StrBegin);
        assert_eq!(t[0].span, Span::new(2, 1)); // \"abc\" 的 StrBegin 在第 2 行
        assert_eq!(t[3].kind, TokenKind::Newline);
        assert_eq!(t[4].kind, TokenKind::StrBegin);
        assert_eq!(t[4].span, Span::new(3, 1)); // 第二个字符串在第 3 行第 1 列
        assert_eq!(t[6].kind, TokenKind::Ident("xy".into()));
        assert_eq!(t[6].span, Span::new(3, 4)); // 插值内 `xy`
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
