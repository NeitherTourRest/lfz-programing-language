# language-architect — 工作状态
> 最后更新：2026-11-04 by language-architect

## 当前状态
三spec §26及T20-contract.md10245B正式冻结，ADR追加；root可据精确接口GO。提案为历史，已合root/tool最终stdin与ReplCapacity裁决。未源改/编译/Git。

## 精确接口
core parser::ReplInputStatus Complete/Incomplete；probe_repl_input(&str)->R<ReplInputStatus>，内部typed EOF不猜中文；error.rs ReplModuleUnsupported与ExecutionLimitReason::ReplCapacity（第4reason、仍13class），旧三message不变。
runtime ReplConfig{context,args,timeout_ms}/ReplSession::new、eval_cell(&str)->Result<Option<TracedRun>,ReplHostError>、reset；HostError InvalidConfig/Capacity/Poisoned/Origins。单大栈!Send/Sync session，sharedglobals/每cell home来源，旧UserFn shape不改。每cell新step/相对timeout，host绝对deadline/token不reset。
tooling CLI repl/typed probe/行循环/错误渲染/stdio/prompt/exit与formatter评估；不加伪formatter。input保旧语义，读REPL物理行后释放stdin lock再eval、无额外预读。

## 边界
非nil echo/print nil不重复；全部cell错误累计exit2、reset不抹计数。MVP禁import/export。
1MiB cell/16MiB累计源/4096cell/65536funcID，源码admission无本cell副作用；midcell ReplCapacity不回滚并poison，不可attempt；cleanup poison同样只reset/quit/help。
旧来源文本保至reset，fault outcome先存后清执行状态不事务回滚；REPL模板Some定义上下文，旧run None保持。

## 证据/下一步
文档diff检查exit0，仅换行提示。全旧功能/独立REPL/新Agent包版本实测及最终性能待owner完成，R304OPEN。原用户28/1/1保护。仅T20交付后收口，不automation/后继。
