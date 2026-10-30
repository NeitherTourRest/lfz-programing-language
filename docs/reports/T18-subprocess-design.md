# T18-A 子进程调用精确设计提案

日期2026-10-30，language-architect。状态：提案待team-lead裁决，非冻结spec、非已实现。T17独立PASS由parent报告（1946ab9/624/134/app40）；本角色只读实际接口、不编译。HTTP独立T18-B不进入本批。R304仍OPEN。

## 1. 已审代码及推荐取舍
实际builtins::Invoke为可调用LFZ函数的FnMut窄接口，并非权限/资源上下文；不能借此偷偷传全局能力。BuiltinFn签名保持fn(&[Value],Span)->R<Value>，call/call_with旧API保持。evaluator实际EvalLimits为max_steps/deadline/cancel三公开字段，三类_and_limits入口已经存在；为兼容Rust struct literal不得再向EvalLimits添加能力字段。

推荐新增上下文包装，保持现有入口全部默认无子进程权限。LFZ只新增一个特殊内置exec，沿现argv/attempt的显式Interp分派，保留作用域同名函数遮蔽规则、不让它成为可逃逸权限的first-class builtin。内置总数73→74，名字注册/帮助/skill由各owner机械更新，既有73项断言不可删减。

关键决策：零crate不等于只用跨平台std。std Child无Drop kill/wait；仅双reader线程+kill主进程，会在后代继承pipe时无限join。为可靠timeout/输出限额必须控制进程组/Job及pipe生命周期。推荐允许隔离平台模块中的小范围原生FFI；否则须另准经过审查的依赖，不能用静默降级掩盖。root先裁决，不在本提案直接冻结unsafe接口。

## 2. LFZ最小API与返回值
推荐固定三实参：
    exec(argv, timeoutMs, maxOutputBytes)
argv是非空array，每项必须string；argv[0]非空且是绝对可执行文件路径，所有项拒NUL。第一版不查PATH，不隐式shell，不支持Windows .bat/.cmd；Windows原生.exe，Linux原生可执行文件（内核shebang行为是OS语义，非LFZ拼shell）。直接显式运行shell是host授权任意进程后的能力，不能据“无隐式shell”宣称限制了shell使用。argv[0]之后的参数顺序、空字符串、空格、Unicode原样交给平台后端。Windows参数序列只对标准Windows C运行时argv解码兼容工具保证保真，拥有自定义命令行解码的工具须按其协议，不宣称Windows内核存在独立argv数组。

timeoutMs为int且1..600000；maxOutputBytes为int且0..16777216，合并stdout+stderr的原始字节总量。每次调用必带明确限额，不接受nil/隐含无限。上限固定用于首版收口，后续host可配置上限另审。超量条件为实际观测总量>max，恰等于允许；max=0允许静默程序正常退出。
成功返回普通struct，键顺序固定：
    {stdout:string, stderr:string, exitCode:int|nil, signal:int|nil}
非零退出正常返回，不抛异常；Unix信号终止exitCode=nil/signal=信号编号；Windows正常退出码按无符号32位扩到i64，signal=nil。不增加ok冗余字段；用户按exitCode==0判断业务成功。Unix无法提供信号数的异常状态是IOError，不捏造0。
各stream保原始顺序，不提供跨stream全序。无CRLF/换行/BOM规范化。成功输出严格UTF-8解码；不合法则ValueError，不lossy替换、不把二进制塞JSON；二进制工具应自行编码文本。stdout/stderr仅作为返回值，不转发到LFZ父进程stdout/stderr，不破坏--json唯一对象。
stdin设为null即EOF；继承本次host cwd和环境，不注入shell字符串、不提供本批env/cwd/stdin扩展。能力是任意可执行权限，继承环境的secret暴露由host决定；不称OS沙箱。

