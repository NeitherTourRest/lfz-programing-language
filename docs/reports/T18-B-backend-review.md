# T18-B backend 可行性 review（tooling-dev，2026-10-30）

本报告只读源码及Microsoft官方文档，未改产品、未编译、未发网络请求。基线main740fd32、T18-A候选4093714；用户仅授权本轮HTTP交付后收口。以下是待architect裁决的建议，不是已冻结语义或已验收能力。

## 推荐与否决项

Windows std+原生WinHTTP FFI零crate技术上可行，推荐异步单owner状态机作为最小原型路线，但先过生命周期闸门再GO产品。同步WinHTTP加四个SetTimeouts不能兑现总timeout、活动取消与可靠回收；不接受“另线程同步请求，超时关闭/忘记线程”方案。若契约要求取消后固定时间内完全没有回调/线程残留，进程内WinHTTP不能由文档证明；需要另行批准Job隔离可信helper，或者明确正常OS条件下协作取消、回收异常宿主诊断的承诺边界。

SetTimeouts覆盖解析、连接、发送、接收阶段；零/-1是无限，原默认解析无限，设置时须非零正数。多阶段、持续小块数据和多地址尝试不能直接合成单个端到端预算。这是根据API分阶段定义作出的工程推论。[SetTimeouts](https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsettimeouts)

