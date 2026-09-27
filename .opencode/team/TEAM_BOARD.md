# 团队任务看板
> 最后更新: 2026-09-27 by team-lead

本文件为活文档，唯一写者 team-lead。其他角色只读，通过结构化汇报请求 team-lead 更新。

## 🔵 进行中
| ID | 任务 | 负责 | 依赖 | 状态 | 产出/证据 |
| -- | -- | -- | -- | -- | -- |
| T11-10 | 重建 `dist/lfz.exe` + 刷新 README/报告计数（445→**482** / 85→**90**）+ `Cargo.toml`→1.0.1 + 标签 **`v1.2.1`** + 原子提交并推送 | release-manager | T11-09 ✅ | **进行中** | `dist/` + README + git 提交 + tag |
| （后续）T11-③ | **v1.1 迭代**：P0 文档 7 条（ai-dx-engineer ‖ docs-writer）+ 语言 7 项（architect → core/runtime）→ **同题盲测重跑** | 见 🟡 待办 T11-04/05/06 | T11-10 ✅ | 待派 | — |

### T11-① 复验结论（verifier 独立验收，2026-09-27）
| 条款 | 结果 |
| -- | -- |
| 1 `repeat` 溢出 → `OverflowError` | **PASS** |
| 2 `range` 超大 n | **PASS** |
| 3 `PARSE_DEPTH_LIMIT=1000` | **PASS（8/8）** |
| 4 `AST_DEPTH_LIMIT=10000` | **PASS（7/7）** |
| 5 `;;`×`--json` 同通道 | **PASS（3/3）** |
| 6 越界写 span 对齐 | **PASS（4/4）** |
| 7 §9.2 样例 B 逐字实跑 | **PASS** |
| 8 全量回归 | **PASS**（482 单测 / 90 黑盒 / 0 warning） |
| 9 顺带裁定 `bug-20260927-02` | **不成立 → 关闭**（P9 旧记录对应修复前产物） |
| 10 **对抗性巡检**（新构造找 panic/abort） | **PASS —— 无 reachable panic，无新缺陷** |
| **合计** | **全部 PASS / FAIL = 0** → 报告：`docs/reports/T11-reverification.md` |

> **一句话**：**T11 修复批次 通过**（"现有计划完全实现无误"的核对闭环达成②），尚待 T11-10 发布收口（①的产物/计数刷新）。

### T11-② 修复批次结果（用户 2026-09-27 批准「全修」；**6 项已落地并经 team-lead 亲测**）
| # | 缺陷 | 修法（ADR） | 我的实测证据 | 状态 |
| -- | -- | -- | -- | -- |
| 1 | `repeat` 容量溢出 → Rust panic | 新消息 `容量溢出：所需容量超出可分配上限` | `repeat(4611686018427387904,"ab")` → **exit 2**（原 101） | ✅ |
| 2 | `range` 超大 n → Rust panic | 同上（`OverflowMsg::Capacity`） | `range(4611686018427387904)` → **exit 2** | ✅ |
| 3 | **解析深嵌套 → 主线程栈溢出** | `PARSE_DEPTH_LIMIT=1000`（§3.8 / R-S1）+ `SyntaxMsg::NestingTooDeep` | `(`×1000→**0**；`(`×1001 / `{"a":`×1001 / `[`×1001 / `fn(){`×1001→**2** | ✅ |
| 4 | **深左偏 AST（左结合链）→ 求值/析构栈溢出** | `AST_DEPTH_LIMIT=10000`（§3.9 / R-S3）+ `SyntaxMsg::ExprTooDeep` | `1+1+…` 9999→**0**；10000→**2**；**100000/200000→2**（原 -1073741571） | ✅ |
| 5 | `;;` × `--json` 使 stdout 非单个 JSON | dump 通道随 `print` 转 stderr（复用 `P4.2-fix`） | `--json run`（含 `;;`）→ stdout **恰 1 行** `{"ok":true}`，dump 落 stderr | ✅ |
| 6 | 越界写 `a[5]=9` 插入符 span 偏差 1 | 写路径 span 统一为 `target.span` | 插入符 **col 1**（基座），与读取路径一致 | ✅ |
| 7 | `syntax.md` §9.2 样例 B 期望输出自相矛盾 | 仅订正样例文本为 `the/fox/quick`（**实现是对的**） | spec 侧，无运行时证据 | ✅ |
| 8 | `--help` 文案陈旧误导 | 区分裸调用 / `run` + 扩展名豁免说明 | 实测新文案正确 | ✅ |

