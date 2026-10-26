# T12 独立验收：阶段报告

记录日期：2026-10-25 22:27（Asia/Shanghai）。验证人：verifier。**当前功能验收 PASS；R-304 效率验收未通过，T12 总体验收尚不可交付。** 本报告只绑定下述产物，不提前认证后续修复版。账号额度中断前已完成的原始验证在本次恢复后核对归档，未伪称全部测试是在恢复时间重新执行。

## 产物与范围

- release `dist/lfz.exe`，版本1.4.0，821248 B，SHA256 `fb8e5653827de1e1c3f73b81d37f21314ad41adfdc4d99f37cb8ca5f6491f47d`。
- 依据：`docs/spec/{syntax,semantics,interface-contract}.md` §16（含argv一等值/词法遮蔽补钉）及需求R-123～R-127、R-205、R-304、R-406、R-504。
- 只验证：未修改解释器、规范、正式测试、应用或指南；未执行Git写操作。全部临时材料位于项目Temp。
- 本阶段独立功能进程均实跑；性能结论复核已留存的测量/阈值与失败事实，尚未在本阶段另跑完整性能套件，等待最终修复hash后独立抽查。

## 验收清单

| 需求 | 阶段结论 | 独立证据 |
| -- | -- | -- |
| R-123 argv参数上下文 | PASS | 自编140witness中的六入口原样参数、保存别名/局部遮蔽；独立Rust程序顺序调用六个求值上下文，旧API仍空参数 |
| R-124 文件/-e/stdin | PASS | 六形态实际进程、stdin源、字面空串/中文/--json/-e/--/--help；CLI精确进程脚本91/91 |
| R-125 机器通道 | PASS | 自编业务JSON/状态通道检查；91进程验收含错误对象与行列/伪路径检查 |
| R-126 JSON解析 | PASS | 固定seed42261025差分数据，精确i64、有限f64位级往返、Unicode/控制字符；12个自定精确错误与UTF-8偏移检查 |
| R-127 确定性编码 | PASS | 输出交独立Python解析且逐层核对UTF-8键序、类型与浮点hex；共享引用/环、128与129层、方法过滤、数据引用语义检查 |
| R-205 测试与兼容 | PASS | 全Cargo517通过；官方166全部通过；旧99清单前缀及已有LFZ文件逐字节保持；索引duplicate-first/更新/clone独立库程序 |
| R-304 效率与复现 | FAIL（开放） | 同路径最终19单元仅16过原冻结线；数字循环500k旧104.259ms、新189.340ms，原阈131.497ms；dist新探针171.409ms同样超阈 |
| R-406 指南与旧证据保护 | PASS（范围见下） | 人类/AI新用法交叉核对与独立入口/JSON抽查；旧题集/报告无Git差异，原始ZIP及9项输入逐一hash匹配 |
| R-504 参数化实际应用 | PASS | 独立复跑jsonflow37项及sortviz11项自检，含输出保留、输入不变、固定参数重现、旧默认输出逐字节一致 |

## 实际执行及原始证据

从仓库根目录执行。Rust环境：`RUSTUP_HOME=Temp/toolchain/rustup`、`CARGO_HOME=Temp/toolchain/cargo`；所有进程TEMP/TMP指向`Temp/T12/verifier`。

| 命令/检查 | 结果 | 原始材料 |
| -- | -- | -- |
| `Temp/toolchain/cargo/bin/cargo.exe test` | exit0；399+48+28+12+15+15=517 passed，0 failed/ignored | [cargo-test.txt](T12-independent-evidence/cargo-test.txt) |
| `D:/app/ANACONDA/python.exe -B tests/t12_cli_check.py --lfz dist/lfz.exe --output docs/reports/T12-independent-evidence/cli.json --full-suite` | exit0；91/91；其中一次官方runner166/166，不重复合并计数 | [cli.json](T12-independent-evidence/cli.json) |
| `D:/app/ANACONDA/python.exe -B app/verify_jsonflow.py --lfz dist/lfz.exe --output docs/reports/T12-independent-evidence/jsonflow.json` | exit0；37项通过 | [jsonflow.json](T12-independent-evidence/jsonflow.json) |
| `D:/app/ANACONDA/python.exe -B app/verify_sortviz.py --lfz dist/lfz.exe --before-exe Temp/t12-perf/lfz-before.exe --output docs/reports/T12-independent-evidence/sortviz.json` | exit0；11项通过 | [sortviz.json](T12-independent-evidence/sortviz.json) |
| 自编差分/错误/引用/入口检查 | exit0；140个进程witness全通过 | [check.py](T12-independent-evidence/check.py)、[independent-witness.json](T12-independent-evidence/independent-witness.json) |
| 独立Rust库检查 | 六次求值上下文隔离与旧API默认空数组；索引first-match、更新、2000键和clone均通过 | [library.rs](T12-independent-evidence/library.rs)、[library-build.txt](T12-independent-evidence/library-build.txt)、[library-result.txt](T12-independent-evidence/library-result.txt) |

