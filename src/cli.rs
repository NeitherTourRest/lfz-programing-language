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
//! **已做**：`lfz run <file>`、`lfz run --json <file>`、`lfz test [路径...]`
//! （实现见 [`crate::test_runner`]）、`lfz test --json`、`--help` / `-h`、`--version` / `-V`、
//! 错误格式化、退出码、`--json` 机器可读输出（`semantics.md` §8.3 示例 4 / §8.4）。
//! **不做**：REPL。
//!
//! # 流约定
//!
//! - 程序自身输出（`print` / `eprint` / `;;`）默认由解释器 `builtins` 直接写 **stdout / stderr**，
//!   本模块**不**接管、不缓冲；`--json` 下 `print` / `input` 提示被重定向到 stderr（见下）。
//! - CLI 的**错误诊断**写 **stderr**；`--help` / `--version` 写 **stdout**。
//!   （spec 未规定错误流；采用 Python 约定 → stderr。）
//! - **`--json` 模式**：stdout **只**接收**唯一一行 JSON**（结果对象）；人类可读的报告 / 诊断
//!   一律转 **stderr**（硬性设计约束）。**P4.2-fix（已解决）**：CLI 在运行前把解释器内建
//!   `print` / `input` 提示重定向到 **stderr**（`lfz::builtins::set_stdout_to_stderr`），故 stdout
//!   恒为单个 JSON，程序输出不再与之交错（见 `docs/tooling/runner-contract.md` §9.3）。
//!
//! # 退出码（D-008）
//!
//! `0` 成功；`1` 测试有用例失败（`lfz test` 专用，P4）；`2` CLI 参数错误 /
//! LFZ 语法或运行时错误 / 运行环境错误。**`--json` 错误仍退出 `2`**（§8.3 / §8.4）。

use crate::json::{self, Json};
use lfz::error::{LzError, TraceFrame};
use lfz::evaluator::{self, TracedRun};
use lfz::lexer;
use lfz::loader::{self, Loaded};
use lfz::parser;
use lfz::span::Span;
use std::io::Write;

/// 成功。
pub const EXIT_OK: i32 = 0;
/// 测试有用例失败（`lfz test` 专用；由 [`crate::test_runner`] 使用）。
pub const EXIT_TEST_FAIL: i32 = 1;
/// CLI 参数错误 / LFZ 语法或运行时错误 / 运行环境错误。
pub const EXIT_ERROR: i32 = 2;

/// traceback 折叠（`semantics.md` §8.2 / `interface-contract.md` §10.3）：帧数 `T` **大于**
/// [`TRACEBACK_FOLD_THRESHOLD`] 时，保留的**最外层**帧数（规范性常量 `TRACEBACK_HEAD = 10`）。
pub const TRACEBACK_HEAD: usize = 10;
/// traceback 折叠：保留的**最内层**帧数（规范性常量 `TRACEBACK_TAIL = 30`）。
pub const TRACEBACK_TAIL: usize = 30;
/// traceback 折叠阈值 = `TRACEBACK_HEAD + TRACEBACK_TAIL`（= 40）；`T ≤` 此值时逐帧原样输出。
pub const TRACEBACK_FOLD_THRESHOLD: usize = TRACEBACK_HEAD + TRACEBACK_TAIL;

/// `lfz --help` 文本（写 stdout）。
const HELP: &str = "\
LFZ 解释器（最小命令行）

用法:
  lfz run <file>       运行 LFZ 脚本（.lfz 文件要求首行为 #42）
  lfz test [路径...]   运行黑盒测试（默认发现 tests/**/*.lfz，排除 tests/fixtures）
  lfz --help, -h       显示本帮助
  lfz --version, -V    显示版本

选项:
  --json               以机器可读 JSON 输出（stdout 只写 JSON；人类可读诊断转 stderr）

退出码:
  0  成功（所有用例通过）
  1  测试有用例失败（assert / fail 抛 AssertionError）
  2  CLI 参数错误 / LFZ 语法或运行时错误 / 用例 error / 运行环境错误