> **架构师共出 6 条 ADR**：`range`/`repeat` 统一 `OverflowError` · §9.2 样例订正 · `float` 显示「最短往返优先」 · 零帧错误无 Traceback 头 · `PARSE_DEPTH_LIMIT=1000` · `AST_DEPTH_LIMIT=10000`（**两个上限口径正交、缺一不可**，量化安全系数 ≥14× / ≈9×）。
> **质量基线变化**：`cargo test` **445 → 482**（379+48+28+12+15；0 failed / 0 ignored）；`lfz test` **85 → 87**；`cargo build --all-targets` 0 warning。
> ⚠️ **待办**：`dist/lfz.exe` 仍是修复前构建（704000 B），README 仍写 445 —— **须由 release-manager 重建 + 刷新**（T11-10）。

### T11-① 规范符合性审计结果（2026-09-27 完成 · 3 路并行 · 全部实跑 witness · 只验证不修复）
| 审计域 | 报告 | 条款 | PASS | FAIL | 结论档位 |
| -- | -- | -- | -- | -- | -- |
| **A** 词法/文法/解析/5 特色 | `docs/reports/conformance-A-syntax.md`（35451 B） | 79 | 76 | **1（spec 侧）** | 有条件通过（**解释器侧 FAIL = 0**） |
| **B** 求值语义/错误模型 | `docs/reports/conformance-B-semantics.md`（29537 B） | 28 | 27 | **1** | 有条件通过 |
| **C** 内置 54/loader/runner/CLI | `docs/reports/conformance-C-builtins-cli.md`（32323 B） | 50 | 46 | **3** | 有条件符合 |
| **合计** | — | **157** | **149** | **5**（= 4 个独立缺陷） | **不完全无误** |

> **对用户问题「现有计划完全实现无误？」的回答**：**不是**。共 **3 个真实实现缺陷**（其中 **2 个能让解释器进程 panic 崩溃**）+ **1 个 spec 样例自相矛盾**；另有 6 项低危观察、1 项归属待裁定。
> ⚠️ 这 3 个缺陷**既没被 445 单测抓到、也没被 85 黑盒抓到** —— 再次印证「绿了 ≠ 对」，也印证了本轮条款级审计的必要性。

### 缺陷台账（审计产出）
| 编号 | 严重度 | 内容 | 证据 | owner | 状态 |
| -- | -- | -- | -- | -- | -- |
| **bug-20260927-03** | 🔴 高 | **`repeat` 溢出 → Rust panic（进程崩溃）**；spec 要求 `OverflowError` | C 域 C-25 | runtime-dev | ✅ 已修（见 T11-② 结果表） |
| **bug-20260927-04** | 🟡 中 | **`range` 超大 n → Rust panic**；**spec 静默**（需先裁定行为） | C 域 C-26 | language-architect → runtime-dev | ✅ 已修（见 T11-② 结果表） |
| **bug-B-20260927-01** | 🟡 中 | **`;;` × `--json` 使 stdout 非单个 JSON**（§3.6「与 print 同通道」在 `--json` 下不成立）；**B、C 两域独立复现** | B 域 B-14 / C 域 C-43b | tooling-dev | ✅ 已修（见 T11-② 结果表） |
| **BUG-A-01** | 🟢 低 | `syntax.md` §9.2 样例 B 期望输出 与 §10.7「keys 字节序 + sortBy 稳定」**自相矛盾**（解释器输出才是对的：the/fox/quick） | A 域 A-74 | language-architect（**只订正样例文本，不得改实现**） | ✅ 已修（见 T11-② 结果表） |
| obs-A-02 | 🟢 低 | CLI `--help` 文案陈旧误导（称"须 .lfz 结尾"，实际按 §2.2.0 豁免） | A 域 | tooling-dev | 观察 |
| obs-A-03 | 🟢 低 | 个别 SyntaxError 消息未点明真实原因（类名/位置正确） | A 域 | core-dev | 观察 |
| obs-B-01 | 🟢 低 | 文件不存在时 `IOError: 无法读取` **无 Traceback 头**（零帧运行期错误，§8.2 未覆盖） | B 域 | language-architect / tooling-dev | 观察 |
| obs-B-02 | 🟢 低 | `|x|≥1e16` 的整值浮点显示 `1e16` 无 `.0`（"最短往返"与"整值 .0"两条规则冲突） | B 域 | language-architect | 观察（规范歧义） |
| obs-B-03 | 🟢 低 | 索引写入越界 `a[5]=9` 插入符指 col 2，读取越界 `a[5]` 指 col 7（疑下标写入 span 偏差 1） | B 域 | runtime-dev | 观察 |
| bug-20260927-02 | ⚪ 待裁定 | P9 记录「`test --json` stdout 非单行」，但 C 域本轮**实测为单行** → 归属需裁定 | C 域 §③ | team-lead | 待裁定 |
| obs-A-01 | N-A | `-e` / stdin 入口模式未实现（spec §6-B9 已标 ⏸ 延后，**不属违反**） | A 域 | — | N-A |

