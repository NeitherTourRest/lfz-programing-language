# T19运行时可行性评审（提案，未冻结）
2026-11-03，runtime-dev。基线c2a3d39，完整启动读全局入口/看板/自身状态/NEXT并审现代码。只读源码，不编译、不改旧测试、不Git；范围仅T19逐行/JSONL，不T20/automation。

## 代码事实与最小接口建议
NEXT-ITERATIONS的T19明确“可释放资源逐行读取与JSONL流水线”，不引入生成器/async。builtins.rs:1264的readFile使用fs::read再String::from_utf8，现有O(文件大小)及消息必须保持；不能偷偷改readFile为不同返回或规范化换行。
Invoke是&mut dyn FnMut(&Value,&[Value],Span)->R<Value>（builtins.rs:201），仅含调用能力，不含取消/deadline/宿主诊断能力。evaluator::call_value→call_user已有home module/captured cells/self/trace逻辑；不要在文件reader复制求值器、直接调用Body或构造新Interp。
推荐新增特殊内置eachLine(path,maxLineBytes,callback)；callback(line,lineNo)返回严格bool，true继续/false早停。返回visited:int，false那次计入visited；空文件0，不为末尾LF制造额外空行。既有同名binding可遮蔽；无first-class builtin，通用builtins::call不可无上下文绕过native终止/诊断控制。具体名字/计数/返回格式由architect冻结，当前仅建议。
三参所有类型/范围/函数可调用及恰2参数先校验，再open。若只等首行才call_user检查arity，空文件将静默接受坏callback，行为依赖文件内容；应把该前置规则写入规范。callback验证需沿现call_user的用户函数规则，不能把argv等不符arity函数当“function类型就行”。
首轮JSONL建议组合eachLine与既有jsonParse/attempt：回调内attempt(()=>jsonParse(line))失败记日志/计数后继续，成功处理record。这样空行/空白行/坏JSON与业务失败策略由调用方显式决定；不把JSON parse塞在callback前导致坏行无法局部捕获。若要求直接eachJsonLine，必须冻结callback收到Result还是原record、空行和解析错误策略，避免两套隐式容错。无需增加生成器/独立读取句柄/用户close语法。

## 有界读行与文本口径
用固定容量BufReader<File>加自有有界Vec<u8>当前行；fill_buf→查LF→只复制允许前缀→consume，跨buffer拼接。不能用无限read_line/read_until后才检查len，也不能lines().collect/read_to_string。buf容量如8KiB固定，当前行最多maxLineBytes加明确的小常数，超量立即报错而不是继续读完整超长行。
最容易验证的候选口径：maxLineBytes计LF前raw bytes（含CR）；交付line删除一个终止LF及其紧邻CR，独立CR保留。这样CRLF的CR计入cap，需手册明确，不用字符数冒充字节。若希望CRLF两个终止符均不计，则reader需一字节CR延迟判断/至多cap+1暂存并精确测试，不得先无界缓存后strip。两种仅选一种冻结，不混用。
UTF8在整条有界行聚齐且去终止符后严格解码；跨8KiB边界的多字节字符合法，非法UTF8受控Io或专用ValueMsg由architect选定，不lossy替换、不猜编码。BOM只作为首行普通字符保留，与旧readFile一致；JSONL中BOM是否接受由现jsonParse规则决定。NUL是合法字符串数据，不因是行内容当路径NUL拒绝。
空文件无callback；“a\n”一行；“\n”一条空行；“\n\n”两空行；“a”末行一次；“a\r”保留CR；“a\r\n”一行文本a；分割只以LF，不能把Unicode行分隔符当换行。超长末行和带LF行同样受cap。文件修改不是snapshot，callback append/truncate/重命名后读到的结果沿OS读取语义，不能承诺事务一致或自动重试。
不要一次预分配用户cap全长；按实际需要增长，但增长策略/try_reserve应防溢出和过度capacity，不能把容量上界描述得比实现小。cap范围建议1..16MiB，零行长是否允许另定；空行仍可读。用户callback把每行push到外部数组/保留closure捕获时可自行导致O(文件大小)，只承诺reader自身O(buffer+最长行)，不称整个程序恒定内存。

