# 项目全局状态
> 最后更新: 2026-09-27 by team-lead

本文件为活文档，唯一写者 team-lead。其他角色只读。

## 1. 一句话目标
设计并实现 LFZ 解释型脚本语言及其解释器，配套黑盒测试、性能对比、人/AI 开发指南与 Agent 开发应用，完成 8 项交付物并通过线下验收。

## 2. 当前阶段
**P0–P10 全部完成（8 项交付物齐备，tag `v1.2.0`）→ 现进入 T11「先核对、后迭代」阶段。**

- **T11 第一步（已完成，结论：不完全无误）**：用户 2026-09-27 指令「**先确定现有计划完全实现无误，然后开始迭代**」。3 路 verifier 并行完成**规范符合性审计**（spec 条款 ↔ 实现行为，**逐条实跑 witness**、只验证不修复）：
  - **A 词法/文法/解析/5 特色**（79 条款：**76 PASS / 1 FAIL (spec 侧) / 2 N-A**）→ `docs/reports/conformance-A-syntax.md`；**解释器侧 FAIL = 0**
  - **B 求值语义/错误模型**（28 条款：**27 PASS / 1 FAIL**）→ `docs/reports/conformance-B-semantics.md`
  - **C 内置 54/loader/runner 契约/CLI**（50 条款：**46 PASS / 3 FAIL**）→ `docs/reports/conformance-C-builtins-cli.md`
  - **合计 157 条款 → 149 PASS / 5 FAIL（= 4 个独立缺陷）**。**3 个真实实现缺陷**：🔴 `bug-20260927-03`（`repeat` 溢出 → **Rust panic 崩溃**，高）、🟡 `bug-20260927-04`（`range` 超大 n → **Rust panic**，规范静默，中）、🟡 `bug-B-20260927-01`（`;;` × `--json` 使 stdout 非单个 JSON，中，B/C 双域独立复现）；另 **1 个 spec 侧缺陷** `BUG-A-01`（§9.2 样例 B 期望输出 与 §10.7 键序/排序稳定性自相矛盾——**解释器输出才是对的**）。
  - ⚠️ **这 3 个缺陷 445 单测与 85 黑盒都没抓到** → 「绿了 ≠ 对」再次被证实；缺陷台账见 `TEAM_BOARD.md`。
- **T11 前置（已完成）**：`FEATURE-AUDIT.md`（特性缺口审计，39238 B）+ skill **盲测 2/8**（8 题零上下文 agent，隔离工作区）。
- **T11 第二步（待办）**：依审计结论进入 **v1.1 迭代** = 7 条 P0 文档修订（D1–D7）+ 7 项语言补强（见 `TEAM_BOARD.md` T11-04/05/06）。
- **交付现状**：8 项交付物**全部齐备**（详见表 4）；质量基线：`cargo build` 0 warning、`cargo test` **445/0/0**、`lfz test` **85/85 exit 0**、应用 `app/sortviz.lfz` 341 行 exit 0。

## 3. 里程碑进度表
| 阶段 | 里程碑 | 状态 | 负责人 |
| -- | -- | -- | -- |
| P0 | 团队就绪（脚手架通过 S1–S5） | ✅ 已完成 | team-lead |
| P0.5 | 版本基线（git init + 初始提交 + `v0.1.0` + GitHub 远程） | ✅ 已完成 | release-manager |
| P1 | 需求矩阵+验收标准 | ✅ 已完成 | requirements-analyst |
| P2 | 语言设计（语法+语义+接口契约+ADR） | ✅ 已完成（`docs/spec/` 冻结，D-016） | language-architect |
| P3 | 核心实现（lexer/parser/AST/eval/builtins，TDD） | ✅ 已完成（P3.11 终验 PASS） | core-dev + runtime-dev |
| P4 | 工具链（CLI/一键测试 runner/打包） | ✅ 已完成 | tooling-dev |
| P5 | 黑盒测试（全量测试集+覆盖矩阵） | ✅ 已完成（85/85） | test-engineer |
| P6 | 性能（LFZ vs Python 基准+报告） | ✅ 已完成 | perf-engineer |
| P7 | 文档（人类手册 + AI 指南/skill） | ✅ 已完成 | docs-writer + ai-dx-engineer |
| P8 | 应用（≥200 行 LFZ 应用 + 开发记录） | ✅ 已完成（341 行） | app-dev |
| P9 | 验证（独立验收报告） | ✅ 已完成（CONCERNS→遗留项闭环） | verifier |
| P10 | 发布+答辩（git 历史+交付清单+PPT） | ✅ 已完成 | release-manager + ppt-presenter |
| **T11** | **核对现有实现（规范符合性审计）→ v1.1 迭代** | **🔄 进行中（3 审计并行）** | verifier → language-architect → devs |

