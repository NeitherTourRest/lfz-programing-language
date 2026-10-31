# LFZ 需求矩阵（唯一需求事实源 · v1.1 增量对齐版）

> 最后更新: 2026-10-31 by requirements-analyst
> 版本: **v12（T18-B HTTP四项需求通过；R-304开放，非正式发布）**
> 本文件是**唯一需求事实源**，唯一写者 requirements-analyst，他人只读。
> 来源: `task-info.md`（课程作业原始要求）+ 用户指令 + 已定案 ADR（见 `DECISIONS.md`）。

## T18-B 增量需求（2026-10-31，HTTP功能及指南通过）

依据三spec §24/T18-B-contract，冻结SHA256 `18d328d9e48780f8e6ae638f9ad24f53186d94f2207ca24da8928cbbba0d0916`、1029120B、75内置/13错误类。T18-B-independent-verification.md最终PASS：667harness、HTTP公开49、HTTPplan7、独立HTTP11及全部旧回归通过，40轮Barrier真实取消/完成竞态与closing/active断言实际执行。没有产品重编或网络由需求分析师执行。人指南21/21、AI25/25最终作者实跑锁同hash，五AI文档前后SHA一致、server_closed=true；默认AI17/25中8网络12029失败保留。R137/213/412/509四项通过；指南是作者成绩，独立功能667/49/应用7/新11，不冒称独立复跑人/AI全部示例。

新增EvalCapabilities.http为Rust公开literal机械迁移，旧非context入口默认false；process能力不授HTTP，不宣称零宿主API变化。Windows实测，非WindowsUnsupported且无exec/curl降级；TLS只审计自签拒绝12175/appbytes0，未独立重跑TLS、不承诺可信TLS成功或公网HTTPS验收，无系统信任修改。R304仍OPEN、无新性能窗/正式tag；本轮B收口，不自动化、不T19。旧A/此前停止HTTP只是各历史授权时点，当前本轮新授权范围以本节为准。

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-137 | httpRequest六参、独立http能力默认禁用，完整类型/方法/URL/header/body/范围先验后触网；成功status/headers/body三键，重复header保留、4xx/5xx普通值，body严格UTF8。直连不自动cookie/auth/redirect/decompress，无应用隐式重试 | 三spec §24.1/24.2/24.4 | 同冻结`cargo test --release`667通过；按T18-B-blackbox.md执行tests/t18b_cli_http.py→49/49；独立own.py11验证404、重复头、302只一次、cookie不跨session、POST Unicode/query、无效环境proxy仍直连、后继运行默认禁用无连接。权限分域/跨模块共享与跨session隔离；不能以单次请求见证宣称OS从无TCP重传/PAC全配置穷举或服务端副作用恰一次 | 通过 | http_control/Windows平台（tooling-dev）、evaluator/builtin注册（runtime-dev）、错误细分（core-dev）、独立报告/公开证据 | 1、2 |
| R-213 | 响应限额按完整RAW_HEADERS_CRLF API文本严格UTF16→UTF8计数加body raw bytes，非wire；有界native头/256项/8KiB scratch。局部timeout/响应超限可恢复、全局取消deadline不可吞；单owner关闭后等待HANDLE_CLOSING与active=0才释放context/buffer；双diagnostics宿主成功门禁失败闭合。CLI仅--allow-http显式授权、严格flag/类别优先与CODE/argv保护 | §24.2–24.5 | 独立667/公开49/自编11核原生头59+body2=N61成功/N60失败，绑定API文本/u16元数据SHA而非猜wire；慢头/慢体/drip总timeout与local可attempt/global穿透。真实40Barrier取消/完成竞态逐轮closing/active断言、close故障注入保系统reason、双sidecar CLI/runner拒假成功均已实际同harness执行。无detach/提前free、非Windows无fallback；OS异常closing延迟非硬返回时界保证 | 通过 | Windows async后端/CLI/runner（tooling-dev）、context/HTTP sidecar（runtime-dev）、独立667/49/11证据 | 1、2 |
| R-412 | 人/AI指南实跑HTTP工作流，说明独立权限及literal迁移、六参/三返回、API文本非wire限额、状态码/重复头、局部恢复与全局终止、双sidecar完整性门禁和平台/TLS限制 | §24；R411后继 | 人指南最终21/21锁18d328，默认12029/handler关闭reset与服务shutdown/join证据保留；AI新例按T18-B-skill-evidence报告25/25完整实跑并绑定同hash，server_closed=true且五文档前后SHA相同，默认17/25网络失败保留。不得将自签拒绝说成可信TLS成功、作者例说独立重跑或性能通过，也不宣称非Windows已支持；按最终人21/AI25作者回执收口，非独立示例复跑 | 通过 | 人指南（docs-writer）、skill/AI入口（ai-dx-engineer）及本轮实跑manifest | 4 |
| R-509 | HTTPplan预校验计划、确定性处理普通4xx及可恢复局部失败；全局deadline穿透attempt、不执行后继任务；默认无权限零连接与会话权限不泄漏，旧应用保持 | app/T18-B-HTTPPLAN.md；§24 | 按app/verify_httpplan.py接口对冻结18d328运行7/7，独立httpplan7.json7/7；同hash旧toolplan7/T17预算app40/C模块app33/jsonbatch38/jsonflow37/sortviz11回归通过。所有loopback服务器shutdown/join，不访问公网、不装包或修改信任库，不扩大本轮范围 | 通过 | app/httpplan.lfz/verify_httpplan.py/T18-B-HTTPPLAN.md（app-dev）、独立应用证据 | 5 |

所有复跑须恢复封存字节和本轮loopback夹具、使用不存在的项目Temp输出；默认权限/准备脚本失败保留，同字节正常OS许可通过不等于环境限制消失。功能通过不能替代原19项效率验收。

## T18-A 增量需求（2026-10-30，Windows受控exec通过，本轮后停止）

依据三spec §23及最终T18-A-independent-verification.md追加PASS。冻结SHA256 `4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86`、984064B、74内置/13错误类；最终同十一harness649、公开45、toolplan7、自编真实process12及旧全回归（T17公开134/app40、C29/app33、attempt13/check117/CLI91含178、旧apps37+11/jsonbatch38/JSON140）通过。人指南22/AI26作者结果已到；原失败、CONCERNS、新NUL/keys oracle错误、默认sandbox close探针C0000008、E5阶段及修复记录不抹。

本批只声明Windows实测能力；Linux/macOS及其它平台返回Unsupported，**不是已支持**。controlled Job/绝对exe与能力开关不是权限隔离沙箱，也不承诺OS异常下绝对回收时界。R304仍进行中/OPEN，无本轮性能窗，不正式发布。按用户最新范围，仅完成本轮交付后停止，不实施HTTP/T18-B/T19等后继；下方“仅规划T18”是此前历史状态，不抹各时点证据。

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-136 | exec(argv,timeoutMs,maxOutputBytes)默认能力禁用；显式EvalContext权限仅当前session且模块共享，不随函数home或attempt泄漏。实参完整验证后绝对.exe按CRT argv保真，无PATH/隐式shell；返回stdout/stderr/exitCode/signal四键，非零退出正常值，严格UTF8 | §23.1/23.3；73→74内置 | `cargo test --release`同冻结649通过；按T18-A-blackbox.md接口对最终exe与固定fixture执行tests/t18a_cli_process.py公开45/45；独立own.py12/12验证空串/Unicode/空格/引号/反斜杠、u32无符号退出、禁用与后继会话隔离。旧入口capabilities默认false，无上下文builtins调用不能绕权；NUL真实源码由FILE入口验证，不把Popen拒NUL当LFZ产品结果 | 通过 | evaluator/builtin注册（runtime-dev）、process_control/Windows平台模块（tooling-dev）、error细分（core-dev）、公开与独立证据 | 1、2 |
| R-212 | 双stream公平有界drain，raw合并=N成功/>N失败，不返回部分成功；局部timeout/output/UTF8是可恢复业务ValueError，全局cancel/deadline不可捕获且优先。Windows suspended→Job纳管→resume，成功/失败/超限均处理后代与reader；Close等清理诊断独立保存并使宿主成功门禁失败闭合 | §23.2–23.5 | 同649、公开45及独立真实process12全PASS，覆盖合并0/N/N+1和尾部超量、200KB双流、UTF8、局部timeout可attempt/全局终止穿透、后代PID经OpenProcess+Wait确认回收。实际同harness故障注入Job/Assign无用户副作用、pipe/read/wait/kill/close及资源主错误+诊断均执行；host_diagnostics非空不能仅凭result Ok宣称成功，CLI/runner无成功JSON、stderr诊断exit2。不是实际OS不可回收的绝对保障，不真实OOM/abort或任意系统清理 | 通过 | Windows后端/RAII与CLI/test_runner（tooling-dev）、context与诊断（runtime-dev）、T18-A独立649/45/12证据 | 1、2 |
| R-411 | CLI仅运行入口--allow-process显式授权，重复/等号/非执行命令拒绝，混合限额按首次新选项类别裁定文案；CODE与--后argv保真。人/AI指南准确说明资源/权限/native协作与平台限制 | §23.5；R211/410后继 | 公开45及旧T17/C回归验证flag、usage优先级、默认禁用不spawn、权限不跨execute泄漏；T18-A-guide-evidence.md22/22、T18-A-skill-evidence.md26/26锁4093714，同期独立最终审文档指纹及证据。示例不冒称宿主RustAPI新编译见证、非Windows支持、强隔离沙箱或性能通过；旧失败保全，不以本作者计数替代649/45/app7 | 通过 | CLI（tooling-dev）、人指南（docs-writer）、skill/AI入口（ai-dx-engineer）、独立审计 | 1、2、4 |
| R-508 | toolplan完整预校验计划后显式exec编排，结果确定性、可恢复错误不中断后继任务，全局终止穿透且保留已发生副作用；能力默认禁用、旧应用和≥200行sortviz交付保留 | app/T18-A-TOOLPLAN.md；§23 | 按app/verify_toolplan.py接口传--lfz/--expected-sha/--output对冻结4093714执行7/7；独立toolplan7.json7/7。两次九任务六成功/三恢复错误逐字节同、无授权零副作用、计划后项非法不执行前项、空计划成功、deadline真实exec行号且后继KEEP不写；旧模块app33/T17预算app40及其它旧app继续通过 | 通过 | app/toolplan.lfz/verify_toolplan.py/T18-A-TOOLPLAN.md（app-dev）、独立toolplan7证据 | 5 |

复跑仅恢复冻结产物/对应固定fixture，并使用不存在的项目Temp输出；如需许可控制按已有环境证据执行，不改产品绕权限。当前通过仅功能与有界生命周期验收，不关闭效率门禁。

## T17 增量需求（2026-10-30，执行限额最终功能通过）

依据三spec §22及T17-design，当前冻结SHA256 `1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45`、943104B、73内置，**新增第13类ExecutionLimitError**，下方旧12类是各历史时点，原12类触发/消息保持。`T17-independent-verification.md`最后“裁定后最终独立PASS”确认624harness、公开134、预算应用40、自编14、旧attempt13/check117/CLI91含178、旧apps37+11/jsonbatch38/JSON140/C公开29与四模块app33通过；人指南32/AI38作者实跑与11份文档SHA经独立审计，未冒称独立复跑全部作者例子。

