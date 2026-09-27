# P9 — 交付物级独立验收报告（verifier）

> 交付物：P9（独立验收，无独立分值，支撑 8 项提交物） ｜ 作者：verifier ｜ 日期：2026-09-27
> 验收依据：`task-info.md`（课程原始要求）+ `.opencode/team/REQUIREMENTS.md`（需求矩阵/验收清单，v2）+ `docs/spec/`（语法/语义唯一事实源）
> **独立性声明**：本报告**只验证、不修复**。全程**未修改**任何 `src/**`、`docs/spec/**`、`app/**`、`tests/**`、`docs/guide/**`、`benchmarks/**`。
> 环境：Windows 11 / PowerShell 5.1 ｜ rustc 1.98.1 / cargo 1.98.1 ｜ CPython 3.13.9 ｜ 命令均在**项目根**执行（先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`）。

---

## 0. 基线与过程声明（先读）

| 项 | 值 | 说明 |
|---|---|---|
| 验收起始 HEAD | **`c224adb`** | `git rev-parse HEAD`；工作树 **clean** |
| 验收期间 HEAD | **`c224adb`**（未变） | 全过程中 HEAD 未漂移 |
| 验收期间工作树变化 | `M benchmarks/results/raw.json`（被我的基准复跑覆盖）→ **已 `git checkout` 还原**；结束时 `?? docs/slides/`（**并发产出**，非我所为） | 见 §2.4 / §2.8 |
| 解释器基线 | `cargo build` **0 warning**；`cargo test` **431 passed / 0 failed / 0 ignored**；`cargo run -- test` **82/82 exit 0** | 全部本地实测 |
| 标签 | `v0.1.0`→`e060b21`、`v0.2.0`→`931e2b2`、`v0.3-tested`→`f794d6c`、`v0.4-app`→`c224adb` | 远程一致（§2.7） |

> ⚠️ **并发写声明（写给自己与 team-lead）**：本轮开工时 `docs/slides/` **不存在**（`Test-Path` = False）；核验中途出现 `docs/slides/LFZ-defense.pptx`（84 152 bytes，**未提交**，`??`）。故交付物 8 的状态以**核验结束时刻**为准（§2.8）。这印证了既有经验：**验证期间仓库可能被并发写入，必须两次核 HEAD/status**。

---

## 1. 验收清单与结论总表

| 交付物 | 位置 | 结论 | 一句话依据 |
|---|---|---|---|
| 1. 语法规则文档 | `docs/spec/` | ✅ **达标** | 3 件套字节数与 R-401 声明**逐字节一致**；含 EBNF（syntax §7）；文首「唯一事实源（冻结于 2026-09-23）」 |
| 2. 解释器源程序 | `src/` | ✅ **达标** | clean 重建 **0 warning**；431 单测全绿；hello/5 特色/`#42`/错误模型全部实跑通过；`[dependencies]` 空 |
| 3. 完整黑盒测试集 | `tests/` | ✅ **达标** | **一个命令** `cargo run --quiet -- test` → **82 PASS / 0 FAIL / 0 ERROR，exit 0**；覆盖矩阵 + `cases.json` 56 负例 |
| 4. 性能测试报告 | `benchmarks/` + `docs/reports/performance.md` | ✅ **达标** | 报告含方法/环境/数据/结论 + 预热/多轮/中位数；harness 复跑 **All outputs matched: True** |
| 5. 开发指南（人+AI） | `docs/guide/` + `.opencode/skills/lfz-programming/` | ✅ **达标** | 5 篇人类手册 + AI 指南 + 实测记录；3 个示例逐行复现一致；SKILL 含 `#42`×21 与 12 类名，无 `E-xxx` |
| 6. 应用 + 开发记录 | `app/` + `app/DEV_RECORD.md` | ✅ **达标** | `app/sortviz.lfz` **341 行**（LF 计数核实）、首行 `#42`、5 算法、实跑 **exit 0**；DEV_RECORD 含提示词/6 轮迭代/踩坑 |
| 7. Git 历史记录 | `.git/` + 远程 | ✅ **达标** | **65 commits**（`e060b21`→`c224adb`）、4 标签、`origin/main` = 本地 HEAD |
| 8. 系统介绍 PPT | `docs/slides/` | 🟡 **部分** | `LFZ-defense.pptx`（14 页、可解析）**已产出但未提交/未纳入 Git**；§2.8 详述 |

**总表结论**：8 项中 **7 项达标**，**1 项部分**（PPT 实体已产出，但未进 Git，交付完整性未闭环）。5 个评分项（20/20/10/20/30）的交付物**全部存在且经实跑验证可用**。

---

## 2. 逐项核验（位置 → 验证方式 → 实测结果 → 结论）

### 2.1 交付物 1 — `docs/spec/` 语法规则文档

**位置**：`docs/spec/{syntax.md, semantics.md, interface-contract.md}`

**验证方式（可复制）**：
```powershell
Get-ChildItem docs/spec -File | Select-Object Name,Length
Get-Content docs/spec/syntax.md -TotalCount 8
Select-String -Path docs/spec/syntax.md -Pattern 'EBNF'
```

**实测结果（原文）**：
```
Name                  Length
----                  ------
interface-contract.md  30592
semantics.md           33931
syntax.md              62389
```
- R-401 声明字节 `62,389 / 33,931 / 30,592` — **逐字节一致**。
- 文首含「本文件是 LFZ v1 的唯一事实源（冻结于 2026-09-23）。变更须先 ADR，后改文档」。
- `syntax.md` §7 为 **EBNF（ISO EBNF，v0.5 修订版）**（L482「## 7. EBNF（ISO EBNF，v0.5 修订版）」+ L486 `LFZ v0.5 grammar`）。
- `interface-contract.md` §8.1 给出 12 错误类实现映射；§10.7 为 54 内置表。
- 已确认 `semantics.md` §3.7 / §4.1（L51）明确 `s.k ≡ s["k"]` … 并调用（`self` 绑定）——**这是缺陷单 bug-20260927-01 的规范依据**（见 §4）。

**结论**：✅ **达标**。文档完整（词法/EBNF/语义/错误/接口契约），是冻结 v1 唯一事实源，字节数可核。

---

### 2.2 交付物 2 — `src/` 解释器源程序（评分项 1）

