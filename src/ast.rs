//! AST 节点定义（每个可抛错节点至少携带起始 `Span`）。
//!
//! 契约依据：`docs/spec/interface-contract.md` §10.2（AST 必须携带源码位置）、
//! §10.3（`TraceFrame { func_id, span }`）、§10.5 / §10.6（`;;` → `Dump` 节点、
//! 管道脱糖为 `Call`）；语法依据：`docs/spec/syntax.md` §3–§7。
//!
//! 本模块**只定义类型**：不含任何解析 / 求值逻辑（那是 `parser.rs` /
//! `evaluator.rs` 的职责）。`ast.rs` 是 parser 与 evaluator 之间的**唯一交接物**。
//!
//! # 位置信息承载方式（本模块的选择）
//!
//! 采用 **[`Spanned<T>`] 泛型包装** + **辅助节点内嵌 `span` 字段** 的混合方式：
//!
//! - 两个递归大枚举 [`Expr`] / [`Stmt`] 分别是 `Spanned<ExprKind>` /
//!   `Spanned<StmtKind>` —— 位置与「种类」解耦、构造点唯一（`Spanned::new`），
//!   将来若要补 `end` 只需改 [`Spanned`] 一处；
//! - 结构与位点强绑定的辅助节点（[`Block`] / [`FnDecl`] / [`StructDecl`] /
//!   [`FieldInit`] / [`Lvalue`] / [`LvalueSeg`]）**内嵌 `pub span: Span`** ——
//!   避免「`Spanned<Block>` 里套 `Vec<Stmt>`」这种读起来费劲的双层包装。
//!
//! 共同保证：**每个可抛错节点都能通过 `.span` 直接拿到起始 `Span`**
//! （`Expr`/`Stmt` 用 `.span` 字段，辅助节点用其内嵌 `span` 字段）。
//!
//! # 与 spec 的两点实现说明
//!
//! 1. **无 `Pipe` 节点**：`syntax.md` §4.3 与 `interface-contract.md` §10.6
//!    规定管道在**解析期脱糖**（`L |> F(a)` → `Call`），运行时零开销，故本
//!    AST **不保留**脱糖前的管道节点。
//! 2. **整数字面量存 `i64`**：`INT` 记号承载原文（B12），由 parser 按
//!    §10.8 规则解析（含 `-9223372036854775808` → `i64::MIN` 特判），AST
//!    直接存**已解析的 `i64`**，供 evaluator 零转换消费。

use crate::env::ScopeId;
use crate::span::Span;

// ===========================================================================
// 位置包装
// ===========================================================================

/// 携带起始位置的节点包装：`node` 是节点本体，`span` 是其起始位置（§10.2）。
#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<T> {
    /// 节点本体。
    pub node: T,
    /// 节点起始位置（1-based 行 / 列）。
    pub span: Span,
}

impl<T> Spanned<T> {
    /// 用节点本体与其起始位置构造一个带位置的节点。
    #[must_use]
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    /// 取得起始位置（与 `.span` 字段等价的阅读友好写法）。
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
}

// ===========================================================================
// 程序
// ===========================================================================

/// 一个编译单元（`syntax.md` §7：`program = { NEWLINE | statement } , EOF`）。
///
/// 前导 `#42` 由 loader 消费，**parser 不可见**（M2），故 `Program` 不含前导。
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    /// 起始位置（第一个语句 / 文件末尾的位置）。
    pub span: Span,
    /// 顶层语句序列。
    pub stmts: Vec<Stmt>,
}

// ===========================================================================
// 语句（`syntax.md` §3 / §7）
// ===========================================================================

/// 一条语句 = `StmtKind` + 起始 [`Span`]。
pub type Stmt = Spanned<StmtKind>;

