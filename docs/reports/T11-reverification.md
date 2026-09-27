# T11 修复批次全量复验报告（独立验收 · 只验证不修复）

> 复验人：verifier
> 日期：2026-09-27
> 任务：回答一个二值问题——**这 7 类修复是否真正通过？**
> 依据：`DECISIONS.md` 2026-09-27 的 6 条 ADR · `docs/reports/conformance-{A,B,C}-*.md` · `TEAM_BOARD.md` T11-② 结果表
> 方法：**只验证、不修复**；全部条款**实际执行**（命令 + 原始输出 + 退出码）；夹具在 `<repo>/Temp/T11/`，收工前自清。

---

## ① 复验基线（产物路径 + 字节数 + mtime + 与源码同源证明）

⚠️ **本轮绝未使用 `dist/lfz.exe`**（修复前构建）。全部复验使用 **`cargo build --release` 直接从当前工作树源码构建**的产物。

| 项 | 值 | 说明 |
| -- | -- | -- |
| 复验产物 | `target\release\lfz.exe` | 由 `cargo build --release` 产出 |
| 字节数 | **710144 B** | 与修复前 `dist/lfz.exe`（704000 B）不同 |
| mtime | **2026-09-27 22:26:59** | 本次构建时刻 |
| SHA256 | `619AFD2ED0C194240A774D026C4B721A36384518D3899476A390BB118F433051` | |
| 源码基点 | HEAD `34e445f2128b9099b23f4de18b712a55ffd6ee4b` | + 工作树未提交修改 |
| 未提交的 `src/**` | `builtins.rs` `cli.rs` `error.rs` `evaluator.rs` `parser.rs` | 修复批次改动 |

**同源证明（可复现）：**
1. 首跑 `cargo build --release` → `Compiling lfz v1.0.0` + `Finished release profile in 10.60s`（**真的编译了**当前源码）。
2. 二跑 `cargo build --release` → `Finished release profile in 0.03s`（**无重编译**，即产物已是最新 = 与源码一致）。
3. 全部 `src/*.rs` 的 mtime 均 ≤ **2026-09-27 22:13:19**（`parser.rs`，最晚）< 产物 mtime **22:26:59**。
4. 复验全程结束后复核：产物仍为 710144 B / 同 SHA256（未被复验动作污染）。

**本轮被弃用（对照）：** `dist\lfz.exe` = 704000 B，mtime `2026-09-27 13:01:12`，SHA256 `BF6912990E7B675F464BA039A8D61D1F2262542B510E8E7290F0DDBB5EAA51DF` —— **修复前**构建。

**环境：** Windows 10/11 + rustc/cargo 1.98.1（`x86_64-pc-windows-msvc`），PowerShell 5.1。构建 profile：`release`（`lto=true, codegen-units=1, strip=true`）。

---

## ② 逐条复验（条款 → 命令 → 原始输出/退出码 → 结论）

### 条款 1 — `repeat` 容量溢出 → `OverflowError` + exit 2（原 panic / 101）

夹具 `Temp/T11/t_repeat_big.lfz`：
```
#42
repeat(4611686018427387904, "ab")
```
命令：`target\release\lfz.exe run t_repeat_big.lfz`
```
EXIT: 2  (0x00000002)
STDOUT_BYTES: 0
STDERR_BYTES: 190
Traceback (most recent call last):
  File "t_repeat_big.lfz", line 2, in <module>
    repeat(4611686018427387904, "ab")
    ^
OverflowError: 容量溢出：所需容量超出可分配上限
```
**结论：PASS**（exit 2，非 101；错误类与消息逐字符符合 ADR `OverflowMsg::Capacity`）。

### 条款 2 — `range` 超大 n → `OverflowError` + exit 2

夹具 `Temp/T11/t_range_big.lfz`：
```
#42
range(4611686018427387904)
```
命令：`target\release\lfz.exe run t_range_big.lfz`
```
EXIT: 2  (0x00000002)
STDOUT_BYTES: 0
STDERR_BYTES: 182
Traceback (most recent call last):
  File "t_range_big.lfz", line 2, in <module>
    range(4611686018427387904)
    ^
OverflowError: 容量溢出：所需容量超出可分配上限
```
**结论：PASS**（exit 2，非 panic）。

### 条款 3 — 解析嵌套深度上限（`PARSE_DEPTH_LIMIT=1000`）

