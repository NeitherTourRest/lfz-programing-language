# test-engineer — 工作状态
> 最后更新: 2026-09-27 by test-engineer

## 当前状态
**P5.4 ✅ 完成**（黑盒测试集**定稿批**：12 类错误矩阵核对 + 查漏补缺 + 覆盖矩阵定稿 + `tests/REPORT.md`）。
`cargo run --quiet -- test` → **82 PASS / 0 FAIL / 0 ERROR，exit 0（全绿）**。
`cargo clean -p lfz; cargo build` → **0 warning / 0 error**；`cargo test` → **431 passed / 0 failed / 0 ignored**。

## 进行中
- （无）

## 已完成
- **P5.4（2026-09-27）**：黑盒测试集定稿批
  - **12 类错误核对**（`interface-contract.md` §8.1）：**11/12** 类有黑盒 `expect.error` 触发用例；唯 **`IOError` 黑盒跳过**（runner 无 stdin 约定，`input()` 可能 EOF/**阻塞**、不可复现），按任务授权注明**由 Rust 单测覆盖**（`src/builtins.rs::tests::input_reads_line_crlf_and_eof` EOF→`IOError`；`src/error.rs::tests::class_name_all_twelve_match_spec_exactly` 12 类名逐一）。基类 `LfzError` 不直接抛出 → 不适用。
  - **查漏补缺 §10.7 内置**：54 个 → **53 黑盒覆盖 + 1 跳过（`input`）**；`print`/`eprint`/`check` 输出文本不可捕获 → 仅断「返回值/不报错」，已注明替代覆盖者。
  - `tests/coverage-matrix.md` **重构定稿**：§0 总览 / §1 特性覆盖表 + 特性域总览 / §2 内置 54 逐个 / §3 **错误类 12 逐个** / §4 **不可断言项清单（8 条）** / §5 负例 56 / §6 正向 25 / §7 缺陷单 / §8 证据 / §9 后续批次。
  - **新建 `tests/REPORT.md`**：测试方法（黑盒 + 契约 T-R1…T-R4）、运行方式与退出码 0/1/2、用例构成（82 = 25 文件/547 assert + 56 负例 + 1 豁免）、覆盖摘要、不可断言项、复现命令、回归记录。
  - 追加 ADR：`.opencode/team/DECISIONS.md` [2026-09-27 18:20]（黑盒覆盖边界，影响 verifier）。
  - 证据：`cargo run --quiet -- test` → 82/82 PASS，exit 0；`cargo clean -p lfz; cargo build` → 0 warning / 0 error；`cargo test` → 431 passed / 0 failed。
- **P5.3（2026-09-27）**：内置函数全表 54 个逐项覆盖（8 文件 / 263 assert + 23 负例）。
- **P5.2（2026-09-27）**：控制流 / 函数 / 闭包 / 结构体 / 管道 / 富插值 / `;;` / A1 / A6 / `RecursionError`（10 文件 / 170 assert + 12 负例）。
- **P5.1（2026-09-27）**：`tests/` 黑盒测试基础设施（7 文件 / 114 assert + 21 负例 + 1 豁免）。

## 阻塞 / 需要支持
- **缺陷单 bug-20260927-01（已提交 team-lead，未修）**：`p["方法"]()` 经方括号取值再调用时**未绑定 `self`** → `NameError`。本批未新增影响（`test_structs.lfz` 仅断言 `type(p["norm2"])=="function"`，未用 `box["get"]()` 调用）。修复后需补端到端用例并回归。
- （56 条负例的实际错误类与 §8.1 逐项吻合；本批**无新解释器缺陷单**。）
- **待裁定（tooling 间隙，非黑盒缺陷）— `--json` 下 `;;` 输出未重定向**：契约 §9.3 称 `--json` 的 stdout 恒为唯一一行 JSON，但输出重定向仅覆盖内建 `print`/`input` 提示；`;;` dump 文本仍写 stdout，故含 `;;` 的 `test_dump.lfz` 会使 `test --json` 的 stdout 出现**多行**（JSON 恒在**最后一行**，实测 `ConvertFrom-Json` 取末行可解析：`ok=true total=82 passed=82`）。`src/**` 未改；已记录于 `tests/REPORT.md` §3.1「注意」与 `tests/coverage-matrix.md` §4。建议 team-lead 转 tooling-dev 裁定（重定向 `;;` 或收紧契约表述）。严重度：低；不阻塞主交付命令（非 `--json`）。

## 下一步计划
- **P5.5**：可移植性（BOM/CRLF/LF/CR、非 UTF-8 → `SyntaxError`）、入口一致性（REPL/stdin/`-e` 豁免）。
- **回归**：bug-20260927-01 修复后补 `p["方法"]()` 端到端用例并复跑全量。
- **待工具支持**：若 tooling-dev 提供 stdin / 程序 stdout 捕获能力，则把 `input`、`;;`/`print` 输出文本、错误 span 升级为端到端断言（当前见矩阵 §4）。

## 关键经验（写给未来的自己）
- **runner 契约**：默认根 = `tests/`；发现 `tests/**/*.lfz` 且按目录名排除 `fixtures`；每个被发现的 `.lfz` 首行必须恰为 `#42`；负例只能放 `tests/fixtures/` 并在 `tests/cases.json` 用 `expect.error`（§8.1 的 **12 类名之一**）声明。
- **判定（重要）**：清单声明 `expect.error` 且实际类名一致 → **PASS**（**含 `AssertionError`**：`fail` 用例在清单声明下判 PASS）。退出码 `0` 全 PASS / `1` 有 FAIL / `2` 有 ERROR。
- **stdout/stderr 不可断言**：非 `--json` 下 `print`/`eprint`/`check`/`;;` 写 stdout/stderr，runner 不捕获程序输出 → 只断言返回值/不报错。
- **内置名非一等值**：`type(type)` / 把内置名作实参 → `NameError`；spec §10.7 未声明可作值。**不要写 `type(type)`**。
- **`input` 有阻塞风险**：runner 不注入 stdin，交互终端下会挂起 → 黑盒**不写** `input()` 用例；由 Rust 单测覆盖。
- **计数口径**：正向 25 文件 / 547 assert；负例 56；合计 82。断言数用 `(Get-Content -Raw f) | Select-String 'assert\(' -AllMatches` 统计。
- **LFZ 语法陷阱**：语句分隔只能是换行；条件位/可迭代位禁裸 struct 字面量；`let` 不可重绑定（循环外累加用模块/函数内 `var`）；闭包按 cell 捕获。
- **环境**：PowerShell 下 `cargo` 写 stderr 会被包成 `NativeCommandError`，用 `2>&1 | Out-String` 再筛；批量写临时探针文件时用**单引号** PS 字符串避免 `"` 转义地狱；读完删干净临时文件（探针放 `%TEMP%\opencode`）。
