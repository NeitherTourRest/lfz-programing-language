//! 最小命令行界面（P3.10）：`lfz run <file>` + `--help` / `--version`。
//!
//! 单一事实源（本模块**只读引用**，不偏离）：
//! - `docs/spec/semantics.md` §8.2（用户可见输出格式）/ §8.3（逐字符精确示例）。
//! - `docs/spec/interface-contract.md` §8.1（错误类 ↔ 退出码）。
//! - `.opencode/team/DECISIONS.md` D-008（退出码 `0/1/2`）。
//!
//! # 职责边界（契约 §10.6）
//!
//! CLI **只做**两件事：**错误格式化**与**退出码映射**。它不定义语法 / 语义，
//! 不改解释器核心，不复制任何词法 / 语法 / 求值逻辑——只按固定顺序调用稳定库入口：
//! `loader::load_file` → `lexer::lex` → `parser::parse` → `evaluator::eval_module_traced`。
//!
//! # 本批范围
//!
//! **已做**：`lfz run <file>`、`--help` / `-h`、`--version` / `-V`、错误格式化、退出码。
//! **不做**（留 P4）：REPL、`--json`、`lfz test` runner。
//!
//! # 流约定
//!
//! - 程序自身输出（`print` / `eprint` / `;;`）由解释器 `builtins` 直接写 **stdout / stderr**，
//!   本模块**不**接管、不缓冲。
//! - CLI 的**错误诊断**写 **stderr**；`--help` / `--version` 写 **stdout**。
//!   （spec 未规定错误流；采用 Python 约定 → stderr。）
//!
//! # 退出码（D-008）
//!
//! `0` 成功；`1` 测试有用例失败（`lfz test` 专用，P4）；`2` CLI 参数错误 /
//! LFZ 语法或运行时错误 / 运行环境错误。

use lfz::error::LzError;
use lfz::evaluator::{self, TracedRun};
use lfz::lexer;
use lfz::loader::{self, Loaded};
use lfz::parser;
use lfz::span::Span;
use std::io::Write;

/// 成功。
pub const EXIT_OK: i32 = 0;
/// 测试有用例失败（`lfz test` 专用；P4 的 runner 使用，本批仅保留常量）。
#[allow(dead_code)]
pub const EXIT_TEST_FAIL: i32 = 1;
/// CLI 参数错误 / LFZ 语法或运行时错误 / 运行环境错误。
pub const EXIT_ERROR: i32 = 2;

/// `lfz --help` 文本（写 stdout）。
const HELP: &str = "\
LFZ 解释器（最小命令行）

用法:
  lfz run <file>       运行 LFZ 脚本（.lfz 文件要求首行为 #42）
  lfz --help, -h       显示本帮助
  lfz --version, -V    显示版本

退出码:
  0  成功
  1  测试失败（lfz test，P4）
  2  CLI 参数错误 / LFZ 语法或运行时错误 / 运行环境错误

示例:
  lfz run examples/hello.lfz
";

/// `lfz --version` 文本（写 stdout）。
const VERSION: &str = concat!("lfz ", env!("CARGO_PKG_VERSION"));

/// 解析后的命令行。
#[derive(Debug, PartialEq, Eq)]
enum Command {
    /// `lfz run <file>`。
    Run { path: String },
    /// `--help` / `-h`。
    Help,
    /// `--version` / `-V`。
    Version,
}

/// CLI 入口：解析 `args`（**不含** `argv[0]`），写 `out` / `err`，返回退出码。
///
/// 与真实进程解耦：`out` / `err` 可为任意 [`Write`]（生产用 stdout/stderr，测试用内存缓冲）。
pub fn execute(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    match parse_args(args) {
        Ok(Command::Help) => {
            let _ = write!(out, "{HELP}");
            EXIT_OK
        }
        Ok(Command::Version) => {
            let _ = writeln!(out, "{VERSION}");
            EXIT_OK
        }
        Ok(Command::Run { path }) => run_file(&path, err),
        Err(msg) => {
            let _ = writeln!(err, "lfz: {msg}");
            let _ = writeln!(err, "用法: lfz run <file>（更多: lfz --help）");
            EXIT_ERROR
        }
    }
}

/// 解析参数为 [`Command`]；失败返回中文错误信息（由调用方写 stderr）。
fn parse_args(args: &[String]) -> Result<Command, String> {
    let mut it = args.iter();
    let first = match it.next() {
        Some(a) => a,
        None => return Err("缺少子命令".to_string()),
    };
    match first.as_str() {
        "--help" | "-h" => Ok(Command::Help),
        "--version" | "-V" => Ok(Command::Version),
        "run" => {
            let path = match it.next() {
                Some(p) => p.clone(),
                None => return Err("'run' 需要一个 <file> 参数".to_string()),
            };
            if it.next().is_some() {
                return Err("'run' 只接受一个 <file> 参数".to_string());
            }
            Ok(Command::Run { path })
        }
        other => Err(format!("未知命令 '{other}'")),
    }
}

