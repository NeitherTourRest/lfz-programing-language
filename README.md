# LFZ 语言与解释器 — 程序设计课程作业

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
> 最后更新: 2026-10-30 by release-manager
> 仓库地址: https://github.com/NeitherTourRest/lfz-programing-language

LFZ 是一个解释型通用脚本语言及其解释器，配套黑盒测试、性能对比、人/AI 开发指南与 Agent 开发应用。项目由 14 名 opencode AI 开发团队协作完成。

## 本轮结束：T18-A Windows受控子进程候选（2026-10-30）

当前1.4.0候选、74内置/13错误类，新增`exec(argv, timeoutMs, maxOutputBytes)`，默认禁用，仅显式`--allow-process`启用本session能力。只接受显式绝对.exe，不搜索PATH或隐式拼shell；用户仍可显式选择shell，不视为安全沙箱。Windows后端已验收，非Windows受控Unsupported，不声称跨平台已实现。

最终快照984064B、SHA256 `4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86`，[递归源码/测试/exe归档](docs/reports/T18-A-release-evidence/build.json)冻结26成员，包含Windows后端及fixture源码。[最终独立验收](docs/reports/T18-A-independent-verification.md)追加PASS：649 harness、公开45、toolplan7、自编真实进程12、旧全部回归（含T17公开134/预算app40、C公开29/app33）通过；人指南22/AI26为分列作者实跑。

详见[子进程指南](docs/guide/subprocess.md)及[工具计划应用](app/T18-A-TOOLPLAN.md)。输出按两流合并原始字节限额、严格UTF8，局部timeout可被attempt恢复，全局deadline/取消仍穿透；Windows Job与显式清理不等于权限隔离或硬实时保障。执行继承host环境/cwd、stdin为EOF，不提供任意env/cwd/stdin配置。真实资源关闭错误诊断不会伪装为成功。

默认受限环境的C0000008/权限失败、E5阶段Close修正、NUL新oracle首次失败及全部原始证据均保留，不改写为一次通过。R304仍OPEN，无本轮性能计时，候选非正式发布、不打tag。**按用户要求本轮到此停止，不启动HTTP或其它后继设计/源码；其余计划未实施。** 最终提交/推送与停止自动化由主会话确认实际回执。以下T17/C为历史指纹记录。

## T17 历史协作式执行限额候选（绑定1946ab）

T17提供可选`--max-steps N`与`--timeout-ms N`，默认不限额；Rust宿主另可注入取消token。步数按实际stmt/expr入口计，模块初始化、函数和回调共享会话余量；执行终止为新增第13类ExecutionLimitError，不能被attempt吞掉，既有副作用不回滚。

当前快照943104B、SHA256 `1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45`，[精确源码/测试/exe归档](docs/reports/T17-release-evidence/build.json)已封存。[最终独立报告](docs/reports/T17-independent-verification.md)后续追加PASS：624 harness、最终公开134、预算应用40、自编14，以及旧全回归和C公开29/应用33通过。指南32/AI38为作者实跑，并经独立SHA审计，不冒称独立逐例重跑。首次121/122矩阵与CONCERNS、oracle订正及全部原始失败记录均保留，不重写历史结论。

```powershell
./dist/lfz.exe check --json job.lfz
./dist/lfz.exe --max-steps 10000 --timeout-ms 1000 run job.lfz -- '输入'
```

限额只在求值节点边界协作检查，native计算或阻塞IO内部不强制中断；timeout不含读取/解析/构图，不是内存/权限沙箱、CPU配额或硬实时保障。check/test/help/version不接受限额，参数及CODE/argv保护规则见[限额指南](docs/guide/execution-limits.md)。R304仍OPEN，无本批性能计时，候选非正式发布、不打tag；后继T18活动实现不属于本冻结验收。以下C/B为历史指纹。

## T16-C 历史模块执行候选（绑定cdedc9）

