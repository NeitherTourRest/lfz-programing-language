use std::{fs, path::PathBuf, process::{Command, Output, Stdio}, sync::atomic::{AtomicUsize, Ordering}, time::{Duration, Instant}};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Work(PathBuf);
impl Work {
    fn new() -> Self {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Temp/T21-tooling/cli").join(format!("{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn file(&self, name: &str, raw: &[u8]) {
        fs::write(self.0.join(name), raw).unwrap();
    }
    fn run(&self, args: &[&str]) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_lfz")).args(args).current_dir(&self.0)
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while child.try_wait().unwrap().is_none() {
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("LFZ exceeded fixture watchdog: {args:?}");
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        child.wait_with_output().unwrap()
    }
}

#[test]
fn stdout_is_exact_and_input_is_untouched() {
    let work = Work::new();
    let raw = "\u{feff}#42\r\n\tlet x = \"中文\" \t\r\t \n\t// kept \t\n\tprint(x) \t";
    let expected = "\u{feff}#42\r\n let x = \"中文\"\r\t \n // kept \t\n print(x)";
    work.file("x.lfz", raw.as_bytes());
    let result = work.run(&["fmt", "x.lfz"]);
    assert_eq!(result.status.code(), Some(0), "{result:?}");
    assert_eq!(result.stdout, expected.as_bytes());
    assert!(result.stderr.is_empty());
    assert_eq!(fs::read(work.0.join("x.lfz")).unwrap(), raw.as_bytes());
    work.file("out.lfz", &result.stdout);
    assert_eq!(work.run(&["fmt", "out.lfz"]).stdout, expected.as_bytes());
}

#[test]
fn invalid_source_missing_file_and_invalid_options_keep_stdout_empty() {
    let work = Work::new();
    for (name, raw) in [("utf.lfz", b"\xff".as_slice()), ("header.lfz", b"#42 \n1".as_slice()), ("lex.lfz", b"#42\n@".as_slice()), ("parse.lfz", b"#42\nlet =".as_slice())] {
        work.file(name, raw);
        let result = work.run(&["fmt", name]);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }
    for args in [vec!["fmt", "missing.lfz"], vec!["--json", "fmt", "missing.lfz"], vec!["fmt", "missing.lfz", "--json"], vec!["fmt", "missing.lfz", "--allow-process"], vec!["fmt", "missing.lfz", "--max-steps", "1"], vec!["fmt", "missing.lfz", "--", "arg"]] {
        let result = work.run(&args);
        assert_eq!(result.status.code(), Some(2), "{args:?}");
        assert!(result.stdout.is_empty(), "{args:?}");
        assert!(!result.stderr.is_empty());
    }
}

#[test]
fn old_bare_fmt_and_explicit_execution_keep_their_meaning() {
    let work = Work::new();
    work.file("fmt", b"print(17)\n");
    let bare = work.run(&["fmt"]);
    assert_eq!(bare.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bare.stderr).contains("只支持 .lfz 脚本文件：'fmt'"));
    for args in [["fmt", "--json"], ["--json", "fmt"]] {
        let result = work.run(&args);
        assert_eq!(result.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&result.stdout).contains("\"error\":\"IOError\""));
        assert!(result.stderr.is_empty());
    }
    let old_argv = work.run(&["fmt", "--", "argument"]);
    assert_eq!(old_argv.stderr, bare.stderr);
    assert_eq!(work.run(&["run", "fmt"]).stdout, b"17\n");
    work.file("fmt.lfz", b"#42\nprint(18)\n");
    assert_eq!(work.run(&["fmt.lfz"]).stdout, b"18\n");
}

#[test]
fn formatting_preserves_observable_error_columns_and_effects() {
    let work = Work::new();
    let raw = b"#42\n\tlet result=attempt(()=>missing()) \t\n\tprint(result.error.line)  \n\tprint(result.error.col) \t\n\tif true { \t\n\t\tappendFile(\"effect.txt\",\"x\") /* keep */ \t\n\t} \t";
    work.file("same.lfz", raw);
    let formatted = work.run(&["fmt", "same.lfz"]);
    assert_eq!(formatted.status.code(), Some(0), "{formatted:?}");
    assert!(!work.0.join("effect.txt").exists());
    let before = work.run(&["same.lfz"]);
    assert_eq!(before.status.code(), Some(0), "{before:?}");
    assert_eq!(fs::read(work.0.join("effect.txt")).unwrap(), b"x");
    work.file("same.lfz", &formatted.stdout);
    work.file("effect.txt", b"");
    let after = work.run(&["same.lfz"]);
    assert_eq!(after.status.code(), Some(0), "{after:?}");
    assert_eq!(after.stdout, before.stdout);
    assert_eq!(before.stdout, b"2\n25\n");
    assert_eq!(fs::read(work.0.join("effect.txt")).unwrap(), b"x");
}

#[test]
fn module_source_formats_without_loading_dependencies() {
    let work = Work::new();
    work.file("module.lfz", b"#42\r\n\timport missing from \"./not-present.lfz\" \t\n\tprint(\"unexecuted\") \t");
    let result = work.run(&["fmt", "module.lfz"]);
    assert_eq!(result.status.code(), Some(0), "{result:?}");
    assert_eq!(result.stdout, b"#42\r\n import missing from \"./not-present.lfz\"\n print(\"unexecuted\")");
    assert!(result.stderr.is_empty());
}
