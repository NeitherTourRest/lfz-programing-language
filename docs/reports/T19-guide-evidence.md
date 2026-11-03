# T19 人类指南最终实跑

状态：最终候选小样21场景、42个check/run进程通过；这不是独立全量验收或性能结论。

二进制：target/release/lfz.exe，1041920B，SHA256 `5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5`。每个进程前核hash，结束核hash及六份指南SHA。未编译、改产品或Git，无网络或大文件。

事实来源：三spec §25与正式T19-contract.md。五指南增加入口，line-processing.md给出固定解析闭包JSONL data-last示例、严格bool、rawLF前bytes含CR、末CR/BOM/empty/tailLF/UTF8、恢复差异、共享预算/native协作poll、普通文件TOCTOU与std Drop关闭错误不可见边界。旧readFile不变，76内置/13错误类。

命令（从项目根）：

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T19-guide-evidence/verify.py --prepare-only
D:/app/ANACONDA/python.exe -B docs/reports/T19-guide-evidence/verify.py --binary target/release/lfz.exe --expected-sha256 5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5 --output Temp/T19-docs/final-docs.json
```

prepare-only exit0，提取脚本并生成12个小样，executed:false；最终exit0，42进程全部通过，stdout/stderr/exit/argv与过程耗时在runs.json。正常默认文件权限即可，不需提权；所有子进程subprocess.run同步回收，timeout15s，无服务器。Python宿主既有“Failed to find real location…”提示保留，不是LFZ失败。

首次运行保全：first-failure.json含40个通过进程以及第41个budgetNotCaught/check的真实SyntaxError。verify-first.py保留初版oracle：while块与return之间缺换行，违反原语法。只修验证器换行，不修改产品或期望错误类。line-processing-first.md保留原先逐行新建解析闭包的指南。随后按root提醒改为外部currentLine cell与一次parseCurrent闭包，第二次42进程通过，second-runs.json保留；最终文字加入实际输出后再锁指南SHA验证，runs.json为最终结果。

场景：空、CRLF/末CR、两空行、尾LF、BOM、Unicode字节N/N+1、NUL内容、空白/#42数据、跨8KiB、坏UTF8、早停不处理后坏行/长行、严格bool、空文件错arity、cap范围、外层attempt副作用与全局步数不可吞。JSONL实测accepted2/rejected2/sum3/lines4，固定解析闭包代码从教程提取，不另写替身。

内存限制：reader的O(当前行+8KiB)不等于任意callback整体内存有界；func_names可随新建UserFn增长，故教程复用固定闭包。小样未测吞吐/峰值，不将此脚本当规模证据。

未在本指南声称验证host取消/硬时间、OS close错误、TOCTOU攻击、跨平台、排他重开/handle plateau、16MiB或大规模峰值；这些须对应owner证据。原始数据与各文件SHA由manifest.json绑定，指南SHA见runs.json及manifest。R304OPEN，本轮T19后收口，不T20/automation。