早期公开110/121及118/122、新oracle修正与CONCERNS完整保留。空for不虚构节点、CODE双负号保持旧语义；帮助合法拼写--help/-h及--version/-V，裸help/version仍文件路径。最终纠正仅新oracle与契约歧义，没有改产品追绿或篡改旧计数。R304继续进行中/OPEN，无T17性能窗，不打正式标签；T18仅规划/提案未实现，不把外部进程/HTTP/强制终止纳入当前通过。

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-135 | 宿主可配置单次执行单调步数、协作取消及绝对Instant截止；节点poll依cancel→deadline→budget优先级。新ExecutionLimitError三固定reason/消息/真实位置，不可被attempt捕获或恢复预算；模块初始化/函数/default共享session资源且旧默认不限额 | 三spec §22.1–22.4 | 项目工具链`cargo test --release`624/624；独立own.json14验证单表达式0/1/2及let4步、空程序零poll、deadline优先预算、多层attempt穿透、普通失败不补步、菱形11成功/10失败、默认来源和后继session。已执行host单元验证首语句后mpsc握手跨线程Release取消/下一stmt Acquire观察、七HOF穿透、模板真实来源；不宣称新增外部宿主API程序验收。native内部无poll，末native后无节点可成功，空for迭代不伪造步，不是硬实时/CPU或权限沙箱 | 通过 | error.rs（core-dev）、evaluator.rs与单元（runtime-dev）、T17独立报告/证据 | 1、2 |
| R-211 | 执行CLI严格--max-steps N与--timeout-ms N；保护CODE/--后argv，拒绝非法/重复/等号/缺值/非执行命令，usage先于IO；check/test/合法帮助版本拒限额，裸help/version旧路径行为保持。资源错误exit2并沿旧单JSON和真实模块trace协议 | 三spec §22.5–22.6 | 对冻结T17字节执行`D:/app/ANACONDA/python.exe -B tests/t17_cli_limits.py --binary target/release/lfz.exe --expected-sha 1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45 --output Temp/T17/req-public-new.json`134/134；N仅ASCII十进制u64、允许0/前导0，CODE/argv原样，重复/等号精确usage无副作用；timeout前端后建立且非硬超时。独立134/40最终复验及旧全回归通过，不将旧CONCERNS或历史121/122冒充当前总数 | 通过 | cli.rs/cli_t17.rs（tooling-dev）、公开134（test-engineer）、独立public134证据 | 1、2 |
| R-410 | 人/AI指南实跑资源受限Agent工作流，区分宿主token与CLI、节点步数与native工作、协作deadline与硬实时、默认unlimited和T15不可捕获；明确同模块session共享及副作用不回滚 | §22；R409延伸 | T17-guide-evidence.md聚焦32/32、T17-skill-evidence.md38/38绑定1946ab9；独立guide-audit.json核六人指南/五AI文档11个SHA与作者manifest一致。默认权限失败保全，同字节许可复核不改canonical规则；不把作者示例审计算独立重跑，不声称新盲测/性能门禁通过或强杀线程支持 | 通过 | 人指南（docs-writer）、skill/AI入口（ai-dx-engineer）与独立文档指纹审计 | 4 |
| R-507 | JSONL模块化应用演示预算耗尽、共享预算及已完成副作用保留，系统终止不被attempt当坏数据吞掉；旧无预算应用和课程≥200行sortviz保持 | app/T17-BUDGETS.md；§22 | 按T17-BUDGETS.md和verify_jsonbatch_budgets.py接口对冻结1946ab9执行40/40；独立app40-final.json40/40，回调预算28在真实record模块终止，已完成SIDE不回滚，不能伪造正常混批成功。旧C四模块33、jsonbatch38/jsonflow37/sortviz11仍通过，首次独立相对路径oracle失败保全不当作产品缺陷 | 通过 | app/verify_jsonbatch_budgets.py、T17-BUDGETS.md（app-dev）、独立app40证据 | 5 |

复跑需恢复冻结T17产物与工具链、使用尚不存在的项目内Temp输出；后继T18新源码不能冒充本节测量/功能时点。限额功能通过不是效率门禁或资源隔离证明。

## T16-C/D 增量需求（2026-10-30，模块执行及诊断通过）

本节按三spec §21及T16-C-design收口，冻结产物SHA256 `cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006`、902656B、73内置/12错误类。`T16-C-independent-verification.md`与证据确认同9个release harness599、公开29、四模块app33、自编15、旧attempt13/check117/CLI91含178、旧apps37+11/jsonbatch38/JSON140全部PASS；人指南11/AI18最终实跑通过，默认权限失败记录保留，同冻结字节许可复核未绕过canonical身份。

**当前CLI run已执行经完整图验证的模块；下方B的run unsupported/初始化前拒绝仅是8ea375过渡版本历史，不是当前CLI限制。** 旧无图eval_module入口仍按§20 guard拒模块Program，新图执行入口必须经合法图；check仍不初始化。以下通过不代表所有未来模块能力、效率或§22执行预算通过。T17 §22仅规范冻结/后继实现准备，尚未验收；计划第13类ExecutionLimitError不能记作本C产物已支持，当前12类仍有效。R304继续OPEN，无C性能窗，不正式tag/发布，旧v2及失败证据不变。

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-134 | 单session按图后序独立globals初始化，共享canonical模块只一次、不同session不复用缓存；namespace标量快照/引用共享，UserFn home globals及模板定义默认值上下文正确，argv统一，attempt完整恢复来源及caller而不回滚副作用 | 三spec §21.1–21.5 | 项目工具链`cargo test --release`599全过；对冻结C字节运行`D:/app/ANACONDA/python.exe -B tests/t16c_cli_run.py --binary target/release/lfz.exe --expected-sha cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006 --output Temp/T16-C/req-public-new.json`29/29；独立own.json15验证菱形、两次session、同名caller不串、逃逸闭包、namespace字段不回写私有标量、默认值定义处/显式覆盖caller及恢复后的真实故障来源。仅本地已验证图，无远程/动态import/循环live binding | 通过 | evaluator.rs/value.rs（runtime-dev）、§21（language-architect）、公开与独立模块见证（test-engineer/verifier） | 1、2 |
| R-210 | CLI file/-e/stdin run启用图执行，check仍仅图检查；跨文件human/JSON逐帧真实来源及原下标折叠，宿主坏图/Origins/HostFailure受控exit2且runner失败闭合，不伪装LFZ成功或attempt结果 | 三spec §21.6–21.7；R209后继 | 上述599与29/15、旧13/117/91含178/apps/JSON回归全PASS；独立45层递归JSON47帧全保留、human40个File记录且最深真实lib；默认值失败定位定义文件、caller显式覆盖错误定位入口。实际599内host_failure_is_not_a_passing_case_even_without_lfz_error_class覆盖HostFailure及Located wrapper；源码路径核对runner为Verdict::Error，旧origins=None输出保持。B的旧CLI模块拒绝断言经批准迁移，旧eval guard仍验证 | 通过 | CLI/test_runner（tooling-dev）、origins运行时与独立报告/证据 | 1、2 |
| R-409 | 人/AI指南同步真实本地模块工作流，明确check与run、namespace快照、home函数、模板默认/覆盖、跨文件CLI诊断与attempt不含file、宿主API迁移及非沙箱边界 | §21；R408延伸 | T16-C-guide-evidence.md命令聚焦11/11、T16-C-skill-evidence.md命令18/18均绑定cdedc9；默认权限失败与许可同产物复核保留。指南不得将Result三/五/三字段扩成含file或声称RustAPI零迁移；UserFn.home_module/StructDef.default_context及旧helper None引用规范，不把示例实跑冒称新盲测或RustAPI独立编译验收 | 通过 | 人指南（docs-writer）、skill四文件/AI入口（ai-dx-engineer）及实跑证据 | 4 |
| R-506 | 四文件模块化JSONL应用实际复用模块导出，混合坏行继续处理，保留文件/参数错误边界；不以小应用替代既有≥200行sortviz交付 | app/T16-JSONBATCH-MODULES.md；§21 | `D:/app/ANACONDA/python.exe -B app/verify_jsonbatch_modules.py --lfz target/release/lfz.exe --expected-sha cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006 --output Temp/T16-C/req-app-new.json`33/33；独立app33.json同hash33/33，Python oracle核对四模块正常/坏行结果，既有jsonbatch38/jsonflow37/sortviz11仍通过 | 通过 | app/jsonbatch_modules/及verify_jsonbatch_modules.py（app-dev）、独立app33证据 | 5 |

复跑须恢复该C冻结源码/字节和对应工具链、使用不存在的项目Temp输出路径，并按环境权限控制执行；后继T17工作树/新错误类不能冒充本节599测试时点。

## T16-B 增量需求（2026-10-30，仅静态模块图通过）

范围对应三spec §20–20.10及T16-B-design。冻结B产物SHA256 `8ea375dcd08098ef61d040a57a7b1ad28a28c2fdc25a5ca4921a75b728944480`、859136B。`docs/reports/T16-B-independent-verification.md`确认同产物584 release harness、公开38、自编18、旧attempt13/check117/CLI91含官方178、app37+11/jsonbatch38/JSON140全部PASS。本节三项通过只指静态声明、完整依赖图check及执行前拒绝，不提供模块初始化/执行或跨模块home globals运行。C的§21契约虽已冻结且实现中，尚未独立验收，不能标整个T16通过。

默认sandbox canonicalize的AccessDenied及作者默认失败结果保留；独立584和图过程使用同一冻结产物在许可权限控制下执行，未改产品绕过canonical身份规则，不能声称默认环境限制消失。R304仍进行中/OPEN，无B性能计时或正式标签；旧无模块行为及历史v2成绩不重写。后继C源码/二进制不能冒充本B冻结对象。

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-132 | 顶层static import alias from显式相对.lfz字面路径、contextual export列表及有限导出/别名链接检查；import在其他顶层声明前，路径/重复/冲突/缺导出按规范定位。旧export标识符表达式、普通重声明规则保持，不承诺完整静态类型/名称分析 | 三spec §20.1/20.9 | 项目工具链`cargo test --release`全584通过，parser/module新增及旧测试保持；运行`D:/app/ANACONDA/python.exe -B tests/t16b_cli_check.py --binary target/release/lfz.exe --expected-sha 8ea375dcd08098ef61d040a57a7b1ad28a28c2fdc25a5ca4921a75b728944480 --output Temp/T16-B/req-public-new.json`应38/38。限定使用封存同hash和不存在输出路径；nested import/export、动态/绝对/非法路径、alias冲突及非法导出拒绝，旧export变量仍可运行 | 通过 | AST/parser/error/loader（core-dev）、tests/cli_t16b.rs与公开矩阵、独立报告 | 1、2 |
| R-133 | check对模块root构建完整依赖图但不初始化；canonical身份去重、稳定迭代DFS、受控循环/节点/深度/原字节限额，依赖错误归真实文件。load/parse/析构在大栈，旧无模块root不受新增图限额 | 三spec §20.2–20.10 | R132公开38及独立graph-witnesses.json18全部通过；依赖含writeFile/input/while true时check返回且哨兵字节不变/新文件不创建。合法菱形/重复身份、缺文件/UTF8/#42/语法/cycle真实依赖路径定位、2MiB恰好/多1、深度128/129及无模块>2MiB旧例通过；1024节点/16MiB总量与§20.10字节优先在独立执行core harness覆盖。最多读取min(单限额,总剩余)+1，只对已读字节判单文件优先，不补读猜文件尾；图上限非RSS沙箱或性能证明 | 通过 | src/modules.rs/loader.rs（core-dev）、CLI图检查/诊断（tooling-dev）、图unit与独立进程证据 | 1、2 |
| R-209 | B阶段run及旧eval遇整个root任何模块声明，在任何LFZ副作用前明确拒绝暂不执行；保持A旧shape、T15固定Result、73内置和无模块CLI/app行为 | 三spec §20.6/20.8；R208延伸 | `cargo test --release`584/584；独立run前写语句后export仍受控SyntaxError且哨兵不变；旧attempt13/check117/CLI91含178、jsonflow37/sortviz11/jsonbatch38/JSON140全PASS。旧无模块>2MiB源码check/run仍成功，check只图检查不执行；模块运行/缓存初始化次数/跨模块恢复身份需由后继C独立验收，不能由B菱形check推断已实现模块执行 | 通过 | evaluator执行预检（runtime-dev）、CLI/check（tooling-dev）、T16-B-independent-verification.md及证据目录 | 1、2 |

## T16-A 增量需求（2026-10-29，仅来源基础通过）

