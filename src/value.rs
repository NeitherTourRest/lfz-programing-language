//! 运行时值模型：`enum Value` + `Rc<RefCell<...>>` 引用语义（A1）。
//!
//! 依据（只读消费）：
//! - `docs/spec/semantics.md` §4.5.0（值模型总览）、§4.5.2（A1 引用语义）、§4.5.3（A2 闭包捕获）、
//!   §4.5.6（`float` IEEE / 排序全序）、§4.5.7（`int → float` 唯一隐式加宽 + 精确比较）、
//!   §4.5.9（A6 环安全 `==` / A5 数据面）、§4.5.11（字符串不可变）、§3.7（显示形式）。
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

use crate::ast::{Body, Expr};
use crate::env::{Cell, ScopeChain};
use std::cell::RefCell;
use std::cmp::Ordering;
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
        Value::StructDef(Rc::new(StructDef {
            name: name.into(),
            fields: Vec::new(),
            methods: Vec::new(),
        }))
    }

    /// 构造带字段默认值 / 方法的 `struct` **模板**（P3.7 求值器用于 `struct` 声明）。
    ///
    /// - `fields`：`(字段名, 默认值表达式)`，按**声明序**；实例化时逐次重新求值（§4.5.8）。
    /// - `methods`：`(方法名, 闭合的函数值)`；调用时绑定 `self`（§3.7 / A5）。
    pub fn struct_template(
        name: impl Into<Rc<str>>,
        fields: Vec<(Rc<str>, Expr)>,
        methods: Vec<(Rc<str>, Rc<Closure>)>,
    ) -> Self {
        Value::StructDef(Rc::new(StructDef {
            name: name.into(),
            fields,
            methods,
        }))
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

    // ----- 深相等（A6，§4.5.9） --------------------------------------------

    /// **深结构相等**（A6，§4.5.9）：`==` / `!=` 标量 + 容器语义的**唯一**共享实现。
    ///
    /// - **身份优先**：两侧为同一 `Rc` 分配（`Rc::ptr_eq`）→ `true`（含自引用容器，直接短路）。
    /// - **标量**：按 §4.2 / §4.5.6 / §4.5.7 比较——`NaN != NaN`、`+0.0 == -0.0`；
    ///   `int`/`float` 混合按**数学精确值**（不先加宽）；`string` 按**内容**；`function` 仅**同一性**。
    /// - **容器**（`array` / `struct`）：维护「**已访问有序对集合**」——`(a, b)` 已在集合中
    ///   ⇒ **视为相等**（环安全，**不报错、不死循环**）；否则入集后比较形状（长度 / 数据字段键集）
    ///   与逐元素 / 逐字段 `==`。
    /// - **`struct` 数据面**（A5）：比较时**忽略函数值字段**，键集按 **UTF-8 字节序升序**、**键序无关**。
    /// - 类型不同（如 `array` vs `struct`、`number` vs `string`）→ `false`。
    ///
    /// 注 1：`struct` **模板**（`StructDef`）的相等**规范未明确定义**（§4.5.9 未列该类）；
    /// 本实现保守取**同一性**（同 `Rc` 分配 → `true`，否则 `false`）。
    /// 注 2：本方法**不施加深结构上限**（`limit = u32::MAX`），仅保既有签名；需要 §4.5.5 的
    /// 10000 层上限时请用 [`Value::deep_eq_bounded`]（求值器 `==` / `!=` 走后者）。
    #[must_use]
    pub fn deep_eq(&self, other: &Value) -> bool {
        // 环安全由 `seen`（已访问有序对集合）保证，与深度无关，故 `u32::MAX` 不改变环行为。
        self.deep_eq_bounded(other, u32::MAX).unwrap_or(false)
    }

    /// **深结构相等（带上限）**（§4.5.5 / A6）：`deep_eq` 的深度受限版本。
    ///
    /// 递归处理容器时逐层递增深度；**深度超过 `limit`** → `Err(实际达到的深度)`，
    /// 调用方据此构造 `LzError::Recursion`（`semantics.md` §4.5.5：深结构处理超限 → `RecursionError`）。
    /// 未超限 → `Ok(bool)`；语义与 [`Value::deep_eq`] 完全一致（含环安全 / 数据面 / 精确比较）。
    pub fn deep_eq_bounded(&self, other: &Value, limit: u32) -> Result<bool, u32> {
        let mut seen: Vec<(usize, usize)> = Vec::new();
        eq_rec(self, other, &mut seen, 0, limit)
    }

    // ----- 全序比较（§4.5.6 / §4.5.7） --------------------------------------

    /// 本值可参与**全序比较**的**类别**（§4.5.6）：数值组（`int` / `float` 混用）或字符串组；
    /// 其余类型（`bool` / `nil` / `array` / `struct` / `function`）不可全序 → `None`。
    #[must_use]
    pub fn order_kind(&self) -> Option<OrderKind> {
        match self {
            Value::Int(_) | Value::Float(_) => Some(OrderKind::Num),
            Value::Str(_) => Some(OrderKind::Str),
            _ => None,
        }
    }

    /// **全序比较**（§4.5.6 / §4.5.7）：`sort` / `sortBy` / `min` / `max` / `minBy` / `maxBy` 的
    /// **唯一**共享实现。
    ///
    /// - **仅同类别可比较**：数值组 ↔ 数值组、字符串组 ↔ 字符串组；否则 → `None`
    ///   （调用方据 §10.7 转 `TypeError`）。
    /// - 数值：`-Inf < 任何有限值 < +Inf < NaN`（`NaN` 排最后）；此全序与 `==` 的 IEEE 语义
    ///   **并存**（`==` 仍 `NaN != NaN`）；`int`/`float` 混合按 §4.5.7 **数学精确**比较（不先加宽）。
    /// - 字符串：UTF-8 字节序（§4.2）。
    ///
    /// 注：本全序**仅**服务排序 / 极值；运算符 `< <= > >=` 的 IEEE 语义（涉及 `NaN` → `false`）
    /// **不**由本函数承担——P3.7 求值器须另行处理。
    #[must_use]
    pub fn total_cmp(&self, other: &Value) -> Option<Ordering> {
        match (self, other) {
            (Value::Int(x), Value::Int(y)) => Some(x.cmp(y)),
            (Value::Str(x), Value::Str(y)) => Some(x.as_bytes().cmp(y.as_bytes())),
            (Value::Int(_) | Value::Float(_), Value::Int(_) | Value::Float(_)) => {
                Some(num_order(self, other))
            }
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
// 相等（A6）与全序（§4.5.6）辅助
// ===========================================================================

/// 可全序比较的**类别**（§4.5.6）。见 [`Value::total_cmp`] / [`Value::order_kind`]。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrderKind {
    /// 数值组（`int` / `float` 混用，按 §4.5.7 数学精确比较）。
    Num,
    /// 字符串组（UTF-8 字节序，§4.2）。
    Str,
}

/// `2^63`：`i64` 上下界的 `f64` 表示。
///
/// `int(float)` 越界判定（`floor` / `ceil` / `round` / `int`）与 `int`/`float` 精确比较共用同一常量。
pub(crate) const TWO_POW_63: f64 = 9_223_372_036_854_775_808.0;

/// 两侧是否**同一 `Rc` 分配**（A6 步骤 3a「身份优先」）。
fn same_identity(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Str(x), Value::Str(y)) => Rc::ptr_eq(x, y),
        (Value::Array(x), Value::Array(y)) => Rc::ptr_eq(x, y),
        (Value::Struct(x), Value::Struct(y)) => Rc::ptr_eq(x, y),
        (Value::StructDef(x), Value::StructDef(y)) => Rc::ptr_eq(x, y),
        (Value::Func(x), Value::Func(y)) => Rc::ptr_eq(x, y),
        _ => false,
    }
}

