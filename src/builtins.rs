//! 内置函数表（`data-last`，契约 §10.7 全表；本批 P3.9a 实现**非高阶**部分）。
//!
//! 单一事实源（本模块**只读消费**，不得偏离）：
//! - `docs/spec/interface-contract.md` §10.7（内置表 + 「数值内置形参加宽」规范性补钉）、§8.1（错误类 ↔ 变体）、
//!   §10.8（`R<T>` / 冷路径构造）。§10.7 约定：被操作的数据（`array`/`struct`/`string`）**恒为最后一个参数**；
//!   **凡涉及容器更新的内置一律「返回新值、不改原容器」（A1）**；参数个数 / 类型不符 → `TypeError`。
//! - `docs/spec/semantics.md` §3.7（显示）、§4.2（`/` vs `div` vs `%`）、§4.5.6（排序全序：`-Inf < 有限 < +Inf < NaN`）、
//!   §4.5.7（`int→float` 唯一加宽 + `int(float)` 极窄边界）、§4.5.10（`assert`/`check`/`fail`）、§4.5.11（字符串不可变）、
//!   §8.1（错误消息）。
//!
//! # 调用约定（稳定 ABI，供 P3.7 求值器调用）
//!
//! ```text
//! pub type BuiltinFn = fn(&[Value], Span) -> R<Value>;
//! pub fn lookup(name: &str) -> Option<Builtin>;   // 未实现的名字（含 7 个高阶内置）→ None
//! pub fn call(name: &str, args: &[Value], span: Span) -> R<Value>;
//! pub const BUILTIN_NAMES: &[&str];               // 本批全部 47 个名字（字母序）
//! ```
//!
//! - **位置（位置红线）**：`Span` 为**调用点**位置，由求值器在脱糖后的 `Call` 节点处传入；每个内置的
//!   所有 `LfzError` 均携带该 `Span`（`print`/`eprint`/`input` 的 `IOError` 亦然）。
//!   （契约任务书「建议」`fn(&[Value]) -> R<Value>`；本实现**扩展为携带 `Span`**，否则无法满足
//!   「运行时错误必须带位置」红线——见收工汇报「决策」。）
//! - **参数个数**：`Builtin::call` 集中校验 `[min_args, max_args]`，不符 → `TypeError::ArgCount`
//!   （省略型如 `input(prompt?)` / `assert(cond, msg?)` 的 `n` 取越界侧边界）。
//! - **参数类型**：不符 → `TypeError::BadOperands`（§8.1 唯一的通用「类型不符」消息模板；
//!   以 `op = 内置名`、`lt = 实参类型`、`rt = 期望类型` 填槽）。
//!
//! # 本批范围
//!
//! 实现 §10.7 中**全部非高阶**内置（**47 个**）。以下 **7 个高阶内置**（须调用用户函数）**留待 P3.9b**
//! （求值器 P3.7/P3.8 就绪后落地）：
//!
//! `map` `filter` `reduce` `sortBy` `minBy` `maxBy` `each`
//!
//! （`sort` 本身**非**高阶，已在本批实现。）
//
// TODO(P3.9b): map / filter / reduce / sortBy / minBy / maxBy / each —— 需要 `call_user_fn` 能力与
//               `Env`/闭包调用帧（P3.7/P3.8）。签名与语义见 §10.7；`filter` 谓词须返回 `bool` 否则
//               `TypeError`；`sortBy` 稳定、按 `keyFn` 全序（§4.5.6）。

use crate::error::{
    assert_fail, div_zero, field as field_error, index as index_error, io as io_error, name as name_error,
    overflow, type_error, value as value_error, LzError, R, TypeMsg, ValueMsg, OverflowMsg,
};
use crate::span::Span;
use crate::value::{StructObj, Value};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::io::{BufRead, Write};
use std::rc::Rc;

// ===========================================================================
// 稳定 ABI / 函数表
// ===========================================================================

/// 内置函数统一 ABI：`fn(实参, 调用点 Span) -> R<Value>`。
pub type BuiltinFn = fn(&[Value], Span) -> R<Value>;

/// 一个内置函数的注册条目。
#[derive(Clone, Copy)]
pub struct Builtin {
    /// `§10.7` 中的内置名（`data-last`）。
    pub name: &'static str,
    /// 最少实参数。
    pub min_args: usize,
    /// 最多实参数。
    pub max_args: usize,
    /// 实现；`func` 私有，外部只能经 [`Builtin::call`] 调用（统一做参数个数校验）。
    func: BuiltinFn,
}

impl Builtin {
    /// 校验参数个数后调用本内置。
    ///
    /// 个数不符 → `TypeError::ArgCount`：
    /// - `m < min_args` → `n = min_args`（「期待 `min` 个，得到 `m`」）；
    /// - `m > max_args` → `n = max_args`（可选参数 / 变参越界时给出上界）。
    #[must_use]
    pub fn call(self, args: &[Value], span: Span) -> R<Value> {
        let m = args.len();
        if m < self.min_args || m > self.max_args {
            let n = if m < self.min_args {
                self.min_args
            } else {
                self.max_args
            };
            return Err(type_error(
                TypeMsg::ArgCount {
                    name: self.name.to_string(),
                    n,
                    m,
                },
                span,
            ));
        }
        (self.func)(args, span)
    }
}

/// 本批实现的内置名（字母序，与 [`TABLE`] **逐项对应**）。
pub const BUILTIN_NAMES: &[&str] = &[
    "abs", "assert", "ceil", "check", "del", "div", "drop", "entries", "eprint", "fail", "float",
    "floor", "has", "input", "insert", "int", "join", "keys", "len", "lower", "max", "min", "pop",
    "pow", "print", "push", "rand", "randInt", "range", "removeAt", "repeat", "replace", "round",
    "seed", "slice", "sort", "split", "sqrt", "startsWith", "str", "sum", "swap", "take", "trim",
    "type", "upper", "values",
];

/// 变参内置的「无上界」哨兵。
const VARIADIC: usize = usize::MAX;

/// 内置函数表（字母序，与 [`BUILTIN_NAMES`] 逐项对应；由测试锁定一致性）。
static TABLE: [Builtin; 47] = [
    Builtin { name: "abs", min_args: 1, max_args: 1, func: b_abs },
    Builtin { name: "assert", min_args: 1, max_args: 2, func: b_assert },
    Builtin { name: "ceil", min_args: 1, max_args: 1, func: b_ceil },
    Builtin { name: "check", min_args: 1, max_args: 2, func: b_check },
    Builtin { name: "del", min_args: 2, max_args: 2, func: b_del },
    Builtin { name: "div", min_args: 2, max_args: 2, func: b_div },
    Builtin { name: "drop", min_args: 2, max_args: 2, func: b_drop },
    Builtin { name: "entries", min_args: 1, max_args: 1, func: b_entries },
    Builtin { name: "eprint", min_args: 0, max_args: VARIADIC, func: b_eprint },
    Builtin { name: "fail", min_args: 0, max_args: 1, func: b_fail },
    Builtin { name: "float", min_args: 1, max_args: 1, func: b_float },
    Builtin { name: "floor", min_args: 1, max_args: 1, func: b_floor },
    Builtin { name: "has", min_args: 2, max_args: 2, func: b_has },
    Builtin { name: "input", min_args: 0, max_args: 1, func: b_input },
    Builtin { name: "insert", min_args: 3, max_args: 3, func: b_insert },
    Builtin { name: "int", min_args: 1, max_args: 1, func: b_int },
    Builtin { name: "join", min_args: 2, max_args: 2, func: b_join },
    Builtin { name: "keys", min_args: 1, max_args: 1, func: b_keys },
    Builtin { name: "len", min_args: 1, max_args: 1, func: b_len },
    Builtin { name: "lower", min_args: 1, max_args: 1, func: b_lower },
    Builtin { name: "max", min_args: 1, max_args: 1, func: b_max },
    Builtin { name: "min", min_args: 1, max_args: 1, func: b_min },
    Builtin { name: "pop", min_args: 1, max_args: 1, func: b_pop },
    Builtin { name: "pow", min_args: 2, max_args: 2, func: b_pow },
    Builtin { name: "print", min_args: 0, max_args: VARIADIC, func: b_print },
    Builtin { name: "push", min_args: 2, max_args: 2, func: b_push },
    Builtin { name: "rand", min_args: 0, max_args: 0, func: b_rand },
    Builtin { name: "randInt", min_args: 2, max_args: 2, func: b_rand_int },
    Builtin { name: "range", min_args: 1, max_args: 1, func: b_range },
    Builtin { name: "removeAt", min_args: 2, max_args: 2, func: b_remove_at },
    Builtin { name: "repeat", min_args: 2, max_args: 2, func: b_repeat },
    Builtin { name: "replace", min_args: 3, max_args: 3, func: b_replace },
    Builtin { name: "round", min_args: 1, max_args: 1, func: b_round },
    Builtin { name: "seed", min_args: 1, max_args: 1, func: b_seed },
    Builtin { name: "slice", min_args: 3, max_args: 3, func: b_slice },
    Builtin { name: "sort", min_args: 1, max_args: 1, func: b_sort },
    Builtin { name: "split", min_args: 2, max_args: 2, func: b_split },
    Builtin { name: "sqrt", min_args: 1, max_args: 1, func: b_sqrt },
    Builtin { name: "startsWith", min_args: 2, max_args: 2, func: b_starts_with },
    Builtin { name: "str", min_args: 1, max_args: 1, func: b_str },
    Builtin { name: "sum", min_args: 1, max_args: 1, func: b_sum },
    Builtin { name: "swap", min_args: 3, max_args: 3, func: b_swap },
    Builtin { name: "take", min_args: 2, max_args: 2, func: b_take },
    Builtin { name: "trim", min_args: 1, max_args: 1, func: b_trim },
    Builtin { name: "type", min_args: 1, max_args: 1, func: b_type },
    Builtin { name: "upper", min_args: 1, max_args: 1, func: b_upper },
    Builtin { name: "values", min_args: 1, max_args: 1, func: b_values },
];