范围严格对应三spec §19、T16-A-design.md。当前候选SHA256 `65a29eaea54f32d1d2b404f22aed09389c3ffa71e6a81acb8fa6351d32fa42b7`、783872B、73内置；team-lead已确认dist/target同字节，独立报告`docs/reports/T16-A-independent-verification.md`实测对象为target，不冒称verifier复跑了dist复制。独立562 harness、attempt13/check117、CLI91（含178官方）、旧app37+11/新38/JSON140及新Rust API见证通过；16程序×human/JSON两模式共32组/64进程的stdout/stderr/exit逐字节一致。

本节只通过模块前置宿主来源元数据及旧行为兼容，**不是import/export加载、模块图、模块执行、home globals切换或跨文件CLI诊断通过**。T16-B仅设计草案，不标实现通过。新增公开UserFn.home_module属于Rust构造API迁移，外部手写literal须补字段，不能宣传完全无Rust API变化；旧Span/LzError/TraceFrame/Loaded/Program形状、CLI和T15固定Result字段保持。R304仍进行中/OPEN，无T16-A性能窗，不正式发布；旧T15/T14及v2证据与成绩各自保持。

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-131 | 来源基础API提供分域SourceId/ModuleId、稳定SourceMap追加编号、LocatedFailure保留旧错误、函数整数home_module=ENTRY及TracedRun来源sidecar。来源attach完整校验后原子替换，拒绝坏长度/ID/错误来源条件且保留旧sidecar；越界/legacy查询安全None。不加载或执行模块 | 三spec §19.1–19.3 | 在项目Temp工具链环境执行`cargo test --release`全部562通过；按T16-A-independent-verification.md的rustc命令链接并运行`Temp/T16-A-verifier/witness.exe`→exit0，覆盖SourceMap空/稳定ID/未知ID/loaded=None、校验规范顺序及原子拒绝、不同帧不同来源、host后改frames/result仍安全、home_module ENTRY；容量编号上限以有界构造验证，禁止真实巨表/OOM/abort。API记录UserFn新字段迁移，无旧语言字段或错误类扩展 | 通过 | src/source.rs/lib.rs（core-dev）、value.rs/evaluator.rs（runtime-dev）、三spec与T16-A独立witness/报告（language-architect/verifier） | 1、2 |
| R-208 | T16-A旧无模块语言与CLI兼容、T15恢复协议不变；新增基础元数据不能被当作真实多模块求值或跨文件诊断 | IC §19.4；R206/207延伸 | `cargo test --release`562/562，独立直接运行7harness亦562；沿原公开脚本attempt13/check117/CLI91含官方178、jsonflow37/sortviz11/jsonbatch38/JSON140全PASS；`D:/app/ANACONDA/python.exe -B docs/reports/T16-A-independent-evidence/differential.py`在保全独立报告要求的旧T15副本后复现16程序32组完整双流/退出码相同（重新运行前另行保全既有证据，不覆盖唯一历史记录）。旧API默认origins=None、T15三/五/三字段不读sidecar、Span/LzError/TraceFrame/Loaded/Program旧shape及旧错误JSON保持；差分属于有界见证非穷举证明，无A性能提升声明 | 通过 | 交付物2/3；作者core/runtime新增单测，T16-A-independent-evidence与报告（verifier） | 1、2 |

## T15 增量需求（2026-10-29，独立功能通过）

依据三spec §18及T15-design。当前作者候选73内置/12错误类，SHA256 `1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c`、782848B。作者549release/178官方/13公开CLI/jsonbatch38、人指南20通过；AI新增25/25及四skill/入口同步回执已到，最终独立报告`docs/reports/T15-independent-verification.md`及证据目录确认同hash的549 release、178官方（含于91旧CLI）、attempt13、check117、旧app37+11、新jsonbatch38、旧JSON/argv140及38自编见证全PASS；完整trace/depth恢复、逃逸闭包与资源错误传播均有独立证据，资源边界采用安全单元构造而非真实abort/OOM。以下四项通过。需求分析师仅核对文档与回执，不自行运行产品验收。历史T14及T12通过保持各自时点；R304仍进行中/OPEN，无T15性能测窗，非正式发布，不能借attempt功能恢复宣称效率改善。

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-130 | attempt(f)严格1参零参可调用函数，仅未遮蔽直接名调用及管道；返回字段固定ok/value/error的普通Result。错误记录class/message/line/col/traceback，保持原消息位置与完整外→内帧，不添加attempt帧 | 三spec §18 | `dist/lfz.exe -e 'assert(attempt(() => 1/0).error.class == "ZeroDivisionError")'`应exit0；正式LFZ及公开脚本核成功nil/共享容器/函数、argv、遮蔽、管道、记录字段/帧顺序。外层arity/非函数/回调arity及求参错误不属于本层捕获 | 通过 | spec、runtime、tests；language-architect/runtime-dev/test-engineer | 1、2 |
| R-207 | 恢复白名单与状态符合§18：可恢复错误恢复调用前完整trace/depth，不回滚副作用或闭包ID；语法/前导/递归/容量及宿主故障不伪装业务Result；旧回归保留 | IC/semantics §18 | `cargo test --release`全部通过；`dist/lfz.exe test --json`178/178；`D:/app/ANACONDA/python.exe -B tests/t15_cli_check.py --binary dist/lfz.exe --expected-sha256 1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c --output Temp/T15/req-cli-new.json`全部通过；独立验证nested恢复、连续失败后成功/递归、失败后未捕获错误无旧帧、逃逸闭包名称、输出/IO/cell不回滚。资源类使用单元构造，不做真实abort/OOM压测 | 通过 | runtime单测、黑盒与独立报告；runtime-dev/test-engineer/verifier | 1、2 |
| R-408 | 人/AI指南实跑attempt批处理工作流，说明延迟回调、外层校验、Result及错误边界、非事务性、非一等内置与check仅解析；旧盲测成绩不改 | 三spec §18；R407延伸 | 人指南`D:/app/ANACONDA/python.exe -B docs/reports/T15-guide-evidence/verify.py --binary target/release/lfz.exe`20/20；AI按T15报告命令新增/受影响25/25通过并锁同hash；独立CLI语义与指南约定一致，不把作者示例冒称新盲测或性能通过 | 通过 | 五指南、skill四文件/AI入口与实跑证据；docs-writer/ai-dx-engineer | 4 |
| R-505 | jsonbatch按行捕获JSON错误继续下一行，输出确定性结果及逐行错误；读取/配置等边界错误不被静默吞掉，错误恢复不提供文件事务 | app/T15-JSONBATCH.md；§18 | `D:/app/ANACONDA/python.exe -B app/verify_jsonbatch.py --lfz dist/lfz.exe --expected-sha 1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c --output Temp/T15/req-app-new.json`38/38；正常/坏JSON混批仍输出后续有效记录、行号及错误，坏参数/输入IO受控exit2，文件保护与现有≥200行sortviz交付保持 | 通过 | jsonbatch.lfz/verify_jsonbatch.py/T15-JSONBATCH.md；app-dev | 5 |

## T14 增量需求（2026-10-29，check独立功能通过）

来源为用户连续自动推进授权、team-lead定向任务及三spec §17。当前作者候选为1.4.0、72内置、12错误类，target/dist SHA256 `83f56a194e216862c45377f4c3971b085744a7ba2363163e1edac6af5e85ac79`、774656B；team-lead回执作者release542、公开check117/117、人指南27/27通过。verifier最终报告`docs/reports/T14-independent-verification.md`及对应证据目录已确认同hash的542 release、117 check、91旧CLI（含166官方）、app37+11、140旧独立及57新增非执行见证全部通过，源码调用路径无求值且大栈内析构正确。因此R-128/R-129/R-206标为**通过**；R-407按team-lead最终AI回执与实存报告关闭：人指南27/27、AI新增及最小基线27/27，四skill和AI入口同步，均绑定同hash；这些是作者示例实跑与已独立验收CLI协议的对齐证据，不冒称新Agent盲测或verifier重新跑全部作者示例。需求分析师只核证据映射，未自行运行产品验收。

下方T12各项通过和精确hash是当轮历史验收，不被T14覆盖。T13候选9cda功能530通过，但唯一完整窗口及独立审计为15 PASS/2环境不确定/2 FAIL；两项生产优化已拒收，恢复T12对应生产前缀，仅保留8新增回归测试，候选源码与exe及全部失败数据保全。证据 `T13-independent-verification.md`、`T13-performance.md`、`T13-runtime-evidence/retraction-source-check.json`。**R-304保持进行中/OPEN，原19门槛不变；当前未有T14性能测窗，不用T13或T12结果替代T14效率验收，也不正式发布。**

以下命令在项目根执行；Python为 `D:/app/ANACONDA/python.exe -B`，所有新输出须使用不存在的项目内Temp路径。

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-128 | `lfz check`接受FILE、`-e CODE`、stdin `-`单一源码，只加载/lex/parse及既有AST深度检查；不得求值或执行程序IO/循环。文件沿run加载规则，CODE原样保留，CODE外任何`--`及check --help拒绝；成功不承诺名称/类型/arity/IO可用性 | 用户连续推进授权；syntax/semantics/IC §17 | `dist/lfz.exe check -e 'print(1 / 0)'` →exit0且双流空；`dist/lfz.exe check -e 'while true {}'`不执行循环且正常返回；`dist/lfz.exe check -e 'return 1'` →exit2；`D:/app/ANACONDA/python.exe -B tests/t14_cli_check.py --binary dist/lfz.exe --output Temp/T14/req-check-fresh.json` →全部通过，并核三来源哨兵已有文件字节不变/新文件不创建、后续前端错误也不触发副作用，CODE/usage/加载位置/原执行入口兼容。不宣称零宿主IO或安全沙箱 | 通过 | 交付物1/2/3；三spec（language-architect）、src/cli.rs/main.rs（tooling-dev）、tests/t14_cli_check.py（test-engineer） | 1、2 |
| R-129 | 合法check的JSON显式区分检查与执行；正常成功双流空exit0，前端失败exit2。机器成功恰一行三个键ok/mode/executed，失败加既有错误字段/精确位置，旧run/test JSON不变；CLI usage错误即使带--json仍stderr文本/stdout空 | IC §17.2；R-112/125延伸 | `dist/lfz.exe check --json -e 'print(1 / 0)'` →stdout恰`{"ok":true,"mode":"check","executed":false}`一行、stderr空、exit0；`dist/lfz.exe check --json -e 'return 1'` →exit2，唯一对象按ok/mode/executed/error/message/file/line/col/traceback顺序，executed=false；`dist/lfz.exe check --json --help` →exit2、stdout空、stderr为usage文本；R-128公开脚本核字段/顺序/转义和旧run JSON | 通过 | 交付物1/2/3；三spec、src/cli.rs、tests/cli_t14.rs、公开CLI脚本 | 1、2 |
| R-206 | T14新增CLI集成及公开进程验收覆盖正常/边界/错误与非执行见证，旧用例全部保留；深AST加载解析及析构保持既有受控边界，不把CLI过程检查数冒充LFZ runner用例数 | IC §17.3；R-201/205延伸 | 项目Temp工具链环境下`cargo test --release` →全部通过且0 failed/ignored，包含保留8项T13回归及12项cli_t14；运行R-128公开脚本→全部通过；`dist/lfz.exe test --json`仍166/166。原12/91 CLI及独立非执行/前端深度见证应绑定同一最终hash，独立报告提供可复现原始双流/退出码与文件哨兵，不能将166与包含该runner的91重复相加 | 通过 | 交付物2/3；tests/cli_t14.rs（tooling-dev）、公开脚本与覆盖报告（test-engineer）、独立报告（verifier） | 1、2 |
| R-407 | 人/AI指南与§17一致并实跑check→run工作流；说明三来源、JSON检查标记、usage例外、CODE保护与前导规则、非执行与非静态承诺，保持LFZ check软断言及历史v2成绩独立 | R-402/406延伸；三spec §17 | `D:/app/ANACONDA/python.exe -B docs/reports/T14-guide-evidence/verify.py --binary dist/lfz.exe --expected-sha256 83f56a194e216862c45377f4c3971b085744a7ba2363163e1edac6af5e85ac79 --output Temp/T14/req-guide-fresh.json` →27/27；AI新示例按T14-skill-evidence.md命令27/27通过并绑定最终hash，旧v2 manifest/原8/8与131/131不改；指南/skill按§17协议同步，作者实跑与T14独立CLI协议结果对齐，不把语法通过说成运行成功或性能通过 | 通过 | 交付物1/5；五指南（docs-writer）、skill四文件及docs/guide/ai入口（ai-dx-engineer）、T14示例实跑证据 | 4 |

