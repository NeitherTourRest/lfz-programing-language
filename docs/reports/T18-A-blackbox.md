# T18-A 公开进程黑盒

最终结论：2026-10-30，Windows最终4093714b/984064B，独占fixture 51b83f05，公开 **45/45 PASS，exit0**。以下准备记录保留；末节记录实际过程与脚本错误。

2026-10-30，test-engineer。依据§23/T18-A-design，当前**45例准备完成，尚无最终T18-A验收**。本轮交付后停止，不进入HTTP/T19/T20。

新增 `tests/t18a_cli_process.py`；只用公开CLI、项目内Rust fixture，不导入产品代码、不编译，不改旧测试/产品/Git。fixture契约读取 `Temp/T18-A-tooling/fixture.rs`，当前路径 `fixture.exe`，SHA5baef68cc702694edcfd92175fb8eab1db935b0485cdb4a19d0457b08dae4374；这是准备时指纹，最终须由tooling确认，不称fixture或产品验收通过。

45例覆盖：类型/arity/空argv/NUL/绝对exe/相对与PATH名/批处理拒绝、两数值范围、默认禁用与后续独立进程仍禁用；空串/空格/Unicode argv、stdin EOF、非零exit成功、signal nil、严格UTF8与BOM/CRLF不改；0/5/6合并双输出，超过pipe缓冲双输出及尾部N+1；局部timeout被attempt恢复，全局deadline不能吞；能力flag重复/等号/非执行命令/CODE/argv保护；依赖错误真实来源；主进程退出后后代持pipe必须回收，使用fixture写出PID及本机tasklist观测。

实际准备命令为脚本附`--prepare-only`：AST PASS，exit0/cases_total45/executed:false。所有临时在 `Temp/T18-A-public`，D约216MiB。旧版红命令：

```text
D:/app/ANACONDA/python.exe -B tests/t18a_cli_process.py --binary Temp/T17-release/lfz-T17-1946ab.exe --fixture Temp/T18-A-tooling/fixture.exe --expected-sha 1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45 --output Temp/T18-A-public/T17-red.json --red-only
```

真实结果0/1、exit1；LFZ exit2/stdout空，旧版不识别能力flag，usage“裸文件调用只接受一个 <file> 参数”。仅缺功能红证据，无T18执行。结果与计划复制至 `T18-A-blackbox-evidence/`，manifest和-text属性保全。

测试脚本初版把JSON解析失败也走taskkill，旧版LFZ已退出、未启动fixture；原始清理返回1（sandbox用户名/密码错误）完整保存。脚本现仅实际外部TimeoutExpired才回收树，binary与fixture逐例及最后核hash。正式执行应由root协调许可进程查询/清理权限；taskkill/tasklist只作watchdog和进程生命周期观测，不能当LFZ业务oracle。超时父进程7秒，清理/communicate各3秒，异常留原输出，不隐藏失败。不能在权限不足时声称watchdog已可靠清理。

边界：Job创建/纳管/pipe/read/kill/wait故障注入及host_diagnostics门禁由tooling白盒负责，本脚本不冒充这些证据；库context同进程隔离与token由runtime unit补足。仅Windows本地fixture，未验证Linux/macOS，无公网/外装/系统shelloracle。不保证OS内核不可中断时的硬时限。R304 OPEN，无性能结论。最终hash到位后用新output完整许可实跑，保留所有失败与oracle修订证据。

## 最终实际验收（2026-10-30 18:53）

按root冻结GO，产品SHA4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86/984064B。fixture从tooling按SHA复制至独占Temp/T18-A-public/owned/fixture.exe，初末SHA均51b83f054f128e381fa034adbe980d6398f24f76621703ba27da4d7e54d27bee，避免独立harness重建源fixture的race。复制身份、watchdog-preflight及最终manifest均保存在专属evidence。

许可普通OS权限下先启动唯一身份fixture的sleep PID，tasklist查询、taskkill该PID树、communicate回收均PASS。再完整运行：

```text
D:/app/ANACONDA/python.exe -B tests/t18a_cli_process.py --binary target/release/lfz.exe --fixture Temp/T18-A-public/owned/fixture.exe --expected-sha 4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86 --fixture-sha 51b83f054f128e381fa034adbe980d6398f24f76621703ba27da4d7e54d27bee --output docs/reports/T18-A-blackbox-evidence/first-complete-permitted.json
```

结果45/45 PASS、exit0；script SHAeeef3ced3d038f1c428b205be961d2a42aec2b53541be8417210c79c831c9dd8。双输出/超量/局部timeout/全局deadline及module源均符合预期。后代PID44416由fixture记录，返回后tasklist无匹配进程，descendant_reaped=true；没有靠watchdog清理才声称正常返回已回收。

首次完整调用在NUL case被Python CreateProcess拒绝（embedded null character），未把该case送入LFZ；此前内存行未落结果，不虚构计数。script-first.txt及first-script-error.txt保留错误、命令与说明。仅修新harness将literal NUL源码写入项目内nul-case.lfz，保持原LFZ NUL参数期望，之后首个完整可完成运行45全通过。产品/fixture未改、不重编译，没有放宽产品断言。验后源码逐一匹配tooling T18-A-close manifest；product/fixture逐case及末尾核hash。

本公开45与tooling649/独立验收各自计数，不重复合计，也不替代白盒故障注入。未修改旧测试/Git/dist/spec，无网络/安装。R304 OPEN，无性能结论；遵守T18-A交付后停止，不接后续迭代。
