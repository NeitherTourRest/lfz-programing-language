# test-engineer — 工作状态
> 最后更新: 2026-09-27 by test-engineer

## 当前状态
**P5.8 ✅ 完成** —— 解析期上限回归补测（为 `syntax.md` §3.8 `PARSE_DEPTH_LIMIT = 1000` / §3.9 `AST_DEPTH_LIMIT = 10000` 补「限内放行正例 + 超限负例」双向回归）。
`cargo run -q -- test` → **90 PASS / 0 FAIL / 0 ERROR，exit 0**（87→90：+2 负例夹具 +1 正例文件）。
`cargo run -q -- test --json` → 单行 `{"ok":true,"total":90,"passed":90,"failed":0,"errored":0,...}`。
直跑负例均 `SyntaxError` + `$LASTEXITCODE = 2`（不再 panic / abort）；直跑正例 `$LASTEXITCODE = 0`、stderr 空。
`cargo build` → **0 warning / 0 error**；`cargo test` → **482 passed / 0 failed / 0 ignored**（lib 379 + main 48 + cli 28 + test_runner 12 + unit 15；`src/**` 含 §3.8/§3.9 变更后的实测，非本批引入）。

## 进行中
- （无）

## 已完成
- **P5.8（2026-09-27）**：解析期上限回归补测（§3.8 / §3.9）
  - **新增负例 +2**（大体积夹具，按 T-R2 放非自动发现目录 `tests/fixtures/`）：
    - `tests/fixtures/parse_nesting_overflow.lfz` = `#42` + `(`×1001 `1` `)`×1001 → `SyntaxError: 嵌套深度超限（超过 1000 层）`；
    - `tests/fixtures/ast_depth_overflow.lfz` = `#42` + `1` `+1`×9999（10000 项）→ `SyntaxError: 表达式嵌套过深（超过 10000 层）`。
  - **`tests/cases.json` 61→63 条**（60→62 `expect.error` + 1 豁免）：两条声明 `expect.error = "SyntaxError"`（仅断言错误类，不逐字断消息，符合 §4 纪律）。
  - **新增正例文件 `tests/lfz/test_parse_limits.lfz`（2 `assert`）**：`(`×1000（嵌套深度恰 1000）合法、`1+1+…` 9999 项（AST 深度恰 10000）合法 → 限内放行。
  - **断言口径**：runner 只断言**错误类**；「退出码 = 2 / = 0、消息关键片段」**以直跑 `lfz run <file>` 取证**（runner 为进程内执行，无退出码断言通道，见 REPORT §8）。
  - **`tests/coverage-matrix.md` 同步**：§0.1（合计 87→90、正向 26→27 文件 / 597→599 assert、负例 60→62）、§0.3 覆盖结论、§1.1 新增 2 行（§3.8 / §3.9 解析期上限）、§1.2 新增 1 行、§3 错误类表（`SyntaxError` 11→13、合计 62）、§5 新增 **§5.6 P5.8（编号 61/62）**、§6.5 正向（1 文件 / 2 assert，合计 27/599）、§8 运行证据（90/90 + 直跑 exit 2）；header 状态加「解析期上限回归（P5.8）」。
  - **`tests/REPORT.md` 同步**：§1（87→90 + P5.8 结论）、§2.2 T-R2/T-R4（60→62 / 26→27）、§3.1（`--json` 87→90）、§4 计数表（+P5.8 行、合计 90）、错误类分布（`SyntaxError` 11→13、合计 62）、§5.1 新增解析期上限行、§5.3（`SyntaxError` 13）、§8 复现（90）、§9 回归表（P5.8 四行）。
  - **未做（按指令）**：未新增 100000 项夹具（任务仅要求 2 条负例；100000 项仅作带外证据直跑，结果 exit 2）；未改 `src/**`、`docs/spec/**`、`app/**`、`docs/guide/**`、`.opencode/skills/**`；未删除/改写任何既有用例（仅新增 + 更新矩阵/计数）。
  - 证据：`cargo run -q -- test` → 90/90，exit 0；`--json` 单行 total 90；直跑两负例 → `SyntaxError` + exit 2；直跑正例 → exit 0；`cargo build` 0 warning / 0 error；`cargo test` 482 passed。