**位置**：`src/`（15 文件：`ast/builtins/cli/env/error/evaluator/json/lexer/lib/loader/main/parser/span/test_runner/value`）

**验证方式（可复制）**：
```powershell
cargo clean; cargo build          # 强制全量重建，看 warning
cargo test
cargo run -q -- run examples/hello.lfz
cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz
cargo run -q -- run docs/reports/fixtures-p3/05_pipe.lfz
Get-Content Cargo.toml            # [dependencies] 应为空
```

**实测结果（原文，摘录）**：
```
$ cargo clean; cargo build
     Removed 826 files, 218.2MiB total
   Compiling lfz v0.1.0 (...)
    Finished `dev` profile ... in 3.84s
（无任何 warning 行）
```
```
$ cargo test
running 361 tests ... test result: ok. 361 passed; 0 failed; 0 ignored
running 42 tests  ... test result: ok. 42 passed; 0 failed; 0 ignored
running 16 tests  ... test result: ok. 16 passed; 0 failed; 0 ignored
running 12 tests  ... test result: ok. 12 passed; 0 failed; 0 ignored
（合计 431 passed / 0 failed / 0 ignored）
```
```
$ cargo run -q -- run examples/hello.lfz
Hello, LFZ!
（exit=0）
```
```
$ cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz
1 2 3
99 99
[3, 1, 2]  [1, 2, 3]
{me: <cycle>}
true
（exit=0，5 行，与 R-105 期望逐行一致）
```
```
$ cargo run -q -- run docs/reports/fixtures-p3/05_pipe.lfz
4
3
15
15
5
[0, 2, 4, 6, 8]
[1, 2, 3]
[2, 4]
ABC
（exit=0，9 行，与 R-106 期望一致）
```
`Cargo.toml` 的 `[dependencies]` 段**为空**（零第三方依赖，R-115 达标）。

补充实跑（全部 exit 0）：`02_control.lfz`、`03_functions.lfz`、`04_containers.lfz`、`06_interp.lfz`、`08_builtins.lfz` 均通过，覆盖 R-102/R-104/R-105/R-108/R-109。

**结论**：✅ **达标**。clean 重建 0 warning；431 单测全绿；`hello`、5 特色、容器/控制/函数/插值/内置均可运行。

---

### 2.3 交付物 3 — `tests/` 完整黑盒测试集（评分项 2）

**位置**：`tests/lfz/**/*.lfz`（25 正向用例）+ `tests/fixtures/**`（56 负例夹具）+ `tests/cases.json`（清单）+ `tests/coverage-matrix.md` + `tests/REPORT.md`

**验证方式（可复制）**：
```powershell
cargo run --quiet -- test          # 一个命令跑全部
```
（另核对 `tests/lfz` 首行、清单类名、覆盖矩阵）

**实测结果（原文，末尾）**：
```
...
PASS  tests/lfz/test_reference_semantics.lfz
PASS  tests/lfz/test_structs.lfz

汇总：共 82 个用例，通过 82，失败 0，错误 0
（exit=0）
```
- 全部 25 个 `tests/lfz/*.lfz` 首行均为 `#42`（脚本核对：`non-#42 count = 0`）。
- `tests/cases.json` 共 **57** 条（56 负例 `expect.error` + 1 条非 `.lfz` 正向豁免 `plain_ok.txt`）；类名均取自 12 类。
- 正向 `assert(` 调用计数 = **545**（覆盖矩阵自报 547，差 2；计数口径差异，非实质缺口）。
- `tests/coverage-matrix.md`（425 行）逐特性列出正常/边界/错误用例数，且每特性 **≥3 用例**（正常+边界+错误）。
- `tests/REPORT.md` §5.2：内置 **53/54** 黑盒覆盖（`input` 由 Rust 单测覆盖）；§5.3：错误类 **11/12** 黑盒（`IOError` 由 Rust 单测）。

**结论**：✅ **达标**。满足 R-201/R-202/R-203/R-204：**一个命令跑全部**、覆盖全特性、每特性 ≥3 用例、负例经清单声明、失败可定位。

---

### 2.4 交付物 4 — `benchmarks/` + `docs/reports/performance.md`（评分项 3）

**位置**：`benchmarks/{run_all.py, lfz/*.lfz(6), python/*.py(6), fixtures/noop.*, results/raw.json}` + `docs/reports/performance.md`

**验证方式（可复制）**：
```powershell
cargo build --release
python benchmarks/run_all.py --quick --warmup 1 --runs 3
```

**实测结果（原文，摘录）**：
```
  warmup        : 1
  runs          : 3
Measuring startup baseline (noop) ...
  LFZ    noop median = 10.74 ms
  Python noop median = 39.22 ms

benchmark                N  LFZ med(s)   PY med(s)    ratio   LFZ net    PY net   status
----------------------------------------------------------------------------------------
numeric_loop         10000     0.01122     0.04108     0.27   0.00047   0.00186       ok
function_calls       10000     0.01744     0.04210     0.41   0.00670   0.00288       ok
recursion_fib           20     0.01943     0.04130     0.47   0.00869   0.00208       ok
array_builtins        1000     0.01000     0.04714     0.21   0.00000   0.00792       ok
struct_ops            5000     0.04742     0.04000     1.19   0.03668   0.00078       ok
string_ops            3000     0.05648     0.04088     1.38   0.04574   0.00165       ok

All outputs matched: True
```
- 方法学（`run_all.py` 源码 L80–98）：**预热 `warmup` 轮不计时 + `runs` 轮计时 + 取中位数**，并记录 min/max；**每轮断言 `LFZ stdout == Python stdout`** → 复跑 **All outputs matched: True**。
- 报告 `docs/reports/performance.md`（339 行）含：计时口径、公平对比约束、复现命令、环境、6 基准清单、结果表、结论、瓶颈分析、局限诚实声明。
- 复现的量级与趋势与报告一致（小 N 比值 <1 源于启动优势；`struct_ops`/`string_ops` 比值 >1 且随 N 上升）。

**过程声明**：harness 会覆写 `benchmarks/results/raw.json`，故复跑后工作树出现 `M benchmarks/results/raw.json`。**我已 `git checkout -- benchmarks/results/raw.json` 还原**，工作树恢复 clean（未修改任何交付物）。

