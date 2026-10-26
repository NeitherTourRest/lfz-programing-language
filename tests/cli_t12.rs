//! T12 CLI regression tests. Fixtures stay inside project Temp/.
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
const BIN: &str = env!("CARGO_BIN_EXE_lfz");
static SERIAL: AtomicU64 = AtomicU64::new(0);
fn run(args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(BIN).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}
fn ok(args: &[&str], input: &[u8], expected: &str) {
    let r = run(args, input);
    assert_eq!(r.status.code(), Some(0), "stderr={}", String::from_utf8_lossy(&r.stderr));
    assert_eq!(String::from_utf8(r.stdout).unwrap(), expected);
    assert!(r.stderr.is_empty(), "stderr={:?}", r.stderr);
}
fn bad(args: &[&str], input: &[u8], expected: &str) -> Output {
    let r = run(args, input);
    assert_eq!(r.status.code(), Some(2), "out={:?} err={:?}", r.stdout, r.stderr);
    let combined = format!("{}{}", String::from_utf8_lossy(&r.stdout), String::from_utf8_lossy(&r.stderr));
    assert!(combined.contains(expected), "{combined}");
    r
}
struct Script(PathBuf);
impl Script {
    fn new(ext: &str, body: &str) -> Self {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Temp/T12/cli-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(format!("{}_{}.{}", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed), ext));
        std::fs::write(&p, body).unwrap(); Self(p)
    }
    fn path(&self) -> &str { self.0.to_str().unwrap() }
}
impl Drop for Script { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
#[test]
fn eval_both_spellings_and_empty_program() {
    ok(&["-e", "print(1+2)"], b"", "3\n");
    ok(&["run", "-e", "print(1+2)"], b"", "3\n");
    ok(&["-e", ""], b"", "");
    ok(&["run", "-e", "", "--json"], b"", "{\"ok\":true}\n");
    ok(&["-e", "42"], b"", "");
}
#[test]
fn argv_preserves_all_literal_values_after_separator() {
    let code = "print(jsonStringify(argv()))";
    let expected = "[\"\",\"中文🙂\",\"two words\",\"--json\",\"--\",\"-e\",\"--help\",\"test\"]\n";
    for prefix in [vec!["-e", code], vec!["run", "-e", code]] {
        let mut a = prefix;
        a.extend(["--", "", "中文🙂", "two words", "--json", "--", "-e", "--help", "test"]);
        ok(&a, b"", expected);
    }
}
#[test]
fn file_and_bare_file_receive_args_but_keep_header_rules() {
    let f = Script::new("lfz", "#42\nprint(jsonStringify(argv()))\n");
    ok(&["run", f.path(), "--", "x", "--json"], b"", "[\"x\",\"--json\"]\n");
    ok(&[f.path(), "--", "x"], b"", "[\"x\"]\n");
    ok(&[f.path(), "--"], b"", "[]\n");
    let no_header = Script::new("lfz", "print(1)\n");
    bad(&["run", no_header.path(), "--", "x"], b"", "CosmosAnswerError");
    let txt = Script::new("txt", "print(jsonStringify(argv()))\n");
    ok(&["run", txt.path(), "--", "x"], b"", "[\"x\"]\n");
    bad(&[txt.path(), "--", "x"], b"", "只支持 .lfz");
}
#[test]
fn missing_code_and_undelimited_extra_args_are_errors() {
    for a in [vec!["-e"], vec!["run", "-e"], vec!["-e", "", "extra"], vec!["run", "a.lfz", "extra"]] {
        let r = run(&a, b""); assert_eq!(r.status.code(), Some(2)); assert!(r.stdout.is_empty());
        assert!(String::from_utf8_lossy(&r.stderr).contains("lfz:"));
    }
}
#[test]
fn code_operands_that_look_like_options_are_never_removed() {
    for code in ["--json", "--", "--help"] {
        for prefix in [vec!["-e", code], vec!["run", "-e", code]] {
            let r = bad(&prefix, b"", "<command>"); assert!(r.stdout.is_empty());
        }
    }
}
#[test]
fn json_flags_before_and_after_code_are_status_only() {
    for a in [vec!["--json", "-e", "print(3)"], vec!["run", "--json", "-e", "print(3)", "--json"]] {
        let r = run(&a, b""); assert_eq!(r.status.code(), Some(0));
        assert_eq!(r.stdout, b"{\"ok\":true}\n"); assert_eq!(r.stderr, b"3\n");
    }
    let r = bad(&["--json", "-e", "--json"], b"", "\"file\":\"<command>\"");
    assert!(r.stderr.is_empty()); assert_eq!(r.stdout.iter().filter(|&&c| c == b'\n').count(), 1);
}
#[test]
fn stdin_source_both_spellings_normalizes_bom_and_crlf() {
    for a in [vec!["-", "--", "x"], vec!["run", "-", "--", "x"]] {
        ok(&a, "\u{feff}print(jsonStringify(argv()))\r\n".as_bytes(), "[\"x\"]\n");
    }
    ok(&["-"], b"", ""); ok(&["--json", "run", "-"], b"", "{\"ok\":true}\n");
}
#[test]
fn stdin_invalid_utf8_is_io_error_without_fabricated_location() {
    let r = bad(&["--json", "-"], &[0xff], "无法读取标准输入");
    let s = String::from_utf8(r.stdout).unwrap();
    assert!(s.contains("\"error\":\"IOError\"")); assert!(s.contains("\"file\":\"<stdin>\""));
    assert!(s.contains("\"line\":null,\"col\":null,\"traceback\":[]")); assert!(r.stderr.is_empty());
}
#[test]
fn command_and_stdin_errors_have_correct_line_origin() {
    let code = "let x = 1\n1 / 0\n";
    for (a, input, path) in [(vec!["-e", code], b"".as_slice(), "<command>"), (vec!["-"], code.as_bytes(), "<stdin>")] {
        let r = bad(&a, input, "ZeroDivisionError"); let s = String::from_utf8(r.stderr).unwrap();
        assert!(s.contains(&format!("File \"{path}\", line 2")), "{s}");
    }
}
#[test]
fn eval_and_file_read_input_but_stdin_code_is_at_eof() {
    ok(&["-e", "print(input())"], b"hello\n", "hello\n");
    let f = Script::new("lfz", "#42\nprint(input())\n"); ok(&[f.path()], b"hello\n", "hello\n");
    bad(&["-"], b"input()\n", "IOError");
}
#[test]
fn no_implicit_stdin_or_last_value_printing() {
    bad(&[], b"print(42)\n", "缺少子命令"); ok(&["-e", "42", "--json"], b"", "{\"ok\":true}\n");
}
#[test]
fn json_argv_does_not_activate_status_mode() {
    ok(&["-e", "print(argv()[0])", "--", "--json"], b"", "--json\n");
    let r = run(&["--json", "-e", "print(argv()[0])", "--", "--json"], b"");
    assert_eq!(r.status.code(), Some(0)); assert_eq!(r.stdout, b"{\"ok\":true}\n"); assert_eq!(r.stderr, b"--json\n");
}