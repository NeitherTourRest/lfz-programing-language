# T18-A 冻结设计
2026-10-30。root已批准，以下同interface-contract §23；历史T18-subprocess-design为提案。
## 23. T18-A 受控子进程（2026-10-30 冻结）
依据team-lead批准T18-subprocess-design全部四点及cleanup载荷。仅本批Windows后端实施/实测；Linux设计保留，未经真实平台验收前返回受控Unsupported，不启用半成品。HTTP独立B。本节覆盖历史提案的未决项；冻结非功能完成。

### 23.1 LFZ API与完整校验
新增特殊内置exec(argv,timeoutMs,maxOutputBytes)，共74内置；与argv/attempt同派发优先级，已有局部/全局同名绑定可遮蔽，不新增first-class builtin。BuiltinFn/Invoke/call/call_with旧ABI不变，直接无上下文builtins调用exec必须禁用，不可绕过能力。
实参按旧左到右求值；之后在任何spawn前依次检查：
1. 恰3个实参，否则TypeMsg::ArgCount{name:"exec",n:3,m}。
2. argv必须array，逐项必须string（先检查全部元素类型，再值约束）；类型错误沿BadOperands op="exec"，lt实际类型，rt分别"array"/"string"/"int"。
3. array非空；按索引检查NUL；argv[0]非空且绝对文件路径，Windows必须非设备的盘符绝对.exe路径或UNC.exe路径，扩展大小写不敏感；拒相对/盘符相对/设备路径/.bat/.cmd，不查PATH、不隐式shell。Linux未来启用时绝对可执行文件。
4. timeoutMs/maxOutputBytes均int，先按参数顺序检查类型，再值范围；前者1..600000，后者0..16777216。
5. 能力subprocess为true，否则ProcessDisabled；随后平台支持检查；最后cancel/deadline预检。被禁用在任何平台均报告禁用。
空串参数/空格/Unicode原样传递。绝对exe可以是用户显式选定shell，不声称阻止shell。Windows只保证标准C运行时命令行解码约定，不宣称内核有argv数组。stdin null/EOF，继承host环境/cwd；无env/cwd/stdin扩展、无重试。

成功返回普通struct按以下键序：stdout:string、stderr:string、exitCode:int|nil、signal:int|nil。非零exit仍成功值；Windows u32退出码无符号扩到i64，signal=nil。未来Unix信号退出code=nil/signal编号。每stream字节顺序保持、跨stream不承诺全序；严格UTF8，BOM/换行不规范化，任一非法UTF8整体ProcessUtf8错误。输出不转发父stdout/stderr，不返回部分成功。

### 23.2 资源与错误细分（core唯一error.rs）
新增ValueMsg无字段variants及精确message：
- ProcessDisabled：“子进程能力未启用”
- ProcessArgvEmpty：“子进程 argv 不能为空”
- ProcessArgNul：“子进程参数不能包含 NUL”
- ProcessExecutable：“子进程程序必须是绝对可执行文件路径”
- ProcessTimeoutRange：“子进程 timeoutMs 必须在 1..600000 范围内”
- ProcessOutputRange：“子进程 maxOutputBytes 必须在 0..16777216 范围内”
- ProcessUnsupported：“当前平台不支持受控子进程”
- ProcessTimeout：“子进程执行时间上限已到”
- ProcessOutputLimit：“子进程输出超过字节上限”
- ProcessUtf8：“子进程输出不是合法 UTF-8”
不新增第14错误类，以上ValueError可attempt。OS spawn/job/pipe/read/wait/kill等操作故障为Io，msg精确前缀“子进程{operation}失败：{message}”，operation为后端固定动作名，message为OS原因；调用点Span Some，真实来源走C。
全局取消/截止沿ExecutionLimitError Cancelled/Deadline，固定消息不变、不被attempt吞；局部timeout不复用全局Deadline。native等待不扣步、不因remaining==0自行StepBudget。首次启动成功后建立local deadline，建立/纳管/恢复前也检查会话全局token/deadline；本地timeout覆盖运行及输出drain，spawn本身OS调用不承诺硬中断。
每轮观测优先级cancel→全局deadline→局部timeout→合并输出超量→读/进程OS错误→普通退出完成；首次确定reason锁存，cleanup不改写reason。仅说同一轮可观测事件优先，不承诺并发发生全序。最终返回普通成功前再做一轮同优先级检查，这属于exec原生协作poll，不扩T17 AST计步/其它native规则。
stdout+stderr按观测raw bytes合并，>N失败、=N允许。0用于无输出命令。共享保留缓存总量<=N，任一read最多8KiB；溢出只保留允许前缀并立即终止收集，不返回部分值。内核pipe/子进程自身内存不属此缓存上限；有限scratch最多两块8KiB，不允许无界channel。两stream公平轮转，poll等待目标<=10ms，非硬实时。解码仅在全部成功回收、完成限额检查后；UTF8错误前也必须回收，不留下进程或reader。