/// 语句种类（对照 `syntax.md` §7 的 `statement` 产生式）。
#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    /// 变量声明 `let x = expr` / `var x = expr`（`decl_stmt`）。
    ///
    /// `mutable`：`let` → `false`，`var` → `true`。
    Decl {
        /// 是否可变绑定（`var` = `true`）。
        mutable: bool,
        /// 绑定名（`IDENT`；`_` 不合法，见 A24）。
        name: String,
        /// 初始化表达式（文法要求必有 `=` 初值）。
        init: Expr,
    },

    /// 赋值 `lvalue op= expr`（`assign_stmt`；`op` ∈ `= += -= *= /= %=`）。
    Assign {
        /// 赋值目标（受限 lvalue 形式）。
        target: Lvalue,
        /// 赋值运算符。
        op: AssignOp,
        /// 右值表达式。
        value: Expr,
    },

    /// 命名函数声明 `fn name(params) body`（`fn_decl`）。
    FnDecl(FnDecl),

    /// 结构体模板声明 `struct Name { ... }`（`struct_decl`）。
    StructDecl(StructDecl),

    /// 条件语句 `if cond block [else ...]`（`if_expr` 的语句用法）。
    ///
    /// 注意：`if` 本身是**表达式**（有意为值）；此处仅作为语句出现。
    If(Expr),

    /// `while cond block`（`while_stmt`）。
    While {
        /// 循环条件。
        cond: Expr,
        /// 循环体块。
        body: Block,
    },

    /// `for var in iter block`（`for_stmt`）。
    For {
        /// 迭代变量名（`IDENT`）。
        var: String,
        /// 可迭代表达式。
        iter: Expr,
        /// 循环体块。
        body: Block,
    },

    /// `return [expr]`（`return_stmt`；行尾无表达式即返回 `nil`，A19）。
    Return(Option<Expr>),

    /// `break`（`break_stmt`）。
    Break,

    /// `continue`（`continue_stmt`）。
    Continue,

    /// `;;` 变量 dump（`dump_stmt`；独占逻辑行，A10–A12）。
    ///
    /// `scope` 为编译期决议的**最内层活动作用域** id（`interface-contract.md`
    /// §10.5；字段类型照 runtime-dev P3.6 ADR 取 `crate::env::ScopeId`）。
    Dump {
        /// 最内层活动作用域的 id（沿 `parent` 链内→外遍历）。
        scope: ScopeId,
    },

    /// 表达式语句（`expr_stmt`）。
    Expr(Expr),
}

/// 赋值运算符（`syntax.md` §7：`assign_op`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignOp {
    /// `=`
    Assign,
    /// `+=`
    AddAssign,
    /// `-=`
    SubAssign,
    /// `*=`
    MulAssign,
    /// `/=`
    DivAssign,
    /// `%=`
    RemAssign,
}

/// 赋值目标（`syntax.md` §7：`lvalue = IDENT , { "." IDENT | "[" expression "]" }`）。
///
/// 基座只可能是 `IDENT` 或 `self`（A21：表达式后接 `assign_op` 时左侧须为 lvalue）。
#[derive(Clone, Debug, PartialEq)]
pub struct Lvalue {
    /// 起始位置（基座 token）。
    pub span: Span,
    /// 基座：名字或 `self`。
    pub base: LvalueBase,
    /// 后缀访问链（字段 / 下标，按出现顺序）。
    pub path: Vec<LvalueSeg>,
}

/// lvalue 基座。
#[derive(Clone, Debug, PartialEq)]
pub enum LvalueBase {
    /// 标识符 `name`。
    Name(String),
    /// `self`。
    SelfValue,
}

/// lvalue 后缀链的一节。
#[derive(Clone, Debug, PartialEq)]
pub struct LvalueSeg {
    /// 该节的起始位置（`.` 或 `[`）。
    pub span: Span,
    /// 该节的种类。
    pub kind: LvalueSegKind,
}

/// lvalue 后缀链种类。
#[derive(Clone, Debug, PartialEq)]
pub enum LvalueSegKind {
    /// `.name`
    Field(String),
    /// `[expr]`
    Index(Expr),
}

/// 命名函数声明（`fn_decl`）。
#[derive(Clone, Debug, PartialEq)]
pub struct FnDecl {
    /// 起始位置（`fn` 关键字）。
    pub span: Span,
    /// 函数名。
    pub name: String,
    /// 形参名列表（`params = IDENT , { "," IDENT }`）。
    pub params: Vec<String>,
    /// 函数体（`block` 或 `=> (block | expression)`）。
    pub body: Body,
}

/// 结构体模板声明（`struct_decl`）。
#[derive(Clone, Debug, PartialEq)]
pub struct StructDecl {
    /// 起始位置（`struct` 关键字）。
    pub span: Span,
    /// 模板名。
    pub name: String,
    /// 成员列表（字段初始化 + 方法，按声明序）。
    pub members: Vec<StructMember>,
}

/// 结构体模板成员（`member = fn_decl | IDENT ":" expression`）。
#[derive(Clone, Debug, PartialEq)]
pub enum StructMember {
    /// 方法（`fn ...`）。
    Method(FnDecl),
    /// 数据字段初始化（`name: expr`）。
    Field(FieldInit),
}

/// 字段初始化（`field_init`；也用于 struct 字面量的成员表）。
///
/// `name` 可来自 `IDENT` 或字符串字面量（`syntax.md` §7：`( IDENT | STRING )`）。
#[derive(Clone, Debug, PartialEq)]
pub struct FieldInit {
    /// 起始位置（键 token）。
    pub span: Span,
    /// 字段名（已解码的字符串内容）。
    pub name: String,
    /// 字段初值表达式。
    pub value: Expr,
}

/// 块 `{ ... }`（`syntax.md` §7：`block`）。
///
/// 块是**值**（值 = 最后一条表达式的值，见 §1）；仅出现在 block-required 位。
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    /// 起始位置（`{`）。
    pub span: Span,
    /// 块内语句序列。
    pub stmts: Vec<Stmt>,
}

