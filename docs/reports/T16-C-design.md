# T16-C 冻结设计与接口交接

日期2026-10-30。责任人language-architect。实际B接口审查后获team-lead批准；三spec §21已冻结。B独立PASS后方可GO源码，本报告不宣称实现完成。

## 21. T16-C 单session模块执行（2026-10-30 冻结）

### 21.1 激活边界与公开宿主API

B独立PASS后方可实现/启用本节。只激活已由B检查的本地ModuleGraph，不新增语法、内置、动态import、远程包、循环live binding或并行初始化。check仍仅构图不执行。旧无模块CLI及旧eval_module*入口完全保持；旧eval入口遇模块Program继续§20 guard，不能绕过图验证。

runtime新增evaluator公开（GraphExecutionError derive Debug，并实现Display/std::error::Error）：
- pub enum GraphExecutionError { InvalidGraph { reason: String }, Origins(crate::source::SourceMetadataError) }
- pub fn eval_graph_traced_on_thread_with_args(graph: &crate::modules::ModuleGraph, args: &[String]) -> Result<TracedRun, GraphExecutionError>

该函数借用B图，调用方已在on_eval_stack内，parse/build/eval/drop graph全在该大栈。禁止重读图文件、重复parse或克隆全部Program。外层Err仅宿主坏图/元数据不变量，内部TracedRun.result保留LFZ成功/错误。InvalidGraph Display为reason，Origins Display为“来源元数据不一致：{error:?}”；不新增LFZ错误类，不让attempt捕获宿主错。

CLI宿主Err为stdout空、stderr精确“lfz: 模块执行上下文无效：{detail}\n”、exit2，属于合法JSON承诺中的宿主异常例外，不伪造LFZ业务成功/ValueError或InternalError大类。普通build_graph产生的合法图不得走该异常分支。

### 21.2 执行前图验证及内部状态

任何LFZ初始化之前完整验证：
1) entry与所有units.id有效，units[i].id=ModuleId(i)，每unit唯一source_id且存在、loaded为Some；
2) initialization_order是units的无重复全覆盖排列，entry最后；每import target存在且在导入者之前；
3) 每unit.imports与Program直接ImportDecl数量/书写序/alias及span一致，alias不重复且不与其它直接绑定冲突；export metadata与Program导出名/顺序/span一致，名字有直接顶层绑定且不重复；import仍先于非import。
失败返回InvalidGraph，reason可取稳定宿主原因，不改语言消息，零LFZ副作用。该检查是结构一致性，不重新parse或证明宿主手工伪造AST与文本对应，不构成安全沙箱。metadata无法通过A校验返回Origins，不expect/unwrap掩盖不变量。

Session保持单Interp，新增私有模块实例表，实例持该模块Rc<RefCell<Env>>、可选namespace及Uninitialized/Initializing/Ready/Failed状态。模块实例表只归本次执行，不用全局static缓存。graph由入口借用，Env不持整个Program。现globals作为active模块Env别名，active_module与它一致；UserFn只整数home_module及captured cells，不强Rc持home Env。func_names/next_func_id统一分配，模块间不重置，不被attempt回滚。

### 21.3 初始化、导出与结果

按B initialization_order依赖后序串行初始化，每模块新独立globals。初始化前按ResolvedImport书写序绑定不可重绑定alias，值为目标Ready namespace同一Rc；未Ready是InvalidGraph，不给空对象继续或重复执行。graph模式ImportDecl/ExportDecl仅声明动作，返回Flow::Value(Nil)，不做IO或加载；legacy模式继续guard。普通语句按旧规则，结果为该Program最后Flow Value/Return，防御性Break/Continue仍Nil。

模块顶层push独立MODULE_FUNC_ID=0帧，名称仍<module>、source为该unit真实来源；初始化成功清理该模块帧，下一模块不嵌套伪造根执行栈。失败立即停止整个session，不执行剩余计划或entry；故障trace只含实际已进入的模块/函数。图阶段与依赖初始化错误不在尚未运行的entry attempt边界内。无隐式初始化重试。

成功后按ExportBinding书写序读取最终模块直接绑定，构造新普通struct namespace，Ready。无导出为空struct。标量是完成时快照，数组/struct/函数按现引用拓扑共享；namespace字段可按原规则修改，不回写模块变量绑定。多alias/菱形复用同namespace与一次初始化副作用。keys/JSON仍A5过滤方法，不作为导出函数反射。entry namespace也可建立，但最终TracedRun.result是entry求值结果，不自动返回namespace或业务JSON。

所有模块argv使用同一显式args快照，每次argv调用仍新数组。module身份与cache只在本次session有效，不新增跨session函数Value调用API。IO/RNG及初始化次序可观察，不并行。

### 21.4 UserFn home切换与故障来源

UserFn现home_module在graph make_closure记录active_module。call_user先沿旧顺序做argv/placeholder/arity/递归检查；普通函数在定义模块globals下建fresh参数帧，捕获cell及self_override优先级保持。保存调用者active_module/globals，切home后执行，但必须显式收集outcome而非让?绕过上下文清理。成功和Err都恢复调用者active/globals；成功才沿旧规则pop trace/depth，Err保留故障trace/depth。

新增trace_sources与trace严格同长度，每个module/函数/默认诊断帧push/pop同步。实际错误处保存fault_source，或从尚完整的故障帧来源获取；恢复active之后不可用当前caller猜错误文件。未压callee之前的参数/arity错误属于caller。最初故障来源在冒泡中不能覆盖为外层来源。

