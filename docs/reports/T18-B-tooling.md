# T18-B HTTP 工具链作者报告

## 当前候选与范围

2026-10-31，tooling唯一实现http_control.rs/私有windows.rs、CLI/main相关及lib登记，root明确授权runner最小双diagnostics门禁。core/runtime独立所有权，等二者停源后唯一release整编。未改旧测试、Git/dist、系统信任、代理或安装；本轮后停止，无automation/T19。

最终作者候选target/release/lfz.exe：1029120B，SHA256 `18d328d9e48780f8e6ae638f9ad24f53186d94f2207ca24da8928cbbba0d0916`。完整12harness667/667=499lib+56bin+28cli+12T12+12T14+7B+5C+9T17+6T18A+3T18B+15runner+15unit，0失败忽略。release no-run/build均0。日志保留Windows Unsupported dead_code和环境canonicalize警告，不称零warning。独立/公开方法body/重定向/重试矩阵由root接续，本报告不冒称已完成整个验收。

## 实现与安全审查

纯owned HttpRequest/Control/Report/Response/Failure及三个无网络validator沿§24；runtime与backend共用。Windows WinHTTP零crate、NO_PROXY独立session、禁cookie/redirect/auth，单次Send/无RESEND循环，不打开自动解压或忽略证书校验。非Windows固定Unsupported，无shell/curl/helper fallback。

所有WinHTTP操作由一个调用者推进，一次一个pending operation。回调仅写有界completion槽/closing标记并通知，不调用LFZ、不推进递归读取，不分配流量队列。Box context/read buffer与请求body在整个异步生命周期保持地址有效。注册非零context/HANDLES，Close request一次后安全等待HANDLE_CLOSING且active0，再逆序close connection/session。回调最后active--和notify在同一mutex临界区内；owner需取得该mutex才能确认结束，回调解锁后不再访问context。callback只处理已知完成及错误，忽略信息通知，无用户代码或panic展开。execute外围catch_unwind保证意外展开也先完成同样回收再resume，不提前析构发送/读取缓冲。

本地deadline在创建session前checked_add，等待5ms间隔poll，cancel/global/local先于完成/OS。SetTimeouts只阶段辅助，不能称硬总时界；取消后安全关闭可因异常OS延后返回，不detach/提前free。close故障注入仍执行真实close并等closing，不制造泄漏；正常结果diag变Io，主Deadline保持。真实kernel故障不宣称已制造/覆盖。

响应头先native size query，含NUL硬限65536 native bytes再分配，依据第二次查询长度去API终止NUL，拒内嵌NUL/非法UTF16。严格文本UTF8 bytes加raw body累计，N恰可/N-1拒；header pairs从原文本另解析，保重复/API观测顺序，最多256。body固定8KiB buffer，不信Content-Length分配、不包含chunk framing。原文与pairs绝不重建替代计数。

## 作者实测与阶段划分

- 原4093714 --allow-http -e 1真实usage2红见证，未触网络。
- 首编compile-first通过，生命周期首4/4=20normal完成、live cancel/global deadline、慢头/体/drip总timeout、404重复头API exactN。原始日志保留；该阶段没有将20normal冒充取消竞态。
- verifier指出normal循环不是race后保原循环，新增40次真实请求marker后Barrier同步取消线程和服务端响应完成，允许Ok/Cancelled合法胜者。每个注册request run收尾test断言closing=true/active=0。第二次release源码新7/7生命周期包含真实race、close诊断、公开专属API头见证，0.68s，40秒外watchdog未触发。第二次完整667/667属于最终hash。
- CLI三组新进程测试覆盖独立能力默认禁用、验证先于触网、mixed类别扫描错误序、重复/等号、CODE/argv保护、跨module错误真实来源；合成CLI门禁测HTTP单列表/双列表成功两格式stdout空exit2，以及全局原JSON与双stderr顺序；runner合成成功不得PASS。旧process文案/断言保持。
- 两次编译与生命周期测试不并发；最终12程序各90秒外watchdog、所有fixture/loopback在项目与临时端口，无watchdog触发。

## API原文精确N证据

重复头fixture实际RAW_HEADERS_CRLF头82 UTF8 bytes/native164 bytes，body3，N85；API重排Connection/Content-Length/X-Dup，不能用原wire顺序猜。public固定wire `HTTP/1.1 200 Test\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok` 单独真实调用见证：API头59 UTF8/native118 bytes，body2，N61，exact61成功/60 ResponseLimit。public-header-witness.json含实际raw_headers_crlf与fixture_wire_sha256 `685f7bfbaa0613d701d83e0ffe7b42567be633933ee26cefdb17df29866ca221`。对应txt/u16le原件均保全，不加产品协议键。观察槽test-only thread_local仅保存当前测试的API原件，无生产能力TLS/global，不受其它并行测试覆盖。

## 本地TLS拒绝

使用机器已存在Conda openssl离线生成项目内临时单日自签SAN localhost/127.0.0.1证书，未安装/导入信任；Python std ssl监听127.0.0.1临时端口，LFZ真实HTTPS调用。首openssl因默认config缺失失败日志保留，显式项目config成功。服务端可能先完成TLS层握手，不等于客户端信任；最终观测服务端application_bytes=0，客户端WinHTTP12175、LFZ IOError/exit2、stderr空。tls-rejection-final.json保原记录。不称可信TLS成功、公网TLS、DNS迟延已实测；不改Windows trust/CRL策略。无关临时测试私钥不放交付ZIP。

## 回执及限制

永久证据docs/tooling/evidence/T18-B/保存红/首阶段/最终编译与watchdog日志、API头原件和TLS证书及拒绝、递归src manifest及精确source/exe ZIP。不覆盖旧A阶段证据。下一步root安排公开49与独立，包括方法/body/query/redirect/请求次数及坏协议大矩阵。异常OS不硬实时、Windows结果不代表其他平台支持，R304仍OPEN。
