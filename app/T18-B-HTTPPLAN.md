# T18-B httpplan：显式能力的HTTP任务批处理

日期2026-10-31，app-dev。依据三spec §24/T18-B-contract；在LFZ skill的既有JSON/attempt工作流上引用新HTTP契约，旧应用和oracle不改。准备阶段先冻结应用与oracle；最终同字节实跑7/7通过，证据见文末。仅本轮B收口，不进入T19。

## 接口与实用命令

`app/httpplan.lfz`接受CONFIG.json OUTPUT.json两参数。配置为任务array，每项显式填写name、method、url、headers、body、timeoutMs、maxResponseBytes。headers是pair array，不能用对象合并重复头；body是UTF8字符串，不自动编码为JSON。

```json
[{"name":"local","method":"POST","url":"http://127.0.0.1:18080/echo","headers":[["Content-Type","application/json"]],"body":"{\"id\":1}","timeoutMs":5000,"maxResponseBytes":4096}]
```

调用者先启动其信任的本地服务并填写实际port，准备好项目Temp输出父目录后执行：

```powershell
./dist/lfz.exe check app/httpplan.lfz
./dist/lfz.exe run --allow-http app/httpplan.lfz -- Temp/T18-B-app/plan.json Temp/T18-B-app/report.json
```

应用不自动启动服务、不使用子进程、系统shell或代理。未授权HTTP的非空有效计划在首个Disabled错误后转外层AssertionError“httpplan: 需要 --allow-http”，退出2且不写报告；空计划没有网络调用。--allow-process不授HTTP，能力不跨运行泄漏。method须契约允许的大写方法，URL非ASCII路径需调用者percent编码，GET/HEAD body须为空；其余具体URL/headers/body值校验委托httpRequest后端，不能靠应用参数验证冒充权限沙箱。

## 结果与错误策略

先验证所有任务的必要字段和外层类型，再按顺序调用。每项attempt包住一个httpRequest，局部timeout、响应大小超量、非法UTF8及其它可恢复错误形成{name,ok,value,error}，后续任务继续。成功value严格{status,headers,body}；404/500等HTTP状态属于正常响应，ok=true不表示业务成功，业务应再检查status。

响应headers是API观测的lowercase名称pair array，保留重复值及API顺序，不承诺原wire大小写；body严格UTF8，不自动解析JSON、不猜charset。302响应原样返回，不跟随Location。最终报告total/succeeded/failed/rows以jsonStringify确定性编码，最后一次覆盖输出，stdout只打印统计。CLI --json状态与业务输出仍分开。

全局deadline/cancel是ExecutionLimitError，不能被attempt变成普通失败行，整程序退出2；已完成的HTTP请求及服务端副作用不回滚。报告写入前完成验证和编码不等于原子输出，也不能保证服务端处理只发生一次。只检查配置与输出字面不同，实际路径别名由调用者避免；响应及整计划结果驻内存，非流式处理。

maxResponseBytes合并最终WinHTTP RAW_HEADERS_CRLF API文本的UTF8字节和原始body bytes，包含statusline/CRLF，非wire bytes；即使无body也计头。应用测试用1字节上限稳定覆盖超量，不凭HTTP线格式猜精确N。超时是整个请求的协作截止（包括headers/body），不是每次read刷新或硬实时return保证。HTTPS沿系统默认信任验证；本应用不关闭校验、不改系统信任、也没有可信公网TLS成功测试。

## 自包含回环验收准备

`verify_httpplan.py --prepare-only`只检查配置与源码指纹：AST PASS、解释器启动0、server未启动。收到最终hash后执行：

```powershell
D:/app/ANACONDA/python.exe -B app/verify_httpplan.py --lfz target/release/lfz.exe --expected-sha FINAL_SHA256 --output Temp/T18-B-app/final.json
```

测试仅绑定127.0.0.1临时port，标准库ThreadingHTTPServer。每个连接2秒读超时，慢端点固定400ms；主server shutdown/close等待活动handler，server线程必须退出。子解释器12秒watchdog只作为测试外层防挂，不能当HTTP deadline精度保证。全部结果实时保留，失败也保存stdout/stderr/输出hex和server请求记录。

预设7次进程：禁用与仅process能力均不连服务；八任务批次重复两次（GET、POST Unicode body/percent URL/显式header、404普通值、302不跟随、坏UTF8/局部timeout/大小超量三个失败后继续）；全局deadline须实际发生在httpRequest匿名回调行/帧，之后任务不发送；所有任务先验证，后面缺字段时前面的合法任务也不能触网；空计划成功。固定fixture不用Date/Server动态头，正常重复批次报告应逐字节一致。POST路径/body/header与返回重复X-Reply由独立服务端/JSON oracle验证，不调用解释器内部实现。

服务端日志中/target必须始终不存在，用来验证禁用/302/全局终止/前置验证均无隐式后续请求。默认sandbox若拒监听或WinHTTP访问，保留失败并报告，经许可用同产物重跑，不修改产品绕过。未申请公网、证书信任修改、外部安装或编译/Git。R304仍OPEN，准备状态不等于实测或正式发布。

## 最终实跑（2026-10-31）

冻结 `target/release/lfz.exe` 1029120 bytes，SHA256 `18d328d9e48780f8e6ae638f9ad24f53186d94f2207ca24da8928cbbba0d0916`。执行上述命令，将FINAL_SHA256替换为该哈希，输出分别为 `Temp/T18-B-app/final-default.json` 与获正常OS权限后的 `Temp/T18-B-app/final-permitted.json`。

首个默认sandbox结果保留在 `app/t18b-evidence/default-permission-failure.json`：已执行3进程，前2项禁用检查通过，首批8项全为WinHTTP 12029连接失败，服务收到0请求，关闭完成。没有修改应用或oracle，随后用同一哈希获许可运行，`app/t18b-evidence/results.json` 记录7/7 PASS、exit0、server_shutdown_complete=true。此差异是权限环境证据，不据首轮推断产品故障。

两次八任务批次均5成功3局部失败，报告字节相同；POST UTF8正文/percent路径/header、重复响应头、404/302正常值、坏UTF8/局部timeout/字节超量后继续均过。全局deadline定位httpRequest匿名回调，KEEP输出保留且后续请求未发送；禁用能力及前置验证均无连接，空计划成功。所有请求只到测试自行创建的127.0.0.1动态端口，服务完整回收。应用源码哈希与准备阶段相同，未修改旧应用、产品或旧oracle，无编译/Git操作。

`manifest.json`锁定应用、oracle、本文及两份完整报告。此为应用作者验收，独立验收由verifier负责；R304仍OPEN，不扩大为公网HTTPS成功或正式发布结论。本轮B交付后停止。