C已实现模块初始化/执行、定义模块中的函数与闭包查名、跨文件错误来源及模块化应用，先完成B静态图检查再初始化依赖。当前C快照1.4.0、902656B、SHA256 `cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006`；[精确源码、测试和exe归档](docs/reports/T16-C-release-evidence/build.json)冻结24项，后继活动源码不能冒用为C验收输入。

[最终独立报告](docs/reports/T16-C-independent-verification.md)：599 release harness、公开29、四文件JSONL应用33、自编真实模块15、旧attempt13/check117/CLI91含178/app37+11/jsonbatch38/JSON140全部PASS；人类指南11/AI示例18为分列作者实跑，不虚增独立数。模块缓存仅在本次session内，共享依赖只初始化一次；模块函数和逃逸闭包在定义模块查名，失败来源与trace使用真实文件。CLI check仍只做图检查，不运行初始化。

模块写法与边界见[模块指南](docs/guide/modules.md)、[应用](app/T16-JSONBATCH-MODULES.md)及spec §21。T17 §22仅冻结后继设计，本C没有预算执行功能或资源隔离承诺。R304OPEN，无本轮性能计时、非正式发布，不打tag；不拿B/C功能通过代替效率门槛。以下B/A为历史指纹记录。

## T16-B 历史非执行模块图检查候选（绑定8ea375）

B提供import/export声明解析与静态依赖图检查，**仅graph check，不初始化或执行模块**。C执行层§21是后继设计，不属于B已验收能力；后续活动源码可能进入C，实现版本须按独立指纹区分。

B快照1.4.0、859136B、SHA256 `8ea375dcd08098ef61d040a57a7b1ad28a28c2fdc25a5ca4921a75b728944480`，[精确18源码/2Cargo/exe归档](docs/reports/T16-B-release-evidence/build.json)保全21成员，不含C设计文档。已落盘最终独立验收报告确认：584 harness、公开38、旧attempt13/check117/CLI91含178/app37+11/jsonbatch38/JSON140，另18独立模块进程PASS；见[独立报告](docs/reports/T16-B-independent-verification.md)。默认受限环境canonicalize失败及作者控制记录保留，不能把失败环境结果冒充通过。

模块图验证遵循[§20契约](docs/reports/T16-B-design.md)，图错误与来源归属不等于模块已经运行。73内置保持，旧T15 Result与CLI无模块路径保持。R304OPEN，无本批性能计时、不打正式标签；旧v2及原始测试不改。下文A/T15均属历史指纹。

## T16-A 历史来源身份基础候选（绑定65a29）

本批只提供Rust侧来源元数据、错误来源载荷与函数定义模块身份基础，**尚无import/export解析、模块加载或模块执行**。现CLI、T15 Result/error/trace字段及73内置保持；不是已经可用的模块系统。

当前target/dist为1.4.0候选，783872B，SHA256 `65a29eaea54f32d1d2b404f22aed09389c3ffa71e6a81acb8fa6351d32fa42b7`，[源码与产物指纹](docs/reports/T16-A-release-evidence/build.json)已保全。[最终独立验收](docs/reports/T16-A-independent-verification.md)已通过：562 release harness、attempt13、check117、旧CLI91（含178官方黑盒）、旧应用37+11、jsonbatch38、旧JSON/argv140及新Rust API witness编译/执行全部PASS。另16程序×2模式=32组、64进程的新旧差分逐字节一致，源码/Cargo测前后指纹一致。本批没有新增LFZ语法或CLI行为。Rust witness首次默认LTO链接配置不匹配的失败日志保留，按现有库匹配编译参数后通过，不冒称产品运行缺陷。

Rust库使用者注意：公开`UserFn`新增`home_module`字段，手写结构体构造须迁移；旧Span、LzError、TraceFrame、Loaded与Program形状保持。SourceId/ModuleId分域，来源表是可选元数据，不会自动读文件、规范化路径或切换globals。见[设计](docs/reports/T16-A-design.md)和[契约§19](docs/spec/interface-contract.md)。

