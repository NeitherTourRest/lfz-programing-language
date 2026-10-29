# T16-C 模块运行时设计草案

日期2026-10-29。责任人language-architect。仅草案，B源码接入与独立验收后再冻结，不改spec或产品。A/B既有签名及T15Result保持。

## 可交付结果与范围

消费B已检查ModuleGraph快照，使本地模块实际初始化、导入namespace、导出值和跨模块函数调用可用。静态路径、循环拒绝、图预算继续归B。无动态import/远程包/循环live binding/热重载/并行初始化，不再读取图已加载文件。

建议新入口eval_graph_on_thread_with_args(graph:&ModuleGraph,args:&[String])->TracedRun；借用图期间所有Program及结果诊断移交在既有大栈机制内，CLI最终drop graph也在该栈。旧eval_module*只支持无模块Program且继续B guard，避免旧API绕过静态图验证；CLI对模块root使用新入口，无模块完全旧路径。精确接口冻结时再与现B实现核对。

## Session与Env身份

同一Interp拥有整个session、模块实例Vec、全局func_names/next_func_id、统一script_args和逐帧source信息。ModuleInstance建议{env:Rc<RefCell<Env>>,namespace:Option<Rc<RefCell<StructObj>>>,state}，state为Uninitialized/Initializing/Ready/Failed。先为每个图unit建独立Env，后按initialization_order顺序初始化；模块缓存只属于本次Interp，绝不静态跨case复用。

保留现Interp.globals作为当前模块Env别名，增加active_module；进入初始化或函数调用时由模块表选择Env，不能混用调用者globals。UserFn.home_module只存ModuleId，函数/模板不强Rc持home Env，避免Env→函数→Env必然引用环。函数表在session内全局分配，失败/attempt不回滚func_names与next_func_id，防逃逸闭包ID错配。Rust旧API没有通用跨session调用Value，C也不新增；ModuleId不能被另一session解释。

模块初始化先绑定其imports（不可重绑定，按import书写序），值为目标Ready namespace同一Rc。所有依赖应已Ready；不满足是内部图/运行计划错误，不能悄悄重复执行或给空namespace。exec ImportDecl/ExportDecl在新graph模式只执行声明作用，不重新加载模块；旧eval模式仍拒绝。在B保证无循环后，Initializing是防御状态，不支持半初始化live导出。

模块普通语句执行成功后，按export声明书写序读取本模块当前顶层绑定值，建立普通struct namespace，标Ready；无导出为空struct。标量为初始化完成时快照，容器按A1共享；namespace字段可改，改它不重绑定模块顶层。导出函数/结构体方法仍A5数据面过滤，keys/JSON不能当列举所有函数的反射。若需动态获取模块变量，显式导出getter。导入多个alias/菱形边共享唯一namespace实例与初始化副作用。

每个模块顶层初始化独立module frame，func_id仍可用MODULE_FUNC_ID=0显示<module>，真正来源靠逐帧SourceId；在lib初始化失败前root尚未执行，不伪造root执行帧。初始化任何失败立即终止session，不运行其它模块或entry，不提供捕获后重新导入重试。module graph阶段失败也不在attempt执行范围内。所有模块初始化顺序是可观察副作用，不并行。

## 调用home globals及恢复

call_user先按原规则检查builtin argv/placeholder/arity/recursion；普通用户函数按home_module取定义Env与SourceId，在该Env之下新建参数帧。保存调用前active_module/globals，设置home上下文，再按原trace压入函数帧。闭包捕获cell、自定义self_override优先级、形参可变性与词法遮蔽都保持；自由全局读取定义模块而非调用者模块。方法内lambda、逃逸闭包和递归均记录定义模块。

调用成功和失败都必须恢复调用者active_module/globals；失败的trace及其来源保留供诊断，不能因恢复active丢失故障来源。原depth在不可恢复错误传播时按既有方式保留，attempt随后恢复受控边界；不要为module恢复同时把故障帧pop掉。

attempt快照增加active_module/globals（以及新逐帧source序列），成功/可恢复失败完整restore；先基于故障trace构建原固定Result，再restore。不可捕获错误继续保留trace/source，不回滚模块对象/namespace/用户IO/RNG/函数ID表。T15 Result/error/trace三/五/三字段不增file；跨模块Result行列是真实各文件坐标，但没有文件名，限制须直说，不能把func显示名拼路径绕过协议。

## 逐帧来源与最终诊断

Interp维护与trace严格同长度的trace_sources:Vec<SourceId>；每次压帧、成功pop、attempt恢复同时处理，top-level module frame也有实际SourceId。故障source从故障帧来源或错误发生时显式保存，不能在call_user恢复active之后用当前模块猜。参数校验错误在新callee frame之前发生，定位调用者；函数体错误定位定义模块。多层错误传播不覆盖最初error source。

完成TracedRun时使用A attach_origins一次附加graph.sources、trace_sources与错误source；成功error_source=None，错误必须Some且有效。所有校验失败是宿主元数据错误，不用入口file fallback。CLI runtime JSON仍原字段/键序，但file与每个traceframe.file真实跨文件；人类下划线使用错误SourceRecord.loaded，None则无摘录。不变更无模块脚本的现字节显示。

StructDef.fields中默认值Expr当前未记录home module，这是必须专门审查的路径：模块导出struct后，在其它模块实例化默认表达式应使用定义模块globals/source，不能只处理UserFn遗漏字段默认值。建议模板增加整数home_module（与UserFn同类Rust literal迁移），模板实例化保存/恢复上下文；或明确冻结默认值定义上下文的等价独立载荷。必须先审实际代码与测试再定案，不能用“方法已经有home”宣称模板全覆盖。

## owner与实施顺序

runtime唯一改evaluator/value及新Session内部实现；core仅在发现B图/API需要补钉时由team-lead另派，不擅改source/B AST。tooling接新graph求值入口及真实跨文件render/JSON；先运行时单元，再CLI整合唯一编译；spec由architect冻结后才动。

验证：三文件、菱形初始化一次、重复路径、两次执行缓存隔离；caller同名全局不污染callee；闭包/方法/self/默认字段表达式home来源；导出快照/共享容器/namespace写回；module argv统一；lib初始化失败零root副作用；attempt跨模块捕获后新错误无旧帧/来源；所有旧T15/T14/JSON/apps；B图预算与check无初始化保持。新fixture避免runner误发现库。需最终实际字节性能回归；R304仍OPEN，功能不代替性能。

本草案不承诺C已实现，不改A/B/T15规范。最高风险是模板默认值与函数home的共同定义上下文，以及active恢复与故障来源保存两者不可混淆。