- **P5.7（2026-09-27）**：容量溢出回归补测（bug-20260927-03/04）—— `repeat`/`range` 超大入参 → 负例 `OverflowError`；`cargo run -q -- test` 87/87。
- **P5.6（2026-09-27）**：等价路径双路补测（`test_equivalence_paths.lfz` 41 assert + `test_structs.lfz` 补强 + 2 负例）。
- **P5.4（2026-09-27）**：黑盒测试集定稿批（12 类错误核对 11/12；54 内置 → 53 黑盒 + 1 跳过；矩阵定稿 + REPORT）。
- **P5.3 / P5.2 / P5.1**：内置全表 54 逐项；控制流/函数/闭包/结构体/管道/插值/`;;`/A1/A6；`tests/` 基础设施。

## 阻塞 / 需要支持
- **（工具链限制，非阻塞）退出码断言通道缺失**：`cases.json` schema 仅支持 `expect.error`（错误类），无「退出码」字段；故本轮「退出码 = 2」仍以直跑夹具取证（REPORT §8）。若需机器断言退出码，须 tooling-dev 扩 runner 契约。
- **（环境，非阻塞）3 个孤儿 `lfz.exe` 进程**：早前一次管道式抓取在 20 KB stderr 上死锁，残留 3 个阻塞在已关闭管道写端的 `lfz.exe`（`taskkill` 报拒绝访问，无害、不占项目文件锁）。已改用文件重定向取证，后续不再复现。

## 下一步计划
- **P5.5**：可移植性（BOM/CRLF/LF/CR、非 UTF-8 → `SyntaxError`）、入口一致性（REPL/stdin/`-e` 豁免）。
- **建议（待 team-lead 定夺）**：为「旧崩溃点 `1+1+…` 100000 项不得 abort」补一条大体积负例夹具（本轮仅带外直跑取证，未落夹具）。
- **回归**：`src/**` 每次变更后复跑全量，结果追加 `tests/REPORT.md` §9。

## 关键经验（写给未来的自己）
- **进程上限回归铁律**：任何「非法/超大入参 → 进程不得崩溃」的修复，都要在 `tests/fixtures/` 补「断言错误类」负例；runner 为**进程内执行**，若 panic/abort 复现，整轮 `lfz test` 会被带崩（远超单用例失败）——该负例本身就是强回归护栏。
- **超限负例用最小构造**：§3.8 用 `(`×1001（2 KB）、§3.9 用 `1+1+…`×10000（20 KB），正好越界一格；大体积夹具按 T-R2 放 `fixtures/`（自动发现排除）。
- **限内边界要正例**：只测「超限报错」会漏掉「上限过小、把合法程序误杀」的回归；必须成对（恰 1000 / 恰 9999 放行）。
- **改动 20 KB 级别夹具勿用管道抓 stderr**：错误块会回显整行源码（≈20 KB），`ReadToEnd()` 顺序读 stdout/stderr 会**管道死锁**；用 `cmd /c ... > out 2> err` 文件重定向，再以 UTF-8 读。
- **PowerShell 数组陷阱**：`@('prefix' + $var, ...)` 会把 `'prefix'` 与 `$var` 拆成两个元素（中间插入换行）→ 生成的多行程序被解析为「行尾不能终止表达式」；须**先算 `$line = 'prefix' + $var` 再放入数组**。
- **runner 契约**：默认根 = `tests/`；发现 `tests/**/*.lfz` 且排除 `fixtures`；每个被发现的 `.lfz` 首行必须恰为 `#42`；负例只能放 `tests/fixtures/` 并在 `tests/cases.json` 用 `expect.error`（§8.1 的 12 类名之一）声明。
- **退出码无黑盒通道**：`cases.json` 只有 `expect.error`；要证「退出码 = 2」只能直跑 `lfz run <fixture>` 取 `$LASTEXITCODE`，并在 REPORT §8 留证据。
- **stdout/stderr 不可断言**：非 `--json` 下 `print`/`eprint`/`check`/`;;` 写 stdout/stderr，runner 不捕获 → 只断言返回值/不报错。
- **计数口径**：正向 27 文件 / 599 assert；负例 62；合计 90。断言数用 `(Get-Content -Raw f) | Select-String 'assert\(' -AllMatches` 统计。
