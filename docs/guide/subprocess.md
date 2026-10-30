# 显式启用受控子进程

本章以三spec §23和T18-A-design为准；本轮仅Windows后端已验，非Windows返回受控Unsupported。HTTP不属于本轮，完成本轮交付后停止，不延伸新迭代。最终示例字节与结果见[T18-A指南证据](../reports/T18-A-guide-evidence.md)。

## 先check，再显式授权run

保存为tool.lfz：

```lfz
#42
let result = exec([argv()[0], "echo", "", "a b", "雪"], 1000, 4096)
assert(result.exitCode == 0)
assert(result.signal == nil)
assert(result.stderr == "")
print(jsonStringify(result))
```

此例使用项目的Rust测试fixture，模式echo按UTF8字节长度打印参数；它不是通用系统命令。在仓库根目录，以最终候选替换系统旧PATH解释器：

```powershell
./target/release/lfz.exe check tool.lfz
$fixture = (Resolve-Path 'Temp/T18-A-tooling/fixture.exe').Path
./target/release/lfz.exe --allow-process run tool.lfz -- $fixture
```

check不执行exec且不接受--allow-process。真实应用把示例fixture换成明确选择的绝对.exe及其argv；不会搜索PATH、补扩展名或隐式拼接shell。首参是非空array[string]，程序必须Windows非设备盘符绝对.exe或UNC.exe，拒绝相对/盘符相对/.bat/.cmd/NUL。参数中的空串、空格和Unicode保持，Windows遵循标准CRT命令行解码，不宣称内核存在argv数组。用户显式选择绝对shell仍可能执行shell，接口不把该选择自动变安全。

exec恰三参：argv、timeoutMs整数1..600000、maxOutputBytes整数0..16777216。权限默认禁用，--allow-process仅当前运行session有效，不跨调用泄漏；所有模块共享该session权限，函数home不携带权限。exec不是新增一等函数值，原遮蔽规则保持。内置总数74，错误类仍13。

成功结果四键按stdout/stderr/exitCode/signal插入；jsonStringify仍按自己的UTF8键序输出。Windows exitCode为u32无符号扩到i64、signal=nil；非零退出是正常结果，调用者自行决定是否失败。子进程stdout/stderr作为字符串返回，不直接转发父进程终端；stdin为EOF/null，继承host环境和cwd，无env/cwd/stdin参数及隐式重试。

## 输出上限与错误恢复

输出上限按stdout+stderr合并原始bytes，不是字符个数；=N允许、>N失败，0允许无输出命令。两stream公平读取，每stream顺序保持，不承诺跨stream总顺序。任一流非UTF8则整体ValueError“子进程输出不是合法 UTF-8”，BOM/CRLF不归一化，不返回部分成功。缓存上限不是子进程自身内存、内核pipe或OS总内存限制。

局部timeout覆盖运行和drain，错误为可被attempt恢复的ValueError“子进程执行时间上限已到”。输出超量、禁用、非法参数、Unsupported等约定ValueError也可恢复；OS操作失败为带动作名的IOError。请用 `attempt(() => exec(args, 1000, 4096))`，不要先执行exec再交给attempt。

会话全局deadline/取消仍ExecutionLimitError，不被attempt吞掉；exec等待不扣AST步数，remaining==0不会自行变StepBudget。观察优先cancel→全局deadline→局部timeout→合并输出超量→OS错误→完成，同一轮可观测优先不代表并发事件的绝对顺序。创建进程的OS调用不保证硬中断，等待目标轮询也不是硬实时承诺。

## 权限选项和CODE/argv保护

--allow-process是无值选项，重复报“选项 '--allow-process' 不能重复”；任何--allow-process=文本报“选项 '--allow-process' 不接受值”。只适用于运行命令；check/test/--help/-h/--version/-V报“子进程能力选项只适用于运行命令”。裸help/version仍按文件解释，不是新别名。

-e之后紧邻CODE以及完整执行来源后-- argv原样保护：`lfz -e 'print(jsonStringify(argv()))' -- --allow-process`不会启用能力。合法执行选项可与T17限额组合；非执行命令混合能力/限额时，以选项区首次出现的新类别决定适用性文案，--json不算类别，扫描遇缺值/非法/重复/等号仍立即按token序报错。usage即使--json仍stdout空/stderr文本/exit2。

## 回收诊断与平台边界

Windows后端先挂起创建并纳管Job，成功后恢复；正常或异常返回均应结束受控后代、回收直系和reader。纳管失败不降级裸spawn，不允许用户代码先运行。不是恶意进程/权限逃逸沙箱，内核不可中断或OS回收失败不能承诺绝对回收时界。

回收诊断独立于LFZ错误：result Err保原错误，stderr另附“lfz: 子进程回收诊断：”；result Ok但有诊断时CLI stdout空、stderr以“lfz: 子进程资源回收失败”开头、exit2，哪怕--json也属于宿主异常例外，不伪造业务JSON成功。attempt不能清掉诊断；库宿主不能只看result Ok，必须检查TracedRun.host_diagnostics()。

Rust EvalContext{limits,capabilities}及三个owned context入口保留旧EvalLimits形状；旧API默认subprocess:false，新权限不使用global/thread_local。API精确签名见interface-contract §23，本章不把CLI示例当Rust宿主编译或取消token验证。未经实际平台验收，Linux/macOS等不启用后端，不能从Windows通过推断跨平台支持。R304OPEN，不宣称性能验收或正式发布通过。
