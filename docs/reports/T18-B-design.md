# T18-B HTTP 请求提案

日期2026-10-30，language-architect。提案未冻结，源不GO。基线由root提供main740fd32/T18A4093714。用户仅授权本轮HTTP实现验收后收口，不自动T19、不恢复自动化。D116MiB，不编译/安装/公网功能测试。HTTP能力与进程能力独立，不用exec调用curl/powershell绕过网络能力。

## 实码与推荐API
实际EvalCapabilities仅subprocess:bool，EvalContext持limits+capabilities；T18-A已有三context入口、TracedRun私有host_diagnostics/read accessor及HostFailure门禁，process backend owned report。沿用这些入口/门禁，不再扩入口族；新增http:bool，Default false，所有本仓库及外部Rust struct literal需要机械补http:false（或..Default），必须作为兼容迁移公开记录。旧EvalLimits/UserFn/StructDef/Invoke不改shape。

推荐LFZ唯一新特殊内置：
    httpRequest(method, url, headers, body, timeoutMs, maxResponseBytes)
共六必填实参；method/url/body为string，headers为array，每项恰两string的array。无body用""，无头用[]。timeoutMs int 1..600000；maxResponseBytes int 0..16777216。不用opts struct未知键/隐含默认制造歧义，后续若需要二进制/流式另定。本批内置74→75，非first-class，按既有特殊内置及同名遮蔽规则。
返回普通struct三键固定顺序：
    {status:int, headers:array, body:string}
headers=[[lowercaseName,value],...]，保重复（Set-Cookie不逗号合并），无状态行/空终结行，保持API观测字段顺序，不承诺原服务端wire拼写/大小写/全序。status 100..599的最终响应（1xx中间通知不当返回）；非2xx/3xx/4xx/5xx均普通值，不抛异常。body严格UTF8，空body可成功；不按Content-Type/charset转码、不自动JSON解析、不lossy替换。响应头值由WinHTTP查询文本转Rust string，不声称二进制头透明。

## 校验与请求语义
实参先旧LFZ左到右求值；完整校验后才建网络句柄：数量→六项外层类型→headers逐pair长度/元素类型→值约束→能力→平台→全局资源预检。坏参数不能先发请求才失败。
method仅ASCII大写GET/HEAD/POST/PUT/PATCH/DELETE/OPTIONS白名单，禁止CONNECT/TRACE及任意扩展方法。GET/HEAD要求body空，避免不同后端行为；其它方法发送body的UTF8原始字节，最多16MiB。请求头总编码上限64KiB、最多256对，大小包含name、": "、value和CRLF；不声称该上限覆盖WinHTTP生成头。
URL只绝对http://或https://，ASCII主机（国际域名使用调用者显式punycode），显式port1..65535可选；允许标准括号IPv6；缺path按/，query保留。不含userinfo/fragment/NUL/ASCII控制/空白/反斜线；不自动转义Unicode路径，非ASCII路径调用者百分号编码，百分号必须紧随两hex。非法URL先ValueError，不依赖WinHTTP宽松修复。不放宽TLS/代理设置、不支持unix socket/file/data协议，不把独立http能力称SSRF防护或OS沙箱。
header名非空ASCII token（HTTP token字符集），value仅可打印ASCII及HTAB，禁止CR/LF/NUL，去除值两端SP/HTAB后发出。调用者禁设Host、Content-Length、Transfer-Encoding、Connection、Proxy-Authorization、Proxy-Connection、Accept-Encoding、Upgrade（大小写不敏感），由后端控制authority/framing/无压缩。重复其它头保序提交，不静默覆盖；Authorization/Cookie允许显式值，但无默认凭据/cookie。
不自动重定向：302等原样返回含Location，调用者显式发新请求。无应用重试、无基于状态码重试、无ERROR_WINHTTP_RESEND_REQUEST手动重发；无法保证TCP分段重传不发生，也不宣称服务端失败前没收到body。WinHTTP若存在不可关闭的请求重放行为必须报告，不能声称已达到exactly-once。每调用独立session，禁自动cookie/认证/代理/PAC/WPAD和跨session连接池；直连系统DNS，避免代理发现阻塞与凭据泄漏。TLS系统证书链/主机名/有效期验证保持，不提供insecure flag，不自动客户端证书认证；协商失败IOError，不能为了本地HTTPS测试跳验证。

