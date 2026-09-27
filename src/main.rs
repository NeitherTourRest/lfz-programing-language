//! LFZ 解释器 CLI 入口。
//!
//! 命令行为见 [`cli`]：`lfz run <file>` / `lfz test [路径...]` / `lfz --help` / `lfz --version`。
//! 错误格式化与退出码映射（`0/1/2`）在 [`cli`]；一键测试 runner 在 [`test_runner`]；
//! `cases.json` 清单解析在 [`json`]。
//!
//! **本批不做**：REPL、`--json`。

mod cli;
mod json;
mod test_runner;

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
