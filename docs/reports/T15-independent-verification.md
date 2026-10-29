# T15 attempt 独立验收

日期：2026-10-29。责任人：verifier。结论：当前候选功能与有界恢复语义PASS；R304仍OPEN，未进行T15性能测量，不批准整体正式发布。

dist/target均782848B，SHA256 `1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c`。直接运行7个release测试harness合计549通过，无编译；原始stdout、stderr和harness hash见[harnesses.json](T15-independent-evidence/harnesses.json)。

| 实际检查 | 结果 | 原始证据 |
| -- | -- | -- |
| T15精确进程 | 13/13 | attempt13.json |
| T14非执行check | 117/117 | check117.json |
| 旧CLI与官方runner | 91/91，其中官方178/178，不重复叠加 | cli91.json |
| jsonflow/sortviz | 37/37、11/11 | jsonflow.json、sortviz.json |
| jsonbatch | 38/38 | jsonbatch.json |
| 旧独立JSON/argv | 140/140 | independent140.json |
| 自编独立进程 | 38/38 | witnesses.py、witnesses.json |

以上JSON均位于T15-independent-evidence；commands.json保留实跑命令/退出码及输出。复用的历史脚本可能含历史schema/date标签；本轮实际执行日期为2026-10-29、binary hash以各结果和本报告为准，未重写旧脚本或旧证据。旧131条manifest前缀和旧v2冻结输入由既有独立脚本实际核对，官方从166增至178对应4组新LFZ与8个负例。

自编38进程包含human/JSON两模式：成功nil/返回函数、嵌套捕获校验错误、容器别名、失败前写出的逃逸闭包及函数ID不串、修改error.traceback不影响后续解释器状态、12000次恢复后正常递归、argv/管道/遮蔽/self、JSON深度输入ValueError可恢复、整数Overflow可恢复、实参求值仅一次；外层非函数/arity/实参求值错误直接传播，容量及嵌套容量错误不捕获；后续未捕获NameError只有module/fresh帧，没有旧gone失败帧。完整源和双流均保全，全部期望校验通过。

## 实现路径与安全资源见证

只读审阅evaluator的eval_call、attempt_call、attempt_recoverable及原call_user：变量遮蔽优先、实参原eval_args只求一次；校验在边界外；错误记录取故障trace后再恢复完整trace/span/depth；非捕获Err原样返回；func_names/next_func_id及用户状态未回滚。白名单按LzError variant匹配，IntegerOutOfRange可恢复、Capacity不可，未用消息字符串猜类，未使用catch_unwind。lookup/HOF旧ABI保持，attempt只通过Interp特殊路由。

直接执行的549单测包含t15_nonrecoverable_classification_and_preserved_state和t15_restores_complete_callers_span_on_success_and_failure：前者构造Recursion/Syntax/Cosmos/Capacity枚举证明不可捕获，并将Interp.depth置现有上限，调用实际attempt_call得到受控RecursionError且state保持；后者成功和失败均验证完整caller span及depth恢复。此为安全合成状态验证，不宣称真实压到物理栈上限，未故意制造abort、OOM或崩溃转储。

只授权同步的原3项元数据测试保留严格73名/去重/特殊attempt并集，普通65+高阶7仍72、HOF7原断言和旧行为检查均保留。未删除旧测试。check_input路径仍只load/lex/parse并在on_eval_stack内drop Program，不调用求值；check117回归实跑通过。

## 限制与交接

本报告为有界独立验收，不是所有LFZ程序形式证明，也不宣称跨平台/全部负载效率。无新编译、计时、产品代码修改、Git操作。人/AI指南独立作者实跑由各自角色报告提供，后续变动须另行核验。所有临时均在项目Temp，证据原字节保留。R304开放且没有T15性能窗口，不能用T13或T14读数替代当前产物。
