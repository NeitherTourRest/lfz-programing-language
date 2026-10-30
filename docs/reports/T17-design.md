# T17 执行控制精确设计
日期：2026-10-30。已按team-lead独立ADR冻结。下列为interface-contract §22同文契约；实现尚待C独立验收后GO。历史draft/proposal不再作为实现接口。

## 22. T17 执行预算、取消与协作截止时间（2026-10-30 冻结）

依据DECISIONS中team-lead 2026-10-30 08:00独立ADR。本节新增第13个错误类，优先于旧章节“12类”的封闭计数；旧12类触发条件/消息及C接口不变。规范冻结不代表实现验收；源码须待C独立PASS后GO。

### 22.1 错误契约（core唯一写error.rs）
新增以下公开类型，既有LzError其余variants/Span/TraceFrame不改：
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ExecutionLimitReason { StepBudget, Cancelled, Deadline }
    LzError::ExecutionLimit { reason: ExecutionLimitReason, span: Span }
    pub fn execution_limit(reason: ExecutionLimitReason, span: Span) -> Box<LzError>
helper沿既有冷错误路径标记cold/inline(never)。class_name固定ExecutionLimitError，span()恒Some(span)。message仅按reason返回：StepBudget“执行步数预算已耗尽”；Cancelled“执行已取消”；Deadline“执行时间上限已到”。Display仍“类名: 消息”，不追加预算数/token/耗时，无新的JSON键。
Rust下游穷尽match必须新增分支；注册表从12机械扩13，原12行为测试不可删减或泛化成非空断言。R<T>不变。ExecutionLimit不是GraphExecutionError，不伪装Recursion/Overflow，也不属于attempt白名单。T15 Result仍三/五/三字段，系统终止不会被转换成r.error。

### 22.2 宿主入口（runtime唯一写evaluator.rs）
公开于lfz::evaluator，依赖仅std：
    #[derive(Clone, Debug, Default)]
    pub struct EvalLimits {
        pub max_steps: Option<u64>,
        pub deadline: Option<std::time::Instant>,
        pub cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    }
    pub fn eval_module_traced_with_args_and_limits(
        program: &Program, args: &[String], limits: EvalLimits
    ) -> TracedRun
    pub fn eval_module_traced_on_thread_with_args_and_limits(
        program: &Program, args: &[String], limits: EvalLimits
    ) -> TracedRun
    pub fn eval_graph_traced_on_thread_with_args_and_limits(
        graph: &ModuleGraph, args: &[String], limits: EvalLimits
    ) -> Result<TracedRun, GraphExecutionError>
limits按值消费；Clone只共享token身份，deadline值复制；每次调用重新从max_steps建立私有remaining，不共享余量。Default三项None。旧全部公开求值入口签名及返回类型保持，委托Default路径；旧run/eval_module默认unlimited，不额外增加无args/非traced/graph跨线程wrapper。本节不改变unsafe Transfer的约束，EvalLimits不持LFZ Value/Rc/Env。

第一新入口沿现on_eval_stack创建大栈；另外两个仅供已在大栈调用者使用。graph仍先完整validate_execution_graph，坏graph外层Err且无LFZ执行；有效graph的资源终止位于Ok(TracedRun{result:Err(..),..})，origins按C校验。前端失败/旧eval模块声明拒绝发生在求值前，优先于limits；库传入已过deadline不覆盖SyntaxError/InvalidGraph。解析/加载/图构建不扣求值预算。

### 22.3 单调计步与poll
执行会话持私有预算状态，只有一次Interp；模块初始化、导出函数、闭包、方法、graph模板默认值、高阶回调全共享。每次实际进入exec_stmt和eval_expr各算一个求值步；不是其inner递归以外另计，也不按Rust调用次数计。import/export经过exec_stmt各1步；缓存模块不重复执行。函数入栈、绑定、namespace构造、默认值诊断帧本身不额外计步；其中实际stmt/expr仍计。现有优化不得在有预算路径跳过语义要求的节点入口。
每个节点在其自身任何求值副作用前，以当前节点span及当前真实source执行一次poll，固定优先级：cancel置位→deadline已到→remaining==0。三者均未触发且remaining为Some则减1，随后进入节点。预算N允许最多N个节点进入；减到0不立刻失败，下一节点才StepBudget。u64减法只在大于0后，unlimited不累加计数、不允许wrap。无公开used_steps统计。
poll失败须通过现有note_error_span/source路径在故障点记录当前span/source，active/globals恢复前保存；不能直接在包裹层?返回而漏记fault。成功poll无需改写既有可见trace位置，避免改变非资源IO错误定位；“当前节点定位”指poll用该节点参数并在失败时先记该节点，不能使用上一节点位置。不限额路径沿旧trace行为。
零语句Program不进入节点，即使max_steps=0、token已取消或deadline已过也成功nil；空模块同理。没有入口额外poll，不为构图/空程序伪造语句。单表达式程序1有Stmt+Int两步：N=0在Stmt失败，N=1在Int失败，N=2成功。let x=1换行x共4步，N=4成功；供确定性边界见证。
节点路径相同则步数相同；不把HashMap探测、Rust格式化、graph验证计入。资源配置不改变普通表达式的结果与副作用顺序；终止前副作用保留，失败节点本身未开始，但其已执行的父/兄弟节点副作用不回滚。

