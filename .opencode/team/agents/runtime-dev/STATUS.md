# runtime-dev 状态
2026-11-04 T20 REPL运行时源码已停写交tooling，未编译。

evaluator ReplConfig/HostError/Session按§26，单Interp/sharedglobals/独立cell home/source、template定义上下文；真实来源snapshot及失败后非事务恢复、每cellsteps/timeout与hosttoken保持、admission上限与动态ReplCapacity poison/reset。make_closure私有R传播所有构造，旧公开API/Value/UserFn不改。7新unit未运行，diff check过；SHA25F94AE7…，报告docs/reports/T20-runtime-review.md。

SourceMap保旧outcome时COW深clone历史Loaded成本已披露，不称恒定内存；不擅改core source/CLI。待tooling唯一统一release反馈。无Git/编译/外装/网络，R304OPEN。

最新：tooling首lib527/1新增oracle失败，已仅修本轮compile/import测试。非法后置import保ImportOrder，另合法前置import及合法export精确ReplModuleUnsupported且n无副作用；生产/旧tests不改。新SHA049FF34A…，diff check过，未编译停写交tooling，首失败日志保留。