| 子项 | 夹具 | 命令 | 退出码 | 末行/JSON | 结论 |
| -- | -- | -- | -- | -- | -- |
| 3a `(`×1000（平衡） | `t_par_1000_bal.lfz` | `run` | **0** | （空） | **PASS** |
| 3b `(`×1001（平衡） | `t_par_1001_bal.lfz` | `run` | **2** | `SyntaxError: 嵌套深度超限（超过 1000 层）` | **PASS** |
| 3c `(`×1001（前缀） | `t_par_1001_pre.lfz` | `run` | **2** | 同上 | **PASS** |
| 3d `[`×1001（前缀） | `t_brk_1001_pre.lfz` | `run` | **2** | 同上 | **PASS** |
| 3e `[`×1001（平衡） | `t_brk_1001_bal.lfz` | `run` | **2** | 同上 | **PASS** |
| 3f `{"a":`×1001 | `t_struct_1001_pre.lfz` | `run` | **2** | 同上 | **PASS** |
| 3g `fn(){`×1001 | `t_fn_1001_pre.lfz` | `run` | **2** | 同上 | **PASS** |
| 3h `(`×1001 `--json` | `t_par_1001_bal.lfz` | `--json run` | **2** | `{"ok":false,"error":"SyntaxError","message":"嵌套深度超限（超过 1000 层）","file":"t_par_1001_bal.lfz","line":2,"col":1001,...}` | **PASS** |

关键点证实：
- 全部 `exit 2`，**无一是 `-1073741571`（0xC0000005）**（原栈溢出 abort）。
- `(`×1000 → exit 0 且无输出；`(`×1001 → 受控 `SyntaxError`。
- `--json` 的 `col = 1001`（第 1001 层开启记号），12 个 `error` 取值不变（仍 `SyntaxError`）。

**结论：PASS（8/8）。**

### 条款 4 — 表达式/结构深度上限（`AST_DEPTH_LIMIT=10000`）

| 子项 | 夹具 | 命令 | 退出码 | 末行 | 结论 |
| -- | -- | -- | -- | -- | -- |
| 4a `1+1+…` 9999 项 | `t_add_9999.lfz` | `run` | **0** | （空） | **PASS** |
| 4b `1+1+…` 10000 项 | `t_add_10000.lfz` | `run` | **2** | `SyntaxError: 表达式嵌套过深（超过 10000 层）` | **PASS** |
| 4c `1+1+…` 10001 项 | `t_add_10001.lfz` | `run` | **2** | 同上 | **PASS** |
| 4d `1+1+…` 100000 项 | `t_add_100000.lfz` | `run` | **2** | 同上（**未 abort**） | **PASS** |
| 4e `a[0][0]…` 10001 层 | `t_idx_10001.lfz` | `run` | **2** | 同上 | **PASS** |
| 4f `a[0][0]…` 100000 层 | `t_idx_100000.lfz` | `run` | **2** | 同上（**未 abort**） | **PASS** |
| 4g `1+1+…` 10000 项 `--json` | `t_add_10000.lfz` | `--json run` | **2** | `{"ok":false,"error":"SyntaxError","message":"表达式嵌套过深（超过 10000 层）","...","line":2,"col":1,...}` | **PASS** |

关键点证实：
- 边界精确：9999 → 0，10000 → 2（含 `ExprStmt` 包装层 ⇒ 深度 10001）。
- **N=100000 从 `-1073741571`（abort）变为受控 exit 2**（第 4 类残余已收口）。
- 索引链同类示例（10001 / 100000）均为受控 `SyntaxError`。

**结论：PASS（7/7）。**

### 条款 5 — `;;` × `--json` 同通道（`bug-B-20260927-01`）

夹具 `t_dumpjson.lfz` = `#42` / `let z = 5` / `;;`；`t_dumpjson_interleave.lfz` = `#42` / `print("A")` / `let z = 5` / `;;` / `print("B")`。

| 子项 | 命令 | 退出码 | stdout | stderr | 结论 |
| -- | -- | -- | -- | -- | -- |
| 5a `--json run` 含 `;;` | `--json run t_dumpjson.lfz` | **0** | **12 B / 恰 1 行** `{"ok":true}` | **8 B** `z ： 5` | **PASS** |
| 5b 普通 `run` 回归 | `run t_dumpjson.lfz` | **0** | **8 B** `z ： 5` | **0 B** | **PASS** |
| 5c 交织顺序 | `--json run t_dumpjson_interleave.lfz` | **0** | 12 B 单 JSON `{"ok":true}` | **12 B** `A` / `z ： 5` / `B`（按执行序） | **PASS** |

