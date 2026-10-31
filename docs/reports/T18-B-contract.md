# T18-B HTTP 冻结契约
## 24. T18-B HTTP（2026-10-30 冻结）

root批准六参/API/限额/WinHTTP async及runtime review收敛。仅本轮HTTP实现验收交付后收口，不自动T19或恢复自动化。Windows先验，非Windows返回Unsupported；源码仍待root GO。本节不修改§23子进程文案/权限/限额。

### 24.1 调用、响应与校验
新增特殊内置httpRequest(method,url,headers,body,timeoutMs,maxResponseBytes)，75内置，派发/同名遮蔽/非first-class规则同exec；无上下文builtins调用必须HttpDisabled，不绕过权限。成功struct键序status:int、headers:array、body:string；最终HTTP状态100..599（非中间1xx）包含404/500均正常值。headers为[[lowercaseName,value],...]保重复及API观测顺序，不合并Set-Cookie，不保证原wire大小写。body严格UTF8，保BOM/换行、不猜charset、不解析JSON、不lossy替换；不支持任意二进制响应。

实参先旧左到右求值；完整校验后才触网，固定序：
1 数量恰6，沿ArgCount{name:"httpRequest",n:6,m}。
2 按六参顺序检查外层类型string/string/array/string/int/int；随后headers逐索引须array/恰2项/两项string，类型沿BadOperands op="httpRequest"及期望"string"/"array"/"int"；pair非2项为HttpHeaders。
3 方法仅ASCII大写GET/HEAD/POST/PUT/PATCH/DELETE/OPTIONS。
4 URL最多16384个ASCII bytes，绝对http://或https://（scheme大小写不敏感），非空ASCII host或括号IPv6、可选port1..65535；不含userinfo/fragment/反斜线/空白/控制/NUL；每百分号后恰两hex，非ASCII路径由调用者显式percent编码；缺path按/，保query。不支持其它scheme，不查代理/PAC。
5 headers最多256对：名称非空ASCII token（字母数字及!#$%&'*+-.^_、反引号、竖线、波浪号）；value仅SP..~与HTAB，去两端SP/HTAB后提交，禁止CR/LF/NUL。大小写不敏感禁止Host/Content-Length/Transfer-Encoding/Connection/Proxy-Authorization/Proxy-Connection/Accept-Encoding/Upgrade。按原输入name+": "+value+CRLF的ASCII bytes合计<=65536。按索引检查值/保留头再检查总量，任一不合HttpHeaders。
6 body UTF8<=16777216；GET/HEAD要求空body，否则HttpBody。
7 timeoutMs在1..600000、maxResponseBytes在0..16777216，按此顺序值检查。
8 http能力，否则Disabled；平台；全局token/deadline预检。
URL解析/头校验由http_control纯验证helper作为runtime和后端共用事实源，helper不能触网。后端还防御校验owned request，不能panic。Authorization/Cookie允许调用者显式提交，不隐式凭据/自动cookie。

### 24.2 网络策略和限额
直连、禁自动代理/PAC/WPAD、自动认证、cookie存储/回发、重定向及跨session池；每次调用独立session。302等原响应含Location，不自动访问新目标。无应用隐式重试，不处理RESEND_REQUEST自动重发；不得把OS TCP重传宣传成未发生，不能承诺服务端副作用恰一次。若平台内部不可禁的HTTP请求重放发现，须报告并修订支持承诺，不掩盖。
HTTPS使用系统默认证书链/主机名/有效期验证，无insecure选项/忽略证书标志，无自动客户端证书；TLS失败Io。系统TLS协商策略不自造证书信任库。本地自签拒绝不是可信TLS成功见证，不声明公网上TLS已验。
禁自动解压；响应body计平台read返回的原始content-coded bytes，不含chunk/TLS framing。响应头计最终WinHttpQueryHeaders RAW_HEADERS_CRLF完整API文本，含statusline、所有CRLF及末空行，排除一个API终止UTF16 NUL，严格UTF16解码后UTF8 bytes；不得规范化/重建后再计数。内嵌NUL或非法UTF16为Io协议错误。该口径明确非wire bytes。返header pairs从文本解析，去statusline/空行、lowercase名称、trim值SP/HTAB，不改变计数文本；API无法无损表示原header octet不承诺二进制往返。
查询头size含终止NUL最大65536原生bytes，先查size并限制再分配；最终响应头最多256项，超出ResponseLimit。maxResponseBytes合并上述头UTF8bytes+body rawbytes，恰N允许、>N拒绝；0通常连头都不能容纳，HEAD/204仍计头。正文固定8KiB scratch，不按Content-Length/QueryDataAvailable无界预分配；保留头文本+body计数字节<=N，转换/返回pairs为64KiB/256项约束下有界额外分配。头原生缓冲硬限独立于N。多余不可表示/非法头协议不能lossy成功。