## 限额精确定义待root裁决
推荐maxResponseBytes对两部分合并：WinHTTP API可观测响应头文本按UTF8编码的字节数（含状态行/CRLF，以一次原始头查询文本为准）+响应body读取的原始bytes。不以body解码后字符数计，不使用Content-Length当已读真实量。WinHTTP raw header查询是UTF16文本，不是TCP/TLS wire原始bytes；禁止声称此上限等于在线报文原始总量。headers UTF16查询缓冲硬限64KiB，超出即局部超量；请求body另硬限16MiB。实现者必须冻结并测试同一文本序列化口径，不能把UTF16 length乘2当UTF8。
若root坚持wire精确header原始字节，则WinHTTP该抽象不够，须改HTTP解析/后端方案，不可假装已满足。body不启用自动解压，返回实际content-coded bytes，gzip往往非UTF8而报错；用户可不发压缩协商，透明二进制另批。
响应合并恰N允许、N+1拒绝，缓存body与头合计<=N（输出头表示构造另有有界开销）；单scratch8KiB。body不能一次按Content-Length分配，不用WinHttpQueryDataAvailable尺寸无界分配，固定块read。chunk framing/TLS开销不计；HEAD有头无body仍计头，因此max=0正常HTTP响应一般会超量，这是明确语义不是无输出成功保证。

## 错误、deadline、清理与门禁
不新增第14错误类：参数/禁用/Unsupported/局部timeout/响应超量/UTF8为ValueError，可attempt；DNS/连接/协议/TLS/OS错误为IOError，可attempt。全局T17 Cancelled/Deadline仍ExecutionLimit不可吞、固定reason/message/trace，不因网络本地错误替换。等待只poll全局token/deadline，不扣AST步、不因remaining0自行StepBudget。
同轮优先cancel→globaldeadline→localtimeout→超量→OS/协议错误→正常完成，首个确定reason锁存，收尾不篡改。局部deadline从开始创建网络session之前建立，覆盖句柄建立/DNS/connect/TLS/send/receive/read到完成；不是每次read刷新，不以SetTimeouts阶段值代替总时限。每轮poll目标<=10ms但不承诺OS硬实时，回收在内核异常下不宣称绝对时界。全局token可从host置位，http和process共用单session资源策略。
无论成功/UTF8失败/参数之外任何网络失败，最终释放request→connection→session，不能留下detached worker/回调悬空/无限channel。主请求先终止再释放其引用的发送/读取buffer；DNS/TLS待定状态必须能发起取消、保持回调context活到终结。SetTimeouts仅辅助各阶段，不能拿同步worker+watchdog close跨线程当安全总deadline。
cleanup失败沿T18-A diagnostics：runtime合并到Interp，不被attempt恢复；result Err资源错误保持原JSON+stderr附加诊断；result Ok但diagnostics非空沿HostFailure stdout空/stderr/exit2，即使--json也为宿主异常例外，不伪LFZ业务错误。库调用者必须同时检查diagnostics；未证明回收的路径不算验收通过。

## Windows后端可靠性方案（待实施安全review）
使用std+WinHTTP原生FFI零crate，Windows先验，其它平台明确Unsupported。WinHttpOpen ASYNC，单owner驱动request状态机，callback只报告完成/错误/最终关闭事件，不把Value/Rc/Env/Interp传给OS线程，不让panic穿FFI边界。
每个request一次只发一个异步操作，buffer/callback context通过owned Arc或严格等价生命周期稳定持有；禁止把栈地址临时传dwContext。close request发起pending操作取消，parent session close不能代替request cancel。CloseHandle返回后仍可能到callback，必须等最终handle-closing/已确认完成状态再释放相关buffer/context；同一handle只close一次、不close后重用。无“超时后返回并遗留后台Rust线程”。
原生API调用本身可能同步做少量工作，必须审具体阶段；同步解析/代理/证书辅助工作若无法受控取消，须保留限制并按真实证据定交付能力。不能宣称ASYNC自动使所有DNS/TLS/cleanup绝对无阻塞。回调异步完成与取消同时发生，owner必须锁存reason并排除close/API调用竞态；不在另一个正阻塞WinHTTP调用时强close。
无回调引用活对象后的detached状态，bounded内部事件队列（建议一个pending completion slot+有限通知），不能让恶意慢头或小chunk放大无界事件积累。FFI常量/结构/签名对照系统SDK及Microsoft文档，独立安全review。Linux/macOS本批不实现请求，不用外部curl fallback绕过能力。

## Rust边界与owner建议
不扩EvalLimits/已有三context函数签名；EvalCapabilities新增http公开字段记录Rust构造API变更。新src/http_control.rs与platform私有实现由tooling唯一写，evaluator/builtins runtime唯一，error.rs新增Http系列ValueMsg由core唯一；tooling负责CLI/main/lib注册，串行避免builtins/lib冲突。
建议pub(crate)纯owned请求HttpRequest{method:String,url:String,headers:Vec<(String,String)>,body:Vec<u8>,timeout_ms:u64,max_response_bytes:usize}；HttpControl{deadline:Option<Instant>,cancel:Option<Arc<AtomicBool>>}；HttpResponse{status:u16,headers:Vec<(String,String)>,body:Vec<u8>}；HttpFailure{Cancelled,Deadline,Timeout,ResponseLimit,Unsupported,Io{operation:String,message:String}}；HttpReport{result:Result<HttpResponse,HttpFailure>,diagnostics:Vec<String>}。run(request,control)->report。runtime做LFZ/UTF8转换，后端复核不可信owned请求不panic；精确类型/值错误及消息待批准再冻§24。