**结论**：✅ **达标**。满足 R-301/R-302/R-303。

---

### 2.5 交付物 5 — `docs/guide/` + `.opencode/skills/lfz-programming/`（评分项 4）

**位置**：`docs/guide/{README,tutorial,reference,errors,testing}.md` + `docs/guide/ai/README.md` + `docs/guide/ai/examples/{01,02,03}_*.lfz`；`.opencode/skills/lfz-programming/{SKILL.md, prompt-template.md, VERIFICATION.md}`

**验证方式（可复制）**：
```powershell
Get-ChildItem docs/guide -Recurse -File | Select-Object Name,Length
$env:Path += ";$env:USERPROFILE\.cargo\bin"
cargo run --quiet -- run docs\guide\ai\examples\01_hello.lfz
cargo run --quiet -- run docs\guide\ai\examples\02_basics.lfz
cargo run --quiet -- run docs\guide\ai\examples\03_students.lfz
Select-String -Path .opencode/skills/lfz-programming/SKILL.md -Pattern '#42'   # 计数
```

**实测结果（原文，摘录）**：
```
docs\guide\ai\examples\01_hello.lfz    exit=0   Hello, LFZ!
docs\guide\ai\examples\02_basics.lfz   exit=0
  hello, LFZ / 7 / 2 = 3.5 / div(7, 2) = 3 / -7 % 3 = 2 / round(2.5) = 2 /
  round(3.5) = 4 / grade = high / （;; dump: name ： LFZ / count ： 4 / grade ： high）
docs\guide\ai\examples\03_students.lfz exit=0
  ranking (desc): / Alice:  93 / Cara:  88 / Bob:  67 /
  members: Alice, Bob, Cara / average: 82.67 / top: Alice (93)
```
- 三个示例输出与 `VERIFICATION.md` 记录**逐行一致**；示例首行均为 `#42`。
- `SKILL.md` 含 `#42` **21 处**；**12 个错误类名全部出现**（Cosmos/Syntax/Name/Type/Index/Field/ZeroDivision/Overflow/Value/IO/Assertion/Recursion）；`E-[0-9]` 正则命中 1 处经核实为**误报**（`2.5e-3` 科学计数法，非旧编号错误码）。
- `VERIFICATION.md` 含「SKILL 硬性要求→落点对照」「实测程序+输出+退出码」「错误类实测表」「实测驱动的 6 条修正清单」「54 内置数目核对」「复现命令」。

**结论**：✅ **达标**。满足 R-402/R-403/R-404：人类手册 + 给 AI 的 skill 齐备，且 AI 指南经**全新示例实测**（3/3 复现一致）。

---

### 2.6 交付物 6 — `app/` 应用 + 开发记录（评分项 5，30 分）

**位置**：`app/sortviz.lfz`（+ `app/README.md` + `app/DEV_RECORD.md`）

**验证方式（可复制）**：
```powershell
$env:Path += ";$env:USERPROFILE\.cargo\bin"
cargo run --quiet -- run app/sortviz.lfz
# 行数（LF 计数核实，避免 PowerShell GBK 失真）
$t=[Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes("app\sortviz.lfz")); ([regex]::Matches($t,"`n")).Count
```

**实测结果（原文，摘录）**：
```
exit=0      （stderr = 0 bytes）
```
输出（节选，原文）：
```
APP_NAME ： LFZ 排序算法可视化
N_SMALL ： 8
N_BIG ： 600
SEED ： 42
...
== 阶段 1：小规模逐帧可视化（N=8） ==
原始数据: [13, 91, 58, 64, 50, 62, 25, 8]
数据摘要: min=8  max=91  sum=371  n=8
...
  bubble     比较=    28  交换=    19  步数=    28  [校验通过]
  selection  比较=    28  交换=     5  步数=    28  [校验通过]
  insertion  比较=    25  交换=    19  步数=    25  [校验通过]
  merge      比较=    15  交换=     0  步数=    24  [校验通过]
  quick      比较=    26  交换=     5  步数=    26  [校验通过]
== 阶段 2：大规模仅统计对比（N=600，不输出逐帧） ==
  bubble     比较=179510 ... / selection 179700 / insertion 93640 / merge 4803 / quick 6473  [校验通过]
== 阶段 3：属性自测（随机数组 × 5 算法，check 非致命） ==
  自测 150 次排序，失败 0 次（check 非致命）
== 完成：5 种算法全部通过正确性校验 ==
```
- 首行 `#42`（`Get-ChildItem app -Recurse -Filter *.lfz` 首行核对）。
- 行数：**LF 计数 = 341**（与 DEV_RECORD 自报 341 一致；≫200 达标）。`Measure-Object -Line` 因 PowerShell GBK 解码问题会失真为 302，**我已用 LF 字节计数复核**。
- `app/DEV_RECORD.md`（299 行）含：规模清单、选题理由、需求分析、**6 轮提示词与迭代**、踩坑、回归证据。

**结论**：✅ **达标**。满足 R-501/R-502/R-503：可运行（exit 0）、341 行 ≥200、功能完整（5 算法 + 逐帧可视化 + 统计 + 自校验）、首行 `#42`、开发记录完整。

---

### 2.7 交付物 7 — Git 历史记录

**位置**：`.git/` + 远程 `https://github.com/NeitherTourRest/lfz-programing-language`

**验证方式（可复制）**：
```powershell
git rev-list --count HEAD
git log --oneline --reverse | Select-Object -First 3
git tag
$env:HTTPS_PROXY="http://127.0.0.1:7890"; $env:HTTP_PROXY="http://127.0.0.1:7890"
git ls-remote origin
```

**实测结果（原文，摘录）**：
```
65
e060b21 chore: initial commit — LFZ scaffold, team constitution, frozen v1 language spec
6873fe7 docs(team): record version-management discipline ADR + release-manager status/journal
35e5f62 docs: add repository url and clone instructions
```
```
v0.1.0
v0.2.0
v0.3-tested
v0.4-app
```
```
c224adb...	HEAD
c224adb...	refs/heads/main
fa8d8ae...	refs/tags/v0.1.0
e060b21...	refs/tags/v0.1.0^{}
21fd1d7...	refs/tags/v0.2.0
931e2b2...	refs/tags/v0.2.0^{}
a319a94...	refs/tags/v0.3-tested
f794d6c...	refs/tags/v0.3-tested^{}
ae4169d...	refs/tags/v0.4-app
c224adb...	refs/tags/v0.4-app^{}
```
- **65 commits**，跨度 2026-09-23 → 2026-09-27，覆盖脚手架/P2 冻结/P3 实现/P4 工具/P5 测试/P6 性能/P7 文档/P8 应用。
- 远程 `main` = 本地 HEAD = `c224adb`；4 个标签全部已推送。

