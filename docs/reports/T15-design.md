# T15 attempt 冻结设计
日期：2026-10-29。team-lead已确认，规范性定义为三spec §18。

## 18. T15 受控错误恢复（2026-10-29 冻结）

新增内置调用 `attempt(f)`，不新增记号、文法或try/catch。f必须是能以零实参调用的函数值；用 `attempt(() => jsonParse(text))` 延迟可能失败的表达式。管道 `f |> attempt` 沿原脱糖规则等价。attempt仍可被局部/捕获/全局同名绑定遮蔽；attempt本身不是一等函数值，既有argv一等范围不扩大。普通参数表达式仍左到右先求值；求参错误不在本次attempt捕获范围内。

返回普通struct，成功/失败及错误字段见semantics §18、interface-contract §18。`lfz check`只解析上述调用，不检查回调参数个数或执行回调。不改变文件前导、旧run/test JSON或未捕获错误。
## 18. T15 attempt 的恢复语义（2026-10-29 冻结）

`attempt(f)`恰一参；先完整求参，再校验函数值与零参数可调用性，然后进入恢复边界并调用一次f。返回新struct，字段插入序固定 `ok,value,error`：成功为 `{ok:true,value:v,error:nil}`，v是原返回值（含nil/数组/struct/函数），不深复制；可恢复失败为 `{ok:false,value:nil,error:e}`。e为新struct，字段序 `class,message,line,col,traceback`；类名/消息直接沿既有LzError，位置是1-based整数，无位置为nil。traceback是失败发生时完整外→内帧数组，每帧新struct序 `func,line,col`，func按现func_id命名规则为命名函数、`<fn>`或`<module>`。包括原调用者链，不加虚构attempt用户帧、不折叠。Interp没有文件路径，错误及帧不提供file。记录在清栈前形成；它是普通可变数据，修改不会反向修改解释器。普通jsonStringify沿A5等原规则，不能保证含函数/环的成功value可完整JSON编码。

边界之外的校验：attempt参数数目错沿`函数 attempt 期待 1 个参数，得到 {m}`；非函数沿`不可调用：{t} 不是函数`；用户回调需要非零参数时沿`函数 {name} 期待 {n} 个参数，得到 0`。name沿原规则匿名为`<fn>`。这些TypeError直接传播，不作为本次失败Result，也不调用f。既有一等argv可作为零参数函数，返回当前argv快照。placeholder函数值缺用户体且非argv时沿原NotCallable(function)。外层另一个attempt仍可捕获其内部发生的上述TypeError。

捕获白名单：NameError、TypeError、IndexError、FieldError、ZeroDivisionError、ValueError、IOError、AssertionError，以及OverflowError的IntegerOutOfRange。assert/fail在边界内成为失败值，边界外仍致命。check(false)仍打印原警告并返回false，不是错误。JSON128层输入验证仍是ValueError，可恢复。

不捕获：CosmosAnswerError、SyntaxError、RecursionError、OverflowError::Capacity；保留原错误与帧/深度状态继续传播。宿主panic、abort、栈溢出、分配失败不是业务错误，禁止catch_unwind来伪装Result。本版IOError仍是原运行时IO错误类别，不能宣称可区分所有底层OS资源故障；未来预算/取消终止必须走不可捕获路径。

边界快照在求参/校验后、回调调用前取得。成功及可恢复失败都恢复调用前depth和完整trace（包括调用者原span）；仅truncate帧不足以恢复被改写的调用者span。嵌套attempt各保存自己的边界，内部恢复不破坏外部。不可捕获错误不恢复后继续执行，沿原异常通路保留最内错误帧。恢复不回滚可见副作用：全局/捕获cell、容器修改、IO、输出、随机状态以及闭包分配都保留；不提供事务。func_names和next_func_id不可回滚，否则失败回调写出的逃逸闭包会在后续trace中错配。
## 18. T15 attempt 实现契约（2026-10-29 冻结）

### 18.1 注册及ABI

内置集合由72增至73，BUILTIN_NAMES字母序含attempt、is_builtin("attempt")为true。它是需要Interp上下文的特殊内置，不伪装普通Builtin/Hof；lookup/lookup_hof不登记attempt。无Interp的builtins::call/call_with("attempt",...)保持通用表未找到名字的NameError，不模拟空trace。旧Builtin/Hof/Invoke公开ABI不改，不泛化函数值。

eval_call的Ident路径先执行原变量遮蔽查找；仅无绑定且名为attempt时按原eval_args一次求完实参，控制流仍按原规则传播，再调用私有Interp helper。不得通过再次求值AST重跑实参。helper依semantics §18先校验1参/Func/零形参（argv允许、无用户体placeholder拒绝），校验错误span取attempt调用Call节点。未进入f不新增帧；helper内部经原call_value/call_user调用f，保留fresh环境、self捕获、参数/cell及原深度检查，不造attempt帧。

### 18.2 状态、错误与数据

semantics §18的三字段Result、五字段error、三字段traceframe及插入顺序是规范性协议；构造用Value普通struct/array/string/int/nil，不手拼JSON。错误span与消息逐字复用LzError。函数显示名复用现规则；错误记录在恢复前取完整trace，存普通Value字符串，不保留对可变内部trace的别名。成功value保留原引用拓扑。

可恢复分类须匹配Rust错误variant，OverflowMsg::IntegerOutOfRange可恢复，Capacity不可；不靠中文message字符串判断。snapshot trace.clone与depth在实际调用前获取；成功/可恢复Err恢复整个快照，不回滚func_names/next_func_id/用户状态。不可恢复Err原样返回，不能先清帧。此操作不要求更改旧call_user的错误保帧行为；仅新helper在受控边界处理。

### 18.3 最低验收矩阵

成功nil/标量/共享容器/返回函数；多行坏JSON混批；全部可捕获类及白名单外错误（资源错误优先单元构造验证，禁止真实abort压测）；outer arity/nonfunc/callback arity/实参求值错误位于边界外；argv、遮蔽与管道；嵌套成功/失败/外层接住内部校验错误；连续大量失败后成功、恢复后正常递归；完整trace名称/位置及失败后后续未捕获错误无旧帧；捕获失败前写出的闭包后续调用及trace标识不串；数组/cell/输出/IO不回滚；check(false)仍警告且成功；旧542测试和CLI/check/JSON协议回归。人/AI指南只引用本节并实跑，不以失败数等于0代替上述状态恢复见证。
本报告为设计与测试输入，未实现、未测量、不宣布功能或性能通过；R304仍OPEN。
