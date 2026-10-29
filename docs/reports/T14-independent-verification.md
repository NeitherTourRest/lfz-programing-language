# T14 check 最终独立验收

日期：2026-10-29。责任人：verifier。结论：当前83f56候选的check功能、非执行边界及兼容回归PASS。R304仍OPEN，未进行T14性能测量，不批准整体正式发布。

## 指纹及执行证据

dist/target均774656B，SHA256 `83f56a194e216862c45377f4c3971b085744a7ba2363163e1edac6af5e85ac79`。本角色直接运行7个release测试harness，无重新编译；全部542 passed、0 failed/ignored。原始7份harness输出与二进制hash在[harnesses.json](T14-independent-evidence/harnesses.json)。

| 实际检查 | 结果 | 证据 |
| -- | -- | -- |
| 新check公开进程检查 | 117/117 | [check117.json](T14-independent-evidence/check117.json) |
| 原CLI及官方runner | 91/91，内含166/166官方测试，不重复叠加计数 | [cli91.json](T14-independent-evidence/cli91.json) |
| jsonflow / sortviz | 37/37、11/11 | [jsonflow.json](T14-independent-evidence/jsonflow.json)、[sortviz.json](T14-independent-evidence/sortviz.json) |
| 旧独立JSON/argv见证 | 140/140，旧9冻结输入及99清单条目保持 | [independent140.json](T14-independent-evidence/independent140.json) |
| 本轮独立进程见证 | 57/57 | [witnesses.json](T14-independent-evidence/witnesses.json)、[verify.py](T14-independent-evidence/verify.py) |

既有T12脚本硬编码的历史日期/schema不代表本轮执行日期；本报告日期2026-10-29及新产物hash为本轮定位依据。自编probe最初误用LFZ不支持的中文标识符，检查正确返回SyntaxError；修正为ASCII变量及Unicode字符串后全部通过，该问题属于验证输入假设而非产品缺陷。首次使用公开脚本时也误传--lfz参数，按其--binary接口修正；没有改变产品或已有测试。

## 非执行与位置检查

独立57见证包括三来源（文件/-e/stdin）、正常/JSON双模式：脚本同时包含writeFile/appendFile、print/eprint、dump/input时，现存二进制哨兵文件字节不变，缺失目标不创建且无程序输出；while true与递归入口在8秒进程超时内完成检查而不求值；未定义名、类型错、除零、assert失败和不存在readFile均只检查语法通过。带前置写语句的晚语法错、不执行分支中的语法错均被拒绝而无副作用。

语法负例另通过原run前端得到参考对象，check去掉mode/executed后所有错误字段（含file/line/col/traceback）与原前端精确相同。检查成功只有ok/mode/executed三键，executed恒false；合法JSON检查stderr空。CODE恰为--json/--/check/空串被保护；CODE外--、缺源、缺CODE、check --help按既有usage风格stderr文本exit2，stdout空，不伪造LFZ JSON错误。117正式检查脚本还覆盖loader前导/BOM/UTF8/CRLF、深度界限及更多CLI组合。

## 源码路径独立审阅

审阅src/cli.rs::execute的Check分支、check_input、check_loaded_on_stack以及option来源处理。Check分支独立进入check_input，load_file/load_source及stdin UTF8读取均位于on_eval_stack闭包，随后仅lex/parse；成功Program在该线程显式drop，主线程接收CaseEval状态或原Loaded/error诊断。该路径未调用eval_module、Interp、run执行入口或内置执行，不建立TracedRun；JSON复用push_error_fields。仅使用evaluator模块的大栈包装函数不代表LFZ求值。对非执行的判断同时依据调用路径审阅与实际IO/输出/终止见证，而非关键词搜索。

此功能只验证加载及现有前端规则，不保证名字、类型、arity、IO资源或运行终止正确，也不是零IO或资源安全沙箱。T13两项生产优化已由对应负责人撤回、8新增回归测试保留；本验收对象是T14当前指纹，不把T13失败性能数据当成本产物测量。

## 复现

从项目根运行（公开脚本要求输出为尚不存在的新项目内路径）：

```powershell
D:/app/ANACONDA/python.exe -B tests/t14_cli_check.py --binary dist/lfz.exe --output Temp/T14-verifier/new-check-run.json
D:/app/ANACONDA/python.exe -B tests/t12_cli_check.py --lfz dist/lfz.exe --output Temp/T14-verifier/new-cli-run.json --full-suite
D:/app/ANACONDA/python.exe -B docs/reports/T12-independent-evidence/check.py --output Temp/T14-verifier/new-witness-run.json
```

独立verify.py会在本轮独立目录写派生输出，审阅原证据后若复跑请复制脚本到项目Temp并指定新的E目录。目录* -text与manifest保留原始字节。无源码/官方测试修复、无Git、无panic/abort压测；性能R304与正式发布继续留待主调度处理。
