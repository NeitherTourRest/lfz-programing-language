# test-engineer — 工作状态
> 最后更新: 2026-09-27 by test-engineer

## 当前状态
**P5.6 ✅ 完成**（关闭「等价路径接缝盲区」：verifier《P9-verification.md》rev.2 §9.4 建议项；本项目第 3 次同类盲区）。
`cargo run --quiet -- test` → **85 PASS / 0 FAIL / 0 ERROR，exit 0**（82→85）。
`cargo clean -p lfz; cargo build` → **0 warning / 0 error**；`cargo test` → **432 passed / 0 failed / 0 ignored**（lib 362，`src/**` 未改）。

## 进行中
- （无）

## 已完成
- **P5.6（2026-09-27）**：等价路径双路补测
  - 新增 `tests/lfz/test_equivalence_paths.lfz`（**41 `assert`**）：`s.k≡s["k"]` 读 / 写 / 调用 / 改 `self`（同一实例）；复合赋值脱糖；管道 data-last 脱糖（sum/take/slice/map/filter/reduce/len/裸函数/占位符）；块注释 ≡ 空格；`a--b ≡ a-(-b)`；逻辑分组 + 短路；`/` vs `div` **非等价并置**。
  - 补强 `tests/lfz/test_structs.lfz`（**27→36 `assert`**）：新增 `p["norm2"]()==25`、`p["describe"]()`、`cc["bump"]()` 写回**同一实例**、两实例分别用点/方括号驱动最终值相同、数据字段写 `.k`/`["k"]` 两路可见；**删除过时注释**（"实现仅在 `.字段()` 调用点绑定"在 `6aabdf5` 后已不成立）。
  - 负例 **+2**：`tests/fixtures/{field_dot_missing,field_bracket_missing}.lfz` → `FieldError`（缺失键读取两取法等价报错）；`tests/cases.json` 56→58。
  - `tests/coverage-matrix.md`：新增 **§1.3「等价路径双路覆盖小节」**（E1–E11 + 诚实列出「未找到等价表述 / 无法断言」5 条）；§0/§1/§3/§5/§6/§7/§8 全量同步；bug-01 标记**已修复**。
  - `tests/REPORT.md`：同步 82→85、547→597、56→58、bug-01 已修复、回归表加 P5.6 行。
  - 追加 ADR：`.opencode/team/DECISIONS.md` [2026-09-27 19:45]（等价双路纪律，影响 verifier）。
  - 证据：`cargo run --quiet -- test` → 85/85 PASS，exit 0；`cargo clean -p lfz; cargo build` → 0 warning / 0 error；`cargo test` → 432 passed（lib 362）。
- **P5.4（2026-09-27）**：黑盒测试集定稿批
  - 12 类错误核对（`interface-contract.md` §8.1）：**11/12** 黑盒 `expect.error`；唯 `IOError` 黑盒跳过（runner 无 stdin）→ Rust 单测覆盖。
  - 内置 §10.7：54 → **53 黑盒 + 1 跳过（`input`）**。`tests/coverage-matrix.md` 重构定稿 + 新建 `tests/REPORT.md`；ADR [18:20]。
- **P5.3（2026-09-27）**：内置函数全表 54 个逐项覆盖（8 文件 / 263 assert + 23 负例）。
- **P5.2（2026-09-27）**：控制流 / 函数 / 闭包 / 结构体 / 管道 / 富插值 / `;;` / A1 / A6 / `RecursionError`（10 文件 / 170 assert + 12 负例）。
- **P5.1（2026-09-27）**：`tests/` 黑盒基础设施（7 文件 / 114 assert + 21 负例 + 1 豁免）。

## 阻塞 / 需要支持
- **bug-20260927-01（已修复，本轮补测）**：`p["方法"]()` 未绑定 `self` → 已由 runtime-dev 于 commit **`6aabdf5`** 修复；本批补端到端用例（`struct_method_call_bracket` / `method_bracket_mutates_same_instance` / `equiv_self_mutation_*`）并回归全绿。verifier P9 rev.2 §9.1 复验已闭合。
- **待裁定（tooling 间隙，非黑盒缺陷）— `--json` 下 `;;` 输出未重定向**：契约 §9.3 称 `--json` 的 stdout 恒为唯一一行 JSON，但输出重定向仅覆盖内建 `print`/`input` 提示；`;;` dump 文本仍写 stdout，故含 `;;` 的 `test_dump.lfz` 会使 `test --json` 的 stdout 出现**多行**（JSON 恒在**最后一行**，实测末行 `{"ok":true,"total":85,...}` 可解析）。`src/**` 未改；记录于 `tests/REPORT.md` §3.1 与矩阵 §4。建议 team-lead 转 tooling-dev 裁定。严重度：低；不阻塞主命令（非 `--json`）。

## 下一步计划
- **P5.5**：可移植性（BOM/CRLF/LF/CR、非 UTF-8 → `SyntaxError`）、入口一致性（REPL/stdin/`-e` 豁免）。
- **回归**：`src/**` 每次变更后复跑全量，含本批等价用例；结果追加 `tests/REPORT.md` §9。
- **待工具支持**：若 tooling-dev 提供 stdin / 程序 stdout 捕获，则把 `input`、`;;`/`print` 输出文本、错误 span 升级为端到端断言（当前见矩阵 §4）。

## 关键经验（写给未来的自己）
- **等价双路（新增铁律）**：凡 spec 出现「等价 / ≡ / 脱糖为 / 两写法」，**两条路径都必须有断言且结果一致**；单路覆盖视为未覆盖。本项目已踩 3 次（`/` vs `div`、`_` 占位符、`s["k"]()` 未调用）→ 专节矩阵 §1.3。
- **runner 契约**：默认根 = `tests/`；发现 `tests/**/*.lfz` 且排除 `fixtures`；每个被发现的 `.lfz` 首行必须恰为 `#42`；负例只能放 `tests/fixtures/` 并在 `tests/cases.json` 用 `expect.error`（§8.1 的 **12 类名之一**）声明。
- **判定**：清单声明 `expect.error` 且实际类名一致 → **PASS**（含 `AssertionError`）；退出码 `0` 全 PASS / `1` 有 FAIL / `2` 有 ERROR。
- **stdout/stderr 不可断言**：非 `--json` 下 `print`/`eprint`/`check`/`;;` 写 stdout/stderr，runner 不捕获 → 只断言返回值/不报错。
- **内置名非一等值**：`type(type)` / 内置名作实参 → `NameError`；**不要写**。
- **`input` 有阻塞风险**：黑盒**不写** `input()`；由 Rust 单测覆盖。
- **LFZ 语法陷阱**：语句分隔只能是换行；条件位/可迭代位禁裸 struct 字面量；`let` 不可重绑定；闭包按 cell 捕获。**已验证可用**：`a[i] += v`、`s.k += v`、`5--3`、内联 `/*c*/`、`L |> f(_)`、`p["k"]()`。
- **计数口径**：正向 26 文件 / 597 assert；负例 58；合计 85。断言数用 `(Get-Content -Raw f) | Select-String 'assert\(' -AllMatches` 统计。
- **环境**：PowerShell 下 `cargo` 写 stderr 会被包成 `NativeCommandError`，用 `2>&1 | Out-String` 再筛；临时探针放 `%TEMP%\opencode`，读完删干净（本轮 9 个 probe 已清理）。
