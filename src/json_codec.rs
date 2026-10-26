//! T12 严格 JSON 数据编解码。单遍字节游标，整数精确，递归容器深度有界。
//! 独立于bin历史runner协议；规范见interface-contract §16。
use crate::error::{value as value_error, R, ValueMsg};
use crate::span::Span;
use crate::value::Value;
use std::collections::{BTreeSet, HashSet};
use std::rc::Rc;

pub const JSON_DEPTH_LIMIT: usize = 128;
const INVALID: &str = "不是合法的 JSON";

/// 输入必须是一个完整JSON值；错误偏移按UTF-8字节计算。
pub fn parse(text: &str, span: Span) -> R<Value> {
    let mut p = Parser { text, pos: 0, span };
    p.ws();
    let value = p.value(0)?;
    p.ws();
    if p.pos != text.len() { return Err(p.error(INVALID)); }
    Ok(value)
}

struct Parser<'a> { text: &'a str, pos: usize, span: Span }
impl Parser<'_> {
    fn at_error(&self, offset: usize, reason: &str) -> Box<crate::error::LzError> {
        value_error(ValueMsg::JsonParse { offset, reason: reason.into() }, self.span)
    }
    fn error(&self, reason: &str) -> Box<crate::error::LzError> { self.at_error(self.pos, reason) }
    fn peek(&self) -> Option<u8> { self.text.as_bytes().get(self.pos).copied() }
    fn ws(&mut self) { while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) { self.pos += 1; } }
    fn expect(&mut self, c: u8) -> R<()> {
        if self.peek() == Some(c) { self.pos += 1; Ok(()) } else { Err(self.error(INVALID)) }
    }
    fn value(&mut self, depth: usize) -> R<Value> {
        match self.peek() {
            Some(b'[' | b'{') if depth >= JSON_DEPTH_LIMIT => Err(self.error("嵌套深度超过 128 层")),
            Some(b'[') => self.array(depth + 1),
            Some(b'{') => self.object(depth + 1),
            Some(b'"') => self.string().map(Value::string),
            Some(b't') => { self.word(b"true")?; Ok(Value::Bool(true)) }
            Some(b'f') => { self.word(b"false")?; Ok(Value::Bool(false)) }
            Some(b'n') => { self.word(b"null")?; Ok(Value::Nil) }
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.error(INVALID)),
        }
    }
    fn word(&mut self, word: &[u8]) -> R<()> {
        for &c in word { self.expect(c)?; }
        Ok(())
    }
    fn array(&mut self, depth: usize) -> R<Value> {
        self.pos += 1;
        self.ws();
        let mut items = Vec::new();
        if self.peek() == Some(b']') { self.pos += 1; return Ok(Value::array(items)); }
        loop {
            items.push(self.value(depth)?);
            self.ws();
            if self.peek() == Some(b']') { self.pos += 1; break; }
            self.expect(b',')?;
            self.ws();
        }
        Ok(Value::array(items))
    }
    fn object(&mut self, depth: usize) -> R<Value> {
        self.pos += 1;
        self.ws();
        let mut fields = Vec::new();
        let mut keys = BTreeSet::new();
        if self.peek() == Some(b'}') { self.pos += 1; return Ok(Value::object(fields)); }
        loop {
            let key_start = self.pos;
            let key = self.string()?;
            if !keys.insert(key.clone()) { return Err(self.at_error(key_start, "对象键重复")); }
            self.ws();
            self.expect(b':')?;
            self.ws();
            fields.push((key, self.value(depth)?));
            self.ws();
            if self.peek() == Some(b'}') { self.pos += 1; break; }
            self.expect(b',')?;
            self.ws();
        }
        Ok(Value::object(fields))
    }
    fn string(&mut self) -> R<String> {
        self.expect(b'"')?;
        let mut s = String::new();
        loop {
            match self.peek() {
                None | Some(0..=31) => return Err(self.error(INVALID)),
                Some(b'"') => { self.pos += 1; return Ok(s); }
                Some(b'\\') => {
                    self.pos += 1;
                    let c = match self.peek() {
                        Some(b'"') => '"', Some(b'\\') => '\\', Some(b'/') => '/',
                        Some(b'b') => '\u{8}', Some(b'f') => '\u{c}',
                        Some(b'n') => '\n', Some(b'r') => '\r', Some(b't') => '\t',
                        Some(b'u') => {
                            self.pos += 1;
                            let first = self.hex4()?;
                            let scalar = if (0xd800..=0xdbff).contains(&first) {
                                self.expect(b'\\')?; self.expect(b'u')?;
                                let low_start = self.pos;
                                let low = self.hex4()?;
                                if !(0xdc00..=0xdfff).contains(&low) { return Err(self.at_error(low_start, INVALID)); }
                                0x10000 + ((first - 0xd800) << 10) + low - 0xdc00
                            } else if (0xdc00..=0xdfff).contains(&first) {
                                return Err(self.at_error(self.pos - 4, INVALID));
                            } else { first };
                            s.push(char::from_u32(scalar).expect("已校验Unicode标量"));
                            continue;
                        }
                        _ => return Err(self.error(INVALID)),
                    };
                    self.pos += 1;
                    s.push(c);
                }
                Some(_) => {
                    let c = self.text[self.pos..].chars().next().expect("非空有效UTF-8");
                    self.pos += c.len_utf8();
                    s.push(c);
                }
            }
        }
    }
    fn hex4(&mut self) -> R<u32> {
        let mut n = 0;
        for _ in 0..4 {
            let digit = match self.peek() {
                Some(b'0'..=b'9') => u32::from(self.peek().unwrap() - b'0'),
                Some(b'a'..=b'f') => u32::from(self.peek().unwrap() - b'a') + 10,
                Some(b'A'..=b'F') => u32::from(self.peek().unwrap() - b'A') + 10,
                _ => return Err(self.error(INVALID)),
            };
            n = n * 16 + digit;
            self.pos += 1;
        }
        Ok(n)
    }
    fn digits(&mut self) -> R<()> {
        let start = self.pos;
        while matches!(self.peek(), Some(b'0'..=b'9')) { self.pos += 1; }
        if self.pos == start { return Err(self.error(INVALID)); }
        Ok(())
    }
    fn number(&mut self) -> R<Value> {
        let start = self.pos;
        if self.peek() == Some(b'-') { self.pos += 1; }
        match self.peek() {
            Some(b'0') => { self.pos += 1; if matches!(self.peek(), Some(b'0'..=b'9')) { return Err(self.error(INVALID)); } }
            Some(b'1'..=b'9') => self.digits()?,
            _ => return Err(self.error(INVALID)),
        }
        let mut is_float = false;
        if self.peek() == Some(b'.') { is_float = true; self.pos += 1; self.digits()?; }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true; self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) { self.pos += 1; }
            self.digits()?;
        }
        let token = &self.text[start..self.pos];
        if is_float {
            let n = token.parse::<f64>().map_err(|_| self.at_error(start, "浮点数超出有限范围"))?;
            if !n.is_finite() { return Err(self.at_error(start, "浮点数超出有限范围")); }
            Ok(Value::Float(n))
        } else {
            token.parse::<i64>().map(Value::Int).map_err(|_| self.at_error(start, "整数超出 i64 范围"))
        }
    }
}