示例:
  lfz run examples/hello.lfz
  lfz run --json examples/hello.lfz
  lfz test
  lfz test --json tests/smoke
";

/// `lfz --version` 文本（写 stdout）。
const VERSION: &str = concat!("lfz ", env!("CARGO_PKG_VERSION"));

/// 解析后的命令行。
#[derive(Debug, PartialEq, Eq)]
enum Command {
    /// `lfz run <file>`。
    Run { path: String },
    /// `lfz test [路径...]`。
    Test { paths: Vec<String> },
    /// `--help` / `-h`。
    Help,
    /// `--version` / `-V`。
    Version,
}

/// CLI 入口：解析 `args`（**不含** `argv[0]`），写 `out` / `err`，返回退出码。
///
/// 与真实进程解耦：`out` / `err` 可为任意 [`Write`]（生产用 stdout/stderr，测试用内存缓冲）。
///
/// `--json` 是**全局开关**：出现在任意位置即可，被先行摘除后再解析子命令。开启后 stdout 只写 JSON。
pub fn execute(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let json = args.iter().any(|a| a == "--json");
    let rest: Vec<String> = args.iter().filter(|a| *a != "--json").cloned().collect();
    match parse_args(&rest) {
        Ok(Command::Help) => {
            let _ = write!(out, "{HELP}");
            EXIT_OK
        }
        Ok(Command::Version) => {
            let _ = writeln!(out, "{VERSION}");
            EXIT_OK
        }
        Ok(Command::Run { path }) => run_file(&path, json, out, err),
        Ok(Command::Test { paths }) => crate::test_runner::run(&paths, json, out, err),
        Err(msg) => {
            let _ = writeln!(err, "lfz: {msg}");
            let _ = writeln!(err, "用法: lfz run <file> | lfz test [路径...]（更多: lfz --help）");
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
        // `lfz test [路径...]`：零或多个路径参数（目录 → 递归发现；文件 → 直接运行）。
        "test" => Ok(Command::Test {
            paths: it.cloned().collect(),
        }),
        other => Err(format!("未知命令 '{other}'")),
    }
}

/// 单个用例的执行结果：供 `run`（单文件）与 [`crate::test_runner`]（批量）共用。
///
/// **只有**「跑完全部阶段且无 `LfzError`」为 [`CaseEval::Passed`]；任一阶段报错都进
/// [`CaseEval::Failed`] 并携带渲染所需的上下文（`loaded` / `traced` / `err`）。
pub(crate) enum CaseEval {
    /// 加载 → 词法 → 语法 → 求值全部成功（无任何 `LfzError`；`check` 失败不在此列）。
    Passed,
    /// 某阶段抛 `LfzError`。
    Failed {
        /// 该错误（已从 `Box` 解出）。
        err: LzError,
        /// 加载成功时的源码视图（加载期失败为 `None`）。
        loaded: Option<Loaded>,
        /// 运行期错误的帧栈（加载 / 解析期失败为 `None`）。
        traced: Option<TracedRun>,
    },
}

impl CaseEval {
    /// 错误的**类名**（`Passed` → `None`）。
    #[must_use]
    pub(crate) fn error_class(&self) -> Option<&'static str> {
        match self {
            CaseEval::Passed => None,
            CaseEval::Failed { err, .. } => Some(err.class_name()),
        }
    }

    /// 错误对象（`Passed` → `None`）。
    #[must_use]
    pub(crate) fn error(&self) -> Option<&LzError> {
        match self {
            CaseEval::Passed => None,
            CaseEval::Failed { err, .. } => Some(err),
        }
    }

    /// 按 `semantics.md` §8.2 / §8.3 渲染该错误的完整输出块（`Passed` → `None`）。
    #[must_use]
    pub(crate) fn render(&self, path: &str) -> Option<String> {
        match self {
            CaseEval::Passed => None,
            CaseEval::Failed { err, loaded, traced } => {
                Some(render_error(path, loaded.as_ref(), err, traced.as_ref()))
            }
        }
    }
}