/// 按名字查内置（线性扫描，47 项；字母序便于短路）。未实现 / 非内置名字（含 7 个高阶内置）→ `None`。
#[must_use]
pub fn lookup(name: &str) -> Option<Builtin> {
    TABLE.iter().copied().find(|b| b.name == name)
}

/// 是否为已实现的内置。
#[must_use]
pub fn is_builtin(name: &str) -> bool {
    lookup(name).is_some()
}

/// 按名字调用内置（供 P3.7 求值器）。
///
/// - 名字非内置 → `NameError`（求值器据此在全局找不到名字时统一报未定义）；
/// - 已找到 → 交 [`Builtin::call`]（含参数个数校验）。
pub fn call(name: &str, args: &[Value], span: Span) -> R<Value> {
    match lookup(name) {
        Some(b) => b.call(args, span),
        None => Err(name_error(name.to_string(), span)),
    }
}

// ===========================================================================
// 参数取值 / 错误构造辅助
// ===========================================================================

/// 取第 `i` 个实参（`&Value`）。缺参 → `TypeError::ArgCount`（防御性；正常路径已被 `Builtin::call` 拦截）。
fn at<'a>(name: &str, args: &'a [Value], i: usize, span: Span) -> R<&'a Value> {
    args.get(i).ok_or_else(|| {
        type_error(
            TypeMsg::ArgCount {
                name: name.to_string(),
                n: i + 1,
                m: args.len(),
            },
            span,
        )
    })
}

/// 类型不符的 `TypeError`（`op` = 内置名，`lt` = 实参类型，`rt` = 期望类型）。
fn bad_operands(name: &str, got: &Value, expected: &str, span: Span) -> Box<LzError> {
    type_error(
        TypeMsg::BadOperands {
            op: name.to_string(),
            lt: got.type_name().to_string(),
            rt: expected.to_string(),
        },
        span,
    )
}

/// 第 `i` 个实参须为 `int`。
fn need_int(name: &str, args: &[Value], i: usize, span: Span) -> R<i64> {
    let v = at(name, args, i, span)?;
    v.as_int().ok_or_else(|| bad_operands(name, v, "int", span))
}

/// 第 `i` 个实参须为 `int` / `float`，经 [`Value::as_f64`]（§4.5.7 唯一加宽入口）取 `f64`。
fn need_number(name: &str, args: &[Value], i: usize, span: Span) -> R<f64> {
    let v = at(name, args, i, span)?;
    v.as_f64()
        .ok_or_else(|| bad_operands(name, v, "number（int / float）", span))
}

/// 第 `i` 个实参须为 `array`，返回其共享容器（引用语义）。
fn need_array(name: &str, args: &[Value], i: usize, span: Span) -> R<Rc<RefCell<Vec<Value>>>> {
    let v = at(name, args, i, span)?;
    match v {
        Value::Array(a) => Ok(Rc::clone(a)),
        _ => Err(bad_operands(name, v, "array", span)),
    }
}

/// 第 `i` 个实参须为 `struct` 实例，返回其共享容器。
fn need_struct(name: &str, args: &[Value], i: usize, span: Span) -> R<Rc<RefCell<StructObj>>> {
    let v = at(name, args, i, span)?;
    match v {
        Value::Struct(s) => Ok(Rc::clone(s)),
        _ => Err(bad_operands(name, v, "struct", span)),
    }
}

/// 第 `i` 个实参须为 `string`。
fn need_str<'a>(name: &str, args: &'a [Value], i: usize, span: Span) -> R<&'a str> {
    let v = at(name, args, i, span)?;
    v.as_str().ok_or_else(|| bad_operands(name, v, "string", span))
}

/// 第 `i` 个实参须为 `bool`（`assert` / `check` 的条件）。
///
/// 非 `bool` → `TypeError::ConditionNotBool`（§8.1「条件必须是 bool，得到 {t}」——条件类实参的
/// 专用消息，比通用 `BadOperands` 更贴合规范）。
fn need_bool(name: &str, args: &[Value], i: usize, span: Span) -> R<bool> {
    let v = at(name, args, i, span)?;
    v.as_bool().ok_or_else(|| {
        type_error(
            TypeMsg::ConditionNotBool {
                t: v.type_name().to_string(),
            },
            span,
        )
    })
}

/// 取 `array` 实参的**元素克隆**（浅拷贝；嵌套容器仍共享 `Rc`）——所有「返回新数组」的内置从此起步，
/// 从而**绝不修改原容器**（A1）。
fn array_snapshot(name: &str, args: &[Value], i: usize, span: Span) -> R<Vec<Value>> {
    let xs = need_array(name, args, i, span)?;
    let snapshot = xs.borrow().to_vec();
    Ok(snapshot)
}

/// 规范化数组下标（支持负索引）；越界 → `IndexError`（`idx` = **用户传入的原始下标**）。
fn normalize_index(i: i64, len: usize, span: Span) -> R<usize> {
    let n = len as i64;
    let norm = if i < 0 { i + n } else { i };
    if norm < 0 || norm >= n {
        Err(index_error(i, len, span))
    } else {
        Ok(norm as usize)
    }
}

// ===========================================================================
// 数值辅助（§4.5.6 全序 / §4.5.7 加宽与精确比较）
// ===========================================================================

/// `2^63`：`i64` 上下界的 `f64` 表示（用于 `int(float)` / `floor` / `ceil` / `round` 的越界判定）。
const TWO_POW_63: f64 = 9_223_372_036_854_775_808.0;

/// **唯一** `int → float` 加宽（委托 [`Value::as_f64`]，§4.5.7）——禁止在别处散落 `as f64` 转换 `Value`。
fn int_to_float(i: i64) -> f64 {
    Value::Int(i)
        .as_f64()
        .expect("Value::Int 的 as_f64 恒为 Some")
}

/// `float → i64` 的显式转换（§4.5.7 极窄边界；`int` / `floor` / `ceil` / `round` 共用）：
/// `NaN` → `ValueError`；`±Inf` → `OverflowError`；有限值**向零截断**，截断后超出 `i64` → `OverflowError`。
fn float_to_int(x: f64, span: Span) -> R<i64> {
    if x.is_nan() {
        return Err(value_error(
            ValueMsg::Convert {
                src: "float".to_string(),
                dst: "int".to_string(),
                text: "nan".to_string(),
            },
            span,
        ));
    }
    if x.is_infinite() {
        return Err(overflow(span, OverflowMsg::IntegerOutOfRange));
    }
    let t = x.trunc();
    if t >= TWO_POW_63 || t < -TWO_POW_63 {
        return Err(overflow(span, OverflowMsg::IntegerOutOfRange));
    }
    Ok(t as i64)
}

/// 银行家舍入（四舍六入五成双；Python `round`）——先算 `floor`，再按小数部分与偶奇决定进位。
fn banker_round(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let fl = x.floor();
    let diff = x - fl;
    if diff > 0.5 {
        fl + 1.0
    } else if diff < 0.5 {
        fl
    } else if fl % 2.0 == 0.0 {
        fl
    } else {
        fl + 1.0
    }
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

/// 数值全序（§4.5.6）：`-Inf < 有限 < +Inf < NaN`（NaN 排最后）。
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
        _ => Ordering::Equal, // 已由 validate_orderable 排除
    }
}

/// 可排序元素类别：数值（`int` / `float` 混用）或 `string`。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Num,
    Str,
}

fn order_kind(v: &Value) -> Option<Kind> {
    match v {
        Value::Int(_) | Value::Float(_) => Some(Kind::Num),
        Value::Str(_) => Some(Kind::Str),
        _ => None,
    }
}

/// 校验 `items` 可全序比较（§4.5.6）：须全部为数值或全部为 `string`，否则 `TypeError`。
fn validate_orderable(items: &[Value], name: &str, span: Span) -> R<()> {
    let Some(first) = items.first() else {
        return Ok(());
    };
    let kind = order_kind(first).ok_or_else(|| bad_operands(name, first, "number / string", span))?;
    for v in items {
        if order_kind(v) != Some(kind) {
            let expected = match kind {
                Kind::Num => "number",
                Kind::Str => "string",
            };
            return Err(bad_operands(name, v, expected, span));
        }
    }
    Ok(())
}

/// 已通过 [`validate_orderable`] 的全序比较（不会失败）：数值走 §4.5.6/§4.5.7，字符串走 UTF-8 字节序（§4.2）。
fn total_cmp_ok(a: &Value, b: &Value) -> Ordering {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::Str(x), Value::Str(y)) => x.as_bytes().cmp(y.as_bytes()),
        _ => num_order(a, b),
    }
}

// ===========================================================================
// 核心 / 数组
// ===========================================================================

/// `len(x) -> int`：array 元素数；struct **数据字段**数（不含方法，A5）；string 的 Unicode 标量数。
fn b_len(args: &[Value], span: Span) -> R<Value> {
    let v = at("len", args, 0, span)?;
    match v {
        Value::Array(a) => Ok(Value::Int(a.borrow().len() as i64)),
        Value::Struct(s) => Ok(Value::Int(s.borrow().data_len() as i64)),
        Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
        _ => Err(bad_operands("len", v, "array / struct / string", span)),
    }
}

/// `range(n) -> array[int]`：`[0, 1, …, n-1]`；`n < 0` → 空数组。
fn b_range(args: &[Value], span: Span) -> R<Value> {
    let n = need_int("range", args, 0, span)?;
    if n <= 0 {
        return Ok(Value::array(Vec::new()));
    }
    let items: Vec<Value> = (0..n).map(Value::Int).collect();
    Ok(Value::array(items))
}

/// `push(v, xs) -> array`：追加 `v` 的**新**数组（不改 `xs`，A1）。
fn b_push(args: &[Value], span: Span) -> R<Value> {
    let v = args[0].clone();
    let mut items = array_snapshot("push", args, 1, span)?;
    items.push(v);
    Ok(Value::array(items))
}