官方并发规则限制同步请求仍在调用期间关闭句柄；异步pending操作则可关闭request请求取消，但关闭后操作仍可能成功。不能靠关闭session替代关闭pending request，也不能复用已关闭handle数值。[并发规则](https://learn.microsoft.com/en-us/windows/win32/winhttp/concurrency-in-winhttp)

## 异步所有权与总预算方案

建议每请求单独session/connect/request，WINHTTP_FLAG_ASYNC。唯一调用线程驱动Send完成→Receive完成→Headers→固定缓冲Read完成→EOF；每handle同一时间只一个pending操作。回调只把有限状态/错误/长度复制到预分配槽并通知条件变量，不调用LFZ、不持锁调用WinHTTP、不在回调中推进下一次Read，防同步完成递归、死锁和无限消息队列。FFI回调不得展开panic。

堆稳定context持有请求/读取缓冲、同步槽、closing标记；注册非零context及HANDLES通知。主线程各调用前后和短周期等待中检查cancel→T17deadline→本地deadline→大小→OS错误→完成，锁定原因后不由后续成功或回收错误覆盖。本地deadline在后端开始checked_add，覆盖DNS/TCP/TLS/发送/头/body；实际预算取与T17deadline较早者。SetTimeouts作为第二层限时，不是总预算证明。

取消时唯一owner在当前API调用返回后显式Close request一次；等待HANDLE_CLOSING再释放context/缓冲，之后逆序关闭connect/session并收集失败诊断。Close返回后仍可能回调；取消注册回调也不保证与已在运行回调同步，不能据此提前free。官方没有给HANDLE_CLOSING最大到达延迟，这正是有限回收承诺的缺口。[Close生命周期](https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpclosehandle)

正常EOF也须相同关闭握手，不能只timeout路径严谨。回收失败走T18-A host_diagnostics同类门禁；但“记录诊断然后释放仍在回调的内存”依旧非法，“泄漏context以安全返回”也不能冒充无残留。没有实测及架构认可的异常回收策略前，不应冻结进程内强时界承诺。

若改可信helper：同步WinHTTP可封在本项目helper进程，父进程用已有Job挂起纳管/公平有界pipe/终止wait路径约束总预算和后代，HTTP能力与process能力独立，用户不获得任意exec。IPC必须固定版本/长度、有界响应、错误sidecar、不可混stdout业务输出。额外风险是库调用方current_exe可能是测试或嵌入宿主，不能假设有LFZ worker入口；helper分发、校验、进程创建权限和构建空间均需明确方案。本轮不自行扩展隐藏CLI/helper部署。

## 协议及内存边界建议

- 首批仅http/https、GET或契约明定方法；拒URL凭据/NUL/CRLF，明确fragment、端口、IDN/IPv6和百分号编码。URL解析尽量WinHttpCrackUrl，不能手工按冒号拆IPv6。请求header拒注入；请求body也必须有独立输入限额。
- NO_PROXY显式直连，不读环境代理或自动发现/PAC，避免隐藏配置与额外请求；不修改系统配置。[WinHttpOpen](https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopen)
- 明确禁止自动redirect、自动认证/cookie、应用级重试；3xx/4xx/5xx返回状态值。设置选项失败应受控失败。connect多地址尝试、失败连接重发和应用重试不同，不能靠CONNECT_RETRIES一个选项宣称所有自动重试已关闭。选项可用性/准确数值须实施时查SDK及本机支持，不能吞ERROR_INVALID_OPTION。[选项与retry说明](https://learn.microsoft.com/en-us/windows/win32/winhttp/option-flags)
- 用系统TLS验证，绝不设置忽略证书错误，证书错误受控失败；默认并不等于启用吊销检查，若启用须考虑CRL/OCSP可能额外联网。本地仅HTTP fixture不能证明TLS链/主机名验证已实测。[TLS安全说明](https://learn.microsoft.com/en-us/windows/win32/winhttp/winhttp-security-considerations)
- body固定8KiB缓冲累积，读到超额即失败，无部分成功；Content-Length仅提前拒绝，不能信它替代实际累计。WinHTTP去chunk framing，默认不主动启用解压；建议契约定义返回body字节上限，不称原始wire字节限制。缓冲在异步Read完成前不得移动/释放；异步字节数出参必须NULL。[ReadData](https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreaddata)
- 单独设header上限（建议64KiB）与条数，WinHTTP内部header cap加查询分配前检查；QueryHeaders长度是字节而非u16个数，严格处理终止符、重复header和状态行，不用map静默覆盖重复值。body严格UTF8可沿runtime解析，保BOM/CRLF、不做有损替代；header是WinHTTP宽字符语义，不能称原始头UTF8字节保真。[QueryHeaders](https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpqueryheaders)

## Context/CLI待定案

现EvalCapabilities{subprocess:bool}/EvalContext及旧API默认关闭已读。直接加http字段会使下游Rust struct literal源码不兼容；保留旧API运行行为不等于源码shape兼容。请architect明确批准字段扩展或新增owned context/API，不由tooling私改。HTTP能力默认false，旧API/runner/check禁用；仅本次execute的--allow-http授权，不能因--allow-process启用HTTP，反向亦同。新flag无值/重复/等号/非执行适用性沿既有scanner，CODE和--后argv保护；多能力+限额首类别文案需冻结。

## 无公网原型闸门及验收矩阵

本地Rust std TcpListener绑定127.0.0.1:0，显式端口、项目Temp内fixture，外watchdog有界收工；不安装、不公网、不改证书库/防火墙/代理。fixture本身监听/accept/read也需非阻塞与停止token/有限wait，不能测试结束遗留线程。

1. 请求到达marker后再取消，证明真实活动取消；分别卡响应头、body中段，验证T17deadline不可attempt吞和本地timeout可恢复。缓慢每块小于receive timeout但总时间超预算，抓分阶段伪总timeout。
2. 正常200/204/404/500、重复头、chunked、Content-Length缺失/不符、早断连接、BOM/CRLF、非法UTF8、零body及exactN/N+1，header边界与大头；内存保留不超预算。
3. 302 Location指向另一个loopback计数listener，确认零次第二请求；401不自动凭据；fixture关闭/重置连接记录请求次数，审重试。仅真实覆盖条件可称已禁，TLS未本地覆盖须明确列缺口。
4. 高频完成/取消竞态，记录request创建/close/closing/context释放计数恰一次；close/登记/分配失败注入仍保持OS可靠回收，无UAF、泄漏或重复close。主reason优先级与diag门禁合成/真实路径同时测。
5. DNS迟延与TLS握手迟延不应通过外部域名或机器配置制造。loopback仅覆盖connect后阶段，DNS/TLS需可控注入/独立TLS fixture能力或标未实测，不能拿HTTP慢body成绩替代。

下一步建议：architect先裁决强时界/cleanup异常及context兼容，再授权最小生命周期原型。当前结论是条件可行，尚无编译/实跑证据；本报告不是产品完成声明。

## 给architect的最小可交付推荐（root边界确认）

本轮首选进程内异步WinHTTP，不以官方未承诺硬实时为否决。沿T18-A承诺：deadline约束请求推进直到发起取消；正常OS条件下Close request并安全等待HANDLE_CLOSING后返回，绝不detach或释放仍活callback，异常OS回收不保证绝对墙钟时界。正常本地各阶段取消/关闭/回调释放由外watchdog实证；watchdog触发是失败证据，不能算产品按时成功。

最小实现为单owner请求状态机、预分配8KiB读缓冲、有限回调槽、cancel/global/local主因锁定、独立diagnostics门禁；不额外建设helper部署。可信helper仅作为将来如果要求更强隔离的备选，当前库宿主current_exe非lfz的问题尚无部署解法，不能跨原生权限或自行安装。建议GO前冻结此承诺边界与HTTP context ABI，然后授权本地生命周期闸门；并非当前有技术阻塞必须新增进程架构。