R304仍OPEN，无本批性能计时，不打正式标签、不移动旧标签与v2冻结材料。后继T16-B设计草案不是已实现能力；以下T15/T14为历史候选记录。

## T15 历史受控错误恢复候选（绑定1a5a）

当前项目内解释器仍为1.4.0候选，**73内置**，新增`attempt(f)`；target/dist均782848B，SHA256 `1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c`。[构建指纹](docs/reports/T15-release-evidence/build.json)保全当前源码/Cargo与精确产物。[最终独立功能验收](docs/reports/T15-independent-verification.md)已通过：549 release harness、178官方黑盒（包含在旧CLI91 full内）、attempt公开13、check117、旧应用37+11、新jsonbatch38、旧JSON/argv140及额外38独立进程全部PASS。人类指南20/20、AI示例25/25为各作者实跑结果，与独立验收分列，不累加重叠用例。

`attempt`接收零参数函数并调用一次，返回`ok/value/error`三字段struct；成功nil与失败有明确区分。仅恢复规定的业务错误及整数越界，保留完整错误消息、位置和traceback。语法/前导错误、递归超限、容量溢出及宿主panic/abort/分配失败不被捕获；参数求值及attempt自身校验在恢复边界之外。它不回滚已发生的容器/cell修改、文件写入、输出或随机状态，不是事务或资源隔离。详见[契约§18](docs/spec/semantics.md)与[实现](docs/reports/T15-runtime.md)。

```lfz
#42
let result = attempt(fn() { jsonParse("bad JSON") })
print(result.ok)
```

[jsonbatch](app/T15-JSONBATCH.md)演示good/bad/good JSONL记录继续处理、按输入顺序输出错误数据；IO与资源边界保持原契约，输出路径别名及非原子写限制见应用说明。旧check前端检查仍不执行attempt回调。

R-304仍OPEN，本轮不做T15性能计时，不把T12/T13旧成绩当当前二进制测量。仍是功能候选、非正式发布，不打v1.4.0标签，不移动旧标签与v2冻结材料。T13两项生产优化已拒收，失败证据保留。下文T14/T12数据均是各自指纹的历史记录。

## T14 历史非执行前端检查候选（绑定83f56）

项目内当前 `dist/lfz.exe` 为1.4.0候选、774656B，SHA256 `83f56a194e216862c45377f4c3971b085744a7ba2363163e1edac6af5e85ac79`；[构建来源与源码指纹](docs/reports/T14-release-evidence/build.json)已保全。[最终独立验收](docs/reports/T14-independent-verification.md)已通过：542项release测试、新check公开117/117、旧CLI91/91（内含166官方黑盒）、应用37+11、旧独立140及本轮自编57见证全部PASS。人类指南27/27、AI示例27/27作者实跑通过；作者证据与独立验证分别记录，不合并虚增计数。

```powershell
./dist/lfz.exe check examples/hello.lfz
./dist/lfz.exe check -e 'print(1 / 0)'
./dist/lfz.exe --json check -e 'print(1 / 0)'
# {"ok":true,"mode":"check","executed":false}
```

`check FILE`、`check -e CODE`、`check -`只执行加载、词法、语法和现有AST深度检查，不运行代码。普通成功双流空，失败退出2；机器结果明确`executed:false`。检查成功不代表名字、类型、参数个数、IO路径或实际执行正确，也不是资源隔离机制。它与LFZ内置`check(condition)`不同，且不接受`--`脚本参数。详见[契约§17](docs/spec/syntax.md)、[实现证据](docs/reports/T14-tooling.md)、[公开验收](docs/reports/T14-blackbox.md)、[指南](docs/reports/T14-guide-evidence.md)及[AI示例](docs/reports/T14-skill-evidence.md)。公开验收首次116/117的非法UTF8文件oracle错误按既有spec订正，首次证据保留，没有修改产品或降低旧测试要求。

