# T18-B 静态独立预审

2026-10-31，verifier。读取启动文件、§24/T18-B-contract及活动源。仅静态预审，没有编译、启动LFZ或连接loopback/公网；不构成最终功能验收。预审源hash在T18-B-independent-evidence/preaudit-sources.json，最终仍须新hash重新核对。

## 生命周期审阅

WinHTTP FFI只平台模块，调用约定system，handle为pointer/context为usize，AsyncError repr(C)含DWORD_PTR与DWORD字段。callback仅记录有限slot，不调用LFZ；REQUEST_ERROR的info在callback内拷出error，不把临时info指针留给owner。Context为run的Box，发送body由run按值持有的HttpRequest保留，8KiB read scratch为Box；close完成前均不释放。发送header UTF16局部用于SendRequest，本次审阅不将此局部当异步body buffer。

同线程owner发起单个pending Send/Receive/Read，再wait对应状态；callback不发新请求。Close request后等待HANDLE_CLOSING与active==0，再关闭connection/session。active加计在访问state之前，减计/notify在state锁内；owner读取closing/active及离开等待需同锁，避免callback仍访问context时提前释放。close前未注册callback路径不等虚构notification；上下文设置先于callback注册，因此Send前取消仍能关闭通知。无detach/helper。若OS Close失败/通知永不到，等待可能延迟；契约已明确非硬返回时限，最终正常loopback cancellation/error均须watchdog证明收口，不能把无限等待当已验。

## 请求与输出限制

每调用独立session，Open access=NO_PROXY，不查PAC；request disable cookies/redirect/auth，未设置忽略TLS证书标志或自定义信任。HTTPS secure flag与系统默认校验；不把代码存在当作已验可信TLS成功。禁止decompression相关策略待最终实际响应验证；不公网、不修改信任库。

纯helper限制方法/ASCII URL、schema、userinfo/fragment/控制/反斜线/percent/IPv6/port，后端再次调用同helper后才parse unwrap。headers token/value/保留头/输入合计限制独立于能力，runtime六参外层类型及pair类型优先，然后值约束，之后独立http能力与平台。

响应头先查询原生含NUL字节并64KiB封顶，严格UTF16且排除唯一API终止NUL，API文本UTF8 bytes（含statusline/CRLF）与body合并，不用重建pairs替代计数，不声称wire octet。最多256头、每read8KiB、固定scratch，不依Content-Length无界预分配。body严格UTF8在runtime完成，不lossy。最终真实N必须采用WinHttpQueryHeaders观测文本/UTF16证据，不能猜loopback发送字符串。

主poll cancel→globaldeadline→localtimeout，wait/每个阶段与最后成功均检查，local从backend入口一次建立不逐read延长。cleanup diagnostics独立sidecar，runtime先吸收再处理result，attempt不清空，CLI/runner双列表门禁与旧子进程文案分离。最终必须实跑同时事件/close注入/原主reason和source以及caughtIo不能HostSuccess。

## 已报告的确定证据缺口

当前unit cancellation_completion_races_repeat_without_callbacks_left仅20次normal请求、control(cancel=None)，没有取消与完成竞态。名字不能支撑race通过声明，已报告root，请owner补真实marker/同步token和完成竞态并核closing退出，无源码修复权。另当前只见首闸门四unit，不以作者初次结果替代完整错误/故障注入/TLS/公开矩阵与最终独立验收。

## 最终验收计划

最终exe与递归src/Cargo/fixtures冻结后再GO；同产品旧A/T17/C/应用回归，新公开HTTP/真实本地fixture独立再验。loopback仅临时port、有界服务器read/write与shutdown/join/watchdog；验证status404/500、POST字节/query/percent路径/重复头/302不跟随/单请求计数、cookie不自动跨调用/无解压、API头精确N/N-1与HEAD/204、非法UTF8/坏长度/断连、慢headers/body/drip总timeout、全局取消/截止穿透与完成race、module/session权限四组合、双sidecar门禁和关闭注入。HTTPS仅自签拒绝，绝不等同可信TLS或公网TLS成功。所有证据Temp项目内，不编译产品、不改源/旧测试、不Git。非Windows明确Unsupported，R304OPEN，本批完成后收口不T19。