/// 函数 / lambda 体（`syntax.md` §7：`body = block | "=>" , ( block | expression )`）。
///
/// M3：`=>` 后紧跟 `{` 恒为块；要返回 struct 字面量须写 `=> ({ ... })`。
#[derive(Clone, Debug, PartialEq)]
pub enum Body {
    /// `{ ... }` 或 `=> { ... }`。
    Block(Block),
    /// `=> expr`。
    Expr(Expr),
}

// ===========================================================================
// 表达式（`syntax.md` §4 / §5 / §7）
// ===========================================================================

/// 一个表达式 = `ExprKind` + 起始 [`Span`]。
pub type Expr = Spanned<ExprKind>;

/// 表达式种类（对照 `syntax.md` §7 的表达式产生式）。
#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    /// 整数字面量（已解析为 `i64`，含 `i64::MIN` 特判，见 §10.8）。
    Int(i64),
    /// 浮点字面量。
    Float(f64),
    /// 纯字符串字面量（不含 `${}` 插值）。
    Str(String),
    /// 富字符串（含至少一个 `${}` 插值段，§2.8）。
    Interp(InterpString),
    /// 布尔字面量。
    Bool(bool),
    /// `nil`。
    Nil,

    /// 标识符（变量 / 函数名）。
    Ident(String),
    /// `self`。
    SelfRef,

    /// 数组字面量 `[a, b, c]`（`array_lit`）。
    Array(Vec<Expr>),
    /// 结构体字面量 `Name { k: v }` 或匿名 `{ k: v }`（`struct_lit`）。
    StructLit(StructLit),

    /// 字段访问 `object.name`（`field`）。
    Field {
        /// 被访问对象。
        object: Box<Expr>,
        /// 字段名。
        name: String,
    },
    /// 下标 `object[index]`（`index`）。
    Index {
        /// 被索引对象。
        object: Box<Expr>,
        /// 下标表达式。
        index: Box<Expr>,
    },
    /// 调用 `callee(args)`（`call`；**管道脱糖后亦生成此节点**，§4.3 / §10.6）。
    Call {
        /// 被调用者。
        callee: Box<Expr>,
        /// 实参列表。
        args: Vec<Expr>,
    },

    /// 一元运算 `-x` / `!x`（`unary`）。
    Unary {
        /// 运算符。
        op: UnaryOp,
        /// 操作数。
        operand: Box<Expr>,
    },
    /// 二元运算（算术 / 比较 / 相等；`syntax.md` §4.1）。
    Binary {
        /// 运算符。
        op: BinaryOp,
        /// 左操作数。
        left: Box<Expr>,
        /// 右操作数。
        right: Box<Expr>,
    },
    /// 短路逻辑运算 `&&` / `||`。
    Logical {
        /// 运算符。
        op: LogicalOp,
        /// 左操作数。
        left: Box<Expr>,
        /// 右操作数。
        right: Box<Expr>,
    },

    /// `if cond { ... } [else ...]`（`if_expr`）。
    If(IfExpr),
    /// 函数 / 闭包字面量（`lambda`）。
    ///
    /// 用 `Box` 断开 `Body → Expr → ExprKind → Lambda → Body` 的递归环。
    Lambda(Box<Lambda>),
}

/// 富字符串插值（`syntax.md` §2.8：分段 = 文本 / 表达式）。
#[derive(Clone, Debug, PartialEq)]
pub struct InterpString {
    /// 有序分段（文本段与表达式段交替，可多段）。
    pub parts: Vec<StrPart>,
}

/// 富字符串的一段。
#[derive(Clone, Debug, PartialEq)]
pub enum StrPart {
    /// 普通文本段（转义已在词法期解码）。
    Text(String),
    /// 插值表达式段 `${expr[:format_spec]}`。
    Expr {
        /// 插值表达式。
        expr: Expr,
        /// 格式说明符原始文本；**无 `:` 时为 `None`**（M6）。
        format_spec: Option<String>,
    },
}

/// 结构体字面量（`syntax.md` §7：`struct_lit = [ IDENT ] , "{" , [ field_list ] , "}"`）。
#[derive(Clone, Debug, PartialEq)]
pub struct StructLit {
    /// 可选模板名（`Some(name)` = `Name { ... }`；`None` = 匿名 `{ ... }`）。
    pub type_name: Option<String>,
    /// 字段初始化列表（按书写顺序）。
    pub fields: Vec<FieldInit>,
}

/// `if_expr`（`syntax.md` §7）。
#[derive(Clone, Debug, PartialEq)]
pub struct IfExpr {
    /// 条件表达式。
    pub cond: Box<Expr>,
    /// 条件为真时的块。
    pub then_block: Block,
    /// 可选 `else` 分支。
    pub else_branch: Option<ElseBranch>,
}