## 4. 交付物对照表（8 项提交物）
| # | 提交物 | 评分权重 | 状态 | 负责人 | 位置 |
| -- | -- | -- | -- | -- | -- |
| 1 | LFZ 语法规则文档 | 20（评分项 4 一部分） | ✅ 已冻结（v1 三件套，910/382/298 行） | language-architect | `docs/spec/` |
| 2 | LFZ 解释器源程序 | 20（评分项 1） | ✅ 已完成（445 单测全绿；`dist/lfz.exe` 704000 B） | core-dev + runtime-dev | `src/` |
| 3 | LFZ 完整黑盒测试集 | 20（评分项 2） | ✅ 已完成（85 用例全绿 + 覆盖矩阵） | test-engineer | `tests/` |
| 4 | LFZ 性能测试报告 | 10（评分项 3） | ✅ 已完成（LFZ 28–31× 慢于 CPython，诚实结论） | perf-engineer | `benchmarks/` + `docs/reports/performance.md` |
| 5 | 开发指南（人 + AI） | 20（评分项 4 一部分） | ✅ 已完成（`docs/guide/` 5 篇 + skill 包 + 4 实测程序） | docs-writer + ai-dx-engineer | `docs/guide/` + `.opencode/skills/lfz-programming/` |
| 6 | 应用源代码 + 开发记录 | 30（评分项 5） | ✅ 已完成（`app/sortviz.lfz` 341 行 + `DEV_RECORD.md`） | app-dev | `app/` |
| 7 | Git 历史记录 | 无独立分值（交付完整性） | ✅ 已建立（7 个标签，最新 `v1.2.0`） | release-manager | `.git/` + GitHub |
| 8 | 系统介绍 PPT | 无独立分值（答辩载体） | ✅ 已完成（14 页 + 演示脚本 + 问答预案） | ppt-presenter | `docs/slides/` |

## 5. 关键事实
- **技术栈（D-011）**：解释器用 **Rust**，仅 std、零第三方依赖（`Cargo.toml` version = 1.0.0）。模块：`loader` / `lexer` / `ast` / `parser` / `value` / `env` / `evaluator` / `builtins`（§10.7 **54/54**）/ `cli`。
- **语言事实（冻结 v1，D-016）**：`.lfz` 首行必须严格 `#42`；Python 式中文错误（12 类 + 基类，无 `E-xxx`）；退出码 `0` 成功 / `1` 测试失败 / `2` 任何错误（D-008）；54 内置 data-last；v1 = 核心 + 5 特色（管道 `|>`、合一 struct、富插值、结构化错误 + `assert`/`check`、确定性语义）+ `float`。
- **运行命令**：`lfz run <file>`、**`lfz <file>`（裸调用等价）**、`lfz test`（一键黑盒，85 用例）、`lfz test --json`（stdout 恒为单个 JSON；程序输出走 stderr）。`lfz` 已装到 `%LOCALAPPDATA%\Programs\lfz\` 并加入用户 PATH（`scripts\install-lfz.ps1 -Uninstall` 可回滚，备份 `path-backup.txt`）。
- **目录约定**：`docs/spec/`（语法/语义/接口契约）、`src/`、`tests/`（`lfz/` 黑盒 + `fixtures/` 负例 + `cases.json` + `coverage-matrix.md` + `REPORT.md`）、`benchmarks/`、`docs/guide/`、`docs/reports/`、`docs/slides/`、`app/`、`.opencode/`（团队 + 技能 + 插件 + 脚本）。
- **Git 与远程**：`origin` = https://github.com/NeitherTourRest/lfz-programing-language.git（Public）；默认分支 `main`；协议 **MIT**。⚠️ 认证账号 login = **`NeitherTourRest`**（display name = `MakeChase`），非字面账号 `MakeChase`。
- **环境坑**：cargo 不在默认 PATH（`$env:Path += ";$env:USERPROFILE\.cargo\bin"`）；PowerShell 5.1 把无 BOM 的 `.ps1` 按 ANSI 解码 → **所有 `.ps1` 必须纯 ASCII**；`gh` 不读 git 的 `http.proxy`，须注入 `HTTPS_PROXY=http://127.0.0.1:7890`；`input()` 非交互下不可靠（EOF/阻塞），黑盒测试避免使用。
- **临时工作区（用户 2026-09-27 指令，ADR 已记）**：所有 agent 的临时夹具/输出一律写入 **`<repo>/Temp/`**（项目内 D 盘，已 `.gitignore`，不入库）；**禁止**再写 C 盘 `%TEMP%`。每波任务收工后由 team-lead 盘点；一手证据须**先复制进 `docs/reports/**` 再清**临时件。
- **成本策略**：非高峰（DeepSeek 官方：高峰 = 周一至周五 01:00–04:00 & 06:00–10:00 UTC，其余 5 折）。`.opencode/plugin/offpeak.ts` 在高峰拦截 `task`/`call_omo_agent`；`scripts/offpeak.ps1` / `offpeak-runner.ps1` / `offpeak-task.cmd` 为配套（自启动已启用）。