## T12 增量需求与作者验收快照（2026-10-25）

来源：用户授权自主方向「有特色、实用、适合 agent 使用以及高效」及 team-lead T12 任务书。语言契约以 `docs/spec/{syntax,semantics,interface-contract}.md` §16 为准。当前项目内主产物为 **1.4.0候选 checkpoint / 72内置 / 12错误类 / 零文法改动**，759808B、SHA256 `47ed2bfe2a2694b11bd611172cfcf0000d0bfc58e3b9bd809ca95f066dd0261f`。构建证据 `docs/reports/T12-release-evidence/final-opt2/build-final.json`；release profile为opt-level=2、lto=true、codegen-units=1、strip=true，取舍证据T12-runtime-startup-candidates.md。该产物不是正式发布，不更新项目外全局安装。历史首版FB8E/821248B作者证据保留，不冒充最终候选。

本节八项功能需求已按verifier最终回执标为**通过**：绑定最终47ED候选的独立release 522（404+48+28+12+15+15）、166官方/91进程、140自编witness、app37+11均PASS；§16.7旧149b对新47ED的16程序/8类作用域差分，stdout/stderr/exit逐字节相同。证据`docs/reports/T12-independent-verification.md`及`T12-independent-final/`。需求分析师仅核对文档与证据映射，不自行执行产品验收。性能 R-304 保持**进行中/OPEN、未通过**，整体仅候选checkpoint，不能因功能通过合并称为整轮通过或正式发布。

复现均在项目根执行，Rust 的 RUSTUP_HOME/CARGO_HOME/TEMP/TMP 按项目 `Temp/toolchain` 环境设置；Python使用 `D:/app/ANACONDA/python.exe -B`。证据运行器拒绝覆盖已有结果，复跑须指定项目内新的输出路径。

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-123 | `argv()` 严格零参、只含 `--` 后原始字符串；独立求值快照，每次返回全新数组。仅 argv 新增一等内置值，支持别名/闭包/容器/高阶调用、身份与词法遮蔽；旧69内置和两JSON内置不因本条获得独立取值能力 | 用户 2026-10-25；IC §16.1/16.6 | `dist/lfz.exe -e 'print(jsonStringify(argv()))' -- --json -e '中文 空格'` → exit0，stdout `["--json","-e","中文 空格"]`；`tests/t12_cli_check.py` 验证空串/旗标保真、别名/遮蔽/新数组；`cargo test` 相关上下文测试验证旧API默认空、不同求值不串扰、用户同名函数仍执行自身函数体。作者证据 T12-blackbox.md、T12-runtime-evidence.md；最终47ED独立功能PASS | 通过 | 交付物1/2/3；spec（language-architect），src/evaluator.rs、builtins.rs、value.rs（runtime-dev），CLI（tooling-dev），tests（test-engineer） | 1、2 |
| R-124 | 接受 `[-e CODE]`/`run -e CODE`、`-`/`run -` stdin源码入口；-e紧随项无条件源码，`--`只能在入口完整后分隔；无分隔符额外参数仍错误。伪路径分别 `<command>`/`<stdin>`、首行行号1，不要求也不消费文件前导 | 用户 2026-10-25；syntax §16 | `dist/lfz.exe -e 'print(42)'` 与 `'print(42)' \| dist/lfz.exe run -` → stdout42/exit0；空CODE和空stdin为成功空程序，缺CODE或无分隔符extra为exit2；stdin非法UTF8为无span的`IOError: 无法读取标准输入`，stdin源码读至EOF后input仍按既有IOError。文件缺#42仍CosmosAnswerError；91进程作者记录覆盖BOM/CRLF/源码旗标token/行号/EOF，最终47ED独立功能PASS | 通过 | 交付物1/2/3；docs/spec（language-architect），src/cli.rs（tooling-dev），tests/t12_cli_check.py（test-engineer） | 1、2 |
| R-125 | 宿主 --json保留前置/后置幂等形式，但不剥除CODE或脚本参数；stdout恰一行状态/错误对象，print/;;/input提示走stderr。不返回最后表达式值；正常模式print(jsonStringify(...))输出业务JSON | 用户 2026-10-25；syntax §16；R-112延伸 | `dist/lfz.exe --json -e 'print(42)'` → exit0/stdout仅`{"ok":true}`/stderr42；`dist/lfz.exe --json -e '1 / 0'` → exit2/单JSON且error=ZeroDivisionError；文件/-e/stdin及源码/参数字面--json均经91进程检查，证据T12-blackbox-evidence/release-cli.json；不将业务JSON与状态JSON合并。最终47ED独立功能PASS | 通过 | 交付物1/2/3；src/cli.rs（tooling-dev），tests（test-engineer） | 1、2 |
| R-126 | `jsonParse(text)` 严格1参string，标准JSON→nil/bool/string/array/匿名struct；无小数/指数整数精确i64且超界拒绝，含小数/指数按有限f64舍入/下溢，浮点负零保留；拒BOM/重复解码键/非标准语法，支持Unicode与UTF16代理对 | 用户 2026-10-25；IC §16.2/16.3 | `dist/lfz.exe -e 'assert(jsonParse("9007199254740993") == 9007199254740993)'` → exit0；运行 `dist/lfz.exe test --json` 中test_t12_json_values/unicode及负例全PASS；深度计同时打开array/object，128成功/129拒绝。ValueError精确模板`JSON 解析失败（字节偏移 {offset}）：{reason}`，五reason为不是合法的 JSON/对象键重复/整数超出 i64 范围/浮点数超出有限范围/嵌套深度超过 128 层；字节offset、参数/类型错误由91进程精确核对。作者证据T12-blackbox.md；最终47ED独立功能PASS | 通过 | 交付物1/2/3；spec（language-architect），src/json_codec.rs、builtins.rs、error.rs（runtime-dev），tests（test-engineer） | 1、2 |
| R-127 | `jsonStringify(value)` 严格1参，紧凑无尾换行、UTF8键字节序；精确整数、有限float最短往返并保留.0/负零、合法控制字符转义且不转义美元。struct按A5过滤函数方法字段，根/数组函数及模板拒绝；路径环拒绝，共享无环允许且不改输入 | 用户 2026-10-25；IC §16.3 | `dist/lfz.exe -e 'print(jsonStringify(jsonParse("{\"z\":1,\"a\":2}")))'` → stdout`{"a":2,"z":1}`/exit0；正式references/Unicode正负例与91进程使用Python独立oracle核对全32控制字符、共享/环、128/129边界、float类型。ValueError模板`JSON 编码失败：{reason}`，四reason为不支持函数或结构体定义/浮点数必须有限/循环引用/嵌套深度超过 128 层；按数组顺序/已排序数据字段深度优先先错。作者证据T12-blackbox.md；最终47ED独立功能PASS | 通过 | 交付物1/2/3；spec（language-architect），src/json_codec.rs、builtins.rs（runtime-dev），tests（test-engineer） | 1、2 |
| R-205 | T12正常/边界/错误黑盒与独立验收齐备，既有测试不删除不篡改；argv一等值范围及旧名称兼容、struct索引语义均纳入回归 | 用户 2026-10-25；R-201延伸 | `cargo test --release` →522 passed/0 failed/ignored（原494保留，新增while作用域等价性回归）；作者最终记录T12-release-evidence/final-opt2/cargo-test-release.txt；`dist/lfz.exe test --json` →166/166 PASS、原130保留、新4组64assert+32负例；`D:/app/ANACONDA/python.exe -B tests/t12_cli_check.py --lfz dist/lfz.exe --output Temp/T12/requirements-recheck.json --full-suite` →91/91（含一次166全量，不能相加虚增）；首版作者证据T12-cargo-full.txt与T12-blackbox.md保留；最终47ED独立522/166/91/140及新增scope差分/app复验以T12-independent-final/为证据目录，最终结论PASS | 通过 | 交付物2/3；tests（test-engineer），独立报告（verifier） | 1、2 |
| R-304 | T12旧新版同条件性能：19旧负载门禁、大对象收益目标、JSON规模与新入口开销；原始失败保留，不因结果放宽测前阈值 | 用户 2026-10-25；T12-performance.md测前冻结 | 按T12-performance.md脚本命令复跑，2预热+7正式、15s超时，记录命令/hash/原始值。旧单元合格线max(原中位×1.20,原+5ms)，noop为max(原×1.30,原+5ms)；8000键目标≤原×0.70。JSON2000/4000/8000键每进程20次，net≤5ms或启动占比≥50%不报吞吐，足够可测时翻倍net≤3.0，否则排查。首版3单元超线及复查保留；同SHA不同执行路径出现显著差异，见T12-runtime-performance-diagnostic.md，未定因且未声称源码修复。最终47ED使用同一Temp/T12/perf-subject.exe切换旧新bytes、逐次hash核对、每copy固定等5秒、2预热+7正式，原19线不改；opt2-assessment.json判15/19原绝对线通过、3环境不确定、string_map8k新版22.6906ms高于原22.5214ms且同期旧版21.8713ms在线，整体不通过。同期struct8k93.4315→21.1978ms、numeric500k104.9399→90.8044ms是局部收益；string_ops4k虽过原线，同期新版97.7181ms比旧75.879ms慢28.8%，明确保留。原始证据benchmarks/t12/results/opt2-same-path.json、opt2-new-features.json、opt2-assessment.json。全部观察输出正确不等于效率通过，无峰值RSS声称 | 进行中 | 交付物4；benchmarks/t12/、docs/reports/T12-performance.md（perf-engineer），诊断报告（runtime-dev） | 3 |
| R-406 | 人/AI指南与§16一致，实际运行新增例子；明确argv唯一新增一等内置、词法遮蔽、JSON数值/错误/键序/复杂度、stdin与input限制、状态与业务JSON；历史v2证据保持原字节与成绩 | 用户 2026-10-25；R-402~405延伸 | `D:/app/ANACONDA/python.exe -B docs/reports/T12-skill-evidence/run_examples.py` →30 passed；`D:/app/ANACONDA/python.exe -B docs/reports/T12-guide-evidence/verify.py` →SUMMARY35/35、10/10，其中35代码块28成功7教学负例，另91runner复跑不算新增；两报告T12-skill-evidence.md/T12-guide-evidence.md为FB8E解释器作者证据；最终候选指南模式由verifier独立抽查及最终进程复验补证，不虚称作者全部示例已在47ED重跑。旧v2 raw ZIP SHA仍90a6a3e3d894e7a813b06463506d5cc540e0a4570d7b005a3fd27ccc6a776d84，当前active skill/binary变化不得重写旧manifest；无新Agent盲测声明。作者完成/最终47ED独立功能PASS | 通过 | 交付物1/5；docs/guide五指南（docs-writer），skill四文件及docs/guide/ai入口（ai-dx-engineer） | 4 |
| R-504 | jsonflow通过CONFIG INPUT OUTPUT三路径参数读取JSON配置与记录，filter/map筛选投影、确定性写JSON；sortviz扩参数且默认行为保留，课程≥200行应用不以小示例替代 | 用户 2026-10-25；app/README.md | `D:/app/ANACONDA/python.exe -B app/verify_jsonflow.py --lfz dist/lfz.exe --output Temp/T12/requirements-jsonflow.json` →37/37；`D:/app/ANACONDA/python.exe -B app/verify_sortviz.py --lfz dist/lfz.exe --before-exe Temp/t12-perf/lfz-before.exe --output Temp/T12/requirements-sortviz.json` →11/11。jsonflow73行、坏配置/记录/参数退出2且验证失败保留输出/输入；只识别字面相同路径、非原子IO且整批驻内存限制明确。sortviz362行/311有效行、150自测/默认stdout旧版字节一致；作者首版证据T12-app.md；最终47ED独立jsonflow/sortviz证据位于T12-independent-final/，终验结论PASS | 通过 | 交付物6；app/jsonflow.lfz、sortviz.lfz、README/DEV_RECORD与两验证器（app-dev） | 5 |

