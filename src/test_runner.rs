//! `lfz test` 一键黑盒测试 runner（P4.1，评分项 2 的基础设施）。
//!
//! # 单一事实源（只读引用）
//!
//! - `docs/spec/interface-contract.md` §11.2（test-runner 契约 **T-R1 … T-R4**）、§8.1（错误类 + 退出码）。
//! - `docs/spec/semantics.md` §8.2 / §8.3（失败输出的位置信息与格式）、§4.5.10（`assert` / `check` / `fail`）。
//! - `.opencode/team/DECISIONS.md` D-008（退出码 `0` / `1` / `2`）。
//! - 派生说明（发现规则全文 / `cases.json` schema / 裁定与输出格式）：`docs/tooling/runner-contract.md`。
//!
//! # 判定模型（契约裁定见 `docs/tooling/runner-contract.md`）
//!
//! | 情形 | 判定 | 退出码贡献 |
//! |---|---|---|
//! | 无 `LfzError`（`check` 失败非致命，A4） | `PASS` | 0 |
//! | 清单声明 `expect.error` 且实际类名一致（负例，T-R2） | `PASS` | 0 |
//! | 仅 `AssertionError`（`assert` / `fail`） | `FAIL` | 1 |
//! | 其余任一错误类（含缺 `#42` → `CosmosAnswerError`，T-R1） | `ERROR` | 2 |
//! | 清单声明 `expect` 但类名不符 / 未报错 | `FAIL` | 1 |
//!
//! 整体退出码：存在 `ERROR` → `2`；否则存在 `FAIL` → `1`；否则 `0`（`ERROR` 优先于 `FAIL`）。
//!
//! 本模块属 **bin crate**（`main.rs` 内 `mod test_runner;`），不进库 crate。

use crate::cli::{self, CaseEval};
use crate::json::{self, Json};
use std::io::Write;
use std::path::{Path, PathBuf};

/// 默认发现根目录（§11.2 T-R4）。
const DEFAULT_ROOT: &str = "tests";
/// 清单文件名（§11.2 T-R2）。
const MANIFEST_NAME: &str = "cases.json";
/// 自动发现时**跳过**的目录名：`tests/fixtures/**` 不得被自动发现（T-R2 / T-R4）。
const EXCLUDED_DIR: &str = "fixtures";
/// 12 个错误类名（§8.1），用于校验清单 `expect.error`。
const ERROR_CLASSES: [&str; 12] = [
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
];

/// 一个待运行用例。
struct Case {
    /// 传给 `loader::load_file` 的路径（相对 / 绝对均可）。
    path: String,
    /// 报告中显示的用例名（默认 = `path`；清单可用 `name` 覆盖）。
    display: String,
    /// 清单声明的期望错误类名（`None` = 期望正常结束）。
    expect: Option<String>,
}

/// 单用例判定。
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// 通过。
    Pass,
    /// 测试失败（`assert` / `fail`，或期望未达成）。
    Fail,
    /// 意外错误（非 `AssertionError` 的任一错误类）。
    Error,
}

/// runner 入口：发现 → 逐用例执行 → 打印报告 → 返回退出码。
///
/// `out` = 测试报告（每用例一行 + 失败明细 + 汇总结论）；`err` = runner 自身错误（参数 / 环境）。
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let cases = match discover(args, err) {
        Ok(c) => c,
        Err(()) => return cli::EXIT_ERROR,
    };
    if cases.is_empty() {
        let _ = writeln!(err, "lfz test: 未发现任何测试用例");
        return cli::EXIT_ERROR;
    }

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut errored = 0usize;
    for case in &cases {
        let eval = cli::eval_case(&case.path);
        match judge(&eval, case.expect.as_deref()) {
            Verdict::Pass => {
                passed += 1;
                let _ = writeln!(out, "{:<5} {}", "PASS", case.display);
            }
            Verdict::Fail => {
                failed += 1;
                report(out, "FAIL", case, &eval);
            }
            Verdict::Error => {
                errored += 1;
                report(out, "ERROR", case, &eval);
            }
        }
    }

    let total = cases.len();
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "汇总：共 {total} 个用例，通过 {passed}，失败 {failed}，错误 {errored}"
    );

    if errored > 0 {
        cli::EXIT_ERROR
    } else if failed > 0 {
        cli::EXIT_TEST_FAIL
    } else {
        cli::EXIT_OK
    }
}

