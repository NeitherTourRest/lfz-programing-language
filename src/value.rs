//! 运行时值模型：`enum Value` + `Rc<RefCell<...>>` 引用语义（A1）。
//!
//! 依据（只读消费）：
//! - `docs/spec/semantics.md` §4.5.0（值模型总览）、§4.5.2（A1 引用语义）、§4.5.3（A2 闭包捕获）、
//!   §4.5.7（`int → float` 唯一隐式加宽）、§4.5.11（字符串不可变）、§3.7（显示形式）。
//! - `docs/spec/interface-contract.md` §10.5（闭包携带 `ScopeDebug` 链）、§10.7（`type` 依赖 `type_name`）。
//!
//! ## 内存布局
//!
//! | 类别 | 表示 | 语义 |
//! |------|------|------|
//! | `nil` / `bool` / `int(i64)` / `float(f64)` | 内联标量 | 值语义；`Clone` 即拷贝 |
//! | `string` | `Rc<String>` | **不可变**（§4.5.11）；`Clone` 仅计数 +1 |
//! | `array` | `Rc<RefCell<Vec<Value>>>` | **引用语义**（A1）：`Clone` 共享同一容器 |
//! | `struct` 实例 | `Rc<RefCell<StructObj>>` | **引用语义**（A1） |
//! | `struct` 模板 | `Rc<StructDef>` | 非实例；`type` 报 `"struct"`，显示 `<struct 名字>` |
//! | `function` | `Rc<Closure>` | 闭包按 **cell 引用**捕获（A2；cell 由 [`crate::env`] 承载） |
//!
//! 复合值的 `Clone` 共享同一 `Rc` 分配，故 `let b = a` 后 `b[i] = v` / `b.k = v` 对 `a` 可见
//! （A1：`a` 与 `b` 是同一容器）。标量按值拷贝。
//! **唯一**允许的隐式转换 `int → float` 集中入口是 [`Value::as_f64`]（§4.5.7）——禁止在别处散落。

use crate::env::ScopeChain;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

// ===========================================================================
// 值
// ===========================================================================

/// LFZ v1 运行时值。
///
/// 覆盖 `type()` 的八类名字（`nil` / `bool` / `int` / `float` / `string` / `array` /
/// `struct` / `function`，§10.7），另含 `struct` **模板**（非实例，`type` 亦报 `"struct"`，
/// 显示为 `<struct 名字>`，§4.5.0 / §3.7）。
#[derive(Clone, Debug)]
pub enum Value {
    /// `nil`。
    Nil,
    /// `bool`。
    Bool(bool),
    /// `int`（有符号 64 位）。
    Int(i64),
    /// `float`（IEEE-754 双精度）。
    Float(f64),
    /// `string`（**不可变**，§4.5.11）。
    Str(Rc<String>),
    /// `array` —— 引用语义（A1）。
    Array(Rc<RefCell<Vec<Value>>>),
    /// `struct` 实例 —— 引用语义（A1）。
    Struct(Rc<RefCell<StructObj>>),
    /// `struct` 模板（非实例）。
    StructDef(Rc<StructDef>),
    /// 函数 / 闭包 —— 可调用值；相等仅按同一性（§4.2，由 P3.8 求值器实现）。
    Func(Rc<Closure>),
}

impl Value {
    // ----- 构造便捷函数 -----------------------------------------------------

    /// 构造 `string`（不可变）。
    pub fn string(s: impl Into<String>) -> Self {
        Value::Str(Rc::new(s.into()))
    }

    /// 构造 `array`。
    pub fn array(items: Vec<Value>) -> Self {
        Value::Array(Rc::new(RefCell::new(items)))
    }

    /// 构造 `struct` **实例**（字段按书写序存放）。
    pub fn object(fields: Vec<(String, Value)>) -> Self {
        Value::Struct(Rc::new(RefCell::new(StructObj::from_fields(fields))))
    }

    /// 构造匿名 `struct` 实例。
    pub fn empty_object() -> Self {
        Value::Struct(Rc::new(RefCell::new(StructObj::new())))
    }

    /// 构造 `function`（函数 / 闭包）。
    pub fn function(closure: Closure) -> Self {
        Value::Func(Rc::new(closure))
    }

    /// 构造 `struct` **模板**。
    pub fn struct_def(name: impl Into<Rc<str>>) -> Self {
        Value::StructDef(Rc::new(StructDef { name: name.into() }))
    }

    // ----- 类型名（§10.7 `type`） -------------------------------------------