### 22.4 attempt、来源与生命周期
remaining/deadline/token永远不加入attempt快照，成功/可恢复失败均不补步、不重置token或延长deadline。多层attempt不能吞ExecutionLimit；终止原样穿透、保留故障frame/source/depth。沿C每次函数/默认值执行的Ok/Err active/globals清理继续执行；终止不重置其它模块状态或重试初始化。函数ID/用户容器/IO/RNG不回滚。
legacy资源错误origins仍None；graph错误真实source/folded traceback沿C，不新增Result/JSON顶层字段。Cancel token由host以store(true,Ordering::Release)置位，poll load(Ordering::Acquire)；Interp不写/reset共享token。不同调用默认互不关联；host明确共享已取消token则各运行在下一节点取消。host不得把运行中的LFZ Value/Env跨线程共享。
deadline为绝对Instant，比较now>=deadline；不用系统日期/时区。各次poll按固定优先级，实际到期与并发置位的观测时刻不确定，不承诺同时事件的全局顺序。
本批只保证stmt/expr边界poll，不新增native内部poll或将其算步；大range/JSON/排序/format及input/readFile等宿主调用可能在下一poll前持续运行。末次native调用完成后若再无节点，可成功返回，即使期间超过deadline或token置位。宿主有需要可在返回后自行判断时钟，但不能改写本次已完成的LFZ结果。不是CPU配额、内存/文件权限沙箱或硬实时超时。禁止强杀解释线程或catch_unwind伪装资源终止；T18的外部进程/HTTP生命周期另定义。

### 22.5 CLI选项（tooling唯一写cli.rs/必要main）
仅执行命令（裸FILE、run FILE、-e CODE、run -e CODE、-、run -）支持：
    --max-steps N
    --timeout-ms N
N必须非空ASCII十进制[0-9]+，允许0和前导0，值在u64范围；拒绝符号、空白、下划线、小数及溢出。只支持独立两token，不支持等号简写。每项最多一次；与--json及彼此可任意选项顺序，允许前/后于执行来源；必须保持现输入来源数量/文件名判断，不增加新文件转义语法。
scanner先保护-e后的紧邻一个原始CODE token，即使CODE恰为--max-steps/--timeout-ms/--/--json也不识别为选项。-e此前的限额选项先摘除再识别CODE位置。已完整执行来源之后的--终止全部选项扫描，之后每个token原样argv；不得因剥除限额值吞掉--。限额参数必须在--之前；缺值/下个token是--视为缺值。未知选项沿旧规则拒绝。重复--json沿旧规则，不受本节每项唯一限制。
以下等价设置，argv均为["--max-steps","9"]：
    lfz --max-steps 10 run --timeout-ms 100 -e "1" -- --max-steps 9
    lfz run -e "1" --timeout-ms 100 --max-steps 10 -- --max-steps 9
check/test/help/version不接受限额选项，不静默忽略（即使原help/version会忽略多余token也先拒限额）；无新限额选项时旧行为不改。check -e CODE内文本仍受保护。解析所有限额、重复及命令适用性在读文件/stdin前完成，无LFZ副作用。

精确新增usage消息（沿现stderr文本及用法尾行，exit2；--json时仍非JSON usage）：
- 缺值：“选项 '{flag}' 需要一个非负十进制 u64 参数”
- 非法/溢出：“选项 '{flag}' 的值必须是非负十进制 u64”
- 重复：“选项 '{flag}' 不能重复”
- 非执行命令：“执行限额选项只适用于运行命令”
- Instant.checked_add失败：“选项 '--timeout-ms' 超出当前平台可表示的时间范围”
flag为完整--max-steps或--timeout-ms。单项遇到即报缺值/非法/重复；多项按argv自左向右，值解析错误优先于最终非执行命令检查。timeout转换Duration::from_millis(N)，成功前端（含静态build_graph）之后、调用eval入口前从Instant::now().checked_add建立deadline；checked_add失败走usage拒绝，无求值副作用。CLI提供的N是求值poll时间窗口，不含root/依赖读、lex/parse、静态build_graph和输出渲染；graph入口内部的宿主结构校验及Interp建立开销包含窗口，结构校验失败仍优先于节点poll。宿主绝对deadline可由host自行提前建立。不新增公开graph验证API或为deadline重复验证整图，不重读源码。
timeout=0且有节点时首poll Deadline；空程序成功。步数+时间+token均启用时按22.3，CLI不提供token开关。Ctrl-C沿OS默认，无跨平台handler/第三方依赖/结构化Cancelled承诺；后续须单独平台ADR。
资源错误exit2，--json仍唯一原形错误对象，class ExecutionLimitError、message固定、line/col/traceback真实；--json下程序输出仍stderr。成功exit0。host GraphExecutionError沿C stderr协议，不伪装LFZ终止或业务成功。

