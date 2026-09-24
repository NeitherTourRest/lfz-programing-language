//! 错误模型 `LfzError`（12 变体 + 1 基类）+ 子消息枚举 + 结果别名 `R<T>` + traceback 帧。
//!
//! 单一事实源（本模块**只读消费**，不得偏离）：
//! - `docs/spec/interface-contract.md` §8.1（类名 ↔ 变体映射）、§10.3（`TraceFrame`）、
//!   §10.4（`LfzError` 12 变体与 `class_name` / `message` / `span`）、§10.8（`R<T>` / `#[cold]`）。
//! - `docs/spec/semantics.md` §8.1（各错误类**触发条件**与**中文消息细分表**）、§8.2（Traceback 头规则）。
//!
//! 设计口径：
//! - **无 `E-xxx` 编号**：用户可见输出一律「类名 + 中文消息」。
//! - `class_name()` 返回**类名**（如 `"SyntaxError"`），**不**返回任何错误码。
//! - 子消息枚举（`SyntaxMsg` / `TypeMsg` / `OverflowMsg` / `ValueMsg`）以 `semantics.md` §8.1
//!   细分表的**每一条消息**为最小完备集，逐条对应一个变体；`message()` 据此产出中文消息。
//! - `Display` 仅产出 `<类名>: <中文消息>`（供 CLI 组装）；**Traceback 由显示层拼装**，不在此打印。

use crate::span::Span;
use std::fmt;

/// 解释器统一结果类型：`Err` 侧为 `Box<LzError>`，使 `R<Value>` 保持寄存器友好（§10.8）。
///
/// 例：`Result<Value, Box<LzError>>` 仍为**指针大小**（`Box` 的非空 niche 被 `Result` 复用）。
pub type R<T> = Result<T, Box<LzError>>;

// ===========================================================================
// 子消息枚举
// ===========================================================================

/// `SyntaxError` 细分消息 —— `semantics.md` §8.1「`SyntaxError` 细分消息」表，**17 条**。
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SyntaxMsg {
    /// `非法字符 '{c}'`
    IllegalChar { c: char },
    /// `'#' 只能出现在文件首行的前导位；(字符串 / 注释 / 格式说明符内的 '#' 除外)`
    HashPosition,
    /// `字符串字面量在此处未闭合`
    UnterminatedString,
    /// `块注释在此处未闭合（缺少 '*/'）`
    UnterminatedBlockComment,
    /// `这里期待 {expected}，但得到 {got}`
    UnexpectedToken { expected: String, got: String },
    /// `表达式未结束：行尾不能终止表达式；请用括号跨行`
    IncompleteExpr,
    /// `单独的 ';' 非法；打印变量请用 ';;'`
    LoneSemicolon,
    /// `语句之间必须有换行`
    TwoStatements,
    /// `赋值左侧必须是变量、字段或下标`
    InvalidAssignTarget,
    /// `占位符 '_' 只能出现在管道右侧的调用实参中`
    PlaceholderPosition,
    /// `管道右侧调用最多只能有一个 '_'`
    PipeMultiplePlaceholder,
    /// `'{kw}' 只能出现在循环体内`
    BreakContinueOutsideLoop { kw: String },
    /// `'return' 只能出现在函数体内`
    ReturnOutsideFunction,
    /// `文件不是合法的 UTF-8 编码（首个非法字节位于字节偏移 {off}）`
    NotUtf8 { off: usize },
    /// `字符串中不支持的转义 '\{c}'`
    UnknownEscape { c: char },
    /// `插值表达式不能跨行；请把表达式写在一行内`
    InterpolationNewline,
    /// `整数字面量超出 i64 范围`
    IntegerOutOfRange,
}

impl SyntaxMsg {
    /// 产出该细分场景的中文消息（`semantics.md` §8.1）。
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            SyntaxMsg::IllegalChar { c } => format!("非法字符 '{c}'"),
            SyntaxMsg::HashPosition => {
                "'#' 只能出现在文件首行的前导位；(字符串 / 注释 / 格式说明符内的 '#' 除外)".to_string()
            }
            SyntaxMsg::UnterminatedString => "字符串字面量在此处未闭合".to_string(),
            SyntaxMsg::UnterminatedBlockComment => {
                "块注释在此处未闭合（缺少 '*/'）".to_string()
            }
            SyntaxMsg::UnexpectedToken { expected, got } => {
                format!("这里期待 {expected}，但得到 {got}")
            }
            SyntaxMsg::IncompleteExpr => {
                "表达式未结束：行尾不能终止表达式；请用括号跨行".to_string()
            }
            SyntaxMsg::LoneSemicolon => "单独的 ';' 非法；打印变量请用 ';;'".to_string(),
            SyntaxMsg::TwoStatements => "语句之间必须有换行".to_string(),
            SyntaxMsg::InvalidAssignTarget => "赋值左侧必须是变量、字段或下标".to_string(),
            SyntaxMsg::PlaceholderPosition => {
                "占位符 '_' 只能出现在管道右侧的调用实参中".to_string()
            }
            SyntaxMsg::PipeMultiplePlaceholder => {
                "管道右侧调用最多只能有一个 '_'".to_string()
            }
            SyntaxMsg::BreakContinueOutsideLoop { kw } => {
                format!("'{kw}' 只能出现在循环体内")
            }
            SyntaxMsg::ReturnOutsideFunction => "'return' 只能出现在函数体内".to_string(),
            SyntaxMsg::NotUtf8 { off } => {
                format!("文件不是合法的 UTF-8 编码（首个非法字节位于字节偏移 {off}）")
            }
            SyntaxMsg::UnknownEscape { c } => format!("字符串中不支持的转义 '\\{c}'"),
            SyntaxMsg::InterpolationNewline => {
                "插值表达式不能跨行；请把表达式写在一行内".to_string()
            }
            SyntaxMsg::IntegerOutOfRange => "整数字面量超出 i64 范围".to_string(),
        }
    }
}