**内部优化验收边界（IC §16.7）**：仅while体无直接Decl/FnDecl/StructDecl时允许省略无绑定物理Env，扫描一次；有直接声明仍每轮新child，其它block/for/函数环境不变。逻辑作用域、捕获cell、自身self、dump顺序/slot、Flow传播/错误帧均应等价；最终新增16程序/8类作用域差分与522全量独立复验均PASS，性能门槛仍未通过，不以测得加速替代语义证明。profile与while优化共同属于最终产物条件，不将全部收益单独归因某一个改动。

**当前事实与历史边界**：R-113的69内置与R-401旧spec字节数仅为T11快照，当前72内置及§16补钉以本节为准。R-405既有UTF8前缀扫描、循环push/拼接O(n²)等结论保留；当前新增StructObj索引仅平均查找改善，不是所有操作最坏O(1)，JSON parse为O(n+Σk log k)、stringify包含展开输出与对象键排序成本，键比较成本另计。指南作者均据此同步，性能仍以R-304未完成状态为准。

T11正式v2首轮报告`docs/reports/agent-benchmark-v2/v2-20261024-r01/SUMMARY.md`保留8/8整题、131/131功能与协议审查PASS；下方早期快照的「未重跑」仅是当时历史。新版本不重算旧成绩、不将旧2/8与v2直接比较。
## 当前增量快照（2026-10-24，T11-16）

- 用户已单独追认本轮文件 IO：`readFile` / `writeFile` / `appendFile`。规范与实现现为 **69 个内置**（原 54 + 新增 15），另扩展 `range` 双参，并固化 string 索引拒绝的既有行为；没有新增文法或错误类，仍为 12 类。
- 证据来源为 verifier 的 `docs/reports/T11-v11-reverification.md` 与同名 `-evidence/`：**494 Rust 测试 / 130 工作区黑盒 / 74 独立 witness PASS**（494 全量由 tooling-dev 执行，日志 `Temp/tooling-v01-cargo-test.txt`；verifier 修复前独立单测为 491，未冒记其重跑）。需求分析师只维护需求与证据映射，不将文档检查视作另一次测试执行。
- **本批实现及测试环境可交付，V01 已关闭**：初验干净 CWD 为 129 PASS / 1 ERROR，原失败证据保留；tooling 修复后全量 **494/494 PASS**，verifier 13:41 使用原目录/原 tests 副本确认无 Temp 后重跑，**130/130 PASS、exit 0**；Temp 为普通文件的冲突场景 **exit 2、stdout 空、sentinel 内容保留**。证据 `docs/reports/T11-v11-reverification-evidence/clean-cwd-fixed.json`、`temp-file-conflict.json`。R-202 通过；该结论不包含下述未重跑的盲测。
- **8 同题盲测未重跑**：原始 8 题任务书/验收记录不齐；历史 **2/8** 原值保留，74 witness 不能替代盲测结果。R-403 的新一轮改善不作通过声明。
- 本节及新增/修订条目为当前增量口径。下方 v2 的历史计数、P1 阶段覆盖汇总及验收命令示例保留作来源记录，不能据此推断本轮完成状态；本次未重新验收性能、应用或答辩交付。
## 0. 使用说明（team-lead / verifier 先读）

- **文档定位**：P9 验收时 verifier 以本文件 §1/§6 为对表依据；每条需求都能落到「**一条可原样执行的命令 + 期望结果**」。
- **状态取值（§1 主矩阵）**：`待办 | 进行中 | 待验收 | 通过 | 失败 | 已撤销`（严格枚举）。
- **覆盖取值（§4/§5 追溯）**：`✅ 已满足 | 🟡 部分 | ⬜ 未做`。
- **验收标准纪律**：可观察、可命令复现、二值可判；**禁止**主观词（优秀/完善/健壮/友好/高效/优雅/基本可用）；性能类用具体数值阈值。
- **命令约定**：所有命令在**项目根**执行。`cargo run -q -- <args>` ≡ 构建后运行 CLI；已 `cargo build` 可直接用 `lfz <args>`。PowerShell 下退出码取 `$LASTEXITCODE`。
- **当前基线（2026-09-27 实测）**：HEAD `7527792`；`cargo build` **0 warning**；`cargo test` **431 passed / 0 failed / 0 ignored**（lib 361 + bin 42 + tests/cli.rs 16 + tests/test_runner.rs 12）；标签 `v0.1.0`、`v0.2.0`。

---

## 1. 需求矩阵（v1 基线 + v1.1 增量）

> 列：R-ID | 需求 | 来源（task-info + ADR）| 验证方式（可命令复现）| 状态 | 交付物 | 评分项。
> **R-1xx** 语言与解释器 · **R-2xx** 自动测试 · **R-3xx** 性能 · **R-4xx** 文档 · **R-5xx** 应用 · **R-6xx** 环境与提交物。

### 1.1 语言与解释器（评分项 1 = 20 分）

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-101 | 支持 `int` / `float` / `string` / `bool` / `nil` 基本数据类型及其字面量与运算（`/` 恒返回 float；`div` 向下取整；`%` Python 取模） | task-info §1 + D-007/D-011/D-015 A3 | `cargo run -q -- run examples/hello.lfz` → stdout `Hello, LFZ!`、exit 0；`cargo test`（361 库单测含类型/运算用例）全绿 | 通过 | 交付物 1、2；`src/value.rs`,`src/evaluator.rs` | 1 |
| R-102 | 支持数组与 struct 类型（含**合一 struct**：struct 兼作字典/实例模板） | task-info §1 + D-007/D-011 | `cargo run -q -- run docs/reports/fixtures-p3/04_containers.lfz` → exit 0；`cargo test`（value/struct 用例） | 通过 | 交付物 1、2；`src/value.rs` | 1 |
| R-103 | 支持基本输入输出（`print` / `eprint` / `input`；`;;` 变量 dump 通道随 `print`（普通模式 stdout，`--json` 模式 stderr）） | task-info §1 + D-012 | `cargo run -q -- run examples/hello.lfz` → 输出 `Hello, LFZ!`；`cargo test`（`cli`/`test_runner` 集成含 print 流断言） | 通过 | 交付物 1、2；`src/builtins.rs` | 1 |
| R-104 | 支持分支循环控制（`if`/`else` 及 `if` 表达式、`while`、`for-in`） | task-info §1 | `cargo run -q -- run docs/reports/fixtures-p3/02_control.lfz` → exit 0；`cargo test`（parser/evaluator 控制流用例） | 通过 | 交付物 1、2；`src/parser.rs`,`src/evaluator.rs` | 1 |
| R-105 | 支持函数定义与调用（`fn` 声明 / 函数字面量 / 闭包（按 cell 引用捕获，A2）/ 递归） | task-info §1 + D-015 A2 | `cargo run -q -- run docs/reports/fixtures-p3/03_functions.lfz` → exit 0；`cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz` → 5 行 `1 2 3 / 99 99 / [3, 1, 2]  [1, 2, 3] / {me: <cycle>} / true`、exit 0 | 通过 | 交付物 1、2；`src/evaluator.rs`,`src/env.rs` | 1 |
| R-106 | **特色①** 管道 `\|>` + `_` 占位（解析期脱糖为 `Call`；`\|>` 优先级比 `+ - * / %` 松、比比较紧） | task-info §1 注意 + D-007/D-011/D-012 | `cargo run -q -- run docs/reports/fixtures-p3/05_pipe.lfz` → 9 行输出（`4 3 15 15 5` + `[0,2,4,6,8] [1,2,3] [2,4] ABC`）、exit 0 | 通过 | 交付物 1、2；`src/parser.rs` | 1（+4 语法说明） |
| R-107 | **特色②** 合一 struct（`s.k ≡ s["k"]`；方法字段不入数据面，A5） | task-info §1 注意 + D-007/D-011/D-015 A5 | `cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz` 第 4 行 `{me: <cycle>}`；`cargo test`（A5 方法字段用例） | 通过 | 交付物 1、2；`src/value.rs`,`src/builtins.rs` | 1（+4） |
| R-108 | **特色③** 富字符串插值 `${}`（`{` 在字符串内为字面字符；lexer `CODE/STR/INTERP` 模式栈；format_spec 支持） | task-info §1 注意 + D-007/D-012 | `cargo run -q -- run docs/reports/fixtures-p3/06_interp.lfz` → exit 0；`cargo test`（lexer/parser 插值用例） | 通过 | 交付物 1、2；`src/lexer.rs`,`src/parser.rs` | 1（+4） |
| R-109 | **特色④** 结构化错误 + `assert`/`check`/`fail`（`check` **非致命**：警告→stderr、返回 false、不中断；仅 `assert`/`fail` 致命 `AssertionError`） | task-info §1 注意 + D-007/D-011/D-015 A4 | `cargo run -q -- run docs/reports/fixtures-p3/08_builtins.lfz` → exit 0；`cargo test`（`check` 非致命用例） | 通过 | 交付物 1、2；`src/builtins.rs`,`src/evaluator.rs` | 1（+4） |
| R-110 | **特色⑤** 确定性语义「无魔法」（求值顺序左→右 B1；`for` 迭代快照 B2；键 UTF-8 字节序 B3；越界/缺字段/溢出/类型不符一律结构化报错；NaN/±Inf 规则 B5） | task-info §1 注意 + D-007/D-015 B1–B3/B5 | `cargo test`（evaluator 求值顺序/键序/快照用例）全绿；`cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz` | 通过 | 交付物 1、2；`src/evaluator.rs` | 1（+4） |
| R-111 | `.lfz` **文件**首行**恰好** `#42`（三字符 + 行终止符）；**仅按扩展名**（ASCII 大小写不敏感 `.lfz`/`.LFZ`）触发，REPL/stdin/`-e` 豁免；违规 → `CosmosAnswerError: 你忘记了宇宙的答案`（无源码行/插入符） | D-013/D-014/D-016 B2 | `Set-Content -Encoding utf8 _t.lfz 'print(1)'; cargo run -q -- run _t.lfz` → stderr 末行 `CosmosAnswerError: 你忘记了宇宙的答案`、exit 2；`cargo test`（loader/`#42` 用例） | 通过 | 交付物 2；`src/loader.rs`,`src/cli.rs` | 1 |
| R-112 | **Python 风格错误模型**：基类 `LfzError` + **12 个具体错误类**（`CosmosAnswerError`/`SyntaxError`/`NameError`/`TypeError`/`IndexError`/`FieldError`/`ZeroDivisionError`/`OverflowError`/`ValueError`/`IOError`/`AssertionError`/`RecursionError`）；输出 `类名: 中文消息` + `line N, col M` + 源行 + 插入符；运行期帧栈非空才带 `Traceback`（首 10+尾 30 折叠；零帧省略头）；**禁止 `E-xxx`**；退出码 **0/1/2**（成功 / `lfz test` 失败 / 其他错误·环境） | D-008/D-013/D-014/D-015 A7/D-021 | `cargo run -q -- run --json <含 1/0 的程序>` → stdout 单行 JSON，`error` 为 12 类名之一且不含 `E-xxx`、exit 2；`cargo test`（error/cli 格式化用例） | 通过 | 交付物 2；`src/error.rs`,`src/cli.rs` | 1 |
| R-113 | `§10.7` 内置函数表 **69 个**（54 + 15），data-last；容器更新返回新值 A1 | D-015 B9 + v1.1 ADR | `cargo test` 中内置名称表一致性测试全绿；`& ./docs/reports/T11-v11-reverification-evidence/run.ps1` → 74/74 PASS（含新增内置参数错误） | 通过 | 交付物 2；`src/builtins.rs`（runtime-dev）；T11-v11-reverification.md | 1 |
| R-114 | 语法体现**自身特色**（5 特色组合），**不与任何现存语言完全相同** | task-info §1 注意 + D-007 | `docs/spec/` 记录 5 特色；`cargo run -q -- run docs/reports/fixtures-p3/05_pipe.lfz`（`\|>`+`_` 组合）与 `spec_9_4_current.lfz`（环安全 `<cycle>`）可运行 | 通过 | 交付物 1；`docs/spec/` | 1（+4） |
| R-115 | 实现语言 = **Rust**，**仅 std、零第三方依赖** | D-011 | `Get-Content Cargo.toml` 的 `[dependencies]` 段为空；`cargo build` 0 warning | 通过 | 交付物 2；`Cargo.toml` | 1 |

