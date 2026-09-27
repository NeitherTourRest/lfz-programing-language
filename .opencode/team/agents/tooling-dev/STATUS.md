# tooling-dev — 工作状态
> 最后更新: 2026-09-27 17:05 by tooling-dev（P4.2-fix：`--json` 下 `print` 重定向 stderr）

## 当前状态
**P4.2-fix 已完成**：`--json` 模式下 stdout 恒为**唯一一个 JSON**，被运行程序 / 用例正文的
`print`（及 `input` 提示）输出**重定向到 stderr**。原硬约束「`--json` 下 stdout 只能是 JSON」
现真正满足（此前 `print` 与 JSON 混排，见旧 §9.3「已知限制」）。

改动范围（**唯一一次跨模块例外，经 team-lead 书面授权**）：
- `src/builtins.rs`（**仅**输出目标切换，行 **1134–1223**，IO 段内）：新增进程级
  `static STDOUT_TO_STDERR: AtomicBool`（默认 `false`）+ `pub fn set_stdout_to_stderr(on: bool)`
  + 私有 `stdout_to_stderr()` + 写行内核 `write_line<T: Write>`；`b_print` / `b_input` 按开关择流；
  **`b_eprint`（1168–1174）逐字节未改**；未触碰任何其它内置的行为 / 签名 / 返回类型 / 错误消息。
- `src/cli.rs::run_file`、`src/test_runner.rs::run`：运行前 `lfz::builtins::set_stdout_to_stderr(json);`。
- `tests/cli.rs` +2、`tests/test_runner.rs` +1（补此前「程序不 print」的测试盲区）。
- `docs/tooling/runner-contract.md`：§6/§9.3「已知限制」→「**已解决**」；状态 **v1.2**。

证据：`cargo build --all-targets` **0 warning**；`cargo test` **431 passed / 0 failed**
（lib 361 + bin 42 + `tests/cli.rs` 16 + `tests/test_runner.rs` 12；**原 428 零破坏**，新增 3）。
实测：`run --json examples/hello.lfz` → stdout 恰 `{"ok":true}`（`ConvertFrom-Json` 解析通过）、
`Hello, LFZ!` 在 stderr、exit 0；`run --json <print+1/0>` → stdout 单行错误 JSON、marker 在 stderr、
exit 2；`test --json <含 print 用例目录>` → stdout 恰唯一汇总 JSON、marker 在 stderr、exit 0。

## 进行中
- （无；本批任务已交付，等待调度）

## 阻塞 / 需要支持
- （无）
- **需 team-lead 转达 test-engineer**：runner 契约已升 **v1.2**（2026-09-27）——**§9.3 由「已知限制」改为「已解决」**：`--json` 下用例正文可安全 `print`（内容进 stderr），旧「print 会污染 stdout JSON」假设作废；非 `--json` 行为不变（print 仍写 stdout）。
- 历史待仲裁项（沿用）：`test_runner` 汇总 schema 为 spec 外扩展、`FAIL`/`ERROR` 并存取 `2` 等，见 ADR 2026-09-27 11:23。

## 下一步计划（P4 剩余）
1. REPL（可选、非阻塞）。
2. `scripts/package.ps1`（打包，先向 team-lead 确认 8 项提交物清单）+ `scripts/lfz.ps1`/`lfz.bat` 启动器（Windows 优先）。
3. 运行章节与 docs-writer 协作（`lfz run` / `lfz test` / `--json` 用法 + stdout/stderr 分流 + 跨平台注意）。
4. 与 test-engineer 联调 `lfz test` 跑其最小套件（P5 放量前）。

## 关键经验（写给未来的自己）
- **`--json` 全局开关**：`cli::execute` 先 `filter` 掉所有 `--json` 再 `parse_args`，故 `lfz run --json f` 与 `lfz run f --json` 等价，`run` 仍要求恰一个 `<file>`。
- **流纪律（P4.2-fix 后）**：`--json` 下 **stdout 恒为唯一一行 JSON**。`print` / `input` 提示经 `builtins::set_stdout_to_stderr(true)` 转 **stderr**；`eprint` / `check` 恒 stderr。开关**进程级**（`AtomicBool`，默认 stdout），由 `run_file` / `test_runner::run` 以 `json` 形参置位 → `run` 与 `test`、成功与失败两路径都覆盖。
- **单进程测试的并发注意**：`set_stdout_to_stderr` 是进程级全局；**含 `print` 的 `--json` 断言一律放集成测试**（`tests/*.rs` 以 `Command` 起独立进程，分进程隔离）。bin 内联单测勿依赖「程序不 print」以外的假设。
- **字段同源**：`cli::push_error_fields` 是唯一错误字段构造器，`run --json`（`error_json`）与 `test --json`（`test_runner::case_json`）都复用 → 两处 schema 不会漂移。
- **JSON 编码（`json.rs::encode`）**：紧凑单行（`:`/`,` 无空格），**非 ASCII 原样**不转 `\uXXXX`，仅转义 `"`/`\`/`U+0000..1F`。
- **测试隔离**：单测与 e2e 全部用**系统临时目录**（`tests/` 下只留 `*.rs`），符合「不占 P5 领地」。
- **PowerShell 5.1 坑（证据采集）**：`>` 重定向写 UTF-16LE；采集原始 UTF-8 字节用 `Start-Process -RedirectStandardOutput <file>`（原始字节直写）。**控制台不渲染 CJK**（显示 `������`），但文件字节正确；用 Read 工具或 `[IO.File]::ReadAllBytes` + 十六进制确认（本次实测含 UTF-8 `E9 99 A4…`= 除以零）。不要用函数形参名 `$args`（与自动变量冲突 → `ArgumentList` 为 Null）。