140个自编witness包含115个JSON差分输入、12个精确错误、6种入口及引用/深度等补充检查，具体分类以JSON逐项记录为准，不把组内断言另加到进程数。JSON差分使用独立Python标准库作为语义oracle，额外检查对象键UTF-8顺序与float.hex()，避开Python默认接受重复键/非有限数/代理项所造成的宽松误判。生成数据本身只含有效Unicode标量与有限数字。额外错误用例不是用实现的错误构造器计算期望。

作者指南全部示例验证与verifier抽查应区分：人类指南作者35个LFZ块+10探针、AI作者30项是作者证据；本报告没有将其冒称为verifier逐段重跑。verifier读取新指南/skill并实际运行相同公开入口与JSON使用模式，确认72内置、仅argv新增一等值、JSON高阶映射需lambda、stdin与input限制、业务JSON/CLI状态通道、循环push复杂度等描述一致。

## 冻结基线保护

- 原始v2归档`raw-inputs-submissions.zip` SHA256仍为`90a6a3e3d894e7a813b06463506d5cc540e0a4570d7b005a3fd27ccc6a776d84`。
- 对旧manifest的9项输入逐项在ZIP内匹配原始字节SHA，全部通过；记录见`independent-witness.json/frozen_v2`。
- `git diff HEAD -- docs/validation/agent-benchmark-v2 docs/reports/agent-benchmark-v2/v2-20261024-r01`为空。active skill及当前binary变化属于新版本边界；没有重写旧manifest或回算旧8/8。
- 旧`tests/cases.json`99项结构保持前缀相等；HEAD已有LFZ测试文件逐字节保持。

## 开放缺陷与回归门槛

**T12-PERF-OPEN：R-304尚未通过。** 初版不同路径比较存在条件差异，随后同一路径换旧/新版字节并预热交错测量，仍有3组超过原阈。详见[性能报告](T12-performance.md)与`benchmarks/t12/results/final-same-path.json`。runtime早期观察同hash在某路径约99ms是特定轮次控制结果，后续dist出现171.409ms；不能据前一观察宣布路径问题已解决，不能把结构体收益替代整体效率合格。

性能超阈原因仍待定位，本报告不猜测缓存、操作系统、安全软件或编译器原因，也不把同路径仍失败称作已修复。所有首测、复查、同路径数据均保留，原合格线不变。

下一步：runtime完成修复后，由perf重新按原门槛测量并给出最终hash；verifier针对最终字节复跑受影响回归、JSON/argv/应用/通道及少量性能抽查，再追加最终验收结论。本阶段不批准T12整体发布。
## 不依赖旧临时夹具的复现