    /// 返回 `type(x)` 的字符串（§10.7）：`"int"` / `"float"` / `"string"` / `"bool"` /
    /// `"nil"` / `"array"` / `"struct"` / `"function"`。`struct` 模板亦报 `"struct"`。
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Str(_) => "string",
            Value::Array(_) => "array",
            Value::Struct(_) | Value::StructDef(_) => "struct",
            Value::Func(_) => "function",
        }
    }

    // ----- 唯一隐式转换：int → float（§4.5.7） ------------------------------

    /// **唯一**的 `int → float` 加宽入口（§4.5.7）。
    ///
    /// - `int` → IEEE-754 最近偶数舍入（`|int| > 2^53` 时**可能不精确**，不再称"无损"）；
    /// - `float` → 原样返回；
    /// - 其它类型 → `None`（不得隐式转换）。
    ///
    /// 混合算术与 `floor` / `ceil` / `round` / `sqrt` / `pow` 的 `int` 实参加宽**必须**走此处，
    /// 禁止在别处写 `as f64` 散落隐式转换。`abs` 同型、不走此处（§10.7）。
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Float(x) => Some(*x),
            _ => None,
        }
    }

    // ----- 取值访问器（严格同型，不隐式转换） -------------------------------

    /// 严格取 `int`（不做任何收窄 / 加宽）。
    #[must_use]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// 严格取 `float`（**不**加宽 `int`；加宽请用 [`Value::as_f64`]）。
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(x) => Some(*x),
            _ => None,
        }
    }

    /// 严格取 `bool`。
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// 严格取 `string` 内容。
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// 取 `array` 的共享容器（引用语义）。
    #[must_use]
    pub fn as_array(&self) -> Option<&Rc<RefCell<Vec<Value>>>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// 取 `struct` 实例的共享容器（引用语义）。
    #[must_use]
    pub fn as_struct(&self) -> Option<&Rc<RefCell<StructObj>>> {
        match self {
            Value::Struct(s) => Some(s),
            _ => None,
        }
    }

    /// 取 `function` 的闭包。
    #[must_use]
    pub fn as_closure(&self) -> Option<&Rc<Closure>> {
        match self {
            Value::Func(c) => Some(c),
            _ => None,
        }
    }

    // ----- 显示（§3.7） -----------------------------------------------------

    /// 顶层显示形式（§3.7）：`string` **原样不加引号**。等价于 `Value::to_string()`。
    #[must_use]
    pub fn render(&self) -> String {
        self.to_string()
    }

    /// 内部渲染：`top == true` 表示顶层（`string` 裸输出）；容器递归走 `top == false`。
    /// `path` 为**当前路径**上的容器身份集合（环安全，§3.7 / §4.5.9）。
    fn fmt_value(
        &self,
        f: &mut fmt::Formatter<'_>,
        top: bool,
        path: &mut Vec<usize>,
    ) -> fmt::Result {
        match self {
            Value::Nil => f.write_str("nil"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(x) => fmt_float(f, *x),
            Value::Str(s) => {
                if top {
                    f.write_str(s)
                } else {
                    fmt_quoted(f, s)
                }
            }
            Value::Array(cell) => {
                let id = Rc::as_ptr(cell) as usize;
                if path.contains(&id) {
                    return f.write_str("<cycle>");
                }
                path.push(id);
                f.write_str("[")?;
                let mut first = true;
                for v in cell.borrow().iter() {
                    if !first {
                        f.write_str(", ")?;
                    }
                    first = false;
                    v.fmt_value(f, false, path)?;
                }
                f.write_str("]")?;
                path.pop();
                Ok(())
            }
            Value::Struct(cell) => {
                let id = Rc::as_ptr(cell) as usize;
                if path.contains(&id) {
                    return f.write_str("<cycle>");
                }
                path.push(id);
                // 数据面：跳过函数值字段（A5），键按 UTF-8 字节序升序（B3）。
                let data = cell.borrow();
                f.write_str("{")?;
                let mut first = true;
                for (k, v) in data.data_fields_sorted() {
                    if !first {
                        f.write_str(", ")?;
                    }
                    first = false;
                    write!(f, "{k}: ")?;
                    v.fmt_value(f, false, path)?;
                }
                f.write_str("}")?;
                drop(data);
                path.pop();
                Ok(())
            }
            Value::StructDef(d) => write!(f, "<struct {}>", d.name),
            Value::Func(c) => match &c.name {
                Some(n) => write!(f, "<fn {n}>"),
                None => f.write_str("<fn>"),
            },
        }
    }
}