**结论**：✅ **达标**。满足 R-603/R-610：Git 历史存在、覆盖主要阶段、远程可达且一致。

---

### 2.8 交付物 8 — `docs/slides/` 系统介绍 PPT

**位置**：`docs/slides/LFZ-defense.pptx`

**验证方式（可复制）**：
```powershell
Test-Path docs/slides
Get-ChildItem docs/slides -Recurse -Force | Select-Object FullName,Length
# 解析 pptx（zip）内的 slide 数与文字
$zip=[IO.Compression.ZipFile]::OpenRead((Resolve-Path docs\slides\LFZ-defense.pptx))
$zip.Entries | Where-Object { $_.FullName -match '^ppt/slides/slide\d+\.xml$' }
```

**实测结果（原文，摘录）**：
```
开工时：Test-Path docs/slides → False
核验结束时：docs\slides\LFZ-defense.pptx   84152 bytes   （git status: ?? docs/slides/）
zip magic: 50 4b 03 04  （合法 OOXML/ZIP）
slide count = 14
slide1  : #42 | LFZ 系统介绍 PPT | 一句话定位… | 系统介绍答辩 ｜ 第 8 组 | 14 名 opencode AI 开发团队协作开发 | 2026-09-27 ｜ 版本 v0.4-app ｜ Rust 实现，仅 std ｜ github…
slide2  : 评分矩阵 ｜ 总览对照表 … 1 解释器实现 20 … 2 自动测试 20 … 3 性能 10 …
slide4/5: 语法设计（5 特色 + #42）/ 解释器架构（Rust 仅 std）
slide7  : 测试验证（431 单测 / 82 黑盒 / 0 告警 / 10/10 缺陷闭环）
slide8/9: 性能（LFZ vs Python 实测）/ 性能债与优化路线
slide10 : 评分项 4 开发指南 / slide11 : 评分项 5 Agent 应用 sortviz（341 行）
slide12 : 团队与工程纪律 / slide13 : 成果与展望 / slide14 : 结尾致谢
```
- PPT **实体可解析**（14 页），内容覆盖 5 个评分项、语法、架构、测试、性能、应用、团队、致谢——**内容覆盖充分**。
- ⚠️ **但该文件为未跟踪（`??`）状态，未 `git add`/commit**；且它是在我核验**期间并发产出**（开工时不存在）。

**结论**：🟡 **部分**。**实体已产出且可解析（14 页，内容覆盖评分点）**，但**未纳入 Git 版本历史**，交付完整性未闭环。此前 PROJECT_STATE/REQUIREMENTS 记录的「PPT 待 P10」状态在核验结束时刻已变为「已产出待提交」。

> 说明：本项**不评美观**，只验存在性与完整性；「能正常打开」以 zip 结构 + slide XML 可解析为证据。

---

## 3. 关键命令独立复现（原文粘贴汇总）

> 全部在项目根、`$env:Path += ";$env:USERPROFILE\.cargo\bin"` 后执行。退出码用 `$LASTEXITCODE`。

| # | 命令 | 期望 | 实测（原文） | 判定 |
|---|---|---|---|---|
| 1 | `cargo clean; cargo build` | 0 warning | `Compiling lfz v0.1.0` → `Finished dev profile ... in 3.84s`（无 warning） | ✅ |
| 2 | `cargo test` | 全绿 | `361 + 42 + 16 + 12 = 431 passed; 0 failed; 0 ignored` | ✅ |
| 3 | `cargo run -q -- test` | 82/82 exit 0 | `汇总：共 82 个用例，通过 82，失败 0，错误 0`；exit 0 | ✅ |
| 4 | `cargo run -q -- run examples/hello.lfz` | `Hello, LFZ!` exit 0 | `Hello, LFZ!`；exit 0 | ✅ |
| 5 | `cargo run -q -- run app/sortviz.lfz` | exit 0 | 三阶段输出、5 算法全部 `[校验通过]`；exit 0；stderr 0 bytes | ✅ |
| 6 | 缺 `#42` 负例（`print(1)` 存为 `.lfz`） | exit 2 + `CosmosAnswerError` | stderr 前 2 行：`File "...\nohdr.lfz", line 1` + `CosmosAnswerError: 你忘记了宇宙的答案`；**exit 2**；**UTF-8 字节逐字符匹配**（hex 比对 True） | ✅ |
| 7 | `--json`（`1 / 0`） | stdout 单行 JSON | stdout（263 bytes）= `{"ok":false,"error":"ZeroDivisionError","message":"除以零","file":"...","line":2,"col":7,"traceback":[...]}`；**1 行**、不含 `E-`；stderr 0 bytes；exit 2 | ✅ |

**补充边界/错误路径抽查（不只 happy path）**：

| 场景 | 命令 | 实测 | 判定 |
|---|---|---|---|
| 文件不存在 | `run <does_not_exist.lfz>` | `IOError: 无法读取：...`；exit 2 | ✅ 报错不崩溃 |
| 空 `.lfz` 文件 | `run empty.lfz`（0 字节） | `File "...", line 1` + `CosmosAnswerError`；exit 2 | ✅ |
| 语法错误 | `run syntax.lfz`（`let x =`） | `File "...", line 2` + 源行 + `^` 插入符 + `SyntaxError: 表达式未结束…`；exit 2 | ✅ |

**字节级证据（关键命令 6）**：
```
STDERR hex tail : 436f736d6f73416e737765724572726f723a20e4bda0e5bf98e8aeb0e4ba86e5ae87e5ae99e79a84e7ad94e6a188
EXPECTED   tail : 436f736d6f73416e737765724572726f723a20e4bda0e5bf98e8aeb0e4ba86e5ae87e5ae99e79a84e7ad94e6a188
stderr contains expected tail? True
```

---

## 4. 评分矩阵抽查（20 / 20 / 10 / 20 / 30）

