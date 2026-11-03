# T19 逐行/JSONL处理契约提案

日期2026-11-03，language-architect。仅提案、未冻结/源GO，基线mainc2a3d39及18d328B1029120由root提供。完整启动并读NEXT/§18attempt/§22limits/§24及实际Invoke、each、call_value/context。仅当前T19轮交付收口，不自动T20/恢复自动化；R304OPEN，原用户28/1/1保护。

## 推荐唯一接口与返回语义
    forEachLine(callback, maxLineBytes, path)
callback(line:string,lineNo:int)->bool，data-last与既有HOF习惯一致；maxLineBytes为int 1..16777216，path为string且不含NUL。callback返回true继续，false立即停止；其它类型TypeError，不能以truthy规则模糊。返回普通struct按lines:int、stopped:bool键序：lines是已实际调用callback次数，包含返回false的本行，stopped仅callback false为true。EOF为false，空文件{lines:0,stopped:false}。若callback错误或读取错误不返回部分成功struct；已发生的用户副作用保留。
不引入生成器/async/new Value/迭代器对象，不返回文件句柄；不新增JSONL专属解析builtin，用jsonParse+attempt组合。内置75→76，建议Interp特殊分派允许同名绑定遮蔽，非first-class，与exec/http不同无新增能力flag：沿既有readFile已授权读取边界，旧readFile签名/编码/消息/全文件行为不改。

实参先旧左到右求值，之后完整检查再打开文件：数量3→callback必须实际可调用函数→maxLineBytes int→path string→callback精确接收2参数→范围→NUL→资源预检→普通文件检查/open。argv等builtin function不能收2参，须在空文件也提前TypeError，不等首次回调才发现。无用户body/默认表达式为参数校验副作用；闭包按现函数规则，不新增回调适配层。
只普通文件；路径相对host cwd（与readFile相同，不按callback home模块），允许指向普通文件的符号链接。预metadata拒目录/设备/FIFO后open并复核已打开file metadata。不能把TOCTOU/阻塞open读宣称沙箱或硬可取消；本批不支持stdin/FIFO/socket/管道/目录，源文件前导ext/#42规则不应用于数据。

## 逐行字节/编码精确定义
LF(0x0A)唯一分隔；不输出LF。仅当该行由LF终止且紧邻前一个byte为CR时再去掉一个CR。孤立CR为内容；EOF无LF的末行保留末CR。CRLF规范化只发生本新接口，原readFile不变。
- 空文件0行；"a"为一行a；"a\n"为一行a；"\n"为一行空串；"\n\n"为两行空串；"a\r\nb\r"为两行a与b\r。
- 不剥BOM，U+FEFF可留在首行；不trim空白、不跳空行/注释。JSON坏行/空行是否跳由用户程序决定。
maxLineBytes计LF前原始bytes（包括将被剥离的一个CR），不含LF，不按Unicode字符数；恰N允许，发现N+1立即失败，不继续读到行末，不调用该坏行callback。因此"a\r\n"长度预算为2，"a\n"为1。UTF8多字节字符按全部bytes计。最后一行同规则（没有LF）；完整一行边界后严格UTF8验证，非法报ValueError，不lossy替换。跨8KiB块的多字节字符必须拼完整再验证。
坏UTF8/超长行在能构造LFZ string之前失败，不能让callback内部attempt“跳过”该坏字节行。若调用者外包attempt可捕获整次forEachLine失败，但不会恢复流offset继续；此前已成功callback的用户效果保留。JSON语法错误是可解码行里的业务失败，可在callback内attempt(()=>jsonParse(line))继续。
lineNo从1开始、int i64，含空行；在准备第i64::MAX+1行之前ValueError，禁止wrap或Float。lines计调用次数，不计缓冲预读行。callback false后即使下一行非法UTF8/超长也不检查、不失败；允许缓冲器预读有限后续bytes，但不处理/再回调。

## 流式内存/读取算法边界
建议8KiB BufReader/等价固定缓冲，逐fill_buf查LF且每次只复制仍允许的prefix；不可用无界read_line/read_until先读完再判长度。当前行Vec保留<=maxLineBytes，再加固定8KiB读缓冲；超限通过已有缓冲中第N+1字节观测，不再分配巨行。不用read_to_string/read_to_end/fs::read，不把全文件拆行后称stream。
将行owned String移入Value，不每行保留历史数组；callback参数只持当前行和编号。用户可主动保存字符串，内存将随用户保留内容增长，不能据引擎有界声称整个任意程序恒定内存。解析JSON也受单行内容/用户保留影响，语言无GC环/自引用行为不变。
处理期间文件修改按OS实际读取的流观测，不承诺不可变快照、文件锁、读完整个原始版本或原文件总行数；提前停止是实用价值。多个嵌套forEachLine允许，各自独立文件/缓冲且回调depth沿原Recursion上限，不能持RefCell借用跨call_value。