## 回调、attempt与终止的准确边界
evaluator.rs:629 poll同时检查cancel/deadline/steps并扣step，native循环不能复用此函数，否则凭读取次数新增AST步破坏§22。应另立只检查cancel Acquire→deadline的native检查（不检查remaining、不扣步），在open前、每次fill_buf前/后、每次callback前以及正常/早停返回前检查；具体最终false与同时取消优先由architect冻结。其错误必须先note_error_span，不能?直接返回漏真实source。
callback本身照常通过call_value/call_user进入，实际stmt/expr仍扣单一会话remaining，跨行/嵌套eachLine/attempt不重置，callback错误/取消/deadline/早停均到同一owner回收路径。局部attempt坏JSON只恢复既有trace/depth/source，不补预算、不回滚副作用/文件游标；callback返回false是成功控制值，不伪装异常。非bool返回TypeError并关闭；函数内return false只退出那次函数再使reader停，不当作外层LFZ return/break穿透。
call_user已有caller active/globals恢复（包括Err）且故障帧保留；每次callback用其home/captured/self，reader不临时切换active到数据文件。文件数据不是LFZ源码，不能把lineNo冒充LzError.span/source；读/UTF8/行限额消息可包含路径与数据行号，span仍LFZ调用点，callback脚本错误仍callback真实module来源与函数帧。先在回调内局部attempt失败再继续，后续未捕获错误不得继承上行fault_source。
native协作检查不能硬中断已经阻塞的open/read；普通文件、网络共享、设备/命名管道不能一概保证deadline内返回。建议首轮明确面向普通文件；任何is_file检查都不是恶意竞态沙箱，不能宣传绝对阻塞上界。若要支持阻塞源另做可取消后端，不能临时线程超时detach遗留reader。有限文件取消测试用同步checkpoint而非纯sleep，timeout有外部watchdog而不强杀LFZ解释线程。

## owner与close/诊断门禁
reader owner绑定整个eachLine调用，任何返回路径都显式finish后才返回；RAII Drop是兜底，不是“close错误已处理”的证据。std File drop没有R<T>回传路径，不能假装Drop能汇报关闭失败。建议tooling提供小型跨平台owned reader后端，显式finish消费owner并报告close失败；平台必须保证句柄唯一所有者/只关一次，raw handle转换和失败路径由tooling审查。不要使用File::sync_all伪装close，它不是释放资源的等价操作。
读取主错误/回调ExecutionLimit存在时保原reason/帧/source，close失败作为独立诊断；无主错时close失败可Io。即使该Io被外层attempt捕获，诊断仍阻断宿主成功。推荐延续私有file_diagnostics/accessor，保原process/http两个列表与文案；CLI/runner门禁需冻结三列表次序及主标题选取，旧文案字节不变，不把文件关闭错标HTTP/子进程。
新增列表不加入attempt快照，各session独立。不新增EvalCapabilities字段：既有readFile已允许普通文件读，不应以逐行功能突然改旧IO授权；若团队决定新权限应另明确迁移，不能隐式借HTTP/process授权。
最小跨owner边界建议LineReader::open(path)->owned session、next_line(max)->Result<Option<Vec<u8>>,ReadFailure>、finish(self)->CloseReport；next_line内部只做有界读，不接Value/Rc/Env。runtime在reader调用边界native poll及callback，不把Invoke ABI扩大。纯Vec字节可以在runtime解码并生成Value，每次不缓存之前记录。
若后端需要长行内每8KiB也poll，可next_chunk返回有限片段让runtime驱动，或传纯owned cancel/deadline control到reader；只读检查不能调用AST扣步poll。请architect与tooling择一冻结，避免同一逻辑两owner各写一半。

## 验收与规模证据
必测行边界：空/单LF/连续LF/CRLF/孤CR/末行无LF，UTF8多字节跨buffer/非法UTF8、cap精确N/N+1（含CR计法）、超长无LF；callback false后文件后部坏UTF8不应读取处理；异常/非bool/错arity不打开文件；callback内坏JSON连续失败后成功；跨module home/capture/default调用与真实trace；嵌套reader/读中修改文件；cancel/deadline前/回调中/两行间，零remaining的native不另扣但callback下一AST停止。
资源释放必须正常/早停/异常/全局终止/close注入都覆盖。Windows运行返回后重命名/替换文件是有用见证，Unix单独rename不证明fd关闭，应辅以句柄计数/受控关闭probe及重复循环无FD增长；本机Windows通过不代表Linux实测。若无Linux执行环境，报告明确未验，不伪造跨平台关闭结论。
大规模oracle按确定性生成器逐条写项目Temp文件，独立累计计数/校验和/坏行位置，不把待测eachLine当oracle。至少小/中/大三个文件尺寸，当前行上限固定；独立子进程量峰值RSS/工作集、吞吐与walltime，含启动成本、预热/重复次数、样本与最终exe SHA。结果规模正确且reader内存随文件大小不线性才支持stream结论；callback只累计而不collect，不把OS文件缓存算成语言堆。readFile对照使用安全规模避免OOM，保全部失败及环境噪声。
本轮测量只证明T19 workload；R304原19项仍OPEN，不拿新大文件读吞吐替全语言性能达标。

本评审已提交root/architect，以上API/错误/close诊断均为待冻结建议；未获GO前不改产品。
