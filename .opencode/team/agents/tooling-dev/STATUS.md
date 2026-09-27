# tooling-dev — 工作状态
> 最后更新: 2026-09-27 21:35 by tooling-dev（T11-CLI 修复：`;;`×`--json` 同通道 + `--help` 文案）

## 当前状态
**T11 CLI 缺陷修复已完成（用户批准"全修"）**：修 `bug-B-20260927-01`（`;;` dump 在 `--json` 下未与
`print` 同通道 → stdout 非单个 JSON）+ `obs-A-02`（`--help` 一刀切「须以 .lfz 结尾」与 §2.2.0 矛盾）。
`OBS-01`/`OBS-02` 经查无 spec 依据，**只汇报不动手**。未 commit / tag / push。

### 本批改动（唯一范围；已逐一列函数供 verifier 定向复验）
- `src/builtins.rs`：**新增** `pub fn write_dump(text: &str, span: Span) -> R<()>`（`;;` 写通道，复用
  `print` 的进程级开关 `STDOUT_TO_STDERR`）。**除该新函数外，本文件其余 diff 属 runtime-dev（`b_range`/`b_repeat` 容量硬化），非本批。**
- `src/evaluator.rs`：`Interp::exec_stmt_inner` 的 `StmtKind::Dump` 分支——原直写 `std::io::stdout()` 改为
  `builtins::write_dump(&text, stmt.span)?`；移除随之无用的导入 `use std::io::Write` 与 `io as io_error`。
  **除该分支/导入外，本文件其余 diff 属 runtime-dev（格式符上限 / 索引写 span），非本批。**
- `src/cli.rs`：`HELP` 常量改写（对齐 §2.2.0 + 明示裸调用等价）；模块流约定文档补 `;;`；新增 1 单测
  `cli::tests::help_text_matches_extension_exemption`。
- `tests/cli.rs`：新增 4 条 e2e（`json_run_dump_success_redirects_to_stderr` /
  `json_run_dump_and_print_share_channel_in_order` / `json_run_dump_before_error_redirects_to_stderr` /
  `dump_without_json_still_writes_stdout`）。

### 证据（逐字节）
- **修复前** `target\debug\lfz.exe --json run dumpjson.lfz`：exit 0；stdout **20 B** = `7A 20 EF BC 9A 20 35 0A 7B 22 6F 6B 22 3A 74 72 75 65 7D 0A`（`z ： 5\n{"ok":true}\n`，2 行）；stderr 0 B。
- **修复后**：exit 0；stdout **12 B** = `7B 22 6F 6B 22 3A 74 72 75 65 7D 0A`（`{"ok":true}\n`，单行合法 JSON）；stderr **8 B** = `7A 20 EF BC 9A 20 35 0A`（`z ： 5\n`）。
- `--json` 下 print/;; 同通道且有序：stderr = `A\nz ： 5\nB\n`（12 B），stdout 仍 12 B。非 `--json`：`;;` 写 stdout（8 B），stderr 空。
- `cargo build` **0 warning**；`cargo test` **456 passed / 0 failed / 0 ignored**（lib 368 + main 48 + tests/cli 28 + tests/test_runner 12）；`lfz test` **87/87 exit 0**。
- 改动文件字节数：`src/cli.rs` 40548 B、`src/builtins.rs` 96304 B、`src/evaluator.rs` 141770 B、`tests/cli.rs` 23476 B（后三者含并发角色改动）。

## 进行中
- （无；等待调度/verifier 复验）

## 阻塞 / 需要支持
- （无）
- **环境事实（须转达）**：默认 `target\` 被另一路**只读**探针脚本持续占用（`%TEMP%\opencode\lfz-parsedepth`，
  反复 `Start-Process target\debug\lfz.exe run q.lfz`），导致 `cargo build` 间歇 `os error 5`（无法删除
  `target\debug\lfz.exe`）。本批曾用 `CARGO_TARGET_DIR=%TEMP%\opencode\lfz-verify-target` 隔离复现；
  待并发探针结束后，默认 `target\` 的 `cargo build`/`cargo test` 亦已复跑通过（0 warning / 456 passed）。
- **历史待仲裁项（沿用）**：`test_runner` 汇总 schema 为 spec 外扩展、`FAIL`/`ERROR` 并存取 `2` 等（ADR 2026-09-27 11:23）。

## 下一步计划（P4 剩余）
1. REPL（可选、非阻塞）。
2. `scripts/package.ps1`（8 项提交物打包，先向 team-lead 确认清单）+ `scripts/lfz.ps1`/`lfz.bat` 启动器。
3. 运行章节与 docs-writer 协作（`lfz <file>` / `lfz run` / `lfz test` / `--json` 用法 + 跨平台注意）。

## 关键经验（写给未来的自己）
- **`;;` 与 `print` 同通道 = 一个进程级开关**：`--json` 下 `print`/`input` 提示/`;;`(dump) **全部**经
  `builtins::set_stdout_to_stderr(true)` 转 stderr；任何**绕开该开关直写 `std::io::stdout()`** 的程序输出都会
  破坏「stdout 恒为单个 JSON」。新增输出通道时**必须**走 `builtins` 的写内核（`write_line` / `write_dump`）。
- **help 文案的"作用域"陷阱**：`lfz <file>`（裸调用）要求 `.lfz`，而 `lfz run <file>` 接受任意扩展名并
  按 §2.2.0 对非 `.lfz` 豁免 `#42`。给 `<file>` 写"须以 .lfz 结尾"必须**限定到裸调用形态**，否则与 run 矛盾。
- **Rust 单元测试落点**：`src/cli.rs`/`src/test_runner.rs` 的内联 `#[cfg(test)]` 编译进 **bin** 目标
  （`cargo test --bin lfz`），不在 `--lib`；本仓库**无** `tests/unit/` 目录，e2e 在 `tests/cli.rs`（cargo 只发现 `tests/*.rs`）。
- **并发写者检测**：`git status` + 文件 mtime + `Get-CimInstance Win32_Process` 可快速判定有人同时在跑
  `lfz.exe` 探针；`cargo build` 的 `os error 5`（拒绝访问）即"目标 exe 被占用"，用独立 `CARGO_TARGET_DIR` 绕开。
- **控制台不渲染 CJK（证据采集）**：含中文输出用 `Start-Process -RedirectStandardOutput <file>` + `[IO.File]::ReadAllText(utf8)` / `Format-Hex` 落原始字节。