impl fmt::Display for Value {
    /// §3.7 显示形式（供 `print` / `str` / `;;` 复用）；**环安全**（已在本路径上的容器 → `<cycle>`）。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_value(f, true, &mut Vec::new())
    }
}

/// 显示 `float`（§3.7 / §4.5.6）：最短往返；整值浮点带 `.0`；`NaN` → `nan`、`±Inf` → `inf` / `-inf`。
fn fmt_float(f: &mut fmt::Formatter<'_>, x: f64) -> fmt::Result {
    if x.is_nan() {
        f.write_str("nan")
    } else if x.is_infinite() {
        f.write_str(if x > 0.0 { "inf" } else { "-inf" })
    } else {
        // `{:?}` 对 f64 为最短往返且保 `.0`（`1.0`）；`{}` 会丢小数点。
        write!(f, "{x:?}")
    }
}

/// 显示**嵌套**字符串（§3.7）：加 `"`，内部按转义输出。
///
/// 转义集取自词法集 `\n \t \r \\ \" \e`（`\e` = ESC 0x1B，syntax.md §2.8）；
/// 另将 `${`（插值触发序列）中的 `$` 转义为 `\$`，其余 `$` 原样（保证可往返）。
fn fmt_quoted(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    f.write_str("\"")?;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => f.write_str("\\\\")?,
            '"' => f.write_str("\\\"")?,
            '\n' => f.write_str("\\n")?,
            '\r' => f.write_str("\\r")?,
            '\t' => f.write_str("\\t")?,
            '\u{1b}' => f.write_str("\\e")?,
            '$' if chars.peek() == Some(&'{') => f.write_str("\\$")?,
            other => write!(f, "{other}")?,
        }
    }
    f.write_str("\"")
}

// ===========================================================================
// struct 实例
// ===========================================================================

/// `struct` **实例**（引用语义，A1）。
///
/// - 字段按**插入序**存储（含方法字段）；支持**动态加字段**（§4.5.8）。
/// - **数据面**（显示 / `keys` / `values` / `entries` / `len` / `has` / `==`）一律
///   **过滤函数值字段**（A5），且按键的 **UTF-8 字节序升序**访问（B3）。
#[derive(Clone, Debug, Default)]
pub struct StructObj {
    fields: Vec<(String, Value)>,
}

impl StructObj {
    /// 空实例。
    #[must_use]
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// 由字段（插入序）构造。
    #[must_use]
    pub fn from_fields(fields: Vec<(String, Value)>) -> Self {
        Self { fields }
    }

    /// 读取字段（**含**方法字段；`s.k` 与 `s["k"]` 均走此处）；缺失 → `None`。
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// 写入 / 新增字段（**原地**修改，A1）：已存在则替换，否则动态新增（§4.5.8）。
    pub fn set(&mut self, key: &str, value: Value) {
        for (k, v) in &mut self.fields {
            if k == key {
                *v = value;
                return;
            }
        }
        self.fields.push((key.to_string(), value));
    }

    /// 是否存在该字段（**含**方法字段）。
    #[must_use]
    pub fn contains(&self, key: &str) -> bool {
        self.fields.iter().any(|(k, _)| k == key)
    }

    /// 全部字段（插入序，**含**方法字段）。
    #[must_use]
    pub fn raw_fields(&self) -> &[(String, Value)] {
        &self.fields
    }

    /// **数据字段**（跳过函数值字段，A5），按**键的 UTF-8 字节序升序**（B3）。
    #[must_use]
    pub fn data_fields_sorted(&self) -> Vec<(&str, &Value)> {
        let mut out: Vec<(&str, &Value)> = self
            .fields
            .iter()
            .filter(|(_, v)| !matches!(v, Value::Func(_)))
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        out.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        out
    }

    /// **数据字段**数量（不含方法，A5）——`len(s)` 用。
    #[must_use]
    pub fn data_len(&self) -> usize {
        self.fields
            .iter()
            .filter(|(_, v)| !matches!(v, Value::Func(_)))
            .count()
    }
}

// ===========================================================================
// struct 模板 与 函数 / 闭包
// ===========================================================================

/// `struct` **模板**（非实例）。`type` 报 `"struct"`；显示 `<struct 名字>`（§3.7 / §4.5.0）。
///
/// P3.6 仅承载模板名；字段声明 / 默认值 / 方法表由 P3.8 实例化语义（§4.5.8）落地时扩展本结构。
#[derive(Clone, Debug)]
pub struct StructDef {
    /// 模板名（`;;` 可见绑定，§3.6）。
    pub name: Rc<str>,
}

