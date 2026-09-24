# core-dev — 工作状态
> 最后更新: 2026-09-24 08:49 by core-dev

## 当前状态
**P3.11 裁定 1 落地（`src/error.rs`：新增 `TypeMsg::ImmutableRebind { name }`）✅ 完成** —— `cargo build` **0 warning**、`cargo test` lib harness **359 passed / 0 failed / 1 ignored**（`1 ignored` 为 runtime-dev 占位，未动）。待 team-lead 核验 + release-manager 提交；**runtime-dev 的 bug-06 阻塞就此解除**。
- 依据：`DECISIONS.md` [2026-09-24 00:30]「P3.11 验收 3 处规范裁定」**裁定 1**（表 #1 指派 core-dev）+ `docs/spec/semantics.md` §4.5.2 / §8.1（逐字照抄，未自行发明）。
- 交付：变体 `ImmutableRebind { name: String }` + `message()` 分支；顶部注释「**6 条**」→「**7 条**」。**未改** `class_name()`、**未新增错误类**、**无 `E-xxx`**、**未改其它变体**。

## 进行中
- （无）

## 最近完成
- **P3.11 裁定 1：`TypeMsg::ImmutableRebind { name }`**（2026-09-24 08:49，轻量启动，直接落盘再测）。
  - **新增变体**（`src/error.rs:125` 附近）：`ImmutableRebind { name: String }`，文档注释标注「重绑定 `let` 变量（v1 补钉，§4.5.2 / §8.1）」。
  - **`message()` 分支**（逐字符照抄规范）：`format!("不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容")`。
  - **单测（折入既有测试，保持总数 359）**：
    - `type_msg_covers_all_seven_rows`（由 `..._six_rows` 更名）：新增 `ImmutableRebind` 行的 `assert_eq!` + 3 条 `assert_chars_eq`（`a` / `counter` / `_tmp`，**逐字符**断言规范模板）。
    - `cold_constructors_build_correct_class`：新增断言 `type_error(ImmutableRebind, ..).class_name() == "TypeError"` + `LzError::Type { .. }.to_string() == "TypeError: 不能重新赋值 let 变量 'counter'；..."`。
  - 证据：
    - `git diff --stat -- src/error.rs` → `1 file changed, 60 insertions(+), 2 deletions(-)`（仅 `src/error.rs`）。
    - `cargo build` → `BUILD_EXIT=0`、`WARN_COUNT=0`（`$out | Select-String 'warning' | Measure-Object` → 0）。
    - `cargo test` → lib harness `test result: ok. 359 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out`，`TEST_EXIT=0`（聚合 3 harness：359 + 9 + 7 = 375）。
  - 边界纪律：**仅改** `src/error.rs`；**未改** `span.rs` / `loader.rs` / `lexer.rs` / `ast.rs` / `parser.rs` / `evaluator.rs` / `cli.rs` / `docs/spec/` / `Cargo.toml`；**未** commit / tag / push。
- 更早：**P3.4a `src/ast.rs`**（09-23）✅、**P3.3b lexer 第二批**（09-24）✅、**P3.9a `ValueMsg` 新增 2 变体**（09-24）✅、**P3.3a lexer 第一批** / **P3.1 基座** / **P3.2 加载器** ✅。

## 阻塞 / 需要支持
- **无阻塞**。
- 契约缺口（`let` 重绑定错误类）已由 architect 裁定并**在 core-dev 侧落地**；剩余接线（evaluator）属 runtime-dev，非本模块问题。

## 下一步计划
- 等 team-lead 派发 **P3.4b / P3.5 parser**：按 `syntax.md` §7 EBNF 递归下降，消费 lexer 记号流与本 AST；实现 NL 模式栈 `SIG/IGN`、`NO_BRACE_LITERAL`、管道脱糖、`_` 绑定（M4）、`i64::MIN` 特判（§10.8）、`Dump` 填 `scope`。
- 观察 runtime-dev 接线 `exec_assign` 的结果；若 AST 侧有歧义 → 升级 language-architect。

## 关键经验（写给未来的自己）
- **本机 cargo/rustc 不在默认 PATH**：须先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **任务书给的确切测试计数要照做**：本次任务书写「期望 `359 passed; 1 ignored`」——若新增独立 `#[test]` 函数会把 lib harness 计数顶到 361。改为把新断言**折入既有测试函数**（`type_msg_covers_all_seven_rows` / `cold_constructors_build_correct_class`），计数即保持 359，**且断言覆盖要求全部满足**。
- **`cargo test` 有 3 个 harness**：lib(359) + 集成(9) + 集成(7) = 375；team 提到的「375 passed」是**聚合值**，「359 passed」单指 lib harness —— 汇报时要说清是哪一层，勿混淆。
- **子消息变体新增 = 复用类、零 `E-xxx`**：只加 `TypeMsg` 变体 + `message()` 分支 + 文档注释计数，**绝不动** `class_name()` 映射（守红线）。
- **PowerShell 抓 `cargo` 输出**：`cargo ... 2>&1 | Tee-Object -Variable out`；cargo 写 stderr 的正常进度会被记成 `NativeCommandError`，**以 `$LASTEXITCODE` 为准**；warning 计数 `$out | Select-String 'warning' | Measure-Object`。
- **git diff 在 PowerShell 直出会乱码**（UTF-8 → GBK 显示），但**源文件本身完好**——以 `cargo test` 的逐字符中文断言通过为准，勿被控制台乱码误导。
- **共享工作区会撞他人半成品**：`cargo test` 可能因他人（runtime-dev）**正在编辑**的模块瞬时编译失败；先看 mtime、隔一会儿重跑，勿越界修他人文件。