T13两项省分配生产优化因唯一性能窗口有回归而拒收，已恢复T12生产行为，仅保留新增回归测试。其9cda候选、16源码/Cargo/exe和全部失败证据已[独立封存](docs/reports/T13-release-evidence/candidate-snapshot.json)，[性能结果](docs/reports/T13-performance.md)为15 PASS/2 ENV/2 FAIL，R-304仍OPEN。T14不另开性能窗口，不能把T13结果当作83f新二进制的性能成绩。本轮仍是功能候选，非正式发布，不创建v1.4.0标签、不移动v1.3.0；旧v2基线和原始材料保持不变。

## T12 历史候选工作快照（以下计数绑定47ED，不代表当前T14）

当前源码与项目内构建版本为 **1.4.0 候选工作快照、72 内置，非正式发布**；不创建 `v1.4.0` 标签，旧 `v1.3.0` 不移动。最终47ED二进制的功能已独立复验通过：522项Rust release、166项黑盒、91项进程检查、应用37+11项、140个自编witness，另16个旧新差分覆盖8类while范围，stdout/stderr/exit逐字节一致。R-304 保持开放：19 项原绝对性能线中 15 项达线、3 项环境不确定、1 项未通过。源码和完整成功/失败证据可作为候选 checkpoint 提交保存，不代表整体效率或正式版本验收完成。

- `argv()` 与 `--` 脚本参数，支持空字符串、Unicode 和字面量选项；`-e CODE`、stdin 源码入口方便 Agent 直接执行片段。
- `jsonParse` / `jsonStringify`：精确 i64、有限浮点、Unicode、严格重复键检查、有界深度和环拒绝；struct 数据面按 UTF-8 键序输出确定性 JSON。
- [jsonflow](app/README.md)：用配置和参数驱动 JSON 筛选、字段投影及结果文件；[sortviz](app/README.md) 新增规模、种子与帧显示参数，默认输出与旧版一致。
- [T12 黑盒](docs/reports/T12-blackbox.md)、[应用验证](docs/reports/T12-app.md)、[人类指南](docs/reports/T12-guide-evidence.md)、[AI 指南](docs/reports/T12-skill-evidence.md)及[性能报告](docs/reports/T12-performance.md)分别保留证据。功能结果不替代性能门槛，最终验收与交付状态以项目状态和最终报告为准。

## 课程阶段记录（历史里程碑，当前T14以本页顶部为准）
| 阶段 | 内容 | 状态 |
| -- | -- | -- |
| P0 | 团队就绪（脚手架通过 S1–S5） | ✅ 已完成 |
| P0.5 | 版本基线（git init + 初始提交 + `v0.1.0` 标签） | ✅ 已完成 |
| P2 | 语言设计（`docs/spec/` v1 三件套冻结，D-016） | ✅ 已完成 · 里程碑 **`v0.2.0`** |
| P3 | 解释器核心（loader / lexer / ast / parser / evaluator / builtins(72) / CLI） | ✅ 已完成 |
| P4 | 工具链（CLI、一键测试 runner、打包脚本） | ✅ 已完成 |
| P5 | 黑盒测试集（35 正向 + 130 负例 + 1 豁免，共 166 用例；覆盖矩阵） | ✅ 已完成 · 里程碑 **`v0.3-tested`** |
| P6 | 性能基准（LFZ vs Python，预热 + 多轮 + 中位数） | ✅ 已完成 |
| P7 | 文档（人类手册 5 篇 + AI 指南 / skill） | ✅ 已完成 |
| P8 | 应用（`app/sortviz.lfz` 362 行，5 算法；JSON 数据脚本 jsonflow） | ✅ 已完成 · 里程碑 **`v0.4-app`** |
| P9 | 独立验收（交付物级，verifier） | ✅ 已完成 · 结论 **CONCERNS（无阻塞）** |
| P10 | 交付完整性（README 刷新 + 交付清单核对 + 提交） | ✅ 已完成 · 里程碑 **`v1.2.1`** |

