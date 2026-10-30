# T17 CLI只读接入审查与最小实施方案

日期2026-10-30 12:55。tooling-dev。只读审查src/cli.rs、src/test_runner.rs及runtime已草成的EvalLimits/API；契约为三spec §22/T17-design。此阶段不改任何源码、不编译、不运行新产物，不声称功能通过。

## 已有路径及风险

1. split_options当前返回(rest,script_args,json)，execute和测试用parse_args消费；仅顶层/run/check的-e紧邻token保护。execution_ready负责执行来源后--切换。新增flag必须就在此单次扫描中提取，不能在扫描前全局过滤，否则CODE和argv损坏。
2. Command旧variants受bin旧测试直接比较，应保持形状；原始CLI限额独立持有max_steps/timeout_ms两个Option<u64>。parse_args可委托新增扫描器、仅抽Command，原测试不需改期望。
3. help/version现在直接忽略额外token。必须先完成限额扫描（包括错误），再parse_command，最后检查非执行命令适用性，不能因help提前成功忽略新flag。check/test同样拒绝真实选项；CODE内文字不算选项。
4. run file/source共用eval_loaded_on_stack，graph在该函数parse后构建；deadline必须分别放在legacy即将调用eval和graph构建成功即将调用graph eval处，不在execute或根加载前建立。Default/仅max_steps不得调用Instant::now建立无用deadline。
5. checked_add失败是CLI usage，不是C的HostFailure。不能输出“模块执行上下文无效”前缀，也不能伪造LfzError。建议新增内部受限流水线Result<CaseEval,String>，成功为既有CaseEval、Err是精确usage；各前端/GraphFailure仍Ok(CaseEval错误)以保持优先级。旧eval_case/runner走Default无限制包装且不会产生deadline转换错误，不给test隐式预算。不要把所有CaseEval错误统归usage。
6. src/test_runner.rs::ERROR_CLASSES目前硬编码12，应机械加ExecutionLimitError为13并新增注册见证，原12测试原样；原judge已对host异常fail-closed，不能倒退。注册不代表默认runner隐式启用限额。
7. 有模块source/path和folded frame index保持C实现；仅转调新with_args_and_limits API，不重读/重复parse/重复graph结构验证，不把Rc/图移出大栈。

## 精确扫描算法

自左向右，先延续-e原始CODE保护和完整来源后--原样argv，再识别--json/两项限额。此前限额项已摘除，rest中的来源形状才用于判定-e。flag遇到时先查该flag是否已出现（重复立即报“选项 '{flag}' 不能重复”），然后要求下一token存在且不等于--（否则缺值），再要求非空ASCII数字串并parse u64（非法/溢出按精确消息）。不能吞--作值，不接受+、-、空格、下划线、小数或Unicode数字。0与任意前导0合法；u64::MAX合法，不按字符数任意截短。

两项彼此及--json可任意选项区位置；源码/--后完全不扫描。全部限额值错误按argv出现序优先，之后才判断check/test/help/version是否带有效限额并报“执行限额选项只适用于运行命令”。没有真实新flag时旧help/version/test行为保持。等号简写不支持，应在实现和公开测试中明确拒绝；对help后--max-steps=1这类旧help忽略多余token与新简写禁止的交界，建议按新flag族的非法形式报usage，而不扩大其它旧未知token的规则；最终以architect确认为准。

## 建立EvalLimits

原始超时仅存u64毫秒。前端成功后构造EvalLimits { max_steps, deadline, cancel:None }；timeout为Some时Duration::from_millis后Instant::now().checked_add，None转精确usage“选项 '--timeout-ms' 超出当前平台可表示的时间范围”。graph内部验证仍由新API负责，不重复验证，坏graph优先于节点poll。前端加载/语法/graph错误在这之前直接返回原错误，不被timeout0覆盖。

不修改runtime节点计步或attempt；CLI成功/资源错误沿既有0/2和JSON字段，ExecutionLimitError第13类无需新键。0超时有节点首poll终止、空程序成功，由runtime保证。不提供取消flag/Ctrl-C handler，不宣传阻塞IO或native调用被硬中断。

## 后续验证计划（未执行）

新tests/cli_t17.rs进程测试：2/4步边界、预算0/前导0/u64最大、任意顺序、所有执行入口、CODE恰为两flag/--json/--、argv字面flag；重复/缺值/--缺值/非法/溢出/等号拒绝；check/test/help/version拒绝及错误优先；前端错误优先于timeout0、空程序timeout0成功、有节点timeout0 Deadline、模块依赖真实错误来源、attempt不吞终止、终止前已发生副作用保留。超时测试只用确定性0及宽容进程watchdog，不断言毫秒硬实时。

checked_add平台溢出如本平台u64毫秒仍可表示，则用纯转换helper的可控基准Instant单元策略或记录该平台不可触发，不虚构u64::MAX必溢出。旧全部C及旧CLI默认无预算回归保留；新增flag红测用旧C固定产物，不拿产品编译失败当功能红测。

## 交接

runtime仍唯一evaluator实现/lib编译。本角色准备完成后停止，待root明确GO才改CLI/runner注册、添加新测试并成为唯一完整release编译者。所有Temp在项目内、只复用target/release、无依赖安装，无Git/dist修改。R304仍OPEN，设计审查不替代功能/性能验收。
