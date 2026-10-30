# T17 公开进程黑盒

最终结论：2026-10-30，同最终1946ab9c/943104B完整许可运行 **134/134 PASS**。准备、首轮失败、oracle修正及架构裁定的完整证据如下，历史记录未改写。

2026-10-30，test-engineer。读取三 spec §22、T17-design、状态/看板与 runtime 交接。当前 **91 例准备完成，T17 功能尚未实跑**，不得用 C 旧版或库阶段成绩称 T17 CLI 通过。

新脚本 `tests/t17_cli_limits.py` 不导入产品代码、不编译，不改旧测试和官方计数。所有 fixtures 在 `Temp/T17-public`，每个真实进程外部 4 秒 watchdog；超时记失败及原始异常，避免无限循环挂住验收。只有限额用例运行循环，旧 C 红测只运行有限表达式 1。

| 范围 | 固定 oracle |
| -- | -- |
| 精确节点预算 | 1 的 0/1 失败、2 成功；let x=1 换行 x 的 3 失败、4 成功，各两次；空程序 0 成功；u64 最大不 wrap、前导零接受 |
| 循环/递归与七 HOF | 受控 StepBudget；map/filter/reduce/sortBy/minBy/maxBy/each |
| attempt | 嵌套无限体不可吞 ExecutionLimit；连续业务失败最终预算耗尽，不运行无限不限额路径 |
| CLI 严格值 | 两项分别测试空、符号、空白、下划线、小数、非 ASCII、溢出；缺值、后接 --、重复、等号拒绝、左到右优先 |
| 适用命令 | check/test/help/version/--help/--version 拒限额 |
| CODE/argv | -e 紧邻选项形状 CODE 保持源码；check 同样保护；选项前后重排与 argv 原样保留 |
| 来源 | 裸文件/run 文件/stdin/run stdin/run -e |
| 模块共享预算 | 叶表达式 2 + 入口 import 1 + 入口表达式 2 = 5，4 失败/5 成功；1 步依赖错误定位 leaf；导出函数/默认值定位真实定义文件 |
| deadline | 0 有节点失败、空成功；优先于 0 步；远未来有限程序成功；10ms 无限空循环最终 Deadline，外部 4s 宽容 watchdog，不断言毫秒硬响应；语法错误先于限额 |

AST 检查实际 PASS。prepare-only 实际 exit0，最终 `prepared-v2.json` cases_total=91、executed=false。最初准备 88 后补齐三种 HOF 为 91，两份计划均保存。

真实旧版红命令：

```text
D:/app/ANACONDA/python.exe -B tests/t17_cli_limits.py --binary Temp/T16-C-release/lfz-C-cdedc9.exe --expected-sha cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006 --output Temp/T17-public/C-feature-red.json --red-only
```

结果：0/1，exit1；C binary 902656B，LFZ exit2/stdout空/usage“裸文件调用只接受一个 <file> 参数”，未支持 `--max-steps`。这是缺功能红证据，不是 T17 失败结论。脚本补 HOF 前的实际指纹保留在 red，不改写旧结果。证据已复制至 `T17-blackbox-evidence/`，manifest 及 `* -text` 防转换。

收到最终 T17 hash 后，使用新 output 路径完整运行；脚本逐进程及末尾核 executable SHA，保留 args/stdin/exit/双通道/检查和异常，禁止覆盖已有输出。模块 canonicalize 环境限制由 team-lead 协调许可运行，不修改产品规避身份契约。

边界：CLI 不提供 cancel token，预置/并发取消、attempt 精确余量不回滚、宿主图不变量及执行清理细节由 runtime unit 与独立验收补足。本套连续失败最终耗尽不是精确余量恢复证明。input/readFile/native 内部没有 poll，阻塞 IO 不承诺在 deadline 立即返回；不制造阻塞管道来宣称硬超时。暂无平台 Instant 最大值硬编码 oracle。diamond 单次初始化的旧 C 见证保留，T17 本套聚焦共享精确总量与真实来源。R304 OPEN，不宣称零成本或效率提升，不正式发布。

## 2026-10-30 13:25 等号契约补钉

依据§22最新flag族边界，将尚未T17实跑的准备矩阵由91扩为121：替换两项宽泛等号拒绝为精确消息首行，补空值/合法数/任意含等号文本，以及--help/--version；合法赋值后等号形式仍报等号错误，不误判重复。新增CODE/check CODE保护、--后argv保持，--max-stepsX=2/--timeout-msX=2不得匹配新等号错误。所有新增usage检查精确首行为“lfz: ”加冻结消息。仅等号外旧未知选项检查沿原usage类别，不发明其消息。

AST PASS；prepare-only exit0，cases_total121/executed:false，实际没有运行任何LFZ候选。prepared-v3-equals.json和manifest-v3-equals.json保全本次指纹；原88/91计划、manifest、C红测不改。等待最终T17 hash；620修前结果不作为本套最终验收。以上是未验收新测试的契约跟进，不修改旧已验行为测试。

## 2026-10-30 13:37 首轮与oracle修正实跑

锁最终target/release/lfz.exe 943104B/SHA1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45。默认首完整105/121；同产物许可运行110/121，五项图路径权限失败消失。两份原始结果与原脚本script-first.txt完整保留（原SHA dd7828b7…）。

7项属于测试作者oracle错误：`--max-steps`/`--timeout-ms`/`--json`作为CODE实际是双负号合法表达式，因此run为NameError、check成功，不是SyntaxError；空for体没有逐轮stmt/expr，25步有限空for应成功。新测试保留空for成功并增加body i的受控预算失败，修正后122例许可实跑118/122。

剩余4项是裸help/version+两限额：实际按文件入口产生IOError，没有冻结非执行命令usage。已向team-lead提交裁决（旧CLI公开入口为--help/--version）；未删除、放宽或掩盖四失败。其余118完整通过。当前不宣称整套通过。来源hash验后逐一匹配tooling final-manifest，记录manifest-run.json。没有编译/改产品/旧测试或Git。

## 2026-10-30 13:46 架构裁定后最终通过

三spec §22及T17-design已明确：help/version指--help/-h、--version/-V，裸拼写沿原文件入口规则。此前4失败是新测试错误预期，产品不需修改。本次保留4裸路径用例，改用JSON断言IOError；补齐短长合法命令两flag精确拒绝。新增真实同名help/version文件（内容1），经显式run加载非.lfz，分别验证0步失败、2步成功、0ms Deadline、60000ms成功。裸入口仍按扩展规则IOError，没有把同名文件误当帮助别名。

最终完整命令（许可普通文件权限）：

```text
D:/app/ANACONDA/python.exe -B tests/t17_cli_limits.py --binary target/release/lfz.exe --expected-sha 1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45 --output docs/reports/T17-blackbox-evidence/final-adjudicated-permitted.json
```

结果 **134/134 PASS，exit0**；943104B、SHA如上，脚本dd5f8fb1744f92acd2434c365bd5f288c530d1c6f9afd9afbf673ed8c1d52ed0。二进制所有轮次一致，没有产品修复或重新编译。验后源码逐项匹配tooling最终manifest。manifest-final封存本次原始结果、所有前轮105/121、110/121、118/122、旧脚本及准备证据，未覆盖失败。此次调整只涉及T17新oracle，未动旧测试或正式178计数。

公开功能通过不替代独立verifier、应用、指南、宿主token与安全合成测试；精确attempt余量细节仍由runtime unit补足。无性能计时，R304继续OPEN。