> **T11 路线（用户 2026-09-27 指令）**：「**先确定现有计划完全实现无误，然后开始迭代**」——
> **第一步**（进行中）= 上表三项规范符合性审计：把 `docs/spec/` 冻结 v1 的规范性条款逐条对照实现行为，每条必须给可跑的 witness，产出 PASS/FAIL 矩阵 + 缺陷单（**只验证不修复**）。
> **第二步** = 依审计结论进入 **v1.1 迭代**（见下 🟡 待办 T11-04/05/06）。

## 🟡 待办
| ID | 任务 | 负责 | 映射评分项 | 通过条件 |
| -- | -- | -- | -- | -- |
| T11-04 | **迭代-文档：7 条 P0 skill 修订**（D1 字符串不可下标+`split("",s)` 惯用法 / D2 `range` 完整签名 / D3 循环体 `let` 每轮新绑定 / D4 退出码语境让 / D5 对齐 `<`·`>`·`^`+fill / D6 `len(string)` 合法且计 Unicode 标量 / D7 5 处隐性语法正面示例）+ 新增「逐字符构建字符串 → 先 `push` 到数组再 `join`」的 O(n) 惯用法 | ai-dx-engineer（人类文档由 docs-writer 同步） | 评分项 4 | skill 修订落地 + 用 lfz 实测；同题盲测通过率提升 |
| T11-05 | **迭代-语言：v1.1 七项补强**（① spec 补钉：string 取下标 → `TypeError` ② `range(lo,hi)` 半开 ③ 字符串方法族 `indexOf/endsWith/padEnd/padStart/substring` ④ 文件 IO `readFile/writeFile/appendFile` ⑤ `ord`/`chr` ⑥ math `sin/cos/log/exp` ⑦ `contains`）—— 严格走 **ADR → 改 `docs/spec/` → 实现 → 回归** | language-architect → core-dev/runtime-dev | 评分项 1 | ADR 记录；spec 更新；445 单测 + 85 黑盒全绿；新特性各有黑盒用例 |
| T11-06 | **迭代-验证：同题 8 题盲测重跑**，量测改进（基线 **2/8**） | verifier | — | 报告给出前后对比 + 新发现的缺口 |
| — | **明确 OUT（18 项，见 `FEATURE-AUDIT.md` §4）**：`try/catch`、标签 break、`match`、生成器、一等区间 `..`、`in` 运算符、Unicode 折叠、`hint`（永久 OUT）、动态宽度、模块、类/继承、类型注解、独立字典类型、可选链、`range` 3 参、工具链项、二进制/目录 IO、`charAt` | — | — | 审计已给逐条理由（克制也是结论）；如需重开须先请示用户 |

> **评分项对照（合计 100 分）**：评分项 1 解释器（20）= P3 + P4；评分项 2 自动测试（20）= P5；评分项 3 性能（10）= P6；评分项 4 语法说明 + 人/AI 指南（20）= P2 + P7；评分项 5 Agent 应用（30）= P8。P0.5/P1/P9/P10 为支撑阶段，无独立分值。
> **版本纪律（自 P0.5 起全程生效）**：release-manager 在每个阶段里程碑做**原子提交 + 附注标签 + 实时更新 README + push**；远程 = https://github.com/NeitherTourRest/lfz-programing-language（Public）。

### T11 前置：特性审计（已完成）
| ID | 任务 | 负责 | 状态 | 产出/证据 |
| -- | -- | -- | -- | -- |
| T11-00 | **特性缺口审计**：现状盘点 + 12 张候选卡（C1–C12）+ 明确 OUT 18 项 + 6 焦点裁定 + v1.1 IN 7 项 + P0 文档 7 条 | language-architect | ✅ | `.opencode/team/FEATURE-AUDIT.md`（39238 B / 407 行） |
| — | **skill 盲测**（8 题零上下文 agent，隔离工作区，仅给 skill + `lfz.exe`） | team-lead 派发 | ✅ | 通过 **2/8**（maze_bfs ✅ / knapsack_dp ✅）；报告见 `%TEMP%\opencode\lfz-skilltest\programs\*.report.md` |

