# runtime-dev — 工作状态
> 最后更新: 2026-09-27 21:19 by runtime-dev

## 当前状态
**T11 runtime 修复批次已完成（bug-20260927-03 / bug-20260927-04 / obs-B-03 + 抽样扩查新发现 2 类 panic），待 team-lead / verifier 核验。**

- 基线（本轮实测）：`cargo build` + `cargo build --all-targets` → **0 warning**；`cargo test` → **451 passed / 0 failed / 0 ignored**（lib 368 + main 47 + cli 24 + test_runner 12；基线 445，新增 **6**）；`target\debug\lfz.exe test` → **PASS 85 / FAIL 0 / ERROR 0**，exit **0**。
- 改动文件：`src/builtins.rs`（60+/4-）、`src/evaluator.rs`（198+/10-）、`src/error.rs`（32+/2-）；`git diff --stat -- src/` = **290 insertions(+), 16 deletions(-)**。

## 进行中
- （无）—— 待核验本批修复。

## 最近完成
### T11 修复批次 —— runtime panic 硬化（2026-09-27）
来源：team-lead 任务书「修复 runtime 侧缺陷 + 全量 panic 硬化排查（用户批准全修）」；依据 `docs/reports/conformance-C-builtins-cli.md` §③、`docs/reports/conformance-B-semantics.md` §3、`docs/spec/interface-contract.md` §10.7/§10.8、`semantics.md` §8.1/§8.2；**range/repeat 行为按 architect ADR `[2026-09-27 21:10]`**。

1. **bug-20260927-03（🔴）：`repeat` 溢出 → Rust panic（exit 101）** → 修为受控 `OverflowError`
   - 根因：`b_repeat` 的 `s.len().checked_mul(n)` 仅挡 `usize` 溢出；乘积落在 `(isize::MAX, usize::MAX]` 窄带时 `str::repeat` 库内 panic。
   - 修复：`src/builtins.rs b_repeat` 补 `total > isize::MAX as usize` → `overflow(span, OverflowMsg::Capacity)`；**同源** `string * int`（`src/evaluator.rs repeat_str`）同修。
   - 消息（ADR 精确文案）：`容量溢出：所需容量超出可分配上限`。
2. **bug-20260927-04（🟡）：`range` 超大 n → Rust panic（exit 101）** → 按 ADR 修为受控 `OverflowError`
   - 修复：`b_range` 加「`n × size_of::<Value>()` 超 `isize::MAX` → `OverflowError(Capacity)`」容量预检。
3. **obs-B-03（低）：索引越界写入插入符 span 偏差 1** → 修为与读取一致（指向基座首字符）
   - 修复：`exec_assign` 写路径统一用 `target.span`（基座 span）替代 `seg.span`；`read_segment` 增参 `err_span`。`a[5]=9` 由 col 2（`[`）→ col 1（基座 `a`）；`print(a[5])` = col 7（基座 `a`）。
4. **抽样扩查新发现 2 类可达 panic（审计报告未列）** → 修为受控 `ValueError`
   - ① 格式说明符**超大 width**（`"${1:99999999999d}"`）→ `memory allocation of 99999999998 bytes failed`（abort，exit -1073740791）→ `MAX_FMT_WIDTH = 1_000_000` 上限。
   - ② 超大 **precision**（`"${1.5:.65536f}"` 起）→ `core::fmt` panic `Formatting argument out of range`（exit 101）→ `MAX_FMT_PRECISION = 65_535` 上限（实测 65535 正常 / 65536 panic）。
   - 均归 `ValueError`（`格式说明符非法：…`，syntax §2.8「非法说明符 → ValueError」）。
5. **`src/error.rs`**：按 ADR 新增 `OverflowMsg::Capacity`（无字段）+ `message()` 分支（ADR 明示由 runtime-dev 落地 `error.rs`；归 `LfzError::Overflow` → 仍 `OverflowError`，**不新增错误类**）。

### 决策与证据（本轮）
- 新增单测 6 条：`builtins::tests::repeat_overflow_is_controlled_overflow_error`、`builtins::tests::range_huge_n_is_controlled_capacity_overflow_error`、`error::tests::overflow_msg_capacity_is_the_second_template`、`evaluator::tests::string_mul_overflow_is_controlled_overflow_error`、`evaluator::tests::index_write_oob_span_points_to_base_like_read`、`evaluator::tests::format_spec_extent_is_bounded_without_panic`。
- 修复前/后证据（命令原文与退出码）见 `JOURNAL.md` 本轮条目。
- 改动边界：仅 `src/builtins.rs` / `src/evaluator.rs` / `src/error.rs`；**未改** `src/cli.rs`、`src/value.rs`、`tests/**`、`docs/spec/**`、`app/**`、`docs/guide/**`、`.opencode/skills/**`；未 commit / tag / push。`error.rs` 的修改依据 architect ADR 授权（非越界）。

