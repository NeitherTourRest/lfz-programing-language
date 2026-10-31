# T18-B AI HTTP指南准备记录

日期2026-10-31，责任人ai-dx-engineer。状态：**prepare-only；未执行解释器、未启动网络服务、未绑定最终产物hash，不能宣称HTTP示例通过**。

四skill与AI指南入口按三spec§24/T18-B-contract及人指南http.md同步新增§6.7：先check图不执行、显式--allow-http限额run、六参三键与75内置/13错误类、独立能力与Rust literal新增http迁移；保重复header/4xx正常值/严格UTF8；API头完整RAW_HEADERS_CRLF文本UTF8字节+body限额（非wire）；无自动代理/auth/cookie/重定向/retry；局部错误恢复/全局终止不吞；安全HANDLE_CLOSING及最后callback退出，不硬时界；两个sidecar宿主异常JSON例外与Windows先验。

## 准备验证

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T18-B-skill-evidence/run_examples.py --prepare-only
```

exit0，只从两份AI指南提取LFZ批请求示例写项目Temp/T18-B-ai，解释器启动0、网络启动0。旧T18-A产物未用于HTTP验收。最终hash由team-lead指定后，用--binary/--expected-sha256/--output新路径实跑，输出原始args/stdout/stderr/exit、五文档前后hash与loopback访问日志。

验证器计划仅127.0.0.1临时port服务，包含两指南提取check/run、200与404批请求、重复header、POST UTF8、302不访问目标、HEAD仍计头、strictUTF8、局部timeout可attempt/全局deadline不可吞、默认禁用及process不能授权HTTP、后继session隔离、完整校验先于能力、flag等号/重复/混合类别/扫描错误/CODEargv保护。服务每连接有限timeout、slow端点固定200ms，finally shutdown/server_close/join；无公网、无安装或TLS信任库改动。

不从Python wire猜精确N，不宣称已覆盖真实API exact-N/native buffer上限、TLS/Barrier关闭race/故障注入/库并发token；这些由后端及独立验收提供。RustAPI迁移为契约引用，没有文档角色编译新见证。本轮功能候选R304OPEN，不代表性能或正式发布；不重跑旧v2，不恢复自动化，B收口后不自动T19。
