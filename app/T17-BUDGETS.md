# T17 应用执行预算：Agent运行现有四模块JSONL任务

日期2026-10-30，app-dev。依据三spec §22/T17-design。不修改现有应用、旧oracle或语言实现；本节为最终candidate待验的使用说明。

## 实用命令

从仓库根目录执行，数据与输出放项目Temp，先建输出父目录：

```powershell
./dist/lfz.exe run --max-steps 100000 --timeout-ms 60000 app/jsonbatch_modules/main.lfz -- app/fixtures/t15/mixed.jsonl Temp/T17-app/result.json
./dist/lfz.exe run --json --max-steps 0 app/jsonbatch_modules/main.lfz -- app/fixtures/t15/mixed.jsonl Temp/T17-app/result.json
./dist/lfz.exe run --json --timeout-ms 0 app/jsonbatch_modules/main.lfz -- app/fixtures/t15/mixed.jsonl Temp/T17-app/result.json
```

第一个是小夹具的宽裕预算，不是任意输入都足够的保证。后两条对非空程序在首节点退出2，`ExecutionLimitError`分别为“执行步数预算已耗尽”“执行时间上限已到”；程序业务JSON与CLI状态JSON仍按原stdout/stderr规则区分。预算选项须在脚本参数分隔符`--`之前，只支持两个token，不能用等号；check只前端检查不接受执行限额。

步数是实际进入语句/表达式的节点预算，包括模块初始化与跨模块回调，不是JSON记录条数、CPU指令数或内存配额。共享模块缓存不重复执行初始化；整个会话共享预算，attempt不会补步或吞系统终止。正常业务坏JSON仍可恢复逐行继续。

## 副作用与协作式timeout

预算在节点开始前检查。此应用在所有解析和编码完成后才writeFile，因此在首模块初始化阶段终止时既有输出保持；不能据此承诺任何预算失败都保留输出。如果writeFile已经完成后在print或其它节点终止，已写输出不回滚。

专属验证器另用导入实际common模块的临时入口先写SIDE，再进入有限预算的无限循环；应受控ExecutionLimit终止且SIDE仍在，证明确无事务回滚。该文件是测试专属Temp哨兵，不影响实际用户输出。

`--timeout-ms`从前端通过后建立截止时间，只在语句/表达式入口协作检查。加载/解析/图构建不在窗口内；正在执行的JSON解析、文件IO等宿主调用可能持续到下一节点。不是强杀线程、硬实时超时、权限或内存沙箱，也不新增CLI取消token接口。

## 验证准备

`app/verify_jsonbatch_budgets.py`的`--prepare-only`只核四模块源码指纹，启动解释器0次。AST自检已通过；等待root通知最终hash后才执行：

```powershell
D:/app/ANACONDA/python.exe -B app/verify_jsonbatch_budgets.py --lfz target/release/lfz.exe --expected-sha FINAL_SHA256 --output Temp/T17-app/final.json
```

范围：不限额/足够步数/足够时间/组合四次结果逐字节同；零/一步预算、零超时与两者零值优先级四次系统终止且KEEP输出保留；已完成IO不回滚；真实record模块回调预算穿透；check拒绝执行限额。回调见证以0..128逐一尝试预算，保留每个样本，找到错误file=record.lfz且最内匿名JSON回调的位置；这是可重复的节点边界探测，不是挑快性能样本。错误仍沿跨文件JSON诊断，attempt返回记录本身不新增file字段。

运行中每个进程前核产物hash，执行后核旧模块源码没改。结果实时保存，失败也保留行与stdout/stderr；15秒测试watchdog不作为CLI timeout精度证明。若沙箱拒绝canonicalize，保留原始结果并报告，许可后才用同字节提权重跑，不修改产品绕过。

尚未实跑最终候选，不宣称通过或性能收益；原R304仍OPEN。旧T15/T16应用与oracle保持原样，未编译或Git。

## 最终实跑（2026-10-30）

team-lead冻结最终T17产物：target/release/lfz.exe 943104B，SHA256 `1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45`。核hash后默认sandbox首次unlimited模块访问/canonicalize拒绝，exit2/IOError；[失败证据](t17-evidence/default-permission-failure.json)保留，没有改产品绕过。同一精确产物获许可提权原命令，**40/40通过，exit0**，[原始结果](t17-evidence/results.json)逐进程保全。

40项为4次正常结果等价、4次早期限额错误并保留KEEP、1次已完成IO不回滚、1次不限额probe最终AssertionError、29次预算0..28探测、1次check拒限额。预算28明确在真实record.lfz匿名JSON回调内部终止，最终CLI仍为ExecutionLimitError而非attempt失败记录，证实资源终止穿透；所有前置预算也保留，未隐藏任何探测读数。

正常四模块应用在unlimited/100000步/60000ms/组合配置下，结果输出逐字节一致（3条、2成功、1坏JSON，公共模块计数3）。0/1步与0ms均在common真实模块位置报告受控终止，输出KEEP字节不变；两者均0时Deadline按规定优先。临时探针已写SIDE后预算耗尽，SIDE保持，不能把早期输出保留解释成所有错误会回滚。错误file/line/col和traceback按既有跨文件协议验证；T15 Result仍不增加file。

原四模块源码SHA在运行前后相同。测试未修改旧应用/旧oracle、产品源码或配置；未编译/Git，也未额外复制二进制。15秒外部watchdog仅用于测试防挂，不当作协作timeout精度保证。R304仍OPEN，40项是应用功能见证，不是性能达标或正式发布批准。