/// 判定单用例（契约见模块头表）。
fn judge(eval: &CaseEval, expect: Option<&str>) -> Verdict {
    let class = eval.error_class();
    match expect {
        // 负例（T-R2）：类名一致 → 通过；未报错或类名不符 → 失败。
        Some(exp) => {
            if class == Some(exp) {
                Verdict::Pass
            } else {
                Verdict::Fail
            }
        }
        // 正例：无错 → 通过；仅 AssertionError → 失败；其余错误类 → error（D-008）。
        None => match class {
            None => Verdict::Pass,
            Some("AssertionError") => Verdict::Fail,
            Some(_) => Verdict::Error,
        },
    }
}

/// 输出一个非 `PASS` 用例：状态行 + `(路径:行:列)` + 缩进的 §8.2 错误块。
fn report(out: &mut dyn Write, status: &str, case: &Case, eval: &CaseEval) {
    let mut line = format!("{status:<5} {}", case.display);
    if let Some(span) = eval.error().and_then(|e| e.span()) {
        line.push_str(&format!("  ({}:{}:{})", case.path, span.line, span.col));
    }
    let _ = writeln!(out, "{line}");
    if let Some(block) = eval.render(&case.path) {
        for l in block.lines() {
            let _ = writeln!(out, "  {l}");
        }
    }
}

/// 发现用例集（T-R4）：
/// - 无参数 → 根 = `tests`，递归发现 `*.lfz`（按 `loader::is_lfz`），**跳过 `fixtures/`**；
/// - 目录参数 → 同上（相对该目录）；
/// - 文件参数 → 直接作为用例（不做扩展名过滤）。
///
/// 每个被扫描的目录根若含 `cases.json`，则据清单附加 `expect` 或新增夹具用例（T-R2）。
fn discover(args: &[String], err: &mut dyn Write) -> Result<Vec<Case>, ()> {
    let mut cases: Vec<Case> = Vec::new();
    if args.is_empty() {
        let root = PathBuf::from(DEFAULT_ROOT);
        if !root.is_dir() {
            let _ = writeln!(err, "lfz test: 未发现测试目录 '{DEFAULT_ROOT}'");
            return Err(());
        }
        add_dir(&root, &mut cases, err)?;
    } else {
        for arg in args {
            let p = PathBuf::from(arg);
            if p.is_dir() {
                add_dir(&p, &mut cases, err)?;
            } else if p.is_file() {
                push_case(&mut cases, &p, None, None);
            } else {
                let _ = writeln!(err, "lfz test: 路径不存在：{arg}");
                return Err(());
            }
        }
    }
    // 稳定排序 → 输出可复现。
    cases.sort_by(|a, b| norm(&a.path).cmp(&norm(&b.path)));
    Ok(cases)
}

/// 扫描一个目录根：递归收集 `.lfz` + 应用同目录 `cases.json`。
fn add_dir(root: &Path, cases: &mut Vec<Case>, err: &mut dyn Write) -> Result<(), ()> {
    let mut files = Vec::new();
    collect_lfz(root, &mut files);
    for f in files {
        push_case(cases, &f, None, None);
    }
    let manifest = root.join(MANIFEST_NAME);
    if manifest.is_file() {
        apply_manifest(&manifest, root, cases, err)?;
    }
    Ok(())
}

