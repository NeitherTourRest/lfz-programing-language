# T18-B运行时接口评审（只读提案）
2026-10-30，runtime-dev。依据当前§23及已交付源码；本文件不是冻结规范或实现。未改源码/编译，无网络请求。范围仅本轮HTTP，不恢复自动化，不进入T19。

## 现有接口证据与兼容结论
- evaluator.rs:223 EvalCapabilities只有公开subprocess bool，225 EvalContext公开limits/capabilities。cli.rs:213存在完整EvalCapabilities{subprocess:...} literal。直接增加http bool会使旧Rust literal编译失败；新增私有字段或non_exhaustive同样破坏已有外部构造，builder单独存在不能替旧shape保存一个新bit。
- 三_and_context入口现在消费EvalContext，旧limits入口用Default capabilities。模块权限是单Interp快照，不绑函数home，不入attempt恢复。
- evaluator.rs:123 host_diagnostics返回私有Vec<String>的slice；consume_process_report在匹配业务结果前合入诊断。attempt仅恢复trace/source/module/globals/depth，Value/Io可恢复、ExecutionLimit不可恢复。
- cli.rs:760/773及test_runner.rs:114诊断标题固定“子进程”。HTTP若直接塞同一sidecar，失败会被误标子进程；这是必须在契约中处理的跨owner影响，不能静默改旧文案和旧断言。
- StructObj.raw_fields保留插入顺序、含方法；data_fields_sorted过滤函数。HTTP请求校验应检查raw_fields，不能用过滤后的keys/len让函数字段逃过unknown/type校验。

推荐保留旧EvalLimits/EvalCapabilities/EvalContext shape和三个旧context函数的签名。新增明确命名的HttpEvalContext { evaluation: EvalContext, http: bool }（最终名称由architect冻结），三组_and_http_context新owned入口；旧所有入口降到同一私有会话构造且http=false，新入口显式授权。独立HTTP开关不推导subprocess，反向亦然。以后扩展版context可设计私有字段+builder，但不得追称旧公开literal兼容。若团队选择直接新增http字段，应明确列Rust源兼容迁移、机械适配全部构造并写ADR，不能声称保旧shape。

## 最小LFZ请求与结果建议
建议http(request,timeoutMs,maxResponseBytes)，特殊派发与exec一致，允许同名绑定遮蔽，无first-class builtin；无上下文builtins call固定禁用。
request采用固定四字段普通struct：method:string,url:string,headers:array,body:string；首版全部必填，headers=[]/body=""表达无头无正文，拒未知/重复字段，避免默认值分叉。headers是[[name,value],...]，均string且pair恰2；保留重复及顺序，不能用普通struct字典压平。真实后端可能会规范化名称或顺序，只有其可观测序列能承诺，不能声称保原始wire字节。
结果建议固定三键status:int,headers:同pair数组,body:string；HTTP非2xx仍返回值，调用方决定业务失败；传输错误才是Io。不要statusText/parsed JSON/自动抛HTTP状态错误，LFZ jsonParse另行组合。
首版文本请求/响应：请求body按UTF8发送；响应body严格UTF8，非法整体可attempt ValueError，保留BOM/换行；不按Content-Type猜编码、不替换非法字节。明确这不支持任意二进制下载。若必须二进制，应先批准bytes/base64 API和额外分配上限，不能把二进制静默损坏成字符串。
响应头推荐明确受限文本策略：名字ASCII token；值只能后端无损表示的文本，遇不可表示字节受控失败，不能lossy UTF8转换。若Windows后端只给宽字符字段，契约需说明该返回值是平台解码视图，不能保证任意原始header octet往返。重复头（尤其不能合并的头）必须使用逐项枚举；后端若拿不到重复项应拒收或缩小支持而非静默合并。