/// `TypeError` 细分消息 —— `semantics.md` §8.1「`TypeError` 细分消息」表，**7 条**。
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeMsg {
    /// `运算符 '{op}' 不支持 {lt} 与 {rt}`
    BadOperands { op: String, lt: String, rt: String },
    /// `条件必须是 bool，得到 {t}`
    ConditionNotBool { t: String },
    /// `不可调用：{t} 不是函数`
    NotCallable { t: String },
    /// `管道右侧必须是函数，得到 {t}`
    PipeRhsNotFunction { t: String },
    /// `函数 {name} 期待 {n} 个参数，得到 {m}`
    ArgCount { name: String, n: usize, m: usize },
    /// `格式说明符 '{spec}' 不适用于 {t}`
    FormatSpecMismatch { spec: String, t: String },
    /// 重绑定 `let` 变量（v1 补钉，`semantics.md` §4.5.2 / §8.1）：
    /// `不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容`。
    ImmutableRebind { name: String },
}

impl TypeMsg {
    /// 产出该细分场景的中文消息（`semantics.md` §8.1）。
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            TypeMsg::BadOperands { op, lt, rt } => {
                format!("运算符 '{op}' 不支持 {lt} 与 {rt}")
            }
            TypeMsg::ConditionNotBool { t } => format!("条件必须是 bool，得到 {t}"),
            TypeMsg::NotCallable { t } => format!("不可调用：{t} 不是函数"),
            TypeMsg::PipeRhsNotFunction { t } => {
                format!("管道右侧必须是函数，得到 {t}")
            }
            TypeMsg::ArgCount { name, n, m } => {
                format!("函数 {name} 期待 {n} 个参数，得到 {m}")
            }
            TypeMsg::FormatSpecMismatch { spec, t } => {
                format!("格式说明符 '{spec}' 不适用于 {t}")
            }
            TypeMsg::ImmutableRebind { name } => {
                format!("不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容")
            }
        }
    }
}

/// `OverflowError` 子消息 —— `semantics.md` §8.1 该行仅给出**一条**消息模板，故为**单变体**枚举
/// （保留枚举形态以对齐契约 §10.4 `Overflow { span, msg: OverflowMsg }` 与未来扩展）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OverflowMsg {
    /// `整数溢出：结果超出 i64 范围`（涵盖 `int` 运算溢出与 `int(float)` 的 `±Inf` / 截断超界）。
    IntegerOutOfRange,
}

impl OverflowMsg {
    /// 产出中文消息（`semantics.md` §8.1）。
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            OverflowMsg::IntegerOutOfRange => "整数溢出：结果超出 i64 范围".to_string(),
        }
    }
}

/// `ValueError` 子消息 —— `semantics.md` §8.1 该行给出**四条**消息模板，故为**四变体**枚举。
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ValueMsg {
    /// `无法把 {src} 转换为 {dst}（'{text}'）`
    Convert {
        src: String,
        dst: String,
        text: String,
    },
    /// `格式说明符非法：'{spec}'`
    BadFormatSpec { spec: String },
    /// `空数组没有极值（{func}）` —— `min` / `max` / `minBy` / `maxBy` 空数组（`semantics.md` §8.1）。
    EmptyExtremum { func: String },
    /// `区间非法：{lo} >= {hi}` —— `randInt(lo, hi)` 且 `lo >= hi`（`semantics.md` §8.1）。
    BadRange { lo: i64, hi: i64 },
}

impl ValueMsg {
    /// 产出该细分场景的中文消息（`semantics.md` §8.1）。
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            ValueMsg::Convert { src, dst, text } => {
                format!("无法把 {src} 转换为 {dst}（'{text}'）")
            }
            ValueMsg::BadFormatSpec { spec } => format!("格式说明符非法：'{spec}'"),
            ValueMsg::EmptyExtremum { func } => format!("空数组没有极值（{func}）"),
            ValueMsg::BadRange { lo, hi } => format!("区间非法：{lo} >= {hi}"),
        }
    }
}

