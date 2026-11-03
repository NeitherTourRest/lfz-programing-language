# T19 冻结契约
## 25. T19 有界逐行与JSONL组合（2026-11-03 冻结）

root批准主体并合runtime/tool review。仅本轮T19交付后收口，不自动T20/automation。无新文法/Value/能力/context字段；原readFile全文件读取/消息/编码不变，R304OPEN。本节与T19-contract同文。

### 25.1 API、返回与校验
新增特殊内置forEachLine(callback,maxLineBytes,path)，data-last，共76内置；原同名绑定可遮蔽，不新增first-class builtin。callback(line:string,lineNo:int)返回严格bool，true继续/false早停；其它返回TypeError（BadOperands op="forEachLine",lt实际类型,rt="bool"）。
成功普通struct键序lines:int、stopped:bool，lines是实际已调用callback数，含false那行；stopped仅false回调早停为true，EOF为false。空文件{lines:0,stopped:false}。任何错误不返回部分成功值，已有用户副作用不回滚。
实参先原左到右求值；之后数量3（ArgCount name="forEachLine",n=3）→callback可调用Func→maxLineBytes int→path string→callback恰2参数→max范围1..16777216→path NUL→native全局资源预检→打开。类型沿BadOperands，期望"function"/"int"/"string"。callback须user function可接2参，argv等builtin不符2参仍先ArgCount（name沿原function名），无可调用body的Func沿NotCallable；空文件也完整预检，不打开后再等首行发现错arity。绑定self/captured沿原call_value规则，无新回调适配器。
相对path按host cwd，非callback定义module；文件名空串不额外ValueError，沿OS打开失败。无需http/process能力，沿readFile既有读取边界；不扩大或缩减旧IO权限。

### 25.2 数据口径与有界读取
只普通文件，预metadata跟随符号链接并is_file，open后File::metadata再次is_file；拒目录/设备/FIFO/socket/stdin。检查不是安全沙箱，TOCTOU可在metadata/open间替换为设备，OS open/read仍可能阻塞，不能声称消灭竞态。允许普通文件符号链接，读中修改/追加/截断沿OS观测，非快照/锁/事务。
LF唯一分隔，交付不含LF；仅有LF终止且前byte为CR时去掉一个CR，孤CR及EOF末CR保留。BOM保留、不trim/跳空行/跳注释，不应用LFZ loader ext/#42前导。
空0行；a为1行a；a+LF为1行a；LF为1行空串；LF+LF为2空行；a+CR+LF+b+CR为a及b+CR。尾LF不造额外末空行。NUL在行内容合法，不与path NUL混淆。
maxLineBytes计LF前raw bytes，包含随后剥离的CR，不含LF；恰N允许、N+1立即TooLong，不读完整坏行、不回调坏行。a+CR+LF需预算2；a+LF需1。UTF8字符按bytes，拼好有界行/去终止符后严格UTF8解码，不lossy、跨块多字节必须合法。EOF末行同预算。非法UTF8在构造回调string前终止全流。
lineNo从1到i64::MAX，含空行；准备再交付一行会溢出时报NumberOverflow，无wrap/float。callback false后不再解析/解码下一行，后面坏UTF8/超长不影响早停；有限缓冲允许预读后续bytes但不处理/再回调。
固定8KiB BufReader（或等价固定缓冲），fill_buf/consume逐块找LF，有界复制；当前行Vec的length/capacity均<=maxLineBytes，不先预分配16MiB；按实际需要有界reserve，禁止几何扩容超过cap。加固定8KiB输入缓冲；可有String/返回包装的明确有界开销，不得无界read_line/read_until/read_to_end/collect全文件。发现第N+1 byte用现块观察，不额外累计坏行。成功String从Vec转移，reader不保留历史行。
引擎空间O(当前最长行+8KiB)，用户把行/JSON记录保进数组仍可O(文件)；不宣称任意脚本恒定内存。嵌套forEachLine各自独立reader/缓冲，回调depth受原Recursion，文件不放Env/closure/Value且无逃逸handle。

### 25.3 错误、JSONL、预算与来源
core唯一error.rs新增ValueMsg：
LineLimitRange（无字段）：“逐行读取 maxLineBytes 必须在 1..16777216 范围内”
LinePathNul（无字段）：“逐行读取路径不能包含 NUL”
LineTooLong{line:i64,limit:usize}：“第 {line} 行超过逐行字节上限 {limit}”
LineUtf8{line:i64}：“第 {line} 行不是合法 UTF-8”
LineNumberOverflow（无字段）：“逐行读取行号超出 i64 范围”
均ValueError，可外层attempt捕获整次读取失败；不新增第14错误类。非普通文件/metadata/open/read失败Io，精确msg“无法逐行读取：{path}”，span调用点Some；保底后端request校验失败也映射该Io，不泄露随机内部消息。readFile旧Io文本/非法UTF8类不改，新接口坏UTF8 ValueError是明确新契约。
坏UTF8/超长不能由当前callback恢复（该行未交付）；外包attempt不会恢复offset继续流，但此前用户副作用保留。JSONL组合在callback内部attempt(()=>jsonParse(line))，JSON坏行/空行/业务处理策略由调用者决定；不隐式JSON解析或跳行。
native检查仅cancel Acquire→deadline now>=deadline，不扣/检查remaining，不复用现Interp.poll（其扣AST步）。reader在open前、每块fill_buf前（含已有缓冲块）、每次返回行/EOF前检查；runtime在回调前和正常回调返回后（含false）检查。global终止ExecutionLimitError原reason/message不可attempt吞。参数表达式/回调真实stmt/expr仍单会话扣步；callback剩余0下一AST终止，不按文件bytes/块/行额外加步。
已确定读取/解码/callback错误锁存并drop，不在清理时追加poll覆盖主错误。false回调正常返回后如果观测取消，取消优先早停成功。同步metadata/open/read无法被这些poll硬中断，不开worker/detach/强杀线程。
callback一律call_value沿原home globals/captured/self/source/trace，不能新Interp或直接eval Body。native错误映射note_error_span，数据lineNo仅消息/实参，不把数据文件当LFZ源码Span/source。callback错误原位置、跨module真实file保持；原attempt成功/可恢复失败恢复trace/source但不恢复预算/用户效果。不持RefCell借用跨用户回调。