## 3. 校验、错误和attempt边界（待批准精确消息）
原有实参求值先按LFZ左到右执行；任何进程副作用前，按实参数量→argv数组类型→逐元素string/NUL/argv0路径→两个int类型及范围→平台支持→能力→资源预检顺序完整校验。不得先spawn后发现最后一个参数类型错。TypeError沿已有ArgCount/BadOperands；值约束增ValueMsg小变体或专用固定消息（core唯一error.rs，最终冻结时列逐variant）。
推荐固定业务消息：
- 禁用能力：ValueError“子进程能力未启用”
- argv空/含NUL/首项非法/范围错误：ValueError，精确细分由root批准后冻结
- 局部timeout：ValueError“子进程执行时间上限已到”
- 输出超量：ValueError“子进程输出超过字节上限”
- 非UTF8：ValueError“子进程输出不是合法 UTF-8”
- 不支持平台：ValueError“当前平台不支持受控子进程”
spawn/pipe/read/wait/kill等OS错误为IOError，消息含动作及OS原错，不伪造成功。调用点Span恒Some，真实module/source走C。局部timeout/超量是本次工具调用的可恢复业务失败，可attempt；不能把它们复用ExecutionLimitReason::Deadline，后者专指会话全局终止。无新增第14错误类。
会话cancel/deadline复用T17，等待进程时也协作poll，固定cancel→全局deadline→局部timeout→输出超量→普通完成。全部错误先完成回收再返回；系统ExecutionLimitError仍不被attempt吞。首次确定的终止原因锁存，回收期间的新timeout不得篡改原原因；回收失败不得静默成功，须保留原终止类并在host诊断路径报告cleanup故障，最终carrier需root裁决，不能临时改T15 Result字段。
输出溢出/读取错误/取消任一个发生都停止收集新返回数据；不返回部分stdout伪装完整结果。计数按两stream观测的原始byte合计，保留内存绝不超过上限；pipe内核缓冲/子进程自身内存不属于此上限，不能声称子进程最多写了N个bytes。
T17剩余步数不在native等待扣除；在exec等待中不把remaining==0当新节点触发StepBudget。cancel/deadline单独poll不补步、不改变T17 stmt/expr计步定义。调用exec的节点已扣步，返回后下一节点才可能StepBudget。

## 4. Rust上下文与CLI推荐接口
    #[derive(Clone, Debug, Default)]
    pub struct EvalCapabilities { pub subprocess: bool }
    #[derive(Clone, Debug, Default)]
    pub struct EvalContext {
        pub limits: EvalLimits,
        pub capabilities: EvalCapabilities,
    }
公开于evaluator；三个新入口与T17对应，后缀_and_context，最后一参owned EvalContext，返回类型照旧：
- eval_module_traced_with_args_and_context(&Program,&[String],EvalContext)->TracedRun
- eval_module_traced_on_thread_with_args_and_context同参数/返回
- eval_graph_traced_on_thread_with_args_and_context(&ModuleGraph,&[String],EvalContext)->Result<TracedRun,GraphExecutionError>
旧无limits和T17 limits入口均构造capabilities Default(false)后委托，签名/公开shape不改。Interp持context/预算，所有module同一能力快照；函数只持整数home/cells，不携带或扩大权限。不用static/thread_local开关，不在attempt快照恢复权限；同一进程随后新会话默认禁用。旧builtins::call("exec",...)若公开可达必须返回禁用/要求上下文错误，不能绕过Interp授权。

CLI新增执行区布尔--allow-process，仅裸/run/-e/stdin执行有效，默认false；CODE与--后原样保护；check/test/--help/-h/--version/-V拒绝。重复拒绝、等号形式拒绝，不新增裸help/version别名；与T17/--json选项顺序规则一致。check可语法检查含exec源码但不运行/授权。test runner旧API默认禁用，不能因父CLI授权默认给所有用例执行能力。没有--deny-process等冗余选项。CLI子进程能力仅本次execute上下文，不影响后续库调用。
公开EvalContext未来HTTP/流资源扩展仍需兼容审查；本批不预加空Http/client字段。