先从仓库根目录构建并准备当前`dist/lfz.exe`。Python脚本会自行创建新临时输入；输出路径必须尚不存在，不会覆盖本报告原始证据。默认对照原测试保护使用固定`v1.3.0`引用，避免未来HEAD变化影响基准定义：

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T12-independent-evidence/check.py --output Temp/T12/new-independent-run.json
./docs/reports/T12-independent-evidence/run-library.ps1
```

库脚本自动创建临时目录并构建当前lib，优先项目内工具链；若项目工具链不存在则使用PATH中的Rust。无须此前的`Temp/T12/verifier`或旧编译出的探针。脚本与库源码均已保全。2026-10-25恢复时已按上述新方式再验证140/140和库检查通过，记录[independent-rerun.json](T12-independent-evidence/independent-rerun.json)、[library-rerun.txt](T12-independent-evidence/library-rerun.txt)。这是相同功能证据的复跑，不能加算为另一套280用例。
## 最终可审阅候选复验（2026-10-25 23:36）

**功能与§16.7窄作用域语义通过；效率门槛未全过。结论为可审阅候选checkpoint，非正式发布，R-304保持开放。** 前文FB8E阶段及失败记录原样保留，不能混用其hash或计数。

### 本次产物

- `dist/lfz.exe`和`target/release/lfz.exe`逐字节SHA256同为`47ed2bfe2a2694b11bd611172cfcf0000d0bfc58e3b9bd809ca95f066dd0261f`，759808 B，版本1.4.0。
- 源码加入§16.7许可的无直接声明while体空环境消除；扫描在循环外一次完成，命中直接Decl/FnDecl/StructDecl（包括不可达声明）仍走原exec_block。for/其它block/函数帧未在此优化中修改。
- Cargo release profile为获批准的`opt-level=2`，保留LTO、单codegen、strip。该profile与while优化同时存在，测量收益不得全部归因某一个改动。

### 最终功能实跑

所有证据在[T12-independent-final](T12-independent-final/)，均为本verifier实际执行，不沿用旧产物认证。

| 检查 | 结果 | 证据 |
| -- | -- | -- |
| `cargo test --release` | 404+48+28+12+15+15=522 passed，0 failed/ignored，exit0 | [cargo-test-release.txt](T12-independent-final/cargo-test-release.txt) |
| 公开CLI脚本`--full-suite` | 91/91，其中官方全量166/166，不叠加重复计数 | [cli.json](T12-independent-final/cli.json) |
| jsonflow / sortviz脚本 | 37/37及11/11；真实文件、参数、失败保留、旧默认输出一致 | [jsonflow.json](T12-independent-final/jsonflow.json)、[sortviz.json](T12-independent-final/sortviz.json) |
| 自编独立JSON/argv等witness | 140/140，绑定最终47ed字节；原9冻结输入及旧99清单仍保持 | [independent-140.json](T12-independent-final/independent-140.json) |
| 新增while旧/新差分 | 16/16程序，实际8类；stdout/stderr/退出码逐字节完全相同 | [while-differential.json](T12-independent-final/while-differential.json)、[while_compare.py](T12-independent-final/while_compare.py) |

新增差分覆盖：①无声明体闭包捕获外层可变cell；②直接let/var/fn/struct声明每轮fresh（4个程序）；③不可达直接声明；④嵌套if遮蔽、for每轮绑定和nested while；⑤`;;`可见顺序及闭包内dump；⑥break/continue/return传播；⑦空体与条件求值次数；⑧未知名赋值不隐式定义、let不可写和错误traceback。最后一类期望exit2，其余期望exit0；旧版和新版一致。这是有界语义差分回归，不冒称穷举所有程序的形式证明。

六入口/业务JSON/机器状态/应用的最终140+91+37检查也覆盖了新指南/AI模板的可复制调用模式；未对未改动的35个历史块再次全量重跑。

### 最终性能审查与限制

本次没有再选择新的计时窗口或为了凑门槛反复重跑。独立审阅`final_compare.py`、`new_features.py`、`process_measure.py`、`affinity_control.py`，并用[自编审计脚本](T12-independent-final/audit_performance.py)逐项重算原始JSON；[performance-audit.json](T12-independent-final/performance-audit.json)保全结果。

核查通过的内容：最终47ed/旧149b字节hash和长度、共同subject路径、每次换字节后固定静默5秒、预热2+正式7、旧新组次序交替、每次进程前验hash、固定输入/源码与原基线一致、全部266次旧新正式输出及63次新能力输出、7次交付路径探针记录、每组wall/user/kernel中位数、原绝对线公式未变、判断分类及paired比值、评估文件绑定原始JSON的SHA256。CPU短进程计数有15.625ms量化和0值，不能把0解释为无计算；峰值内存未测。

最终19组按原绝对线为**15通过、3环境不确定、1失败**：

- 3组同期旧版也超原线，按预定规则保留环境不确定，未将其改判通过。
- `string_map`8000项：同期旧21.871ms，新22.691ms，原线22.521ms；新版仍超线，不能四舍五入成通过。
- 明确的局部收益：结构体8000键93.431→21.198ms，约77.3%降时；数字500k104.940→90.804ms；启动16.287ms在原16.307ms线内。这些不能抵消未过组。JSON/CLI共63次正式输出均正确，但旧版没有相同JSON接口，不虚构跨版JSON加速比。

因此R-123～R-127、R-205、R-406、R-504在已声明功能范围内通过；**R-304未通过，T12整体正式发布仍不批准**。可以保存为可审阅候选checkpoint，继续以相同原门槛跟踪效率开放项；本报告不替perf宣布所有路径、所有环境或全部负载高效。

### 最终证据复现

从仓库根目录，输出使用尚不存在的新路径；旧exe可由保留的v2原始ZIP还原到项目内，也可通过`--old`指定原149b字节副本。所有临时输入由脚本自行生成：

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T12-independent-evidence/check.py --output Temp/T12/final-140-new-run.json
D:/app/ANACONDA/python.exe -B docs/reports/T12-independent-final/while_compare.py --old Temp/t12-perf/lfz-before.exe --new dist/lfz.exe --output Temp/T12/final-while-new-run.json
D:/app/ANACONDA/python.exe -B docs/reports/T12-independent-final/audit_performance.py
```

最后一条只审计冻结原始性能数据，不制造新计时窗口。其输出为本独立最终目录的性能审计JSON，语义结果可重复核对。目录manifest记录本次证据字节SHA，后续候选不得回写冒充本次测量。