### 23.3 上下文和兼容（runtime evaluator唯一）
公开Clone/Debug/Default：
    pub struct EvalCapabilities { pub subprocess: bool }
    pub struct EvalContext { pub limits: EvalLimits, pub capabilities: EvalCapabilities }
三个新owned context入口：
    eval_module_traced_with_args_and_context(&Program,&[String],EvalContext)->TracedRun
    eval_module_traced_on_thread_with_args_and_context(&Program,&[String],EvalContext)->TracedRun
    eval_graph_traced_on_thread_with_args_and_context(&ModuleGraph,&[String],EvalContext)->Result<TracedRun,GraphExecutionError>
首个沿大栈wrapper，后两个调用方已有大栈；旧API、EvalLimits三字段shape不变，所有旧入口构造Default capabilities false。不用global/thread_local权限，不跨会话泄漏；全部modules共享同session权限快照，函数home不携带权限。权限不加入attempt可恢复快照。不改UserFn/StructDef/Value公开shape。
TracedRun新增私有host_diagnostics:Vec<String>，公开pub fn host_diagnostics(&self)->&[String]；旧内部literal机械填空。Interp积累cleanup失败诊断，attempt不恢复/清空这份记录。库使用者必须将非空host_diagnostics视为宿主执行完整性失败，单看result Ok不足以证明回收成功。
backend有系统资源主错误时保持原ExecutionLimit及trace/source，cleanup故障只附独立诊断；无资源主错误则本次操作回收失败为Io。即使该Io被attempt捕获，诊断仍保留，宿主不得冒充完全成功。CLI result Err时原样渲染该业务/系统错误并把诊断逐行写stderr；result Ok但诊断非空时沿C HostFailure原则：stdout为空，stderr先写“lfz: 子进程资源回收失败”并输出诊断，exit2；即使--json也属于既有宿主异常JSON例外，不合成伪LFZ业务IOError/位置/trace。result Err的--json仍输出唯一原错误对象，无新键；诊断行前缀“lfz: 子进程回收诊断：”。不虚构脚本错误位置，不让可catch IO覆盖原系统终止。非空诊断路径必须有故障注入测试，不代表OS不可回收风险已解决。

### 23.4 后端边界和生命周期（tooling唯一）
新src/process_control.rs及私有平台子模块；lib.rs仅tooling增加pub(crate) mod process_control注册，不暴露不稳定平台ABI。
跨owner纯owned Rust边界，pub(crate)：
    ProcessRequest { argv:Vec<String>, timeout_ms:u64, max_output_bytes:usize }
    ProcessControl { deadline:Option<Instant>, cancel:Option<Arc<AtomicBool>> }
    ProcessOutput { stdout:Vec<u8>, stderr:Vec<u8>, exit_code:Option<i64>, signal:Option<i64> }
    ProcessFailure { Cancelled, Deadline, Timeout, OutputLimit, Unsupported,
                     Io { operation:String, message:String } }
    ProcessReport { result:Result<ProcessOutput,ProcessFailure>, diagnostics:Vec<String> }
以上struct字段pub(crate)，类型derive Debug（Control可Clone）；pub(crate) fn run(request:ProcessRequest,control:ProcessControl)->ProcessReport。
runtime负责LFZ类型/值检查、权限、UTF8及转ValueMsg/LzError；tooling后端在内部再次校验可信前提，非法host request返回Io operation="请求校验"，无panic。后端不接Value/Rc/Env/Invoke，不自行输出终端；返回diagnostics由runtime合并。无reader线程跨传LFZ值。