## 6. 当前风险与开放决策
- **✅ T11-① 审计结论**（157 条款 / 149 PASS / 5 FAIL）→ **T11-② 修复批次已落地**（用户批准"全修"）：**4 类崩溃 + JSON 缺陷 + span 缺陷全部收口**（明细见 `TEAM_BOARD.md` T11-② 结果表）：
  | 缺陷 | 修法 | team-lead 亲测 |
  | -- | -- | -- |
  | `repeat` 容量溢出 → panic | 新消息 `容量溢出：所需容量超出可分配上限` | **exit 2**（原 101） |
  | `range` 超大 n → panic | 同上 | **exit 2** |
  | **解析深嵌套 → 栈溢出**（真阈值 56–62 层） | `PARSE_DEPTH_LIMIT=1000` + `NestingTooDeep` | `(`×1000→0 / ×1001→2 |
  | **深左偏 AST → 求值/析构栈溢出** | `AST_DEPTH_LIMIT=10000` + `ExprTooDeep` | 9999→0 / 10000→2 / **100000→2**（原 abort） |
  | `;;`×`--json` 破坏单 JSON | dump 通道随 `print` 转 stderr | stdout **恰 1 行** |
  | 越界写插入符 span 偏差 | 写路径 span = `target.span` | 插入符 **col 1** |
  - **质量基线**：`cargo test` **445 → 482**（0 failed / 0 ignored）；`lfz test` **85 → 87**；`cargo build --all-targets` 0 warning；**错误类仍 12**（无第 13 类）。
  - **关键教训**：3 个实现缺陷（含 2 个可达 panic）**445 单测与 85 黑盒都没抓到** → 条款级独立审计不可省；且**审计自身也漏了第 3/4 类崩溃**（由 runtime-dev 的"全量 panic 硬化排查"和架构师的量化论证挖出）→ **"顺带硬化排查"必须写进修复任务书**。
- **⏳ T11-③ 待办（已授权）**：`FEATURE-AUDIT.md` 的 **IN 7 项 + P0 文档 7 条**（用户选择"全修 + 复验 + 再迭代"已授权）；**OUT 18 项**为审计建议、用户未逐条追认，将照办并标出。
- **⏳ 待收口**：① T11-08 test-engineer 补黑盒负例；② **T11-09 verifier 全量复验**（7 类修复 + 482 单测 + 87 黑盒）；③ **T11-10 release-manager 重建 `dist/lfz.exe`**（当前仍是修复前 704000 B）+ **刷新 README/报告计数（445→482）** + 提交批次；④ `bug-20260927-02`（P9 记录 vs C 域实测矛盾）随复验裁定；⑤ `REQUIREMENTS.md` R-401 的 spec 字节数已陈旧（架构师提示）。
- **已知语言侧缺口（审计已裁定）**：**不加 `s[i]`**（UTF-8 不可变 → `chars().nth(i)` 会造成 O(n²)），改为文档化 `split("", s)` 惯用法；**动态宽度 OUT**（用 `padEnd`）；`let` 循环语义 / `len(string)` / `<` 对齐 **spec 早已规定，属 skill 文档缺口**。
- **架构级性能发现（审计 §7.1）**：string 不可变导致 **循环内 `s = s + c` 拼接是 O(n²)**（盲测 `json_mini`/`expr_eval` 即如此）→ ① doc 引导 `push` + `join`（O(n)）；② runtime 可选优化（`Rc` 强计数为 1 时就地追加，**不改语义、不改 A1**，需 runtime-dev 评估）。
- **回归面**：v1.1 七项均为「内置新增/签名扩展 + 1 条 spec 补钉」，对 A1–A7 与 §4.5 确定性**全兼容**；445 单测中**仅 1 处待核**（若存在 `range` 二元 `ArgCount` 断言）；85 黑盒不受影响。
- **遗留（非阻塞）**：`examples/life.lfz`（128 行，Game of Life，exit 0）与 `examples/test.lfz`（**盲测 agent 越界写入的散落探针，非交付物，待处理**）尚未提交。
- **风险**：`cargo test` / 黑盒 / 盲测三处「全绿」都不等于「规范完全符合」—— 这正是 T11 符合性审计要独立回答的问题。
