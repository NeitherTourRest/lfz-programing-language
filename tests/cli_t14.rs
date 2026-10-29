//! T14 non-executing check command process regressions. Scratch files stay in Temp/.
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
const BIN: &str = env!("CARGO_BIN_EXE_lfz");
const CHECK_OK: &[u8] = b"{\"ok\":true,\"mode\":\"check\",\"executed\":false}\n";
static SERIAL: AtomicU64 = AtomicU64::new(0);
fn run(args: &[&str], input: &[u8]) -> Output {
    let temp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Temp/T14/cli-tests");
    std::fs::create_dir_all(&temp).unwrap();
    let mut child = Command::new(BIN).args(args).env("TEMP", &temp).env("TMP", &temp)
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    if let Some(mut pipe) = child.stdin.take() { let _ = pipe.write_all(input); }
    let deadline = Instant::now() + Duration::from_secs(5);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline { let _ = child.kill(); let _ = child.wait(); panic!("check timed out: {args:?}"); }
        std::thread::sleep(Duration::from_millis(5));
    }
    child.wait_with_output().unwrap()
}
fn ok(args: &[&str], input: &[u8], json: bool) {
    let result = run(args, input);
    assert_eq!(result.status.code(), Some(0), "args={args:?} stdout={:?} stderr={:?}", result.stdout, result.stderr);
    assert_eq!(result.stdout, if json { CHECK_OK } else { b"" });
    assert!(result.stderr.is_empty(), "stderr={:?}", result.stderr);
}
fn front_error(args: &[&str], input: &[u8], class: &str, json: bool) -> Output {
    let result = run(args, input);
    assert_eq!(result.status.code(), Some(2), "args={args:?} stdout={:?} stderr={:?}", result.stdout, result.stderr);
    if json {
        let text = String::from_utf8_lossy(&result.stdout);
        assert!(text.starts_with("{\"ok\":false,\"mode\":\"check\",\"executed\":false,\"error\":"), "{text}");
        assert!(text.contains(&format!("\"error\":\"{class}\"")), "{text}");
        assert_eq!(result.stdout.iter().filter(|&&b| b == b'\n').count(), 1);
        assert!(result.stderr.is_empty());
    } else {
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains(class));
    }
    result
}
struct Fixture(PathBuf);
impl Fixture {
    fn new(ext: &str, bytes: &[u8]) -> Self {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Temp/T14/cli-tests");
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join(format!("{}_{}.{}", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed), ext));
        std::fs::write(&path, bytes).unwrap(); Self(path)
    }
    fn path(&self) -> &str { self.0.to_str().unwrap() }
}
impl Drop for Fixture { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
fn quoted(path: &str) -> String { format!("\"{}\"", path.replace('\\', "\\\\").replace('"', "\\\"")) }
#[test]
fn check_three_sources_and_both_output_modes() {
    let source = "print(1 / 0)\n";
    let file = Fixture::new("lfz", format!("#42\n{source}").as_bytes());
    for json in [false, true] {
        let suffix = if json { vec!["--json"] } else { vec![] };
        let mut args = vec!["check", file.path()]; args.extend(suffix.clone()); ok(&args, b"", json);
        let mut args = vec!["check", "-e", source]; args.extend(suffix.clone()); ok(&args, b"", json);
        let mut args = vec!["check", "-"]; args.extend(suffix); ok(&args, source.as_bytes(), json);
    }
}
#[test]
fn check_code_options_and_empty_code_are_preserved() {
    ok(&["check", "-e", ""], b"", false);
    ok(&["--json", "check", "--json", "-e", "", "--json"], b"", true);
    // These expressions parse as unary minus operators and a name; they must not become options.
    ok(&["check", "-e", "--json"], b"", false);
    ok(&["check", "-e", "--help", "--json"], b"", true);
    front_error(&["check", "-e", "--", "--json"], b"", "SyntaxError", true);
    ok(&["check", "-e", "let text = \"中文🙂 --json --\"\nprint(text)", "--json"], b"", true);
}
#[test]
fn check_usage_rejects_separators_and_extra_operands_even_in_json_mode() {
    for args in [vec!["check"], vec!["check", "-e"], vec!["check", "a", "b"], vec!["check", "--help"],
        vec!["check", "--"], vec!["check", "a", "--"], vec!["check", "-", "--", "arg"],
        vec!["check", "-e", "", "--"], vec!["check", "-e", "", "extra"],
        vec!["check", "--unknown"], vec!["--json", "check", "-e", "", "--"],
        vec!["--json", "check", "-e"]] {
        let result = run(&args, b"");
        assert_eq!(result.status.code(), Some(2), "{args:?}"); assert!(result.stdout.is_empty(), "{args:?}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("lfz:"), "{args:?}");
    }
}
#[test]
fn check_file_loading_keeps_preamble_extension_and_line_offsets() {
    let uppercase = Fixture::new("LFZ", b"#42\nunknownName\n"); ok(&["check", uppercase.path()], b"", false);
    let plain = Fixture::new("txt", b"unknownName\n"); ok(&["check", plain.path()], b"", false);
    let missing_header = Fixture::new("lfz", b"unknownName\n");
    front_error(&["check", missing_header.path(), "--json"], b"", "CosmosAnswerError", true);
    let bad = Fixture::new("lfz", b"#42\nlet a =\n");
    let result = front_error(&["check", bad.path(), "--json"], b"", "SyntaxError", true);
    assert!(String::from_utf8_lossy(&result.stdout).contains("\"line\":2,"));
}
#[test]
fn check_bom_crlf_and_invalid_encoding_follow_loader() {
    let file = Fixture::new("lfz", "\u{feff}#42\r\nprint(1)\r\n".as_bytes()); ok(&["check", file.path()], b"", false);
    ok(&["check", "-", "--json"], "\u{feff}print(1)\r\n".as_bytes(), true);
    let invalid = Fixture::new("lfz", &[0xff]);
    front_error(&["check", invalid.path(), "--json"], b"", "SyntaxError", true);
    let result = front_error(&["check", "-", "--json"], &[0xff], "IOError", true);
    let text = String::from_utf8_lossy(&result.stdout);
    assert!(text.contains("无法读取标准输入")); assert!(text.contains("\"line\":null,\"col\":null,\"traceback\":[]"));
}
#[test]
fn check_reports_source_paths_and_frontend_error_without_runtime_trace() {
    for (args, input, path) in [(vec!["check", "-e", "let x =", "--json"], b"".as_slice(), "<command>"),
        (vec!["check", "-", "--json"], b"let x =".as_slice(), "<stdin>")] {
        let result = front_error(&args, input, "SyntaxError", true);
        let text = String::from_utf8_lossy(&result.stdout);
        assert!(text.contains(&format!("\"file\":\"{path}\""))); assert!(text.contains("\"func\":\"<module>\""));
    }
    let missing = Fixture::new("lfz", b"#42\n"); std::fs::remove_file(&missing.0).unwrap();
    let result = front_error(&["check", missing.path(), "--json"], b"", "IOError", true);
    assert!(String::from_utf8_lossy(&result.stdout).contains("\"line\":null,\"col\":null,\"traceback\":[]"));
}
#[test]
fn check_reuses_parser_context_and_checks_unreachable_bodies() {
    for source in ["return 1", "break", "continue", "fn f() { let x = }", "if false { let x = }"] {
        front_error(&["check", "-e", source, "--json"], b"", "SyntaxError", true);
    }
}
#[test]
fn check_does_not_claim_runtime_name_type_or_arity_validation() {
    for source in ["unknownName", "1 + true", "assert(false)", "fn f(a) => a\nf()", "let x=1\nx=2", "readFile(\"missing.lfz\")"] {
        ok(&["check", "-e", source, "--json"], b"", true);
    }
}
#[test]
fn check_never_runs_program_output_input_dump_or_loops() {
    for source in ["print(1)\neprint(2)\n;;\ninput(\"PROMPT\")", "while true {}", "fn f() => f()\nf()"] {
        ok(&["check", "-e", source], b"", false);
        ok(&["check", "-e", source, "--json"], b"", true);
    }
}
#[test]
fn check_never_creates_or_modifies_files() {
    let existing = Fixture::new("txt", b"sentinel\0unchanged");
    let absent = Fixture::new("txt", b""); std::fs::remove_file(&absent.0).unwrap();
    let source = format!("writeFile({}, \"overwrite\")\nappendFile({}, \"extra\")\nwriteFile({}, \"new\")\n", quoted(existing.path()), quoted(existing.path()), quoted(absent.path()));
    ok(&["check", "-e", &source, "--json"], b"", true);
    assert_eq!(std::fs::read(&existing.0).unwrap(), b"sentinel\0unchanged"); assert!(!absent.0.exists());
    let source = format!("{source}let broken =");
    front_error(&["check", "-e", &source, "--json"], b"", "SyntaxError", true);
    assert_eq!(std::fs::read(&existing.0).unwrap(), b"sentinel\0unchanged"); assert!(!absent.0.exists());
}
#[test]
fn check_deep_parser_and_ast_boundaries_stay_controlled() {
    let legal = format!("{}0{}", "(".repeat(1000), ")".repeat(1000));
    ok(&["check", "-e", &legal, "--json"], b"", true);
    let illegal = format!("{}0{}", "(".repeat(1001), ")".repeat(1001));
    front_error(&["check", "-e", &illegal, "--json"], b"", "SyntaxError", true);
    let legal = "1+".repeat(9998) + "1";
    ok(&["check", "-", "--json"], legal.as_bytes(), true);
    let illegal = "1+".repeat(9999) + "1";
    front_error(&["check", "-", "--json"], illegal.as_bytes(), "SyntaxError", true);
}
#[test]
fn check_help_and_existing_run_protocol_remain_distinct() {
    let help = run(&["--help"], b""); assert_eq!(help.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&help.stdout).contains("lfz check"));
    let result = run(&["-e", "check(true)", "--json"], b"");
    assert_eq!(result.status.code(), Some(0)); assert_eq!(result.stdout, b"{\"ok\":true}\n"); assert!(result.stderr.is_empty());
    let result = run(&["-e", "print(argv()[0])", "--", "--json"], b"");
    assert_eq!(result.status.code(), Some(0)); assert_eq!(result.stdout, b"--json\n"); assert!(result.stderr.is_empty());
}
