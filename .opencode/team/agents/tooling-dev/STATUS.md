# tooling-dev — 工作状态
> 最后更新: 2026-09-23 by tooling-dev（P3.10 最小 CLI + 端到端打通）

## 当前状态
P3.10 **已完成**：`src/cli.rs` + `src/main.rs` 实现 `lfz run <file>`（含错误格式化 + 退出码 0/1/2 + `--help`/`--version`），`examples/hello.lfz` 产出，端到端链路（加载→词法→语法→求值→错误输出）真实打通。
待 team-lead 交付 P3.11（verifier 独立验收）与后续 P4（runner / REPL / `--json` / 打包）。

## 进行中
- （无；本批任务已交付，等待调度）

## 阻塞 / 需要支持
- （无）
- 提示：`src/parser.rs` 由 core-dev 并行补齐（`fn`/`struct`/控制流/`--json` 语法面）。本 CLI 只依赖稳定库入口，未受其影响；P3.10 的 3 条验收命令在**当前** parser 状态下全部通过。

## 下一步计划（P4 待 team-lead 排期）
1. **runner 契约先行**（铁律）：先写 `docs/tooling/runner-contract.md`（发现规则 / 断言接口 / 失败格式 / 退出码 / 汇总格式 + 2 个契约示例用例），再通知 team-lead 转 test-engineer。
2. `lfz test` 一键 runner（承载评分项 2）+ `--json`（§8.3 示例 4 字段）+ REPL（可选）+ `scripts/package.ps1` + `scripts/lfz.ps1`/`lfz.bat` 启动器。
3. 运行章节与 docs-writer 协作（Windows 优先，UTF-8，跨平台注意）。

## 关键经验（写给未来的自己）
- **实现落点**：CLI 作为 bin crate 的子模块（`main.rs` 内 `mod cli;`），**不改 `lib.rs`**（任务禁区边界）；`cli.rs` 用 `lfz::...` 路径引用库模块。
- **错误渲染必须用 `eval_module_traced`**（`evaluator.rs`），拿 `TracedRun.frames` + `frame_name()` 组装 §8.3 traceback；`eval_module`/`run` 只返回结果、无帧栈（DECISIONS P3.8 影响段明确要求）。
- **加载期 vs 运行期**：`CosmosAnswerError`/`SyntaxError` **无** Traceback 头；其余类 **有**。**加载期 IOError/NotUtf8 也无** Traceback（按「阶段」而非「类」判定；见本轮 ADR）。
- **CosmosAnswerError 只有 `File "<path>", line 1` 一行**（无缩进、无源码行、无插入符）——严格照 `semantics.md` §8.3 示例 3。
- **`line_base` 偏移**：`.lfz` 的 `Loaded.text` 首行 = 文件第 2 行 → 源码下标 = `line - line_base - 1`。
- **错误流 = stderr，程序输出 = stdout**（spec 未规定错误流，采用 Python 约定；见本轮 ADR）。
- **无第三方依赖的 e2e 测试**：`tests/cli.rs` 用 `env!("CARGO_BIN_EXE_lfz")` + `std::process::Command`；临时 `.lfz` 写系统 temp 目录并在 `Drop` 删除。
- **PowerShell 5.1 抓 stderr 有坑**：`& exe ... 2>file` 会把原生 stderr 变成 PS ErrorRecord；要用 `Start-Process -RedirectStandardError`，并用 `[System.Text.UTF8Encoding]::new($false)` 读写以保证中文逐字节可比对。
- **已知 spec 排版瑕疵（未改）**：`semantics.md` §8.3 示例 2 的内层帧（`n / 0`）比通用帧格式少 4 空格（源码行/插入符均少 4），与示例 1 及同例外层帧不一致；本实现按 §8.2 通用规则（源码行前缀 4 空格、插入符 = 4 + (col-1)）统一处理。已在汇报中提示 language-architect。
