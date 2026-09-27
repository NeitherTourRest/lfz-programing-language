//! `lfz run` 端到端集成测试（**无第三方依赖**：用 `std::process::Command` 调用编译出的二进制）。
//!
//! 证据口径：`docs/spec/semantics.md` §8.2 / §8.3；`.opencode/team/PLAN-P3.md` 验收命令 3–5；
//! 退出码约定见 `.opencode/team/DECISIONS.md` D-008（`0` 成功 / `1` 测试失败 / `2` 错误）。

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// 编译出的 `lfz` 二进制（Cargo 为集成测试注入的绝对路径）。
const LFZ_BIN: &str = env!("CARGO_BIN_EXE_lfz");

/// 一次运行的捕获结果。
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

/// 运行 `lfz <args...>`，捕获退出码与两个流。
fn run_lfz(args: &[&str]) -> Run {
    let output = Command::new(LFZ_BIN)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("无法启动 {LFZ_BIN}: {e}"));
    Run {
        code: output.status.code().expect("进程应正常退出（非信号）"),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

// ---------------------------------------------------------------------------
// 临时 `.lfz` 夹具（创建于系统临时目录，`Drop` 时删除；不在项目内留残留）
// ---------------------------------------------------------------------------

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempLfz(PathBuf);

impl TempLfz {
    /// 写一个内容为 `contents` 的临时 `.lfz` 文件，返回其句柄。
    fn new(contents: &str) -> Self {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let name = format!("lfz_cli_{}_{}_{}.lfz", std::process::id(), nanos, n);
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, contents).unwrap_or_else(|e| panic!("写临时文件失败: {e}"));
        TempLfz(path)
    }

    fn path(&self) -> String {
        self.0
            .to_str()
            .expect("临时路径应为 UTF-8")
            .to_string()
    }
}