## panic 硬化审计清单（`src/**`，可由 LFZ 程序触发）
| # | 位置 | 触发构造 | 可达性 | 处置 | 状态 |
|---|------|---------|--------|------|------|
| 1 | `builtins.rs b_repeat` | `repeat(2^62,"ab")` | ✅ 实测 panic（101） | → `OverflowError(Capacity)` | ✅ 已修 |
| 2 | `evaluator.rs repeat_str` | `"ab" * 2^62` | ✅ 实测 panic（101） | → `OverflowError(Capacity)` | ✅ 已修 |
| 3 | `evaluator.rs format_value`（`f`/`e`） | `"${1.5:.65536f}"` | ✅ 实测 panic（101） | precision 上限 → `ValueError` | ✅ 已修 |
| 4 | `evaluator.rs FmtSpec::pad`（width） | `"${1:99999999999d}"` | ✅ 实测 alloc abort | width 上限 → `ValueError` | ✅ 已修 |
| 5 | `builtins.rs b_range` | `range(2^62)` | ✅ 实测 panic（101） | 容量预检 → `OverflowError(Capacity)` | ✅ 已修（ADR） |
| 6 | `parser.rs` 递归下降 | `((((…))))`/`[[[[…]]]]` ~2e5 层 | ✅ 实测 main 栈溢出（exit -1073741571） | 加解析深度上限 + `SyntaxError` | ⚠️ **非本模块**（core-dev），已上报 |
| 7 | `value.rs Display` 递归 | `a=[a]×N` | 2e5 实测**未**溢出（eval 大栈线程）；极大 N 理论残余 | 无（`==` 已由 `RecursionError` 保护） | ⚠️ 残余，已上报 |
| 8 | `evaluator.rs`/`builtins.rs` 的 `expect`/`unreachable!` | — | 均**逻辑不可达**（`current.expect`（op==Assign 已分支）/`total_cmp.expect`（validate_orderable 前置）/`as_f64.expect`（both_num 前置）等） | 保留（不 panic 于可达输入） | ✅ 已论证安全 |
| 9 | `cli.rs` / `json.rs` / `test_runner.rs` 的 `unwrap/expect/panic` | — | 均在 `#[cfg(test)]` 或 CLI 内部（非 LFZ 程序可达）/ 可信 `cases.json` | 不改（非 runtime 域） | N/A |
| 10 | `loader.rs` / `lexer.rs` | — | 迭代实现，无可达 panic | — | N/A |

## 阻塞 / 需要支持
- **无硬阻塞**。三项未决（已上报 team-lead，未自行决定）：
  1. `src/parser.rs` 深嵌套源 → main 线程栈溢出（core-dev 域；建议 parse 深度上限 + `SyntaxError`）。
  2. `range` 容量预检边界为 `isize::MAX` 字节（ADR 所定）；`n×16 ≤ isize::MAX` 但物理不可分配时仍会 OOM-abort（彻底可控需 `Vec::try_reserve`，待裁定）。
  3. `Display` 无深度上限（实测 2e5 未溢出，仅理论残余）。

## 下一步计划
- 待 team-lead 核验本批；建议 release-manager 以原子提交落地（建议信息：`fix(runtime): controlled OverflowError for repeat/range capacity overflow; fix index-write span; bound format-spec width/precision (T11)`）。
- 建议 verifier 按 C 域原命令复现（`e_repeat_min.lfz` / `e_range_big.lfz` 应 exit 2 + `OverflowError`）；建议 test-engineer 为 `range`/`repeat` 超大入参补黑盒负例（只断类，不断消息）。

## 关键经验（写给未来的自己）
- 本机 **cargo/rustc 不在默认 PATH**：先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`。
- **⚠️ 改 `src/*.rs` 只用 `edit` / `write` 工具**（PowerShell `Get-Content`/`Set-Content` 会按 ANSI/GBK 读写 UTF-8，结构损坏）。
- **warning/error 计数**：`cargo build --tests --message-format=json 2>$null | Select-String '"level":"warning"'`（勿 `2>&1`）。
- **测 panic / 看 CLI 中文 stderr 的坑**：PowerShell `Set-Content -Value "...`f..."` 会把 `` `f `` 当 **form feed**（0x0C），务必用**单引号拼接**或 `[IO.File]::WriteAllText`。cmd `2> file` 落盘的 stderr 是 **UTF-8 原始字节**；控制台按 GBK 显示会乱码，用 `[Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes(...))` 或 hex 核验（`容量溢出：…` = `e5 ae b9 e9 87 8f …`）。
- **panic 排查先找"窄带"**：`checked_mul` 只挡 `usize` 溢出，**挡不住 `isize::MAX` 容量上限**（Rust `String`/`Vec` 分配上限）→ 会出现 exit 101 `capacity overflow`。
- **`core::fmt` 动态精度硬上限**：`format!("{:.*}", p, x)` 在 `p ≥ 65536` 直接 panic（实测 65535 ok）；**width 无此限制**但超大 width 会 OOM-abort。
- **§8.2 插入符位置**：读取路径（`ExprKind::Index`/`Field`）span = **基座起始**（parser `parse_postfix` 不变量）；写入路径的 `LvalueSeg.span` = `.`/`[` 位置，**不同口径** → 统一取 `target.span`（Lvalue 基座 span）。
- **`.lfz` 文件必须有 `#42\n` 前缀**；`cmd 里 echo %ERRORLEVEL% 不可信`，取退出码用 `$LASTEXITCODE` / `[Diagnostics.Process]::ExitCode`。
- **模块边界红线**：`cli.rs`（tooling-dev）、`tests/**`（test-engineer）、`docs/spec/**`（language-architect）、`app/**`/`docs/guide/**`/`.opencode/skills/**` 一律不改。`error.rs` 本轮按 architect ADR 明示授权落地 `OverflowMsg::Capacity`。