> **本 README 由 release-manager 在每个阶段里程碑实时更新**，以反映最新交付状态、Git 基线与版本标签。

## T12 当前功能验证（不等同于最终发布验收）
| 项 | 命令 | 实测结果 |
| -- | -- | -- |
| 构建 | `cargo build --release` | **exit 0**（仅 std；本机 rustup 路径 canonicalize 提示，不宣称零 warning） |
| Rust release 回归 | `cargo test --release` | **522 passed / 0 failed / 0 ignored**（lib 404 + main 48 + cli 28 + cli_t12 12 + test_runner 15 + unit 15） |
| 黑盒测试 | `cargo run -- test` | **166 个用例，通过 166，失败 0，错误 0**（exit 0，一个命令跑全部） |
| 端到端 | `cargo run -- run examples/hello.lfz` | `Hello, LFZ!`（exit 0） |
| 应用 | `cargo run -- run app/sortviz.lfz` | exit 0；**5 算法**全部 `[校验通过]`；**362 行**，150 次排序自测无失败 |
| 最终候选构建 | `dist\lfz.exe` | **759808 B**（`lfz 1.4.0`，release opt-level=2；与 target/release 同 hash，非正式发布） |

> **T11 / v1.3.0 历史发布基线**：当轮已实现 15 个新内置（总计 69）与 `range(lo, hi)`，含获用户追认的 `readFile` / `writeFile` / `appendFile`。独立定向验证 74/74，runner 干净工作目录 130/130；详见 [T11-v11-reverification.md](docs/reports/T11-v11-reverification.md)。用户已授权改用新八题建立 v2 基线，正式首轮结果见下方报告；历史 2/8 保留且不作跨题集提升比较。Cargo 版本仍为 1.0.1，`v1.3.0` 交付里程碑包含上述已验收实现、测试与 v2 正式首轮证据；Git 附注标签为发布凭据，解释器版本保持 1.0.1。
>
> **T11 历史产物** SHA256：`149B184E442FFBF250590431FA0AEFFAB401EAC526FAF60B4B435F620725D5AB`。`dist/` 为忽略的构建目录；可用 `scripts/build-release.ps1` 重建。构建与发布版黑盒原始证据见 [T11-v11-release-evidence/](docs/reports/T11-v11-release-evidence/)。
候选 SHA256：`47ED2BFE2A2694B11BD611172CFCF0000D0BFC58E3B9BD809CA95F066DD0261F`；[最终构建与522项日志](docs/reports/T12-release-evidence/final-opt2/build-final.json)绑定 Cargo、profile 和源码哈希。profile 采用实测的体积/启动取舍 `opt-level=2`，保留 LTO、单 codegen 与 strip；不能将多项优化的全部效果单独归因于字段索引。

[性能报告](docs/reports/T12-performance.md)披露同期同路径：结构体8000键 93.431→21.198ms（耗时约减少77.3%），500k数值循环104.940→90.804ms；startup 16.287ms在原16.307ms线内、余量很小。string_map8000项22.691ms高于原22.521ms线；另外3项因同期旧版也超参考而标环境不确定，均不算通过。部分负载即使在绝对线内仍比同期旧版慢，完整原始值和失败记录保留。