/// `pop(xs) -> array`：去掉末元素的**新**数组；空 → `IndexError`。
fn b_pop(args: &[Value], span: Span) -> R<Value> {
    let mut items = array_snapshot("pop", args, 0, span)?;
    if items.is_empty() {
        return Err(index_error(-1, 0, span));
    }
    items.pop();
    Ok(Value::array(items))
}

/// `removeAt(i, xs) -> array`：去掉下标 `i`（支持负索引）的**新**数组；越界 → `IndexError`。
fn b_remove_at(args: &[Value], span: Span) -> R<Value> {
    let i = need_int("removeAt", args, 0, span)?;
    let mut items = array_snapshot("removeAt", args, 1, span)?;
    let idx = normalize_index(i, items.len(), span)?;
    items.remove(idx);
    Ok(Value::array(items))
}

/// `insert(i, v, xs) -> array`：在 `i` 处插入 `v` 的**新**数组；`i ∈ [0, len]`，否则 `IndexError`。
fn b_insert(args: &[Value], span: Span) -> R<Value> {
    let i = need_int("insert", args, 0, span)?;
    let v = args[1].clone();
    let mut items = array_snapshot("insert", args, 2, span)?;
    let n = items.len() as i64;
    if i < 0 || i > n {
        return Err(index_error(i, items.len(), span));
    }
    items.insert(i as usize, v);
    Ok(Value::array(items))
}

/// `swap(i, j, xs) -> array`：交换 `i` / `j`（支持负索引）的**新**数组；越界 → `IndexError`。
fn b_swap(args: &[Value], span: Span) -> R<Value> {
    let i = need_int("swap", args, 0, span)?;
    let j = need_int("swap", args, 1, span)?;
    let mut items = array_snapshot("swap", args, 2, span)?;
    let a = normalize_index(i, items.len(), span)?;
    let b = normalize_index(j, items.len(), span)?;
    items.swap(a, b);
    Ok(Value::array(items))
}

/// `slice(from, to, xs) -> array`：`xs[from..to]` 的**新**数组；下标夹取到 `[0, len]`，`from >= to` → 空。
fn b_slice(args: &[Value], span: Span) -> R<Value> {
    let from = need_int("slice", args, 0, span)?;
    let to = need_int("slice", args, 1, span)?;
    let items = array_snapshot("slice", args, 2, span)?;
    let len = items.len() as i64;
    let f = from.clamp(0, len) as usize;
    let t = to.clamp(0, len) as usize;
    let out = if f >= t { Vec::new() } else { items[f..t].to_vec() };
    Ok(Value::array(out))
}

/// `min` / `max` 共用：空 → `ValueError`；否则按 §4.5.6 全序取极值。
fn extremum(args: &[Value], span: Span, want_max: bool, name: &str) -> R<Value> {
    let items = array_snapshot(name, args, 0, span)?;
    let Some(first) = items.first() else {
        return Err(empty_collection_value_error(name, span));
    };
    validate_orderable(&items, name, span)?;
    let mut best = first;
    for v in &items[1..] {
        let ord = total_cmp_ok(v, best);
        let better = if want_max {
            ord == Ordering::Greater
        } else {
            ord == Ordering::Less
        };
        if better {
            best = v;
        }
    }
    Ok(Value::clone(best))
}

/// `min(xs) -> value`：全序（§4.5.6）；空 → `ValueError`。
fn b_min(args: &[Value], span: Span) -> R<Value> {
    extremum(args, span, false, "min")
}

/// `max(xs) -> value`：全序（§4.5.6）；空 → `ValueError`。
fn b_max(args: &[Value], span: Span) -> R<Value> {
    extremum(args, span, true, "max")
}

/// `sum(xs) -> number`：全 `int` → `int`（溢出 → `OverflowError`）；含 `float` → `float`；空 → `int 0`。
fn b_sum(args: &[Value], span: Span) -> R<Value> {
    let items = array_snapshot("sum", args, 0, span)?;
    let mut acc_int: i64 = 0;
    let mut acc_float: f64 = 0.0;
    let mut has_float = false;
    for v in &items {
        match v {
            Value::Int(i) => {
                if has_float {
                    acc_float += int_to_float(*i);
                } else {
                    acc_int = acc_int
                        .checked_add(*i)
                        .ok_or_else(|| overflow(span, OverflowMsg::IntegerOutOfRange))?;
                }
            }
            Value::Float(x) => {
                if !has_float {
                    acc_float = int_to_float(acc_int);
                    has_float = true;
                }
                acc_float += x;
            }
            _ => return Err(bad_operands("sum", v, "number", span)),
        }
    }
    Ok(if has_float {
        Value::Float(acc_float)
    } else {
        Value::Int(acc_int)
    })
}

/// `sort(xs) -> array`：升序**新**数组（**稳定**）；全序（§4.5.6）；元素类型须一致可比，否则 `TypeError`。
fn b_sort(args: &[Value], span: Span) -> R<Value> {
    let mut items = array_snapshot("sort", args, 0, span)?;
    validate_orderable(&items, "sort", span)?;
    items.sort_by(total_cmp_ok); // `slice::sort_by` 稳定
    Ok(Value::array(items))
}

/// `take(n, xs) -> array`：前 `n` 个（`n` 夹取到 `[0, len]`）的**新**数组。
fn b_take(args: &[Value], span: Span) -> R<Value> {
    let n = need_int("take", args, 0, span)?;
    let items = array_snapshot("take", args, 1, span)?;
    let k = n.clamp(0, items.len() as i64) as usize;
    Ok(Value::array(items[..k].to_vec()))
}

/// `drop(n, xs) -> array`：去前 `n` 个（`n` 夹取到 `[0, len]`）的**新**数组。
fn b_drop(args: &[Value], span: Span) -> R<Value> {
    let n = need_int("drop", args, 0, span)?;
    let items = array_snapshot("drop", args, 1, span)?;
    let k = n.clamp(0, items.len() as i64) as usize;
    Ok(Value::array(items[k..].to_vec()))
}

// ===========================================================================
// struct / 字典
// ===========================================================================

/// `keys(s) -> array[string]`：**数据字段**键，**字节序升序**（A5/B3）。
fn b_keys(args: &[Value], span: Span) -> R<Value> {
    let s = need_struct("keys", args, 0, span)?;
    let keys: Vec<Value> = s
        .borrow()
        .data_fields_sorted()
        .iter()
        .map(|&(k, _)| Value::string(k))
        .collect();
    Ok(Value::array(keys))
}

/// `values(s) -> array`：与 `keys` 同序（A5）。
fn b_values(args: &[Value], span: Span) -> R<Value> {
    let s = need_struct("values", args, 0, span)?;
    let values: Vec<Value> = s
        .borrow()
        .data_fields_sorted()
        .iter()
        .map(|&(_, v)| Value::clone(v))
        .collect();
    Ok(Value::array(values))
}

/// `entries(s) -> array[[k,v]]`：元素为 `[key, value]` 二元数组，按 `keys` 序。
fn b_entries(args: &[Value], span: Span) -> R<Value> {
    let s = need_struct("entries", args, 0, span)?;
    let entries: Vec<Value> = s
        .borrow()
        .data_fields_sorted()
        .iter()
        .map(|&(k, v)| Value::array(vec![Value::string(k), Value::clone(v)]))
        .collect();
    Ok(Value::array(entries))
}

/// `has(k, s) -> bool`：仅**数据字段**；方法 → `false`（A5）。
fn b_has(args: &[Value], span: Span) -> R<Value> {
    let k = need_str("has", args, 0, span)?;
    let s = need_struct("has", args, 1, span)?;
    let present = s
        .borrow()
        .raw_fields()
        .iter()
        .any(|(key, v)| key == k && !matches!(v, Value::Func(_)));
    Ok(Value::Bool(present))
}

/// `del(k, s) -> struct`：去掉 `k` 的**新** struct；缺失 → `FieldError`。
fn b_del(args: &[Value], span: Span) -> R<Value> {
    let k = need_str("del", args, 0, span)?;
    let s = need_struct("del", args, 1, span)?;
    let raw = s.borrow().raw_fields().to_vec();
    if !raw.iter().any(|(key, _)| key == k) {
        return Err(field_error(k.to_string(), span));
    }
    let fields: Vec<(String, Value)> = raw.into_iter().filter(|(key, _)| key != k).collect();
    Ok(Value::Struct(Rc::new(RefCell::new(StructObj::from_fields(fields)))))
}

// ===========================================================================
// 字符串
// ===========================================================================

/// `split(sep, s) -> array[string]`：`sep` 为空串 → 按字符切分。
fn b_split(args: &[Value], span: Span) -> R<Value> {
    let sep = need_str("split", args, 0, span)?;
    let s = need_str("split", args, 1, span)?;
    let parts: Vec<Value> = if sep.is_empty() {
        s.chars().map(|c| Value::string(c.to_string())).collect()
    } else {
        s.split(sep).map(Value::string).collect()
    };
    Ok(Value::array(parts))
}

/// `join(sep, xs) -> string`：元素须为 string，否则 `TypeError`。
fn b_join(args: &[Value], span: Span) -> R<Value> {
    let sep = need_str("join", args, 0, span)?;
    let xs = need_array("join", args, 1, span)?;
    let items = xs.borrow().to_vec();
    let mut strs: Vec<&str> = Vec::with_capacity(items.len());
    for v in &items {
        match v {
            Value::Str(s) => strs.push(s.as_str()),
            _ => return Err(bad_operands("join", v, "string", span)),
        }
    }
    Ok(Value::string(strs.join(sep)))
}

/// `trim(s) -> string`：去首尾空白。
fn b_trim(args: &[Value], span: Span) -> R<Value> {
    let s = need_str("trim", args, 0, span)?;
    Ok(Value::string(s.trim()))
}