**原始逐字节证据（C5a）：**
```
EXIT: 0  STDOUT_BYTES: 12   STDOUT: {"ok":true}
             STDERR_BYTES: 8    STDERR: z ： 5
```
证明：`--json` 下 stdout 恰为**单个 JSON**（12 B = `{"ok":true}\n`，1 行）；`;;` dump 随 `print` 转 **stderr**；**非 json 回归**下 `;;` 仍在 **stdout**（8 B）。

**结论：PASS（3/3）。**

### 条款 6 — 越界写 span 对齐（`obs-B-03`）

夹具 `t_oob_write.lfz` = `#42` / `let a = [1, 2, 3]` / `a[5] = 9`；`t_oob_read.lfz` = 同前 / `print(a[5])`。

```
[写] run  t_oob_write.lfz   EXIT 2
  File "t_oob_write.lfz", line 3, in <module>
    a[5] = 9
    ^                        ← 插入符指向基座 a（源列 1）
  IndexError: 下标 5 越界（长度 3）

[写] --json run t_oob_write.lfz  EXIT 2
  {"ok":false,"error":"IndexError","message":"下标 5 越界（长度 3）","file":"t_oob_write.lfz","line":3,"col":1,...}

[读] run  t_oob_read.lfz    EXIT 2
  File "t_oob_read.lfz", line 3, in <module>
    print(a[5])
          ^                  ← 插入符指向基座 a（源列 7）
  IndexError: 下标 5 越界（长度 3）

[读] --json run t_oob_read.lfz   EXIT 2
  {"ok":false,"error":"IndexError","...","line":3,"col":7,...}
```
关键点证实：写路径 `col = 1`（`a` 基座，**原为 col 2**）；读路径 `col = 7`（`a` 基座）；**二者指向同一语义节点（基座标识符）**，口径一致。

**结论：PASS（4/4）。**

### 条款 7 — `syntax.md` §9.2 样例 B 逐字复制实跑

夹具 `t_s92_b.lfz` = 从 `docs/spec/syntax.md` 第 723–768 行**逐字提取**（UTF-8 无 BOM）。
命令：`target\release\lfz.exe run t_s92_b.lfz` → **EXIT 0**，`STDOUT_BYTES: 377`，stderr 0。
```
Word ： <struct Word>
countWords ： <fn countWords>
text ： the quick the fox the dog a quick fox
freq ： {a: {count: 1, text: "a"}, dog: {count: 1, text: "dog"}, fox: {count: 2, text: "fox"}, quick: {count: 2, text: "quick"}, the: {count: 3, text: "the"}}
词频 Top 3
         the │   3
         fox │   2
       quick │   2
pi=3.142  n=00042  hex=2a  right=    42
```
将实际输出的程序主体 5 行与 spec 订正后的期望块（第 783–787 行）做**逐字节比较**：
```
EXPECTED: 词频 Top 3 /          the │   3 /          fox │   2 /        quick │   2 / pi=3.142  n=00042  hex=2a  right=    42
ACTUAL  : （同上）
BYTE_EXACT_MATCH=True
```
**结论：PASS**（顺序为 `the / fox / quick`，与 spec 订正版**逐字节一致**）。

### 条款 8 — 全量回归

**8a `cargo test`**（exit 0）——五个 harness（+1 空 doc harness）：
```
test result: ok. 379 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   （unittests src\lib.rs）
test result: ok.  48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   （unittests src\main.rs）
test result: ok.  28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   （tests\cli.rs）
test result: ok.  12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   （tests\test_runner.rs）
test result: ok.  15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   （tests\unit\main.rs）
test result: ok.   0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   （doc tests）
```
合计 **482 passed / 0 failed / 0 ignored**（379+48+28+12+15）。**PASS**（符合期望 482）。

**8b `lfz test`**（`target\release\lfz.exe test`，exit 0）：
```
PASS_LINE_COUNT = 90
总计：共 90 用例，通过 90，失败 0，错误 0
```
**实际 90/90，exit 0**。⚠️ 任务书期望 89/89 —— **计数偏差 +1**（详见 §⑤ OBS-T11-R1；非功能失败，全部通过）。**结论：PASS（功能），附计数偏差观察。**

**8c `cargo build --all-targets`**（先 `cargo clean -p lfz` 强制重编译）：
```
Compiling lfz v1.0.0 (...)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.89s
WARNING_COUNT = 0
```
**PASS**（强制重编译后 0 warning / 0 error）。

### 条款 9 — 顺带裁定 `bug-20260927-02`（`lfz test --json` 是否单个 JSON）