## 🔴 阻塞
| ID | 任务 | 负责 | 阻塞原因 | 需要支持 |
| -- | -- | -- | -- | -- |
| bug-20260927-02 | `lfz test --json` 是否单行（P9 报告 vs C 域实测**矛盾**） | team-lead | 需专项复验收口（**不阻塞**主流程，但影响 P9 报告结论准确性） | 随 T11-09 复验一并做 |

## ✅ 已完成
| ID | 阶段 | 交付物 | 负责 | 映射评分项 | 通过条件 | 完成日期 |
| -- | -- | -- | -- | -- | -- | -- |
| P0 | 团队就绪 | 脚手架通过 S1–S5 | team-lead | — | 全部场景通过 | 2026-09-22 |
| P0.5 | 版本基线 | 本地 `git init` + 初始提交 `e060b21` + `v0.1.0`；远程 `github.com/NeitherTourRest/lfz-programing-language`（Public） | release-manager | 交付物 7 | `git ls-remote`：`main`=`35e5f62`、tag=`v0.1.0`；本地 = 远程 | 2026-09-23 |
| P1 | 需求 | `REQUIREMENTS.md` 需求矩阵 + 验收标准（对齐冻结 spec v1） | requirements-analyst | 全部（需求基线） | 矩阵完整 + 可跑验收清单 | 2026-09-23 |
| P2 | 语言设计 | `docs/spec/{syntax,semantics,interface-contract}.md` 冻结（910/382/298 行）+ ADR D-016 | language-architect | 评分项 4（+支撑评分项 1） | 冻结门禁：core-dev PASS；runtime-dev 3 项非架构级已闭合 | 2026-09-23 |
| P3 | 核心实现 | `loader`/`lexer`/`ast`/`parser`/`value`/`env`/`evaluator`/`builtins`（54/54）/`cli` | core-dev + runtime-dev | 评分项 1 | P3.11 终验 rev.3 **PASS**（10/10 缺陷闭环、0 回归）；tag `v0.2.0` | 2026-09-24 |
| P4 | 工具链 | `lfz run <file>`、**`lfz <file>` 裸调用**、`lfz test`（T-R1–T-R4）、`--json`（stdout 纯 JSON）、release profile、`scripts/build-release.ps1` + `install-lfz.ps1`（dry-run + `-Uninstall`）、`dist/lfz.exe` | tooling-dev | 评分项 1（+支撑评分项 2） | 命令实测通过；`dist/lfz.exe` **704000 B** | 2026-09-27 |
| P5 | 黑盒测试 | `tests/lfz/`（26 正向 + 58 负例 + 1 豁免 = **85 用例**）+ `coverage-matrix.md` + `REPORT.md` | test-engineer | 评分项 2 | `cargo run -- test` → **85/85，exit 0**（一个命令跑全部）；tag `v0.3-tested` | 2026-09-25 |
| P6 | 性能 | `benchmarks/`（6 对 LFZ/Python + `noop` 启动探针 + `run_all.py` + `results/raw.json`）+ `docs/reports/performance.md` | perf-engineer | 评分项 3 | 预热 + 多轮 + 中位数；**诚实结论：LFZ 28–31× 慢于 CPython**，启动更快（11.67 ms vs 42.34 ms） | 2026-09-25 |
| P7 | 文档 | `docs/guide/`（README/tutorial/reference/errors/testing）+ AI skill `.opencode/skills/lfz-programming/`（SKILL 28.7 KB / VERIFICATION 15 KB 含需求追溯表 / README / prompt-template） | docs-writer + ai-dx-engineer | 评分项 4 | 评分项 4 完整；4 个实测程序；tag `v1.2.0` | 2026-09-27 |
| P8 | 应用 | `app/sortviz.lfz`（**341 行**，5 算法 + ASCII 可视化 + 种子化 + 150 轮自检）+ `DEV_RECORD.md` + `README.md` | app-dev | 评分项 5 | `lfz run app/sortviz.lfz` exit 0，5 算法全部 `[校验通过]`（用 skill 开发）；tag `v0.4-app` | 2026-09-26 |
| P9 | 验证 | 交付物级独立验收（8 项逐项实跑） | verifier | — | 结论 **CONCERNS（无阻塞）**；`bug-20260927-01`（`s["k"]()` 未绑定 `self`）已修复并补黑盒用例；tag `v1.0-final` | 2026-09-27 |
| P10 | 发布+答辩 | `docs/reports/delivery-checklist.md`（≥3 轮核对）+ `docs/slides/`（14 页 PPT + `demo-script.md` + `qa-prep.md`）+ README 刷新 + `Cargo.toml` → 1.0.0 | release-manager + ppt-presenter | — | 8 项交付物齐备 | 2026-09-27 |