/// `upper(s) -> string`：ASCII 大写（Unicode 折叠见 v1.1）。
fn b_upper(args: &[Value], span: Span) -> R<Value> {
    let s = need_str("upper", args, 0, span)?;
    Ok(Value::string(s.to_ascii_uppercase()))
}

/// `lower(s) -> string`：ASCII 小写（Unicode 折叠见 v1.1）。
fn b_lower(args: &[Value], span: Span) -> R<Value> {
    let s = need_str("lower", args, 0, span)?;
    Ok(Value::string(s.to_ascii_lowercase()))
}

/// `replace(old, new, s) -> string`：全部替换。
fn b_replace(args: &[Value], span: Span) -> R<Value> {
    let old = need_str("replace", args, 0, span)?;
    let new = need_str("replace", args, 1, span)?;
    let s = need_str("replace", args, 2, span)?;
    Ok(Value::string(s.replace(old, new)))
}

/// `repeat(n, s) -> string`：`n <= 0` → 空串；长度溢出 → `OverflowError`。
fn b_repeat(args: &[Value], span: Span) -> R<Value> {
    let n = need_int("repeat", args, 0, span)?;
    let s = need_str("repeat", args, 1, span)?;
    if n <= 0 {
        return Ok(Value::string(""));
    }
    s.len()
        .checked_mul(n as usize)
        .ok_or_else(|| overflow(span, OverflowMsg::IntegerOutOfRange))?;
    Ok(Value::string(s.repeat(n as usize)))
}

/// `startsWith(prefix, s) -> bool`。
fn b_starts_with(args: &[Value], span: Span) -> R<Value> {
    let prefix = need_str("startsWith", args, 0, span)?;
    let s = need_str("startsWith", args, 1, span)?;
    Ok(Value::Bool(s.starts_with(prefix)))
}

// ===========================================================================
// 数学
// ===========================================================================

/// `abs(x) -> number`：**参数与返回同型、绝不加宽**；`int` 的 `i64::MIN` → `OverflowError`。
fn b_abs(args: &[Value], span: Span) -> R<Value> {
    let v = at("abs", args, 0, span)?;
    match v {
        Value::Int(i) => i
            .checked_abs()
            .map(Value::Int)
            .ok_or_else(|| overflow(span, OverflowMsg::IntegerOutOfRange)),
        Value::Float(x) => Ok(Value::Float(x.abs())),
        _ => Err(bad_operands("abs", v, "number", span)),
    }
}

/// `floor(f) -> int`：向下取整；`int` 实参按 [`Value::as_f64`] 加宽（§10.7）。
fn b_floor(args: &[Value], span: Span) -> R<Value> {
    let x = need_number("floor", args, 0, span)?;
    Ok(Value::Int(float_to_int(x.floor(), span)?))
}

/// `ceil(f) -> int`：向上取整；`int` 实参加宽（§10.7）。
fn b_ceil(args: &[Value], span: Span) -> R<Value> {
    let x = need_number("ceil", args, 0, span)?;
    Ok(Value::Int(float_to_int(x.ceil(), span)?))
}

/// `round(f) -> int`：**四舍六入五成双**（Python banker's rounding）；`int` 实参加宽（§10.7）。
fn b_round(args: &[Value], span: Span) -> R<Value> {
    let x = need_number("round", args, 0, span)?;
    Ok(Value::Int(float_to_int(banker_round(x), span)?))
}

/// `sqrt(f) -> float`：`f < 0` → `NaN`；`int` 实参加宽（§10.7）。
fn b_sqrt(args: &[Value], span: Span) -> R<Value> {
    let x = need_number("sqrt", args, 0, span)?;
    Ok(Value::Float(x.sqrt()))
}

/// `pow(a, b) -> float`：数值幂；溢出 → `±Inf`；`int` 实参加宽（§10.7）。
fn b_pow(args: &[Value], span: Span) -> R<Value> {
    let a = need_number("pow", args, 0, span)?;
    let b = need_number("pow", args, 1, span)?;
    Ok(Value::Float(a.powf(b)))
}

/// `div(a, b) -> int`：两参须 `int`；**向下取整**；`b == 0` → `ZeroDivisionError`（§4.2）。
fn b_div(args: &[Value], span: Span) -> R<Value> {
    let a = need_int("div", args, 0, span)?;
    let b = need_int("div", args, 1, span)?;
    if b == 0 {
        return Err(div_zero(false, span));
    }
    let q = a
        .checked_div(b)
        .ok_or_else(|| overflow(span, OverflowMsg::IntegerOutOfRange))?;
    let r = a % b;
    let floor_q = if r != 0 && ((r < 0) != (b < 0)) { q - 1 } else { q };
    Ok(Value::Int(floor_q))
}

// ===========================================================================
// 随机（自实现 PRNG，无第三方依赖）
// ===========================================================================

thread_local! {
    /// 全局随机状态（`splitmix64`，64 位）。默认以时间播种；`seed(n)` 可固定（§10.7）。
    static RNG_STATE: Cell<u64> = Cell::new(seed_from_time());
}

/// 时间种子（唯一非确定源，§10.7 `seed` 行）。
fn seed_from_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x2545_F491_4F6C_DD1D)
}

/// `splitmix64`：一次状态推进 + 输出混合（无外部 crate）。
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// 取下一个 64 位随机数。
fn next_u64() -> u64 {
    RNG_STATE.with(|s| {
        let mut x = s.get();
        let r = splitmix64(&mut x);
        s.set(x);
        r
    })
}

/// `rand() -> float`：`[0.0, 1.0)`。
fn b_rand(_args: &[Value], _span: Span) -> R<Value> {
    let r = next_u64() >> 11; // 高 53 位
    Ok(Value::Float(r as f64 * (1.0 / 9_007_199_254_740_992.0))) // / 2^53
}

/// `randInt(lo, hi) -> int`：`[lo, hi)`；`lo >= hi` → `ValueError`。
fn b_rand_int(args: &[Value], span: Span) -> R<Value> {
    let lo = need_int("randInt", args, 0, span)?;
    let hi = need_int("randInt", args, 1, span)?;
    if lo >= hi {
        return Err(value_error(
            ValueMsg::Convert {
                src: "int".to_string(),
                dst: "int".to_string(),
                text: format!("{lo} >= {hi}"),
            },
            span,
        ));
    }
    let width = (hi as i128 - lo as i128) as u128; // > 0
    let r = (next_u64() as u128) % width;
    Ok(Value::Int((lo as i128 + r as i128) as i64))
}

/// `seed(n) -> nil`：固定随机序列（唯一非确定源）。
fn b_seed(args: &[Value], span: Span) -> R<Value> {
    let n = need_int("seed", args, 0, span)?;
    RNG_STATE.with(|s| s.set(n as u64));
    Ok(Value::Nil)
}

// ===========================================================================
// 转换 / 类型
// ===========================================================================

/// `str(x) -> string`：显示形式（§3.7，顶层字符串裸输出）。
fn b_str(args: &[Value], span: Span) -> R<Value> {
    let v = at("str", args, 0, span)?;
    Ok(Value::string(v.to_string()))
}

/// `int(x) -> int`：`string`/`float`/`bool` → `int`；`float` **向零截断**（§4.5.7 边界）；
/// 非法串 → `ValueError`。
fn b_int(args: &[Value], span: Span) -> R<Value> {
    let v = at("int", args, 0, span)?;
    match v {
        Value::Int(i) => Ok(Value::Int(*i)),
        Value::Float(f) => Ok(Value::Int(float_to_int(*f, span)?)),
        Value::Bool(b) => Ok(Value::Int(i64::from(*b))),
        Value::Str(s) => match s.trim().parse::<i64>() {
            Ok(parsed) => Ok(Value::Int(parsed)),
            Err(_) => Err(value_error(
                ValueMsg::Convert {
                    src: "string".to_string(),
                    dst: "int".to_string(),
                    text: s.to_string(),
                },
                span,
            )),
        },
        _ => Err(bad_operands("int", v, "int / float / string / bool", span)),
    }
}

/// `float(x) -> float`：`int`/`string`/`bool` → `float`；非法串 → `ValueError`；
/// 支持 `"inf"`/`"-inf"`/`"nan"`。
fn b_float(args: &[Value], span: Span) -> R<Value> {
    let v = at("float", args, 0, span)?;
    match v {
        Value::Float(x) => Ok(Value::Float(*x)),
        Value::Int(i) => Ok(Value::Float(int_to_float(*i))),
        Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        Value::Str(s) => match s.trim().parse::<f64>() {
            Ok(parsed) => Ok(Value::Float(parsed)),
            Err(_) => Err(value_error(
                ValueMsg::Convert {
                    src: "string".to_string(),
                    dst: "float".to_string(),
                    text: s.to_string(),
                },
                span,
            )),
        },
        _ => Err(bad_operands("float", v, "int / float / string / bool", span)),
    }
}

/// `type(x) -> string`：`"int"`/`"float"`/`"string"`/`"bool"`/`"nil"`/`"array"`/`"struct"`/`"function"`。
fn b_type(args: &[Value], span: Span) -> R<Value> {
    let v = at("type", args, 0, span)?;
    Ok(Value::string(v.type_name()))
}

// ===========================================================================
// IO
// ===========================================================================

/// 各实参按显示形式（§3.7）拼接、空格连接。
fn display_join(args: &[Value]) -> String {
    let parts: Vec<String> = args.iter().map(ToString::to_string).collect();
    parts.join(" ")
}

/// `print(...) -> nil`：各参数显示形式、**空格连接** + 末尾 `\n`（stdout）。
fn b_print(args: &[Value], span: Span) -> R<Value> {
    let text = display_join(args);
    let mut out = std::io::stdout();
    writeln!(out, "{text}").map_err(|e| io_error(format!("无法写入：{e}"), Some(span)))?;
    Ok(Value::Nil)
}