/// 编码整个值后才返回；失败不泄漏部分结果。不调用用户方法。
pub fn stringify(value: &Value, span: Span) -> R<String> {
    let mut e = Encoder { out: String::new(), active: HashSet::new(), span };
    e.value(value, 0)?;
    Ok(e.out)
}
struct Encoder { out: String, active: HashSet<(u8, usize)>, span: Span }
impl Encoder {
    fn error(&self, reason: &str) -> Box<crate::error::LzError> {
        value_error(ValueMsg::JsonStringify { reason: reason.into() }, self.span)
    }
    fn enter(&mut self, identity: (u8, usize), depth: usize) -> R<()> {
        if depth >= JSON_DEPTH_LIMIT { return Err(self.error("嵌套深度超过 128 层")); }
        if !self.active.insert(identity) { return Err(self.error("循环引用")); }
        Ok(())
    }
    fn value(&mut self, value: &Value, depth: usize) -> R<()> {
        match value {
            Value::Nil => self.out.push_str("null"),
            Value::Bool(v) => self.out.push_str(if *v { "true" } else { "false" }),
            Value::Int(v) => self.out.push_str(&v.to_string()),
            Value::Float(v) => {
                if !v.is_finite() { return Err(self.error("浮点数必须有限")); }
                let s = v.to_string();
                self.out.push_str(&s);
                if !s.contains(['.', 'e', 'E']) { self.out.push_str(".0"); }
            }
            Value::Str(s) => self.string(s),
            Value::Array(a) => {
                let id = (0, Rc::as_ptr(a) as usize);
                self.enter(id, depth)?;
                self.out.push('[');
                for (i, v) in a.borrow().iter().enumerate() {
                    if i != 0 { self.out.push(','); }
                    self.value(v, depth + 1)?;
                }
                self.out.push(']');
                self.active.remove(&id);
            }
            Value::Struct(s) => {
                let id = (1, Rc::as_ptr(s) as usize);
                self.enter(id, depth)?;
                self.out.push('{');
                for (i, (key, v)) in s.borrow().data_fields_sorted().iter().enumerate() {
                    if i != 0 { self.out.push(','); }
                    self.string(key);
                    self.out.push(':');
                    self.value(v, depth + 1)?;
                }
                self.out.push('}');
                self.active.remove(&id);
            }
            Value::Func(_) | Value::StructDef(_) => return Err(self.error("不支持函数或结构体定义")),
        }
        Ok(())
    }
    fn string(&mut self, s: &str) {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        self.out.push('"');
        for c in s.chars() {
            match c {
                '"' => self.out.push_str("\\\""), '\\' => self.out.push_str("\\\\"),
                '\u{8}' => self.out.push_str("\\b"), '\u{c}' => self.out.push_str("\\f"),
                '\n' => self.out.push_str("\\n"), '\r' => self.out.push_str("\\r"), '\t' => self.out.push_str("\\t"),
                c if c < ' ' => {
                    self.out.push_str("\\u00");
                    self.out.push(HEX[(c as usize) >> 4] as char);
                    self.out.push(HEX[(c as usize) & 15] as char);
                }
                c => self.out.push(c),
            }
        }
        self.out.push('"');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::LzError;
    use crate::value::Closure;
    use crate::env::ScopeChain;
    const SITE: Span = Span { line: 7, col: 11 };
    fn parsed(text: &str) -> Value { parse(text, SITE).unwrap() }
    fn encoded(v: &Value) -> String { stringify(v, SITE).unwrap() }
    fn parse_error(text: &str, offset: usize, reason: &str) {
        let e = parse(text, SITE).unwrap_err();
        assert_eq!(e.span(), Some(SITE));
        assert_eq!(e.class_name(), "ValueError");
        match *e {
            LzError::Value { msg: ValueMsg::JsonParse { offset: actual, reason: actual_reason }, .. } => {
                assert_eq!(actual, offset, "{text:?}"); assert_eq!(actual_reason, reason, "{text:?}");
            }
            e => panic!("unexpected {e:?}"),
        }
    }
    fn encode_error(v: &Value, reason: &str) {
        let e = stringify(v, SITE).unwrap_err();
        assert_eq!(e.span(), Some(SITE));
        assert_eq!(e.message(), format!("JSON 编码失败：{reason}"));
    }
    #[test]
    fn exact_i64_and_float_type_sign_roundtrip() {
        for n in [i64::MIN, -9_007_199_254_740_993, 0, 9_007_199_254_740_993, i64::MAX] {
            assert!(matches!(parsed(&n.to_string()), Value::Int(v) if v == n));
            assert_eq!(encoded(&Value::Int(n)), n.to_string());
        }
        for n in [0.0_f64, -0.0, 1.0, f64::MIN_POSITIVE, f64::from_bits(1), f64::MAX, -1.25e100] {
            let s = encoded(&Value::Float(n));
            match parsed(&s) { Value::Float(v) => assert_eq!(n.to_bits(), v.to_bits()), _ => panic!("{s}") }
        }
        assert_eq!(encoded(&parsed("-0")), "0");
        assert_eq!(encoded(&parsed("-1e-9999")), "-0.0");
        parse_error("9223372036854775808", 0, "整数超出 i64 范围");
        parse_error("-9223372036854775809", 0, "整数超出 i64 范围");
        parse_error("1e9999", 0, "浮点数超出有限范围");
    }
    #[test]
    fn rejects_non_json_grammar_at_first_byte() {
        for (s, offset) in [("", 0), ("+1", 0), ("01", 1), ("-01", 2), ("-", 1),
            ("1.", 2), ("1e+", 3), ("truex", 4), ("[1,]", 3), ("{\"x\":0,}", 7),
            ("{x:0}", 1), ("'x'", 0), ("NaN", 0), ("Infinity", 0), ("/*x*/0", 0),
            ("\u{feff}0", 0), ("\u{a0}0", 0), ("\"x\n\"", 2), ("\"\\$\"", 2),
            ("\"\\u00xz\"", 5), ("\"unterminated", 13)] {
            parse_error(s, offset, INVALID);
        }
        parse_error("\"中\" x", 6, INVALID);
    }
    #[test]
    fn unicode_escapes_and_arbitrary_keys_are_data() {
        let v = parsed(r#"{"":"${x}","中":"\ud83d\ude00","x\u0000y":"\b\f\n\r\t\/\\\""}"#);
        let s = encoded(&v);
        assert!(s.contains("${x}")); assert!(!s.contains("\\$")); assert!(s.contains("😀"));
        assert!(v.deep_eq(&parsed(&s)));
        let all = (0u8..=31).map(char::from).collect::<String>() + "中😀/\\\"${42}";
        let v = Value::string(all.clone());
        assert_eq!(parsed(&encoded(&v)).as_str(), Some(all.as_str()));
        parse_error(r#""\ud800""#, 7, INVALID);
        parse_error(r#""\udc00""#, 3, INVALID);
        parse_error(r#""\ud800\u0061""#, 9, INVALID);
    }
    #[test]
    fn rejects_duplicate_decoded_keys_but_allows_nested_names() {
        parse_error(r#"{"a":0,"\u0061":1}"#, 7, "对象键重复");
        parse_error(r#"{"中":0,"\u4e2d":1}"#, 9, "对象键重复");
        assert_eq!(encoded(&parsed(r#"{"x":{"x":1}}"#)), r#"{"x":{"x":1}}"#);
    }
    #[test]
    fn container_depth_limit_matches_encoder() {
        for n in [0, 1, 127, 128] {
            let s = "[".repeat(n) + "0" + &"]".repeat(n);
            assert_eq!(encoded(&parsed(&s)), s);
        }
        let too_deep = "[".repeat(129) + "0" + &"]".repeat(129);
        parse_error(&too_deep, 128, "嵌套深度超过 128 层");
        let mut v = Value::Int(0);
        for _ in 0..129 { v = Value::array(vec![v]); }
        encode_error(&v, "嵌套深度超过 128 层");
        let mixed = "{\"x\":".repeat(128) + "null" + &"}".repeat(128);
        assert_eq!(encoded(&parsed(&mixed)), mixed);
    }
    #[test]
    fn cycles_rejected_but_shared_subtrees_repeat_and_do_not_mutate() {
        let a = Value::array(vec![Value::Int(1)]);
        let shared = Value::array(vec![a.clone(), a.clone()]);
        assert_eq!(encoded(&shared), "[[1],[1]]");
        if let Value::Array(rc) = &a {
            rc.borrow_mut().push(a.clone());
            encode_error(&a, "循环引用");
            rc.borrow_mut().pop(); // 不在单测保留Rc环。
        }
        assert_eq!(encoded(&shared), "[[1],[1]]");
        let o = Value::empty_object();
        if let Value::Struct(rc) = &o {
            rc.borrow_mut().set("self", o.clone());
            encode_error(&o, "循环引用");
            rc.borrow_mut().set("self", Value::Nil);
        }
    }
    #[test]
    fn method_filtering_key_order_and_invalid_value_precedence() {
        let function = Value::function(Closure::anonymous(Rc::new(ScopeChain::empty())));
        let o = Value::object(vec![("中".into(), Value::Int(3)), ("z".into(), Value::Int(2)),
            ("a".into(), Value::Int(1)), ("method".into(), function.clone())]);
        assert_eq!(encoded(&o), "{\"a\":1,\"z\":2,\"中\":3}");
        encode_error(&function, "不支持函数或结构体定义");
        encode_error(&Value::array(vec![function]), "不支持函数或结构体定义");
        encode_error(&Value::struct_def("X"), "不支持函数或结构体定义");
        for n in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            encode_error(&Value::Float(n), "浮点数必须有限");
        }
        let o = Value::object(vec![("z".into(), Value::Float(f64::INFINITY)),
            ("a".into(), Value::struct_def("X"))]);
        encode_error(&o, "不支持函数或结构体定义");
    }
    #[test]
    fn builtins_check_types_arity_and_default_context() {
        use crate::builtins::call;
        assert_eq!(call("argv", &[], SITE).unwrap().to_string(), "[]");
        for (name, args) in [("argv", vec![Value::Nil]), ("jsonParse", vec![]),
            ("jsonStringify", vec![]), ("jsonStringify", vec![Value::Nil, Value::Nil]),
            ("jsonParse", vec![Value::Int(0)])] {
            let e = call(name, &args, SITE).unwrap_err();
            assert_eq!(e.span(), Some(SITE)); assert_eq!(e.class_name(), "TypeError");
        }
        assert_eq!(call("jsonStringify", &[Value::Nil], SITE).unwrap().as_str(), Some("null"));
    }
}