新增flag族等号边界：仅在未受CODE/argv保护的选项区域，token以--max-steps=或--timeout-ms=开头（含等号后为空/任意文本）时，扫描到即usage拒绝，精确消息“选项 '{flag}' 不支持等号写法，请使用 '{flag} N'”。flag是不含等号的完整名称；此错误按token出现顺序，先于最终命令适用性检查；不把等号形式当一次合法赋值或进入重复检查。因而--help --max-steps=1和--version --timeout-ms=1均exit2，不能被旧help/version忽略额外token放行。其它旧help/version额外token规则保持；不扩大匹配到--max-stepsX等其它名称。CODE恰为该文本或完整来源后--的同名argv仍原样保护。

命令拼写裁定（2026-10-30）：本节“help/version”仅指既有合法--help/-h与--version/-V；“check/test”指既有check/test子命令。不新增裸help/version命令别名，也不把它们列入非执行命令。裸help或version继续按既有文件路径/扩展规则处理，附合法限额时亦如此；可读取的同名路径不得因本节变成帮助命令或被“不适用于运行命令”拒绝。不存在/不合扩展名的路径仍走旧IOError，不应把此结果误判为漏拒帮助命令。--help --max-steps 1与--version --timeout-ms 1须报限额不适用；--help --max-steps=1等号形式按前述扫描规则立即报等号usage。其它旧help额外token规则不改。
计步范围再次明确：for语句本身及迭代来源表达式等实际进入的节点仍计步；未执行的空循环体不虚构节点，每次迭代的Rust控制逻辑不额外计步。大量迭代但没有新增stmt/expr入口不能据此承诺小预算终止。CODE中的双负号按既有lexer/parser/求值规则处理，不因与CLI选项相似而新增语法错误；check只检查语法/模块，不动态查名。上述均为既有节点与CODE保护规则的澄清，不改计步范围或新增语法。

### 22.6 Owner与可复验门禁
- architect：三spec §22、T17-design.md、ADR与本人记录。
- core：唯一src/error.rs新增reason/variant/helper和三个reason的class/message/span/Display单元验证，机械更新13类注册；不写evaluator/CLI。
- runtime：唯一src/evaluator.rs新API/预算、节点poll、attempt/跨module/default/fault保护及runtime单元；保持C/value/AST无额外公开字段迁移。除固定接口必须的消费外不写error.rs。
- tooling：唯一CLI/main选项与limits贯通、帮助、usage/JSON进程验证；老test runner和case helper默认unlimited，不给check/test隐式配置。
- test-engineer公开黑盒/应用owner/人AI指南各按宪法边界；verifier独立，perf最后冻结字节无并发测量。

分批A先error+步数/token，B再deadline+CLI；可同冻结契约但源码在C独立PASS后串行。最小验收：上述2/4步精确阈值重复；0与u64::MAX不wrap；空while/for/递归/7HOF在小预算受控，不能用真实栈溢出abort作测例；普通attempt连续业务失败不补步、嵌套终止不吞；模块diamond仅执行一次、初始化总预算、依赖失败entry无后续副作用；graph函数/模板默认的终止定位真实文件及caller清理；已置token/执行中置位/独立与明确共享token；过去/未来deadline与同时事件优先；有限运行结束后不偷偷追加poll。
库并发取消测试用受控worker与宽容watchdog，超时不能join无限卡死主测试或强杀解释线程；先小步数提供有限兜底，但断言reason必须Cancelled。deadline测试预过期确定性+有界到期见证，不断言毫秒级硬响应。CLI四类入口/任意选项顺序/CODE与argv保护/重复非法溢出/check-test-help拒绝/唯一JSON完整回归；旧默认无预算全套差分，graph坏元数据仍执行前拒绝。原19项性能在实际最终hash一次规定窗口比较，unlimited分支不能称零成本；R304保持OPEN，不自动改变原线。

§22与T18-A混合选项适用性：非执行命令同时带能力与限额时，按选项区首次出现类别报对应既有文案；全部扫描错误仍先于最终适用性，CODE/argv和无新flag旧行为保持。精确例子见interface-contract §23.5。
