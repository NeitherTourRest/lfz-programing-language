# test-engineer — 工作状态
> 最后更新: 2026-09-27 by test-engineer

## 当前状态
**P5.1 ✅ 完成**（黑盒测试第一批：目录结构 + 基础语言特性用例 + 清单 + 覆盖矩阵骨架）。
`cargo run --quiet -- test` → **29 PASS / 0 FAIL / 0 ERROR，exit 0（全绿）**。

## 进行中
- （无）

## 已完成
- **P5.1（2026-09-27）**：`tests/` 黑盒测试基础设施 + 基础特性用例
  - 目录：`tests/lfz/`（正向，自动发现域）、`tests/fixtures/`（负例/豁免夹具，非自动发现）。
  - 用例：正向 **7** 文件（共 **114** 条 `assert`；覆盖字面量 int/float/string(转义)/bool/nil、标识符与 let/var、算术/比较/逻辑/一元、优先级与结合性、`/` 恒 float+`div`+`%`、i64 边界、`#42` 严格性）；负例 **21** 条 + 非 `.lfz` 正向豁免 **1** 条（`tests/cases.json` 以 `expect.error` 声明）。
  - 覆盖矩阵：`tests/coverage-matrix.md`（含 §5 后续批次特性全集骨架）。
  - 证据：`cargo run --quiet -- test` → 29/29 PASS，exit 0；`cargo build` **0 warning**（`cargo clean -p lfz` 后 clean 重编）；`cargo test` **431 passed / 0 failed**。

## 阻塞 / 需要支持
- （无）

## 下一步计划
- **P5.2**：字符串插值 `${}` 与格式说明符、数组（索引/负索引/越界/切片/赋值/快照）、struct（字面量/模板/`.字段`/`["键"]`/方法/动态字段）。
- **P5.3**：控制流（if/else/while/for/break/continue/return）、函数与闭包、管道 `|>`/`_`、`;;` dump。
- **P5.4**：内置函数 54 个（data-last）、值显示与环安全、相等语义、错误模型全量（12 类 + traceback 折叠 + `--json`）。
- **P5.5**：可移植性（BOM/CRLF/LF/CR、非 UTF-8 → SyntaxError）、入口一致性（REPL/stdin/`-e` 豁免）。

## 关键经验（写给未来的自己）
- **runner 契约**：默认根 = `tests/`；发现 `tests/**/*.lfz` 且**按目录名排除 `fixtures`**；每个被发现的 `.lfz` 首行必须恰为 `#42`（否则 ERROR）；负例只能放非自动发现域（`tests/fixtures/`）并在 `tests/cases.json` 用 `expect.error`（§8.1 的 **12 类名之一**）声明；**非 `.lfz` 夹具豁免 `#42`**，但必须**在清单显式声明路径**才会被运行。
- **判定**：`assert`/`fail` 失败 → FAIL；其它错误类 → ERROR；命中 `expect.error` → PASS；`check` 失败非致命 → PASS。
- 断言一律 `assert(cond, "feature: expected ...")`；**错误用例只断言「错误类」**，不逐字断言中文文案（事实源 interface-contract §8.1）。
- **陷阱**：`.txt` 正向夹具放 `fixtures/` 时，清单里**不加** `expect` 字段才会被判正常运行并 PASS。
- **环境**：PowerShell 下 `cargo` 写 stderr 会被包成 `NativeCommandError`；需 `2>&1 | Out-String` 再筛 `warning|error` 才可靠判断 0 warning。
- 目录/路径书写务必核对（本次因一处路径笔误误建项目外目录，已即时删除；写外部文件前先 `Test-Path` 校验父目录）。