## 交付物索引（8 项）
| # | 交付物 | 位置 | 评分项 |
| -- | -- | -- | -- |
| 1 | LFZ 语法规则文档 | `docs/spec/`（syntax / semantics / interface-contract） | — |
| 2 | LFZ 解释器源程序 | `src/`（16 文件，Rust 仅 std） | 1（20 分） |
| 3 | LFZ 完整黑盒测试集 | `tests/`（`lfz/` + `fixtures/` + `cases.json` + `coverage-matrix.md` + `REPORT.md`） | 2（20 分） |
| 4 | LFZ 性能测试报告 | `benchmarks/` + `docs/reports/performance.md` | 3（10 分） |
| 5 | 开发指南（人 + AI） | `docs/guide/` + `docs/guide/ai/` + `.opencode/skills/lfz-programming/` | 4（20 分） |
| 6 | 应用源代码 + 开发记录 | `app/sortviz.lfz` + `app/jsonflow.lfz` + `app/DEV_RECORD.md` | 5（30 分） |
| 7 | Git 历史记录 | `.git/`（仓库历史 + 版本标签） | — |
| 8 | 系统介绍 PPT | `docs/slides/`（`LFZ-defense.pptx` 14 页 + `demo-script.md` + `qa-prep.md`） | — |

## 验收与证据
| 证据 | 位置 |
| -- | -- |
| T12 最终47ED独立功能验收（不含效率通过结论） | [独立报告](docs/reports/T12-independent-verification.md)、[最终原始材料](docs/reports/T12-independent-final/)；522/166/91、应用37+11、140witness、16旧新while差分 |
| T12 应用、指南与构建 | [应用 37+11 项](docs/reports/T12-app.md)、[人类指南 35 代码块+10 探针](docs/reports/T12-guide-evidence.md)、[AI 30 探针](docs/reports/T12-skill-evidence.md)、[最终候选构建证据](docs/reports/T12-release-evidence/final-opt2/build-final.json) |
| T12 性能（R-304 未过门槛）与待发布计划 | [性能报告](docs/reports/T12-performance.md)、[发布准备](docs/reports/T12-release-plan.md) |
| P3 解释器核心验收（rev.3 终验 **PASS**，缺陷闭环 10/10，新增回归 0） | [`docs/reports/P3-verification.md`](docs/reports/P3-verification.md) |
| P9 交付物级独立验收（8 项逐项实跑，**CONCERNS（无阻塞）**） | [`docs/reports/P9-verification.md`](docs/reports/P9-verification.md) |
| 黑盒测试报告（覆盖矩阵说明 + 已知偏差自认） | [`tests/REPORT.md`](tests/REPORT.md) |
| 交付完整性核对报告（对照 8 项，≥3 轮） | [`docs/reports/delivery-checklist.md`](docs/reports/delivery-checklist.md) |
| 项目状态检查 | [`docs/reports/status-check.md`](docs/reports/status-check.md) |
| T11 规范符合性审计 A（词法 / 文法 / 解析 / 5 特色，79 条款） | [`docs/reports/conformance-A-syntax.md`](docs/reports/conformance-A-syntax.md) |
| T11 规范符合性审计 B（求值语义 / 错误模型，28 条款） | [`docs/reports/conformance-B-semantics.md`](docs/reports/conformance-B-semantics.md) |
| T11 规范符合性审计 C（内置 54 / loader / runner 契约 / CLI，50 条款） | [`docs/reports/conformance-C-builtins-cli.md`](docs/reports/conformance-C-builtins-cli.md) |
| T11 修复批次全量复验（37 条款含子项 + 2 补充，**PASS 39 / FAIL 0**） | [`docs/reports/T11-reverification.md`](docs/reports/T11-reverification.md) |
| T11 特性缺口审计（IN 7 项 / OUT 18 项 / P0 文档 7 条） | [`.opencode/team/FEATURE-AUDIT.md`](.opencode/team/FEATURE-AUDIT.md) |
| T11 前置 skill 盲测（8 题零上下文 agent，通过 2/8） | [`docs/reports/blind-test/`](docs/reports/blind-test/) |
| LFZ Agent 编程基准 v2（冻结新八题任务与验收工具） | [`docs/validation/agent-benchmark-v2/TASKS.md`](docs/validation/agent-benchmark-v2/TASKS.md) |
| v2 正式首轮独立验收（8/8 整题，131/131 功能用例；源码及协议审查 PASS） | [`v2-20261024-r01/REVIEW.md`](docs/reports/agent-benchmark-v2/v2-20261024-r01/REVIEW.md) |

