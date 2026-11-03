# T19 reader 工具链作者实现与证据说明

本轮tooling唯一写src/line_reader.rs与lib注册，runtime/core独立所有权；无新CLIflag或生产FFI，不改旧readFile。最终成绩/候选见同目录evidence/T19 manifest，本报告在最终验收后补具体hash。

## 实现

LineReader持有File的BufReader，固定8KiB。打开前metadata及打开后handle metadata均要求普通文件，path/cap防御校验；不承诺TOCTOU消失或阻塞磁盘硬中断。next_line在每块前及成功line/EOF返回前只poll cancel Acquire与deadline，不扣AST预算，不用Interp.poll。

LF唯一分隔，raw行限额在追加前检查、包含随后剥离CR、不含LF；仅LF结束时剥一个CR，EOF末CR与BOM/NUL保留。Vec实际需要时reserve_exact(take additional)，len和capacity均受cap约束，test断言验证；不预分配16MiB、不几何扩容到超过cap、不先read_line全行。UTF8按完整有界行严格解码，跨8KiB字符正确。无额外尾空行，i64::MAX后EOF允许，有下一行才Overflow。String从Vec转移，reader不保留历史行。

finish消费owner并drop，错误亦RAII兜底，不伪称可报告OS close错误；不sync_all、不复用T18诊断。不将资源存入Value/Env/Rc。test-only Drop spy/故障钩子无生产能力或全局权限，实际File仍正常RAII回收。test中的handle count FFI只测进程总量，不进入生产reader。

## 关闭证据设计

Windows默认共享DELETE，rename/delete不能证明关闭。实际测试先验证File/活callback期间share_mode(0)排他重开SharingViolation32，再验证函数返回且原宿主/traced仍存活时同文件排他成功。覆盖EOF、false、UTF8、TooLong、deadline、非bool、业务错、Step、marker后实际另线程cancel以及外attempt返回。不能拿进程退出后成功重开代替。

Drop spy独立证明reader析构一次，配合每个Outcome后的真实排他探针；metadata/open/打开后metadata/read错误为明确合成注入，不宣称制造真实磁盘故障。预热50次后同进程1000次打开/读/finish取句柄plateau，最终完整测试串行执行减少其他case并发句柄噪声，阈值容忍8个系统波动，不称所有OS异常零泄漏证明。

## 阶段与失败保全

原产物真实NameError红测保留；初红输入路径不存在，仅证明缺名字，不能单独作为新接口成功oracle。另用未改dist旧18d328与真实一行文件生成red-valid-input.json，冻结同源码在新产物应成功，准确区分缺功能与无效路径。

首release reader7/7通过（3.81秒），覆盖16MiB、跨块UTF8、CRLF raw限额、BOM/NUL、早停坏尾、行号overflow、普通文件/控制、活callback实际cancel/排他及plateau。其后补test-only故障、Drop spy、全部关闭Outcome实际重开。

首次最终lib515通过/1失败：本人新增测试脚本 `(l,n)=>{while true {} true}`违反既有TwoStatements规则。完整首失败日志/summary保留first-final-failed；只修本人新测试换行，不改parser/runtime/旧测试，不把测试oracle错误称产品bug。修正后重新唯一release no-run并同产物全套，不能沿首7成绩代后续源。

所有测试使用项目Temp、同harness正常OS许可，已知A旧Close探针默认沙箱C0000008不再次故意触发；既有旧失败证据不动。harness每个90秒外watchdog，stdout/stderr原始日志保存。

## 下一步与边界

最终源冻结后root接公开/独立，perf统一10/100/300MiB独立oracle/峰值WS/吞吐窗口，tooling不重复大夹具或把16MiB行边界测当规模成绩。未测Linux/macOS，不将标准库路线当跨平台实测；R304仍OPEN。仅T19交付后收口，无Git/dist/外装/公网/T20/automation。

## 最终作者回执（2026-11-04 01:23）

修后release no-run/build均exit0，新12harness684/684=516+56+28+12+12+7+5+9+6+3+15+15，0失败/忽略；旧A关闭probe仅正常OS许可同harness实跑，未再次默认sandbox诱发abort。每程序90秒外watchdog未触发。reader十一unit、runtime五unit和core新增消息测试均实际同编执行，故障/spy与真实存活host重开路径均通过。首515/1原始失败和修正依据保全。

最终1041920B/SHA256 `5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5`。有效输入红(old dist NameError2)/绿(new exe0)同source见证均落盘；初无效路径红仅作名字缺失阶段记录。永久docs/tooling/evidence/T19/manifest.json递归绑定全部src及Rust tests、Cargo、最终exe和日志；candidate-snapshot.zip精确保全。警告Unsupported与Io未读取字段、环境canonicalize保留，未宣称零warning。

源码/编译停止交root；公开/独立/规模oracle与perf尚待其验收，不在本报告冒称完成。未覆盖dist或Git，原用户块保留。