/// [`Value::deep_eq_bounded`] 的递归内核；`seen` 为**已访问有序对集合**（`(a, b)` 的身份地址对）。
///
/// 重访一对 ⇒ 视为相等（**环安全**，A6 步骤 3b/3d）；容器递归逐层 `depth + 1`，
/// `depth > limit` → `Err(depth)`（§4.5.5 深结构上限）。
fn eq_rec(
    a: &Value,
    b: &Value,
    seen: &mut Vec<(usize, usize)>,
    depth: u32,
    limit: u32,
) -> Result<bool, u32> {
    if same_identity(a, b) {
        return Ok(true); // 身份优先（含自引用容器，直接短路）
    }
    match (a, b) {
        (Value::Nil, Value::Nil) => Ok(true),
        (Value::Bool(x), Value::Bool(y)) => Ok(x == y),
        (Value::Int(x), Value::Int(y)) => Ok(x == y),
        // IEEE（§4.5.6）：`NaN != NaN`、`+0.0 == -0.0`。
        (Value::Float(x), Value::Float(y)) => Ok(x == y),
        // `int`/`float` 混合按数学精确值（§4.5.7）；`NaN` → `false`。
        (Value::Int(x), Value::Float(y)) => Ok(num_eq_int_float(*x, *y)),
        (Value::Float(x), Value::Int(y)) => Ok(num_eq_int_float(*y, *x)),
        (Value::Str(x), Value::Str(y)) => Ok(x == y), // 按内容（`Rc` 不同亦可）
        (Value::Array(x), Value::Array(y)) => {
            let key = (Rc::as_ptr(x) as usize, Rc::as_ptr(y) as usize);
            if seen.contains(&key) {
                return Ok(true); // 重访 ⇒ 视为相等
            }
            if depth >= limit {
                return Err(depth + 1); // §4.5.5：深结构处理超限
            }
            seen.push(key);
            let (bx, by) = (x.borrow(), y.borrow());
            if bx.len() != by.len() {
                return Ok(false);
            }
            for (u, v) in bx.iter().zip(by.iter()) {
                if !eq_rec(u, v, seen, depth + 1, limit)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Struct(x), Value::Struct(y)) => {
            let key = (Rc::as_ptr(x) as usize, Rc::as_ptr(y) as usize);
            if seen.contains(&key) {
                return Ok(true); // 重访 ⇒ 视为相等
            }
            if depth >= limit {
                return Err(depth + 1); // §4.5.5：深结构处理超限
            }
            seen.push(key);
            let (bx, by) = (x.borrow(), y.borrow());
            let dx = bx.data_fields_sorted(); // 数据面：跳方法、键字节序升序（A5/B3）
            let dy = by.data_fields_sorted();
            if dx.len() != dy.len() {
                return Ok(false);
            }
            for ((kx, vx), (ky, vy)) in dx.iter().zip(dy.iter()) {
                if kx != ky || !eq_rec(vx, vy, seen, depth + 1, limit)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        // 类型不同（含 `array` vs `struct`）、`function` / `struct` 模板仅同一性 → 不相等。
        _ => Ok(false),
    }
}

/// `int i == float f`（§4.5.7 精确比较；`NaN` → `false`，§4.5.6）。
fn num_eq_int_float(i: i64, f: f64) -> bool {
    !f.is_nan() && cmp_int_float(i, f) == Ordering::Equal
}

/// `int i` 与 `float f`（**非 NaN**）的**数学精确**比较（§4.5.7 步骤 1–5）。
fn cmp_int_float(i: i64, f: f64) -> Ordering {
    if f == f64::INFINITY {
        return Ordering::Less; // 任何 i64 < +Inf
    }
    if f == f64::NEG_INFINITY {
        return Ordering::Greater; // 任何 i64 > -Inf
    }
    if f >= TWO_POW_63 {
        return Ordering::Less; // §4.5.7 步骤 4
    }
    if f < -TWO_POW_63 {
        return Ordering::Greater;
    }
    let t = f.trunc();
    let ti = t as i64; // |t| < 2^63，安全
    match i.cmp(&ti) {
        Ordering::Equal => {
            let r = f - t; // 精确
            if r > 0.0 {
                Ordering::Less
            } else if r < 0.0 {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }
        other => other,
    }
}

/// 数值全序（§4.5.6）：`-Inf < 有限 < +Inf < NaN`（`NaN` 排最后）。
fn num_order(a: &Value, b: &Value) -> Ordering {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::Int(x), Value::Float(y)) => {
            if y.is_nan() {
                Ordering::Less
            } else {
                cmp_int_float(*x, *y)
            }
        }
        (Value::Float(x), Value::Int(y)) => {
            if x.is_nan() {
                Ordering::Greater
            } else {
                cmp_int_float(*y, *x).reverse()
            }
        }
        (Value::Float(x), Value::Float(y)) => {
            if x.is_nan() && y.is_nan() {
                Ordering::Equal
            } else if x.is_nan() {
                Ordering::Greater
            } else if y.is_nan() {
                Ordering::Less
            } else {
                x.partial_cmp(y).expect("非 NaN 的 f64 必然可全序")
            }
        }
        _ => Ordering::Equal,
    }
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
/// P3.7 起携带**字段默认值表达式**（按声明序；实例化时重新求值，§4.5.8）与**方法表**
/// （函数值；A5 数据面**不含**，但 `s.m` 仍可取到并调用，`self` 绑定）。
#[derive(Clone, Debug)]
pub struct StructDef {
    /// 模板名（`;;` 可见绑定，§3.6）。
    pub name: Rc<str>,
    /// `(字段名, 默认值表达式)`，按声明序（实例化时求值）。
    pub fields: Vec<(Rc<str>, Expr)>,
    /// `(方法名, 函数值)`，按声明序（调用时绑定 `self`）。
    pub methods: Vec<(Rc<str>, Rc<Closure>)>,
}

/// 用户函数 / 闭包 / lambda 的**运行时载荷**（P3.7 求值器）。
///
/// 只读 AST（`params` / `body`）+ A2 **captured cell 列表** + traceback 用的 `func_id`。
#[derive(Clone, Debug)]
pub struct UserFn {
    /// 形参名，按声明序。
    pub params: Vec<String>,
    /// 函数体 AST。
    pub body: Body,
    /// A2 捕获的自由局部变量：`(名字, 共享 cell)`；`Rc` 共享以省 clone。
    pub captured: Rc<Vec<(Rc<str>, Cell)>>,
    /// traceback 帧用的函数表下标（P3.8 组装；本批仅分配不复用）。
    pub func_id: u32,
    /// 定义处捕获的 `self`（方法体内嵌套的闭包沿用）；非方法内为 `None`。
    pub self_val: Option<Value>,
}

/// 函数 / 闭包值（可调用）。
///
/// - `name` / `def_scope`：显示名与词法定义处的 [`ScopeChain`]（§10.5，供 `;;` 看到捕获名）。
/// - `user`：用户函数载荷；`None` 表示**无函数体的函数值**（仅作显示占位，P3.7 起测试用）。
#[derive(Clone, Debug)]
pub struct Closure {
    /// 显示名：命名函数 `Some("inc")` → `<fn inc>`；匿名 `None` → `<fn>`（§3.7）。
    pub name: Option<Rc<str>>,
    /// 词法定义处的 `ScopeDebug` 可见链（`Rc` 共享，§10.5）：供函数体内 `;;` 看到闭包捕获到的外层名字。
    pub def_scope: Rc<ScopeChain>,
    /// 用户函数载荷（形参 / 体 / 捕获 cell）；占位闭包为 `None`。
    pub user: Option<Rc<UserFn>>,
}

impl Closure {
    /// 命名函数 / 闭包（无函数体占位）。
    pub fn named(name: impl Into<Rc<str>>, def_scope: Rc<ScopeChain>) -> Self {
        Self {
            name: Some(name.into()),
            def_scope,
            user: None,
        }
    }

    /// 匿名函数 / λ / 闭包（无函数体占位）。
    pub fn anonymous(def_scope: Rc<ScopeChain>) -> Self {
        Self {
            name: None,
            def_scope,
            user: None,
        }
    }

    /// 带**用户函数载荷**的函数值（P3.7 求值器创建）。
    pub fn user(name: Option<Rc<str>>, def_scope: Rc<ScopeChain>, user: Rc<UserFn>) -> Self {
        Self {
            name,
            def_scope,
            user: Some(user),
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

    // ---- P3.6b：深相等（A6，§4.5.9） --------------------------------------

    /// 两个**独立的**自引用数组（结构相同）→ `==` 为 `true`，不报错、不死循环。
    #[test]
    fn deep_eq_self_referential_arrays_are_equal_and_terminate() {
        let a = Value::array(vec![Value::Int(1)]);
        if let Value::Array(rc) = &a {
            rc.borrow_mut().push(a.clone()); // a = [1, a]
        }
        let b = Value::array(vec![Value::Int(1)]);
        if let Value::Array(rc) = &b {
            rc.borrow_mut().push(b.clone()); // b = [1, b]
        }
        assert!(a.deep_eq(&b));
        assert!(b.deep_eq(&a));
        assert!(a.deep_eq(&a)); // 身份优先
    }

    /// 单元素自引用数组（纯环）：`[a] == [b]`（重访 ⇒ 相等）。
    #[test]
    fn deep_eq_pure_cycle_arrays() {
        let a = Value::array(vec![]);
        if let Value::Array(rc) = &a {
            rc.borrow_mut().push(a.clone()); // a = [a]
        }
        let b = Value::array(vec![]);
        if let Value::Array(rc) = &b {
            rc.borrow_mut().push(b.clone()); // b = [b]
        }
        assert!(a.deep_eq(&b));
    }

    /// 自引用结构体（环）：两个独立但结构相同的环 → `true`。
    #[test]
    fn deep_eq_self_referential_structs() {
        let a = Value::empty_object();
        if let Value::Struct(rc) = &a {
            rc.borrow_mut().set("self", a.clone()); // a = {self: a}
        }
        let b = Value::empty_object();
        if let Value::Struct(rc) = &b {
            rc.borrow_mut().set("self", b.clone()); // b = {self: b}
        }
        assert!(a.deep_eq(&b));
        assert!(a.deep_eq(&a));
    }

    /// 两个结构相同、**互为别名**的环（各自字段指向对方节点）：仍判相等。
    #[test]
    fn deep_eq_mutually_aliased_cycles() {
        // a = {x: b}，b = {x: a} —— 互为别名的两节点环。
        let a = Value::empty_object();
        let b = Value::empty_object();
        if let (Value::Struct(ra), Value::Struct(rb)) = (&a, &b) {
            ra.borrow_mut().set("x", b.clone());
            rb.borrow_mut().set("x", a.clone());
        }
        // c = {x: d}，d = {x: c} —— 与上环同构的另一环。
        let c = Value::empty_object();
        let d = Value::empty_object();
        if let (Value::Struct(rc), Value::Struct(rd)) = (&c, &d) {
            rc.borrow_mut().set("x", d.clone());
            rd.borrow_mut().set("x", c.clone());
        }
        assert!(a.deep_eq(&c));
        assert!(b.deep_eq(&d));
        assert!(a.deep_eq(&a)); // 身份优先
    }

    /// struct 深相等**忽略函数值字段**（A5）。
    #[test]
    fn deep_eq_struct_ignores_method_fields() {
        let s1 = Value::object(vec![
            ("a".to_string(), Value::Int(1)),
            ("m".to_string(), dummy_fn()),
        ]);
        let s2 = Value::object(vec![("a".to_string(), Value::Int(1))]);
        assert!(s1.deep_eq(&s2)); // 方法字段不参与数据面
        // 方法字段与数据字段同名但类型不同 → 数据面不同 → 不等。
        let s3 = Value::object(vec![("a".to_string(), dummy_fn())]);
        let s4 = Value::object(vec![("a".to_string(), Value::Int(1))]);
        assert!(!s3.deep_eq(&s4));
    }

    /// struct 深相等**键序无关**，键集按字节序比较（B3）。
    #[test]
    fn deep_eq_struct_key_order_independent() {
        let s1 = Value::object(vec![
            ("b".to_string(), Value::Int(2)),
            ("a".to_string(), Value::Int(1)),
        ]);
        let s2 = Value::object(vec![
            ("a".to_string(), Value::Int(1)),
            ("b".to_string(), Value::Int(2)),
        ]);
        assert!(s1.deep_eq(&s2));
        let s3 = Value::object(vec![("a".to_string(), Value::Int(1))]);
        assert!(!s1.deep_eq(&s3)); // 数据字段数不同
    }

    /// 标量深相等（§4.2 / §4.5.6 / §4.5.7）。
    #[test]
    fn deep_eq_scalars_and_exact_int_float() {
        assert!(Value::Int(1).deep_eq(&Value::Int(1)));
        assert!(!Value::Int(1).deep_eq(&Value::Int(2)));
        assert!(Value::Int(1).deep_eq(&Value::Float(1.0)));
        assert!(!Value::Int(1).deep_eq(&Value::Float(1.5)));
        // §4.5.7 精确比较：大整数不因加宽丢精度而误判相等。
        assert!(!Value::Int(9_007_199_254_740_993).deep_eq(&Value::Float(9_007_199_254_740_992.0)));
        assert!(Value::string("a").deep_eq(&Value::string("a")));
        assert!(!Value::string("a").deep_eq(&Value::string("b")));
        assert!(Value::Nil.deep_eq(&Value::Nil));
        assert!(Value::Bool(true).deep_eq(&Value::Bool(true)));
        assert!(!Value::Bool(true).deep_eq(&Value::Int(1))); // 类型不同
        // `+0.0 == -0.0`；`NaN != NaN`（§4.5.6）。
        assert!(Value::Float(0.0).deep_eq(&Value::Float(-0.0)));
        assert!(!Value::Float(f64::NAN).deep_eq(&Value::Float(f64::NAN)));
        assert!(!Value::Float(f64::NAN).deep_eq(&Value::Int(0)));
        assert!(!Value::Int(0).deep_eq(&Value::Float(f64::NAN)));
    }

    /// 函数值仅**同一性**（§4.2）。
    #[test]
    fn deep_eq_function_identity_only() {
        let f = dummy_fn();
        let g = dummy_fn();
        assert!(f.deep_eq(&f));
        assert!(!f.deep_eq(&g));
    }

    /// 类型不同（`array` vs `struct`）→ `false`。
    #[test]
    fn deep_eq_different_container_types_are_false() {
        assert!(!Value::array(vec![]).deep_eq(&Value::empty_object()));
        assert!(!Value::array(vec![Value::Int(1)]).deep_eq(&Value::empty_object()));
    }

    /// 嵌套容器深相等（无环）。
    #[test]
    fn deep_eq_nested_containers() {
        let a = Value::array(vec![
            Value::Int(1),
            Value::array(vec![Value::Int(2), Value::Int(3)]),
            Value::object(vec![("k".to_string(), Value::string("v"))]),
        ]);
        let b = a.clone(); // 身份优先
        assert!(a.deep_eq(&b));
        let c = Value::array(vec![
            Value::Int(1),
            Value::array(vec![Value::Int(2), Value::Int(4)]),
            Value::object(vec![("k".to_string(), Value::string("v"))]),
        ]);
        assert!(!a.deep_eq(&c));
    }

    // ---- P3.6b：全序（§4.5.6 / §4.5.7） ----------------------------------

    #[test]
    fn total_cmp_numeric_group() {
        use std::cmp::Ordering::{Equal, Greater, Less};
        assert_eq!(Value::Int(1).total_cmp(&Value::Int(2)), Some(Less));
        assert_eq!(Value::Int(2).total_cmp(&Value::Int(2)), Some(Equal));
        // int / float 精确比较（§4.5.7）：不先加宽。
        assert_eq!(Value::Int(1).total_cmp(&Value::Float(1.0)), Some(Equal));
        assert_eq!(Value::Int(1).total_cmp(&Value::Float(1.5)), Some(Less));
        assert_eq!(Value::Float(1.5).total_cmp(&Value::Int(1)), Some(Greater));
        assert_eq!(
            Value::Int(9_007_199_254_740_993).total_cmp(&Value::Float(9_007_199_254_740_992.0)),
            Some(Greater)
        );
        // +0.0 与 -0.0 全序相等（稳定排序依赖此性质）。
        assert_eq!(Value::Float(-0.0).total_cmp(&Value::Float(0.0)), Some(Equal));
    }

    #[test]
    fn total_cmp_infinities_and_nan_last() {
        use std::cmp::Ordering::{Equal, Greater, Less};
        // -Inf < 任何有限值 < +Inf < NaN（§4.5.6）。
        assert_eq!(
            Value::Float(f64::NEG_INFINITY).total_cmp(&Value::Int(i64::MIN)),
            Some(Less)
        );
        assert_eq!(
            Value::Int(i64::MIN).total_cmp(&Value::Float(f64::NEG_INFINITY)),
            Some(Greater)
        );
        assert_eq!(
            Value::Float(f64::INFINITY).total_cmp(&Value::Int(i64::MAX)),
            Some(Greater)
        );
        assert_eq!(
            Value::Int(i64::MAX).total_cmp(&Value::Float(f64::INFINITY)),
            Some(Less)
        );
        assert_eq!(Value::Float(1.0).total_cmp(&Value::Float(f64::NAN)), Some(Less));
        assert_eq!(Value::Float(f64::NAN).total_cmp(&Value::Float(1.0)), Some(Greater));
        assert_eq!(Value::Float(f64::NAN).total_cmp(&Value::Float(f64::NAN)), Some(Equal));
        assert_eq!(Value::Int(5).total_cmp(&Value::Float(f64::NAN)), Some(Less));
        assert_eq!(Value::Float(f64::NAN).total_cmp(&Value::Int(5)), Some(Greater));
        assert_eq!(
            Value::Float(f64::INFINITY).total_cmp(&Value::Float(f64::INFINITY)),
            Some(Equal)
        );
    }

    #[test]
    fn total_cmp_strings_by_utf8_bytes() {
        use std::cmp::Ordering::{Equal, Greater, Less};
        assert_eq!(Value::string("a").total_cmp(&Value::string("b")), Some(Less));
        assert_eq!(Value::string("b").total_cmp(&Value::string("a")), Some(Greater));
        assert_eq!(Value::string("a").total_cmp(&Value::string("a")), Some(Equal));
        // UTF-8 字节序（§4.2）：'Z'=0x5A < 'a'=0x61。
        assert_eq!(Value::string("Z").total_cmp(&Value::string("a")), Some(Less));
    }

    /// 类别不同 / 不可比较 → `None`（调用方据 §10.7 转 `TypeError`）。
    #[test]
    fn total_cmp_incomparable_categories_return_none() {
        assert_eq!(Value::Int(1).total_cmp(&Value::string("a")), None);
        assert_eq!(Value::string("a").total_cmp(&Value::Int(1)), None);
        assert_eq!(Value::Bool(true).total_cmp(&Value::Bool(false)), None);
        assert_eq!(Value::Nil.total_cmp(&Value::Nil), None);
        assert_eq!(Value::array(vec![]).total_cmp(&Value::array(vec![])), None);
        assert_eq!(Value::empty_object().total_cmp(&Value::empty_object()), None);
    }

    #[test]
    fn order_kind_classification() {
        assert_eq!(Value::Int(1).order_kind(), Some(OrderKind::Num));
        assert_eq!(Value::Float(1.0).order_kind(), Some(OrderKind::Num));
        assert_eq!(Value::string("a").order_kind(), Some(OrderKind::Str));
        assert_eq!(Value::Bool(true).order_kind(), None);
        assert_eq!(Value::Nil.order_kind(), None);
        assert_eq!(Value::array(vec![]).order_kind(), None);
        assert_eq!(Value::empty_object().order_kind(), None);
        assert_eq!(Value::struct_def("P").order_kind(), None);
    }
}
