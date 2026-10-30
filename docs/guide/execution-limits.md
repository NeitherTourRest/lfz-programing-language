# 给Agent脚本设置执行限额

本章按三spec §22及T17-design冻结契约说明；实现的最终哈希与实跑结果由[T17-guide证据](../reports/T17-guide-evidence.md)单独绑定，不能用T16-C或T17修前候选成绩代替。

## 先检查，再限额运行

```powershell
lfz check --json job.lfz
if ($LASTEXITCODE -eq 0) { lfz --max-steps 10000 --timeout-ms 1000 run job.lfz -- '输入' }
```

check只加载/解析/构图，不执行程序；它不接受限额选项。run时才设置求值限额。未指定时默认unlimited，不自动继承上次运行预算。限额不是脚本argv：只有完整执行来源之后的`--`才终止选项扫描，之后所有字符串原样传入argv。

只支持独立两token `--max-steps N` 和 `--timeout-ms N`，不支持`--max-steps=N`或`--timeout-ms=N`。N是非空ASCII十进制u64，允许0及前导0，拒绝符号、空白、下划线、小数、溢出；每个限额选项最多一次。它们与--json、执行来源可前后交换；--json仍允许重复。check/test/help/version不能带限额，参数错误在读取源码之前拒绝，stdout空/stderr文本/exit2，即使带--json也是usage文本。

-e紧随的一个CODE原样保护，即使CODE就是限额名或等号文本；完整执行来源之后--的同名字符串也受保护。`lfz -e 'print(jsonStringify(argv()))' --max-steps 100 -- --max-steps 9`只把后两项传给脚本。不要把CODE内文本当作CLI配置。

## 步数的含义与零边界

每实际进入一个stmt和一个expr各计一步，在该节点自身副作用之前检查。预算N允许N次节点进入，减到0本身不失败，下一节点才报错。不是源码行数、循环次数、CPU指令数或毫秒数，不计parse/build图及Rust内部操作。

```powershell
lfz --max-steps 2 -e '1'
lfz --max-steps 4 -e "let x=1`nx"
```

两个例子恰好通过；单表达式1包括Stmt和Int两步，let x=1换行x共四步。少一步则ExecutionLimitError。空程序没有节点，--max-steps 0或--timeout-ms 0仍成功；有节点的程序timeout0在首poll报Deadline，max-steps0报StepBudget，二者同时启用先检查Deadline。实际函数/闭包/高阶回调/方法/模块初始化/模板默认值共享会话余量，缓存依赖不重复初始化。

## 终止、时间与真实边界

新增第13个错误类ExecutionLimitError；旧12类与旧错误JSON字段不变。三种reason对应固定消息：执行步数预算已耗尽、执行已取消、执行时间上限已到。CLI终止exit2，--json沿原错误对象，行列/跨文件fault来源真实；不把系统终止包装成普通业务Result。

attempt不能捕获ExecutionLimitError，也不恢复步数、延长deadline或重置取消token。终止前已发生的文件、容器、输出、RNG副作用保留，失败节点本身尚未开始；不提供事务回滚。

timeout从成功前端（含静态模块构图）之后、eval入口之前建立；不包含读取、lex/parse/build图及结果渲染。graph内部宿主结构校验的时间包含窗口，但坏图仍优先于节点poll。取消、deadline、步数检查优先级固定为cancel→deadline→steps。并发观测时刻不保证全局顺序。

仅stmt/expr入口协作检查；大range/JSON/排序/格式化或阻塞input/readFile等native调用内部不强制poll，不能承诺毫秒级硬响应。最后一次native完成后若无后续节点，即使期间到期也可成功。不是内存/文件权限沙箱、CPU配额或硬实时中断。

CLI没有取消token开关，也不承诺Ctrl-C会产生结构化Cancelled；Ctrl-C仍由OS默认处理。host可用Arc<AtomicBool>以Release写true，poll以Acquire读，不应跨线程传递LFZ Value/Env。重用已取消token意味着下一次有节点的调用也会取消，Interp不替host复位。

## 精确usage与Rust入口

缺值：`选项 '{flag}' 需要一个非负十进制 u64 参数`；非法或溢出：`选项 '{flag}' 的值必须是非负十进制 u64`；重复：`选项 '{flag}' 不能重复`；非执行命令：`执行限额选项只适用于运行命令`。

任一未受CODE/argv保护的等号族文本（包括等号后空或任意文本）按出现顺序拒绝：`选项 '{flag}' 不支持等号写法，请使用 '{flag} N'`。不会被help/version的旧额外参数规则忽略。Instant范围失败为`选项 '--timeout-ms' 超出当前平台可表示的时间范围`。

宿主可显式构造EvalLimits（max_steps:Option<u64>、deadline:Option<Instant>、cancel:Option<Arc<AtomicBool>>），默认三None；按值消费。三个新owned参数入口：

- eval_module_traced_with_args_and_limits(program,args,limits)：沿现on_eval_stack创建大栈。
- eval_module_traced_on_thread_with_args_and_limits(program,args,limits)：供已在大栈的调用者使用。
- eval_graph_traced_on_thread_with_args_and_limits(graph,args,limits)：已在大栈中消费C图，外层GraphExecutionError与内层LFZ终止区分。

签名与返回类型严格见interface-contract §22，旧公开入口保持签名且默认unlimited，不需要迁移旧调用。每次调用从max_steps建立私有remaining；Clone只共享token身份、不共享步数余量，绝对deadline复制。Rust穷尽匹配LzError的宿主须增加ExecutionLimit分支；这不是修改旧Span/TraceFrame或T15Result字段。此章Rust API部分是规范引用，不冒称文档角色已编译宿主程序。

R304仍OPEN，限额功能不等于unlimited分支零成本或整体效率通过。旧性能阈值必须对实际最终字节独立测量。
