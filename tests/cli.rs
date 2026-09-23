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
