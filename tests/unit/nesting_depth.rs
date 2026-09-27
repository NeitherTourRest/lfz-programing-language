//! §3.8 解析嵌套深度上限（v1.1 补钉）——边界与栈安全。
//!
//! 规范：`docs/spec/syntax.md` §3.8、`docs/spec/semantics.md` §8.1、
//! `docs/spec/interface-contract.md` §10.8 / §10.9（R-S1 / R-S2）。
//!
//! 经 `lfz` 库的稳定入口 `lexer::lex` + `parser::parse` 校验；深嵌套（`(`×1000 等）在
//! 主测试线程栈上会溢出，故一律经 `evaluator::on_eval_stack`（256 MiB ≥ 64 MiB）解析。

use lfz::error::LzError;
use lfz::evaluator::on_eval_stack;
use lfz::parser::{parse, PARSE_DEPTH_LIMIT};

/// 规范逐字符消息（`semantics.md` §8.1）。
const NEST_MSG: &str = "嵌套深度超限（超过 1000 层）";

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
fn limit_constant_is_1000() {
    assert_eq!(PARSE_DEPTH_LIMIT, 1000, "§3.8：PARSE_DEPTH_LIMIT 必须为 1000");
}

#[test]
fn paren_1000_is_legal() {
    // 正例：`(`×1000 `1` `)`×1000 → 深度恰 1000 → 合法。
    let src = format!("{}1{}", "(".repeat(1000), ")".repeat(1000));
    assert!(parse_big(&src).is_ok(), "`(`×1000 应合法");
}

#[test]
fn paren_1001_is_nesting_too_deep() {
    let src = format!("{}1{}", "(".repeat(1001), ")".repeat(1001));
    let err = parse_big(&src).expect_err("`(`×1001 应报 SyntaxError");
    assert_eq!(err.class_name(), "SyntaxError", "仍归 SyntaxError（不新增错误类）");
    assert_eq!(err.message(), NEST_MSG);
    assert_eq!(err.to_string(), format!("SyntaxError: {NEST_MSG}"));
    // span = 第 1001 层开启记号（第 1001 个 `(`）首字符。
    // `line_base = 1`（模拟 `.lfz`：`#42` 占文件第 1 行，程序体第 1 行 → 报表第 2 行）：
    // 第 2 行第 1001 列。
    let span = err.span().expect("SyntaxError 必带位置");
    assert_eq!((span.line, span.col), (2, 1001));
}

#[test]
fn bracket_1001_is_nesting_too_deep() {
    let src = format!("{}1{}", "[".repeat(1001), "]".repeat(1001));
    let err = parse_big(&src).expect_err("`[`×1001 应报 SyntaxError");
    assert_eq!(err.class_name(), "SyntaxError");
    assert_eq!(err.message(), NEST_MSG);
}

#[test]
fn struct_lit_1001_is_nesting_too_deep() {
    // 最坏构造（每层帧最大）：`{"a":`×1001 `1` `}`×1001。
    let src = format!("{}1{}", "{\"a\":".repeat(1001), "}".repeat(1001));
    let err = parse_big(&src).expect_err("`{\"a\":`×1001 应报 SyntaxError");
    assert_eq!(err.class_name(), "SyntaxError");
    assert_eq!(err.message(), NEST_MSG);
}

#[test]
fn fn_block_1001_is_nesting_too_deep() {
    let src = format!("{}1{}", "fn(){".repeat(1001), "}".repeat(1001));
    let err = parse_big(&src).expect_err("`fn(){`×1001 应报 SyntaxError");
    assert_eq!(err.class_name(), "SyntaxError");
    assert_eq!(err.message(), NEST_MSG);
}

#[test]
fn unary_prefix_1001_is_nesting_too_deep() {
    let src = format!("{}1", "-".repeat(1001));
    let err = parse_big(&src).expect_err("`-`×1001 应报 SyntaxError");
    assert_eq!(err.class_name(), "SyntaxError");
    assert_eq!(err.message(), NEST_MSG);
}

#[test]
fn left_assoc_chains_are_not_counted() {
    // §3.8 规则 1「不计层」：左结合链由迭代循环解析，不受 PARSE_DEPTH_LIMIT 约束。
    let add_chain = format!("1{}", "+1".repeat(5000));
    assert!(parse_big(&add_chain).is_ok(), "`1+1+…`（5001 项）应可解析");
    let index_chain = format!("a{}", "[0]".repeat(1001));
    assert!(parse_big(&index_chain).is_ok(), "`a[0][0]…`（1002 项）应可解析");
}