| R-116 | `range(n)` 兼容；`range(lo, hi)` 返回普通 int 数组、半开区间，hi≤lo 为空；仅 1/2 参，容量溢出为 OverflowError | v1.1 ADR；interface-contract §10.7 | `target/debug/lfz.exe run tests/lfz/test_v11_range.lfz` → exit 0；独立 witness 脚本 → range/range_capacity 及 0/3 参、类型错误 PASS | 通过 | 交付物 2、3；src/builtins.rs（runtime-dev）、tests/lfz/test_v11_range.lfz（test-engineer） | 1、2 |
| R-117 | 新增字符串五内置 `endsWith` / `indexOf` / `padEnd` / `padStart` / `substring`；data-last、Unicode 标量、夹取半开区间、新值；空 fill 恒 ValueError | v1.1 ADR；interface-contract §10.7 | `target/debug/lfz.exe run tests/lfz/test_v11_string.lfz` → exit 0；独立 witness unicode/pad/fill_empty/fill_start → PASS（含精确中文错误） | 通过 | 交付物 2、3；src/builtins.rs（runtime-dev）、tests/lfz/test_v11_string.lfz（test-engineer） | 1、2 |
| R-118 | 新增 `ord` / `chr`：恰好单 Unicode 标量与合法码点互转；代理区、非法范围、非单标量 ValueError；无隐式类型转换 | v1.1 ADR；interface-contract §10.7/§10.8 | `target/debug/lfz.exe run tests/lfz/test_v11_unicode_math.lfz` → exit 0；独立 witness codepoints/ord_count 与非法码点 → PASS | 通过 | 交付物 2、3；src/builtins.rs（runtime-dev）、src/error.rs（core-dev）、tests/lfz/test_v11_unicode_math.lfz（test-engineer） | 1、2 |
| R-119 | 新增 `sin` / `cos` / `log` / `exp`：int 加宽 float，IEEE 口径；log(0)=-Inf、log(负数)=NaN、exp 溢出=+Inf | v1.1 ADR；interface-contract §10.7；semantics §4.5.6 | `target/debug/lfz.exe run tests/lfz/test_v11_unicode_math.lfz` → exit 0；独立 witness ieee 与各类型错误 → PASS | 通过 | 交付物 2、3；src/builtins.rs（runtime-dev）、tests/lfz/test_v11_unicode_math.lfz（test-engineer） | 1、2 |
| R-120 | 新增 `contains(v, xs)`：仅 array，按深结构/环安全相等查找；struct 键沿用 has | v1.1 ADR；interface-contract §10.7；semantics §4.5.9 | `target/debug/lfz.exe run tests/lfz/test_v11_string.lfz` → exit 0；独立 witness contains（嵌套、环、int/float 相等）及非 array 类型错误 → PASS | 通过 | 交付物 2、3；src/builtins.rs（runtime-dev）、tests/lfz/test_v11_string.lfz（test-engineer） | 1、2 |
| R-121 | 新增 `readFile` / `writeFile` / `appendFile`：UTF-8 整文件读/覆盖/追加，写追加返回 nil；路径按进程 CWD 原样解析、不展开 ~、不自动创建父目录；失败为既有 IOError，非法 UTF-8 有精确消息 | 用户 2026-10-24 单独追认 + interface-contract §10.7；semantics §8 | `& ./docs/reports/T11-v11-reverification-evidence/run.ps1` → io/cwd/io_invalid/io_missing/io_parent/io_append_parent/io_no_tilde 与类型/参数错误全 PASS；源码/命令/输出见 evidence/witness.json | 通过 | 交付物 2、3；src/builtins.rs（runtime-dev）、tests/lfz/test_v11_io.lfz（test-engineer）；一键测试环境缺陷 V01 另见 R-202 | 1、2 |
| R-122 | 固化既有 string 索引限制：读/写均 TypeError「运算符 '[]' 不支持 string 与 array / struct」；逐标量遍历用 split("", s) 后遍历数组 | v1.1 ADR；semantics §3.7/§4.5.11 | `target/debug/lfz.exe test --json` → v11_string_index_read、v11_string_index_write 两个清单负例 PASS；`target/debug/lfz.exe run tests/lfz/test_v11_string.lfz` → exit 0 | 通过 | 交付物 1、2、3；docs/spec/semantics.md（language-architect）、src/evaluator.rs（runtime-dev，既有行为）、tests/fixtures/v11_string_index_*.lfz（test-engineer） | 1、2、4 |
### 1.2 自动测试（评分项 2 = 20 分）

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-201 | 完整**黑盒测试集**，脚本用 LFZ 编写，覆盖**所有功能**，每特性 ≥3 用例（正常/边界/错误） | task-info §2 + D-021 | `cargo run -q -- test` → 汇总「全部 PASS」、exit 0；`tests/coverage-matrix.md` 覆盖全部特性且每特性 ≥3 用例 | 待办 | 交付物 3；`tests/**/*.lfz`,`tests/coverage-matrix.md` | 2 |
| R-202 | **一个命令**执行所有测试（默认发现、排除 fixtures、稳定排序；0 用例 exit 2）；干净 CWD 无遗留 Temp 也能一键执行 | task-info §2 + D-021 + runner-contract §2/§5；V01 可复现交付要求 | `target/debug/lfz.exe test --json` → 130/130 PASS / exit 0；verifier 在原无 Temp 的干净 CWD 对原 tests 副本同命令复验 130/130 PASS；Temp 同名普通文件冲突 exit 2、stdout 空且内容保留；证据 clean-cwd-fixed.json / temp-file-conflict.json（T11-v11-reverification-evidence/）；runner 15/15、全量 494/494 PASS 由 tooling-dev 另行执行 | 通过 | 交付物 3；src/test_runner.rs（tooling-dev）；docs/reports/T11-v11-reverification.md（verifier，13:41 关闭 V01） | 2 |
| R-203 | 测试失败可定位：用例名 + 期望 vs 实际 + `§8.2` 位置（`File "<path>", line N[, in func]` + 源码行 + 插入符 + `类名: 消息`） | task-info §2 + runner-contract §6 | 注入失败用例 → 报告含 `FAIL <name> (<path>:<line>:<col>)` + 缩进错误块；`cargo test`（`tr3_*` 用例） | 通过 | 交付物 3；`src/test_runner.rs`,`src/cli.rs` | 2 |
| R-204 | 负例须经 `tests/cases.json` 的 `expect.error` 声明并使用**非自动发现**夹具（`tests/fixtures/`）；清单类名取 12 类之一 | D-021 + runner-contract §3/§4 | `cargo run -q -- test` 对清单声明的负例判 PASS；类名非法 → exit 2 | 待办 | 交付物 3；`tests/cases.json`,`tests/fixtures/` | 2 |

### 1.3 性能（评分项 3 = 10 分）

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-301 | 提供 LFZ 性能脚本与**等价** Python 脚本 | task-info §3 | `benchmarks/` 下两脚本存在且可运行（LFZ 脚本首行 `#42`） | 待办 | 交付物 4；`benchmarks/` | 3 |
| R-302 | 性能**报告**，含方法/环境/数据/结论，采用**预热 + 多轮 + 中位数**的公平对比 | task-info §3 | `docs/reports/performance.md` 存在，且含上述四要素与数值表 | 待办 | 交付物 4；`docs/reports/performance.md` | 3 |
| R-303 | 性能结论可复现（报告内命令可原样执行） | D-008 | 按报告命令重跑得同量级结果 | 待办 | 交付物 4；`benchmarks/` | 3 |

### 1.4 文档（评分项 4 = 20 分）

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-401 | LFZ **语法规则文档**（含 EBNF、样例、词法/文法/语义/错误/接口契约；v1 冻结后按 ADR 补钉） | task-info §4/提交 1 + D-016 + v1.1 ADR | `Get-Item docs/spec/syntax.md,docs/spec/semantics.md,docs/spec/interface-contract.md` 均非空；2026-10-24 实际字节数按顺序 **70,043 / 41,143 / 45,830**（记录快照，后续合法补钉可增长）；规范内容对应 R-113、R-116~R-122 | 通过 | 交付物 1、5；`docs/spec/`（language-architect） | 4 |
| R-402 | **人类向**开发指南/手册（教程 + 语法 + 运行/错误说明） | task-info §4 | `docs/guide/` 存在且含入门、语法、运行与错误章节 | 待办 | 交付物 5；`docs/guide/` | 4 |
| R-403 | **给 AI 的**开发指南/skill，并经**全新 agent 实测**验证可顺利用 LFZ 写代码 | task-info §4 | `.opencode/skills/lfz-programming/`（或等价文档）存在 + 一份实测记录（命令与结果） | 待办 | 交付物 5；`.opencode/skills/lfz-programming/` | 4 |
| R-404 | AI 指南**硬性内容**：写死「`.lfz` 首行 `#42`」+ 12 个错误类清单 + 禁止旧 `E-xxx` + 示例一律带头 | D-013/D-014 §下游硬性要求 | 指南文本含 `#42` 与 12 类名；示例代码块首行均为 `#42` | 待办 | 交付物 5；`.opencode/skills/lfz-programming/` | 4 |

### 1.4.1 v1.1 复杂度标注（R-405）

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-405 | 文档复杂度与最新 spec 一致：indexOf 保守最坏 O(n·m)；pad* O(n+m+输出宽度) 含计数；substring 非空 O(min(to,len(s))) 含扫描前缀，空区间可直接返回；ord 成功 O(1)、完整计数错误 O(n)；contains 线性扫描；文件 IO O(文件大小)；循环拼接与循环 push 均可能 O(n²)，批量构造用预分配下标写或 map 后 join（O(片段数+总输出大小)） | interface-contract §10.7 最新 complexity 补钉；semantics §3.7/§4.5.11；ADR 2026-09-27 23:40 | `rg -n 'indexOf\|padEnd\|substring\|ord\|push\|join' docs/spec/interface-contract.md docs/guide .opencode/skills/lfz-programming`，按本条逐项核对指南与 spec 公式一致；任一指南仍宣称循环 push 为 O(n)、ord 所有路径 O(1) 或遗漏扫描成本则失败；2026-10-24 文本核对 PASS：reference.md §3.5/§5、SKILL.md §4.6/§6.1 与 IC §10.7 的上述公式一致；contains 深比较成本另计；历史 VERIFICATION.md 中旧 push 建议已明确否定。未运行性能测试 | 通过 | 交付物 1、5；docs/spec/（language-architect）、docs/guide/（docs-writer）、.opencode/skills/lfz-programming/（ai-dx-engineer） | 4 |
### 1.5 应用（评分项 5 = 30 分）

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-501 | 用 LFZ 实现**功能完整、可运行、代码 ≥200 行**的应用（选题 = 排序算法可视化，ASCII 条形图） | task-info §5 + D-011 | `cargo run -q -- run app/<入口>.lfz` 可运行、exit 0；`app/` 下 `.lfz` 总行数 ≥200 | 待办 | 交付物 6；`app/` | 5 |
| R-502 | **开发记录**（Agent 提示词 / 迭代 / 踩坑 / 修复） | task-info §5 | `app/DEV_RECORD.md` 存在且含上述四类内容 | 待办 | 交付物 6；`app/DEV_RECORD.md` | 5 |
| R-503 | 应用文件合规：`.lfz` 首行 `#42`、错误用类名 + 中文消息、无 `E-xxx` | D-013/D-014 | 扫描 `app/**/*.lfz` 首行均为 `#42`；应用代码不含 `E-` 编号引用 | 待办 | 交付物 6；`app/` | 5 |

