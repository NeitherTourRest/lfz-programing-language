# language-architect — 工作状态
> 最后更新：2026-10-30 by language-architect

## 当前状态
T17已按team-lead 08:00独立ADR冻结三spec §22及T17-design.md（12016B初写，计时边界精确修订后以现值为准）。第13类ExecutionLimitError与StepBudget/Cancelled/Deadline正式契约；源码仍须C独立PASS后GO。本角色未改源码/测试、未编译/Git。

## 精确接口
core唯一error.rs：ExecutionLimitReason和LzError::ExecutionLimit{reason,span}、execution_limit helper；旧12类保留，穷尽match/注册机械扩13。
runtime唯一evaluator.rs：EvalLimits{max_steps:Option<u64>,deadline:Option<Instant>,cancel:Option<Arc<AtomicBool>>}，三个owned limits入口为module traced args、on_thread module args、on_thread graph args的_and_limits后缀；旧API默认Default不限额。
每实际stmt/expr入口poll cancel→deadline→steps；错误点span/source需经过note_error_span，不能?漏fault。成功poll不改旧trace位置；预算N允许N节点，无计数wrap。
跨module/默认字段/HOF同session；attempt快照不含预算/token/deadline，不吞终止。空程序无poll，末原生调用后无节点不追补poll；不承诺阻塞IO硬中断。
CLI --max-steps/--timeout-ms严格u64/重复拒绝/CODE及argv保护/非执行拒绝。timeout静态build_graph完成后eval调用前建立，内部host结构校验开销含窗口；校验错误仍优先poll。不加公开validation helper和Ctrl-C依赖。

## 证据/限制
读取实际C graph API、stmt/expr包裹、attempt及runtime状态。T17-design.md与三spec文档diff检查exit0，仅LF→CRLF提示；本次无产品测试或功能验收声明。C库456/456由parent实测，CLI实施中；B已推9d7e1aa，R304OPEN，无正式发布。

## 下一步
收team-lead/core/runtime接口审查；C独立过后按owner分A资源类+步数/token、Bdeadline/CLI实施与独立验收。T18由parent继续派发，不自行调度；所有临时文件项目内，D低空间不做大任务。