> 正式首轮 `v2-20261024-r01` 使用 8 个独立无历史 Agent 会话，模型记录为 `gpt-6.1-sol / medium`；[认证结果](docs/reports/agent-benchmark-v2/v2-20261024-r01/certified-results.json)为 **8/8 整题、131/131 功能用例**，8 份源码约束及协议均经独立审查。学生共 9 次 LFZ 自调，各题首次自调均 exit 0（8/8）；这是学生自选测试的过程辅指标，不代表首跑就通过冻结验收。隔离限于分题无历史派发、提示词约束和已记录工具动作审计，未实施 OS 隔离。本轮建立新基线，不与历史 2/8 直接比较或计算提升比例；[原始字节归档](docs/reports/agent-benchmark-v2/v2-20261024-r01/raw-inputs-submissions.zip)保留输入与提交证据。
> P3 复核夹具与命令原文见 [`docs/reports/fixtures-p3-rev3/`](docs/reports/fixtures-p3-rev3/)。
> P9 报告同时记录 2 项非阻塞遗留（`bug-20260927-01`：`s["k"]()` 未绑定 `self`；`obs-01`：README 陈旧），本 README 刷新即闭合 `obs-01`。
> T11 复验裁定 `bug-20260927-02`（`lfz test --json` stdout 非单行）**不成立 → 关闭**（当前源码同源构建下 stdout 恒为单个 JSON）；复验证据见 [`docs/reports/T11-reverification.md`](docs/reports/T11-reverification.md)。

## 版本里程碑
| 标签 | 对应阶段 | 说明 |
| -- | -- | -- |
| `v0.1.0` | P0.5 | 项目脚手架 + 团队宪法 + 冻结 v1 语言规范 |
| `v0.2.0` | P2/P3 | 语言设计冻结 + 解释器核心实现 |
| `v0.3-tested` | P5 | 黑盒测试集完成（54 内置覆盖 + 覆盖矩阵） |
| `v0.4-app` | P8 | 排序算法可视化应用（341 行，5 算法）+ 开发记录 |
| `v1.0-final` | P9/P10 | 最终交付（LFZ v1）：规范冻结 + 解释器（445 单测全绿）+ 黑盒集（85 用例全绿）+ 性能报告 + 人/AI 指南 + 341 行排序可视化应用 + Git 历史 + 答辩 PPT |
| `v1.1.0` | P4.4 | 打包：裸文件调用 `lfz <file>`、release profile（lto/strip）、自包含 `dist/lfz.exe` + 构建/安装脚本 |
| `v1.2.0` | P7 | `lfz-programming` skill 包：5 步 Agent 工作流、错误→修复映射、安装 README、需求追溯、4 个实测程序 |
| `v1.2.1` | T11 | 修复批次收口：6 条 ADR（`repeat`/`range` 容量溢出统一 `OverflowError`、`PARSE_DEPTH_LIMIT=1000`、`AST_DEPTH_LIMIT=10000`、§9.2 样例订正、`float` 显示口径、零帧错误渲染）+ `;;`×`--json` 通道修复 + 越界写 span 修复；**482 单测 / 90 黑盒全绿** |
| `v1.3.0` | T11 / Agent benchmark v2 | 15 新内置与 range 双参、文件 IO、494 Rust 测试 / 130 黑盒；v2 正式首轮 8/8 整题、131/131 功能用例与源码/协议独立审查证据；解释器仍为 1.0.1，发布以 Git 附注标签为凭据 |

