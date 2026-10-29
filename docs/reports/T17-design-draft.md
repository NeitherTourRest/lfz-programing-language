# T17 执行预算、取消与时间边界草案

日期2026-10-29。责任人language-architect。仅后继设计，未冻结规范、未实现。依赖T16多模块session/source一致性；保留T15可恢复业务错误与不可吞资源终止的原则。

## 推荐拆批

T17-A先实现确定性求值步数预算与宿主取消token；T17-B再加协作式墙钟截止时间、CLI说明及宿主集成。外部子进程/HTTP/阻塞IO的硬超时由T18专门实现，不能拿求值poll夸称可以打断任意宿主阻塞调用。模块图的1024/128/bytes是独立前端预算，不算求值步数。

推荐公开EvalLimits { max_steps:Option<u64>, deadline:Option<Instant>, cancel:Option<Arc<AtomicBool>> }及新显式with_limits入口，旧所有API默认unlimited不改行为。实际命名等C源事实再冻结；Instant/AtomicBool属于宿主API，不映射为LFZ值，不作为全局static在脚本之间串用。

CLI拟--max-steps N（非负十进制u64）、随后--timeout-ms N；原CODE及--后argv保护不变。不能全局摘除CODE内同名字串。缺值/负数/溢出为usage error，沿现stderr文本规则。N=0允许，空无求值Program可成功，首个求值步前结束；是否空模块声明算步必须按下述规则冻结，不能凭wall time决定。

## 步数定义候选

在每次exec_stmt进入与eval_expr进入各消耗1步，先更新当前trace节点位置，再check取消/deadline/预算，再进行该节点任何副作用。空while body仍不断求值条件，所以会受限；函数调用的函数体/参数表达式、高阶回调的实际LFZ体都会消耗步数。module初始化计入同一session预算，import/export声明是否exec_stmt经手均按一次Stmt执行计1步，缓存复用不重复初始化。

remaining从max_steps开始，执行下一步前若为0返回终止；否则减1并执行。unlimited不作计数溢出，已用步数若公开用饱和计数或checked定义，禁止wrap。检查顺序建议cancel→deadline→steps固定，deadline时刻采用Instant，不看系统时区。步数按语义执行路径确定，不将内部Rust函数调用/HashMap探测/metadata格式化算步；编译器或trace格式化变化不该使同一LFZ路径预算改变。

这是可复现的求值单元预算，不是CPU指令配额：range/join/JSON等单个原生内置可能做大量工作而只进入一次LFZ调用表达式。对这些长循环可另加协作poll但不能重复消耗LFZ步数；阻塞input/readFile无法仅靠poll打断。若追加native poll，应按固定迭代块频率，不假装定义硬deadline精度。

## 终止错误与attempt

建议新增LzError::ExecutionLimit { reason:ExecutionLimitReason, span:Span }，reason为StepBudget/Cancelled/Deadline；class_name建议ExecutionLimitError，消息分别“执行步数预算已耗尽”“执行已取消”“执行时间上限已到”。这意味着错误类从12增加，必须显式版本化批准并同步严格元数据测试，而不能复用RecursionError或Overflow来伪装资源原因。具体字段/是否记录limit由冻结时统一定案。

该variant永远不在attempt可恢复白名单，资源终止穿透所有嵌套attempt；终止时原故障trace/source保留。剩余预算/取消token/截止时间不是可恢复快照状态，不因attempt成功/失败restore而重新补充；否则可用attempt循环绕过预算。语句之前已发生的容器写/IO/RNG仍不回滚，不能宣传事务或无副作用取消。

模块调用active/globals恢复与终止trace保留沿C规则。错误位置为触发预算检查的当前节点，不能落在上一个成功语句；CLI JSON仍原错误字段结构，真实source定位完整。T15错误Result不会出现ExecutionLimitError，因为它不可捕获；普通用户自己用相同class字符串的记录不是真正终止。

## 取消及时间限制的真实保证

宿主可从其它线程置Arc<AtomicBool>，Interp在下一poll返回Cancelled；token由调用者管理，新执行默认独立token。若调用者明确复用已取消token，新执行首poll也取消，不能偷偷reset别人的共享状态。memory ordering采用Acquire读取/Release设置或经证明足够的Relaxed，固定API说明，不引入用户可变共享Value跨线程。

CLI Ctrl-C支持不能只声称std有处理器：当前零第三方依赖，跨平台信号接入需明确原生实现/依赖决策。A可以先仅库取消token，CLI仍系统默认中断行为，不能伪称其必输出结构化Cancelled。B再决定平台支持范围。

deadline仅协作检查，可能在单个内置或阻塞IO期间越过；不强杀解释器线程、不catch_unwind伪装取消，不在仍运行线程上释放AST。若要求真正硬超时，应外层子进程隔离并明确退出语义，属于单独host runner，不能复用unsafe Transfer把执行线程强终止。

## 验收/效率门槛

确定性步数：固定小程序逐预算边界精确检查，重复结果/终止span相同；0/1/足够预算；while/for/函数递归/7HOF/模块初始化共享预算；无限循环受控返回但不真实abort；大量attempt失败无法补步，嵌套attempt不能吞终止；取消已置位、执行中置位、跨执行隔离；Deadline已过/合理到期，采用宽容实测时界不把调度抖动当产品失败。

普通unlimited所有旧行为/CLI保护/图check/JSON/apps保持。原热路径新增poll分支可能影响吞吐，必须冻结最终字节做原19项回归，不以“仅一个if”宣称零开销。明确不保证阻塞宿主IO硬打断、峰值内存限制或OS文件沙箱。R304仍OPEN，现草案不自动放宽门槛。

owner建议architect先批准新增错误类/limits与计步规则；runtime实现session预算/终止与恢复交互，tooling实现精确选项与CLI协议；core只按另派增加错误variant/消息（单写者），测试/指南/独立复核随后。A/B分批，T18再提供真正外部工具超时/输出上限与能力开关。