impl Drop for TempLfz {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

// ---------------------------------------------------------------------------
// 验收命令 3：`lfz run examples/hello.lfz` → 退出码 0 + 正确输出
// ---------------------------------------------------------------------------

#[test]
fn hello_example_runs_and_prints() {
    let hello = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("hello.lfz");
    let hello = hello.to_str().expect("路径应为 UTF-8");

    let r = run_lfz(&["run", hello]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    assert!(
        r.stdout.contains("Hello, LFZ!"),
        "stdout={:?} stderr={:?}",
        r.stdout,
        r.stderr
    );
    assert!(r.stderr.is_empty(), "成功运行不应有 stderr：{:?}", r.stderr);
}

// ---------------------------------------------------------------------------
// 验收命令 4：缺 `#42` 的 `.lfz` → 退出码 2 + CosmosAnswerError（无源码行 / 无 Traceback）
// ---------------------------------------------------------------------------

#[test]
fn missing_preamble_exits_2_with_cosmos_answer() {
    let f = TempLfz::new("print(\"hi\")\n");
    let r = run_lfz(&["run", &f.path()]);

    assert_eq!(r.code, 2, "stderr={}", r.stderr);
    let err = &r.stderr;
    assert!(
        err.contains("CosmosAnswerError: 你忘记了宇宙的答案"),
        "stderr={err:?}"
    );
    assert!(err.contains("File \""), "应含 File 行：{err:?}");
    assert!(
        !err.contains("Traceback"),
        "加载期错误不得有 Traceback 头：{err:?}"
    );
    // §8.3 示例 3：无源码行 / 无插入符。
    assert!(!err.contains('^'), "CosmosAnswerError 不得有插入符：{err:?}");
}

// ---------------------------------------------------------------------------
// 验收命令 5：语法 / 运行错误 → 退出码 2
// ---------------------------------------------------------------------------

#[test]
fn syntax_error_exits_2_with_position_and_no_traceback() {
    // `$` 位于第 2 行第 11 列（1-based）。
    let f = TempLfz::new("#42\nlet x = 1 $ 2\n");
    let r = run_lfz(&["run", &f.path()]);

    assert_eq!(r.code, 2, "stderr={}", r.stderr);
    let err = &r.stderr;
    assert!(err.contains("SyntaxError: 非法字符 '$'"), "stderr={err:?}");
    assert!(err.contains("line 2"), "应含位置行号：{err:?}");
    assert!(err.contains('^'), "应含插入符：{err:?}");
    assert!(
        !err.contains("Traceback"),
        "解析期错误不得有 Traceback 头：{err:?}"
    );
}

#[test]
fn runtime_error_exits_2_with_traceback() {
    let f = TempLfz::new("#42\n1 / 0\n");
    let r = run_lfz(&["run", &f.path()]);

    assert_eq!(r.code, 2, "stderr={}", r.stderr);
    let err = &r.stderr;
    assert!(
        err.contains("Traceback (most recent call last):"),
        "运行期错误必须有 Traceback 头：{err:?}"
    );
    assert!(
        err.contains("ZeroDivisionError: 除以零"),
        "stderr={err:?}"
    );
    assert!(err.contains(", in <module>"), "顶层帧应显示 <module>：{err:?}");
}

// ---------------------------------------------------------------------------
// CLI 自身：`--help` / `--version` / 缺文件 / 参数错误
// ---------------------------------------------------------------------------

#[test]
fn help_and_version_exit_0() {
    let h = run_lfz(&["--help"]);
    assert_eq!(h.code, 0, "stderr={}", h.stderr);
    assert!(h.stdout.contains("lfz run <file>"), "stdout={:?}", h.stdout);

    let v = run_lfz(&["--version"]);
    assert_eq!(v.code, 0, "stderr={}", v.stderr);
    assert!(
        v.stdout.contains(env!("CARGO_PKG_VERSION")),
        "stdout={:?}",
        v.stdout
    );
}

#[test]
fn missing_file_is_io_error_exit_2() {
    let missing = std::env::temp_dir()
        .join("lfz_cli_definitely_missing_12345.lfz")
        .to_str()
        .expect("路径应为 UTF-8")
        .to_string();
    // 确保确实不存在。
    let _ = std::fs::remove_file(&missing);

    let r = run_lfz(&["run", &missing]);
    assert_eq!(r.code, 2, "stderr={}", r.stderr);
    assert!(r.stderr.contains("IOError: 无法读取："), "stderr={:?}", r.stderr);
}

#[test]
fn no_args_is_usage_error_exit_2() {
    let r = run_lfz(&[]);
    assert_eq!(r.code, 2, "stderr={}", r.stderr);
    assert!(r.stderr.contains("缺少子命令"), "stderr={:?}", r.stderr);
}

// ---------------------------------------------------------------------------
// `--json`（P4.2）：stdout 只写一行 JSON；退出码仍 D-008（错误 → 2）
// 权威字段：`docs/spec/semantics.md` §8.3 示例 4 / §8.4。
// ---------------------------------------------------------------------------

/// JSON 顶层标量区（`traceback` 之前），避免与 traceback 内嵌字段混淆。
fn head(json: &str) -> &str {
    json.find("\"traceback\"").map_or(json, |i| &json[..i])
}

/// 成功：`{"ok":true}`（单行）、stderr 空、退出码 0。
#[test]
fn json_run_success_is_ok_true_exit_0() {
    let f = TempLfz::new("#42\nlet x = 1\n");
    let r = run_lfz(&["run", "--json", &f.path()]);

    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    assert_eq!(r.stdout, "{\"ok\":true}\n", "stdout={:?}", r.stdout);
    assert!(r.stderr.is_empty(), "成功不应有 stderr：{:?}", r.stderr);
}

/// 缺 `#42`：`CosmosAnswerError`，`ok:false`，`line:1`/`col:1`，单帧 `<module>`，退出码 2。
#[test]
fn json_run_missing_preamble_shape() {
    let f = TempLfz::new("print(\"hi\")\n");
    let r = run_lfz(&["run", "--json", &f.path()]);

    assert_eq!(r.code, 2, "stderr={}", r.stderr);
    assert_eq!(r.stdout.lines().count(), 1, "stdout={:?}", r.stdout);
    let h = head(&r.stdout);
    assert!(h.starts_with("{\"ok\":false"), "stdout={:?}", r.stdout);
    assert!(h.contains(r#""error":"CosmosAnswerError""#), "{:?}", r.stdout);
    assert!(
        h.contains(r#""message":"你忘记了宇宙的答案""#),
        "{:?}",
        r.stdout
    );
    assert!(h.contains(r#""line":1"#), "{:?}", r.stdout);
    assert!(h.contains(r#""col":1"#), "{:?}", r.stdout);
    assert!(
        r.stdout
            .contains(r#""traceback":[{"file":"#)
            && r.stdout.contains(r#""line":1,"func":"<module>"}]"#),
        "traceback 应为单帧 <module>：{:?}",
        r.stdout
    );
    // JSON 模式不写人类可读块到 stdout。
    assert!(!r.stdout.contains("Traceback"), "{:?}", r.stdout);
    assert!(r.stderr.is_empty(), "JSON 模式失败也不写 stderr：{:?}", r.stderr);
}

/// 语法错：`SyntaxError`，位置 `line:2`/`col:11`，退出码 2。
#[test]
fn json_run_syntax_error_shape() {
    let f = TempLfz::new("#42\nlet x = 1 $ 2\n");
    let r = run_lfz(&["run", "--json", &f.path()]);

    assert_eq!(r.code, 2, "stderr={}", r.stderr);
    let h = head(&r.stdout);
    assert!(h.contains(r#""error":"SyntaxError""#), "{:?}", r.stdout);
    assert!(h.contains(r#""message":"非法字符 '$'""#), "{:?}", r.stdout);
    assert!(h.contains(r#""line":2"#), "{:?}", r.stdout);
    assert!(h.contains(r#""col":11"#), "{:?}", r.stdout);
    assert!(r.stderr.is_empty(), "{:?}", r.stderr);
}

/// 运行期错：`ZeroDivisionError`，`line:3`/`col:5`，traceback 自外→内（§8.3 示例 2）。
#[test]
fn json_run_runtime_error_matches_example2_traceback() {
    let f = TempLfz::new("#42\nfn half(n) {\n    n / 0\n}\nlet r = half(10)\nprint(r)\n");
    let r = run_lfz(&["run", "--json", &f.path()]);

    assert_eq!(r.code, 2, "stderr={}", r.stderr);
    let h = head(&r.stdout);
    assert!(h.contains(r#""error":"ZeroDivisionError""#), "{:?}", r.stdout);
    assert!(h.contains(r#""message":"除以零""#), "{:?}", r.stdout);
    assert!(h.contains(r#""line":3"#), "{:?}", r.stdout);
    assert!(h.contains(r#""col":5"#), "{:?}", r.stdout);
    // 帧表：外层 `<module>`@5 → 内层 `half`@3。
    assert!(r.stdout.contains(r#""line":5,"func":"<module>""#), "{:?}", r.stdout);
    assert!(r.stdout.contains(r#""line":3,"func":"half""#), "{:?}", r.stdout);
    // `print(r)` 未执行 → stdout 无程序输出污染。
    assert_eq!(r.stdout.lines().count(), 1, "stdout={:?}", r.stdout);
}

/// 断言失败：`AssertionError`，`line:2`/`col:1`，退出码 2（`run` 不区分 1）。
#[test]
fn json_run_assert_failure_shape() {
    let f = TempLfz::new("#42\nassert(false, \"boom\")\n");
    let r = run_lfz(&["run", "--json", &f.path()]);

    assert_eq!(r.code, 2, "stderr={}", r.stderr);
    let h = head(&r.stdout);
    assert!(h.contains(r#""error":"AssertionError""#), "{:?}", r.stdout);
    assert!(h.contains(r#""message":"断言失败：boom""#), "{:?}", r.stdout);
    assert!(h.contains(r#""line":2"#), "{:?}", r.stdout);
    assert!(h.contains(r#""col":1"#), "{:?}", r.stdout);
    assert!(r.stderr.is_empty(), "{:?}", r.stderr);
}

/// 缺文件 → `IOError`（无 span）：`line`/`col` 为 `null`、`traceback` 为 `[]`，退出码 2。
#[test]
fn json_run_missing_file_io_error_shape() {
    let missing = std::env::temp_dir()
        .join("lfz_cli_json_definitely_missing_54321.lfz")
        .to_str()
        .expect("路径应为 UTF-8")
        .to_string();
    let _ = std::fs::remove_file(&missing);

    let r = run_lfz(&["run", "--json", &missing]);
    assert_eq!(r.code, 2, "stderr={}", r.stderr);
    let h = head(&r.stdout);
    assert!(h.contains(r#""error":"IOError""#), "{:?}", r.stdout);
    assert!(h.contains(r#""line":null"#), "{:?}", r.stdout);
    assert!(h.contains(r#""col":null"#), "{:?}", r.stdout);
    assert!(r.stdout.contains(r#""traceback":[]"#), "{:?}", r.stdout);
}

/// `--json` 出现在 file 之后同样被识别。
#[test]
fn json_flag_accepted_after_file() {
    let f = TempLfz::new("#42\nlet x = 1\n");
    let r = run_lfz(&["run", &f.path(), "--json"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    assert_eq!(r.stdout, "{\"ok\":true}\n", "stdout={:?}", r.stdout);
}

// ---------------------------------------------------------------------------
// P4.2-fix：`print` 在 `--json` 下重定向到 stderr，stdout 恒为唯一合法 JSON
// ---------------------------------------------------------------------------

/// 程序输出标记（避免与临时路径等偶然子串混淆）。
const PRINT_MARK: &str = "__LFZ_PRINT_MARKER__";

/// 成功路径：程序 `print("x")` + `run --json` → stdout **恰为** `{"ok":true}`
/// （合法 JSON，可被任意解析器解析），`x` 出现在 **stderr**、不在 stdout。
#[test]
fn json_run_print_success_redirects_to_stderr() {
    let f = TempLfz::new(&format!("#42\nprint(\"{PRINT_MARK}\")\n"));
    let r = run_lfz(&["run", "--json", &f.path()]);

    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    // stdout 恰为一个合法 JSON 对象（逐字符），证明未混入程序输出。
    assert_eq!(r.stdout, "{\"ok\":true}\n", "stdout={:?}", r.stdout);
    assert!(
        !r.stdout.contains(PRINT_MARK),
        "print 不得写 stdout：stdout={:?}",
        r.stdout
    );
    assert!(
        r.stderr.contains(PRINT_MARK),
        "print 应重定向到 stderr：stderr={:?}",
        r.stderr
    );
}

/// 失败路径：程序先 `print("x")` 再运行期报错 + `run --json` → stdout 仍是**唯一一行合法 JSON**
/// （§8.3 示例 4 形状），`x` 只在 stderr。
#[test]
fn json_run_print_before_error_redirects_to_stderr() {
    let f = TempLfz::new(&format!("#42\nprint(\"{PRINT_MARK}\")\n1 / 0\n"));
    let r = run_lfz(&["run", "--json", &f.path()]);

    assert_eq!(r.code, 2, "stderr={}", r.stderr);
    assert_eq!(r.stdout.lines().count(), 1, "stdout={:?}", r.stdout);
    // 单行且为 `{...}` 对象（合法 JSON），且为错误形状。
    let s = r.stdout.trim_end();
    assert!(
        s.starts_with("{\"ok\":false") && s.ends_with('}'),
        "stdout 应为单个 JSON 对象：{:?}",
        r.stdout
    );
    let h = head(&r.stdout);
    assert!(h.contains(r#""error":"ZeroDivisionError""#), "{:?}", r.stdout);
    assert!(h.contains(r#""message":"除以零""#), "{:?}", r.stdout);
    assert!(
        !r.stdout.contains(PRINT_MARK),
        "print 不得写 stdout：stdout={:?}",
        r.stdout
    );
    assert!(
        r.stderr.contains(PRINT_MARK),
        "print 应重定向到 stderr：stderr={:?}",
        r.stderr
    );
}
