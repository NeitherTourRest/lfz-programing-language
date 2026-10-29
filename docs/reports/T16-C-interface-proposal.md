# T16-C 实际B代码审查与最小接口提案

日期2026-10-30。责任人language-architect。状态：待team-lead裁决，尚未冻结§21。B全release/hash及独立验收尚在进行，本文不授权提前改源，不修改B §20。

## 实际代码事实

src/modules.rs已有ParsedEntry/ModuleGraph/ModuleUnit/ResolvedImport/ExportBinding/GraphFailure和迭代DFS，图初始化计划为initialization_order。SourceMap只诊断载荷，图持全部Program；CLI check已有counted根、captured cwd、真实GraphFailure来源渲染、stack内drop。root/source计数与所有权不需C再创造接口。

evaluator当前全部全局查找、scope边界判定、闭包捕获都依赖单个globals；UserFn已有home_module但legacy为ENTRY。TracedRun已有可校验origins sidecar、安全frame_source/error_source；call_user目前Err经?提前返回，故障trace/depth故意保留。C若仅切globals后沿旧?返回，会漏恢复active/globals。必须显式收集outcome再在Ok/Err两路径恢复调用者上下文；失败帧/source保留。

StructDef目前没有home/captured上下文；struct_template只有name/fields/methods参数。eval_struct_lit对def.fields直接用实例化caller Scope求值；模板名是StructLit.type_name:Option<String>，没有namespace.Field{}语法。因此首版模块模板使用let P=lib.Point后P{}，不增加字段路径实例化文法。既有无模块动态调用scope默认值不能因C静默改为定义scope。

## 推荐唯一新执行入口

新增evaluator公开：
- pub enum GraphExecutionError { InvalidGraph { reason: String }, Origins(SourceMetadataError) }，derive Debug。
- pub fn eval_graph_traced_on_thread_with_args(graph: &ModuleGraph, args: &[String]) -> Result<TracedRun, GraphExecutionError>。

函数要求调用方已在on_eval_stack，借用图而不重复读/parse。外层Result仅宿主坏图/身份或sidecar不变量错误，内部TracedRun.result仍既有LFZ成功/失败。不要把坏宿主ModuleGraph当用户业务ValueError或可捕获attempt，不新增LFZ错误类。CLI正常build_graph生成图应满足不变量；宿主异常明确stderr“lfz: 模块执行上下文无效：{detail}”、stdout空、exit2（同宿主异常例外，不伪造业务JSON）。若team-lead选择不同宿主异常协议，须批准后再冻结，不能在实现时临时新增InternalError大类。

旧eval_module*入口继续仅支持无模块Program，保留B preflight；CLI file/-e/stdin实际执行时加载、lex、parse一次，若有模块声明build_graph后走新入口，完整图错误在任何初始化前失败，无模块仍旧eval结果路径。run与check都使用同一静态图而check永不调用新eval入口。图及AST全过程在大栈内析构。

## 运行前宿主结构验证

在任何LFZ初始化前验证：entry/各unit.id为有效且唯一的units索引；source_id均存在；initialization_order恰覆盖units且依赖目标在导入者前；每个ResolvedImport target存在，alias/exports不重复；来源meta不缺失；entry存在且在计划末端（build_graph根可唯一连接所有units）。验证不是重新读取文件或一般LFZ静态分析。bad graph返回InvalidGraph且零LFZ副作用，不panic或给空namespace继续。正常图limits仍B处理，不二次收紧。

## Session内部形状和所有权

无需公开Session结构；单Interp新增可选graph session state，推荐私有ModuleInstance { globals: Rc<RefCell<Env>>, namespace: Option<Value>, state: ModuleState }，state Uninitialized/Initializing/Ready/Failed。namespace必须是Value::Struct，Option用于未就绪；全部实例Env预分配，graph生命周期由入口借用管理，不复制所有Program。

现globals保留为active实例Env别名，active_module为其ID，函数全局名字解析以home_module切换到定义Env。ModuleInstance持Env，UserFn只ID与captured cell，不持强home Env；不为普通模块创造必然Env→函数→Env环。T15函数ID表全session统一，不每模块重置；图内多顶层MODULE_FUNC_ID=0显示<module>，来源由trace_sources区分。

按initialization_order顺序初始化，每模块先绑定其imports为不可重绑定alias（按源码序），目标必须Ready且复用同namespace Rc。exec ImportDecl/ExportDecl在graph模式只返回Nil，不做加载或求值；旧eval模式仍B拒绝。其它语句照旧执行，导出声明本身不产生回调/IO。

成功后按export书写序读取最终本模块顶层值建立普通struct namespace，无导出为空struct；标量快照，容器/函数共享引用拓扑，namespace字段修改不回绑定模块全局；多alias/菱形复用同对象。不是live binding，不冻结namespace。A5 keys/JSON继续过滤函数字段，不能当导出反射。失败即终止整session，不初始化entry或其它计划项，不支持重新import重试。

argv在所有模块取同一显式session脚本参数快照，每次仍新数组；IO/RNG初始化顺序可观察，不并行。方法调用self沿原字段调用规则，函数home只决定globals与source，不改变self_override优先级。