### 24.3 错误和真实终止边界
core唯一error.rs新增无字段ValueMsg及message：
HttpDisabled“HTTP 能力未启用”
HttpMethod“HTTP method 不受支持”
HttpUrl“HTTP URL 非法或超出长度上限”
HttpHeaders“HTTP headers 非法或超出上限”
HttpBody“HTTP body 非法或超出上限”
HttpTimeoutRange“HTTP timeoutMs 必须在 1..600000 范围内”
HttpResponseRange“HTTP maxResponseBytes 必须在 0..16777216 范围内”
HttpUnsupported“当前平台不支持受控 HTTP”
HttpTimeout“HTTP 请求时间上限已到”
HttpResponseLimit“HTTP 响应超过字节上限”
HttpUtf8“HTTP 响应 body 不是合法 UTF-8”
以上ValueError可attempt，不新增第14错误类。DNS/connect/TLS/send/receive/read/协议/native错误Io，msg前缀“HTTP {operation}失败：{message}”，调用span Some/真实source沿C。全局Cancelled/Deadline仍ExecutionLimitError不可吞；wait不扣AST step，remaining0不在native额外触发StepBudget。
同轮可观测优先cancel→globaldeadline→localtimeout→大小超限→OS/协议错→普通完成，首确定reason锁存，cleanup不改写。local deadline从backend入口、创建session前建立，覆盖DNS/connect/TLS/send/headers/body至完成检查，不逐read刷新。SetTimeouts只阶段辅助，不能代总截止。目标poll<=10ms非硬实时；截止是协作请求取消时刻，不是函数硬返回时刻。结束正常成功前再同序检查一次，是本native协作poll，不扩其它native/T17计步规则。
取消后单owner close request发起取消，安全等HANDLE_CLOSING及最后callback退出确认，再释放发送/读buffer/context，逆序释放connection/session。close返回仍可能callback；parent close不能替代request cancel；不可detach、提前free、或callback内等自身退出。OS异常cleanup可能延迟返回，无有限绝对回收时界承诺。正常loopback慢头/慢体/TLS拒绝/取消必须watchdog实跑合理收口，失败保留并拒收；不偷偷用同步worker超时detach或unsafe close替代。无Job helper/current_exe假设；如通常DNS/网络无法合理收口，先报告证据再审helper部署方案。