// ===========================================================================
// 主错误枚举
// ===========================================================================

/// LFZ 错误（**12 个具体变体 + 1 个基类概念**；基类 `LfzError` **不直接抛出**）。
///
/// 变体 ↔ 类名映射见 `interface-contract.md` §8.1 / `semantics.md` §8.1。
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LzError {
    /// 加载期：`.lfz` 文件缺少合法 `#42` 前导。`span` 恒为 `line 1, col 1`；消息固定。
    CosmosAnswer { span: Span },
    /// 加载/解析期：词法 / 语法错误。
    Syntax { msg: SyntaxMsg, span: Span },
    /// 运行期：引用未定义的名字。
    Name { name: String, span: Span },
    /// 运行期：类型不符。
    Type { msg: TypeMsg, span: Span },
    /// 运行期：数组下标越界。
    Index { idx: i64, len: usize, span: Span },
    /// 运行期：struct 不存在该字段。
    Field { name: String, span: Span },
    /// 运行期：除零 / 对零取模（`modulo == true` 表示 `%`）。
    DivZero { modulo: bool, span: Span },
    /// 运行期：整数溢出。
    Overflow { span: Span, msg: OverflowMsg },
    /// 运行期：显式转换 / 格式说明符错误。
    Value { msg: ValueMsg, span: Span },
    /// 运行期：IO 错误。`span` 可为 `None`（如无源码关联的读文件失败）。
    Io { msg: String, span: Option<Span> },
    /// 运行期：`assert` / `fail`（致命）。
    Assert { msg: String, span: Span },
    /// 运行期：求值帧 / 深结构处理超限（默认 10000 层）。
    Recursion { depth: u32, limit: u32, span: Span },
}

impl LzError {
    /// 返回**类名**（`interface-contract.md` §8.1 第一列）—— 供 `--json` 的 `error` 字段与显示层使用。
    ///
    /// **不**返回任何 `E-xxx` 错误码（编号制已取消）。
    #[must_use]
    pub fn class_name(&self) -> &'static str {
        match self {
            LzError::CosmosAnswer { .. } => "CosmosAnswerError",
            LzError::Syntax { .. } => "SyntaxError",
            LzError::Name { .. } => "NameError",
            LzError::Type { .. } => "TypeError",
            LzError::Index { .. } => "IndexError",
            LzError::Field { .. } => "FieldError",
            LzError::DivZero { .. } => "ZeroDivisionError",
            LzError::Overflow { .. } => "OverflowError",
            LzError::Value { .. } => "ValueError",
            LzError::Io { .. } => "IOError",
            LzError::Assert { .. } => "AssertionError",
            LzError::Recursion { .. } => "RecursionError",
        }
    }

    /// 返回**中文消息**（`semantics.md` §8.1）。
    ///
    /// `Assert` / `Io` 的 `msg` 字段承载**已组装完成的最终消息**（`assert` 与 `fail` 的包装差异
    /// 由构造点按 §4.5.10 决定），故此处原样返回。
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            LzError::CosmosAnswer { .. } => "你忘记了宇宙的答案".to_string(),
            LzError::Syntax { msg, .. } => msg.message(),
            LzError::Name { name, .. } => format!("未定义的名字 '{name}'"),
            LzError::Type { msg, .. } => msg.message(),
            LzError::Index { idx, len, .. } => format!("下标 {idx} 越界（长度 {len}）"),
            LzError::Field { name, .. } => format!("结构体没有字段 '{name}'"),
            LzError::DivZero { modulo, .. } => {
                if *modulo {
                    "对零取模".to_string()
                } else {
                    "除以零".to_string()
                }
            }
            LzError::Overflow { msg, .. } => msg.message(),
            LzError::Value { msg, .. } => msg.message(),
            LzError::Io { msg, .. } => msg.clone(),
            LzError::Assert { msg, .. } => msg.clone(),
            LzError::Recursion { .. } => "递归深度超限（超过 10000 层）".to_string(),
        }
    }

    /// 返回源码位置；仅 `Io` 可能为 `None`。
    #[must_use]
    pub fn span(&self) -> Option<Span> {
        match self {
            LzError::CosmosAnswer { span }
            | LzError::Syntax { span, .. }
            | LzError::Name { span, .. }
            | LzError::Type { span, .. }
            | LzError::Index { span, .. }
            | LzError::Field { span, .. }
            | LzError::DivZero { span, .. }
            | LzError::Overflow { span, .. }
            | LzError::Value { span, .. }
            | LzError::Assert { span, .. }
            | LzError::Recursion { span, .. } => Some(*span),
            LzError::Io { span, .. } => *span,
        }
    }
}