## 校验与限制需要冻结的次序
先旧规则左到右求实参，再数量3、request struct、四字段存在/额外/重复检查、四字段及所有嵌套pair类型、值约束、两个整数类型、范围、独立能力、平台支持、最后native全局预检。具体缺字段/未知字段优先顺序及method/url/header/body限制需architect逐项定死，按固定字段顺序校验，不能随HashMap/用户字段插入顺序抖动。
建议方法ASCII token并禁止CR/LF/NUL；url只绝对http/https，拒userinfo/fragment/控制字符，URL解析由后端共享纯验证器统一事实源，不自行拼接HTTP文本。头名字与值拒CR/LF/NUL，明确Host/Content-Length/Transfer-Encoding等由后端拥有的保留头禁止用户覆盖。TLS证书验证默认开启，不加忽略验证选项。首版禁重定向、自动重试、cookie会话/凭证、隐式代理，防止一次API调用产生未声明请求链；实际可行性由tooling审定，不能先承诺平台未实现控制。
大小上限必须拆清：body的观测字节N/N+1，加单独固定URL长度/请求头总量/响应头总量/条目数/请求body上限；否则超长头在body cap前就可无界分配。建议复用16MiB body上界、64KiB头文本上界、256条响应头作为待批准初值，不称现有语言保证。后端须在分配和UTF8/UTF16转换前执行上界保护；精确定义计数是平台提供文本编码长度、传输解码后的body长度还是wire字节，不可混称。
禁自动解压可减少字节口径歧义；如支持解压则必须限解压后的body并防超限分配，不按Content-Length信任预分配。chunked/长度缺失/零body/HEAD/204及尾部超量应真实本地server验收。

## 终止与回收
native同一轮优先cancel→全局deadline→本地timeout→大小超限→OS/协议错误→成功；确定首reason后回收不覆盖。局部timeout/超量/UTF8 ValueError可attempt；DNS/connect/TLS/send/read失败Io带调用span；全局ExecutionLimit Cancelled/Deadline原样穿透attempt并记真实module source。native等待不扣AST steps，remaining=0不能在native额外报StepBudget。
局部计时覆盖DNS/connect/TLS/send/headers/body/清理的哪段必须明确，推荐从backend调用开始；同步系统调用阻塞不是硬实时，需要平台可取消句柄/回调取消机制与外层有界测试。不能线程超时返回但遗留阻塞worker，也不能拿exec的进程kill能力冒充HTTP资源回收。
HTTP handles/worker cleanup diagnostics必须先合并再返回，attempt不恢复；全局终止+cleanup失败仍保原ExecutionLimit。为保旧文案，可在私有diagnostic sidecar增加来源类别并保旧host_diagnostics slice accessor，给CLI附加分类accessor；也可HTTP采用单独私有sidecar/accessor与独立HTTP门禁。推荐后者最少碰旧§23，两个列表均阻断宿主成功，Err仍仅原JSON错误对象。不能只保留成功result而忽略cleanup诊断。

## owned边界与建议验收
推荐后端HttpRequest{method,url,headers:Vec<(String,String)>,body:Vec<u8>,timeout_ms,max_response_bytes}、HttpControl{deadline,cancel}、HttpOutput{status,headers,body:Vec<u8>}、HttpReport{result,diagnostics}。不传Value/Rc/Env，runtime负责冻结LFZ次序/UTF8/错误映射，后端再防御校验可信输入。所有正式名字/失败variants由architect与tooling统一，尚未冻结不得实现。
测试应含：默认拒绝无连接、两能力四组合与跨session/module；全部错误优先无side effect；重复header及无损值策略；请求/响应Unicode与非法body、0/N/N+1和头超量；404/500成功值；局部timeout可捕获/全局取消不可捕获/同时事件锁存；同源函数真实trace；HTTP cleanup注入后attempt不能骗宿主成功；旧exec诊断文案逐字兼容。只用项目内本地server，无公网，不声称Windows见证覆盖Linux。

本次只读review完成；待architect冻结/root GO后才能改源。R304仍OPEN，本轮无性能声明。