两种 `--json` 位置均实测（当前同源产物，repo 根目录运行）：
```
lfz test --json      EXIT 0  STDOUT_BYTES 9098  STDOUT_LINES 1  STDERR_BYTES 446  IS_SINGLE_JSON=True
lfz --json test      EXIT 0  STDOUT_BYTES 9098  STDOUT_LINES 1  STDERR_BYTES 446  IS_SINGLE_JSON=True
```
stdout 全文（举例，`test --json`）恰 1 行：
```
{"ok":true,"total":90,"passed":90,"failed":0,"errored":0,"cases":[{"name":"abs_i64_min_overflow",...,"verdict":"PASS"}, ... ]}
```
**裁定：`bug-20260927-02` 不成立（不可复现）→ 关闭。**
- 实测 `lfz test --json`（= P9 使用的 `cargo run --quiet -- test --json`）的 **stdout 恒为单个 JSON（1 行，9098 B，可 JSON 解析）**，程序输出（`check 失败`、`;;` 等）落 stderr（446 B）。
- C 域实测（单行）**正确**；P9 报告记录的“stdout 25 行”对应**更早的产物/修复前状态**，在当前源码同源构建下已不成立。
- 建议 team-lead 据本裁定在 P9 报告/看板标注 `bug-20260927-02` 为 **closed（以当前构建为准）**。

### 条款 10 — 对抗性巡检（reachable panic / abort，新构造）

用 8 个新构造探查 panic/abort（均受控，**0 例崩溃**）：

| 构造 | 命令 | 结果 | 结论 |
| -- | -- | -- | -- |
| `${x:99999999}`（width 超限） | `run t_fmt_width.lfz` | exit 2 · `ValueError: 格式说明符非法：'99999999'` | 受控 |
| `${x:1000001}` | `run t_fmt_width2.lfz` | exit 2 · 同上 | 受控 |
| `${x:1000000}`（=上限） | `run t_fmt_width_ok.lfz` | exit 0 | 上限内正常 |
| `${pi:.70000f}`（precision 超限） | `run t_fmt_prec.lfz` | exit 2 · `ValueError: 格式说明符非法：'.70000f'` | 受控 |
| `${pi:.65536f}` | `run t_fmt_prec2.lfz` | exit 2 · 同上 | 受控 |
| `"ab" * 4611686018427387904` | `run t_strrepeat.lfz` | exit 2 · `OverflowError: 容量溢出：…` | 受控 |
| `repeat(2^62, "")` | `run t_repeat_empty.lfz` | exit **0**（容量 0，不 panic） | 正常 |
| `-`×1001 / `!`×1001 | `run t_unary_1001.lfz` / `t_not_1001.lfz` | exit 2 · `SyntaxError: 嵌套深度超限（超过 1000 层）` | 受控 |
| `floor(1e300)` / `int(1e300)` | `run t_floor_huge.lfz` / `t_int_huge.lfz` | exit 2 · `OverflowError: 整数溢出：结果超出 i64 范围` | 受控 |
| `str()` 遍历 300000 层嵌套数组 | `run t_deep_display.lfz` | exit 0 · 输出 `600002` | 正常 |
| `str()` 遍历 1000000 层嵌套数组 | `run t_deep_disp2.lfz` | exit 0 · 输出 `2000002`（耗时 ~167 s） | 正常（见 OBS-T11-R2） |
| 300000 层深结构 `==` | `run t_deep_eq.lfz` | exit 2 · `RecursionError: 递归深度超限（超过 10000 层）` | 受控 |
| `1 \|> id`×9000 / ×100000 | `run t_pipe_9000.lfz` / `t_pipe_100000.lfz` | exit **0** / exit **2** · `SyntaxError: 表达式嵌套过深（超过 10000 层）` | 边界正确 |
| `take(2^62,[1,2,3])` / `replace` / `split("",…)` | 各自夹具 | exit 0 | 正常 |

**结论：PASS** —— **未发现任何 reachable panic / 进程 abort**（无 `-1073741571` / 无 101 / 无 abort）；无新缺陷单立案。

### 补充（T11-② 第 8 项）— `--help` 文案

```
lfz --help   EXIT 0  （1514 B）
```
- 已**不含**误导性“须以 .lfz 结尾”（全文一刀切）。
- 明示“裸调用 `lfz <file>` 等价 `lfz run <file>`，唯一差别是入口校验”，并说明 `.lfz` 才需 `#42`、非 `.lfz` 豁免（§2.2.0）。
- `--json` 说明“位置不限，以下写法等价”，列出 4 种位置示例。
- `lfz --version` → `lfz 1.0.0`，exit 0。
**结论：PASS。**

---

## ③ 统计