CLI独立--allow-http无值，默认false，重复/等号拒绝；非执行命令拒绝。T17 limits/process/http混用按选项区首类别（限额/进程能力/HTTP能力）选择对应适用性文案，完整扫描错误优先，CODE/argv保护不变。test/check不默认开放网络，原进程flag也不开放http；权限在单次context跨module共享，不跨下一运行。

## 实测门禁
本地loopback服务器只绑定127.0.0.1/::1临时port，独立fixture支持方法/path/query/headers/body回显、重复响应头、状态码、302目标未访问计数、慢头/慢body/无限chunk、提前断连/错误长度/头过大/非UTF8；所有测试必须有有界watchdog与server停止回收，文件仅项目Temp。不依赖公网、DNS成功/公网证书信任或项目外安装。
必测禁用无网络触碰/类型先检、process/http互不授权、跨module/token/deadline/attempt、资源顺序与诊断门禁；限额头+body精确N/N+1及chunk body；本地POST计数不隐式应用重试/重定向；慢头/持续小chunk仍总timeout；取消时callback与buffer释放race可故障注入，重复多次无残线程/句柄增长。
HTTPS若无受信任loopback证书，最小验收仅证书拒绝/HTTPS代码路径故障，明确“未验证可信TLS成功”；不得导入新根证书或禁验证。如root要求HTTPS成功见证，另选已有可信本地证书资源/限定临时server证书并用户已有信任，不把自签错误测试当成功。
全部旧T18-A/T17/C/apps/CLI保留回归；最终hash功能与原19项性能另验。仅本轮T18-B后收口，不T19/自动化。

## 一手事实来源
- https://learn.microsoft.com/en-us/windows/win32/winhttp/concurrency-in-winhttp ：close取消、callback、并发调用安全边界。
- https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsendrequest ：取消时发送buffer生命周期。
- https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsettimeouts ：阶段timeout，非本提案总deadline实现证明。
- https://learn.microsoft.com/en-us/windows/win32/winhttp/option-flags ：redirect/cookie/auth/pooling等显式禁用。

## review收敛补充（root已批准API/限额方向，仍未冻结）
响应头计数唯一口径：使用WinHttpQueryHeaders RAW_HEADERS_CRLF查询最终响应头的完整API文本，保留其状态行、每行CRLF及末空行CRLF，剔除API返回缓冲终止NUL；UTF16严格解码为Rust string再取as_bytes().len()，不另增/删CRLF、不由返回pairs重建。若API文本含多余终止NUL/不合法UTF16则协议IOError，不静默截断。头64KiB硬缓冲按API size含终止NUL字节检查，再分配（不得先无界取回再判）；查询得到的实际文本为独立oracle，不能直接拿本地server发送wire字节数当相同口径。返回header pairs另从此API文本解析，排除statusline及空终结行，lowercase名称、值去两端SP/HTAB、保重复，计数仍使用处理前API文本。
最终头文本UTF8字节+原始body读取字节合并>N即ResponseLimit；HEAD/204/空body也计头。backend report建议额外pub(crate)响应头计数字段只用于unit oracle，若无产品需要则仅测试adapter返回观测数，不向LFZ三字段增字段。

异步取消真实保证：局部/全局deadline是请求协作截止，不是函数硬返回时刻。单owner锁存reason后close request；close返回后仍等待HANDLE_CLOSING及callback执行退出，保持context/buffer/同步对象存活，再关闭connection/session并释放。不承诺微软未给出的closing有限时间上界，不使用detached Rust线程/回调context泄漏换快速返回，也不在cleanup延迟时提前free。需要标记closing阶段不继续发read/send，不让完成回调重启请求；不可在callback内部等自身退出，终结同步由owner负责。
OS不响应回收可能延迟返回，这是沿T18-A的异常内核回收边界。正常loopback各阶段的取消/关闭必须在宽容watchdog内实测：慢头/慢体/持续chunk/TLS拒绝/断连，且重复运行无可测句柄/线程增长。watchdog失败保留证据并拒收/修复，不能改契约为unsafe提前返回。
不先引helper子进程：目前库宿主没有独立lfz-helper定位/版本/部署协议，current_exe可能是嵌入LFZ的任意程序，不能冒充helper路径。如果实际DNS/网络操作通常无法合理取消，报告具体证据后再独立设计Job隔离helper及host显式路径，不偷扩大本轮范围。

根裁决已确认：六参/三字段、默认http false、capabilities新增字段机械迁移、非wire计数、零crate async Windows、系统TLS不insecure与自签拒绝见证。尚待tool/runtime lifecycle review合定，再冻结§24；源仍不GO。
