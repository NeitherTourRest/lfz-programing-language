# core-dev — 工作状态
> 最后更新: 2026-09-23 by core-dev

## 当前状态
P3.0（Cargo 工程骨架）✅ 完成，待 team-lead 核验 + release-manager 提交 `chore(p3): cargo skeleton`。

## 进行中
- （无）

## 最近完成
- **P3.0 Cargo 工程骨架**（2026-09-23）
  - `Cargo.toml`：edition 2021；`[lib]` name `lfz` + `[[bin]]` name `lfz`（path `src/main.rs`）；`[dependencies]` **空**（仅 std）。
  - `src/lib.rs`：`pub mod` 声明全部 10 个模块：`span`/`error`/`loader`/`lexer`/`ast`/`parser`/`value`/`env`/`evaluator`/`builtins`。
  - `src/main.rs`：最小占位 `fn main() {}`（真实 CLI 归 tooling-dev，P3.10）。
  - 10 个模块文件：仅 `//!` 职责注释占位，无任何类型/函数实现。
  - 证据：`cargo build` → `Finished dev profile ... in 2.87s`（exit 0，无 warning）；`cargo test` → 3 个 harness 均 `test result: ok. 0 passed; 0 failed`（exit 0）；`cargo tree` → 仅 `lfz v0.1.0`（**零第三方依赖**）；`Cargo.lock` 已生成（147B，仅含 `lfz`）。
  - 未触碰 `.gitignore`（现存 `/target/` 已在，未追加）、`README.md`、`LICENSE`（mtime 均早于本次工作）。采用**手写**而非 `cargo init` 以规避覆盖风险。

## 阻塞 / 需要支持
- （无）

## 下一步计划
- 等待 team-lead 派发 **P3.1**：`span.rs` + `error.rs`（12 变体 + `class_name`/`message`/`span` + `R<T>`），core-dev 一次性写全（runtime-dev 只读消费）。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：须先 `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`（rustc/cargo 1.98.1）。
- 项目根路径含**空格**，PowerShell 命令中路径需加引号；`Get-Content -LiteralPath`。
- PowerShell 控制台为 GBK 代码页，显示 UTF-8 中文源码会**乱码**——这是**显示层**问题，文件本身是 UTF-8，勿误判为文件损坏。
- `cargo` 把进度写 stderr，PowerShell 会渲染成 `NativeCommandError` 红字——看 `$LASTEXITCODE` 与 `Finished` 行判断成败。
- **不要用 `cargo init`**：会覆盖/追加 `.gitignore`、`README.md`、`LICENSE`。手写 `Cargo.toml` + 目录更安全。
- 目录三方隔离（PLAN-P3 §1）：`src/{lib,span,error,loader,lexer,ast,parser}.rs` 归 core-dev；`{value,env,evaluator,builtins}` 归 runtime-dev；`{cli,main}.rs` 归 tooling-dev。