> 原则：**只依据实测证据**给「当前能拿多少分」的判断依据，不拍脑袋。

| 评分项 | 硬性通过条件（REQUIREMENTS §2） | 实测是否满足 | 证据 | 判断依据 |
|---|---|---|---|---|
| 1. 解释器 20 | hello 输出正确；`cargo test` 全绿；5 特色可运行；`#42`+错误模型生效 | **全部满足** | §2.2、§3 命令 1/2/4/6/7；431 全绿；5 特色 fixture exit 0 | 证据层面**支持满分档**；唯一减分风险 = 缺陷单 bug-20260927-01（struct 方括号取方法再调用的 `self` 绑定，见 §5），建议答辩前修复 |
| 2. 自动测试 20 | 一个命令跑全部；覆盖全部功能；每特性 ≥3 用例；失败可定位 | **全部满足** | §2.3；82/82 exit 0；覆盖矩阵每特性 ≥3；56 负例经清单；runner 失败可定位由 `test_runner` 单测覆盖 | 证据层面**支持满分档** |
| 3. 性能 10 | 报告交付；LFZ vs Python 公平对比（预热/多轮/中位数）；报告可复现 | **全部满足** | §2.4；报告四要素齐；`run_all.py` 实执行预热+多轮+中位数；复跑 output 全等 | 证据层面**支持满分档** |
| 4. 语法+人/AI 指南 20 | 语法文档冻结 + 人类手册 + AI 指南/skill 齐备且 AI 指南经实测 | **全部满足** | §2.1、§2.5；spec 三件套字节可核；5 手册 + skill；3 示例实测复现 | 证据层面**支持满分档** |
| 5. Agent 应用 30 | 应用可运行、≥200 行、功能完整、含开发记录 | **全部满足** | §2.6；exit 0；341 行；5 算法+可视化+统计+自校验；DEV_RECORD 6 轮迭代 | 证据层面**支持满分档** |

**评分项是否达标的证据强度**：5 个评分项的**硬性条件均被独立实跑证据覆盖**。评分项 1 存在 1 处「规范已写、实现未达」的中等缺陷（§5 bug-01），其余 4 项未发现反证。

---

## 5. 缺陷单

### 【缺陷单】bug-20260927-01（**复现确认**）
- **交付物**：`src/`（结构体方法经方括号取值后的 `self` 绑定）｜ 影响：R-107（评分项 1）
- **最小复现步骤**：
  1. 新建 `bug_struct.lfz`：
     ```lfz
     #42
     struct Point {
         x: 0,
         y: 0,
         fn norm2() => self.x * self.x + self.y * self.y,
     }
     let p = Point { x: 3, y: 4 }
     print(p.norm2())
     print(p["norm2"]())
     ```
  2. `cargo run --quiet -- run bug_struct.lfz`
- **期望**（`docs/spec/semantics.md` L51）：`s.k ≡ s["k"]`，两者**都能取到该函数值并调用（`self` 绑定）** → 两行都输出 `25`，exit 0。
- **实际**：
  ```
  stdout: 25            ← p.norm2() 正常
  stderr:
  Traceback (most recent call last):
    File "...\bug_struct.lfz", line 9, in <module>
      print(p["norm2"]())
            ^
    File "...\bug_struct.lfz", line 5, in norm2
          fn norm2() => self.x * self.x + self.y * self.y,
                        ^
  NameError: 未定义的名字 'self'
  exit=2
  ```
  → `p["norm2"]()` 取到方法值后调用时**未绑定 `self`**，抛 `NameError`。
- **严重度**：**中**（spec 明文要求的等价性在「方括号取方法再调用」路径上失败；`.字段()` 调用路径正常；黑盒用例已刻意不对其断言、不删除不放宽）。
- **建议 owner**：**runtime-dev**（evaluator 的方法调用绑定逻辑，需覆盖 `Index`/`Field` 两种取方法路径）；core-dev 会签。修复后由 verifier 复验并回归。

### 【缺陷单】bug-20260927-02（低 / 非阻塞，已知工具链间隙）
- **交付物**：`src/test_runner.rs` + `src/cli.rs`（`lfz test --json` 的 stdout 纯度）
- **最小复现**：`cargo run --quiet -- test --json`
- **期望**（R-112 口径「`--json` 下 stdout 恒为单行 JSON」）：stdout 仅 1 行 JSON。
- **实际**：stdout **25 行**（因 `;;` dump 文本仍写 stdout），**最后一行**才是合法 JSON：
  ```
  {"ok":true,"total":82,"passed":82,"failed":0,"errored":0,"cases":[...]}
  ```
  （`run --json` 单行已满足；本缺陷仅限 `test --json`；`tests/REPORT.md` §3.1 已**自认**为已知工具链偏差。）
- **严重度**：**低**（JSON 恒在最后一行且可解析；主交付命令 `cargo run --quiet -- test` 不受影响）。
- **建议 owner**：tooling-dev（裁定 `;;` 在 `--json` 模式是否重定向到 stderr）。

### 【观察项】其他非缺陷但需处理
| 编号 | 事项 | 证据 | 建议 owner |
|---|---|---|---|
| obs-01 | 根 `README.md` **内容陈旧**：仍写「P4/P5/性能待启动」与「`cargo test` → 377 passed」 | `Select-String README.md` 命中「下一步 P4 工具链」「377 passed」 | release-manager（发 v1.0-final 时更新） |
| obs-02 | `docs/slides/LFZ-defense.pptx` **未纳入 Git**（`??`） | `git status --porcelain` → `?? docs/slides/` | release-manager / ppt-presenter |
| obs-03 | `Cargo.toml` `version = "0.1.0"` 未随标签升至 `0.4`/`1.0` | `Get-Content Cargo.toml` | release-manager（可选） |
| obs-04 | 覆盖矩阵自报 547 `assert`，脚本计数 **545**（差 2，口径差异） | §2.3 | test-engineer（可选核对） |
| obs-05 | 语言命名「LFZ」是否即课程要求的「以本人名字命名」需人工确认 | task-info §1 | team-lead / 学生 |

---

## 6. 回归记录

> 本轮为 P9 交付物级验收；回归对象 = P3 已闭环缺陷 + 本轮发现缺陷。