/// `eprint(...) -> nil`：同 `print`，但写 **stderr**。
fn b_eprint(args: &[Value], span: Span) -> R<Value> {
    let text = display_join(args);
    let mut out = std::io::stderr();
    writeln!(out, "{text}").map_err(|e| io_error(format!("无法写入：{e}"), Some(span)))?;
    Ok(Value::Nil)
}

/// `input` 的可测内核：先打印 `prompt`（无换行）并 flush，再读一行；
/// **EOF → `IOError`**；去掉行尾 `\n` / `\r\n`。
fn input_with<Rd: BufRead, W: Write>(
    prompt: Option<&str>,
    out: &mut W,
    input: &mut Rd,
    span: Span,
) -> R<String> {
    if let Some(p) = prompt {
        out.write_all(p.as_bytes())
            .and_then(|()| out.flush())
            .map_err(|e| io_error(format!("无法写入：{e}"), Some(span)))?;
    }
    let mut line = String::new();
    let n = input
        .read_line(&mut line)
        .map_err(|e| io_error(format!("无法读取：{e}"), Some(span)))?;
    if n == 0 {
        return Err(io_error("输入结束（EOF）".to_string(), Some(span)));
    }
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(line)
}

/// `input(prompt?) -> string`：先打印 `prompt`（无换行）并 flush；EOF → `IOError`。
fn b_input(args: &[Value], span: Span) -> R<Value> {
    let prompt = match args.first() {
        Some(v) => Some(
            v.as_str()
                .ok_or_else(|| bad_operands("input", v, "string", span))?,
        ),
        None => None,
    };
    let stdin = std::io::stdin();
    let mut lock = stdin.lock();
    let mut out = std::io::stdout();
    input_with(prompt, &mut out, &mut lock, span).map(Value::string)
}

// ===========================================================================
// 断言
// ===========================================================================

/// `assert(cond, msg?) -> nil`：`cond` 须 `bool`；`false` → `AssertionError`（致命）；
/// 消息 `断言失败：{msg}`（省略 `msg` 时 `断言失败`）（§4.5.10）。
fn b_assert(args: &[Value], span: Span) -> R<Value> {
    let cond = need_bool("assert", args, 0, span)?;
    if cond {
        return Ok(Value::Nil);
    }
    let msg = match args.get(1) {
        Some(_) => format!("断言失败：{}", need_str("assert", args, 1, span)?),
        None => "断言失败".to_string(),
    };
    Err(assert_fail(msg, span))
}

/// `check(cond, msg?) -> bool`：`cond` 须 `bool`；`false` → stderr 一行警告 + 返回 `false`，
/// **不中断、不产生 `LzError`**（A4 / §4.5.10）。
fn b_check(args: &[Value], span: Span) -> R<Value> {
    let cond = need_bool("check", args, 0, span)?;
    if cond {
        return Ok(Value::Bool(true));
    }
    let warn = match args.get(1) {
        Some(_) => format!("check 失败：{}", need_str("check", args, 1, span)?),
        None => "check 失败".to_string(),
    };
    eprintln!("{warn}");
    Ok(Value::Bool(false))
}

/// `fail(msg?) -> never`：抛 `AssertionError`（致命）；消息 `{msg}`（省略时 `fail()`）（§4.5.10）。
fn b_fail(args: &[Value], span: Span) -> R<Value> {
    let msg = match args.first() {
        Some(_) => need_str("fail", args, 0, span)?.to_string(),
        None => "fail()".to_string(),
    };
    Err(assert_fail(msg, span))
}

// ===========================================================================
// 契约缺口（非阻塞）辅助
// ===========================================================================