### 质量基线（实测，2026-09-27）
| 项 | 命令 | 结果 |
| -- | -- | -- |
| 构建 | `cargo build` | **0 warning / 0 error**（零第三方依赖，仅 std） |
| 单元测试 | `cargo test` | **445 passed / 0 failed / 0 ignored**（lib 362 + main 47 + cli 24 + test_runner 12） |
| 黑盒测试 | `cargo run -- test` | **85/85，exit 0** |
| 端到端 | `lfz examples/hello.lfz` | `Hello, LFZ!`（exit 0） |
| 应用 | `lfz run app/sortviz.lfz` | exit 0；5 算法 `[校验通过]`；341 行 |
| 版本标签 | — | `v0.1.0` · `v0.2.0` · `v0.3-tested` · `v0.4-app` · `v1.0-final` · `v1.1.0` · `v1.2.0` |

### P3 子阶段台账（用户协议：每子阶段 → 独立检查 → 原子提交）
| 子阶段 | 内容 | 负责 / 执行者 | 状态 | 提交 |
| -- | -- | -- | -- | -- |
| P3.0 | Cargo 骨架（lib + bin，std only） | core-dev | ✅ | `69a57d7` |
| P3.1 | `span.rs` + `error.rs`（12 错误类 + 方法 + `R<T>`） | core-dev | ✅ | `4035f86` |
| P3.2 | `loader.rs`（UTF-8/BOM/`ext`/`#42`/`line_base`） | core-dev | ✅ | `7d41e17` |
| P3.3a | `lexer.rs` CODE 模式核心记号 | core-dev | ✅ | `4c68d18` |
| P3.3b | `lexer.rs` STR/INTERP + 未闭合块注释 | core-dev | ✅ | `4f2a22e` |
| P3.4a | `ast.rs` 全节点带 `Span`（共享接口门禁） | core-dev | ✅ | `1dab6f8` |
| P3.4b1 | `parser.rs` 骨架 + 语句层 | Sisyphus-Junior（**改派**） | ✅ | `89b8624` |
| P3.4b2a | `parser.rs` 后缀 + 数组/结构体字面量 | Sisyphus-Junior | ✅ | `81a4b8a` |
| P3.4b2b | `parser.rs` 二元运算符 + 赋值 | Sisyphus-Junior | ✅ | `ae88d38` |
| P3.4b3a | `parser.rs` 控制流语句 | Sisyphus-Junior | ✅ | `53ca3e2` |
| P3.4b3b | `parser.rs` `fn`/`struct` 声明 + 函数字面量 | Sisyphus-Junior | ✅ | `0ce5123` |
| P3.5 | `parser.rs` 特色四件套（管道/`;;`/插值/`i64::MIN`） | Sisyphus-Junior | ✅ | `682d1fb` |
| P3.6a | `value.rs` + `env.rs`（A1/A2/`ScopeDebug`） | runtime-dev | ✅ | `49d485f` |
| P3.6b | 值语义辅助（A6 环安全 `==` + §4.5.6 全序） | runtime-dev | ✅ | `88ec1bc` |
| P3.7 | `evaluator.rs` 核心 | runtime-dev | ✅ | `cab79ba` |
| P3.8 | `evaluator.rs` 语义定稿 | runtime-dev | ✅ | `f9aec70` |
| P3.9a | `builtins.rs` 非高阶 47 个 | runtime-dev | ✅ | `b81680b` |
| P3.9b | `builtins.rs` 高阶 7 个（**§10.7 54/54**） | runtime-dev | ✅ | `d51755a` |
| P3.10 | 最小 CLI + `examples/hello.lfz` | tooling-dev | ✅ | `c7627a6` |
| **P3.11** | **独立验收（rev.1 FAIL → rev.2 CONCERNS → rev.3 PASS，10/10 缺陷闭环、0 回归）→ 里程碑标签 `v0.2.0`** | verifier | ✅ | tag `v0.2.0` |
| 规范 | 7 处契约缺口闭合（先 ADR 后改 `docs/spec`） | language-architect | ✅ | `711d6b9` `5cdcc1b` |

> **执行者改派记录**：`parser.rs` 因原负责人 2 次未交付，自 P3.4b1 起由 team-lead 改派 **Sisyphus-Junior**（`unspecified-high`），并将任务切成 ≤1 模块的小批次，每个批次以「**必须贴出文件实际字节数**」为真假完成判据；改派后 6 个批次全部一次落地。