| 缺陷单 | 来源 | 复验命令 | 结果 | 状态 |
|---|---|---|---|---|
| P3 的 10 项缺陷（3🔴+4🟡+2🟢+1 规范侧） | `docs/reports/P3-verification.md`（rev.3 PASS） | `cargo test`（431 全绿）+ P3 夹具全跑 | 全部仍通过、无回归 | ✅ 已修复（保持） |
| bug-20260927-01 | 本轮 P9 | §5 复现步骤 | **仍可复现**（`NameError`） | 🔴 打开 |
| bug-20260927-02 | 本轮 P9 | `cargo run --quiet -- test --json` | **仍可复现**（stdout 25 行） | 🟡 打开（低） |

---

## 7. 总体结论

### 7.1 结论行

> ## 结论：**CONCERNS（有条件交付）** — 不建议直接打 `v1.0-final`，需先闭合 2 项非阻塞但应处理的事项（PPT 入库 + 中等缺陷裁定）。

**理由（证据链）**：
- ✅ **无阻塞项**：8 项交付物**实体齐备**（PPT 亦已产出），5 个评分项的硬性条件**全部经实跑证据覆盖**；`cargo build` 0 warning、`cargo test` 431 全绿、`cargo run -- test` 82/82 exit 0、`hello`/应用/负例/`--json` 全部符合预期。
- 🟡 **非阻塞顾虑（CONCERNS）**：
  1. **bug-20260927-01**（中）：`s["method"]()` 未绑定 `self`，与 `semantics.md` §4.5(L51) 明文 `s.k ≡ s["k"]` 的等价性冲突 —— **规范已写、实现未达**，属评分项 1 的实质减分点。
  2. **交付物 8 未入库**：`docs/slides/LFZ-defense.pptx`（14 页）已产出，但**未提交进 Git**，交付完整性未闭环（且为核验期间并发产出）。
  3. **`test --json` stdout 非单行**（低，已知并自认）。
  4. **README 陈旧**（377/下一步 P4），发版前应更新。

### 7.2 「距离满分还缺什么」诚实清单

**评分项 1（解释器，20）**：
- [ ] 修复 **bug-20260927-01**：让 `["method"]` 取到的方法值调用时同样绑定 `self`（spec §4.5 L51 / §3.7 要求 `s.k ≡ s["k"]`）。
- [ ] 修复后补一条黑盒端到端用例（现 `test_structs.lfz` 刻意只断言「可取到 function 值」）。

**评分项 2/3/4/5（各 20/10/20/30）**：
- [ ] 这 4 项**未发现反证**，硬性条件已满足；若答辩要求「更足」，可选增强：`--json` 下 `;;` 通道裁定（bug-02）、内置 `input` 的黑盒覆盖（现由 Rust 单测覆盖）。

**支撑（交付完整性 / Git / 答辩）**：
- [ ] 将 `docs/slides/LFZ-defense.pptx` **`git add` + commit**（消除 `??`）。
- [ ] 更新根 `README.md`（431 测试、P4–P8 已完成、v0.4-app 基线、交付物索引全绿）。
- [ ] （可选）`Cargo.toml` 版本对齐标签；人工确认语言命名符合课程「以本人名字命名」要求。
- [ ] 打 `v1.0-final` 前，以**本报告 §0 记录的发版基线**为准复核 HEAD 与工作树 clean。

**结论落地建议**：先由 runtime-dev 修 bug-20260927-01、release-manager 提交 PPT 并刷新 README；verifier 复验上述两项后，可将结论升级为 **PASS（可打 `v1.0-final`）**。

---

## 8. 方法学与环境差异说明

- **证据纪律**：每条结论均附可复制的命令与原始输出；判断「能运行」均**实际运行**；重启一律用 `cmd /c` 重定向取原始 stdout/stderr 字节，避免 PowerShell 5.1 的编码失真。
- **中文消息判定**：Windows 控制台按本地代码页重编码中文，**一切中文消息以 UTF-8 字节解码为准**（§3 hex 比对即为此）。
- **计数口径**：`app/sortviz.lfz` 行数以 **LF 字节计数 = 341** 为准（`Get-Content | Measure-Object -Line` 在本机 GBK 解码下失真为 302/327，不作为判据）。
- **环境**：Windows 11 家庭中文版；rustc/cargo 1.98.1；CPython 3.13.9（Anaconda）；网络经代理 `127.0.0.1:7890` 访问远程。
- **未修改声明**：全程仅写入 `docs/reports/P9-verification.md`；对 `benchmarks/results/raw.json` 的**意外覆盖已还原**（`git checkout`）。

---

> 报告完。证据原文均取自本轮实测转录；如需重跑，按 §3 表格命令逐条执行即可。

---

# 复验（rev.2）— 2026-09-27

> **复验范围**：对 P9 报告的 2 项待闭项做**独立复验**并给出升级结论。**只验证、不修复**；本轮**未修改**任何 `src/**`、`docs/spec/**`、`app/**`、`tests/**`、`docs/guide/**`、`benchmarks/**`、`README.md`（仅追加本报告）。
> 环境同 §0：Windows 11 / PowerShell 5.1 ｜ rustc/cargo 1.98.1 ｜ CPython 3.13.9 ｜ 命令均在**项目根**执行（先 `$env:Path += ";$env:USERPROFILE\.cargo\bin"`）。

## 9.0 复验基线（先读：HEAD 发生了并发漂移）

| 项 | 值 | 说明 |
|---|---|---|
| 复验开工 HEAD | `4727726`（`docs(p10): refresh README…`） | 开工时工作树含 **未提交** 的 `M src/evaluator.rs` |
| 复验结束 HEAD | **`6aabdf5`**（`fix: bind self for methods retrieved via ["k"]`） | 我核验期间该修复被**并发提交**；工作树现 **clean** |
| 基线锁定 | 已验证在 **HEAD `6aabdf5`** 上：`cargo test` = 432 passed；bug-01 复现用例 exit 0 | 我首次 `cargo clean; cargo build` 时文件内容**与提交后一致**（`git status` 现为空），故结论对 `6aabdf5` 成立 |
| 远程一致性 | `origin/main` = 本地 HEAD = `6aabdf5`；`git rev-list --count HEAD` = **68** | 4 标签仍在 |

