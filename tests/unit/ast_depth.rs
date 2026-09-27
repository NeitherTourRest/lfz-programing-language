//! §3.9 AST 深度上限（v1.1 补钉）——边界与栈安全。
//!
//! 规范：`docs/spec/syntax.md` §3.9、`docs/spec/semantics.md` §4.5.5 / §8.1、
//! `docs/spec/interface-contract.md` §10.8 / §10.9（R-S3）。
//!
//! 经 `lfz` 库的稳定入口 `lexer::lex` + `parser::parse` 校验；大深度结构（`1+1+…`、
//! `a[0][0]…`）在主测试线程栈上会因求值 / 析构溢出，故一律经
//! `evaluator::on_eval_stack`（256 MiB ≥ 64 MiB）解析。
//!
//! 度量（§3.9 规则 1）：`depth(n) = 1 + max(直接语法子节点 depth)`（叶 = 1）；
//! 程序 AST 深度 = 顶层语句深度最大值（空程序 = 0）；括号分组透明；左结合链 AST 深度 = 链长。

use lfz::error::LzError;
use lfz::evaluator::on_eval_stack;
use lfz::parser::{parse, AST_DEPTH_LIMIT, PARSE_DEPTH_LIMIT};

/// 规范逐字符消息（`semantics.md` §8.1）。
const DEEP_MSG: &str = "表达式嵌套过深（超过 10000 层）";

/// 在大栈线程上 `lex + parse` 一段程序体（`line_base = 1`）；只取成功 / 错误，
/// 避免深 AST 逃逸到小栈线程析构。
fn parse_big(body: &str) -> Result<(), Box<LzError>> {
    let body = body.to_string();
    on_eval_stack(move || {
        let toks = lfz::lexer::lex(&body, 1)?;
        parse(&toks).map(|_| ())
    })
}

#[test]
fn ast_depth_limit_constant_is_10000() {
    assert_eq!(AST_DEPTH_LIMIT, 10000, "§3.9：AST_DEPTH_LIMIT 必须为 10000");
    // 口径分离（§3.9 规则 6）：与 PARSE_DEPTH_LIMIT 是两个独立常量。
    assert_ne!(
        AST_DEPTH_LIMIT, PARSE_DEPTH_LIMIT,
        "两度量口径不得合并（R-S3）"
    );
}

#[test]
fn add_chain_9999_terms_is_legal() {
    // 正例（§3.9）：`1+1+…`（9999 项）作表达式语句 → 9999 + 1 = 10000 → 合法。
    let src = format!("1{}", "+1".repeat(9998));
    assert!(parse_big(&src).is_ok(), "9999 项 `+` 链应合法（深度恰 10000）");
}

#[test]
fn add_chain_10000_terms_is_expr_too_deep() {
    // 反例（§3.9）：10000 项 → 10000 + 1 = 10001 → `SyntaxError`。
    let src = format!("1{}", "+1".repeat(9999));
    let err = parse_big(&src).expect_err("10000 项 `+` 链应报 SyntaxError");
    assert_eq!(err.class_name(), "SyntaxError", "仍归 SyntaxError（不新增错误类）");
    assert_eq!(err.message(), DEEP_MSG);
    assert_eq!(err.to_string(), format!("SyntaxError: {DEEP_MSG}"));
    // span = 首次使 AST 深度超限的节点首字符（链式构造通常为链起点）。
    // `line_base = 1`：程序体第 1 行报表第 2 行；链起点第 1 列。
    let span = err.span().expect("SyntaxError 必带位置");
    assert_eq!((span.line, span.col), (2, 1));
}

#[test]
fn index_chain_9998_subscripts_is_legal() {
    // 正例：`a[0][0]…`（9998 个 `[0]`）作表达式语句 → 1 + 9998 + 1 = 10000 → 合法。
    let src = format!("a{}", "[0]".repeat(9998));
    assert!(parse_big(&src).is_ok(), "9998 个 `[0]` 应合法（深度恰 10000）");
}

#[test]
fn index_chain_9999_subscripts_is_expr_too_deep() {
    // 反例：9999 个 `[0]` → 1 + 9999 + 1 = 10001 → `SyntaxError`。
    let src = format!("a{}", "[0]".repeat(9999));
    let err = parse_big(&src).expect_err("9999 个 `[0]` 应报 SyntaxError");
    assert_eq!(err.class_name(), "SyntaxError");
    assert_eq!(err.message(), DEEP_MSG);
}

#[test]
fn paren_grouping_is_transparent() {
    // §3.9 规则 1：括号分组透明（无独立节点），`(((1)))` 的 AST 深度 = 1。
    // 取 900 层（< PARSE_DEPTH_LIMIT 1000），验证其 AST 深度不随括号层数增长。
    let src = format!("{}1{}", "(".repeat(900), ")".repeat(900));
    assert!(parse_big(&src).is_ok(), "括号分组应透明、不累加 AST 深度");
}

#[test]
fn parse_nesting_1000_stays_legal_under_ast_limit() {
    // 口径分离回归：`(`×1000（R-S1 正例，解析嵌套恰 1000）的 AST 深度很小 → 仍合法。
    let src = format!("{}1{}", "(".repeat(1000), ")".repeat(1000));
    assert!(parse_big(&src).is_ok(), "`(`×1000 应仍合法（R-S1 正例不受 R-S3 影响）");
}
