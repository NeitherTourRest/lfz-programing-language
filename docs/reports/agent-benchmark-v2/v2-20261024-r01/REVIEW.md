# LFZ Agent Benchmark v2 正式首轮独立验收

run-id：**v2-20261024-r01**；verifier；2026-10-24 18:46（Asia/Shanghai）。

**正式结果：8/8题通过，131/131功能用例通过；8份源码约束审查PASS，8份协议审查PASS。** 本次是冻结新八题的首个v2基线，不与历史2/8计算改善百分比。不是工具自测，也不是对任意任务能力的保证。

## 结果与绑定

| 题 | 功能 | 源码约束 | 协议 | 整题 | 学生LFZ自调次数 | 首次自调 |
|---|---|---|---|---|---|---|
| B01 表达式解释器 | 16/16 | PASS | PASS | PASS | 1 | exit0 |
| B02 CSV解析 | 18/18 | PASS | PASS | PASS | 1 | exit0 |
| B03 碰撞桶哈希表 | 15/15 | PASS | PASS | PASS | 1 | exit0 |
| B04 BFS | 16/16 | PASS | PASS | PASS | 1 | exit0 |
| B05 二维DP背包 | 16/16 | PASS | PASS | PASS | 1 | exit0 |
| B06 账户账本 | 17/17 | PASS | PASS | PASS | 1 | exit0 |
| B07 倒排索引 | 17/17 | PASS | PASS | PASS | 1 | exit0 |
| B08 文件转换 | 16/16 | PASS | PASS | PASS | 2 | exit0 |

- [functional-results.json](functional-results.json)：冻结运行器原始功能报告，未改写其REVIEW_REQUIRED/NOT_CERTIFIED占位字段；SHA256 `84a017c05d2451c79af699405588b06870eec47b56855adb1faac5e71a4cf2cb`。
- [manual-review.json](manual-review.json)：另立独立人工记录，绑定上述报告、manifest及逐题学生源码hash；SHA256 `edb12042b4426e6519597b199568eb0290705629b7ea0a7fa52747984e1c74e9`。
- [certified-results.json](certified-results.json)：按功能PASS＋源码PASS＋协议PASS且hash相符合并，分母恒8，certified_pass_tasks=8。未以自动功能输出代替人工审核。
- 冻结manifest SHA256 `f2046eadca7c9f55376cbac000ccd53b25958298d10e2fc176ed0a14867c2a3e`；本轮run.py SHA256 `5a698248db184d091bd0a41e46b861f241e7f8f2a1051cef640640671eb9e862`，cases.json SHA256 `919cfe6905ef4f4bc003d0321fe4398aab631be0ac8a0873f03c2761af6f22cd`，解释器SHA256 `149b184e442ffbf250590431fa0aeffab401eac526faf60b4b435f620725d5ab`。

## 功能执行及可复现证据

八题收到并封存后，从项目根执行：

~~~powershell
$env:PYTHONIOENCODING='utf-8'
python -B docs/validation/agent-benchmark-v2/run.py --solutions docs/reports/agent-benchmark-v2/v2-20261024-r01/sealed --lfz target/release/lfz.exe --output docs/reports/agent-benchmark-v2/v2-20261024-r01/functional-results.json
~~~

实际退出0，stdout原文：

~~~json
{"functional_pass_tasks":8,"denominator":8,"source_review":"REVIEW_REQUIRED","protocol_review":"REVIEW_REQUIRED","certified_pass_tasks":null}
~~~

131功能case共147次验收进程（B08每case两次），全部成功，0超时，最大单进程0.080458秒。源码、输入、期望、错误通道和B08原始文件hash均在原报告。原始工作目录从Temp复制到 [runner-evidence/](runner-evidence/)，187文件逐字节一致；[functional-integrity.json](review-evidence/functional-integrity.json)记录一致性、提交源码hash匹配及统计。JSON保留执行时Temp路径，归档可用相同Bxx/case-id相对路径在runner-evidence内查阅；不要求旧Temp仍存在。

重新运行须另选尚不存在的输出报告名，不覆盖本轮结果；须使用匹配冻结manifest的构建。Python启动时本机定位诊断不影响实际执行，判定依据是真实exit与结果字节。

## 源码人工约束审查

逐份完整阅读封存solution，无学生反馈、无修补答案、无修改题目或验收；定位证据详见 [source-review.md](review-evidence/source-review.md)。

- B01自行递归下降，expr/term/unary分层、循环左结合、div向下整除；没有固定文本答案表。
- B02 quoted字符状态机，支持引号转义、空/尾字段与CRLF，按实际文本构造数组。
- B03显式8个数组桶；Unicode标量ord求和mod8、完整字符串key比较，局部重置size与桶，非动态struct替代。
- B04预分配FIFO队列，按上右下左扩展，首次入队即写前驱；没有grid/start/goal写操作。
- B05二维DP按原下标加入，候选严格大于才选，末项反向回溯，最终下标升序。
- B06先创建所有涉及账户，再判足额；拒绝不扣款，足额才扣加，自转账处理正确，每次空账本。
- B07原文ASCII筛选后才lower；先构造token与位置索引，query阶段不重扫全文。
- B08真实readFile，按LF分行并trim/int，输出固定LF，单次writeFile覆盖，不写inputPath。

