//! LFZ 解释器最小 CLI 入口（P3.10）。
//!
//! 命令行为见 [`cli`]：`lfz run <file>` / `lfz --help` / `lfz --version`。
//! 错误格式化与退出码映射（`0/1/2`）同样在 [`cli`] 中。
//!
//! **本批不做**：REPL、`--json`、`lfz test` runner（留 P4）。

mod cli;

use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut out = std::io::stdout();
    let mut err = std::io::stderr();
    let code = cli::execute(&args, &mut out, &mut err);
    let _ = out.flush();
    let _ = err.flush();
    std::process::exit(code);
}