/// 运行一个 LFZ 文件：`load_file → lex → parse → eval_module_traced`。
///
/// 任一阶段失败 → 按 §8.2/§8.3 渲染错误到 `err`，返回 [`EXIT_ERROR`]；成功 → [`EXIT_OK`]。
fn run_file(path: &str, err: &mut dyn Write) -> i32 {
    // 加载期：失败（IOError / NotUtf8 / CosmosAnswerError）时无源码视图。
    let loaded = match loader::load_file(path) {
        Ok(l) => l,
        Err(e) => {
            let _ = write!(err, "{}", render_error(path, None, &e, None));
            return EXIT_ERROR;
        }
    };
    // 词法期。
    let tokens = match lexer::lex(&loaded.text, loaded.line_base) {
        Ok(t) => t,
        Err(e) => {
            let _ = write!(err, "{}", render_error(path, Some(&loaded), &e, None));
            return EXIT_ERROR;
        }
    };
    // 语法期。
    let program = match parser::parse(&tokens) {
        Ok(p) => p,
        Err(e) => {
            let _ = write!(err, "{}", render_error(path, Some(&loaded), &e, None));
            return EXIT_ERROR;
        }
    };
    // 运行期：用 `eval_module_traced` 取帧栈（§10.3 / D-008 影响段）。
    let traced = evaluator::eval_module_traced(&program);
    if let Err(e) = &traced.result {
        let _ = write!(err, "{}", render_error(path, Some(&loaded), e, Some(&traced)));
        return EXIT_ERROR;
    }
    EXIT_OK
}

/// 按 `semantics.md` §8.2 / §8.3 渲染错误输出（返回值以 `\n` 结尾）。
///
/// - `source`：加载成功后的源码视图；加载失败（如 `IOError`）时为 `None`。
/// - `traced`：运行期错误的帧栈；`None` 表示**加载 / 解析期**（**无** `Traceback` 头）。
///
/// 结构：
/// - `CosmosAnswerError`：仅 `File "<path>", line 1` → 末行（**无源码行 / 无插入符**）。
/// - `SyntaxError`：`  File "<path>", line N` + 源码行 + 插入符 → 末行。
/// - 运行期：`Traceback (most recent call last):` 头 + 逐帧（最外层→最内层）→ 末行。
pub fn render_error(
    path: &str,
    source: Option<&Loaded>,
    err: &LzError,
    traced: Option<&TracedRun>,
) -> String {
    let mut out = String::new();
    match traced {
        // 加载 / 解析期：无 `Traceback` 头。
        None => render_load_error(&mut out, path, source, err),
        // 运行期：有 `Traceback` 头，逐帧最外层→最内层。
        Some(run) => {
            out.push_str("Traceback (most recent call last):\n");
            for frame in &run.frames {
                let name = run.frame_name(frame);
                push_frame(&mut out, path, frame.span, Some(name.as_ref()), source);
            }
        }
    }
    out.push_str(err.class_name());
    out.push_str(": ");
    out.push_str(&err.message());
    out.push('\n');
    out
}

/// 加载 / 解析期错误渲染（§8.2：**无** `Traceback` 头）。
fn render_load_error(out: &mut String, path: &str, source: Option<&Loaded>, err: &LzError) {
    if err.class_name() == "CosmosAnswerError" {
        // §8.3 示例 3：仅 `File "<path>", line 1`，无源码行 / 插入符。
        out.push_str("File \"");
        out.push_str(path);
        out.push_str("\", line 1\n");
        return;
    }
    // SyntaxError 等：有位置则渲染 `File` 帧（含源码行 + 插入符）。
    if let Some(span) = err.span() {
        push_frame(out, path, span, None, source);
    }
}

/// 追加一个「帧」（§8.2 通用三行格式；源码行 / 插入符仅在可取到源码行时输出）。
///
/// `func = None` → 帧头无 `, in <func>`（加载 / 解析期）。
fn push_frame(
    out: &mut String,
    path: &str,
    span: Span,
    func: Option<&str>,
    source: Option<&Loaded>,
) {
    out.push_str("  File \"");
    out.push_str(path);
    out.push_str("\", line ");
    out.push_str(&span.line.to_string());
    if let Some(name) = func {
        out.push_str(", in ");
        out.push_str(name);
    }
    out.push('\n');
    if let Some(line_text) = source_line(source, span.line) {
        // 源码行原文，固定 4 空格缩进。
        out.push_str("    ");
        out.push_str(line_text);
        out.push('\n');
        // 插入符行 = 4 空格 + (列号 - 1) 空格 + '^'。
        out.push_str("    ");
        for _ in 1..span.col {
            out.push(' ');
        }
        out.push_str("^\n");
    }
}