impl fmt::Display for LzError {
    /// `<类名>: <中文消息>`（供 CLI 组装；**Traceback 不在此打印**）。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.class_name(), self.message())
    }
}

// ===========================================================================
// traceback 帧
// ===========================================================================

/// traceback 帧（`interface-contract.md` §10.3）：`func_id` 是**函数表下标**，
/// **不存 `Rc<str>`**（避免每次调用分配）；仅在**报错时**按 `func_id` 反查名字
/// （`<module>` / `<fn 名字>` / 匿名 `<fn>`）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TraceFrame {
    /// 函数表下标。
    pub func_id: u32,
    /// 该帧当前正在求值的最小 AST 节点的 `Span`（§10.3 N1）。
    pub span: Span,
}

// `VmFrame`（执行 / VM 帧）**本阶段不实现**：P3 采用**树遍历求值器**，无字节码 chunk / ip；
// 其等价结构由 runtime-dev 的调用栈持有。契约 §10.3 允许树遍历器使用等价结构。
// （若 P6 决定上 VM，再由 core-dev / runtime-dev 走 ADR 补齐 `VmFrame`。）

// ===========================================================================
// 冷路径构造器（§10.8：`#[cold]` + `#[inline(never)]`，`Box::new` 只在出错时执行）
// ===========================================================================

/// 通用冷路径装箱：把已构造的 `LzError` 放进 `Box`（热路径只走 `Ok(v)`）。
#[cold]
#[inline(never)]
pub fn boxed(err: LzError) -> Box<LzError> {
    Box::new(err)
}

/// `CosmosAnswerError`（`span` 恒为 `Span::START`）。
#[cold]
#[inline(never)]
pub fn cosmos_answer() -> Box<LzError> {
    Box::new(LzError::CosmosAnswer { span: Span::START })
}

/// `SyntaxError`。
#[cold]
#[inline(never)]
pub fn syntax(msg: SyntaxMsg, span: Span) -> Box<LzError> {
    Box::new(LzError::Syntax { msg, span })
}

/// `NameError`。
#[cold]
#[inline(never)]
pub fn name(name: String, span: Span) -> Box<LzError> {
    Box::new(LzError::Name { name, span })
}

/// `TypeError`。
#[cold]
#[inline(never)]
pub fn type_error(msg: TypeMsg, span: Span) -> Box<LzError> {
    Box::new(LzError::Type { msg, span })
}

/// `IndexError`。
#[cold]
#[inline(never)]
pub fn index(idx: i64, len: usize, span: Span) -> Box<LzError> {
    Box::new(LzError::Index { idx, len, span })
}

/// `FieldError`。
#[cold]
#[inline(never)]
pub fn field(name: String, span: Span) -> Box<LzError> {
    Box::new(LzError::Field { name, span })
}

/// `ZeroDivisionError`（`modulo == true` 表示 `%`）。
#[cold]
#[inline(never)]
pub fn div_zero(modulo: bool, span: Span) -> Box<LzError> {
    Box::new(LzError::DivZero { modulo, span })
}

/// `OverflowError`。
#[cold]
#[inline(never)]
pub fn overflow(span: Span, msg: OverflowMsg) -> Box<LzError> {
    Box::new(LzError::Overflow { span, msg })
}

/// `ValueError`。
#[cold]
#[inline(never)]
pub fn value(msg: ValueMsg, span: Span) -> Box<LzError> {
    Box::new(LzError::Value { msg, span })
}

/// `IOError`（`span` 可为 `None`）。
#[cold]
#[inline(never)]
pub fn io(msg: String, span: Option<Span>) -> Box<LzError> {
    Box::new(LzError::Io { msg, span })
}

/// `AssertionError`（`msg` 为**已组装完成的最终消息**，见 `message()` 注释）。
#[cold]
#[inline(never)]
pub fn assert_fail(msg: String, span: Span) -> Box<LzError> {
    Box::new(LzError::Assert { msg, span })
}