/// `run --json` 成功对象：`{"ok":true}`（§8.4：除 `ok` 外其余字段均为错误专有，成功时省略）。
#[must_use]
pub(crate) fn success_json() -> Json {
    Json::Obj(vec![("ok".to_string(), Json::Bool(true))])
}

/// `run --json` 失败对象：字段与顺序**逐字符对齐** `semantics.md` §8.3 示例 4——
/// `ok` / `error` / `message` / `file` / `line` / `col` / `traceback`。
#[must_use]
pub(crate) fn error_json(path: &str, eval: &CaseEval) -> Json {
    let mut fields = vec![("ok".to_string(), Json::Bool(false))];
    push_error_fields(&mut fields, path, eval);
    Json::Obj(fields)
}

/// 追加一个错误的 §8.4 字段（`error` / `message` / `file` / `line` / `col` / `traceback`），
/// **不含** `ok`。供 [`error_json`]（`run`）与 `test_runner`（逐用例）复用。
///
/// - `error` 恒为 [`LzError::class_name`]（12 类名之一，**禁止** `E-xxx`）；
/// - `line` / `col` 取 [`LzError::span`]；`Io` 无 span 时输出 `null`；
/// - `traceback` 为**自最外层到最内层**的完整帧表（**不折叠**，§8.4）。
pub(crate) fn push_error_fields(fields: &mut Vec<(String, Json)>, path: &str, eval: &CaseEval) {
    let CaseEval::Failed { err, traced, .. } = eval else {
        return;
    };
    fields.push(("error".to_string(), Json::Str(err.class_name().to_string())));
    fields.push(("message".to_string(), Json::Str(err.message())));
    fields.push(("file".to_string(), Json::Str(path.to_string())));
    match err.span() {
        Some(span) => {
            fields.push(("line".to_string(), Json::Int(i64::from(span.line))));
            fields.push(("col".to_string(), Json::Int(i64::from(span.col))));
        }
        // `Io` 可无源码位置（如读文件失败，loader 不给 span）→ `null`（保持字段存在、类型显式）。
        None => {
            fields.push(("line".to_string(), Json::Null));
            fields.push(("col".to_string(), Json::Null));
        }
    }
    fields.push((
        "traceback".to_string(),
        traceback_json(path, err, traced.as_ref()),
    ));
}

/// 构造 `--json` 的 `traceback` 数组（§8.4：元素 `{file,line,func}`，**不折叠**）。
///
/// - 运行期（`traced = Some`）：逐帧取 `frame.span.line` + [`TracedRun::frame_name`]
///   （`<module>` / 函数名 / `<fn>`），顺序自**最外层→最内层**（与 §8.3 示例 2 一致）。
/// - 加载 / 解析期（`traced = None`）：**合成单帧 `<module>`**，行取错误 span（§8.3 示例 4）；
///   无 span（`Io`）时为空数组 `[]`。
#[must_use]
pub(crate) fn traceback_json(
    path: &str,
    err: &LzError,
    traced: Option<&TracedRun>,
) -> Json {
    match traced {
        Some(run) => Json::Arr(
            run.frames
                .iter()
                .map(|frame| {
                    Json::Obj(vec![
                        ("file".to_string(), Json::Str(path.to_string())),
                        ("line".to_string(), Json::Int(i64::from(frame.span.line))),
                        (
                            "func".to_string(),
                            Json::Str(run.frame_name(frame).to_string()),
                        ),
                    ])
                })
                .collect(),
        ),
        None => match err.span() {
            Some(span) => Json::Arr(vec![Json::Obj(vec![
                ("file".to_string(), Json::Str(path.to_string())),
                ("line".to_string(), Json::Int(i64::from(span.line))),
                ("func".to_string(), Json::Str("<module>".to_string())),
            ])]),
            None => Json::Arr(Vec::new()),
        },
    }
}

