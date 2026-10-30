# T18-A Windows受控子进程后端与CLI作者验证

日期2026-10-30。tooling-dev。按三spec§23/T18-A-design实现；本报告只声明Windows作者证据，不替代独立验收或R304性能门禁。本轮结束即停止，不实现HTTP/后续。

## ABI与生命周期

src/process_control.rs实现纯owned ProcessRequest/Control/Output/Failure/Report，lib.rs以pub(crate)登记，Win32 FFI只在私有process_control/windows.rs。后端再次校验host请求，非法数据为请求校验Io，不接Value/Rc/Env或终端输出。非Windows分支返回Unsupported，本批没有真实Linux/macOS验收。

Windows先CreateJobObject/kill-on-close，创建独立双pipe及null stdin，STARTUPINFOEX只白名单继承三标准句柄，以CREATE_SUSPENDED|CREATE_NO_WINDOW启动，AssignJob成功且控制检查通过才Resume。无裸spawn回退。命令行UTF16可写，argv按C runtime quoting处理空参数/空格/Unicode/引号/末尾斜杠。

读取为同线程PeekNamedPipe后只读已available字节，两个stream每轮公平轮转，每次最多8KiB；无reader线程、channel或join，有限scratch一块8KiB。向输出Vec reserve_exact后只保留合并允许前缀，总保留输出不超过N；N允许，N+1失败，不返回截断成功。内核pipe/子进程内存不属于此缓存限额。

cancel/全局deadline/局部timeout优先于输出和普通完成；首次失败reason保留，cleanup不吞系统资源终止。正常直系退出也立即终止Job后代，确认直系signaled和Job active=0，再有限drain至pipe EOF；不等后代无限持pipe。cleanup正常确认有2秒轮询上界，失败留diagnostics及真实失败，Job RAII kill-on-close仍是兜底，不把OS异常下未确认的清理称成功，不承诺内核不可中断情形的绝对时界或安全沙箱。

ABI审查参考微软官方[CreateProcessW](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw)、[创建进程与句柄白名单](https://learn.microsoft.com/en-us/windows/win32/procthread/creating-processes)、[Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)。实现保留RAII逆序释放和本地实测证据，不能只凭API名称宣称回收可靠。

## CLI与完整性门禁

--allow-process无值、每次execute默认false，仅运行命令可用；CODE/--后argv保护沿T17。重复、等号形式及非执行适用性均按冻结消息拒绝。与限额组合按首个受限选项类型选择适用性文案，扫描错误仍token序优先。授权仅本session，通过新EvalContext入口传递，不改旧EvalLimits shape或用全局/thread-local权限。旧求值API和runner/check不隐式授权。help说明绝对exe、输出返回值及能力不是沙箱。

TracedRun.host_diagnostics在CaseEval独立WithDiagnostics载荷中保全。result Err照原错误/JSON渲染，诊断独立stderr前缀；result Ok但非空诊断时stdout空、固定“lfz: 子进程资源回收失败”及逐行诊断、exit2，即使--json也不伪造业务错误。root明确批准最小runner门禁：成功诊断不得PASS，错误诊断保主错误而判ERROR。合成门禁单测验证成功、全局Cancelled错误、runner误PASS防御。未增加JSON键或第14类。

## 本地fixture与作者结果

可复现Rust fixture源码src/process_control/fixture.rs，项目rustc构建到Temp/T18-A-tooling（不安装、不联网、不运行系统命令）。模式包含长度前缀argv、双流262144字节、非法UTF8/BOM/CRLF、u32非零exit、stdin EOF、sleep、marker、父进程退出而后代持pipe。后代PID在返回后用进程句柄确认signaled/不存在；活动native取消在marker证实已执行后由另线程置token。

- 旧T17产物不识别能力选项的真实缺功能red.json保留。
- 首版backend定向7/7通过；后增加真实活动native取消，最终backend八项包含于484lib。
- Job创建/纳管/Pipe/Read/Wait/Kill/CleanupWait故障分支注入，不制造内核损坏；kill/wait诊断注入仍执行真实终止/等待以免测试泄漏。Job/Assign/Pipe故障未让marker执行，cleanup故障非空诊断且不假成功。不能将合成注入称为真实OS内核故障。
- 首个外watchdog遗漏项目RUSTUP_HOME，7个fixture compiler选择失败，原失败日志单独保留；仅补项目工具链env，重跑同编译产物，不下载/安装或改产品规避。每个harness90秒外watchdog，实际未触发；若杀harness，其已持Job kill-on-close负责结束fixture。
- 最新十一release harness646/646：484lib+53bin+28cli+12T12+12T14+7B+5C+9T17+6T18+15runner+15unit，0失败/忽略。涵盖旧全套、能力隔离、模块授权、捕获输出不泄终端、local timeout/output/UTF8可恢复、全局deadline不可吞以及门禁。
- `cargo test --release --no-run`、`cargo build --release`成功。环境canonicalize警告和Windows Unsupported variant未构造dead_code warning如实保留，不称零warning。

首阶段候选（补关闭诊断前）target/release/lfz.exe984576B，SHA256 `e5f57aa823ad7738fc136ee39b585d7c1db288ff9f28253f3cfa8016f8b34682`。docs/tooling/evidence/T18-A/manifest.json递归绑定全部src（含平台及fixture）、Cargo/newCLI测试、原始日志；candidate-snapshot.zip保精确源码/exe/fixture二进制，snapshot.json记zip hash。不要沿用旧平铺src/*.rs遗漏嵌套平台源。

本角色未写runtime/core/spec、未覆盖dist/Git或外部安装。待root确认平台/组合文案补钉及独立实测，保持源码静默；任何后续owner源码修订都须重建新hash，不能拿本成绩代替新源结果。Windows通过不表示Linux支持，能力开关也不是OS权限隔离；R304仍OPEN。

## 关闭诊断补丁与最终作者验收（2026-10-30 18:26）

root要求将CloseHandle本身失败纳入可靠回收。本次run拥有局部Handles注册表，创建成功后立刻登记句柄，必要早关显式close，execute任何Result返回后显式close_all逆序处理剩余句柄，Drop仅意外退出/展开兜底。槽先取空，再单次真实CloseHandle；失败记录独立diagnostics，不重试可能已经消费的句柄值。无生产全局/TLS sink、无Rc跨owner边界。句柄关闭诊断遇正常结果改Io，已有错误原样保留，尤其全局Cancelled仍是不可吞主原因。

新增两unit验证早关/最终逆序关/重复close_all不doubleclose，OS Wait确认注入句柄确已关闭；正常完成、CreateProcess失败、Assign挂起纳管失败、pipe创建失败均保全close诊断。真实fixture marker握手后另一线程取消，验证close诊断不覆盖Cancelled。close注入仅报告故障，仍执行真实CloseHandle；不制造内核损坏或真实句柄泄漏，不声称覆盖所有内核异常。

runtime owner另修非Windows合法绝对路径校验以进入Unsupported并新增1unit，本文不宣称Linux执行验收。重新cargo test --release --no-run与cargo build --release均0；新十一release harness649/649（487+53+28+12+12+7+5+9+6+15+15），0失败/忽略，外watchdog未触发。Windows Unsupported dead_code与环境canonicalize warning仍保留。

最终作者候选target/release/lfz.exe984064B，SHA256 `4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86`。新证据docs/tooling/evidence/T18-A-close/manifest.json递归绑定源码与日志，candidate-snapshot.zip精确保全。旧T18-A的E5F57候选、646及失败环境日志全部保留。本角色源码/编译已停止，交root独立验收；未动dist/Git，仍仅本轮，不HTTP。