/// `RecursionError`。
#[cold]
#[inline(never)]
pub fn recursion(depth: u32, limit: u32, span: Span) -> Box<LzError> {
    Box::new(LzError::Recursion { depth, limit, span })
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 12 个具体错误类的样本（每类恰好 1 个，顺序与 `semantics.md` §8.1 表一致）。
    fn samples() -> Vec<LzError> {
        vec![
            LzError::CosmosAnswer { span: Span::START },
            LzError::Syntax {
                msg: SyntaxMsg::IllegalChar { c: '$' },
                span: Span::new(2, 11),
            },
            LzError::Name {
                name: "x".to_string(),
                span: Span::new(3, 1),
            },
            LzError::Type {
                msg: TypeMsg::ConditionNotBool {
                    t: "int".to_string(),
                },
                span: Span::new(4, 1),
            },
            LzError::Index {
                idx: 5,
                len: 3,
                span: Span::new(5, 1),
            },
            LzError::Field {
                name: "k".to_string(),
                span: Span::new(6, 1),
            },
            LzError::DivZero {
                modulo: false,
                span: Span::new(7, 1),
            },
            LzError::Overflow {
                span: Span::new(8, 1),
                msg: OverflowMsg::IntegerOutOfRange,
            },
            LzError::Value {
                msg: ValueMsg::BadFormatSpec {
                    spec: "q".to_string(),
                },
                span: Span::new(9, 1),
            },
            LzError::Io {
                msg: "输入结束（EOF）".to_string(),
                span: None,
            },
            LzError::Assert {
                msg: "断言失败：boom".to_string(),
                span: Span::new(10, 1),
            },
            LzError::Recursion {
                depth: 10001,
                limit: 10000,
                span: Span::new(11, 1),
            },
        ]
    }

    #[test]
    fn class_name_all_twelve_match_spec_exactly() {
        let got: Vec<&str> = samples().iter().map(LzError::class_name).collect();
        assert_eq!(
            got,
            vec![
                "CosmosAnswerError",
                "SyntaxError",
                "NameError",
                "TypeError",
                "IndexError",
                "FieldError",
                "ZeroDivisionError",
                "OverflowError",
                "ValueError",
                "IOError",
                "AssertionError",
                "RecursionError",
            ],
            "class_name() 必须与 interface-contract.md §8.1 逐字一致"
        );
    }

    #[test]
    fn exactly_twelve_concrete_classes_and_base_never_thrown() {
        let names: Vec<&str> = samples().iter().map(LzError::class_name).collect();
        // 数量：恰好 12 个具体类。
        assert_eq!(names.len(), 12, "必须恰好 12 个具体错误类");
        // 12 个类名互不相同。
        let mut uniq = names.clone();
        uniq.sort_unstable();
        uniq.dedup();
        assert_eq!(uniq.len(), 12, "12 个类名必须互不相同");
        // 基类 LfzError 不直接抛出：无任何变体返回 "LfzError"。
        assert!(
            !names.contains(&"LfzError"),
            "基类 LfzError 不得被直接抛出"
        );
        // 明确禁止 E-xxx 编号：任何类名都不以 "E-" 开头。
        assert!(names.iter().all(|n| !n.starts_with("E-")));
    }

    #[test]
    fn messages_are_the_spec_chinese_text() {
        assert_eq!(
            LzError::CosmosAnswer { span: Span::START }.message(),
            "你忘记了宇宙的答案"
        );
        assert_eq!(
            LzError::Syntax {
                msg: SyntaxMsg::IllegalChar { c: '$' },
                span: Span::START,
            }
            .message(),
            "非法字符 '$'"
        );
        assert_eq!(
            LzError::Name {
                name: "x".to_string(),
                span: Span::START,
            }
            .message(),
            "未定义的名字 'x'"
        );
        assert_eq!(
            LzError::Type {
                msg: TypeMsg::ConditionNotBool {
                    t: "int".to_string(),
                },
                span: Span::START,
            }
            .message(),
            "条件必须是 bool，得到 int"
        );
        assert_eq!(
            LzError::Index {
                idx: 5,
                len: 3,
                span: Span::START,
            }
            .message(),
            "下标 5 越界（长度 3）"
        );
        assert_eq!(
            LzError::Field {
                name: "k".to_string(),
                span: Span::START,
            }
            .message(),
            "结构体没有字段 'k'"
        );
        assert_eq!(
            LzError::DivZero {
                modulo: false,
                span: Span::START,
            }
            .message(),
            "除以零"
        );
        assert_eq!(
            LzError::DivZero {
                modulo: true,
                span: Span::START,
            }
            .message(),
            "对零取模"
        );
        assert_eq!(
            LzError::Overflow {
                span: Span::START,
                msg: OverflowMsg::IntegerOutOfRange,
            }
            .message(),
            "整数溢出：结果超出 i64 范围"
        );
        assert_eq!(
            LzError::Value {
                msg: ValueMsg::Convert {
                    src: "string".to_string(),
                    dst: "int".to_string(),
                    text: "abc".to_string(),
                },
                span: Span::START,
            }
            .message(),
            "无法把 string 转换为 int（'abc'）"
        );
        assert_eq!(
            LzError::Value {
                msg: ValueMsg::BadFormatSpec {
                    spec: "q".to_string(),
                },
                span: Span::START,
            }
            .message(),
            "格式说明符非法：'q'"
        );
        assert_eq!(
            LzError::Io {
                msg: "输入结束（EOF）".to_string(),
                span: None,
            }
            .message(),
            "输入结束（EOF）"
        );
        // Assert 的 msg 承载已组装好的最终消息（assert 与 fail 两种形态）。
        assert_eq!(
            LzError::Assert {
                msg: "断言失败：boom".to_string(),
                span: Span::START,
            }
            .message(),
            "断言失败：boom"
        );
        assert_eq!(
            LzError::Assert {
                msg: "fail()".to_string(),
                span: Span::START,
            }
            .message(),
            "fail()"
        );
        assert_eq!(
            LzError::Recursion {
                depth: 10001,
                limit: 10000,
                span: Span::START,
            }
            .message(),
            "递归深度超限（超过 10000 层）"
        );
    }

    #[test]
    fn syntax_msg_covers_all_seventeen_rows() {
        assert_eq!(
            SyntaxMsg::IllegalChar { c: '$' }.message(),
            "非法字符 '$'"
        );
        assert_eq!(
            SyntaxMsg::HashPosition.message(),
            "'#' 只能出现在文件首行的前导位；(字符串 / 注释 / 格式说明符内的 '#' 除外)"
        );
        assert_eq!(
            SyntaxMsg::UnterminatedString.message(),
            "字符串字面量在此处未闭合"
        );
        assert_eq!(
            SyntaxMsg::UnterminatedBlockComment.message(),
            "块注释在此处未闭合（缺少 '*/'）"
        );
        assert_eq!(
            SyntaxMsg::UnexpectedToken {
                expected: "';'".to_string(),
                got: "')'".to_string(),
            }
            .message(),
            "这里期待 ';'，但得到 ')'"
        );
        assert_eq!(
            SyntaxMsg::IncompleteExpr.message(),
            "表达式未结束：行尾不能终止表达式；请用括号跨行"
        );
        assert_eq!(
            SyntaxMsg::LoneSemicolon.message(),
            "单独的 ';' 非法；打印变量请用 ';;'"
        );
        assert_eq!(SyntaxMsg::TwoStatements.message(), "语句之间必须有换行");
        assert_eq!(
            SyntaxMsg::InvalidAssignTarget.message(),
            "赋值左侧必须是变量、字段或下标"
        );
        assert_eq!(
            SyntaxMsg::PlaceholderPosition.message(),
            "占位符 '_' 只能出现在管道右侧的调用实参中"
        );
        assert_eq!(
            SyntaxMsg::PipeMultiplePlaceholder.message(),
            "管道右侧调用最多只能有一个 '_'"
        );
        assert_eq!(
            SyntaxMsg::BreakContinueOutsideLoop {
                kw: "break".to_string(),
            }
            .message(),
            "'break' 只能出现在循环体内"
        );
        assert_eq!(
            SyntaxMsg::BreakContinueOutsideLoop {
                kw: "continue".to_string(),
            }
            .message(),
            "'continue' 只能出现在循环体内"
        );
        assert_eq!(
            SyntaxMsg::ReturnOutsideFunction.message(),
            "'return' 只能出现在函数体内"
        );
        assert_eq!(
            SyntaxMsg::NotUtf8 { off: 42 }.message(),
            "文件不是合法的 UTF-8 编码（首个非法字节位于字节偏移 42）"
        );
        assert_eq!(
            SyntaxMsg::UnknownEscape { c: 'q' }.message(),
            r"字符串中不支持的转义 '\q'"
        );
        assert_eq!(
            SyntaxMsg::InterpolationNewline.message(),
            "插值表达式不能跨行；请把表达式写在一行内"
        );
        assert_eq!(
            SyntaxMsg::IntegerOutOfRange.message(),
            "整数字面量超出 i64 范围"
        );
    }

    #[test]
    fn type_msg_covers_all_seven_rows() {
        assert_eq!(
            TypeMsg::BadOperands {
                op: "+".to_string(),
                lt: "int".to_string(),
                rt: "string".to_string(),
            }
            .message(),
            "运算符 '+' 不支持 int 与 string"
        );
        assert_eq!(
            TypeMsg::ConditionNotBool {
                t: "int".to_string(),
            }
            .message(),
            "条件必须是 bool，得到 int"
        );
        assert_eq!(
            TypeMsg::NotCallable {
                t: "int".to_string(),
            }
            .message(),
            "不可调用：int 不是函数"
        );
        assert_eq!(
            TypeMsg::PipeRhsNotFunction {
                t: "int".to_string(),
            }
            .message(),
            "管道右侧必须是函数，得到 int"
        );
        assert_eq!(
            TypeMsg::ArgCount {
                name: "f".to_string(),
                n: 2,
                m: 3,
            }
            .message(),
            "函数 f 期待 2 个参数，得到 3"
        );
        assert_eq!(
            TypeMsg::FormatSpecMismatch {
                spec: "d".to_string(),
                t: "string".to_string(),
            }
            .message(),
            "格式说明符 'd' 不适用于 string"
        );
        assert_eq!(
            TypeMsg::ImmutableRebind {
                name: "a".to_string(),
            }
            .message(),
            "不能重新赋值 let 变量 'a'；let 只锁重绑定，不锁内容"
        );
        // P3.11 裁定 1：`ImmutableRebind` 消息由规范逐字符锁定，再逐字符断言一遍
        // （`semantics.md` §4.5.2 / §8.1，`DECISIONS.md` [2026-09-24 00:30]）。
        assert_chars_eq(
            &TypeMsg::ImmutableRebind {
                name: "a".to_string(),
            }
            .message(),
            "不能重新赋值 let 变量 'a'；let 只锁重绑定，不锁内容",
        );
        // 多字符 / 下划线名字须原样嵌入（不截断、不加引号）。
        assert_chars_eq(
            &TypeMsg::ImmutableRebind {
                name: "counter".to_string(),
            }
            .message(),
            "不能重新赋值 let 变量 'counter'；let 只锁重绑定，不锁内容",
        );
        assert_chars_eq(
            &TypeMsg::ImmutableRebind {
                name: "_tmp".to_string(),
            }
            .message(),
            "不能重新赋值 let 变量 '_tmp'；let 只锁重绑定，不锁内容",
        );
    }

    #[test]
    fn overflow_msg_is_single_message() {
        assert_eq!(
            OverflowMsg::IntegerOutOfRange.message(),
            "整数溢出：结果超出 i64 范围"
        );
    }

    /// 逐字符断言两个字符串完全一致（失败时指出首个不符的字符下标）。
    fn assert_chars_eq(got: &str, expected: &str) {
        let actual: Vec<char> = got.chars().collect();
        let want: Vec<char> = expected.chars().collect();
        assert_eq!(
            actual.len(),
            want.len(),
            "字符数不符：得到 {actual:?}（{got:?}），期望 {want:?}（{expected:?}）"
        );
        for (i, (a, e)) in actual.iter().zip(want.iter()).enumerate() {
            assert_eq!(
                a, e,
                "第 {i} 个字符不符：得到 {a:?}，期望 {e:?}（整串 {got:?}）"
            );
        }
    }

    #[test]
    fn value_msg_covers_all_four_rows() {
        assert_eq!(
            ValueMsg::Convert {
                src: "string".to_string(),
                dst: "int".to_string(),
                text: "abc".to_string(),
            }
            .message(),
            "无法把 string 转换为 int（'abc'）"
        );
        assert_eq!(
            ValueMsg::BadFormatSpec {
                spec: "q".to_string(),
            }
            .message(),
            "格式说明符非法：'q'"
        );
        assert_eq!(
            ValueMsg::EmptyExtremum {
                func: "min".to_string(),
            }
            .message(),
            "空数组没有极值（min）"
        );
        assert_eq!(
            ValueMsg::BadRange { lo: 3, hi: 3 }.message(),
            "区间非法：3 >= 3"
        );
    }

    /// P3.9a 裁定 1：`EmptyExtremum` 消息 `空数组没有极值（{func}）`，逐字符断言。
    #[test]
    fn empty_extremum_message_char_by_char() {
        assert_chars_eq(
            &ValueMsg::EmptyExtremum {
                func: "min".to_string(),
            }
            .message(),
            "空数组没有极值（min）",
        );
        // 其余三个函数名。
        assert_chars_eq(
            &ValueMsg::EmptyExtremum {
                func: "max".to_string(),
            }
            .message(),
            "空数组没有极值（max）",
        );
        assert_chars_eq(
            &ValueMsg::EmptyExtremum {
                func: "minBy".to_string(),
            }
            .message(),
            "空数组没有极值（minBy）",
        );
        assert_chars_eq(
            &ValueMsg::EmptyExtremum {
                func: "maxBy".to_string(),
            }
            .message(),
            "空数组没有极值（maxBy）",
        );
    }

    /// P3.9a 裁定 2：`BadRange` 消息 `区间非法：{lo} >= {hi}`，逐字符断言。
    #[test]
    fn bad_range_message_char_by_char() {
        assert_chars_eq(
            &ValueMsg::BadRange { lo: 3, hi: 3 }.message(),
            "区间非法：3 >= 3",
        );
        assert_chars_eq(
            &ValueMsg::BadRange { lo: 5, hi: 2 }.message(),
            "区间非法：5 >= 2",
        );
        // 负数与 i64 边界。
        assert_chars_eq(
            &ValueMsg::BadRange { lo: -1, hi: -4 }.message(),
            "区间非法：-1 >= -4",
        );
        assert_chars_eq(
            &ValueMsg::BadRange {
                lo: 0,
                hi: i64::MIN,
            }
            .message(),
            "区间非法：0 >= -9223372036854775808",
        );
    }

    /// 新变体仍归 `ValueError` 类（`class_name()` 映射**不改**）。
    #[test]
    fn new_value_variants_still_map_to_value_error() {
        assert_eq!(
            value(
                ValueMsg::EmptyExtremum {
                    func: "min".to_string(),
                },
                Span::START,
            )
            .class_name(),
            "ValueError"
        );
        assert_eq!(
            value(ValueMsg::BadRange { lo: 3, hi: 3 }, Span::START).class_name(),
            "ValueError"
        );
        assert_eq!(
            LzError::Value {
                msg: ValueMsg::EmptyExtremum {
                    func: "max".to_string(),
                },
                span: Span::START,
            }
            .message(),
            "空数组没有极值（max）"
        );
        assert_eq!(
            LzError::Value {
                msg: ValueMsg::BadRange { lo: 7, hi: 7 },
                span: Span::START,
            }
            .to_string(),
            "ValueError: 区间非法：7 >= 7"
        );
    }

    #[test]
    fn span_io_none_others_some() {
        for e in samples() {
            match &e {
                LzError::Io { span, .. } => assert!(span.is_none(), "Io 样本应为 None"),
                other => assert!(
                    other.span().is_some(),
                    "{} 的位置应为 Some",
                    other.class_name()
                ),
            }
        }
        // 具体位置值正确。
        assert_eq!(
            LzError::Name {
                name: "x".to_string(),
                span: Span::new(3, 5),
            }
            .span(),
            Some(Span::new(3, 5))
        );
        assert_eq!(LzError::CosmosAnswer { span: Span::START }.span(), Some(Span::START));
        // Io 带位置时返回 Some。
        let io = LzError::Io {
            msg: "无法读取：a.txt".to_string(),
            span: Some(Span::new(1, 1)),
        };
        assert_eq!(io.span(), Some(Span::new(1, 1)));
    }

    #[test]
    fn r_is_pointer_sized() {
        // R<()> = Result<(), Box<LzError>>：Box 的非空 niche 被 Result 复用 → 指针大小。
        assert_eq!(
            std::mem::size_of::<R<()>>(),
            std::mem::size_of::<usize>()
        );
    }

    #[test]
    fn display_is_class_colon_message() {
        let e = LzError::Name {
            name: "x".to_string(),
            span: Span::new(1, 1),
        };
        assert_eq!(e.to_string(), "NameError: 未定义的名字 'x'");

        let c = LzError::CosmosAnswer { span: Span::START };
        assert_eq!(c.to_string(), "CosmosAnswerError: 你忘记了宇宙的答案");

        let z = LzError::DivZero {
            modulo: true,
            span: Span::START,
        };
        assert_eq!(z.to_string(), "ZeroDivisionError: 对零取模");
    }

    #[test]
    fn cold_constructors_build_correct_class() {
        assert_eq!(cosmos_answer().class_name(), "CosmosAnswerError");
        assert_eq!(syntax(SyntaxMsg::LoneSemicolon, Span::START).class_name(), "SyntaxError");
        assert_eq!(name("x".to_string(), Span::START).class_name(), "NameError");
        assert_eq!(
            type_error(
                TypeMsg::NotCallable {
                    t: "int".to_string()
                },
                Span::START
            )
            .class_name(),
            "TypeError"
        );
        // P3.11 裁定 1：新变体 `ImmutableRebind` 仍归 `TypeError` 类（`class_name()` 映射**不改**）。
        assert_eq!(
            type_error(
                TypeMsg::ImmutableRebind {
                    name: "a".to_string()
                },
                Span::START
            )
            .class_name(),
            "TypeError"
        );
        assert_eq!(
            LzError::Type {
                msg: TypeMsg::ImmutableRebind {
                    name: "counter".to_string()
                },
                span: Span::START,
            }
            .to_string(),
            "TypeError: 不能重新赋值 let 变量 'counter'；let 只锁重绑定，不锁内容"
        );
        assert_eq!(index(9, 1, Span::START).class_name(), "IndexError");
        assert_eq!(field("k".to_string(), Span::START).class_name(), "FieldError");
        assert_eq!(div_zero(false, Span::START).class_name(), "ZeroDivisionError");
        assert_eq!(
            overflow(Span::START, OverflowMsg::IntegerOutOfRange).class_name(),
            "OverflowError"
        );
        assert_eq!(
            value(ValueMsg::BadFormatSpec { spec: "q".to_string() }, Span::START).class_name(),
            "ValueError"
        );
        assert_eq!(io("输入结束（EOF）".to_string(), None).class_name(), "IOError");
        assert_eq!(assert_fail("x".to_string(), Span::START).class_name(), "AssertionError");
        assert_eq!(recursion(10001, 10000, Span::START).class_name(), "RecursionError");
        // 通用装箱器。
        assert_eq!(
            boxed(LzError::CosmosAnswer { span: Span::START }).class_name(),
            "CosmosAnswerError"
        );
    }

    #[test]
    fn trace_frame_holds_func_id_and_span() {
        let f = TraceFrame {
            func_id: 7,
            span: Span::new(3, 5),
        };
        assert_eq!(f.func_id, 7);
        assert_eq!(f.span, Span::new(3, 5));
        // TraceFrame 不持有 Rc<str>：仅 u32 + Span（易 Copy）。
        let g = f;
        assert_eq!(g.func_id, f.func_id);
    }
}