Windows std+原生FFI零crate：创建Job并设kill-on-close，创建受控pipe/句柄继承白名单，CREATE_SUSPENDED创建进程，Assign成功才Resume；Job创建/纳管失败关闭资源并kill/wait挂起进程，禁止fallback裸spawn或让用户代码先运行。正确UTF16/标准argv quoting，非设备exe约束；不把线程悬挂后无resume handle当成功。纳管失败必须在用户子进程代码执行前受控拒绝。
并行有界drain两pipe，可用平台非阻塞/可取消读取，不能先wait再读或先读完一边。任何错误/限额/正常主进程退出都结束其受控后代、关闭写句柄，回收直系，完成有限drain/取消read并回收所有reader，再返回。正常成功也不允许遗留后台Job任务；主进程退出而后代持pipe仍须结束后代，不等无限EOF。不得detached reader，无界join或仅drop Child。进程退出后仍需检查超量及读取错误，不能漏最后数据。
清理失败记诊断/真实失败；强杀/OS资源回收不承诺在内核不可中断等异常下绝对时界，不称恶意进程/权限逃逸隔离。FFI只平台模块，所有owned handle单一RAII owner/失败路径逆序释放，需review和本地fixture实跑。
Linux设计独立进程组、非阻塞pipe/poll、组kill/wait；本批编译分支返回Unsupported，后续真实平台验收另行启用，不能以Windows通过宣称支持Linux/macOS。其它平台同样Unsupported。

### 23.5 CLI、owner与验收
新增选项--allow-process，无值，仅执行命令，重复usage“选项 '--allow-process' 不能重复”；--allow-process=任何值立即usage“选项 '--allow-process' 不接受值”。CODE及来源后-- argv保护沿T17。非执行命令check/test/--help/-h/--version/-V报“子进程能力选项只适用于运行命令”；裸help/version仍文件。限额/能力扫描按token序，最终适用性检查前先处理扫描错误。默认false，权限仅当前execute；runner/check不隐式授权。帮助文档必须说明绝对exe、stdout返回值与能力非沙箱。
混合类别适用性补钉（2026-10-30）：全部选项扫描成功且命令为非执行命令时，若同时存在能力--allow-process与限额--max-steps/--timeout-ms，按选项区首次出现的新选项类别决定唯一文案：首次为能力则“子进程能力选项只适用于运行命令”，首次为任一限额则“执行限额选项只适用于运行命令”。忽略--json对类别次序的影响，CODE/argv内文本不参与类别记录。扫描错误仍按token序立即返回，优先于最终适用性；不因前面已见合法能力/限额而提前跳过后面的缺值/非法/重复/等号错误。无新选项时旧行为保持。例如check --allow-process --max-steps 1报能力文案；check --max-steps 1 --allow-process报限额文案；check --allow-process --max-steps x报既有非法值文案。
core唯一error.rs ValueMsg细分；runtime唯一evaluator、builtin名字注册/无上下文拒绝消费；tooling唯一process_control/platform/CLI/main/lib注册，若双方需builtins同文件由runtime唯一编辑；test-engineer/指南/app/verifier按宪法。先core+runtime接口、tooling后端错峰，唯一编译者由root指定。
必须测真实缺功能红测→新fixture：空串/空格/Unicode参数；双pipe同时超过缓冲；合并0/N/N+1及尾部超量；UTF8/非UTF8；非零退出/EOF；缺exe/拒绝批处理/相对/NUL/类型错误无副作用；禁用不spawn；局部timeout可恢复，全局token/deadline不可吞与同轮优先；module/context权限与跨会话隔离；后代持pipe/直系退出/回收无孤儿；Job创建/纳管故障不执行fixture；pipe/read/kill/wait故障注入不panic及diagnostics成功门禁。所有fixture/临时文件在项目内，无shell拼接当oracle、无公网/外部安装。外层watchdog须能清理fixture，不让测试本身无限挂起。
同最终hash旧T17/C/应用/CLI全回归、独立验收和原19项性能另验，R304仍OPEN。未测Linux明确Unsupported，不把提案/编译通过当平台实测。