### 25.4 纯std reader ABI与统一关闭（tooling唯一）
新src/line_reader.rs，lib.rs pub(crate) mod line_reader由tooling唯一注册。纯std，无crate/平台FFI/第三diagnostic sidecar。pub(crate)字段/类型：
    LineControl {deadline:Option<Instant>,cancel:Option<Arc<AtomicBool>>}
    ReadLine {number:i64,text:String}
    LineReadError {Cancelled,Deadline,TooLong{line:i64,limit:usize},
                   InvalidUtf8{line:i64},NumberOverflow,NotRegular,
                   Io{operation:String,message:String}}
LineControl Clone/Debug，ReadLine/Error Debug；LineReader私有File/缓冲/计数，不接R/Value/Rc/Env/Invoke。
    pub(crate) fn LineReader::open(path:&str,max_line_bytes:usize,
                                   control:&LineControl)->Result<Self,LineReadError>
    pub(crate) fn next_line(&mut self,control:&LineControl)
        ->Result<Option<ReadLine>,LineReadError>
    pub(crate) fn finish(self)
方法finish返回()，仅消费owner并drop reader/File，不伪装可观测close结果；Drop仍兜底。open防御非法cap/NUL等返回Io operation="请求校验"，不panic；其余操作名metadata/open/read，message OS原错供unit，不改变对外固定Io文本。read/decode长度检查后、返回成功前native poll；同一已观察失败不被随后cleanup检查改写。NumberOverflow仅在确有下一行要交付时触发，i64::MAX行后EOF可正常完成；测试用安全内部计数注入，不生成天量文件。
runtime调用open后将整个循环封装为一个outcome，再统一finish/drop之后才返回该outcome；不因?从有reader作用域跳过可审查的关闭路径，RAII保证其余所有提前退出也释放。callback false/EOF/非bool/业务error/ExecutionLimit/read/UTF8/TooLong均覆盖；外层attempt返回时文件已释放。std Drop不提供OS close错误回传，不能宣称报告了close异常；sync_all不是close，不引T18 close FFI或fake diagnostics。

### 25.5 owner与可复验门禁
runtime唯一evaluator特殊分派/参数预检/调用与映射/native-only检查，builtins76名字及无上下文调用：无上下文builtins::call/call_with遇forEachLine返回TypeError BadOperands(op="forEachLine",lt="function",rt="evaluator context")，不得偷偷启动无预算读取。原BuiltinFn/Invoke/Context/Value公开shape不改。
core唯一error.rs五个ValueMsg及消息验证；tooling唯一line_reader/lib注册及CLI集成回归、规模/资源probe；test/app/docs/AI/perf/verifier按宪法。reader与evaluator不两边复制分行算法。先owner接口review后root GO，唯一编译者另定；无新CLI flag。
边界测空/CRLF/孤CR/末行/BOM/NUL/空白/UTF8跨8KiB/非法跨块/精确N-N+1/16MiB；错arity空文件也拒且无open；false不检查后坏行；callback非bool/业务错/全局终止、嵌套流、同文件重入/修改、跨module闭包/self/默认值调用、JSON坏行连续失败后成功、fault来源不残留。
关闭证据Windows share_mode(0)排他重开+warmup后同进程handle plateau，controlled reader Drop spy覆盖所有outcome；删除/rename单独不能证明关闭。注入metadata/open/read失败仍真RAII，禁止人为泄漏或虚构close错误可见。非Windows如无实测明确未验，不把标准库可编译当实测。
规模独立生成oracle含有效/坏JSON/Unicode/不同line长，固定种子/计数/校验和，至少10/100/300MiB三档同分布；callback只计数不collect。新进程量peak WS/RSS、walltime/吞吐/bytes、最终exe hash、原始全部样本，生成不计处理窗，记录缓存/硬件/预热与重复方法。100MiB结果oracle与readFile同结果安全规模对照；不用源码O(line)代峰值实测，不以单档耗时声称复杂度。T19工作负载测量不等于原19项/R304达标。旧T18-B/T17/C/apps/CLI全回归、独立验收后交付，仅本轮收口。