八份最终solution均仅函数定义及注释，无顶层input、演示IO或提前退出；未见硬编码验收结果或调用其他语言/程序求解。这是封存源码内容的人工PASS，另有协议工具轨迹核对，二者分别判定。

## 协议审查：依据与边界

### 留存材料

独立运行 `python -B Temp/verifier-formal-r01/check_protocol.py`；必要脚本已归档 [check_protocol.py](review-evidence/check_protocol.py)。[material-check.json](review-evidence/material-check.json)逐题核对：

- 交卷solution、seal、run-metadata、冻结运行器submitted-snapshot的SHA256一致；本题TASK、派发prompt、四skill、解释器hash匹配。
- 8题均在20分钟保守预算内，received相对started约205.65–247.72秒；首批起点取created，真实派发区间与DateTimeOffset元数据修正说明保留，未延长预算。
- B01..B07各1次LFZ执行、B08两次，均≤5；每次命令、UTC开始结束、stdout/stderr、退出码与不可覆盖快照齐备；快照mtime在执行前，快照等于最终源码，自编测试包含该快照函数。
- 所有首个学生自调exit0；这只表示学生自选测试的过程指标8/8，不表示其自测覆盖冻结验收，也不推出任意程序可首写无误。B08第二次补充边界测试，源码快照未变。

初始模型unknown合法保留为历史状态；收到会话证据后核验8个学生turn_context均为 **gpt-6.1-sol，reasoning effort medium**。结论来自留存模型字段，不靠名称猜测，不外推相同模型在其他预算或任务上的表现。

### 已记录工具轨迹

在没有工具轨迹时，隔离/完整调用次数只能暂定UNKNOWN，不能靠功能通过推定。调度者后来提供精确8个学生session的已记录工具调用/结果导出后，verifier逐项审阅了实际命令并独立核对，才给协议PASS：

- [tool-traces/index.json](tool-traces/index.json)及B01..B08.json包含8个不同session、相同父thread、各自agent_path、模型上下文、完成时间及原始session文件hash。
- [tool-trace-review.json](review-evidence/tool-trace-review.json)实测33次工具调用与33次结果call-id逐一配对，7题各4/4、B08为5/5；全部是文件读写/日志包装的functions.exec，无其他工具、网络或委派调用。
- 完整命令索引 [tool-command-index.json](review-evidence/tool-command-index.json)保留原命令及省略here-string后的可读索引。这里的工具调用数不等于LFZ执行数：实际 `& .\lfz.exe run ...` 共9次，逐题1/1/1/1/1/1/1/2，与attempts完全一致，无额外LFZ探针/版本查询。
- 所有exec_command显式workdir均为自己的Bxx目录。读取仅本题TASK、skill、自己源码/日志；写入仅自己solution、测试、快照、报告、日志和B08自测夹具。未见父目录/spec/验收集/参考解/其他题读取，未见联网、外部语言代算或再委派。
- 源码与LFZ自编测试也已审查：文件操作仅B08自身本题夹具，未发现通过执行测试间接读取验收/外部代码。PowerShell用于文件与执行记录，求解发生在LFZ内。

实际复核命令 `python -B Temp/verifier-formal-r01/audit_traces.py` 输出 `PASS 33 33 9`；脚本已归档 [audit_traces.py](review-evidence/audit_traces.py)。raw工具输入/输出与源session hash保留供复核，不包含私有推理、系统/developer内容或其他会话。

**协议PASS的范围是冻结要求的“无历史分题派发＋提示词约束＋已记录工具动作审计”。共享文件系统没有操作系统访问控制，工具轨迹不是OS文件访问日志；不能据此宣称强隔离、证明一切未记录系统行为或排除模型训练先验。** 此限制事前在任务模板中已声明，没有在看到答案后降低协议要求。现有记录足以核对本轮已记录操作遵守允许读写范围和预算，故不再把这一范围内的协议项目留作UNKNOWN。

## 人工记录格式及最终合并

独立使用已安装jsonschema的Draft202012Validator校验manual-review.json与冻结包manual-review.schema.json，输出：`MANUAL_SCHEMA_PASS 8 unique task IDs`。另逐题核对B01..B08恰好一次、源码hash等于功能case的source_sha256、报告hash和manifest一致。schema验证本身不判断算法或隔离，人工判断依据在上文与辅助证据中。

[finalize_manual.py](review-evidence/finalize_manual.py)保留hash绑定和合并计算；其中PASS输入来自本报告人工审查，脚本不代替人工推断。功能结果与人工记录均PASS且绑定相符，因此[certified-results.json](certified-results.json)记录8/8。

## 未修改事项与开放范围

本角色只执行独立验收、读封存产物、写报告及辅助证据；未修改任何学生代码/日志/快照、未把冻结用例反馈给学生、未修复失败以重算分数、未改九份冻结输入。正式只评分这一轮封存版本。没有新增FAIL、未闭合阻塞或因UNKNOWN而剔除分母的题。

本轮证明了该模型在本预算、skill、解释器和新固定八题上的一次成功基线。只有在后续固定同题、记录模型与输入变更的独立轮次才能进行受控比较；本轮不用于宣称历史2/8提升为8/8。