### 1.6 环境与提交物（无独立分值）

| R-ID | 需求 | 来源 | 验证方式（可命令复现） | 状态 | 交付物 | 评分项 |
|------|------|------|------------------------|------|--------|--------|
| R-601 | 解释器实现语言**无限制** | 环境要求 1 | `Cargo.toml` 存在，`cargo build` 成功 | 通过 | 交付物 2；`Cargo.toml` | — |
| R-602 | 编程 Agent 及大模型使用**无限制** | 环境要求 2 | 团队架构与 `.opencode/` 存在 | 通过 | 交付物 7；`.opencode/` | — |
| R-603 | **必须使用 GIT** 版本管理 | 环境要求 3 | `git log --oneline` 非空（含 `e060b21`→`7527792`）；标签 `v0.1.0`/`v0.2.0` | 通过 | 交付物 7；`.git/` | — |
| R-604 | 提交物 1：LFZ 语法规则文档 | 提交内容 1 | 同 R-401 | 通过 | `docs/spec/` | 4 |
| R-605 | 提交物 2：可执行 LFZ 的解释器源程序 | 提交内容 2 | 同 R-101 ~ R-115（`cargo run -q -- run examples/hello.lfz` → `Hello, LFZ!`） | 通过 | `src/` | 1 |
| R-606 | 提交物 3：LFZ 完整黑盒测试集 | 提交内容 3 | 同 R-201 ~ R-204 | 待办 | `tests/` | 2 |
| R-607 | 提交物 4：LFZ 性能测试报告 | 提交内容 4 | 同 R-301 ~ R-303 | 待办 | `benchmarks/`、`docs/reports/performance.md` | 3 |
| R-608 | 提交物 5：开发指南文档 + 给 AI 的文档 | 提交内容 5 | 同 R-402 ~ R-404 | 待办 | `docs/guide/`、`.opencode/skills/lfz-programming/` | 4 |
| R-609 | 提交物 6：应用源代码 + 开发记录 | 提交内容 6 | 同 R-501 ~ R-503 | 待办 | `app/`、`app/DEV_RECORD.md` | 5 |
| R-610 | 提交物 7：所有工作的 Git 历史记录 | 提交内容 7 | 同 R-603；远程 `https://github.com/NeitherTourRest/lfz-programing-language` 可达 | 通过 | `.git/` | — |
| R-611 | 提交物 8：系统介绍 PPT | 提交内容 8 | `docs/slides/` 存在且含 `.pptx`（或等价） | 待办 | `docs/slides/` | — |
| R-612 | 线下验收答辩（讲 PPT + 演示 + 回答提问） | 作业验收 | 演示脚本 + 答辩预案齐备；现场演示 `cargo run -- run examples/hello.lfz` 与 `cargo run -- test` 成功 | 待办 | `docs/slides/` + 演示脚本 | — |

---

## 2. 评分权重映射（20/20/10/20/30）

| 评分项 | 分值 | 对应 R-ID | 硬性通过条件 |
|--------|------|-----------|--------------|
| 1. LFZ 解释器实现 | **20** | R-101 ~ R-122（+ R-605、R-601） | `cargo run -q -- run examples/hello.lfz` → `Hello, LFZ!`；`cargo test` 全绿；5 特色可运行；`#42`+Python 错误模型生效 |
| 2. LFZ 解释器自动测试脚本 | **20** | R-201 ~ R-204（+ R-606） | **一个命令**跑全部；覆盖全部功能；每特性 ≥3 用例（正常/边界/错误）；失败可定位 |
| 3. LFZ 解释器性能测试 | **10** | R-301 ~ R-303（+ R-607） | 报告交付；LFZ vs Python **公平对比**（预热/多轮/中位数 + 报告可复现） |
| 4. 语法说明 + 人/AI 开发指南 | **20** | R-401 ~ R-405（+ R-604、R-608） | 语法文档冻结 + 人类手册 + AI 指南/skill 齐备且 AI 指南经实测 |
| 5. 利用编程 Agent 开发程序 | **30** | R-501 ~ R-503（+ R-609） | 应用可运行、≥200 行、功能完整、含开发记录 |
| （支撑）环境与提交完整性 | 无独立分值 | R-601 ~ R-603、R-610 ~ R-612 | Git 历史完整 + 8 项提交物齐备 + 答辩载体 |

---

## 3. 需求变更记录（开工后需求口径变化）

> 记录**开工后**对初版（R-001 ~ R-023，见附录 A）的口径**推翻 / 补充 / 细化**，逐条附 ADR 依据。初版条目**均未撤销**，仅被细化到 R-1xx ~ R-6xx。

| # | 变更项 | 初版口径（R-0xx） | 现行 v1 口径 | 依据 ADR | 影响 |
|---|--------|-------------------|--------------|----------|------|
| 1 | **实现语言** | R-012「语言无限制」，默认建议 Python 3.13（D-004/D-006 提案） | **Rust**，cargo 工程，**仅 std、零第三方依赖** | D-006（提案）→ **D-011（用户拍板改选 Rust）** | R-115、R-601；架构按 `enum Value` + `Rc<RefCell<…>>` + `Result<T, LfzError>` 重规划 |
| 2 | **v1 功能范围** | R-006「有特色即可」 | **5 特色全纳入 + `float`**（管道/合一 struct/富插值/结构化错误+assert·check/确定性语义）；11 项延后 v1.1（生成器、match、REPL、模块等） | D-007（提案）→ **D-011 确认** | 细化为 R-106 ~ R-110 |
| 3 | **语法口径** | 未规定块/语句/注释风格 | C 风格 `{}` 块；**换行终结**语句（无分隔符）；`;;` 变量 dump；`;` 单独出现恒 `SyntaxError`；多行续行**仅括号内**；`${}` 插值 | D-012（v0.2） | R-103、R-106、R-108 |
| 4 | **文件前导 `#42`** | 无此要求 | `.lfz` 文件首行**恰好** `#42`；**仅扩展名触发**（ASCII 大小写不敏感）；REPL/stdin/`-e` 豁免；违规 → `CosmosAnswerError: 你忘记了宇宙的答案` | D-013（草案）→ **D-014 定案** | R-111；下游 test/docs/ai-dx/app 全部 `.lfz` 须带头 |
| 5 | **错误模型** | 初版无错误模型规定；D-012 曾引入 `E-xxx` 编号 | **Python 风格**：类名 + 中文消息 + `line/col` + 源行 + 插入符；**12 类 + 基类**；运行期带 `Traceback`；**取消 `E-xxx`**；退出码 **0/1/2** | D-013（取代 D-012 的 `E-xxx` 部分）/ D-014 / D-015 A7 新增 `RecursionError` / D-008 退出码 | R-112 |
| 6 | **可移植性** | 无 | 行终止符 `\n`/`\r\n`/`\r`（归一化）；文件开头 UTF-8 BOM 静默跳过；**非 UTF-8 → `SyntaxError`**（先编码校验后前导校验） | D-013/D-014 | R-111、R-112 |
| 7 | **语义定稿 A1–A7** | 无 | 引用语义（A1）；按 cell 捕获（A2）；除法/取模 Python 一致（A3）；`check` 非致命（A4）；方法不入数据面（A5）；环安全 `<cycle>`（A6）；`RecursionError`（A7） | **D-015** | R-101、R-105、R-107、R-109、R-110 |
| 8 | **求值语义补钉 B1–B13** | 无 | 左→右求值；`for` 快照；键 UTF-8 序；递归上限 10000；NaN/Inf 规则；`input()` EOF→`IOError`；struct 平拷贝；54 内置 data-last 等 | **D-015 B** | R-110、R-113 |
| 9 | **可解析性补钉 M1–M6 / A9 / A27** | 无 | `${…}` 内禁裸换行；`=> {…}` 恒为块；`_` 只绑定最近管道 RHS；**无裸块语句**（语句首 `{` = 匿名 struct 字面量）；`.self` 非法（用 `s["self"]`） | D-015 C + P3.11 裁定（bug-08 / bug-07） | R-106、R-107 |
| 10 | **spec v1 冻结** | 无 | `docs/spec/{syntax,semantics,interface-contract}.md` 于 2026-09-23 冻结，**变更须先 ADR 后改文档** | **D-016** | R-401、R-114 |
| 11 | **冻结后补钉（post-v1）** | 无 | `min([])`/`max([])` → `ValueError::EmptyExtremum`「空数组没有极值（{func}）」；`randInt(lo≥hi)` → `ValueError::BadRange`；`pop([])` → `Index{idx:-1,len:0}`；`insert` 不支持负索引；`del` 仅数据字段；未闭合块注释 → `SyntaxError`；`let` 重绑定 → `TypeError::ImmutableRebind`；traceback 折叠（首 10+尾 30） | DECISIONS 2026-09-24 各条（契约缺口闭合 / 未闭合块注释裁定 / P3.11 裁定 1/3） | R-109、R-112、R-113 |
| 12 | **运行命令契约** | 待定（PROJECT_STATE §5 旧注） | `lfz run <file>` / `lfz test [路径…]` / `--json` 全局开关 / `--help`/`--version`；`--json` 下 stdout 恒为单行 JSON | D-008 + runner-contract v1.2（P4.1/P4.2/P4.2-fix） | R-202、R-203、R-204、R-112 |
| 13 | **交付物位置与判定** | 初版未指定路径 | `docs/spec/`、`src/`、`tests/`、`benchmarks/`、`docs/guide/`、`app/`、`.git/`、`docs/slides/` | D-009 / PROJECT_STATE §4 | R-6xx 全部 |
| 14 | **8 项提交物** | R-015 ~ R-022 逐项 | 保留 8 项，逐项配可命令验证 | task-info + D-016 | R-604 ~ R-611 |

---

## 4. 追溯矩阵（需求 ↔ 交付物 ↔ 评分项 ↔ 当前状态）

