# T17 资源终止错误类决策提案

日期2026-10-30。状态：待单独ADR批准；本提案不改变当前12类LzError或T15白名单。

推荐新增真正资源终止variant，而非复用RecursionError/OverflowError。建议ExecutionLimit { reason: ExecutionLimitReason, span: Span }，reason StepBudget/Cancelled/Deadline，class_name统一ExecutionLimitError；message分别“执行步数预算已耗尽”“执行已取消”“执行时间上限已到”。这将从12增加到13类，必须由team-lead批准单独ADR，architect再冻结spec，error.rs owner按单写者实施。现有错误类集合测试只可机械更新注册表期望，不能弱化行为测试。

该variant不在attempt任何白名单，嵌套恢复不得吞掉，剩余步数/token/deadline不得纳入恢复快照。C的active/globals安全清理继续执行，faultSource/trace保留，副作用不回滚。用户伪造class同名struct不是系统终止。

Limits建议max_steps:Option<u64>/deadline:Option<Instant>/cancel:Option<Arc<AtomicBool>>，旧入口unlimited。每exec_stmt/eval_expr进入先更新位置后poll，poll顺序cancel→deadline→steps，步骤预算前剩余0则终止、否则减1；模块初始化共享一个session限额，前端图预算独立。同程序同路径步数可复现，不算内部HashMap/metadata格式化；长native builtin可协作poll但不能额外扣LFZ步骤，阻塞宿主IO不能宣称硬打断。

实施拆T17-A确定性步数与宿主token，T17-B协作deadline/CLI集成。CLI Ctrl-C的零依赖跨平台处理没有现成std能力，必须单独平台/依赖决策；先仅库token不能宣称CLI中断必输出结构化错误。禁止强杀解释器线程或catch_unwind伪装资源取消。T18子进程/HTTP硬超时另处理。

冻结前依赖C实际代码与B/C性能基线；默认unlimited仍新增热路径分支，必须实测而非声称零开销。N=0、空程序、循环/递归/HOF/模块共享预算、attempt无法补步、已取消token与跨执行隔离、deadline当前位置及真实source均需验收。当前只设计文档，无新增资源类/源修改/产品测试，R304仍OPEN。