/// `else` 分支（`else if ...` 或 `else { ... }`）。
#[derive(Clone, Debug, PartialEq)]
pub enum ElseBranch {
    /// `else if ...`（嵌套 if 表达式）。
    If(Box<Expr>),
    /// `else { ... }`。
    Block(Block),
}

/// 函数 / 闭包字面量（`syntax.md` §7：`lambda`，两种书写形式）。
#[derive(Clone, Debug, PartialEq)]
pub struct Lambda {
    /// 形参名列表。
    pub params: Vec<String>,
    /// 函数体（`fn(...) { ... }` 或 `(...) => ...`）。
    pub body: Body,
}

/// 一元前缀运算符（`syntax.md` §4.1 级别 2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    /// `-`（取负）。
    Neg,
    /// `!`（逻辑非）。
    Not,
}

/// 二元中缀运算符（算术 / 比较 / 相等；`syntax.md` §4.1）。
///
/// 短路逻辑 `&&` / `||` 不在此列，见 [`LogicalOp`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    /// `+`（加 / 字符串连接）。
    Add,
    /// `-`（减）。
    Sub,
    /// `*`（乘 / 字符串重复）。
    Mul,
    /// `/`（真除法，返回 `float`，A3）。
    Div,
    /// `%`（Python 取模，A3）。
    Rem,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `==`
    Eq,
    /// `!=`
    Ne,
}