> **`v1.3.0` 交付里程碑**：69 内置、494 Rust 测试与 130 黑盒，以及 v2 正式首轮 8/8 整题、131/131 功能用例的独立认证与原始证据归档；以 Git 附注标签为发布凭据。解释器版本仍为 1.0.1。历史上 `v1.0-final` 为 P10 首次终交付标签，`v1.2.1` 为 T11 修复批次里程碑。

## 团队说明
本项目由 14 名 AI 开发团队运作。团队宪法见 `AGENTS.md`；团队共享记忆（组织架构、看板、需求、决策、状态）见 `.opencode/team/`。任务入口：`task-info.md`（原始要求）、`.opencode/team/REQUIREMENTS.md`（需求矩阵）、`.opencode/team/PROJECT_STATE.md`（全局状态）。

## 快速开始
```bash
# 克隆仓库
git clone https://github.com/NeitherTourRest/lfz-programing-language.git
cd lfz-programing-language

# 构建（Rust 工具链；无第三方依赖）
cargo build

# 运行 LFZ 脚本
#   注意：.lfz 文件首行必须为 #42（LFZ 头标记），否则报 CosmosAnswerError
cargo run -- run examples/hello.lfz
# 输出: Hello, LFZ!

# 裸文件调用：lfz <file> 等价于 lfz run <file>（run 子命令保留）
cargo run -- examples/hello.lfz
# 输出: Hello, LFZ!

# 把 dist\lfz.exe 加进 PATH 后，可直接按文件名调用
lfz Hello.lfz
# 输出: Hello, LFZ!

# 构建项目内开发版（自包含，含冒烟测试；最终字节数以构建证据为准）
powershell -File scripts\build-release.ps1

# 安装到用户 PATH（scripts\install-lfz.ps1 默认 dry-run，带 PATH 备份与 -Uninstall）
powershell -File scripts\install-lfz.ps1

# 运行 Rust release 测试（最终候选522 passed）
cargo test --release

# 运行黑盒测试集（一个命令跑全部，当前 166 用例）
cargo run -- test

# 运行示例应用（排序算法可视化）
cargo run -- run app/sortviz.lfz
```

> `examples/hello.lfz` 内容：
> ```lfz
> #42
> print("Hello, LFZ!")
> ```

## Agent 数据脚本示例（T12 候选工作快照）

使用本项目构建的 `dist/lfz.exe`；已有全局安装可能仍是旧版。源码片段入口不要求 `#42`，保存为 `.lfz` 文件时仍要求首行前导。

```powershell
./dist/lfz.exe -e 'print(jsonStringify(argv()))' -- '中文' '--json' ''
# ["中文","--json",""]
'print(1 + 2)' | ./dist/lfz.exe -
# 3
New-Item -ItemType Directory -Force Temp | Out-Null
./dist/lfz.exe run app/jsonflow.lfz -- app/fixtures/t12/config.json app/fixtures/t12/input.json Temp/result.json
./dist/lfz.exe run app/sortviz.lfz -- --small 4 --big 30 --seed 7 --no-frames
```

`--json` 是解释器状态协议：stdout 为单个状态对象，程序输出转到 stderr。业务 JSON 可在普通模式调用 `print(jsonStringify(value))`。stdin 源码会读到 EOF，不能同时从同一 stdin 再读业务输入。`argv` 是本轮唯一新增可一等取值的内置；JSON 内置用作高阶回调时须由 lambda 包裹。jsonflow 输出路径须与输入、配置实际指向不同文件；其路径别名识别与非原子写入限制见应用 README。

旧 v2 首轮的题目、manifest、成绩及原始 ZIP 保持不变。开发版 skill/二进制已变化，旧 manifest 在当前开发目录中拒绝校验是预期；复现旧基线须从归档在项目 Temp 中恢复旧输入，不重写旧 manifest，不以旧 8/8 作为新版本盲测成绩。

## 许可证
本项目采用 **MIT License**，全文见 [`LICENSE`](LICENSE)。

Copyright (c) 2026 MakeChase
