# language-architect — 工作状态
> 最后更新：2026-10-30 by language-architect

## 当前状态
T18-A仅提案docs/reports/T18-subprocess-design.md已落，未冻spec、未实现。已审NEXT、实际Invoke/BuiltinFn、EvalLimits与三limits API，核一手Rust Child/Windows Job文档。T17独立PASS由parent报告，本角色无产品实测。

## 推荐待裁决
exec(argv,timeoutMs,maxOutputBytes)三参，四键stdout/stderr/exitCode/signal；绝对可执行路径，无隐式shell/PATH，严格UTF8，非零exit正常值。
EvalContext{limits,capabilities}及三_and_context入口，旧EvalLimits shape/旧API保持禁用。CLI --allow-process仅本执行会话。局部timeout/超量ValueError可attempt，全局cancel/deadline ExecutionLimit不可吞，native等待不额外扣步。
重要平台决策已先报root：纯std双阻塞reader+kill直接子进程不能保证后代持pipe回收；推荐Windows挂起启动/Job+可取消pipe、Linux进程组+非阻塞pipe的小范围原生FFI零crate。不是敌对沙箱；未实测平台不声称支持。
cleanup失败建议TracedRun私有host_diagnostics及只读accessor，系统终止reason不被可catch错误覆盖；此迁移待root批准。
owner建议runtime evaluator/特殊调用，tooling平台后端与CLI，core错误细分；最终内部边界待平台决定。

## 证据/限制
提案11893B初写，附cleanup推荐后按现文件实值；文档diff检查通过。未源码/测试/编译/Git；D119MiB不做大任务。HTTP独立B；R304OPEN。

## 下一步
root裁决平台/接口/cleanup载荷后再冻§23，不能据本提案先改源。无用户补充需求，按已授权全自动交parent调度。