/// 短路逻辑运算符（`syntax.md` §4.1 级别 8 / 9）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicalOp {
    /// `&&`（短路与）。
    And,
    /// `||`（短路或）。
    Or,
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sp(line: u32, col: u32) -> Span {
        Span::new(line, col)
    }

    fn int(n: i64, line: u32, col: u32) -> Expr {
        Spanned::new(ExprKind::Int(n), sp(line, col))
    }

    #[test]
    fn spanned_carries_node_and_span() {
        let e = Spanned::new(ExprKind::Bool(true), sp(4, 7));
        assert_eq!(e.span, sp(4, 7));
        assert_eq!(e.span(), sp(4, 7));
        assert_eq!(e.node, ExprKind::Bool(true));
        assert_eq!(Spanned::new(1u8, Span::START).span, Span::START);
    }

    #[test]
    fn literals_cover_all_five_kinds() {
        let ints = Spanned::new(ExprKind::Int(-9223372036854775808i64), sp(1, 1));
        let floats = Spanned::new(ExprKind::Float(2.5e-3), sp(1, 2));
        let strs = Spanned::new(ExprKind::Str("hi".into()), sp(1, 3));
        let bools = Spanned::new(ExprKind::Bool(false), sp(1, 4));
        let nils = Spanned::new(ExprKind::Nil, sp(1, 5));

        assert_eq!(ints.node, ExprKind::Int(i64::MIN));
        assert_eq!(floats.node, ExprKind::Float(2.5e-3));
        assert_eq!(strs.node, ExprKind::Str("hi".into()));
        assert_eq!(bools.node, ExprKind::Bool(false));
        assert_eq!(nils.node, ExprKind::Nil);
        for e in [ints, floats, strs, bools, nils] {
            assert_eq!(e.span().line, 1); // 每个字面量都可取 span
        }
    }

    #[test]
    fn plain_string_vs_interp_string() {
        let plain = Spanned::new(ExprKind::Str("join".into()), sp(2, 1));
        assert_eq!(plain.node, ExprKind::Str("join".into()));

        let rich = Spanned::new(
            ExprKind::Interp(InterpString {
                parts: vec![
                    StrPart::Text("x=".into()),
                    StrPart::Expr {
                        expr: Spanned::new(ExprKind::Ident("x".into()), sp(3, 5)),
                        format_spec: Some(">3".into()),
                    },
                    StrPart::Text("!".into()),
                ],
            }),
            sp(3, 1),
        );
        match rich.node {
            ExprKind::Interp(InterpString { parts }) => {
                assert_eq!(parts.len(), 3);
                match &parts[1] {
                    StrPart::Expr { format_spec, .. } => {
                        assert_eq!(format_spec.as_deref(), Some(">3"));
                    }
                    _ => panic!("第二段应为表达式段"),
                }
            }
            _ => panic!("应为插值字符串"),
        }
    }

    #[test]
    fn interp_format_spec_is_none_without_colon() {
        let e = Spanned::new(
            ExprKind::Interp(InterpString {
                parts: vec![StrPart::Expr {
                    expr: int(1, 1, 5),
                    format_spec: None,
                }],
            }),
            sp(1, 1),
        );
        match e.node {
            ExprKind::Interp(InterpString { parts }) => match &parts[0] {
                StrPart::Expr { format_spec, .. } => assert_eq!(*format_spec, None),
                _ => panic!("应为表达式段"),
            },
            _ => panic!("应为插值字符串"),
        }
    }

    #[test]
    fn postfix_call_index_field() {
        let call = Spanned::new(
            ExprKind::Call {
                callee: Box::new(Spanned::new(ExprKind::Ident("f".into()), sp(1, 1))),
                args: vec![int(1, 1, 3), int(2, 1, 6)],
            },
            sp(1, 1),
        );
        let index = Spanned::new(
            ExprKind::Index {
                object: Box::new(Spanned::new(ExprKind::Ident("xs".into()), sp(2, 1))),
                index: Box::new(int(0, 2, 4)),
            },
            sp(2, 1),
        );
        let field = Spanned::new(
            ExprKind::Field {
                object: Box::new(Spanned::new(ExprKind::SelfRef, sp(3, 1))),
                name: "score".into(),
            },
            sp(3, 1),
        );

        assert!(matches!(call.node, ExprKind::Call { .. }));
        assert!(matches!(index.node, ExprKind::Index { .. }));
        assert!(matches!(field.node, ExprKind::Field { .. }));
        assert_eq!(call.span, sp(1, 1));
        assert_eq!(index.span, sp(2, 1));
        assert_eq!(field.span, sp(3, 1));
    }

    #[test]
    fn unary_binary_logical_nodes() {
        let neg = Spanned::new(
            ExprKind::Unary {
                op: UnaryOp::Neg,
                operand: Box::new(int(3, 1, 2)),
            },
            sp(1, 1),
        );
        let not = Spanned::new(
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Spanned::new(ExprKind::Bool(true), sp(2, 2))),
            },
            sp(2, 1),
        );
        let add = Spanned::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(int(1, 3, 1)),
                right: Box::new(int(2, 3, 5)),
            },
            sp(3, 1),
        );
        let cmp = Spanned::new(
            ExprKind::Binary {
                op: BinaryOp::Ge,
                left: Box::new(int(1, 4, 1)),
                right: Box::new(int(2, 4, 6)),
            },
            sp(4, 1),
        );
        let eq = Spanned::new(
            ExprKind::Binary {
                op: BinaryOp::Eq,
                left: Box::new(int(1, 5, 1)),
                right: Box::new(int(1, 5, 6)),
            },
            sp(5, 1),
        );
        let and = Spanned::new(
            ExprKind::Logical {
                op: LogicalOp::And,
                left: Box::new(Spanned::new(ExprKind::Bool(true), sp(6, 1))),
                right: Box::new(Spanned::new(ExprKind::Bool(false), sp(6, 8))),
            },
            sp(6, 1),
        );
        let or = Spanned::new(
            ExprKind::Logical {
                op: LogicalOp::Or,
                left: Box::new(Spanned::new(ExprKind::Bool(true), sp(7, 1))),
                right: Box::new(Spanned::new(ExprKind::Bool(false), sp(7, 8))),
            },
            sp(7, 1),
        );

        assert_eq!(neg.node, ExprKind::Unary { op: UnaryOp::Neg, operand: Box::new(int(3, 1, 2)) });
        assert!(matches!(not.node, ExprKind::Unary { op: UnaryOp::Not, .. }));
        assert!(matches!(add.node, ExprKind::Binary { op: BinaryOp::Add, .. }));
        assert!(matches!(cmp.node, ExprKind::Binary { op: BinaryOp::Ge, .. }));
        assert!(matches!(eq.node, ExprKind::Binary { op: BinaryOp::Eq, .. }));
        assert!(matches!(and.node, ExprKind::Logical { op: LogicalOp::And, .. }));
        assert!(matches!(or.node, ExprKind::Logical { op: LogicalOp::Or, .. }));
    }

    #[test]
    fn binary_op_covers_all_eleven() {
        let all = [
            BinaryOp::Add,
            BinaryOp::Sub,
            BinaryOp::Mul,
            BinaryOp::Div,
            BinaryOp::Rem,
            BinaryOp::Lt,
            BinaryOp::Le,
            BinaryOp::Gt,
            BinaryOp::Ge,
            BinaryOp::Eq,
            BinaryOp::Ne,
        ];
        assert_eq!(all.len(), 11);
    }

    #[test]
    fn array_and_struct_literals() {
        let arr = Spanned::new(
            ExprKind::Array(vec![int(1, 1, 2), int(2, 1, 5), int(3, 1, 8)]),
            sp(1, 1),
        );
        let named = Spanned::new(
            ExprKind::StructLit(StructLit {
                type_name: Some("Student".into()),
                fields: vec![
                    FieldInit {
                        span: sp(2, 15),
                        name: "name".into(),
                        value: Spanned::new(ExprKind::Str("Alice".into()), sp(2, 21)),
                    },
                    FieldInit {
                        span: sp(2, 30),
                        name: "score".into(),
                        value: int(93, 2, 38),
                    },
                ],
            }),
            sp(2, 1),
        );
        let anon = Spanned::new(
            ExprKind::StructLit(StructLit {
                type_name: None,
                fields: vec![FieldInit {
                    span: sp(3, 2),
                    name: "k".into(),
                    value: int(1, 3, 5),
                }],
            }),
            sp(3, 1),
        );

        match arr.node {
            ExprKind::Array(xs) => assert_eq!(xs.len(), 3),
            _ => panic!("应为数组字面量"),
        }
        match named.node {
            ExprKind::StructLit(StructLit { type_name, fields }) => {
                assert_eq!(type_name.as_deref(), Some("Student"));
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[1].span, sp(2, 30)); // 字段初始化可取 span
            }
            _ => panic!("应为具名 struct 字面量"),
        }
        assert!(matches!(
            anon.node,
            ExprKind::StructLit(StructLit { type_name: None, .. })
        ));
    }

    #[test]
    fn if_expression_with_else_if_and_else_block() {
        let then_block = Block {
            span: sp(1, 10),
            stmts: vec![Spanned::new(StmtKind::Expr(int(1, 1, 12)), sp(1, 12))],
        };
        let else_if = Spanned::new(
            ExprKind::If(IfExpr {
                cond: Box::new(Spanned::new(ExprKind::Ident("c2".into()), sp(1, 30))),
                then_block: Block {
                    span: sp(1, 36),
                    stmts: vec![Spanned::new(StmtKind::Expr(int(2, 1, 38)), sp(1, 38))],
                },
                else_branch: Some(ElseBranch::Block(Block {
                    span: sp(1, 50),
                    stmts: vec![Spanned::new(StmtKind::Expr(int(3, 1, 52)), sp(1, 52))],
                })),
            }),
            sp(1, 25),
        );
        let outer = Spanned::new(
            ExprKind::If(IfExpr {
                cond: Box::new(Spanned::new(ExprKind::Ident("c1".into()), sp(1, 4))),
                then_block,
                else_branch: Some(ElseBranch::If(Box::new(else_if))),
            }),
            sp(1, 1),
        );

        match outer.node {
            ExprKind::If(IfExpr { else_branch, .. }) => match else_branch {
                Some(ElseBranch::If(inner)) => {
                    assert!(matches!(inner.node, ExprKind::If(_)));
                    assert_eq!(inner.span, sp(1, 25));
                }
                _ => panic!("应为 else if"),
            },
            _ => panic!("应为 if 表达式"),
        }
    }

    #[test]
    fn if_without_else_has_none_branch() {
        let e = Spanned::new(
            ExprKind::If(IfExpr {
                cond: Box::new(Spanned::new(ExprKind::Ident("c".into()), sp(1, 4))),
                then_block: Block {
                    span: sp(1, 6),
                    stmts: vec![],
                },
                else_branch: None,
            }),
            sp(1, 1),
        );
        match e.node {
            ExprKind::If(IfExpr { else_branch, .. }) => assert_eq!(else_branch, None),
            _ => panic!("应为 if 表达式"),
        }
    }

    #[test]
    fn lambda_both_forms() {
        // fn (x) { ... } 形式
        let anon_fn = Spanned::new(
            ExprKind::Lambda(Box::new(Lambda {
                params: vec!["x".into()],
                body: Body::Block(Block {
                    span: sp(1, 9),
                    stmts: vec![Spanned::new(
                        StmtKind::Expr(Spanned::new(ExprKind::Ident("x".into()), sp(1, 11))),
                        sp(1, 11),
                    )],
                }),
            })),
            sp(1, 1),
        );
        // (s) => s.score 形式
        let arrow = Spanned::new(
            ExprKind::Lambda(Box::new(Lambda {
                params: vec!["s".into()],
                body: Body::Expr(Spanned::new(
                    ExprKind::Field {
                        object: Box::new(Spanned::new(ExprKind::Ident("s".into()), sp(2, 8))),
                        name: "score".into(),
                    },
                    sp(2, 8),
                )),
            })),
            sp(2, 1),
        );
        // (params) => { ... } 形式（M3：块体）
        let arrow_block = Spanned::new(
            ExprKind::Lambda(Box::new(Lambda {
                params: vec!["a".into(), "b".into()],
                body: Body::Block(Block {
                    span: sp(3, 12),
                    stmts: vec![Spanned::new(
                        StmtKind::Expr(Spanned::new(
                            ExprKind::Binary {
                                op: BinaryOp::Add,
                                left: Box::new(Spanned::new(ExprKind::Ident("a".into()), sp(3, 14))),
                                right: Box::new(Spanned::new(ExprKind::Ident("b".into()), sp(3, 18))),
                            },
                            sp(3, 14),
                        )),
                        sp(3, 14),
                    )],
                }),
            })),
            sp(3, 1),
        );

        for e in [anon_fn, arrow, arrow_block] {
            match e.node {
                ExprKind::Lambda(ref l) => assert!(!l.params.is_empty()),
                _ => panic!("应为 lambda"),
            }
            assert!(e.span().line >= 1);
        }
    }

    #[test]
    fn all_statement_kinds_constructible() {
        let ident = |n: &str, l: u32, c: u32| Spanned::new(ExprKind::Ident(n.into()), sp(l, c));

        let stmts: Vec<Stmt> = vec![
            // let
            Spanned::new(
                StmtKind::Decl {
                    mutable: false,
                    name: "x".into(),
                    init: int(1, 1, 9),
                },
                sp(1, 1),
            ),
            // var
            Spanned::new(
                StmtKind::Decl {
                    mutable: true,
                    name: "y".into(),
                    init: int(2, 2, 9),
                },
                sp(2, 1),
            ),
            // assign with each assign op
            Spanned::new(
                StmtKind::Assign {
                    target: Lvalue {
                        span: sp(3, 1),
                        base: LvalueBase::Name("x".into()),
                        path: vec![],
                    },
                    op: AssignOp::Assign,
                    value: int(3, 3, 5),
                },
                sp(3, 1),
            ),
            Spanned::new(
                StmtKind::Assign {
                    target: Lvalue {
                        span: sp(4, 1),
                        base: LvalueBase::SelfValue,
                        path: vec![
                            LvalueSeg {
                                span: sp(4, 5),
                                kind: LvalueSegKind::Field("count".into()),
                            },
                            LvalueSeg {
                                span: sp(4, 11),
                                kind: LvalueSegKind::Index(int(0, 4, 12)),
                            },
                        ],
                    },
                    op: AssignOp::AddAssign,
                    value: int(1, 4, 15),
                },
                sp(4, 1),
            ),
            Spanned::new(
                StmtKind::Assign {
                    target: Lvalue {
                        span: sp(5, 1),
                        base: LvalueBase::Name("y".into()),
                        path: vec![],
                    },
                    op: AssignOp::SubAssign,
                    value: int(1, 5, 6),
                },
                sp(5, 1),
            ),
            Spanned::new(
                StmtKind::Assign {
                    target: Lvalue {
                        span: sp(6, 1),
                        base: LvalueBase::Name("y".into()),
                        path: vec![],
                    },
                    op: AssignOp::MulAssign,
                    value: int(2, 6, 6),
                },
                sp(6, 1),
            ),
            Spanned::new(
                StmtKind::Assign {
                    target: Lvalue {
                        span: sp(7, 1),
                        base: LvalueBase::Name("y".into()),
                        path: vec![],
                    },
                    op: AssignOp::DivAssign,
                    value: int(2, 7, 6),
                },
                sp(7, 1),
            ),
            Spanned::new(
                StmtKind::Assign {
                    target: Lvalue {
                        span: sp(8, 1),
                        base: LvalueBase::Name("y".into()),
                        path: vec![],
                    },
                    op: AssignOp::RemAssign,
                    value: int(2, 8, 6),
                },
                sp(8, 1),
            ),
            // fn decl
            Spanned::new(
                StmtKind::FnDecl(FnDecl {
                    span: sp(9, 1),
                    name: "f".into(),
                    params: vec!["a".into(), "b".into()],
                    body: Body::Expr(ident("a", 9, 20)),
                }),
                sp(9, 1),
            ),
            // struct decl (with a method + a field)
            Spanned::new(
                StmtKind::StructDecl(StructDecl {
                    span: sp(10, 1),
                    name: "Student".into(),
                    members: vec![
                        StructMember::Field(FieldInit {
                            span: sp(11, 5),
                            name: "name".into(),
                            value: Spanned::new(ExprKind::Str(String::new()), sp(11, 11)),
                        }),
                        StructMember::Method(FnDecl {
                            span: sp(12, 5),
                            name: "isTop".into(),
                            params: vec![],
                            body: Body::Expr(Spanned::new(ExprKind::Bool(true), sp(12, 20))),
                        }),
                    ],
                }),
                sp(10, 1),
            ),
            // if statement
            Spanned::new(
                StmtKind::If(Spanned::new(
                    ExprKind::If(IfExpr {
                        cond: Box::new(ident("c", 13, 4)),
                        then_block: Block {
                            span: sp(13, 6),
                            stmts: vec![],
                        },
                        else_branch: None,
                    }),
                    sp(13, 1),
                )),
                sp(13, 1),
            ),
            // while
            Spanned::new(
                StmtKind::While {
                    cond: ident("c", 14, 7),
                    body: Block {
                        span: sp(14, 9),
                        stmts: vec![],
                    },
                },
                sp(14, 1),
            ),
            // for
            Spanned::new(
                StmtKind::For {
                    var: "i".into(),
                    iter: ident("xs", 15, 10),
                    body: Block {
                        span: sp(15, 13),
                        stmts: vec![],
                    },
                },
                sp(15, 1),
            ),
            // return with / without value
            Spanned::new(StmtKind::Return(Some(int(0, 16, 8))), sp(16, 1)),
            Spanned::new(StmtKind::Return(None), sp(17, 1)),
            // break / continue
            Spanned::new(StmtKind::Break, sp(18, 1)),
            Spanned::new(StmtKind::Continue, sp(19, 1)),
            // dump (;;) — scope field uses crate::env::ScopeId
            Spanned::new(StmtKind::Dump { scope: ScopeId(7) }, sp(20, 1)),
            // expr stmt
            Spanned::new(StmtKind::Expr(ident("z", 21, 1)), sp(21, 1)),
        ];

        assert_eq!(stmts.len(), 19);
        // 每个语句都能取到 span
        for s in &stmts {
            assert!(s.span().line >= 1);
        }
        assert_eq!(stmts[0].span, sp(1, 1));
        assert_eq!(stmts[18].span, sp(21, 1));
    }

    #[test]
    fn dump_scope_uses_env_scope_id() {
        let dump = Spanned::new(StmtKind::Dump { scope: ScopeId(42) }, sp(1, 1));
        match dump.node {
            StmtKind::Dump { scope } => assert_eq!(scope, ScopeId(42)),
            _ => panic!("应为 Dump"),
        }
        assert_eq!(dump.span, span_of(&dump));
    }

    fn span_of<T>(n: &Spanned<T>) -> Span {
        n.span
    }

    #[test]
    fn pipe_is_desugared_to_call_no_pipe_node() {
        // `xs |> sum()` 脱糖 → Call { callee: Ident("sum"), args: [Ident("xs")] }。
        // 本 AST 不含 Pipe 变体：此处验证脱糖后的 Call 结构可被完整表达。
        let desugared = Spanned::new(
            ExprKind::Call {
                callee: Box::new(Spanned::new(ExprKind::Ident("sum".into()), sp(1, 7))),
                args: vec![Spanned::new(ExprKind::Ident("xs".into()), sp(1, 1))],
            },
            sp(1, 1),
        );
        match desugared.node {
            ExprKind::Call { callee, args } => {
                assert!(matches!(callee.node, ExprKind::Ident(ref n) if n == "sum"));
                assert_eq!(args.len(), 1);
            }
            _ => panic!("应为脱糖后的 Call"),
        }
    }

    #[test]
    fn program_aggregates_statements() {
        let program = Program {
            span: sp(2, 1),
            stmts: vec![
                Spanned::new(
                    StmtKind::Decl {
                        mutable: false,
                        name: "x".into(),
                        init: int(1, 2, 9),
                    },
                    sp(2, 1),
                ),
                Spanned::new(StmtKind::Dump { scope: ScopeId(0) }, sp(3, 1)),
            ],
        };
        assert_eq!(program.stmts.len(), 2);
        assert_eq!(program.span, sp(2, 1));
        assert_eq!(program.stmts[1].span, sp(3, 1));
    }

    #[test]
    fn block_and_body_variants() {
        let block = Block {
            span: sp(1, 1),
            stmts: vec![Spanned::new(StmtKind::Break, sp(1, 3))],
        };
        let b1 = Body::Block(block.clone());
        let b2 = Body::Expr(int(1, 2, 1));
        assert_eq!(b1, Body::Block(block.clone()));
        assert!(matches!(b2, Body::Expr(_)));
        // Block 内嵌 span 可获取
        assert_eq!(block.span, sp(1, 1));    }

    #[test]
    fn field_init_span_and_name_accessible() {
        let f = FieldInit {
            span: sp(5, 9),
            name: "\"quoted\"".into(),
            value: int(1, 5, 20),
        };
        assert_eq!(f.span, sp(5, 9));
        assert_eq!(f.name, "\"quoted\"");
        assert_eq!(f.value.span, sp(5, 20));
    }

    #[test]
    fn struct_member_both_variants_carry_spans() {
        let method = StructMember::Method(FnDecl {
            span: sp(10, 5),
            name: "f".into(),
            params: vec![],
            body: Body::Expr(int(0, 10, 15)),
        });
        let field = StructMember::Field(FieldInit {
            span: sp(11, 5),
            name: "k".into(),
            value: int(1, 11, 8),
        });
        match method {
            StructMember::Method(fn_decl) => assert_eq!(fn_decl.span, sp(10, 5)),
            _ => panic!("应为方法成员"),
        }
        match field {
            StructMember::Field(fi) => assert_eq!(fi.span, sp(11, 5)),
            _ => panic!("应为字段成员"),
        }
    }

    #[test]
    fn clone_and_partial_eq_derive_work() {
        let a = int(5, 1, 1);
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(a, int(5, 1, 2)); // span 不同 → 不等
        assert_ne!(a, int(6, 1, 1)); // 值不同 → 不等
    }
}