> ⚠️ **并发写第二次印证**：我在核验 `M src/evaluator.rs` 期间，runtime-dev/release-manager 将其提交为 `6aabdf5`。**报告结论以结束时刻 `6aabdf5` 为准**，并对提交前后内容一致性做了核对（见上「基线锁定」）。

---

## 9.1 待闭项 1 — `bug-20260927-01`（`s["method"]()` 未绑定 `self`）

- **原待闭项**：P9 §5 复现确认：`p["norm2"]()` 取到方法值后调用**未绑定 `self`** → `NameError: 未定义的名字 'self'`、exit 2；而 `p.norm2()` 正常。与 `semantics.md` §4.5(L51) 明文 `s.k ≡ s["k"]`（调用时 `self` 绑定）冲突。严重度 **中**，owner = runtime-dev。

- **修复声明**：`6aabdf5 fix: bind self for methods retrieved via ["k"] (spec s.k === s["k"])`（commit message 原文："a method value obtained through the bracket form now binds self when called…Adds a regression test."）。

- **复验命令（我独立构造，未经他人提供）**：
  ```powershell
  $env:Path += ";$env:USERPROFILE\.cargo\bin"
  # 复现夹具（一次性，置于临时目录，不进入交付物）：
  #   C:\Users\19170\AppData\Local\Temp\opencode\verifier_rev2_bug01.lfz        （assert 版）
  #   C:\Users\19170\AppData\Local\Temp\opencode\verifier_rev2_bug01_values.lfz （显式打印值版）
  cargo run -q -- run C:\Users\19170\AppData\Local\Temp\opencode\verifier_rev2_bug01.lfz
  cargo run -q -- run C:\Users\19170\AppData\Local\Temp\opencode\verifier_rev2_bug01_values.lfz
  ```

  夹具（assert 版，含「经方括号调用方法应写自身字段」的**副作用**证明，而不只是取值相等）：
  ```lfz
  #42
  struct P {
      v: 41,
      n: 0,
      fn get() => self.v + 1,
      fn bump() => {
          self.n = self.n + 1
          self.n
      },
  }
  let p = P { v: 41 }
  assert(p.get() == 42, "dot")
  assert(p["get"]() == 42, "bracket")
  assert(p.n == 0, "n0")
  let b1 = p["bump"]()
  let b2 = p.bump()
  assert(b1 == 1, "b1")
  assert(b2 == 2, "b2")
  assert(p.n == 2, "n2")
  print("bug01 bracket-call OK")
  ```

- **实测（原文）**：
  ```
  $ cargo run -q -- run ...\verifier_rev2_bug01.lfz
  bug01 bracket-call OK
  （ASSERT_EXIT=0，7 条 assert 全过）

  $ cargo run -q -- run ...\verifier_rev2_bug01_values.lfz
  42        ← p.get()
  42        ← p["get"]()       （与上一行相等 → 等价）
  1         ← p.bump()         （self.n: 0→1）
  2         ← p["bump"]()      （self.n: 1→2，证明 self 确为同一接收者）
  2         ← p.n              （副作用落在 p 上）
  （EXIT=0）
  ```
  → 不仅 `p["get"]() == p.get() == 42`（**取值等价**），且经方括号取到的方法调用 `bump()` 能**写回原接收者** `p.n`（**`self` 绑定为同一对象**），两种取法共享同一 `self` 状态（1→2）。

- **修复前证据（无法回退时引用）**：P9 §5 原始复现 —— `stdout: 25`（仅 `p.norm2()` 行）、`stderr: … NameError: 未定义的名字 'self'`、`exit=2`。该失败与规范 `semantics.md` §4.5(L51) 冲突；本轮同一路径的**修复后**复用同型代码全部通过。

- **修复因果核对**：`git show 6aabdf5 -- src/evaluator.rs` 显示新增 `ExprKind::Index { object, index }` 被调分支：当被调对象为 `Value::Struct` 且下标为字符串键时，取出方法字段后以 `Some(recv)` 作为 `self` 传入 `call_func`；同时新增回归单测 `struct_method_via_bracket_index_call_binds_self`（即 `cargo test` 由 431→432 的 +1 来源）。

- **结论**：✅ **已闭合**。`s.k() ≡ s["k"]()` 在取值与副作用（`self` 写回）两个维度均成立。

---

## 9.2 待闭项 2 — `obs-01`（根 `README.md` 陈旧）

- **原待闭项**：P9 §5 obs-01：根 `README.md` 陈旧（命中「377 passed」「下一步 P4」）。owner = release-manager。
- **修复声明**：`4727726 docs(p10): refresh README and add delivery checklist`。

- **复验命令**：
  ```powershell
  Select-String -Path README.md -Pattern '431|361|432|362|377|个用例|341|v0\.4-app|LFZ-defense'
  Get-ChildItem docs/slides -Force                       # 交付物 8 是否入库
  git ls-files docs/slides
  ```

- **实测（逐条核对 README 与实测值）**：

  | README 声明 | 实测值 | 一致？ |
  |---|---|---|
  | `cargo test` **431 passed（lib 361 + main 42 + cli 16 + test_runner 12）**（L30/L86） | **432 passed（lib 362 + main 42 + cli 16 + test_runner 12）** | ❌ **不一致（+1）** |
  | `cargo run -- test` 82 个用例 | 82 PASS / 0 FAIL / 0 ERROR，exit 0 | ✅ |
  | `app/sortviz.lfz` 341 行 | LF 计数 = **341** | ✅ |
  | 4 个标签 | `v0.1.0`/`v0.2.0`/`v0.3-tested`/`v0.4-app` | ✅ |
  | 8 项交付物索引含真实路径 | 逐项 `Test-Path`/`Get-ChildItem` 全部存在（`docs/spec/`、`src/` 15 文件、`tests/`、`benchmarks/`+`performance.md`、`docs/guide/`+`ai/`+skill、`app/`+`DEV_RECORD.md`、`.git/`、`docs/slides/`） | ✅ |
  | 「377 passed / 下一步 P4」等旧值 | **已消失**（不再命中） | ✅ 已刷新 |
  | 交付物 8 `docs/slides/`（14 页 + `demo-script.md` + `qa-prep.md`） | 三文件**均已 Git 跟踪**（`git ls-files` 命中）；pptx slide count = **14** | ✅（**obs-02 亦闭合**） |

