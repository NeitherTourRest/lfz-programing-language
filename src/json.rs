//! 极简 JSON 解析器 + 紧凑序列化器（仅 std）。
//!
//! - **解析**（[`parse`]）：供 `lfz test` 读取 `cases.json` 用例清单。覆盖标准 JSON 词法：
//!   对象 / 数组 / 字符串（含 `\uXXXX` 与常用转义）/ 数字 / `true` / `false` / `null`。
//! - **编码**（[`encode`]）：供 `lfz run --json` / `lfz test --json` 输出机器可读结果
//!   （`semantics.md` §8.3 示例 4 / §8.4）。紧凑单行、非 ASCII 原样（UTF-8）。
//!
//! 只实现"够用且正确"的子集：不追求性能，不做 JSON5 扩展，不加第三方依赖（D-011）。
//!
//! 契约：`docs/tooling/runner-contract.md`（`cases.json` 清单 schema + `--json` 输出 schema）。
//! 本模块属 **bin crate** 私有工具（`main.rs` 内 `mod json;`），**不进** 库 crate。

use std::fmt;

/// JSON 值。
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    /// `null`。
    Null,
    /// `true` / `false`。
    Bool(bool),
    /// 浮点数字（解析器产出；输出构造不产生）。
    Num(f64),
    /// 整数字面量（**仅供输出构造**，如 `--json` 的 `line` / `col` / 计数；解析器恒产出 [`Json::Num`]）。
    Int(i64),
    /// 字符串。
    Str(String),
    /// 数组。
    Arr(Vec<Json>),
    /// 对象（保序键值对；重复键取首次）。
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// 取对象字段（非对象 / 无此键 → `None`）。
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(m) => m.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// 字符串取值（非字符串 → `None`）。
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    /// 数组取值（非数组 → `None`）。
    #[must_use]
    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }
}

/// 序列化为**紧凑单行 JSON**（无多余空白；非 ASCII 原样输出，保持 UTF-8）。
///
/// 供 `--json`（`semantics.md` §8.3 示例 4 / §8.4）输出机器可读结果；对象字段按 `Vec` 顺序，
/// 保证字段序可复现（与示例 4 逐字符对齐）。
#[must_use]
pub fn encode(v: &Json) -> String {
    let mut out = String::new();
    encode_into(v, &mut out);
    out
}

/// 递归写出一个 [`Json`] 值到 `out`（紧凑格式：`:` / `,` 前后无空格）。
fn encode_into(v: &Json, out: &mut String) {
    match v {
        Json::Null => out.push_str("null"),
        Json::Bool(true) => out.push_str("true"),
        Json::Bool(false) => out.push_str("false"),
        Json::Int(n) => out.push_str(&n.to_string()),
        Json::Num(n) => out.push_str(&n.to_string()),
        Json::Str(s) => encode_str(s, out),
        Json::Arr(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                encode_into(item, out);
            }
            out.push(']');
        }
        Json::Obj(fields) => {
            out.push('{');
            for (i, (key, val)) in fields.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                encode_str(key, out);
                out.push(':');
                encode_into(val, out);
            }
            out.push('}');
        }
    }
}

/// 写出一个 JSON 字符串字面量（含 `"` 包裹）。
///
/// - 转义 `"` / `\` 与所有 `U+0000..=U+001F` 控制字符（用 `\b \f \n \r \t` 或 `\u00xx`）；
/// - **非 ASCII（含中文）原样输出**，不转 `\uXXXX`——与 §8.3 示例 4 的 `"message":"你忘记了…"` 一致。
fn encode_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// 解析错误：`msg` + 字符偏移 `pos`（0-based，便于定位）。
#[derive(Debug)]
pub struct ParseError {
    /// 人类可读中文消息。
    pub msg: String,
    /// 出错处的**字符**偏移（0-based）。
    pub pos: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}（字符偏移 {}）", self.msg, self.pos)
    }
}

/// 解析整段文本为一个 JSON 值；**末尾仅允许空白**。
///
/// 容忍开头一个 UTF-8 BOM（`U+FEFF`）——Windows 编辑器 / `Set-Content` 常写入 BOM。
pub fn parse(text: &str) -> Result<Json, ParseError> {
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
    let mut p = Parser {
        chars: text.chars().collect(),
        pos: 0,
    };
    p.skip_ws();
    let v = p.value()?;
    p.skip_ws();
    if p.pos != p.chars.len() {
        return Err(p.err("JSON 末尾存在多余内容"));
    }
    Ok(v)
}

