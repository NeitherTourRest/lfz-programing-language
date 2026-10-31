# T18-B 公开黑盒验收准备

2026-10-31，test-engineer。当前仅测试规划，无脚本、无网络运行、无新功能验收。启动已读全局/看板/自身记忆、NEXT、§23、runtime/backend review。architect已冻结三spec §24与 `T18-B-contract.md`；`T18-B-design.md`及review中的三参request struct建议是历史资料，测试以冻结六参 `httpRequest(method,url,headers,body,timeoutMs,maxResponseBytes)` 为准。

范围只本轮HTTP，完成后收口，不进入T19/T20、不恢复自动化、不改产品/旧测试、不编译。T18-A公开45与全部历史证据保持。公开HTTP用例与旧官方178、Rust/独立计数分别报告；R304 OPEN。

## 服务器和证据约束

后续脚本拟 `tests/t18b_cli_http.py`，fixture只绑定 `127.0.0.1:0`，动态端口，不访问公网；采用项目内Python标准库原始socket服务器，避免HTTP框架自动添加Date/Server或合并重复头。每场景独立有界listener与停止事件，accept/read设置短timeout，固定长度读取请求且请求上限受控。shutdown、close及join全部在finally，join仍超时为失败，不以daemon线程替代收工。服务器记录原始请求、端点、方法/body/header、单调时间和完整请求次数，确认是否到达后才发起活动取消类见证。

每次产品进程独立stdout/stderr/exit/外watchdog，所有LFZ源码、socket日志、必要TLS素材及证据只在项目Temp。服务器先ready再调用客户端；记录请求计数的负例在调用结束后保留宽容观测窗口并停止listener，不以尚未accept误称无请求。任务结束核端口关闭和worker退出。watchdog触发视为产品/环境失败保全，不能算协作超时通过；不伪造绝对墙钟回收保证。

最终产品SHA必须由root冻结；每进程及末尾核SHA。首次完整结果与所有错误oracle原脚本保全，修新测试先解释规范依据。旧冻结4093714仅可做缺httpRequest/flag红证据，不作新功能通过，后续再唤醒写脚本。

## 公开矩阵与oracle

| 类别 | 场景和真实oracle |
| -- | -- |
| 输入校验无请求 | arity、六参外层类型、pair数组/长度/元素类型；小写或未知方法；URL scheme/凭据/fragment/非法percent/端口/NUL/ASCII限制；保留头、CRLF、name token、空值、总数/总量；GET/HEAD body、两整数范围。记录server零到达，固定校验优先级不被禁用/限额抢先覆盖。NUL源码走文件，避免Windows argv宿主先拒绝 |
| 成功请求保真 | GET query、percent Unicode路径；POST/PUT/PATCH body UTF8字节、空body、header pair重复与trim。服务器回显明确字节值；响应status/headers/body三键顺序，重复lowercase header pair保留且不压平，body不自动jsonParse |
| 状态与编码 | 200/204/HEAD、404/500均成功值；BOM/CRLF精确保留、空body、非法UTF8为可attempt的HttpUtf8；chunked去framing、缺长度EOF，错误长度/早断连接受控Io。Content-Encoding场景不自动解压，非法编码字节不得lossy成功 |
| 禁redirect/重试/cookie/auth | 302 Location指向第二个loopback计数端点，第二端点0请求；POST请求计数固定1，早断/传输错误后无应用重试；Set-Cookie后新session第二请求不隐式携带Cookie；401计数/请求头无自动凭据。只声明实际覆盖条件，不声称TCP重传不存在或副作用恰一次 |
| 响应大小 | raw API完整头文本UTF8长度H加body字节B，N=H+B成功、N-1失败、0通常连头失败；HEAD/204仍计头；末块超量、chunked超量、头原生64KiB和256项限制。阈值不能从返回pairs重建，也不能把wire字节直接当API文本 |
| 局部总timeout | 不回响应头、头完成后暂停body、连续滴流每块低于阶段timeout但总时长超预算；用attempt收到固定HttpTimeout，之后简单表达式成功；所有服务器任务在timeout后正常停止。持续滴流覆盖不能逐read刷新本地截止 |
| 全局终止 | 慢头/慢body请求到达后T17deadline先于较长局部timeout；嵌套attempt不能捕获，ExecutionLimitError/Deadline和真实位置保持。CLI没有cancel token；真实活动token取消由runtime/tooling/独立库harness完成，不伪称公开CLI已测Cancelled |
| 来源与权限 | module导出函数发请求并失败时错误文件属于定义模块；默认无HTTP、仅process也无HTTP、仅HTTP不能exec、两项显式授权各自可用；两次独立进程不泄漏权限。库同进程context/模块权限及home不携权限由unit补足 |
| strictCLI | --allow-http重复/等号含空值/未知形式；check/test/--help/-h/--version/-V拒能力；裸help/version按文件；CODE恰flag文本保护（按真实LFZ文法判断，不一律SyntaxError），--后argv原样；与限额/process组合first-category文案及扫描错误token序 |
| HTTPS边界 | 如tooling能提供项目内短寿命自签TLS fixture，在不改信任库条件下握手受控Io、worker回收，无HTTP业务请求到达；仅证明拒绝，不声称可信TLS或公网HTTPS成功。不可生成/安装外部信任、不能访问CRL/OCSP公网当oracle |

## 精确计数与白盒分界

exact N需先由tooling提供可审独立见证：固定响应对应WinHttpQueryHeaders RAW_HEADERS_CRLF实际文本（含statusline/所有CRLF/末空行、不含UTF16 NUL），严格解码后UTF8 bytes。服务器发送的wire bytes和返回pairs仅辅助比对；不能挑出成功阈值后反推“正确H”，也不能以不断加N追绿替代规范oracle。若平台API规范化不同，保留真实观察，架构确认口径后固定输入和阈值。

callback create/close/HANDLE_CLOSING/context free恰一次、取消完成竞态、异常Close/诊断双sidecar门禁、临时buffer生命周期与bounded slot是tooling白盒/独立review责任；公开正常退出和无可见worker残留不能证明无UAF/内部泄漏。若可控故障注入fixture非产品公开接口，不伪称黑盒。global取消+清理失败仍保ExecutionLimit、HTTP与旧process诊断优先序需库/CLI合成路径另计。

公网DNS延迟、TLS可信成功、多地址重连不通过改系统代理/信任/网络制造。loopback慢body不能冒称DNS/TLS取消已实测；未覆盖项明确列出。反复完成/取消的可见资源数量只报告样本及噪声，不凭一个总耗时称无泄漏或性能改善。

## 下一步

等root唤醒、tooling公开本地HTTP fixture/实际header口径与最终hash，再写新脚本、旧版红测、最终许可真实进程、应用指南与独立验收。当前只规划，无测试通过数字；没有新脚本/产品修改、编译、Git或网络请求。
