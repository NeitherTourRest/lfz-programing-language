# T19 公开进程准备

最终公开结论（2026-11-04）：冻结5d2219f8/1041920B，**47/47 PASS，exit0**。首个完整许可运行通过、首脚本未变；6个同活PID排他关闭对照全部通过。以下准备记录保留，末节为最终实跑证据。

2026-11-03，test-engineer。按三spec §25新增 `tests/t19_cli_lines.py`，**47例准备完成，尚无T19最终验收**。只有新脚本/专属报告与记忆，未改产品/旧测试/官方计数、未编译/Git/公网。

当前覆盖空/连续空行/尾LF/无尾换行、CRLF去一个CR/EOF末CR保留、BOM/NUL/空白和#42不加载源码前导、8192边界UTF8、多字节raw cap及含CR N/N+1、坏UTF8/第二行TooLong、false尾毒不交付、callback类型/空文件arity/argv arity/非bool/范围/pathNUL/缺文件目录、JSONL内attempt混合坏行后成功、共享AST受控终止、callback home与真实模块source。

6个关闭case分别EOF/false/callbackError/nonbool/UTF8/tooLong。先无reader的CreateFileW share0独占open正控制；LFZ callback写ready等待go，PID存活且reader保持打开时独占probe必须失败Win32 32负控制；发go后外层attempt收住结果写done，LFZ再等待release，Python此时PID仍活且同文件独占probe成功；最后release令LFZ退出。每个握手2秒、LFZ全局6秒、父进程8秒watchdog，仅kill自己PID。没有删除/rename证据，没有把进程退出后重开冒充调用返回前关闭。ExecutionLimit终止关闭另需同host库证据，不计本公开6个可恢复路径。

实际准备：

```text
D:/app/ANACONDA/python.exe -B -c "import ast,pathlib;ast.parse(pathlib.Path('tests/t19_cli_lines.py').read_text(encoding='utf8'));print('AST PASS')"
D:/app/ANACONDA/python.exe -B tests/t19_cli_lines.py --binary dist/lfz.exe --expected-sha 18d328d9e48780f8e6ae638f9ad24f53186d94f2207ca24da8928cbbba0d0916 --output Temp/T19-public/prepared.json --prepare-only
D:/app/ANACONDA/python.exe -B tests/t19_cli_lines.py --binary dist/lfz.exe --expected-sha 18d328d9e48780f8e6ae638f9ad24f53186d94f2207ca24da8928cbbba0d0916 --output Temp/T19-public/T18B-red.json --red-only
```

结果：AST PASS；prepare-only exit0/cases_total47/executed:false。旧冻结T18B真实红0/1、脚本exit1；LFZ exit2/NameError缺forEachLine，原始输出与计划/脚本hash封于 `T19-blackbox-evidence/`。未拿旧产物当T19成绩，未执行关闭probe。全部fixture在Temp/T19-public，最大的单行约8KiB，未造OOM/abort/大数据负担。

最终GO后核产品SHA，完整许可执行并保首结果；每例与末尾核SHA。Windows排他probe需正常OS权限，失败保留再由root协调，不放宽32负控制。现脚本callback等待使用本地readFile/control文件，不调用外部服务/子进程shell。全局token/资源终止drop、计数overflow、读失败注入、16MiB硬界、嵌套/self/长期handle plateau与10/100/300MiB性能规模仍需unit/独立/perf证据或后续补例，不声称47覆盖所有§25条款。

R304 OPEN，不报告吞吐/RSS或正式发布。本轮T19交付后收口，不自动T20/automation。

## 最终47实跑（2026-11-04）

冻结target/release/lfz.exe1041920B/SHA5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5。许可普通Windows文件访问完整执行：

```text
D:/app/ANACONDA/python.exe -B tests/t19_cli_lines.py --binary target/release/lfz.exe --expected-sha 5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5 --output docs/reports/T19-blackbox-evidence/final-first-permitted.json
```

结果47/47 PASS、exit0，首脚本SHA3be2a83e938345584127b65cae886b8bd1b01d2aea5e6e8623b7f9cea225f05d保持未改；无需新oracle修正或产品修复。原脚本script-first.txt、所有参数/双通道/exit/检查完整保留。

6个关闭case分别EOF/false/callbackError/nonbool/UTF8/tooLong，均unheld_control=true；held_probe=false/error32；after_probe=true/error0；same_pid_alive=true。对照发生在同一LFZ PID尚活且调用返回后，不是进程退出/rename/delete假证据。ExecutionLimit不可捕获导致CLI终止，该关闭证明不混入本6例，仍由同host库证据补足。

所有47 subprocess.communicate完成且无异常，无监听server或后台线程。脚本结束后只读Get-CimInstance按本独占Temp/T19-public命令行过滤LFZ进程，matching_count=0，证据process-cleanup-query.json；未停止他人进程。公开测试已收工，不为后续perf窗口持续跑循环或重复有效测试。

product逐case及末尾核SHA，验后source逐项匹配tooling T19 manifest。manifest-final封存source/binary/script/全部原始结果及旧缺功能红，未编译/改产品/旧测试/dist/Git，无网络请求。684作者harness/本公开47/后续独立与规模各自计数，不冒称RSS/吞吐或原R304已过。仅本轮T19收口，不T20/automation。
