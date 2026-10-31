# 显式授权HTTP请求

本章按三spec §24/T18-B-contract，描述当前HTTP候选。最终hash与实跑见[T18-B指南证据](../reports/T18-B-guide-evidence.md)；准备阶段不能拿T18-A旧产物冒称HTTP通过。本轮HTTP交付后收口，不自动进入T19或恢复自动化。

## 先检查，再授权请求

保存为fetch.lfz：

```lfz
#42
let response = httpRequest("GET", argv()[0], [], "", 1000, 16384)
assert(response.status == 200)
let record = jsonParse(response.body)
print(record.message)
```

先启动自己的本地loopback服务，提供GET /data返回UTF8 JSON对象message=雪；端口以服务实际输出为准。不要把下面PORT当固定可用端口：

```powershell
./target/release/lfz.exe check fetch.lfz
./target/release/lfz.exe --allow-http run fetch.lfz -- 'http://127.0.0.1:PORT/data'
```

check只解析/构图，不发HTTP，也不接受--allow-http。httpRequest恰六参method/url/headers/body/timeoutMs/maxResponseBytes，特殊内置总数75，非一等函数值且可被同名绑定遮蔽。默认禁用HTTP，只在当前run session显式授权；--allow-process不授HTTP，两能力独立，模块共享本次会话权限，不跨运行泄漏。

成功响应恰三键status/headers/body；状态404/500也是正常值，不自动抛错。headers是[[lowercaseName,value],...]，保重复和平台API观测顺序，不合并Set-Cookie；并非struct字典或原wire大小写。body严格UTF8且保BOM/换行，不猜charset、不lossy、不自动jsonParse，不支持任意二进制响应。jsonStringify仍按其自身规则排序输出键。

## 请求校验和网络策略

方法只接受大写GET/HEAD/POST/PUT/PATCH/DELETE/OPTIONS。URL是最多16384 ASCII bytes的绝对http/https，支持host/括号IPv6及1..65535端口；非ASCII路径须显式percent编码，拒userinfo/fragment/反斜线/空白/控制/NUL和非法百分号，缺path按/并保query。URL、头与body完整校验发生在触网前。

headers最多256对，每对恰两个string，名字为非空ASCII token，值仅可见ASCII/SP/HTAB并去两端SP/HTAB后提交，禁止CR/LF/NUL。Host、Content-Length、Transfer-Encoding、Connection、Proxy-Authorization、Proxy-Connection、Accept-Encoding、Upgrade禁止由用户指定（大小写不敏感）。请求头按原输入name+": "+value+CRLF的ASCII bytes合计<=65536；body UTF8<=16777216bytes，GET/HEAD必须空body。timeoutMs整数1..600000，maxResponseBytes整数0..16777216。

直连，不自动代理/PAC/WPAD，不自动认证、cookie存储回发、重定向、跨session连接池或应用重试。302只返回原Location，不访问新目标。Authorization/Cookie可由用户显式提供，但没有自动凭据。OS TCP重传不等于应用隐式重试，不能承诺服务端副作用恰一次。

HTTPS使用系统默认证书链、主机名及有效期验证，没有insecure或忽略证书选项、不自动客户端证书，不修改信任库。TLS失败为IOError；本地自签拒绝不是可信TLS成功见证，本指南仅用本地HTTP，不宣称公网或可信HTTPS成功已验。

## 响应限额不是wire计数

maxResponseBytes合并最终WinHTTP RAW_HEADERS_CRLF完整API文本的UTF8 bytes与read得到的body原始content-coded bytes：头计statusline、全部CRLF及末空行，只排除一个API终止UTF16 NUL；不重建/规范化后计数，不是wire/TLS/chunk framing字节。恰N允许，>N报错；0通常连头也容不下，HEAD/204仍计头。禁止自动解压，body计平台原始content-coded数据。返回pairs虽lowercase/trim，不改变计数文本。

响应头查询原生size含NUL最多65536bytes，最多256返回头；body固定8KiB scratch，不按Content-Length无界预分配。返回pairs/转换的有界额外分配不等于进程总内存上限。精确N必须依据实际API头文本证据，不能从Python发送wire文本猜数。

## 超时、取消与回收

局部timeout从后端入口、创建session前起覆盖DNS/connect/TLS/send/headers/body到完成检查，不逐read刷新。局部超时ValueError“HTTP 请求时间上限已到”、超量/UTF8/参数/禁用等约定ValueError可被 `attempt(() => httpRequest(...))` 捕获；DNS/TLS/协议/OS操作失败为IOError。全局cancel/deadline仍ExecutionLimitError，attempt不能吞、重置或延长；native等待不扣AST步数，不把remaining0直接当StepBudget。

同轮观测优先cancel→全局deadline→局部timeout→大小超限→OS/协议错误→成功，首次reason锁存；这不承诺并发发生全序。async协作取消不是函数硬返回时限：close request后仍须等HANDLE_CLOSING和最后callback退出才释放buffer/context，逆序释放连接/session。异常OS回收可能延迟，没有有限绝对回收时界或恶意网络沙箱承诺。

新增HTTP diagnostics与旧子进程diagnostics独立，attempt不清除；库result Ok不能单独代表完整成功，宿主必须检查host_diagnostics()和http_diagnostics()。CLI result Err保原错误对象并把诊断写stderr（先子进程后HTTP）；result Ok但任一非空时stdout空、stderr宿主完整性失败、exit2，即使--json也属于宿主例外。只有HTTP列表非空时标题“lfz: HTTP 资源回收失败”，每条“lfz: HTTP 回收诊断：”；两者非空先沿旧子进程标题，不伪造第二业务错误或新增JSON键。

## CLI与Rust迁移

--allow-http无值；重复“选项 '--allow-http' 不能重复”，等号形式“选项 '--allow-http' 不接受值”；check/test/--help/-h/--version/-V拒绝，消息“HTTP 能力选项只适用于运行命令”。CODE/来源后-- argv保护保持，裸help/version仍文件。混用process/limits时按选项区首次新类别决定非执行适用性，扫描错误仍token序优先；usage即使--json也stdout空/stderr文本/exit2。

Rust EvalCapabilities新增pub http:bool，手写literal须补该字段或使用Default更新；这是明确源兼容迁移。旧非context API默认http=false，EvalLimits形状与三个owned context入口签名保持。模块和attempt不自行升级权限，两个能力不互推。此部分引用spec，不是文档角色Rust编译或host取消测试。

Windows先验，非WindowsUnsupported且无curl/exec fallback，不把提案当跨平台支持。错误仍13类，不新增第14类。R304OPEN，不以HTTP功能通过宣称性能或正式发布。
