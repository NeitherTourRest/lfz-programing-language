# T18-B HTTP 人类指南最终聚焦实跑

日期2026-10-31，docs-writer。最终target/release/lfz.exe **1029120B / SHA256 18d328d9e48780f8e6ae638f9ad24f53186d94f2207ca24da8928cbbba0d0916**。五人类指南HTTP入口及http.md完成，正常OS网络许可下stdlib loopback聚焦验证 **21/21通过**，exit0。未使用A产品或初次HTTP编译产物冒称通过。

## 命令、权限失败与服务器回收

`D:/app/ANACONDA/python.exe -B docs/reports/T18-B-guide-evidence/verify.py --binary target/release/lfz.exe --expected-sha256 18d328d9e48780f8e6ae638f9ad24f53186d94f2207ca24da8928cbbba0d0916 --output Temp/T18-B-docs/新的结果.json`

默认沙箱教程check成功，首次HTTP GET为IOError“HTTP 异步请求失败：WinHTTP error 12029”/exit2；default-failure.json/default-stderr.txt保存实际两行记录及脚本断言失败。没有改成允许IOError的oracle，也没有关闭产品策略或换外网。按主调度授权，用同字节正常OS网络许可完成21/21；runs.json和server-shutdown.json保全结果/本地请求记录与server_stopped:true。finally执行shutdown/server_close/join，临时端口回收，未留下前台服务。

本次许可运行Python HTTP/1.1 keepalive读线程曾在客户端独立session关闭时向工具输出打印ConnectionResetError（WinError10054）；不把该工具输出说成无warning。它不是LFZ响应解析失败，21条LFZ输出/exit断言全部通过，服务器最终关闭证据为true。没有为抹掉这些输出重复择次运行。

## 实际覆盖

1. 从http.md提取fetch.lfz，先check唯一executed:false对象且无请求，再授权run读本地UTF8 JSON并输出雪。
2. 默认禁用、只allow-process仍禁HTTP、下一独立运行仍禁用；禁用调用服务器hits不增加。
3. 404普通值与响应重复X-Repeat保留，三响应键；POST中文body精确回显，显式重复请求头到达服务器。
4. 302返回且/target从未被访问；非法UTF8整体ValueError；HEAD的maxResponse0仍因计头超量。
5. 局部慢头timeout被attempt变为业务失败；全局deadline穿透attempt为ExecutionLimitError。这里只验证reason和受控返回，不对毫秒级时界或所有race顺序作断言。
6. method非法触网前拒绝、保留头拒绝、GET非空body拒绝；argv同名能力文本保护、等号/重复/非执行命令精确usage拒绝。

每次启动前及结束核最终产物SHA，六人类指南测前测后hash一致。原始逐进程args/stdout/stderr/exit、服务器hits与关闭证据、验证器及manifest按原字节保存。没有修改产品、core/runtime、规格、AI、dist或Git，没有编译。

## 文档及未声称范围

http.md明确六参/三键、headers pairs与重复、4xx正常值、strictUTF8/BOM/换行、独立http能力默认false与Rust EvalCapabilities literal新增http字段迁移、请求头体硬限、maxResponseBytes为WinHTTP RAW_HEADERS_CRLF API文本UTF8+content-coded body而非wire。精确N需backend实际API文本证据，本指南不从本地wire猜数，也不把0限额失败当作精确N门槛验证。

网络策略无自动redirect/cookie/auth/proxy/PAC/retry；显式Authorization/Cookie与应用重试由调用者负责，不能保证服务端副作用恰一次。TLS沿系统验证无insecure，不公网、不改信任库；本地HTTP成功不等于可信HTTPS成功或TLS拒绝见证。

协作式async取消必须等安全closing与最后callback，非硬返回时界。Rust宿主需检查子进程与HTTP两个diagnostics列表，result Ok不充分；CLI宿主例外和原错误JSON协议已写。RustAPI/取消token/cleanup故障注入为规范引用，未冒称由本指南编译或实测。仅Windows先验，其它平台Unsupported，不恶意网络沙箱。

R304仍OPEN，不以本报告称性能或正式发布通过。仅本轮HTTP交付收口，不T19或恢复自动化；所有临时/服务均项目Temp/T18-B-docs。
