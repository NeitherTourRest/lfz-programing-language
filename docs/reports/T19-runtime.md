# T19运行时源码交接
2026-11-03，按冻结§25/T19-contract实施，仅evaluator/builtins及所属unit，未编译、未执行新增测试。

forEachLine(callback,maxLineBytes,path)特殊派发，名字遮蔽优先/非first-class；数量→Func/int/string→可调用body及arity2→cap范围→路径NUL→native资源预检→open。空文件仍检查callback。argv回调按其0参数拒绝；无用户body的占位function NotCallable。无上下文builtins调用固定TypeError(function/evaluator context)。76注册只追加名及精确计数，原表72断言及旧行为保留。

reader使用§25 owned LineReader/LineControl，无分行算法副本。open成功后闭包求整个循环outcome，再统一reader.finish消费owner后返回；所有EOF/false/回调非bool/业务错误/全局终止/read错误均走该路径，RAII仅兜底。没有第三诊断或FFI，无OS close错误回传声明；旧readFile未改。

native单独cancel Acquire→deadline检查，不用会扣步的Interp.poll，也不检查remaining。回调前/正常返回后（含false）检查；reader负责open/块/EOF前后检查。已观察错误不清理时重poll覆盖。callback一律原call_value/call_user，保持home globals/capture/self/trace来源及原预算；lines为实际reader行号，false那行计入，结果键序lines/stopped。参数和实际回调AST仍按原单session预算。

LineReadError映射五ValueMsg或全局ExecutionLimit；NotRegular/Io固定“无法逐行读取：{path}”调用span，note_error_span在native来源恢复前记录。数据文件不伪造LFZ source。外层attempt不能吞终止，局部JSON恢复由callback内现jsonParse/attempt组合。

新增5项unit源码：native remaining0不扣/不检查；false跳过后续坏UTF8；JSONL坏行后续成功及错误outcome关闭；空文件/缺文件错arity先检查；真实跨线程取消关闭。Windows每次eval返回后、宿主仍存活时share_mode(0)排他重开后才删fixture，覆盖正常/false/非bool/业务错/步数终止/Cancelled；不是CLI进程退出后再看文件的伪关闭证据。取消线程以callback writeFile marker作可观测checkpoint，yield观测有5秒上限，host deadline10秒兜底，不纯sleep、不真实abort；运行结果必须Cancelled且checkpoint已见。非Windows分支只有删除，不将其称FD关闭证明，平台reader Drop spy及实测由tooling补齐。

Temp/T19-runtime/preimplementation-search.txt为新符号静态缺失记录，非编译红证据；按调度未运行cargo。evaluator SHA B0A8CEE3F9C74CB29BC73326CD51EE8A1D5670F430F3A39AA71386B980DFED5E；builtins E05D74D076CB33928E17A7A07944DB15C789A9455DA67A58F056C5CE93F97A6D。git diff --check通过，换行提示保留。已停止写源/编译交tooling唯一统一release；测试成功与跨平台/规模内存吞吐均待实际验收，不套旧成绩。

本轮仅T19收口，不T20/automation；R304OPEN，无性能结论。