| 维度 | 值 |
| -- | -- |
| 复验条款（含子项） | **37** |
| 补充项（`--help` / `--version`） | 2 |
| **合计 checked** | **39** |
| **PASS** | **39** |
| **FAIL** | **0** |
| 新立案缺陷单 | **0** |
| 观察项（非阻塞） | 2（OBS-T11-R1 计数漂移 / OBS-T11-R2 深显示耗时） |

按 7 类修复归类：**7/7 通过**（`repeat` 容量 · `range` 容量 · 解析嵌套 · AST 深度 · `;;`×`--json` · 越界写 span · §9.2 样例订正）；另 T11-② 第 8 项（`--help`）亦通过。

---

## ④ 缺陷单

**无。** 本轮为修复批次复验，全部条款通过；对抗巡检未发现新的 reachable panic / abort。

（历史缺陷状态见 §⑤ 观察项与 §⑥ 裁定。）

---

## ⑤ 观察项（非阻塞，供 team-lead 收口）

**OBS-T11-R1（低，文档计数漂移）：`lfz test` 实际 90 用例，非任务书/部分文档的 89/87。**
- 实测：`lfz test` → **90/90 exit 0**；`lfz test --json` → `"total":90`。
- 依据：`test-engineer/STATUS.md` 明载「**87→90：+2 负例（新夹具）+1 正例文件**」；即 `parse_nesting_overflow.lfz`、`ast_depth_overflow.lfz` 两个负例 + `tests/lfz/test_parse_limits.lfz` 正例文件。
- 因此任务书“期望 89/89”应为**笔误/陈旧**（漏计 +1 正例文件）。
- 影响：**非功能缺陷**（全部通过）；但 `TEAM_BOARD.md` / `PROJECT_STATE.md`（team-lead）与 `README.md`（release-manager）仍记 85/87，**须在 T11-10 一并刷新**。owner：team-lead（看板/状态）+ release-manager（README）。

**OBS-T11-R2（低，性能）：`str()` 遍历 1,000,000 层嵌套数组耗时 ~167 s（exit 0，不崩溃）。**
- 对照：300000 层 ~11 s，1000000 层 ~167 s（约 15× 数据 ⇒ 约 15× 时间，非明显线性；疑似深链显示/环检测路径存在超线性开销）。
- 属**性能**观察，非崩溃/正确性缺陷；不影响本批复验结论。若需根治，建议架构师/runtime-dev 评估（`value.rs` 显示与 A6 环检测路径）。owner：language-architect / runtime-dev（**仅建议，非本次范围**）。

---

## ⑥ 顺带裁定：`bug-20260927-02`

> **裁定：不成立（不可复现）→ 关闭。**

- **结论**：在当前源码同源构建下，`lfz test --json` 与 `lfz --json test` 的 **stdout 均为单个 JSON**（1 行，9098 B，`ConvertFrom-Json` 可解析），程序输出落 stderr。
- **证据**：§② 条款 9（两种 `--json` 位置均 `STDOUT_LINES=1`，`IS_SINGLE_JSON=True`）。
- **归属**：P9 报告“stdout 25 行”的旧记录**已过时**（对应修复前产物）；C 域实测（单行）正确。建议 team-lead 在 P9 报告与看板将 `bug-20260927-02` 标 **closed（以当前构建为准）**。

---

## ⑦ 总体结论

> **一句话结论：T11 修复批次 —— 通过（7/7 修复条款 + 全量回归 482 单测/90 黑盒全绿 + 0 warning + 无新 panic），并有条件地建议收口 2 项非阻塞文档/性能观察。**

- 7 类修复**全部真实通过**，且逐条附命令与原始输出（见 §②）。
- 关键崩溃路径（`repeat`/`range` 容量、解析嵌套、深左偏 AST）**全部由 panic/abort 转为受控 `OverflowError` / `SyntaxError`，退出码 2**。
- 全量回归：`cargo test` **482/0/0**；`lfz test` **90/90 exit 0**；`cargo build --all-targets` **0 warning**。
- `bug-20260927-02` 裁定为**不可复现 → 关闭**。
- **残留（不阻塞本批结论）**：OBS-T11-R1（用例计数 90 vs 文档 85/87 漂移，由 T11-10 刷新）；OBS-T11-R2（深显示性能，仅建议）。
- **交付判定**：本修复批次 **可交付**；`dist/lfz.exe` 仍为修复前构建，须由 T11-10 用当前源码重建后才能对外。

---

*本报告所有结论均基于 `target\release\lfz.exe`（710144 B / SHA256 `619AFD2E…`）实跑；夹具位于 `Temp/T11/`，收工自清（见 STATE/JOURNAL）。*