/// 函数 / 闭包值（可调用）。
///
/// P3.6 仅承载**显示名**与**词法定义处的 [`ScopeChain`]**（§10.5）；形参、函数体 AST、
/// 捕获 cell 列表等由 P3.7 / P3.8 在求值器落地时扩展本结构。
#[derive(Clone, Debug)]
pub struct Closure {
    /// 显示名：命名函数 `Some("inc")` → `<fn inc>`；匿名 `None` → `<fn>`（§3.7）。
    pub name: Option<Rc<str>>,
    /// 词法定义处的 `ScopeDebug` 可见链（`Rc` 共享，§10.5）：供函数体内 `;;` 看到闭包捕获到的外层名字。
    pub def_scope: Rc<ScopeChain>,
}

impl Closure {
    /// 命名函数 / 闭包。
    pub fn named(name: impl Into<Rc<str>>, def_scope: Rc<ScopeChain>) -> Self {
        Self {
            name: Some(name.into()),
            def_scope,
        }
    }

    /// 匿名函数 / λ / 闭包。
    pub fn anonymous(def_scope: Rc<ScopeChain>) -> Self {
        Self {
            name: None,
            def_scope,
        }
    }
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::ScopeChain;

    /// 构造一个“无可见变量”的闭包值（测试辅助）。
    fn dummy_fn() -> Value {
        Value::function(Closure::anonymous(Rc::new(ScopeChain::empty())))
    }

    #[test]
    fn value_is_two_words() {
        // 标量内联 + 复合类型仅一个 `Rc` 指针 → `Value` 为 2 个机器字（16B）；
        // 一旦引入更大负载会在此处显式暴露（架构目标见 DRAFT-runtime-arch §2.1）。
        assert_eq!(std::mem::size_of::<Value>(), 16);
    }

    #[test]
    fn type_name_covers_all_eight_classes() {
        let cases: [(Value, &str); 8] = [
            (Value::Nil, "nil"),
            (Value::Bool(true), "bool"),
            (Value::Int(-7), "int"),
            (Value::Float(1.5), "float"),
            (Value::string("s"), "string"),
            (Value::array(vec![]), "array"),
            (Value::empty_object(), "struct"),
            (dummy_fn(), "function"),
        ];
        for (v, want) in cases {
            assert_eq!(v.type_name(), want);
        }
        // `struct` 模板 `type` 亦报 "struct"（§4.5.0）。
        assert_eq!(Value::struct_def("Point").type_name(), "struct");
    }

    #[test]
    fn a1_array_reference_is_shared_between_bindings() {
        let a = Value::array(vec![Value::Int(1)]);
        let b = a.clone(); // 引用语义：b 与 a 指向同一容器（A1）
        if let Value::Array(rc) = &a {
            rc.borrow_mut().push(Value::Int(2)); // 经 a 原地修改
        }
        // b 立即可见（同一容器）。
        assert_eq!(b.to_string(), "[1, 2]");
        assert_eq!(a.to_string(), "[1, 2]");
        match (&a, &b) {
            (Value::Array(x), Value::Array(y)) => assert!(Rc::ptr_eq(x, y)),
            _ => panic!("expected arrays"),
        }
    }

    #[test]
    fn a1_struct_field_write_is_visible_through_aliases() {
        let s = Value::object(vec![("k".to_string(), Value::Int(1))]);
        let t = s.clone();
        if let Value::Struct(rc) = &s {
            rc.borrow_mut().set("k", Value::Int(2)); // 原地修改（A1）
        }
        assert_eq!(t.to_string(), "{k: 2}");
        assert_eq!(s.to_string(), "{k: 2}");
        match (&s, &t) {
            (Value::Struct(x), Value::Struct(y)) => assert!(Rc::ptr_eq(x, y)),
            _ => panic!("expected structs"),
        }
    }

