//! `lfz test` 端到端集成测试（**无第三方依赖**：`std::process::Command` 调用编译出的二进制）。
//!
//! 覆盖：默认发现（T-R4）、缺 `#42` → error（T-R1）、清单期望（T-R2）、
//! `check` 非致命（T-R3 / A4）、退出码（D-008）。契约见 `docs/spec/interface-contract.md` §11.2
//! 与 `docs/tooling/runner-contract.md`。
//!
//! 所有夹具均建于**系统临时目录**并在 `Drop` 时删除，**不在项目内**留任何文件。

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// 编译出的 `lfz` 二进制（Cargo 为集成测试注入的绝对路径）。
const LFZ_BIN: &str = env!("CARGO_BIN_EXE_lfz");

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
            "lfz_e2e_{}_{}_{}",
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

/// 一次运行的捕获结果。
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

/// 在 `cwd` 下运行 `lfz <args...>`。
fn run_in(cwd: &Path, args: &[&str]) -> Run {
    let output = Command::new(LFZ_BIN)
        .current_dir(cwd)
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
// T-R4：默认发现 `tests/**/*.lfz`（cwd 下的 tests/）
// ---------------------------------------------------------------------------

#[test]
fn default_discovery_reports_mixed_pass_and_fail() {
    let d = TempDir::new();
    d.file("tests/a_pass.lfz", "#42\nassert(true, \"ok\")\n");
    d.file("tests/b_fail.lfz", "#42\nassert(false, \"boom\")\n");
    let r = run_in(d.path(), &["test"]);

    assert_eq!(r.code, 1, "stdout={} stderr={}", r.stdout, r.stderr);
    assert!(r.stdout.contains("PASS"), "stdout={}", r.stdout);
    assert!(r.stdout.contains("FAIL"), "stdout={}", r.stdout);
    assert!(
        r.stdout.contains("通过 1，失败 1，错误 0"),
        "stdout={}",
        r.stdout
    );
}

#[test]
fn nested_and_fixtures_exclusion() {
    let d = TempDir::new();
    d.file("tests/sub/deep.lfz", "#42\nassert(true, \"ok\")\n");
    d.file("tests/fixtures/neg.lfz", "no preamble\n"); // 排除，不得被发现
    let r = run_in(d.path(), &["test"]);

    assert_eq!(r.code, 0, "stdout={} stderr={}", r.stdout, r.stderr);
    assert_eq!(
        r.stdout.matches("PASS").count(),
        1,
        "仅 sub/deep.lfz 应被运行：{}",
        r.stdout
    );
}

// ---------------------------------------------------------------------------
// T-R1：`.lfz` 缺 `#42` → error（退出码 2）
// ---------------------------------------------------------------------------

#[test]
fn missing_preamble_in_tests_is_error_exit_2() {
    let d = TempDir::new();
    d.file("tests/no_pre.lfz", "print(\"x\")\n");
    let r = run_in(d.path(), &["test"]);

    assert_eq!(r.code, 2, "stdout={} stderr={}", r.stdout, r.stderr);
    assert!(r.stdout.contains("ERROR"), "stdout={}", r.stdout);
    assert!(
        r.stdout.contains("CosmosAnswerError"),
        "stdout={}",
        r.stdout
    );
}

// ---------------------------------------------------------------------------
// T-R3 / A4：`check` 失败非致命 → PASS + 退出码 0（警告写 stderr）
// ---------------------------------------------------------------------------

#[test]
fn check_failure_is_non_fatal_pass() {
    let d = TempDir::new();
    d.file(
        "tests/soft.lfz",
        "#42\ncheck(false, \"soft\")\nassert(true, \"still ok\")\n",
    );
    let r = run_in(d.path(), &["test"]);

    assert_eq!(r.code, 0, "stdout={} stderr={}", r.stdout, r.stderr);
    assert!(r.stdout.contains("PASS"), "stdout={}", r.stdout);
    assert!(
        r.stderr.contains("check 失败：soft"),
        "check 警告应写 stderr：stderr={}",
        r.stderr
    );
}

// ---------------------------------------------------------------------------
// T-R2：清单声明期望错误类 → 负例判为 PASS
// ---------------------------------------------------------------------------

#[test]
fn manifest_negative_fixture_passes() {
    let d = TempDir::new();
    d.file("tests/fixtures/neg.lfz", "print(\"no preamble\")\n");
    d.file(
        "tests/cases.json",
        r#"{ "cases": [ { "path": "fixtures/neg.lfz", "expect": { "error": "CosmosAnswerError" } } ] }"#,
    );
    let r = run_in(d.path(), &["test"]);

    assert_eq!(r.code, 0, "stdout={} stderr={}", r.stdout, r.stderr);
    assert!(r.stdout.contains("PASS"), "stdout={}", r.stdout);
}

// ---------------------------------------------------------------------------
// 显式路径参数：目录名 → 发现；文件 → 直接运行
// ---------------------------------------------------------------------------

#[test]
fn explicit_directory_argument() {
    let d = TempDir::new();
    d.file("cases/one.lfz", "#42\nassert(true, \"ok\")\n");
    let sub = d.path().join("cases");
    let sub = sub.to_str().expect("路径应为 UTF-8");
    let r = run_in(d.path(), &["test", sub]);

    assert_eq!(r.code, 0, "stdout={} stderr={}", r.stdout, r.stderr);
    assert!(r.stdout.contains("PASS"), "stdout={}", r.stdout);
}

// ---------------------------------------------------------------------------
// CLI：`--help` 必须列出 `lfz test`
// ---------------------------------------------------------------------------

#[test]
fn help_mentions_test_subcommand() {
    let r = run_in(std::env::temp_dir().as_path(), &["--help"]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    assert!(r.stdout.contains("lfz test"), "stdout={}", r.stdout);
}