/// 加载并执行一个 LFZ 文件：`load_file → lex → parse → eval_module_traced`。
///
/// 与 [`run_file`] 的差别：`run_file` 只关心成败与渲染，本函数把**中间上下文**带出，
/// 供 runner 逐用例判定（类名 ↔ 期望、位置信息）。
pub(crate) fn eval_case(path: &str) -> CaseEval {
    // 加载期：失败（IOError / NotUtf8 / CosmosAnswerError）时无源码视图。
    let loaded = match loader::load_file(path) {
        Ok(l) => l,
        Err(e) => {
            return CaseEval::Failed {
                err: *e,
                loaded: None,
                traced: None,
            }
        }
    };
    // 词法期。
    let tokens = match lexer::lex(&loaded.text, loaded.line_base) {
        Ok(t) => t,
        Err(e) => {
            return CaseEval::Failed {
                err: *e,
                loaded: Some(loaded),
                traced: None,
            }
        }
    };
    // 语法期。
    let program = match parser::parse(&tokens) {
        Ok(p) => p,
        Err(e) => {
            return CaseEval::Failed {
                err: *e,
                loaded: Some(loaded),
                traced: None,
            }
        }
    };
    // 运行期：用 `eval_module_traced` 取帧栈（§10.3 / D-008 影响段）。
    let traced = evaluator::eval_module_traced(&program);
    if traced.result.is_ok() {
        return CaseEval::Passed;
    }
    // 先把错误 clone 出来，再把 `traced`（含完整帧栈）移交出去。
    let err = (**traced.result.as_ref().expect_err("已判定为 Err")).clone();
    CaseEval::Failed {
        err,
        loaded: Some(loaded),
        traced: Some(traced),
    }
}