/// `min` / `max` 空数组的 `ValueError`。
///
/// ⚠️ **契约缺口（非阻塞）**：§10.7 规定「空 → `ValueError`」，但 `semantics.md` §8.1 的 `ValueError`
/// 消息模板只有 `无法把 {src} 转换为 {dst}（'{text}'）` 与 `格式说明符非法：'{spec}'`，**均不适用于**
/// 「空集合取极值」。`error.rs` 的 `ValueMsg` 为只读的冻结枚举，无法新增变体，故此处以唯一可用的
/// `Convert` 承载，`text` 写明「空数组」。已列入收工汇报「需支持」，待 language-architect 补钉后改。
fn empty_collection_value_error(name: &str, span: Span) -> Box<LzError> {
    value_error(
        ValueMsg::Convert {
            src: "array".to_string(),
            dst: name.to_string(),
            text: "空数组".to_string(),
        },
        span,
    )
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::ScopeChain;
    use crate::error::R;
    use crate::span::Span;
    use crate::value::Closure;

    // ---- 辅助 -------------------------------------------------------------

    fn run(name: &str, args: &[Value]) -> R<Value> {
        super::call(name, args, Span::START)
    }
    fn ok(name: &str, args: &[Value]) -> Value {
        run(name, args).unwrap_or_else(|e| panic!("{name} 应成功，却得到 {}：{e}", e.class_name()))
    }
    fn cls(name: &str, args: &[Value]) -> &'static str {
        run(name, args).unwrap_err().class_name()
    }
    fn msg(name: &str, args: &[Value]) -> String {
        run(name, args).unwrap_err().message()
    }
    fn arr(vs: Vec<Value>) -> Value {
        Value::array(vs)
    }
    fn s(x: &str) -> Value {
        Value::string(x)
    }
    fn i(x: i64) -> Value {
        Value::Int(x)
    }
    fn f(x: f64) -> Value {
        Value::Float(x)
    }
    fn fnval() -> Value {
        Value::function(Closure::anonymous(Rc::new(ScopeChain::empty())))
    }
    fn st(fields: Vec<(&str, Value)>) -> Value {
        Value::object(fields.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    // ---- 核心 / 数组 ------------------------------------------------------

    #[test]
    fn len_covers_array_struct_string_and_rejects() {
        assert_eq!(ok("len", &[arr(vec![i(1), i(2), i(3)])]).to_string(), "3");
        assert_eq!(ok("len", &[arr(vec![])]).to_string(), "0");
        // struct 只数数据字段（不含方法，A5）。
        assert_eq!(ok("len", &[st(vec![("a", i(1)), ("m", fnval())])]).to_string(), "1");
        // string 数 Unicode 标量。
        assert_eq!(ok("len", &[s("héllo")]).to_string(), "5");
        assert_eq!(ok("len", &[s("你好")]).to_string(), "2");
        assert_eq!(cls("len", &[i(5)]), "TypeError");
    }

    #[test]
    fn range_basic_and_negative() {
        assert_eq!(ok("range", &[i(4)]).to_string(), "[0, 1, 2, 3]");
        assert_eq!(ok("range", &[i(0)]).to_string(), "[]");
        assert_eq!(ok("range", &[i(-3)]).to_string(), "[]");
        assert_eq!(cls("range", &[s("x")]), "TypeError");
    }

    #[test]
    fn a1_push_returns_new_array_original_unchanged() {
        let xs = arr(vec![i(1), i(2)]);
        let out = ok("push", &[i(3), xs.clone()]);
        assert_eq!(out.to_string(), "[1, 2, 3]");
        assert_eq!(xs.to_string(), "[1, 2]"); // 原容器不变（A1）
        match (&out, &xs) {
            (Value::Array(a), Value::Array(b)) => assert!(!Rc::ptr_eq(a, b)),
            _ => panic!("应为 array"),
        }
        assert_eq!(cls("push", &[i(3), i(5)]), "TypeError");
    }

    #[test]
    fn pop_removes_last_and_empty_errors() {
        let xs = arr(vec![i(1), i(2), i(3)]);
        assert_eq!(ok("pop", &[xs.clone()]).to_string(), "[1, 2]");
        assert_eq!(xs.to_string(), "[1, 2, 3]"); // 原容器不变
        assert_eq!(cls("pop", &[arr(vec![])]), "IndexError");
        assert_eq!(msg("pop", &[arr(vec![])]), "下标 -1 越界（长度 0）");
        assert_eq!(cls("pop", &[i(5)]), "TypeError");
    }

    #[test]
    fn remove_at_negative_and_out_of_bounds() {
        let xs = arr(vec![i(1), i(2), i(3)]);
        assert_eq!(ok("removeAt", &[i(1), xs.clone()]).to_string(), "[1, 3]");
        assert_eq!(ok("removeAt", &[i(-1), xs.clone()]).to_string(), "[1, 2]"); // 负索引
        assert_eq!(xs.to_string(), "[1, 2, 3]");
        assert_eq!(cls("removeAt", &[i(5), xs.clone()]), "IndexError");
        assert_eq!(cls("removeAt", &[i(-5), xs.clone()]), "IndexError");
        assert_eq!(cls("removeAt", &[s("a"), xs]), "TypeError");
    }

    #[test]
    fn insert_bounds_and_new_array() {
        let xs = arr(vec![i(1), i(2)]);
        assert_eq!(ok("insert", &[i(1), i(9), xs.clone()]).to_string(), "[1, 9, 2]");
        assert_eq!(ok("insert", &[i(2), i(9), xs.clone()]).to_string(), "[1, 2, 9]"); // i == len
        assert_eq!(xs.to_string(), "[1, 2]");
        assert_eq!(cls("insert", &[i(3), i(9), xs.clone()]), "IndexError"); // i > len
        assert_eq!(cls("insert", &[i(-1), i(9), xs]), "IndexError"); // 负索引不支持
    }

    #[test]
    fn swap_supports_negative_and_errors() {
        let xs = arr(vec![i(1), i(2), i(3)]);
        assert_eq!(ok("swap", &[i(0), i(1), xs.clone()]).to_string(), "[2, 1, 3]");
        assert_eq!(ok("swap", &[i(-1), i(0), xs.clone()]).to_string(), "[3, 2, 1]");
        assert_eq!(xs.to_string(), "[1, 2, 3]");
        assert_eq!(cls("swap", &[i(0), i(5), xs]), "IndexError");
    }

    #[test]
    fn slice_clamps_and_empty() {
        let xs = arr(vec![i(1), i(2), i(3), i(4)]);
        assert_eq!(ok("slice", &[i(1), i(3), xs.clone()]).to_string(), "[2, 3]");
        assert_eq!(ok("slice", &[i(-5), i(99), xs.clone()]).to_string(), "[1, 2, 3, 4]"); // 夹取
        assert_eq!(ok("slice", &[i(2), i(2), xs.clone()]).to_string(), "[]");
        assert_eq!(ok("slice", &[i(3), i(1), xs.clone()]).to_string(), "[]"); // from >= to
        assert_eq!(xs.to_string(), "[1, 2, 3, 4]");
        assert_eq!(cls("slice", &[i(0), i(1), i(5)]), "TypeError");
    }

    #[test]
    fn min_max_total_order_and_empty() {
        assert_eq!(ok("min", &[arr(vec![i(3), i(1), i(2)])]).to_string(), "1");
        assert_eq!(ok("max", &[arr(vec![i(3), i(1), i(2)])]).to_string(), "3");
        // int / float 混合：按数学精确值（§4.5.7）。
        assert_eq!(ok("min", &[arr(vec![i(3), f(1.5), i(2)])]).to_string(), "1.5");
        // string：UTF-8 字节序（§4.2）。
        assert_eq!(ok("min", &[arr(vec![s("b"), s("a"), s("c")])]).to_string(), "a");
        // 全序：NaN 排最后（§4.5.6）。
        assert_eq!(ok("max", &[arr(vec![f(1.0), f(f64::NAN), f(2.0)])]).to_string(), "nan");
        assert_eq!(ok("min", &[arr(vec![f(f64::NAN), f(1.0)])]).to_string(), "1.0");
        // 空 → ValueError。
        assert_eq!(cls("min", &[arr(vec![])]), "ValueError");
        assert_eq!(cls("max", &[arr(vec![])]), "ValueError");
        // 类型不一致 → TypeError。
        assert_eq!(cls("min", &[arr(vec![i(1), s("a")])]), "TypeError");
        assert_eq!(cls("max", &[arr(vec![Value::Bool(true), Value::Bool(false)])]), "TypeError");
    }

    #[test]
    fn sum_int_float_and_overflow() {
        assert!(matches!(ok("sum", &[arr(vec![i(1), i(2), i(3)])]), Value::Int(6)));
        assert_eq!(ok("sum", &[arr(vec![i(1), i(2), i(3)])]).to_string(), "6");
        assert!(matches!(ok("sum", &[arr(vec![i(1), f(2.5)])]), Value::Float(_)));
        assert_eq!(ok("sum", &[arr(vec![i(1), f(2.5)])]).to_string(), "3.5");
        assert_eq!(ok("sum", &[arr(vec![])]).to_string(), "0"); // 空 → int 0
        assert_eq!(cls("sum", &[arr(vec![i(i64::MAX), i(1)])]), "OverflowError");
        assert_eq!(cls("sum", &[arr(vec![i(1), s("a")])]), "TypeError");
    }

    #[test]
    fn sort_stable_total_order_and_type_error() {
        assert_eq!(ok("sort", &[arr(vec![i(3), i(1), i(2), i(1)])]).to_string(), "[1, 1, 2, 3]");
        // 全序：-inf < 有限 < +inf < NaN（§4.5.6）。
        let sorted = ok(
            "sort",
            &[arr(vec![
                f(f64::NAN),
                f(f64::INFINITY),
                f(f64::NEG_INFINITY),
                f(1.0),
                f(-1.0),
                f(0.0),
            ])],
        );
        assert_eq!(sorted.to_string(), "[-inf, -1.0, 0.0, 1.0, inf, nan]");
        // 混合数值：按精确值（§4.5.7）。
        assert_eq!(ok("sort", &[arr(vec![i(3), f(1.5), i(2)])]).to_string(), "[1.5, 2, 3]");
        // 大整数精确比较：9007199254740993 > 9007199254740992.0（加宽会误判相等）。
        assert_eq!(
            ok("sort", &[arr(vec![f(9007199254740992.0), i(9007199254740993)])]).to_string(),
            "[9007199254740992.0, 9007199254740993]"
        );
        assert_eq!(ok("sort", &[arr(vec![])]).to_string(), "[]");
        // 原容器不变。
        let xs = arr(vec![i(3), i(1)]);
        assert_eq!(ok("sort", &[xs.clone()]).to_string(), "[1, 3]");
        assert_eq!(xs.to_string(), "[3, 1]");
        // 类型不一致 → TypeError。
        assert_eq!(cls("sort", &[arr(vec![i(1), s("a")])]), "TypeError");
    }

    #[test]
    fn sort_is_stable() {
        // `-0.0` 与 `0.0` 在全序下相等（`partial_cmp == Equal`）但**可区分**（符号位），
        // 故等值元素的相对次序可观测 —— 稳定排序须保持输入序。
        assert_eq!(ok("sort", &[arr(vec![f(-0.0), f(0.0)])]).to_string(), "[-0.0, 0.0]");
        assert_eq!(ok("sort", &[arr(vec![f(0.0), f(-0.0)])]).to_string(), "[0.0, -0.0]");
    }

    #[test]
    fn take_drop_clamp() {
        let xs = arr(vec![i(1), i(2), i(3)]);
        assert_eq!(ok("take", &[i(2), xs.clone()]).to_string(), "[1, 2]");
        assert_eq!(ok("take", &[i(9), xs.clone()]).to_string(), "[1, 2, 3]");
        assert_eq!(ok("take", &[i(-1), xs.clone()]).to_string(), "[]");
        assert_eq!(ok("drop", &[i(1), xs.clone()]).to_string(), "[2, 3]");
        assert_eq!(ok("drop", &[i(0), xs.clone()]).to_string(), "[1, 2, 3]");
        assert_eq!(ok("drop", &[i(9), xs.clone()]).to_string(), "[]");
        assert_eq!(xs.to_string(), "[1, 2, 3]");
        assert_eq!(cls("take", &[s("a"), xs.clone()]), "TypeError");
        assert_eq!(cls("drop", &[s("a"), xs]), "TypeError");
    }

    #[test]
    fn a1_all_array_ops_leave_original_unchanged() {
        let xs = arr(vec![i(1), i(2), i(3)]);
        let _ = ok("push", &[i(4), xs.clone()]);
        let _ = ok("pop", &[xs.clone()]);
        let _ = ok("removeAt", &[i(0), xs.clone()]);
        let _ = ok("insert", &[i(0), i(0), xs.clone()]);
        let _ = ok("swap", &[i(0), i(1), xs.clone()]);
        let _ = ok("slice", &[i(0), i(1), xs.clone()]);
        let _ = ok("sort", &[xs.clone()]);
        let _ = ok("take", &[i(1), xs.clone()]);
        let _ = ok("drop", &[i(1), xs.clone()]);
        assert_eq!(xs.to_string(), "[1, 2, 3]", "所有数组内置都不得修改原容器（A1）");
    }

    // ---- struct / 字典 ----------------------------------------------------

    #[test]
    fn a5_keys_values_entries_skip_methods() {
        let v = st(vec![("b", i(2)), ("a", i(1)), ("m", fnval())]);
        assert_eq!(ok("keys", &[v.clone()]).to_string(), "[\"a\", \"b\"]"); // 字节序升序
        assert_eq!(ok("values", &[v.clone()]).to_string(), "[1, 2]");
        assert_eq!(ok("entries", &[v.clone()]).to_string(), "[[\"a\", 1], [\"b\", 2]]");
        assert_eq!(ok("has", &[s("a"), v.clone()]).to_string(), "true");
        assert_eq!(ok("has", &[s("m"), v.clone()]).to_string(), "false"); // 方法不算数据字段
        assert_eq!(ok("has", &[s("z"), v.clone()]).to_string(), "false");
        assert_eq!(ok("len", &[v]).to_string(), "2");
    }

    #[test]
    fn del_returns_new_struct_and_missing_field() {
        let v = st(vec![("a", i(1)), ("b", i(2))]);
        let out = ok("del", &[s("a"), v.clone()]);
        assert_eq!(out.to_string(), "{b: 2}");
        assert_eq!(v.to_string(), "{a: 1, b: 2}"); // 原 struct 不变（A1）
        match (&out, &v) {
            (Value::Struct(a), Value::Struct(b)) => assert!(!Rc::ptr_eq(a, b)),
            _ => panic!("应为 struct"),
        }
        assert_eq!(cls("del", &[s("z"), v.clone()]), "FieldError");
        // 方法字段可被 del（原样移除）。
        let with_method = st(vec![("a", i(1)), ("m", fnval())]);
        assert_eq!(ok("del", &[s("m"), with_method]).to_string(), "{a: 1}");
    }

    #[test]
    fn struct_ops_require_struct() {
        assert_eq!(cls("keys", &[i(5)]), "TypeError");
        assert_eq!(cls("values", &[arr(vec![])]), "TypeError");
        assert_eq!(cls("entries", &[s("x")]), "TypeError");
        assert_eq!(cls("has", &[i(1), i(5)]), "TypeError");
        assert_eq!(cls("has", &[s("k"), i(5)]), "TypeError");
        assert_eq!(cls("del", &[s("k"), i(5)]), "TypeError");
    }

    // ---- 字符串 -----------------------------------------------------------

    #[test]
    fn split_join_and_type_errors() {
        assert_eq!(ok("split", &[s(","), s("a,b,c")]).to_string(), "[\"a\", \"b\", \"c\"]");
        assert_eq!(ok("split", &[s(""), s("ab")]).to_string(), "[\"a\", \"b\"]"); // 空分隔 → 按字符
        assert_eq!(ok("split", &[s(""), s("")]).to_string(), "[]");
        assert_eq!(ok("split", &[s(","), s("")]).to_string(), "[\"\"]");
        assert_eq!(ok("join", &[s("-"), arr(vec![s("a"), s("b")])]).to_string(), "a-b");
        assert_eq!(ok("join", &[s("-"), arr(vec![])]).to_string(), "");
        assert_eq!(cls("join", &[s("-"), arr(vec![i(1)])]), "TypeError"); // 元素须 string
        assert_eq!(cls("split", &[i(1), s("a")]), "TypeError");
    }

    #[test]
    fn trim_upper_lower_replace_repeat_starts_with() {
        assert_eq!(ok("trim", &[s("  hi \n\t")]).to_string(), "hi");
        assert_eq!(ok("upper", &[s("aBc")]).to_string(), "ABC");
        assert_eq!(ok("lower", &[s("AbC")]).to_string(), "abc");
        // ASCII-only：非 ASCII 不变。
        assert_eq!(ok("upper", &[s("é")]).to_string(), "é");
        assert_eq!(ok("replace", &[s("a"), s("X"), s("banana")]).to_string(), "bXnXnX");
        assert_eq!(ok("repeat", &[i(3), s("ab")]).to_string(), "ababab");
        assert_eq!(ok("repeat", &[i(0), s("ab")]).to_string(), "");
        assert_eq!(ok("repeat", &[i(-2), s("ab")]).to_string(), "");
        assert_eq!(ok("startsWith", &[s("ba"), s("banana")]).to_string(), "true");
        assert_eq!(ok("startsWith", &[s("na"), s("banana")]).to_string(), "false");
        assert_eq!(cls("trim", &[i(1)]), "TypeError");
        assert_eq!(cls("upper", &[i(1)]), "TypeError");
        assert_eq!(cls("lower", &[i(1)]), "TypeError");
        assert_eq!(cls("replace", &[i(1), s("a"), s("b")]), "TypeError");
        assert_eq!(cls("repeat", &[s("a"), s("b")]), "TypeError");
        assert_eq!(cls("startsWith", &[i(1), s("b")]), "TypeError");
    }

    // ---- 数学 -------------------------------------------------------------

    #[test]
    fn abs_same_type_never_widens() {
        assert!(matches!(ok("abs", &[i(-3)]), Value::Int(3)));
        assert!(matches!(ok("abs", &[f(-3.0)]), Value::Float(x) if x == 3.0));
        assert_eq!(ok("abs", &[i(-3)]).to_string(), "3");
        assert_eq!(ok("abs", &[f(-3.0)]).to_string(), "3.0");
        assert_eq!(ok("abs", &[f(-0.0)]).to_string(), "0.0");
        assert_eq!(cls("abs", &[i(i64::MIN)]), "OverflowError");
        assert_eq!(cls("abs", &[s("x")]), "TypeError");
    }

    #[test]
    fn floor_ceil_round_widen_and_banker() {
        // int 实参按 as_f64 加宽（§10.7）；返回 int。
        assert_eq!(ok("floor", &[i(3)]).to_string(), "3"); // floor(3) == floor(3.0)
        assert_eq!(ok("floor", &[f(3.7)]).to_string(), "3");
        assert_eq!(ok("floor", &[f(-3.2)]).to_string(), "-4");
        assert_eq!(ok("ceil", &[f(3.2)]).to_string(), "4");
        assert_eq!(ok("ceil", &[i(3)]).to_string(), "3");
        // 银行家舍入（四舍六入五成双）。
        assert_eq!(ok("round", &[f(2.5)]).to_string(), "2");
        assert_eq!(ok("round", &[f(3.5)]).to_string(), "4");
        assert_eq!(ok("round", &[f(-2.5)]).to_string(), "-2");
        assert_eq!(ok("round", &[f(0.5)]).to_string(), "0");
        assert_eq!(ok("round", &[f(2.4)]).to_string(), "2");
        assert_eq!(ok("round", &[i(3)]).to_string(), "3");
        assert_eq!(cls("floor", &[s("x")]), "TypeError");
        assert_eq!(cls("ceil", &[arr(vec![])]), "TypeError");
        assert_eq!(cls("round", &[Value::Nil]), "TypeError");
    }

    #[test]
    fn sqrt_pow_and_div() {
        assert_eq!(ok("sqrt", &[i(4)]).to_string(), "2.0"); // int 加宽
        assert_eq!(ok("sqrt", &[f(4.0)]).to_string(), "2.0");
        assert_eq!(ok("sqrt", &[f(-1.0)]).to_string(), "nan"); // f < 0 → NaN
        assert_eq!(ok("pow", &[i(2), i(10)]).to_string(), "1024.0");
        assert_eq!(ok("pow", &[f(2.0), f(0.5)]).to_string(), "1.4142135623730951");
        // div：向下取整，符号随数学地板。
        assert_eq!(ok("div", &[i(7), i(2)]).to_string(), "3");
        assert_eq!(ok("div", &[i(-7), i(2)]).to_string(), "-4");
        assert_eq!(ok("div", &[i(7), i(-2)]).to_string(), "-4");
        assert_eq!(ok("div", &[i(-7), i(-2)]).to_string(), "3");
        assert_eq!(cls("div", &[i(1), i(0)]), "ZeroDivisionError");
        assert_eq!(msg("div", &[i(1), i(0)]), "除以零");
        assert_eq!(cls("div", &[i(i64::MIN), i(-1)]), "OverflowError");
        assert_eq!(cls("div", &[f(1.0), i(2)]), "TypeError"); // 两参须 int
        assert_eq!(cls("sqrt", &[s("x")]), "TypeError");
        assert_eq!(cls("pow", &[s("a"), i(2)]), "TypeError");
    }

    // ---- 随机 -------------------------------------------------------------

    #[test]
    fn rand_seed_deterministic_and_rand_int() {
        assert_eq!(ok("seed", &[i(42)]).to_string(), "nil");
        let a = ok("rand", &[]);
        let b = ok("rand", &[]);
        assert_eq!(ok("seed", &[i(42)]).to_string(), "nil");
        let c = ok("rand", &[]);
        assert_eq!(a.to_string(), c.to_string()); // 同种子 → 同序列
        assert_ne!(a.to_string(), b.to_string());
        match a {
            Value::Float(x) => assert!((0.0..1.0).contains(&x), "rand 须在 [0,1)，得到 {x}"),
            _ => panic!("rand 应返回 float"),
        }
        for _ in 0..32 {
            match ok("randInt", &[i(3), i(7)]) {
                Value::Int(v) => assert!((3..7).contains(&v), "randInt 须在 [3,7)，得到 {v}"),
                _ => panic!("randInt 应返回 int"),
            }
        }
        // randInt(i64::MIN, i64::MAX) 不应溢出。
        assert!(matches!(ok("randInt", &[i(i64::MIN), i(i64::MAX)]), Value::Int(_)));
        assert_eq!(cls("randInt", &[i(5), i(5)]), "ValueError");
        assert_eq!(cls("randInt", &[i(7), i(3)]), "ValueError");
        assert_eq!(cls("randInt", &[s("a"), i(3)]), "TypeError");
        assert_eq!(cls("rand", &[i(1)]), "TypeError"); // 0 参
        assert_eq!(cls("seed", &[]), "TypeError"); // 1 参
        assert_eq!(cls("seed", &[s("a")]), "TypeError");
    }

    // ---- 转换 / 类型 ------------------------------------------------------

    #[test]
    fn int_conversion_boundaries() {
        assert_eq!(ok("int", &[i(5)]).to_string(), "5");
        assert_eq!(ok("int", &[f(2.9)]).to_string(), "2"); // 向零截断
        assert_eq!(ok("int", &[f(-2.9)]).to_string(), "-2"); // 向零，非向下取整
        assert_eq!(ok("int", &[Value::Bool(true)]).to_string(), "1");
        assert_eq!(ok("int", &[Value::Bool(false)]).to_string(), "0");
        assert_eq!(ok("int", &[s("42")]).to_string(), "42");
        assert_eq!(ok("int", &[s(" 7 ")]).to_string(), "7");
        // 极窄边界（§4.5.7）。
        assert_eq!(cls("int", &[f(f64::NAN)]), "ValueError");
        assert_eq!(cls("int", &[f(f64::INFINITY)]), "OverflowError");
        assert_eq!(cls("int", &[f(f64::NEG_INFINITY)]), "OverflowError");
        assert_eq!(cls("int", &[f(1e30)]), "OverflowError");
        assert_eq!(msg("int", &[f(f64::NAN)]), "无法把 float 转换为 int（'nan'）");
        assert_eq!(cls("int", &[s("abc")]), "ValueError");
        assert_eq!(msg("int", &[s("abc")]), "无法把 string 转换为 int（'abc'）");
        assert_eq!(cls("int", &[Value::Nil]), "TypeError");
        assert_eq!(cls("int", &[arr(vec![])]), "TypeError");
    }

    #[test]
    fn float_conversion_including_specials() {
        assert_eq!(ok("float", &[i(3)]).to_string(), "3.0");
        assert_eq!(ok("float", &[f(1.5)]).to_string(), "1.5");
        assert_eq!(ok("float", &[Value::Bool(true)]).to_string(), "1.0");
        assert_eq!(ok("float", &[s("1.5")]).to_string(), "1.5");
        assert_eq!(ok("float", &[s("inf")]).to_string(), "inf");
        assert_eq!(ok("float", &[s("-inf")]).to_string(), "-inf");
        assert_eq!(ok("float", &[s("nan")]).to_string(), "nan");
        assert_eq!(cls("float", &[s("x")]), "ValueError");
        assert_eq!(cls("float", &[arr(vec![])]), "TypeError");
    }

    #[test]
    fn str_and_type() {
        assert_eq!(ok("str", &[i(1)]).to_string(), "1");
        assert_eq!(ok("str", &[f(1.0)]).to_string(), "1.0");
        assert_eq!(ok("str", &[arr(vec![i(1), i(2)])]).to_string(), "[1, 2]");
        assert_eq!(ok("str", &[s("hi")]).to_string(), "hi"); // 顶层字符串裸输出
        assert_eq!(ok("str", &[Value::Nil]).to_string(), "nil");
        assert_eq!(ok("type", &[i(1)]).to_string(), "int");
        assert_eq!(ok("type", &[f(1.0)]).to_string(), "float");
        assert_eq!(ok("type", &[s("x")]).to_string(), "string");
        assert_eq!(ok("type", &[Value::Bool(true)]).to_string(), "bool");
        assert_eq!(ok("type", &[Value::Nil]).to_string(), "nil");
        assert_eq!(ok("type", &[arr(vec![])]).to_string(), "array");
        assert_eq!(ok("type", &[st(vec![])]).to_string(), "struct");
        assert_eq!(ok("type", &[Value::struct_def("Point")]).to_string(), "struct");
        assert_eq!(ok("type", &[fnval()]).to_string(), "function");
        assert_eq!(cls("str", &[]), "TypeError");
        assert_eq!(cls("type", &[]), "TypeError");
    }

    // ---- IO ---------------------------------------------------------------

    #[test]
    fn print_eprint_return_nil() {
        assert_eq!(ok("print", &[s("a"), i(1)]).to_string(), "nil");
        assert_eq!(ok("print", &[]).to_string(), "nil");
        assert_eq!(ok("eprint", &[s("e")]).to_string(), "nil");
        assert_eq!(ok("eprint", &[]).to_string(), "nil");
    }

    #[test]
    fn input_reads_line_crlf_and_eof() {
        // 正常一行 + prompt 原样写出。
        let mut out = Vec::new();
        let mut input = std::io::Cursor::new(b"hello\n".to_vec());
        assert_eq!(
            input_with(Some("P> "), &mut out, &mut input, Span::START).unwrap(),
            "hello"
        );
        assert_eq!(out, b"P> ");
        // CRLF：去掉尾 \r\n。
        let mut out = Vec::new();
        let mut input = std::io::Cursor::new(b"hi\r\n".to_vec());
        assert_eq!(
            input_with(None, &mut out, &mut input, Span::START).unwrap(),
            "hi"
        );
        // EOF → IOError。
        let mut out = Vec::new();
        let mut input = std::io::Cursor::new(Vec::new());
        let e = input_with(None, &mut out, &mut input, Span::START).unwrap_err();
        assert_eq!(e.class_name(), "IOError");
        assert_eq!(e.message(), "输入结束（EOF）");
        // 可选 prompt 的实参类型 / 个数。
        assert_eq!(cls("input", &[i(1)]), "TypeError");
        assert_eq!(cls("input", &[s("a"), s("b")]), "TypeError");
    }

    // ---- 断言 -------------------------------------------------------------

    #[test]
    fn assert_fatal_check_nonfatal_fail() {
        assert_eq!(ok("assert", &[Value::Bool(true)]).to_string(), "nil");

        let e = run("assert", &[Value::Bool(false)]).unwrap_err();
        assert_eq!(e.class_name(), "AssertionError");
        assert_eq!(e.message(), "断言失败");
        let e = run("assert", &[Value::Bool(false), s("boom")]).unwrap_err();
        assert_eq!(e.message(), "断言失败：boom");

        // check 非致命：返回 false，不产生 LzError。
        assert!(matches!(ok("check", &[Value::Bool(true)]), Value::Bool(true)));
        assert!(matches!(ok("check", &[Value::Bool(false)]), Value::Bool(false)));
        assert!(matches!(
            ok("check", &[Value::Bool(false), s("soft")]),
            Value::Bool(false)
        ));

        // fail 致命。
        let e = run("fail", &[]).unwrap_err();
        assert_eq!(e.class_name(), "AssertionError");
        assert_eq!(e.message(), "fail()");
        let e = run("fail", &[s("boom")]).unwrap_err();
        assert_eq!(e.message(), "boom");

        // cond 非 bool → TypeError（ConditionNotBool）。
        assert_eq!(cls("assert", &[i(1)]), "TypeError");
        assert_eq!(cls("check", &[i(1)]), "TypeError");
        assert_eq!(msg("assert", &[i(1)]), "条件必须是 bool，得到 int");
    }

    // ---- 调用约定 / 注册表 ------------------------------------------------

    #[test]
    fn arg_count_errors() {
        assert_eq!(msg("len", &[]), "函数 len 期待 1 个参数，得到 0");
        assert_eq!(msg("len", &[i(1), i(2)]), "函数 len 期待 1 个参数，得到 2");
        assert_eq!(msg("input", &[s("a"), s("b")]), "函数 input 期待 1 个参数，得到 2");
        assert_eq!(msg("rand", &[i(1)]), "函数 rand 期待 0 个参数，得到 1");
        assert_eq!(msg("fail", &[s("a"), s("b")]), "函数 fail 期待 1 个参数，得到 2");
        assert_eq!(cls("pow", &[i(2)]), "TypeError");
        assert_eq!(cls("slice", &[i(0), i(1)]), "TypeError");
    }

    #[test]
    fn lookup_table_names_consistency() {
        assert!(lookup("len").is_some());
        assert!(is_builtin("push"));
        assert!(!is_builtin("nope"));
        assert!(lookup("nope").is_none());
        assert_eq!(BUILTIN_NAMES.len(), 47);
        // 字母序。
        let mut sorted = BUILTIN_NAMES.to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted.as_slice(), BUILTIN_NAMES);
        // 无重复。
        let mut dedup = sorted.clone();
        dedup.dedup();
        assert_eq!(dedup.len(), 47);
        // 表与名单逐项一致。
        let table_names: Vec<&str> = TABLE.iter().map(|b| b.name).collect();
        assert_eq!(table_names.as_slice(), BUILTIN_NAMES);
    }

    #[test]
    fn hof_deferred_names_are_absent() {
        for name in ["map", "filter", "reduce", "sortBy", "minBy", "maxBy", "each"] {
            assert!(lookup(name).is_none(), "{name} 应留待 P3.9b（高阶内置）");
            assert!(!is_builtin(name));
        }
    }

    #[test]
    fn unknown_builtin_call_is_name_error() {
        assert_eq!(super::call("nope", &[], Span::START).unwrap_err().class_name(), "NameError");
    }

    #[test]
    fn builtin_errors_carry_call_site_span() {
        let sp = Span::new(7, 3);
        let e = super::call("len", &[], sp).unwrap_err();
        assert_eq!(e.span(), Some(sp));
        let e = super::call("div", &[i(1), i(0)], sp).unwrap_err();
        assert_eq!(e.span(), Some(sp));
    }

    #[test]
    fn bad_argument_types_are_type_errors() {
        // 覆盖各内置的「类型不符」负例（§10.7：参数类型不符 → TypeError）。
        assert_eq!(cls("len", &[i(1)]), "TypeError");
        assert_eq!(cls("range", &[s("x")]), "TypeError");
        assert_eq!(cls("push", &[i(1), i(2)]), "TypeError");
        assert_eq!(cls("pop", &[s("x")]), "TypeError");
        assert_eq!(cls("removeAt", &[i(0), s("x")]), "TypeError");
        assert_eq!(cls("insert", &[i(0), i(1), s("x")]), "TypeError");
        assert_eq!(cls("swap", &[i(0), i(1), s("x")]), "TypeError");
        assert_eq!(cls("slice", &[i(0), i(1), s("x")]), "TypeError");
        assert_eq!(cls("min", &[s("x")]), "TypeError");
        assert_eq!(cls("max", &[i(1)]), "TypeError");
        assert_eq!(cls("sum", &[i(1)]), "TypeError");
        assert_eq!(cls("sort", &[i(1)]), "TypeError");
        assert_eq!(cls("take", &[i(1), i(2)]), "TypeError");
        assert_eq!(cls("drop", &[i(1), i(2)]), "TypeError");
        assert_eq!(cls("keys", &[i(1)]), "TypeError");
        assert_eq!(cls("values", &[i(1)]), "TypeError");
        assert_eq!(cls("entries", &[i(1)]), "TypeError");
        assert_eq!(cls("has", &[i(1), st(vec![])]), "TypeError");
        assert_eq!(cls("del", &[i(1), st(vec![])]), "TypeError");
        assert_eq!(cls("split", &[s(","), i(1)]), "TypeError");
        assert_eq!(cls("join", &[i(1), arr(vec![])]), "TypeError");
        assert_eq!(cls("trim", &[Value::Nil]), "TypeError");
        assert_eq!(cls("upper", &[Value::Bool(true)]), "TypeError");
        assert_eq!(cls("lower", &[Value::Bool(true)]), "TypeError");
        assert_eq!(cls("replace", &[s("a"), i(1), s("b")]), "TypeError");
        assert_eq!(cls("repeat", &[i(1), i(2)]), "TypeError");
        assert_eq!(cls("startsWith", &[s("a"), i(1)]), "TypeError");
        assert_eq!(cls("abs", &[Value::Nil]), "TypeError");
        assert_eq!(cls("floor", &[Value::Bool(true)]), "TypeError");
        assert_eq!(cls("ceil", &[s("x")]), "TypeError");
        assert_eq!(cls("round", &[s("x")]), "TypeError");
        assert_eq!(cls("sqrt", &[arr(vec![])]), "TypeError");
        assert_eq!(cls("pow", &[i(2), s("x")]), "TypeError");
        assert_eq!(cls("div", &[i(1), s("x")]), "TypeError");
        assert_eq!(cls("randInt", &[i(1), s("x")]), "TypeError");
        assert_eq!(cls("seed", &[Value::Bool(true)]), "TypeError");
        assert_eq!(cls("float", &[Value::Nil]), "TypeError");
        assert_eq!(cls("assert", &[Value::Nil]), "TypeError");
        assert_eq!(cls("check", &[Value::Nil]), "TypeError");
        assert_eq!(cls("fail", &[i(1)]), "TypeError");
        assert_eq!(cls("input", &[Value::Nil]), "TypeError");
    }
}