attempt保存并在成功/可恢复失败后还原trace_sources、fault_source、active_module/globals及旧完整trace/depth；先用故障trace构造旧Result。不可捕获错误保留故障帧/来源，callee的active/globals安全清理仍执行。func_names/next_func_id、module实例及namespace、用户cell/IO/RNG不回滚。T15 Result/error/frame字段三/五/三不增file；跨模块Result只有各帧真实行列，无文件名，指南不得宣称完整文件定位或把路径拼进func字段绕过协议。

图执行结束构造TracedRun并一次attach_origins(graph.sources,trace_sources,Err时Some(fault_source))；成功error_source=None；A校验失败向外Origins传播，不panic或错误文件fallback。legacy无模块入口继续origins=None。

### 21.5 StructDef默认值定义上下文

不新增lib.Point{}语法；导出模板用“let P=lib.Point”再“P{}”。模板/默认值实际所有权由runtime统一实现。

value.rs新增公开TemplateContext（derive Clone/Debug）：
- pub home_module: crate::source::ModuleId
- pub captured: Rc<Vec<CapturedVar>>
- pub self_val: Option<Value>
- pub defaults_func_id: u32

StructDef新增pub default_context: Option<TemplateContext>。旧Value::struct_def与struct_template签名保持，填None；新增pub fn struct_template_in_context(name: impl Into<Rc<str>>, fields: Vec<(Rc<str>,Expr)>, methods: Vec<(Rc<str>,Rc<Closure>)>, context: TemplateContext) -> Self，填Some。外部手写StructDef literal需补字段，属于明确Rust构造迁移；不得顺手改旧Value语义。

legacy无模块声明始终None，eval_struct_lit继续旧实例化caller Scope默认值行为，旧trace不加帧。graph模式声明为Some，记录定义home、按现make_closure捕获算法升级/继承可见局部cells并捕获self；不强Rc持Env。默认表达式在定义globals+captured/self Scope按字段声明序重新求值，每实例可变默认独立；显式字面覆盖字段在所有默认值完成且恢复caller上下文之后，仍按caller Scope/书写序求值。graph定义上下文是新模块语义，不宣称legacy从来如此。

为了默认Expr真实来源诊断，graph声明若存在默认字段，分配session全局唯一defaults_func_id，func_names显示名精确“{TemplateName}::<defaults>”；函数ID不复用不回滚。TemplateContext无默认字段时仍可分配该ID，但实例化不额外push空默认帧。调用用户方法仍原名，不把模板diagnostic frame冒充普通可调用Value。

默认字段非空时，在切home/push之前按与call_user相同RECURSION_LIMIT检查depth；已到上限则在caller构造span返回原RecursionError，不进入home。否则先更新caller span为构造点，depth加1，切定义上下文，push defaults_func_id/首默认Expr span及定义SourceId。每字段eval_expr更新该帧span，错误保存定义fault source。完成全部默认值或任何Ok非Value Flow时，先pop该诊断帧/source、depth退1、恢复caller active/globals再继续/传播Flow；任一Err恢复active/globals但故障帧/source/depth保留，交外层attempt或最终错误处理。

该边界不改变Flow::Return/Break/Continue传播规则，不把默认Expr当一个吞return的新LFZ函数；它只是来源/递归保护诊断帧。禁止graph默认值递归构造绕过depth保护，资源错误不被attempt捕获。相关上限验证用安全合成state或有界单元，不真实制造stack abort/OOM。不得为跨模块默认值复用caller帧而在错误文件画下划线。

当前仓库StructDef构造点为Value::struct_def/struct_template；runtime调用声明路径用struct_template_in_context，保留旧helper None。新增context字段及构造机械迁移列入作者报告，不删除旧默认值行为测试。

### 21.6 CLI、真实文件与owner

runtime唯一value.rs/evaluator.rs：图入口/结构验证/session、函数及模板context、来源记录与attempt恢复；不改B loader/modules/AST/source。tooling唯一cli.rs与必要main说明：run file/-e/stdin根counted/load/lex/parse一次，若模块build_graph→新eval→stack内drop；check只build/drop。无需新语法或额外CLI选项。

模块runtime错误CaseEval由run.error_source提供最深真实display_path/Loaded。push_error_fields.file及人类错误源码摘录使用该source；traceback_json每帧file及push_trace_frame按实际帧下标查run.frame_source。首10尾30折叠须传原实际index，不能切片后enumerate从0；JSON仍全量不折叠，原字段/键序保持。

旧origins=None路径仍用原path/source逐字节；graph路径缺sidecar/来源是宿主Origins/InvalidGraph，不得伪装legacy fallback。无Loaded不借其它文件源码。初始化错误只渲染实际进入的帧，前端GraphFailure仍原B前端定位协议。

### 21.7 验收与实现GO

B独立PASS后方GO源码；runtime单元/整库先，tooling整合唯一编译，再公开与独立验收。必须验证三文件/菱形单次初始化/两次执行隔离，caller同名全局不串，home闭包/方法/self/局部模板/默认值及显式覆盖来源，导出快照/共享容器/namespace写入，统一argv，初始化失败零entry副作用，attempt跨模块恢复后新错无旧帧/来源，模板默认值资源保护，坏宿主图零副作用拒绝，真实跨文件人类/JSON/折叠trace以及全部legacy T15/T14/JSON/apps。check保持不初始化，C不改B限额或T15协议。

新模块热路径与二进制变化必须实际性能测量后才评价效率，R304仍OPEN。规范冻结不等于实现或正式发布完成。