/// 运行一个 LFZ 文件。
///
/// - `json == false`：失败 → 按 §8.2/§8.3 渲染到 `err` 并返回 [`EXIT_ERROR`]；成功 → [`EXIT_OK`]。
/// - `json == true`：stdout **只写一行 JSON**（成功 `{"ok":true}`；失败为 §8.3 示例 4 形状），
///   人类可读错误块**不**写 stdout；退出码同 D-008（错误 → [`EXIT_ERROR`]）。
fn run_file(path: &str, json: bool, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    // P4.2-fix：`--json` 下把 `print` / `input` 提示重定向到 stderr，保证 stdout 恒为单个 JSON。
    lfz::builtins::set_stdout_to_stderr(json);
    let eval = eval_case(path);
    if json {
        let passed = matches!(&eval, CaseEval::Passed);
        let doc = if passed {
            success_json()
        } else {
            error_json(path, &eval)
        };
        let _ = writeln!(out, "{}", json::encode(&doc));
        return if passed { EXIT_OK } else { EXIT_ERROR };
    }
    match eval.render(path) {
        None => EXIT_OK,
        Some(block) => {
            let _ = write!(err, "{block}");
            EXIT_ERROR
        }
    }
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
///   帧数 `T > [`TRACEBACK_FOLD_THRESHOLD`]` 时按 §8.2 **折叠**（首 10 帧 + 省略行 + 尾 30 帧），
///   `T ≤ 阈值` 时逐帧原样输出（浅栈行为逐字节不变）。
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
        // 运行期：有 `Traceback` 头，逐帧最外层→最内层（深栈折叠，§8.2）。
        Some(run) => {
            out.push_str("Traceback (most recent call last):\n");
            push_frames(&mut out, path, run, source);
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

/// 追加运行期**帧栈**（`semantics.md` §8.2 折叠）：帧数 `T ≤ 40` → 逐帧原样；`T > 40` →
/// **首 10 帧** → **省略行** → **尾 30 帧**。
///
/// 折叠**仅作用于此处的人类可读渲染**；`TracedRun.frames` 本身保持完整（§10.3 / §8.4）。
fn push_frames(out: &mut String, path: &str, run: &TracedRun, source: Option<&Loaded>) {
    let frames = &run.frames;
    let total = frames.len();
    if total <= TRACEBACK_FOLD_THRESHOLD {
        for frame in frames {
            push_trace_frame(out, path, frame, run, source);
        }
        return;
    }
    for frame in &frames[..TRACEBACK_HEAD] {
        push_trace_frame(out, path, frame, run, source);
    }
    // 省略行（逐字符，§8.2）：2 空格 + `... 省略 {N} 帧 ...`，`N = T − 40`（十进制）。
    out.push_str("  ... 省略 ");
    out.push_str(&(total - TRACEBACK_FOLD_THRESHOLD).to_string());
    out.push_str(" 帧 ...\n");
    for frame in &frames[total - TRACEBACK_TAIL..] {
        push_trace_frame(out, path, frame, run, source);
    }
}

/// 追加单个 traceback 帧（按 `func_id` 反查显示名，§8.4）。
fn push_trace_frame(
    out: &mut String,
    path: &str,
    frame: &TraceFrame,
    run: &TracedRun,
    source: Option<&Loaded>,
) {
    let name = run.frame_name(frame);
    push_frame(out, path, frame.span, Some(name.as_ref()), source);
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
    fn parse_test_collects_paths() {
        assert_eq!(
            parse_args(&args(&["test"])),
            Ok(Command::Test { paths: Vec::new() })
        );
        assert_eq!(
            parse_args(&args(&["test", "a", "b"])),
            Ok(Command::Test {
                paths: vec!["a".to_string(), "b".to_string()]
            })
        );
    }

    #[test]
    fn help_and_version_exit_zero_on_stdout() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(execute(&args(&["--help"]), &mut out, &mut err), EXIT_OK);
        let text = String::from_utf8(out).expect("UTF-8");
        assert!(text.contains("lfz run <file>"), "help: {text}");
        assert!(text.contains("lfz test"), "help 应列出 test 子命令: {text}");
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

    /// bug-20260924-09 回归：深递归（`RecursionError`）的**人类可读** traceback 按 §8.2 折叠：
    /// 首 10 帧 → `  ... 省略 {T−40} 帧 ...` → 尾 30 帧；`TracedRun.frames` 本身保持**完整**。
    #[test]
    fn deep_recursion_traceback_is_folded() {
        // `fn loop(n) { loop(n) }` 无限递归 → 命中 10000 层上限 → RecursionError。
        let src = "fn loop(n) { loop(n) }\nloop(1)\n";
        let tokens = lexer::lex(src, 0).expect("lex");
        let program = parser::parse(&tokens).expect("parse");
        let traced = evaluator::eval_module_traced(&program);
        let err = traced
            .result
            .as_ref()
            .expect_err("深递归应报 RecursionError");
        assert_eq!(err.class_name(), "RecursionError");
        // 数据层不折叠：完整帧栈（模块帧 + 10000 个用户帧）。
        assert!(
            traced.frames.len() >= 10_000,
            "TracedRun.frames 应完整保留，实得 {}",
            traced.frames.len()
        );

        let loaded = Loaded {
            text: src.to_string(),
            line_base: 0,
        };
        let rendered = render_error("deep.lfz", Some(&loaded), err, Some(&traced));

        // 人类可读渲染：帧行数 = HEAD + TAIL（省略行不以 `  File "` 开头）。
        let frame_lines = rendered
            .lines()
            .filter(|l| l.starts_with("  File \""))
            .count();
        assert_eq!(frame_lines, TRACEBACK_HEAD + TRACEBACK_TAIL, "{rendered}");

        // 省略行逐字符：`  ... 省略 {T−40} 帧 ...`。
        let omitted = traced.frames.len() - TRACEBACK_FOLD_THRESHOLD;
        assert!(
            rendered.contains(&format!("  ... 省略 {omitted} 帧 ...\n")),
            "缺少逐字符省略行：{rendered}"
        );
        assert!(
            rendered.ends_with("RecursionError: 递归深度超限（超过 10000 层）\n"),
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

    // ---- `--json`（§8.3 示例 4 / §8.4） ----------------------------------

    fn failed(err: LzError) -> CaseEval {
        CaseEval::Failed {
            err,
            loaded: None,
            traced: None,
        }
    }

    /// `CosmosAnswerError` 的成功失败对象**逐字符**等于 §8.3 示例 4。
    #[test]
    fn error_json_matches_example4_byte_for_byte() {
        let eval = failed(LzError::CosmosAnswer { span: Span::START });
        assert_eq!(
            json::encode(&error_json("forgot.lfz", &eval)),
            concat!(
                r#"{"ok":false,"error":"CosmosAnswerError","message":"你忘记了宇宙的答案","#,
                r#""file":"forgot.lfz","line":1,"col":1,"#,
                r#""traceback":[{"file":"forgot.lfz","line":1,"func":"<module>"}]}"#
            )
        );
    }

    /// 加载/解析期错误 → 合成单帧 `<module>`，位置取错误 span。
    #[test]
    fn error_json_syntax_error_uses_module_frame_and_position() {
        let eval = failed(LzError::Syntax {
            msg: SyntaxMsg::IllegalChar { c: '$' },
            span: Span::new(2, 11),
        });
        let s = json::encode(&error_json("demo.lfz", &eval));
        assert!(s.contains(r#""error":"SyntaxError""#), "{s}");
        assert!(s.contains(r#""message":"非法字符 '$'""#), "{s}");
        assert!(s.contains(r#""line":2"#), "{s}");
        assert!(s.contains(r#""col":11"#), "{s}");
        assert!(
            s.contains(r#""traceback":[{"file":"demo.lfz","line":2,"func":"<module>"}]"#),
            "{s}"
        );
    }

    /// `Io` 无 span → `line`/`col` 为 `null`、`traceback` 为 `[]`（不伪造型别）。
    #[test]
    fn error_json_io_without_span_is_null_and_empty_traceback() {
        let eval = failed(LzError::Io {
            msg: "无法读取：x.lfz".to_string(),
            span: None,
        });
        let s = json::encode(&error_json("x.lfz", &eval));
        assert!(s.contains(r#""error":"IOError""#), "{s}");
        assert!(s.contains(r#""message":"无法读取：x.lfz""#), "{s}");
        assert!(s.contains(r#""line":null"#), "{s}");
        assert!(s.contains(r#""col":null"#), "{s}");
        assert!(s.contains(r#""traceback":[]"#), "{s}");
    }

    #[test]
    fn success_json_is_ok_true_only() {
        assert_eq!(json::encode(&success_json()), r#"{"ok":true}"#);
    }

    /// `--json` 被摘除后 `run` 正常解析；失败的 `run --json` 只向 stdout 写 JSON（IOError）、
    /// stderr 为空、退出码 `2`。
    #[test]
    fn execute_run_json_writes_only_json_to_stdout() {
        let missing = std::env::temp_dir().join("lfz_cli_json_missing_98765.lfz");
        let _ = std::fs::remove_file(&missing);
        let p = missing.to_string_lossy().into_owned();
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = execute(&args(&["run", "--json", &p]), &mut out, &mut err);
        assert_eq!(code, EXIT_ERROR);
        let s = String::from_utf8(out).expect("UTF-8");
        assert_eq!(s.lines().count(), 1, "stdout 应只有一行 JSON：{s}");
        assert!(s.starts_with(r#"{"ok":false,"error":"IOError""#), "{s}");
        assert!(!s.contains("Traceback"), "JSON 模式 stdout 不得有人类可读块：{s}");
        assert!(err.is_empty(), "JSON 模式错误诊断不进 stderr（run）：{err:?}");
    }

    /// `--json` 同时出现在子命令前后均被识别；成功路径 stdout 为 `{"ok":true}`。
    #[test]
    fn execute_run_json_success_prints_ok_true() {
        let dir = std::env::temp_dir().join(format!("lfz_cli_json_ok_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("建临时目录");
        let file = dir.join("ok.lfz");
        std::fs::write(&file, "#42\nlet x = 1\n").expect("写文件");
        let p = file.to_string_lossy().into_owned();

        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = execute(&args(&["run", &p, "--json"]), &mut out, &mut err);
        assert_eq!(code, EXIT_OK, "err={err:?}");
        assert_eq!(
            String::from_utf8(out).expect("UTF-8"),
            "{\"ok\":true}\n"
        );
        assert!(err.is_empty(), "err={err:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