/// 逐字符下降解析器。
struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn err(&self, msg: &str) -> ParseError {
        ParseError {
            msg: msg.to_string(),
            pos: self.pos,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.pos += 1;
        }
    }

    /// 若当前位置正好是 `w`，消费并返回 `true`。
    fn eat(&mut self, w: &str) -> bool {
        let n = w.chars().count();
        if self.chars.len() >= self.pos + n
            && self.chars[self.pos..self.pos + n]
                .iter()
                .copied()
                .eq(w.chars())
        {
            self.pos += n;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, c: char) -> Result<(), ParseError> {
        if self.peek() == Some(c) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.err(&format!("期待 '{c}'")))
        }
    }

    fn value(&mut self) -> Result<Json, ParseError> {
        match self.peek() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => Ok(Json::Str(self.string()?)),
            Some('t' | 'f' | 'n') => self.literal(),
            Some(c) if c == '-' || c.is_ascii_digit() => self.number(),
            _ => Err(self.err("非法 JSON 值")),
        }
    }

    fn literal(&mut self) -> Result<Json, ParseError> {
        if self.eat("true") {
            return Ok(Json::Bool(true));
        }
        if self.eat("false") {
            return Ok(Json::Bool(false));
        }
        if self.eat("null") {
            return Ok(Json::Null);
        }
        Err(self.err("非法字面量（仅支持 true / false / null）"))
    }

    fn number(&mut self) -> Result<Json, ParseError> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.peek() == Some('.') {
            self.pos += 1;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.pos += 1;
            }
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse::<f64>()
            .map(Json::Num)
            .map_err(|_| ParseError {
                msg: "非法数字".to_string(),
                pos: start,
            })
    }

    fn string(&mut self) -> Result<String, ParseError> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.bump() {
                None => return Err(self.err("字符串未闭合")),
                Some('"') => return Ok(out),
                Some('\\') => {
                    let esc = self.bump().ok_or_else(|| self.err("转义序列未完成"))?;
                    match esc {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'b' => out.push('\u{0008}'),
                        'f' => out.push('\u{000C}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => out.push(self.unicode_escape()?),
                        other => return Err(self.err(&format!("不支持的转义 '\\{other}'"))),
                    }
                }
                Some(c) => out.push(c),
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, ParseError> {
        let mut n: u32 = 0;
        for _ in 0..4 {
            let c = self.bump().ok_or_else(|| self.err("\\u 转义不完整"))?;
            let d = c
                .to_digit(16)
                .ok_or_else(|| self.err("\\u 转义含非十六进制字符"))?;
            n = n * 16 + d;
        }
        Ok(n)
    }

    fn unicode_escape(&mut self) -> Result<char, ParseError> {
        let hi = self.hex4()?;
        // UTF-16 代理对。
        if (0xD800..=0xDBFF).contains(&hi) {
            if !self.eat("\\u") {
                return Err(self.err("\\u 高代理后缺少低代理"));
            }
            let lo = self.hex4()?;
            if !(0xDC00..=0xDFFF).contains(&lo) {
                return Err(self.err("非法 UTF-16 代理对"));
            }
            let cp = 0x1_0000 + ((hi - 0xD800) << 10) + (lo - 0xDC00);
            return char::from_u32(cp).ok_or_else(|| self.err("非法码点"));
        }
        char::from_u32(hi).ok_or_else(|| self.err("非法码点"))
    }

    fn object(&mut self) -> Result<Json, ParseError> {
        self.expect('{')?;
        self.skip_ws();
        let mut out = Vec::new();
        if self.peek() == Some('}') {
            self.pos += 1;
            return Ok(Json::Obj(out));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some('"') {
                return Err(self.err("对象键必须是字符串"));
            }
            let key = self.string()?;
            self.skip_ws();
            self.expect(':')?;
            self.skip_ws();
            let val = self.value()?;
            out.push((key, val));
            self.skip_ws();
            match self.bump() {
                Some(',') => continue,
                Some('}') => return Ok(Json::Obj(out)),
                _ => return Err(self.err("对象期待 ',' 或 '}'")),
            }
        }
    }

    fn array(&mut self) -> Result<Json, ParseError> {
        self.expect('[')?;
        self.skip_ws();
        let mut out = Vec::new();
        if self.peek() == Some(']') {
            self.pos += 1;
            return Ok(Json::Arr(out));
        }
        loop {
            self.skip_ws();
            let val = self.value()?;
            out.push(val);
            self.skip_ws();
            match self.bump() {
                Some(',') => continue,
                Some(']') => return Ok(Json::Arr(out)),
                _ => return Err(self.err("数组期待 ',' 或 ']'")),
            }
        }
    }
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manifest_shape() {
        let text = r#"{ "cases": [
            { "path": "fixtures/a.lfz", "expect": { "error": "CosmosAnswerError" } },
            { "path": "cases/b.lfz" }
        ] }"#;
        let v = parse(text).expect("应解析成功");
        let cases = v.get("cases").and_then(Json::as_array).expect("cases 数组");
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].get("path").and_then(Json::as_str), Some("fixtures/a.lfz"));
        assert_eq!(
            cases[0]
                .get("expect")
                .and_then(|e| e.get("error"))
                .and_then(Json::as_str),
            Some("CosmosAnswerError")
        );
        assert!(cases[1].get("expect").is_none());
    }

    #[test]
    fn parses_escapes_numbers_and_empty_containers() {
        let v = parse(r#"{"s":"a\"b\n\u4f60\u597d","n":-1.5e2,"b":true,"z":null,"e":{},"a":[]}"#)
            .expect("应解析成功");
        assert_eq!(v.get("s").and_then(Json::as_str), Some("a\"b\n你好"));
        assert_eq!(v.get("n"), Some(&Json::Num(-150.0)));
        assert_eq!(v.get("b"), Some(&Json::Bool(true)));
        assert_eq!(v.get("z"), Some(&Json::Null));
        assert_eq!(v.get("e").and_then(Json::as_array), None);
        assert!(v.get("a").and_then(Json::as_array).unwrap().is_empty());
    }

    #[test]
    fn tolerates_leading_bom() {
        let v = parse("\u{FEFF}{ \"cases\": [] }").expect("应容忍 BOM");
        assert!(v.get("cases").and_then(Json::as_array).unwrap().is_empty());
    }

    #[test]
    fn rejects_trailing_garbage_and_malformed() {
        assert!(parse("{} x").is_err());
        assert!(parse("{").is_err());
        assert!(parse("[1,]").is_err());
        assert!(parse("\"unterminated").is_err());
    }

    // ---- `encode`：`--json` 输出（§8.3 示例 4 逐字符） --------------------

    /// 逐字符复刻 `semantics.md` §8.3 示例 4（`CosmosAnswerError`）。
    #[test]
    fn encodes_example4_byte_for_byte() {
        let v = Json::Obj(vec![
            ("ok".to_string(), Json::Bool(false)),
            ("error".to_string(), Json::Str("CosmosAnswerError".to_string())),
            (
                "message".to_string(),
                Json::Str("你忘记了宇宙的答案".to_string()),
            ),
            ("file".to_string(), Json::Str("forgot.lfz".to_string())),
            ("line".to_string(), Json::Int(1)),
            ("col".to_string(), Json::Int(1)),
            (
                "traceback".to_string(),
                Json::Arr(vec![Json::Obj(vec![
                    ("file".to_string(), Json::Str("forgot.lfz".to_string())),
                    ("line".to_string(), Json::Int(1)),
                    ("func".to_string(), Json::Str("<module>".to_string())),
                ])]),
            ),
        ]);
        assert_eq!(
            encode(&v),
            concat!(
                r#"{"ok":false,"error":"CosmosAnswerError","message":"你忘记了宇宙的答案","#,
                r#""file":"forgot.lfz","line":1,"col":1,"#,
                r#""traceback":[{"file":"forgot.lfz","line":1,"func":"<module>"}]}"#
            )
        );
    }

    /// 转义：`"` / `\` / `\n` / `\t` / 控制字符 `\u0001`。
    #[test]
    fn encodes_escapes_and_control_chars() {
        let v = Json::Str("a\"b\\c\nd\te\u{0001}".to_string());
        assert_eq!(encode(&v), "\"a\\\"b\\\\c\\nd\\te\\u0001\"");
    }

    /// 标量与空容器。
    #[test]
    fn encodes_scalars_and_empty_containers() {
        assert_eq!(encode(&Json::Null), "null");
        assert_eq!(encode(&Json::Bool(true)), "true");
        assert_eq!(encode(&Json::Bool(false)), "false");
        assert_eq!(encode(&Json::Int(-7)), "-7");
        assert_eq!(encode(&Json::Arr(Vec::new())), "[]");
        assert_eq!(encode(&Json::Obj(Vec::new())), "{}");
    }

    /// 非 ASCII 原样输出，不转 `\uXXXX`。
    #[test]
    fn encode_keeps_non_ascii_raw() {
        assert_eq!(encode(&Json::Str("中文 ✓".to_string())), "\"中文 ✓\"");
    }
}