/// 取绝对行号 `line` 对应的源码行文本。
///
/// `loaded.text` 的行号与绝对行号相差 `line_base`：`.lfz`（`line_base = 1`）的 `text`
/// 首行 = 文件第 2 行 → 下标 = `line - 2`；非 `.lfz`（`line_base = 0`）→ 下标 = `line - 1`。
/// 行号越界或 `source` 缺失 → `None`（此时帧只输出 `File` 行）。
fn source_line<'a>(source: Option<&'a Loaded>, line: u32) -> Option<&'a str> {
    let loaded = source?;
    let idx = line.checked_sub(loaded.line_base + 1)?;
    loaded.text.lines().nth(idx as usize)
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use lfz::error::SyntaxMsg;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn parse_help_and_version() {
        assert_eq!(parse_args(&args(&["--help"])), Ok(Command::Help));
        assert_eq!(parse_args(&args(&["-h"])), Ok(Command::Help));
        assert_eq!(parse_args(&args(&["--version"])), Ok(Command::Version));
        assert_eq!(parse_args(&args(&["-V"])), Ok(Command::Version));
    }

    #[test]
    fn parse_run_requires_exactly_one_file() {
        assert_eq!(
            parse_args(&args(&["run", "a.lfz"])),
            Ok(Command::Run {
                path: "a.lfz".to_string()
            })
        );
        assert!(parse_args(&args(&["run"])).is_err());
        assert!(parse_args(&args(&["run", "a.lfz", "b.lfz"])).is_err());
        assert!(parse_args(&args(&[])).is_err());
        assert!(parse_args(&args(&["frobnicate"])).is_err());
    }

    #[test]
    fn help_and_version_exit_zero_on_stdout() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(execute(&args(&["--help"]), &mut out, &mut err), EXIT_OK);
        let text = String::from_utf8(out).expect("UTF-8");
        assert!(text.contains("lfz run <file>"), "help: {text}");
        assert!(err.is_empty());

        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(execute(&args(&["--version"]), &mut out, &mut err), EXIT_OK);
        assert_eq!(String::from_utf8(out).expect("UTF-8"), format!("{VERSION}\n"));
        assert!(err.is_empty());
    }

    #[test]
    fn bad_args_exit_two_on_stderr() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(execute(&args(&[]), &mut out, &mut err), EXIT_ERROR);
        assert!(out.is_empty());
        assert!(String::from_utf8(err).expect("UTF-8").contains("缺少子命令"));
    }

    /// §8.3 示例 3：`CosmosAnswerError` 仅 `File` 行，无源码行 / 插入符 / Traceback 头。
    #[test]
    fn cosmos_answer_has_only_file_line() {
        let e = LzError::CosmosAnswer { span: Span::START };
        let rendered = render_error("forgot.lfz", None, &e, None);
        assert_eq!(
            rendered,
            "File \"forgot.lfz\", line 1\nCosmosAnswerError: 你忘记了宇宙的答案\n"
        );
    }

    /// §8.3 示例 1：`SyntaxError` 的 `File` 帧 + 源码行 + 插入符，无 Traceback 头。
    #[test]
    fn syntax_error_renders_frame_with_caret() {
        let loaded = Loaded {
            text: "let x = 1 $ 2\n".to_string(),
            line_base: 1,
        };
        let e = LzError::Syntax {
            msg: SyntaxMsg::IllegalChar { c: '$' },
            span: Span::new(2, 11),
        };
        let rendered = render_error("demo.lfz", Some(&loaded), &e, None);
        assert_eq!(
            rendered,
            "  File \"demo.lfz\", line 2\n    let x = 1 $ 2\n              ^\nSyntaxError: 非法字符 '$'\n"
        );
    }

    /// 运行期错误：含 `Traceback` 头，且末行为 `类名: 中文消息`。
    #[test]
    fn runtime_error_has_traceback_header() {
        let src = "1 / 0\n";
        let tokens = lexer::lex(src, 0).expect("lex");
        let program = parser::parse(&tokens).expect("parse");
        let traced = evaluator::eval_module_traced(&program);
        let err = traced.result.as_ref().expect_err("除以零应报错");
        let loaded = Loaded {
            text: src.to_string(),
            line_base: 0,
        };
        let rendered = render_error("demo.lfz", Some(&loaded), err, Some(&traced));
        assert!(
            rendered.starts_with("Traceback (most recent call last):\n"),
            "运行期错误必须有 Traceback 头：{rendered}"
        );
        assert!(
            rendered.contains("File \"demo.lfz\", line 1, in <module>"),
            "{rendered}"
        );
        assert!(
            rendered.ends_with("ZeroDivisionError: 除以零\n"),
            "{rendered}"
        );
    }

    /// `source_line` 的 `line_base` 偏移（`.lfz` 的 text 首行 = 文件第 2 行）。
    #[test]
    fn source_line_respects_line_base() {
        let loaded = Loaded {
            text: "first\nsecond\n".to_string(),
            line_base: 1,
        };
        assert_eq!(source_line(Some(&loaded), 2), Some("first"));
        assert_eq!(source_line(Some(&loaded), 3), Some("second"));
        assert_eq!(source_line(Some(&loaded), 1), None, "前导行不显示源码");
        assert_eq!(source_line(Some(&loaded), 99), None);
        assert_eq!(source_line(None, 2), None);
    }
}