## 回调/预算/来源/关闭
Interp在每次实际读取块前、交付callback前、callback返回后（含false）、EOF普通成功前执行本native协作poll：只cancel/deadline，优先cancel→deadline，不扣AST steps，不因remaining=0在native额外StepBudget。参数求值/回调函数体真实stmt/expr照T17单会话扣步。callback归属定义module/home globals/captured/self，跨模块trace沿C；forEachLine本身不创建伪LFZ文件数据帧。native错误位置调用点/source，不把数据行号塞进Span冒充LFZ源码；消息带数据lineNo，callback错误仍原函数源码位置。attempt同步恢复现有trace/source，不恢复预算/用户效果。
一旦已确定callback/IO/编码主错误，锁存并清理，不再在清理后poll改写它；新资源终止只在上述poll观测点决定。回调返回false前后若观察到全局取消，资源错误优先于早停成功。同步OS metadata/open/read不能用poll硬中断，不新worker、不detach，不引入HTTP/进程能力。
文件资源由该次调用局部RAII guard持有，不放Env/captured/Value。callback false、EOF、TypeError/业务错误、ExecutionLimit、读取/UTF8/超长错误全部同一清理路径；必须在外层attempt拿到结果前已drop reader/File。不给LFZ逃逸句柄，递归嵌套各归各关闭。std File Drop不提供close错误返回，本批保证所有权释放并调用OS close，不声称可报告所有延迟写/close错误，也不新建第三diagnostic sidecar。若实现需要显式跨平台close报告，再先ADR而非直接改变旧门禁。

## JSONL惯用法（语法示意，非已运行见证）
    var good=0
    var bad=0
    let report=forEachLine((line,n)=>{
        let r=attempt(()=>jsonParse(line))
        if r.ok {good=good+1} else {bad=bad+1}
        true
    },1048576,"input.jsonl")
坏JSON继续；JSON值处理建议在同attempt里完成，避免解析后业务错误退出全流。错误记录要保存n而非假定JSON错误Span是数据位置。副作用聚合不保留全量对象；大规模应用oracle比较good/bad/总行号/校验和。是否另建实际JSONL应用由app owner按冻结契约做，本角色不改产品。

## 推荐错误/owner与待裁决项
core唯一error.rs新增ValueMsg（具体字段/消息待批准冻结）：LineLimitRange、LinePathNul、LineTooLong{line,limit}、LineUtf8{line}、LineNumberOverflow；现TypeError/Io复用，不新增第14类。普通文件/打开/读失败Io“无法逐行读取：{path}”，附行号细分如需另冻，不修改readFile旧文本。坏UTF8与readFile现Io不同是新接口有意的可恢复数据错误，须指南明确。
runtime唯一evaluator特殊分派/closure预检/资源poll、builtins名字注册，以及新src/line_reader.rs持文件的有界reader（纯Rust无crate）。core只error，不拆tooling平台后端避免不必要跨owner。tooling CLI本轮无新flag/contextAPI，只74/75计数等必要集成验证；lib模块注册由runtime唯一一行。test-engineer/app/docs/AI/perf/verifier按宪法。

请root裁决：三参data-last/bool早停/两键返回；默认沿readFile无需新能力；raw行长含CR/UTF8失败终止整流而坏JSON可callback恢复；限定普通文件/OS IO协作边界；std RAII不新增close诊断。实现者review后冻结三spec独立§25及精确owned reader API，不先修改source。

## 可复验门禁与性能计划
边界逐例含空/CRLF/孤CR/末行无LF/尾LF/BOM/空白/Unicode跨缓冲/UTF8非法跨缓冲，max N/N+1和16MiB绝对界；空文件也校验callback，早停不查后面坏行；callback非bool/抛错/多层attempt/跨module闭包/self/默认值中使用/嵌套流；全局token/deadline不可吞、预算在callback共享、remaining0不凭空native扣步。
关闭不能用Windows删除/重命名成功单独证明（共享打开规则允许），用可控reader Drop spy/计数证明所有error/early-return路径+真实打开句柄数稳定/平台独占重开探针组合，不虚构实际close错误可见。故障注入读失败在已成功行之后，确保主错误/句柄清理、回调不重复。
实际大规模JSONL按固定种子生成，至少三档记录数（如1万/10万/100万，同分布/同max行长），不保存所有record；独立oracle总数/坏JSON数/校验和。对比整文件readFile方案须同结果，分别测新进程峰值RSS/Windows PeakWorkingSet以及walltime/吞吐、输入bytes、最终exe hash/原始日志；文件生成不计处理窗，页缓存/硬件状态记录。不能只源码O(line)宣称峰值实测，不以单档耗时断言复杂度，用户保留对象另说明。原19项/R304不改，最终产物旧T18-B/T17/C/CLI/app全回归和独立验收后交付，当前无产品/性能完成声明。

root后续批准：三参/data-last/bool/两键返回、行字节/编码/JSONL边界及单会话poll方向全部认可，仍等runtime/tool review后冻§25。tool建议root批准std RAII统一drop不报告close异常，不sync_all假close，不扩FFI/diagnostic；Windows用share_mode(0)排他重开和warmup后同进程handle plateau实测，删除/rename非关闭证明。峰值计划可采用10/100/300MiB同分布与100MiB oracle，由perf/tooling实际执行。