### 24.4 owned ABI、sidecar和owner
evaluator::EvalCapabilities新增pub http:bool（Clone/Debug/Default，false）。明确Rust公开struct literal源兼容迁移，现代码/测试只机械补字段保所有断言、指南记录；不改旧入口函数/EvalLimits/EvalContext字段，三个context入口复用，旧非context入口http=false。单会话跨module共享能力，进程/http互不推导，attempt不恢复权限，不跨运行泄漏。
tooling唯一新http_control.rs/platform、CLI/main/lib注册。pub(crate)且字段pub(crate)、Debug的owned边界：
HttpRequest {method:String,url:String,headers:Vec<(String,String)>,body:Vec<u8>,timeout_ms:u64,max_response_bytes:usize}
HttpControl {deadline:Option<Instant>,cancel:Option<Arc<AtomicBool>>}
HttpResponse {status:u16,headers:Vec<(String,String)>,body:Vec<u8>}
HttpFailure {Cancelled,Deadline,Timeout,ResponseLimit,Unsupported,Io{operation:String,message:String}}
HttpReport {result:Result<HttpResponse,HttpFailure>,diagnostics:Vec<String>}
pub(crate) fn run(request:HttpRequest,control:HttpControl)->HttpReport
共用纯helper pub(crate) fn valid_url(&str)->bool、valid_headers(&[(String,String)])->bool、valid_method(&str)->bool；值约束如24.1，无触网。Control可Clone，后端不接Value/Rc/Env/Invoke。runtime唯一evaluator/builtins注册、LFZ检查/UTF8/错误映射，core唯一错误变体；lib注册只有tooling一处。后端非法owned请求为Io operation="请求校验"，不要为防御引新公开错误类。

新增TracedRun私有http_diagnostics:Vec<String>及pub fn http_diagnostics(&self)->&[String]，Interp独立HTTP列表，attempt不清除/恢复；所有内部literal机械填空。保§23 host_diagnostics子进程列表/文案逐字不变。backend diagnostics先合入再处理result。cleanup失败若主result已有ExecutionLimit保其类/reason/trace；其它无主错则Io；即使Io被attempt捕获也不能通过宿主门禁。
CLI/runner和库集成检查两个列表：result Err时仍只原错误JSON，附诊断先子进程后HTTP、各列表原顺序；result Ok但任一列表非空HostFailure stdout空/stderr/exit2（JSON宿主异常例外），主标题子进程列表非空用旧子进程标题，否则“lfz: HTTP 资源回收失败”。HTTP每条前缀“lfz: HTTP 回收诊断：”。两列表同非空时主标题只一次先子进程，各类诊断按上述顺序，不改旧子进程文本。库result Ok不足以表示完整成功，必须检查两列表。

### 24.5 Windows状态机、CLI和门禁
WinHTTP ASYNC零crate，每request单owner/单pending操作；callback只owned事件不调用LFZ，不panic穿FFI，不将临时栈地址作为context。buffer/context稳定持有直到最终关闭+callback退出，关闭只一次、close后不再调API、锁存终止禁止回调发新请求。bounded completion slot/事件通知，不允许流量造无界队列。每次API调用/取消竞态需安全review，异步不等于所有OS阶段无阻塞。非Windows固定Unsupported，无curl/exec fallback。
CLI --allow-http无值，重复“选项 '--allow-http' 不能重复”，等号形式“选项 '--allow-http' 不接受值”；非执行类别文案“HTTP 能力选项只适用于运行命令”。check/test/--help/-h/--version/-V拒绝，裸help/version仍文件。与process/limits混用按选项区首次类别文案，扫描错误token序优先；CODE/argv保护不变，无新flag旧行为不改。test/check不隐式授权，不让--allow-process授HTTP。
必须本地loopback fixture：方法/body/query/Unicode percent路径、重复头、404/500普通值、302目标未访问、POST计数无应用重发、慢headers/慢body/持续chunk总timeout、精确API头文本+body N/N+1、HEAD/204/空body、超头/非法UTF8/断连/长度错误；禁用/类型/URL先检无连接，两能力四组合/跨module/session；全局token/deadline不可吞/本地可attempt、callback取消完成race/关闭失败诊断门禁；重复运行无可测泄漏，旧exec文案不变。
服务器仅loopback临时port、有界watchdog且全部shutdown回收，所有临时/fixture项目内，不公网/项目外安装/信任库修改。HTTPS本地自签拒绝明确未验可信TLS成功，不能称完整公网HTTPS验证。最终hash旧T18-A/T17/C/apps/CLI及独立验收、原19项性能另验，R304仍OPEN；本轮后停止。