/// 递归收集 `*.lfz`（`loader::is_lfz`，ASCII 大小写不敏感），跳过名为 `fixtures` 的目录。
///
/// 子目录按路径排序后访问 → 结果确定。
fn collect_lfz(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for p in paths {
        if p.is_dir() {
            let excluded = p
                .file_name()
                .is_some_and(|n| n == std::ffi::OsStr::new(EXCLUDED_DIR));
            if excluded {
                continue;
            }
            collect_lfz(&p, out);
        } else if lfz::loader::is_lfz(&p.to_string_lossy()) {
            out.push(p);
        }
    }
}

/// 把一个路径推入用例集（路径统一规范化为 `/` 分隔，跨平台输出一致）。
fn push_case(cases: &mut Vec<Case>, path: &Path, display: Option<String>, expect: Option<String>) {
    let p = norm(&path.to_string_lossy());
    cases.push(Case {
        display: display.unwrap_or_else(|| p.clone()),
        path: p,
        expect,
    });
}

/// 读取并应用 `cases.json`（T-R2）：`expect` 附加到已发现用例，或新增非自动发现的夹具。
fn apply_manifest(
    manifest: &Path,
    root: &Path,
    cases: &mut Vec<Case>,
    err: &mut dyn Write,
) -> Result<(), ()> {
    let text = match std::fs::read_to_string(manifest) {
        Ok(t) => t,
        Err(e) => {
            let _ = writeln!(err, "lfz test: 无法读取清单 {}：{e}", manifest.display());
            return Err(());
        }
    };
    let doc = match json::parse(&text) {
        Ok(v) => v,
        Err(e) => {
            let _ = writeln!(err, "lfz test: 清单 {} 不是合法 JSON：{e}", manifest.display());
            return Err(());
        }
    };
    let list = match doc.get("cases").and_then(Json::as_array) {
        Some(a) => a,
        None => {
            let _ = writeln!(err, "lfz test: 清单 {} 缺少 'cases' 数组", manifest.display());
            return Err(());
        }
    };

    for (i, item) in list.iter().enumerate() {
        let no = i + 1;
        let rel = match item.get("path").and_then(Json::as_str) {
            Some(s) => s,
            None => {
                let _ = writeln!(
                    err,
                    "lfz test: 清单 {} 第 {no} 项缺少字符串 'path'",
                    manifest.display()
                );
                return Err(());
            }
        };
        let expect = match item
            .get("expect")
            .and_then(|e| e.get("error"))
            .and_then(Json::as_str)
        {
            Some(cls) => {
                if !ERROR_CLASSES.contains(&cls) {
                    let _ = writeln!(
                        err,
                        "lfz test: 清单 {} 第 {no} 项 'expect.error' 未知错误类 '{cls}'",
                        manifest.display()
                    );
                    return Err(());
                }
                Some(cls.to_string())
            }
            None => None,
        };
        let name = item.get("name").and_then(Json::as_str).map(str::to_string);

        // 清单路径相对清单所在目录（= root）。
        let full = root.join(rel);
        let full_str = norm(&full.to_string_lossy());
        if let Some(existing) = cases.iter_mut().find(|c| c.path == full_str) {
            existing.expect = expect;
            if let Some(n) = name {
                existing.display = n;
            }
        } else {
            push_case(cases, &full, name, expect);
        }
    }
    Ok(())
}

/// 路径规范化（`\` → `/`），用于发现结果与清单路径的比较（Windows 友好）。
fn norm(p: &str) -> String {
    p.replace('\\', "/")
}