- **结论**：🟡 **未完全闭合（残留 1 处数值）**。README 主体已刷新到 P10（阶段表、8 项交付物索引含真实路径、4 标签、82 用例、341 行、`docs/slides/` 均正确），且连带闭合了 **obs-02**（PPT 入库）；**但测试数仍是修复前的旧值**：README L30 / L86 写 `431 passed（lib 361）`，而当前 HEAD `6aabdf5` 实测为 **`432 passed（lib 362）`**（修复提交 `6aabdf5` 新增 1 条 lib 回归单测所致）。发 `v1.0-final` 前应由 release-manager 将 L30/L86 改为 **432（lib 362）**。严重度 **低**（仅文档数值，不影响任何实跑）。

  > 归因说明：README 刷新提交（`4727726`）早于 bug-01 修复提交（`6aabdf5`），故测试数在修复落地后自然失效——属**顺序性残留**，非 README 作者失误。

---

## 9.3 关键基线复核（本轮实测，HEAD `6aabdf5`）

| # | 命令 | 期望 | 实测（原文） | 判定 |
|---|---|---|---|---|
| 1 | `cargo clean; cargo build` | 0 warning | `Removed 1578 files, 338.1MiB total` → `Compiling lfz v0.1.0` → `Finished dev profile … in 3.68s`（**无 warning 行**）；exit 0 | ✅ |
| 2 | `cargo test` | 库侧 **362 passed** | `362 passed; 0 failed; 0 ignored`（lib）+ `42` + `16` + `12` = **432 passed / 0 failed / 0 ignored**；exit 0 | ✅ |
| 3 | `cargo run -- test` | 82/82 exit 0 | `PASS` 行 **82**、`FAIL/ERROR` 行 **0**；`汇总：共 82 个用例，通过 82，失败 0，错误 0`；exit 0 | ✅ |
| 4 | `cargo run -- run app/sortviz.lfz` | exit 0 | exit **0**（stdout 4170 bytes / 79 行）；阶段 1+2 共 **10 处 `[校验通过]`**；末行 `== 完成：5 种算法全部通过正确性校验 ==` | ✅ |

- Git：`git rev-list --count HEAD` = **68**；`origin/main` = 本地 HEAD = `6aabdf5`；工作树 clean。

---

## 9.4 额外检查（并记录）— 黑盒测试集是否覆盖 `s["k"]()` 调用形态？

**结论：❌ 未覆盖 → 记为建议项（owner: test-engineer）。**

- 证据：`tests/lfz/test_structs.lfz` L26–L30 —— **只断言可取到函数值，不调用**：
  ```
  L26: // `p["norm2"]` 能取到方法函数值（spec semantics §3.7：s.k ≡ s["k"] 可取到该函数值）。
  L28: // （spec 要求 s.k ≡ s["k"] 且调用时 self 绑定；实现仅在 `.字段()` 调用点绑定）。
  L29: // 缺陷修复前此处只断言「可取到 function 值」，不调用（避免把缺陷行为固化为期望）。
  L30: assert(type(p["norm2"]) == "function", "struct_method_value_via_bracket")
  ```
  对 `p["k"]()` 这一**调用**形态**无任何断言**（对比 L25 `assert(p.norm2() == 25, "struct_method_call_dot")` 有 `.字段()` 调用断言）。
- 说明：该缺口在修复前是**刻意**的（避免固缺陷），但**修复（`6aabdf5`）落地后此路径已可正确执行**，**值得补一条黑盒端到端用例** `assert(p["norm2"]() == 25, …)`，将 §4.5 等价性纳入自动回归；且 L28–L29 注释所述「实现仅在 `.字段()` 调用点绑定」在修复后已**过时**，宜同步更新。
- **性质**：建议项（非阻塞；不影响评分项硬性条件——解释器侧已由 `cargo test` 新增单测覆盖）。

---

## 9.5 复验结论行

> ## 【复验结论】**CONCERNS** — 2 项待闭项中 `bug-20260927-01` **已闭合**、`obs-02` 连带闭合；`obs-01`（README）**未完全闭合**（残留 2 处数值：L30/L86 的 `431 passed（lib 361）` 应为 `432 passed（lib 362）`）。**无阻塞项**；打 `v1.0-final` 前由 release-manager 修上述 2 处数值即可升级为 **PASS**。

**逐项结论**：

| 待闭项 | 结论 | 证据 | 备注 |
|---|---|---|---|
| `bug-20260927-01`（`s["k"]()` 未绑定 `self`） | ✅ **已闭合** | §9.1：独立复现 exit 0；`p["get"]()==42`、`p["bump"]()` 写回 `self.n`（0→1→2） | 修复 `6aabdf5` 含回归单测；`cargo test` 431→432 |
| `obs-01`（README 陈旧） | 🟡 **未完全闭合** | §9.2：主体已刷新（P10/8 交付物/4 标签/82 用例/341 行），**残留 L30/L86 测试数** | 435→432 的 2 处数值；低，仅文档 |
| `obs-02`（PPT 未入库，连带复核） | ✅ **已闭合** | §9.2：`git ls-files docs/slides` 命中 3 文件；14 页 | 工作树 clean |
| 黑盒覆盖 `s["k"]()`（额外检查） | ⚪ **建议项** | §9.4：`test_structs.lfz` 仅断言 `type(...)=="function"` | owner = test-engineer |
| `bug-20260927-02`（`test --json` stdout 非单行，低） | ⚪ **仍打开（非本轮待闭项）** | P9 §5；低、已知并自认 | 非阻塞，可后置 |

**关键基线（HEAD `6aabdf5`）**：`cargo build` **0 warning** ✅ ｜ `cargo test` **432 passed（lib 362）** ✅ ｜ `cargo run -- test` **82/82 exit 0** ✅ ｜ `cargo run -- run app/sortviz.lfz` **exit 0** ✅。

**升级路径（唯一条件）**：release-manager 将 `README.md` L30 / L86 的 `431（lib 361）` 改为 `432（lib 362）` → 本报告 obs-01 即闭合 → 整体可升级为 **`PASS（可打 v1.0-final）`**。（建议项与 bug-02 不阻塞打标。）

> 复验完。本轮复验脚本均为一次性、置于 `C:\Users\19170\AppData\Local\Temp\opencode\`，未进入任何交付物目录；未修改任何交付物。