| R-ID | 交付物（位置） | 评分项 | 状态 | 复核命令 |
|------|----------------|--------|------|----------|
| R-101 | 交付物 2 `src/value.rs`,`src/evaluator.rs` | 1 | ✅ 已满足 | `cargo run -q -- run examples/hello.lfz` |
| R-102 | 交付物 1、2 `src/value.rs` | 1 | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3/04_containers.lfz` |
| R-103 | 交付物 2 `src/builtins.rs` | 1 | ✅ 已满足 | `cargo run -q -- run examples/hello.lfz` |
| R-104 | 交付物 2 `src/parser.rs`,`src/evaluator.rs` | 1 | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3/02_control.lfz` |
| R-105 | 交付物 2 `src/evaluator.rs`,`src/env.rs` | 1 | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz` |
| R-106 | 交付物 1、2 `src/parser.rs` | 1（+4） | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3/05_pipe.lfz` |
| R-107 | 交付物 1、2 `src/value.rs`,`src/builtins.rs` | 1（+4） | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz` |
| R-108 | 交付物 1、2 `src/lexer.rs`,`src/parser.rs` | 1（+4） | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3/06_interp.lfz` |
| R-109 | 交付物 1、2 `src/builtins.rs`,`src/evaluator.rs` | 1（+4） | ✅ 已满足 | `cargo run -q -- run docs/reports/fixtures-p3/08_builtins.lfz` |
| R-110 | 交付物 1、2 `src/evaluator.rs` | 1（+4） | ✅ 已满足 | `cargo test` |
| R-111 | 交付物 2 `src/loader.rs`,`src/cli.rs` | 1 | ✅ 已满足 | 构造无 `#42` 的 `.lfz` → `cargo run -q -- run <f>`（exit 2 + `CosmosAnswerError`） |
| R-112 | 交付物 2 `src/error.rs`,`src/cli.rs` | 1 | ✅ 已满足 | `cargo run -q -- run --json <含 1/0 的程序>` |
| R-113 | 交付物 2 `src/builtins.rs` | 1 | ✅ 已满足 | 69 内置一致性；见 §1.1 与 T11-v11-reverification.md（修复前证据） |
| R-114 | 交付物 1 `docs/spec/` | 1（+4） | ✅ 已满足 | 见 §6 命令 2/4 |
| R-115 | 交付物 2 `Cargo.toml` | 1 | ✅ 已满足 | `Get-Content Cargo.toml`（空 `[dependencies]`）+ `cargo build` |
| R-201 | 交付物 3 `tests/**/*.lfz` | 2 | ⬜ 未做 | `cargo run -q -- test`（P5 后应 exit 0） |
| R-202 | 交付物 3 `src/test_runner.rs` | 2 | 🟡 部分 | `cargo run -q -- test` |
| R-203 | 交付物 3 `src/test_runner.rs`,`src/cli.rs` | 2 | ✅ 已满足 | `cargo test`（`tr3_*`） |
| R-204 | 交付物 3 `tests/cases.json` | 2 | ⬜ 未做 | `cargo run -q -- test` |
| R-301 | 交付物 4 `benchmarks/` | 3 | ⬜ 未做 | — |
| R-302 | 交付物 4 `docs/reports/performance.md` | 3 | ⬜ 未做 | `Test-Path docs/reports/performance.md` |
| R-303 | 交付物 4 `benchmarks/` | 3 | ⬜ 未做 | — |
| R-401 | 交付物 1、5 `docs/spec/` | 4 | ✅ 已满足 | `Test-Path docs/spec/syntax.md,docs/spec/semantics.md,docs/spec/interface-contract.md` |
| R-402 | 交付物 5 `docs/guide/` | 4 | ⬜ 未做 | `Test-Path docs/guide` |
| R-403 | 交付物 5 `.opencode/skills/lfz-programming/` | 4 | ⬜ 未做 | `Test-Path .opencode/skills/lfz-programming` |
| R-404 | 交付物 5 同上 | 4 | ⬜ 未做 | 指南含 `#42` + 12 类名 |
| R-501 | 交付物 6 `app/` | 5 | ⬜ 未做 | `cargo run -q -- run app/<入口>.lfz`；行数 ≥200 |
| R-502 | 交付物 6 `app/DEV_RECORD.md` | 5 | ⬜ 未做 | `Test-Path app/DEV_RECORD.md` |
| R-503 | 交付物 6 `app/` | 5 | ⬜ 未做 | 扫描 `app/**/*.lfz` 首行 = `#42` |
| R-601 | 交付物 2 `Cargo.toml` | — | ✅ 已满足 | `cargo build` |
| R-602 | 交付物 7 `.opencode/` | — | ✅ 已满足 | `Test-Path .opencode` |
| R-603 | 交付物 7 `.git/` | — | ✅ 已满足 | `git log --oneline` |
| R-604 | `docs/spec/` | 4 | ✅ 已满足 | 同 R-401 |
| R-605 | `src/` | 1 | ✅ 已满足 | `cargo run -q -- run examples/hello.lfz` |
| R-606 | `tests/` | 2 | ⬜ 未做 | 同 R-201 |
| R-607 | `benchmarks/`、`docs/reports/performance.md` | 3 | ⬜ 未做 | 同 R-302 |
| R-608 | `docs/guide/`、skills | 4 | ⬜ 未做 | 同 R-402/R-403 |
| R-609 | `app/` | 5 | ⬜ 未做 | 同 R-501 |
| R-610 | `.git/` | — | ✅ 已满足 | `git ls-remote origin`（连通时） |
| R-611 | `docs/slides/` | — | ⬜ 未做 | `Test-Path docs/slides` |
| R-612 | `docs/slides/` + 演示脚本 | — | ⬜ 未做 | 现场演示 |

---

## 5. 覆盖状态汇总

**总计 41 条**（R-101 ~ R-115 共 15；R-201 ~ R-204 共 4；R-301 ~ R-303 共 3；R-401 ~ R-404 共 4；R-501 ~ R-503 共 3；R-601 ~ R-612 共 12）。

| 覆盖状态 | 条数 | 占比 | 主要缺口 |
|----------|------|------|----------|
| ✅ 已满足 | **23** | 56% | —（P3+P4 已交付部分） |
| 🟡 部分 | **1** | 2% | R-202（runner 机制已就绪，缺 LFZ 黑盒用例） |
| ⬜ 未做 | **17** | 41% | 黑盒测试集（P5）、性能（P6）、指南（P7）、应用（P8）、PPT（P10） |

**按评分项看**：

| 评分项 | 状态 |
|--------|------|
| 1. 解释器（20） | ✅ 主体已满足（R-101~R-115 全通过）；P3 独立验收 rev.3 PASS（10/10 缺陷闭环） |
| 2. 自动测试（20） | 🟡 runner 基础设施已就绪（R-202/R-203），**待 P5 落地 LFZ 黑盒用例（R-201/R-204）** |
| 3. 性能（10） | ⬜ 未做（P6） |
| 4. 文档（20） | 🟡 语法文档 ✅（R-401）；**人/AI 指南 ⬜（R-402~R-404，P7）** |
| 5. 应用（30） | ⬜ 未做（P8） |
| 支撑（Git/提交完整性） | 🟡 Git ✅（R-603/R-610）；PPT/答辩 ⬜（R-611/R-612，P10） |

**8 项提交物交付状态**：

| # | 提交物 | 状态 |
|---|--------|------|
| 1 | LFZ 语法规则文档 | ✅ 已冻结（`docs/spec/`，D-016） |
| 2 | 解释器源程序 | ✅ 已交付（`src/`，431 测试全绿，`cargo build` 0 warning） |
| 3 | 完整黑盒测试集 | 🟡 runner 已就绪；LFZ 用例集待 P5 |
| 4 | 性能测试报告 | ⬜ 待 P6 |
| 5 | 开发指南（人 + AI） | ⬜ 待 P7 |
| 6 | 应用源码 + 开发记录 | ⬜ 待 P8 |
| 7 | Git 历史记录 | ✅ 已建立（本地 + 远程 Public） |
| 8 | 系统介绍 PPT | ⬜ 待 P10 |

---

## 6. 验收清单（可直接照跑）

> 在**项目根**运行。命令中 `cargo run -q --` 抑制 cargo 自身输出；`$LASTEXITCODE` 取退出码。

```powershell
# 0. 构建：期望 0 warning / 0 error
cargo build

# 1. 全量测试：期望 431 passed / 0 failed / 0 ignored（数值随迭代增长，关键为 0 failed）
cargo test

# 2. 端到端 Hello：期望 stdout 'Hello, LFZ!'，exit 0
cargo run -q -- run examples/hello.lfz

# 3. 综合样例（引用语义 A1 / 闭包捕获 A2 / 环安全 A6）：
#    期望 5 行：1 2 3 / 99 99 / [3, 1, 2]  [1, 2, 3] / {me: <cycle>} / true；exit 0
cargo run -q -- run docs/reports/fixtures-p3-rev3/spec_9_4_current.lfz

# 4. 5 特色演示（管道 |>/_ 等）：期望 9 行输出、exit 0
cargo run -q -- run docs/reports/fixtures-p3/05_pipe.lfz

# 5. `#42` 前导强制（R-111）：期望 stderr 含 'CosmosAnswerError: 你忘记了宇宙的答案'、exit 2
Set-Content -Encoding utf8 _accept_nohdr.lfz 'print(1)'
cargo run -q -- run _accept_nohdr.lfz; "exit=$LASTEXITCODE"
Remove-Item _accept_nohdr.lfz

# 6. Python 风格错误 + 机器可读（R-112）：期望 stdout 单行 JSON（error 为 12 类名之一、无 E-xxx）、exit 2
Set-Content -Encoding utf8 _accept_div.lfz "#42`nprint(1 / 0)"
cargo run -q -- run --json _accept_div.lfz; "exit=$LASTEXITCODE"
Remove-Item _accept_div.lfz

# 7. 一键黑盒测试（R-201/R-202）：P5 交付后应 exit 0 且全部 PASS；
#    当前无 LFZ 用例 → exit 2 且 stderr 'lfz test: 未发现任何测试用例'
cargo run -q -- test; "exit=$LASTEXITCODE"

# 8. 版本与帮助：期望 'lfz 0.1.0'、exit 0
cargo run -q -- --version

# 9. 交付物存在性断言（期望全部 True）
Test-Path docs/spec/syntax.md, docs/spec/semantics.md, docs/spec/interface-contract.md, examples/hello.lfz, src/lib.rs

# 10. 第三方依赖为零（R-115）：期望 [dependencies] 段为空
Get-Content Cargo.toml | Select-String -Pattern '\[dependencies\]' -Context 0,3

# 11. Git 历史（R-603/R-610）：期望非空、含 v0.1.0 / v0.2.0 标签
git log --oneline -5; git tag
```

**P9 验收对表要点**：verifier 对 R-1xx（评分项 1）以命令 2–6 为主；R-2xx 以命令 7 为主（P5 交付后）；R-4xx 以命令 9 + 人工核 `docs/guide/`、skills；R-5xx 以 `cargo run -- run app/<入口>.lfz` + 行数统计；R-6xx 以命令 9–11。

**已知输入依赖**：命令 5/6 需 PowerShell 以 UTF-8 写文件（`Set-Content -Encoding utf8`）；若在 Windows PowerShell 5.1 下 BOM 干扰，可改用 `[IO.File]::WriteAllText("_accept_div.lfz","#42`nprint(1 / 0)")`（无 BOM）。

---

## 附录 A. 初版需求条目索引（R-001 ~ R-023，保留）

> 初版条目**均未撤销**，作为 task-info 原始要求的来源锚点保留；其口径已细化到 §1 的 R-1xx ~ R-6xx（见「细化到」列）。外部引用（如 `BRAINSTORM.md` 的 R-007/R-008/R-023、`docs/tooling/runner-contract.md` 的 R-008）**依然有效**。

| 初版 R-ID | 需求（保留原文摘要） | 来源 | 细化到 |
|-----------|----------------------|------|--------|
| R-001 | 支持整数、字符串等基本数据类型 | task-info §1 | R-101 |
| R-002 | 支持数组、结构类型 | task-info §1 | R-102 |
| R-003 | 支持基本输入输出 | task-info §1 | R-103 |
| R-004 | 支持分支循环控制 | task-info §1 | R-104 |
| R-005 | 支持函数定义和调用 | task-info §1 | R-105 |
| R-006 | 语法体现自身特色，不与现存语言完全相同 | task-info §1 注意 | R-106 ~ R-110、R-114 |
| R-007 | 完整黑盒测试集，覆盖所有功能 | task-info §2 | R-201 |
| R-008 | 一个命令执行所有测试 | task-info §2 | R-202、R-203 |
| R-009 | 性能测试脚本并与 Python 对比 | task-info §3 | R-301 ~ R-303 |
| R-010 | 开发指南供编程 Agent 使用 | task-info §4 | R-402、R-403 |
| R-011 | Agent 辅助开发 ≥200 行应用 | task-info §5 | R-501、R-502 |
| R-012 | 解释器实现语言无限制 | 环境要求 1 | R-115、R-601 |
| R-013 | 编程 Agent 及大模型使用无限制 | 环境要求 2 | R-602 |
| R-014 | 必须使用 GIT 版本管理 | 环境要求 3 | R-603 |
| R-015 | LFZ 语法规则文档 | 提交内容 1 | R-401、R-604 |
| R-016 | LFZ 解释器源程序 | 提交内容 2 | R-605 |
| R-017 | LFZ 完整黑盒测试集 | 提交内容 3 | R-606 |
| R-018 | LFZ 性能测试报告 | 提交内容 4 | R-607 |
| R-019 | 开发指南文档 + AI 文档 | 提交内容 5 | R-608 |
| R-020 | 应用源代码 + 开发记录 | 提交内容 6 | R-609 |
| R-021 | Git 历史记录 | 提交内容 7 | R-610 |
| R-022 | 系统介绍 PPT | 提交内容 8 | R-611 |
| R-023 | 线下验收答辩 | 作业验收 | R-612 |