// ===========================================================================
// 测试（全部使用系统临时目录，**不在项目内**留任何夹具）
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// 系统临时目录下的一次性目录，`Drop` 时递归删除。
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos());
            let p = std::env::temp_dir().join(format!(
                "lfz_tr_{}_{}_{}",
                std::process::id(),
                nanos,
                n
            ));
            std::fs::create_dir_all(&p).expect("建临时目录");
            TempDir(p)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn str(&self) -> String {
            self.0.to_string_lossy().into_owned()
        }

        /// 写一个（可含子目录的）文件。
        fn file(&self, rel: &str, contents: &str) {
            let p = self.0.join(rel);
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent).expect("建父目录");
            }
            std::fs::write(&p, contents).expect("写文件");
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// 以内存缓冲运行 runner，返回 `(退出码, stdout, stderr)`。
    fn run(args: &[&str]) -> (i32, String, String) {
        let argv: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = super::run(&argv, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).expect("stdout UTF-8"),
            String::from_utf8(err).expect("stderr UTF-8"),
        )
    }

    // ---- T-R1：缺 `#42` 前导 → error（退出码 2） -------------------------

    #[test]
    fn tr1_missing_preamble_is_error_exit_2() {
        let d = TempDir::new();
        d.file("a.lfz", "print(\"hi\")\n"); // .lfz 但无 #42
        let (code, out, _err) = run(&[&d.str()]);
        assert_eq!(code, cli::EXIT_ERROR, "out={out}");
        assert!(out.contains("ERROR"), "out={out}");
        assert!(out.contains("CosmosAnswerError"), "out={out}");
        assert!(out.contains("通过 0，失败 0，错误 1"), "out={out}");
    }

    // ---- T-R4：自动发现排除 fixtures/ ------------------------------------

    #[test]
    fn tr4_fixtures_dir_excluded_from_discovery() {
        let d = TempDir::new();
        d.file("ok.lfz", "#42\nassert(true, \"ok\")\n");
        d.file("fixtures/bad.lfz", "no preamble\n"); // 不得被发现
        let (code, out, _err) = run(&[&d.str()]);
        assert_eq!(code, cli::EXIT_OK, "out={out}");
        assert_eq!(out.matches("PASS").count(), 1, "out={out}");
        assert!(!out.contains("ERROR"), "fixtures 不应被发现：{out}");
    }

    // ---- T-R2：清单声明期望错误类 → 负例判为通过；夹具非自动发现 ---------

    #[test]
    fn tr2_manifest_negative_fixture_passes() {
        let d = TempDir::new();
        d.file("fixtures/neg.lfz", "print(\"x\")\n"); // 无 #42，被 fixtures 排除
        d.file(
            "cases.json",
            r#"{ "cases": [ { "path": "fixtures/neg.lfz", "expect": { "error": "CosmosAnswerError" } } ] }"#,
        );
        let (code, out, _err) = run(&[&d.str()]);
        assert_eq!(code, cli::EXIT_OK, "out={out}");
        assert!(out.contains("PASS"), "out={out}");
    }

    #[test]
    fn tr2_manifest_expect_mismatch_is_fail() {
        let d = TempDir::new();
        d.file("fixtures/neg.lfz", "no preamble\n"); // 实际 CosmosAnswerError
        d.file(
            "cases.json",
            r#"{ "cases": [ { "path": "fixtures/neg.lfz", "expect": { "error": "SyntaxError" } } ] }"#,
        );
        let (code, out, _err) = run(&[&d.str()]);
        assert_eq!(code, cli::EXIT_TEST_FAIL, "out={out}");
        assert!(out.contains("FAIL"), "out={out}");
    }

    #[test]
    fn manifest_unknown_error_class_is_env_error() {
        let d = TempDir::new();
        d.file("ok.lfz", "#42\nassert(true, \"ok\")\n");
        d.file(
            "cases.json",
            r#"{ "cases": [ { "path": "ok.lfz", "expect": { "error": "NopeError" } } ] }"#,
        );
        let (code, _out, err) = run(&[&d.str()]);
        assert_eq!(code, cli::EXIT_ERROR);
        assert!(err.contains("未知错误类 'NopeError'"), "err={err}");
    }

    #[test]
    fn malformed_manifest_is_env_error() {
        let d = TempDir::new();
        d.file("ok.lfz", "#42\nassert(true, \"ok\")\n");
        d.file("cases.json", "{ not json ");
        let (code, _out, err) = run(&[&d.str()]);
        assert_eq!(code, cli::EXIT_ERROR);
        assert!(err.contains("不是合法 JSON"), "err={err}");
    }

    // ---- T-R3：仅 `AssertionError` → failure；位置信息按 §8.2 -------------

    #[test]
    fn tr3_assert_failure_is_fail_exit_1_with_position() {
        let d = TempDir::new();
        d.file("bad.lfz", "#42\nassert(false, \"1+1==2\")\n");
        let (code, out, _err) = run(&[&d.str()]);
        assert_eq!(code, cli::EXIT_TEST_FAIL, "out={out}");
        assert!(out.contains("FAIL"), "out={out}");
        assert!(out.contains("AssertionError: 断言失败：1+1==2"), "out={out}");
        // §8.2 位置信息：`line 2`（文件行）+ 插入符行。
        assert!(out.contains("line 2"), "out={out}");
        assert!(out.contains('^'), "out={out}");
    }

    #[test]
    fn tr3_other_error_class_is_error_exit_2() {
        let d = TempDir::new();
        d.file("boom.lfz", "#42\n1 / 0\n");
        let (code, out, _err) = run(&[&d.str()]);
        assert_eq!(code, cli::EXIT_ERROR, "out={out}");
        assert!(out.contains("ERROR"), "out={out}");
        assert!(out.contains("ZeroDivisionError: 除以零"), "out={out}");
    }

    // ---- 端到端（内存）：混合通过 / 失败 ---------------------------------

    #[test]
    fn mixed_pass_and_fail_reports_counts_exit_1() {
        let d = TempDir::new();
        d.file("a_pass.lfz", "#42\nassert(true, \"ok\")\n");
        d.file("b_fail.lfz", "#42\nassert(false, \"boom\")\n");
        let (code, out, _err) = run(&[&d.str()]);
        assert_eq!(code, cli::EXIT_TEST_FAIL, "out={out}");
        assert_eq!(out.matches("PASS").count(), 1, "out={out}");
        assert_eq!(out.matches("FAIL").count(), 1, "out={out}");
        assert!(out.contains("通过 1，失败 1，错误 0"), "out={out}");
    }

    #[test]
    fn error_dominates_fail_in_exit_code() {
        let d = TempDir::new();
        d.file("a_fail.lfz", "#42\nassert(false, \"boom\")\n");
        d.file("b_err.lfz", "#42\n1 / 0\n");
        let (code, out, _err) = run(&[&d.str()]);
        assert_eq!(code, cli::EXIT_ERROR, "out={out}");
        assert!(out.contains("通过 0，失败 1，错误 1"), "out={out}");
    }

    // ---- 其它：空目录 / 显式文件 / 缺目录 ---------------------------------

    #[test]
    fn empty_dir_is_env_error() {
        let d = TempDir::new();
        let (code, _out, err) = run(&[&d.str()]);
        assert_eq!(code, cli::EXIT_ERROR);
        assert!(err.contains("未发现任何测试用例"), "err={err}");
    }

    #[test]
    fn explicit_file_is_run() {
        let d = TempDir::new();
        d.file("one.lfz", "#42\nassert(true, \"ok\")\n");
        let f = d.path().join("one.lfz").to_string_lossy().into_owned();
        let (code, out, _err) = run(&[&f]);
        assert_eq!(code, cli::EXIT_OK, "out={out}");
        assert!(out.contains("PASS"), "out={out}");
    }

    #[test]
    fn missing_path_is_env_error() {
        let d = TempDir::new();
        let missing = d.path().join("nope").to_string_lossy().into_owned();
        let (code, _out, err) = run(&[&missing]);
        assert_eq!(code, cli::EXIT_ERROR);
        assert!(err.contains("路径不存在"), "err={err}");
    }
}
