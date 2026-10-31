# T18-B AI HTTP指南与最终作者实跑

日期2026-10-31。责任人ai-dx-engineer。四skill与AI入口§6.7按三spec§24/T18-B-contract及人指南http.md同步，75内置/13错误类。本轮HTTP授权覆盖旧A停止边界，B收口不自动T19或恢复自动化。先check不执行，再显式--allow-http限额run，检查HTTP状态及业务body而非只看exit0。

## 最终实跑

最终target/release/lfz.exe1029120B，SHA256 `18d328d9e48780f8e6ae638f9ad24f53186d94f2207ca24da8928cbbba0d0916`。作者同最终产物正常OS网络许可下**25/25**，exit0，五文档测前后SHA一致、每进程核exe SHA、`server_closed:true`。服务仅127.0.0.1临时port，shutdown/server_close/join后无存活服务线程；slow handler有限200ms/连接timeout2s。

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T18-B-skill-evidence/run_examples.py --binary target/release/lfz.exe --expected-sha256 18d328d9e48780f8e6ae638f9ad24f53186d94f2207ca24da8928cbbba0d0916 --output docs/reports/T18-B-skill-evidence/results.json
```

默认沙箱首次**17/25**（8项触网场景WinHTTP error12029导致不符预期）保存在default-results.json；同脚本/同产物提权正常OS网络许可复核25/25，未改产品/预期或移除失败。两次均server_closed=true。prepare.json、原run_examples-pre-final.py和preparation-report.md保留原准备阶段，准备阶段解释器/网络均0启动。最终脚本仅较prepare版补server_closed可核回执，随后才两次实跑。

25项实际范围：两份AI指南提取批请求程序各check/run，check无连接、run同时200和404正常值；默认禁用/process不授HTTP/新session禁用；六参三键body JSON、重复header保序；POST UTF8与服务日志仅一POST /echo；302目标/target从未访问；HEAD0额度因仍计头失败；坏UTF8和局部timeout可attempt，全局deadline不可吞；方法/headers校验先于权限无连接；--allow-http等号/重复/check及混合首次类别、扫描错误优先、CODEargv保护。所有args/stdout/stderr/exit/新连接数及服务访问日志保留在results.json。

文档完整说明API原始RAW_HEADERS_CRLF文本UTF8+body计数、非wire/不重建头；无自动代理/PAC/auth/cookie/重定向/retry；严格UTF8；可恢复局部与全局终止边界；安全HANDLE_CLOSING+最后callback退出而非硬返回时界；EvalCapabilities.http literal迁移、独立host/http诊断sidecar及CLI宿主JSON例外。

本作者未做精确RAW_HEADERS_CRLF API N/N+1、原生头buffer上限、Barrier race/关闭注入、库token或TLS实跑；其文字仅引用契约与对应owner证据，不用Python wire猜API计数，不把本地自签拒绝说成作者所跑或可信TLS成功。仅Windows先验，其它Unsupported、无curl/exec fallback、非沙箱。不联网公网、不改系统信任、不编译/改源/人指南/spec/dist/Git/旧v2。R304OPEN，功能示例不是性能或正式发布批准。