    #[test]
    fn display_scalar_forms() {
        assert_eq!(Value::Nil.to_string(), "nil");
        assert_eq!(Value::Bool(false).to_string(), "false");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Int(-42).to_string(), "-42");
        // 整值浮点带 `.0`（§3.7）。
        assert_eq!(Value::Float(1.0).to_string(), "1.0");
        assert_eq!(Value::Float(-0.5).to_string(), "-0.5");
        assert_eq!(Value::Float(2.5).to_string(), "2.5");
        // IEEE 特殊值（§4.5.6）。
        assert_eq!(Value::Float(f64::NAN).to_string(), "nan");
        assert_eq!(Value::Float(f64::INFINITY).to_string(), "inf");
        assert_eq!(Value::Float(f64::NEG_INFINITY).to_string(), "-inf");
        // 顶层字符串原样、不加引号（§3.7）。
        assert_eq!(Value::string("hi").to_string(), "hi");
        assert_eq!(Value::string("a\nb").to_string(), "a\nb");
    }

    #[test]
    fn display_string_top_raw_nested_quoted() {
        // 嵌套：加引号 + 转义。
        let a = Value::array(vec![Value::string("a\"b\n\t\\")]);
        assert_eq!(a.to_string(), "[\"a\\\"b\\n\\t\\\\\"]");
        // `$` 仅在 `${` 前需转义。
        let b = Value::array(vec![Value::string("${x}")]);
        assert_eq!(b.to_string(), "[\"\\${x}\"]");
        let c = Value::array(vec![Value::string("a$b")]);
        assert_eq!(c.to_string(), "[\"a$b\"]");
    }

    #[test]
    fn display_array_elements_comma_space() {
        let a = Value::array(vec![Value::Int(1), Value::string("x"), Value::Bool(true)]);
        assert_eq!(a.to_string(), "[1, \"x\", true]");
        assert_eq!(Value::array(vec![]).to_string(), "[]");
    }

    #[test]
    fn display_struct_sorts_keys_and_skips_methods() {
        let s = Value::object(vec![
            ("b".to_string(), Value::Int(2)),
            ("m".to_string(), dummy_fn()),
            ("a".to_string(), Value::Int(1)),
        ]);
        // 键按字节序升序；函数值字段（方法）不参与显示（A5/B3）。
        assert_eq!(s.to_string(), "{a: 1, b: 2}");
    }

    #[test]
    fn display_cycle_is_safe() {
        let a = Value::array(vec![Value::Int(1)]);
        if let Value::Array(rc) = &a {
            rc.borrow_mut().push(a.clone()); // 自引用环
        }
        assert_eq!(a.to_string(), "[1, <cycle>]");
    }

    #[test]
    fn display_function_and_struct_template() {
        let named = Value::function(Closure::named("inc", Rc::new(ScopeChain::empty())));
        assert_eq!(named.to_string(), "<fn inc>");
        assert_eq!(dummy_fn().to_string(), "<fn>");
        assert_eq!(Value::struct_def("Point").to_string(), "<struct Point>");
    }

    #[test]
    fn as_f64_is_the_only_widening_entry() {
        assert_eq!(Value::Int(3).as_f64(), Some(3.0));
        assert_eq!(Value::Int(-1).as_f64(), Some(-1.0));
        assert_eq!(Value::Float(2.5).as_f64(), Some(2.5));
        // 非数值不隐式转换。
        assert_eq!(Value::string("3").as_f64(), None);
        assert_eq!(Value::Bool(true).as_f64(), None);
        assert_eq!(Value::Nil.as_f64(), None);
        // `|int| = 2^53` 仍是精确加宽（§4.5.7）。
        assert_eq!(Value::Int(1 << 53).as_f64(), Some((1i64 << 53) as f64));
    }

    #[test]
    fn accessors_are_strict_and_do_not_convert() {
        assert_eq!(Value::Int(5).as_int(), Some(5));
        assert_eq!(Value::Float(5.0).as_int(), None); // 不隐式收窄
        assert_eq!(Value::Float(5.5).as_float(), Some(5.5));
        assert_eq!(Value::Int(5).as_float(), None); // as_float 严格同型；加宽用 as_f64
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::string("hi").as_str(), Some("hi"));
        assert!(Value::array(vec![]).as_array().is_some());
        assert!(Value::empty_object().as_struct().is_some());
        assert!(dummy_fn().as_closure().is_some());
        assert!(Value::Nil.as_array().is_none());
        assert!(Value::Nil.as_str().is_none());
    }

    #[test]
    fn struct_data_plane_helpers() {
        let mut s = StructObj::new();
        s.set("b", Value::Int(2));
        s.set("a", Value::Int(1));
        s.set("m", dummy_fn());
        s.set("a", Value::Int(9)); // 原地替换
        // 数据字段：跳过方法、字节序升序。
        let keys: Vec<&str> = s.data_fields_sorted().iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, vec!["a", "b"]);
        assert_eq!(s.data_len(), 2);
        // 字段读取对方法字段也可见。
        assert!(s.contains("m"));
        assert_eq!(s.get("a").map(|v| v.to_string()), Some("9".to_string()));
        assert_eq!(s.raw_fields().len(), 3);
    }
}
