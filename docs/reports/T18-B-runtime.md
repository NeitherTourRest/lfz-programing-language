# T18-B运行时源码交接
2026-10-31。按冻结三spec§24/T18-B-contract；仅evaluator/builtins及所属unit。不编译、不公网、不Git、不后继迭代。

EvalCapabilities新增pub http字段，属于明确批准的Rust literal源迁移，旧函数/EvalContext/EvalLimits字段不改，Default两能力false。三context入口沿现构造传整个能力快照，modules/home函数不独立携带能力。CLI literal由tooling机械迁移。

新增特殊httpRequest六参，与exec同级且名字绑定可遮蔽；非first-class，通用builtins call/call_with固定HttpDisabled。注册75名字；注册测试仅追加特殊名及计数，旧表72断言不改。

validate_http按六外层类型→headers嵌套类型/shape→方法→URL→headers值→body→两范围→能力；method/url/headers复用tooling纯helper，不重复造规则。头总量在trim前由helper判断，提交前trim SP/HTAB；GET/HEAD空body与UTF8字节长度检查。无网络副作用。

owned HttpRequest/HttpControl传后端；consume_http_report先合并HTTP独立诊断，再映射局部ValueError/IO或全局ExecutionLimit；UTF8严格解码，返回status/headers/body三键，重复头pair数组原API顺序，404/500正常值。body/headers限制、lowercase头名、native终止同轮优先与安全回收均由owned后端保证，不把合成unit当平台实测。

TracedRun/Interp新增私有http_diagnostics列表与slice accessor；legacy preflight literal空、普通/graph完成均转移列表。attempt未增加任何权限/诊断快照，现有恢复代码不触碰两列表；旧host_diagnostics及子进程文案不改。CLI/runner需检查两列表，Err保原JSON并按process→HTTP附诊断，Ok非空须HostFailure，由tooling所有权处理。

新增4项unit未编译：六参/嵌套校验优先与trim；子进程权限不能推导HTTP及通用call禁用；404三键/重复头/非法UTF8；HTTP诊断跨attempt保持、两sidecar隔离和全局不可recover。旧测试未删改。Temp/T18-B-runtime/preimplementation-search.txt是实施前新符号静态搜索缺失记录（非可运行红测）；禁止编译阶段未伪造编译失败。

最终源码evaluator SHA42F1BA0D4E22A9CD9D0C705436B65FA797AAB65E2A0C1FAF3AA5D5C9EAC4B8AD，builtins SHA8CD12A2F415811EB92382F4A0252A4E8C334E8D112331E2C09DAC9F7B4781D52。git diff --check通过（只有换行提示）。已停写交tooling唯一完整release，新增unit/真实loopback/旧回归均待统一验证，不套A成绩。R304OPEN，不声明性能或Linux支持。