## home调用与fault来源

call_user参数校验/递归上限先沿旧顺序完成。普通用户函数取定义ModuleInstance，再记录调用前active/globals，切到home，在home globals下建fresh参数Env；captured cell和self照旧。outcome无论Ok或Err都恢复调用者active/globals，成功才按旧规则pop trace/depth，Err保留故障trace/depth。

新增trace_sources与trace严格平行，每次module顶层/函数/默认值诊断帧push/pop同步。fault_source在最初实际错误处或故障frame来源保存，恢复active之后不能按当前调用者猜file；调用参数错误尚未压callee，位置属于caller。module初始化失败时root未执行，不伪造root帧。

attempt快照同时增加trace_sources、fault_source及active/globals；成功/可恢复失败还原完整快照，先按旧trace构造原固定Result。不可捕获Err保留故障trace/source，原callee切回caller active的安全清理仍执行。预算/函数ID/用户副作用不回滚，未来T17 remaining不能成为attempt快照。

最终TracedRun用A attach_origins绑定graph.sources、逐帧来源与Err的fault_source；验证失败向外GraphExecutionError::Origins传播，不panic或fallback错文件。T15Result严格保持error五字段/trace三字段，跨文件Result只有真实行列没有file，指南明确此限制；不把路径塞函数显示名绕过。

## StructDef默认值：建议显式可选定义上下文

推荐新增私有或公开宿主结构TemplateContext（确切可见性待批准）：
- home_module: ModuleId
- captured: Rc<Vec<CapturedVar>>
- self_val: Option<Value>
- defaults_func_id: u32

StructDef新增default_context: Option<TemplateContext>。旧Value::struct_def/struct_template签名保持并产生None；legacy无模块声明仍None，eval_struct_lit继续当前caller Scope默认值语义，保证旧程序输出不变。graph模式struct声明产生Some：复用make_closure的可见局部cell升级/继承算法捕获定义局部，记录定义self/module；只存cells不持Env，避免强环。新增context构造入口或runtime直接建StructDef均由value/evaluator owner统一，外部手写StructDef literal需同步新增字段，明确Rust构造迁移。

graph Some默认值在定义模块globals+captured/self的Scope按字段声明序重新求值，每实例可变默认值独立；显式覆盖字段仍回到实例化caller Scope求值。声明内局部捕获可逃逸且共享cell，不能只切home globals后遗失定义局部；其引用语义按A2。加入import后的graph模式默认值采用定义上下文是新模块语义，必须明确，不把它说成旧legacy默认值从来如此。

默认值表达式Span属于定义文件，不能沿caller frame来源画错下划线。建议Some上下文分配全局唯一defaults_func_id（显示名"{TemplateName}::<defaults>"），默认值执行压独立诊断帧/source，保存/恢复active/globals，失败保帧；帧参与递归预算，防递归默认构造无call_user时绕过深度限制。它是诊断/预算边界，不吞掉原Flow::Return/Break/Continue；Ok非Value仍按原eval_struct_lit向caller传播，先清理成功帧再回传。模板字段初始化全部结束后pop并恢复caller，再求显式字段。legacy None不增加该帧，不改旧错误trace。

这比只加home_module复杂，但避免模板默认字段在跨模块读错globals/locals或画错文件。若team-lead要进一步裁剪，应显式限制模块导出模板支持，不能悄悄把默认值错误上下文当已实现。

## CLI实际路径与owner

runtime唯一value.rs/evaluator.rs：graph入口/内部validation/session、函数与模板定义上下文、trace_sources/faultSource与attempt交互；不改B graph/AST/loader。core仅在另派补B缺口，不让runtime顺手改source.rs或模块图。

tooling唯一src/cli.rs必要main说明：复用B根counted路径与ParsedEntry，run模块分支build→eval→drop，check仍只build→drop。CaseEval外层当前持path/Loaded，模块runtime Err时以run.error_source取最深真实file/Loaded；push_error_fields的file和人类错误摘录用真实source；traceback_json与push_trace_frame按实际frame索引查run.frame_source，不按func_id猜source。旧origins=None路径原样使用旧path/source，不能把模块坏meta当legacy None静默fallback。

人类trace首10尾30折叠时也要保留原实际index，不能切片enumerate后从0重新查source。JSON完整trace不折叠，每帧file真实；格式/顺序保持。无Loaded不画其它文件文本。

## 需team-lead裁决再冻结

1. 新graph API外层GraphExecutionError及宿主异常呈现，不新增LFZ错误类。
2. graph模板默认定义上下文Some、legacy None保动态caller语义，及独立默认值诊断帧。
3. Runtime/CLI单写者签名冻结后，B独立验收完成才改源。

最低验收：三文件/菱形一次初始化/执行缓存隔离，caller同名全局不串，home闭包/方法/self/模板默认及显式覆盖分属source，失败IO副作用保留，attempt跨模块恢复后新错误无旧source，init失败零root执行，旧T15/T14所有公开回归逐字节。性能需最终字节另测，R304仍OPEN，本提案不宣称C已实现。