## 5. 生命周期与可靠后端验收条件
必须先拿到可纳管进程/两输出/关闭stdin的全部资源，再允许子进程运行用户代码；任一步失败逆序关闭句柄并kill/wait已创建进程。主循环并行drain两输出，不能先wait再read，也不能先读完stdout再读stderr；每轮有界read，避免持续stdout饿死stderr/timeout/token检查。建议8KiB read块、轮询等待最长10ms，输出缓存共享byte预算并保留每stream顺序。10ms是poll目标，不是OS调度/强杀完成硬时界。
正常直系进程退出后也清理受控后代、关闭写端、drain现有管道数据；不能等待后代无限持pipe。超时/取消/超量先锁存reason，终止受控组/Job，wait直系回收，再关闭/取消读并回收reader。所有线程、进程、句柄归属清楚，无detached reader、无无界channel。程序正常返回也不能留下后台任务。不能简单sleep后drop Child当已回收。
Windows推荐Job Object kill-on-close，不允许breakaway，启动挂起→纳入Job成功→恢复线程，失败闭环kill/wait；不使用“先spawn运行再assign”的竞争窗。管道需非阻塞/可取消读取，句柄继承白名单仅必要写端。需要原生CreateProcessW/正确argv quoting与宽字符路径封装；具体安全包装归实施owner，不能只设置CREATE_SUSPENDED后无resume句柄导致永不运行。
Linux推荐独立进程组，纳管完成后执行，非阻塞read+poll，并在完成/终止时清理整组、wait直系。进程可主动setsid等逃逸，组管理不是敌对进程沙箱；Windows同样不承诺对抗恶意/提权程序。
平台后端无法保证回收时fail-closed，不允许退回普通Command无限wait。OS不响应强杀、不可中断内核态等不承诺绝对返回时界；要明确记录cleanup故障，不能把成功设计成需要永远等EOF。跨平台CI未跑的分支只称未验，不能以Windows测试推断Linux支持。macOS/其它平台先明确拒绝。

## 6. 待root裁决与owner
请裁决：①允许隔离原生FFI还是审查依赖（推荐FFI零crate，但需实施者安全审查）；②固定exec三参/四键及绝对路径策略；③局部timeout/超量可恢复ValueError与global资源不可恢复边界；④cleanup失败的host诊断carrier与功能交付平台。以上在本设计批准后再冻结三spec独立§23，不能按提案先改源。
推荐owner：runtime唯一evaluator上下文/特殊分派/exec业务封装；tooling唯一新src/process_control.rs及platform子模块与CLI（复杂native生命周期集中单写者）；core唯一error.rs细分消息；tooling如新module注册需要lib.rs只改注册一行并与runtime错峰；runtime不同时改platform。接口边界建议纯Rust-owned ProcessRequest/ProcessOutput，不把LFZ Value/Rc/Env送给reader线程；后端接Arc token/Instant，只返回owned bytes/status/原因，runtime唯一转LFZ Value/error。最终内部签名在platform方案裁决后冻结，防止假装纯std可行。
fixture由tooling新增独立测试helper进程，绝对路径在项目Temp/target内，不借shell命令语法作oracle。验证argv空格/空串/Unicode、双stream持续输出大于pipe缓冲、精确合并0/N/N+1、非UTF8、非零/信号、stdin EOF、spawn失败、timeout0范围拒绝、真实局部timeout、token/globaldeadline抢占、attempt恢复、模块权限/跨会话不泄漏。后代持pipe/直系退出、异常建立Job/句柄失败、取消时双管满必须有受控fixture与外层watchdog。测试本身要回收fixture及后代，不把产品死锁变成测试无限挂起。
无编译/产品见证完成声明；D119MiB，不复制target、不公网/项目外安装。最后最终hash功能与原19项性能另验。

## 一手资料核对
仅用于平台能力事实，非已完成实现证据：
- Rust std Child文档，明确Child drop不自动kill/wait：https://doc.rust-lang.org/std/process/struct.Child.html
- Microsoft Job Objects，kill-on-close与组管理：https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects
- Microsoft AssignProcessToJobObject：https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-assignprocesstojobobject

## cleanup诊断carrier补充推荐（仍待裁决）
不通过修改ExecutionLimit的固定message丢掉原reason，也不将回收失败转换成可attempt吞的IOError。推荐TracedRun新增私有host_diagnostics:Vec<String>与只读pub fn host_diagnostics(&self)->&[String]，Interp记录cleanup失败，最终CLI附stderr诊断（--json仍原唯一对象）。新增内部构造字段要机械迁移全部literal，public Result/TraceFrame/Span/JSON不变。无系统终止在先而cleanup失败时，IOError作为本次主错误；有系统终止在先保ExecutionLimit并附诊断。无法证明全部回收时必须明确留下未回收风险证据，不称通过；操作系统级不可回收不能靠carrier假装解决。此RustAPI增量同样需root批准后冻结，不在草案直接实施。
