from pathlib import Path
p=Path('src/cli.rs');s=p.read_text(encoding='utf-8-sig');needle='    let (rest, script_args, json, limits) = match scan_options(args)';pos=s.index(needle)
s=s[:pos]+'''    if let [command, path] = args {
        if command == "fmt" && !path.is_empty() && !path.starts_with('-') {
            return evaluator::on_eval_stack(|| format_file(path, out, err));
        }
    }
'''+s[pos:]
s=s.replace('  lfz repl [-- ARG...]', '  lfz fmt FILE        保真格式化一个文件，结果写 stdout\n  lfz repl [-- ARG...]')
s+='''
fn format_file(path: &str, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    use lfz::formatter::{format_source, FormatError};

    let raw = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => {
            let error = LzError::Io { msg: format!("无法读取：{path}"), span: None };
            let _ = write!(err, "{}", render_error(path, None, &error, None));
            return EXIT_ERROR;
        }
    };
    let formatted = match format_source(path, &raw) {
        Ok(bytes) => bytes,
        Err(FormatError::InvalidSource { error, loaded }) => {
            let _ = write!(err, "{}", render_error(path, loaded.as_ref(), &error, None));
            return EXIT_ERROR;
        }
        Err(FormatError::InvariantViolation) => {
            let _ = writeln!(err, "lfz: 格式化结果未通过保真校验");
            return EXIT_ERROR;
        }
    };
    if let Err(error) = out.write_all(&formatted).and_then(|()| out.flush()) {
        let _ = writeln!(err, "lfz: 无法写入格式化结果：{error}");
        return EXIT_ERROR;
    }
    EXIT_OK
}
''';p.write_text(s,encoding='utf-8')
