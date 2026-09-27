# tooling-dev — 工作状态
> 最后更新: 2026-09-27 11:16 by tooling-dev（P4.1 `lfz test` 一键测试 runner）

## 当前状态
**P4.1 已完成**：`lfz test [路径...]` 一键黑盒测试 runner 落地，逐条实现 `interface-contract.md` §11.2 **T-R1…T-R4**，端到端可跑（发现→执行→断言→判定→汇总→退出码）。
派生契约文档 `docs/tooling/runner-contract.md` 已就绪 → **请 team-lead 转告 test-engineer 按此编写 P5 黑盒测试集**。
证据：`cargo build` **0 warning**；`cargo test` **402 passed / 0 failed**（lib 361 + bin 27 + `tests/cli.rs` 7 + `tests/test_runner.rs` 7；原有 377 用例零破坏）；`cargo run -- test` 三档退出码实测 0 / 1 / 2。

## 进行中
- （无；本批任务已交付，等待调度）

## 阻塞 / 需要支持
- （无）
- **需 team-lead 转达 test-engineer**：runner 契约就绪（`docs/tooling/runner-contract.md`），生效时间 = 本轮（2026-09-27）。test-engineer 可按 §7「最小接入清单」编写用例并就地联调。
- **契约待确认（未自行发明，列此待仲裁）**：① 同一轮 `FAIL`+`ERROR` 并存时退出码取 `2`（ERROR 优先）；② 清单 `expect` 类名不符判 `FAIL`；③ 发现 0 用例判环境错误退出 `2`。详见派生文档 §5 与本轮 ADR。

## 下一步计划（P4 剩余）
1. `--json` 错误输出（semantics §8.4，示例 4 字段）。
2. REPL（可选、非阻塞）+ `scripts/package.ps1` + `scripts/lfz.ps1`/`lfz.bat` 启动器（Windows 优先）。
3. 运行章节与 docs-writer 协作（`lfz test` 用法 + 跨平台注意）。
4. 与 test-engineer 联调 `lfz test` 跑其最小套件（P5 放量前）。

## 关键经验（写给未来的自己）
- **runner 落点**：`src/test_runner.rs` + `src/json.rs` 是 **bin crate 私有模块**（`main.rs` 内 `mod`），**未动 `lib.rs`**。`src/cli.rs` 新增 `test` 子命令并把执行链抽成 `pub(crate) fn eval_case() -> CaseEval`（`Passed` / `Failed{err,loaded,traced}`），runner 复用之，**不复制** lexer/parser/evaluator 逻辑。
- **失败渲染直接复用** `cli::render_error`（已有 §8.2/§8.3 逻辑）；runner 只负责「判定 + 缩进 2 格 + 汇总」，不另写格式化。
- **无第三方 JSON**：std 无 JSON → 自写 `src/json.rs`（递归下降，含 `\uXXXX` / 代理对 / **容忍前导 BOM**）。踩坑：PowerShell 5.1 `Set-Content -Encoding utf8` 写 **BOM**，故 parser 必须 `strip_prefix('\u{FEFF}')`。
- **判定模型**（对齐 pytest）：无错=PASS；仅 `AssertionError`=FAIL（退出 1）；其余错误类=ERROR（退出 2，优先于 FAIL）；`check` 失败非致命=PASS（A4）。
- **发现规则**：默认根 `tests`，递归 `*.lfz`（`loader::is_lfz`），**跳过任何名为 `fixtures` 的目录**；`cases.json` 附加 `expect` / 新增夹具。路径统一规范化为 `/` 输出（跨平台一致）。
- **流约定**：测试报告 → stdout；runner 自身错误（参数/缺目录/清单非法/无用例）→ stderr。**已知限制**：内建 `print`/`check` 直接写进程 stdout/stderr，无法接管，正文若有 `print` 会与报告交错。
- **测试隔离**：单测与 e2e 全部用**系统临时目录**（`tests/` 下只留 `*.rs`），符合「不占 P5 领地」；e2e 用 `CARGO_BIN_EXE_lfz` + `Command::current_dir` 测默认发现。
