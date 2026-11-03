# T19 大文件内存与吞吐验收准备

2026-11-03，perf-engineer。依据冻结 §25/T19-contract。**仅准备，未运行 LFZ、未生成大文件、未测性能，不影响 R304 原19项门槛或历史成绩。**

## 已完成的可复验准备

`benchmarks/t19/prepare.py --prepare-only` 在项目 Temp/T19-perf 生成4174B/37行小样（含坏JSON/Unicode分布），Python第二次逐行 json.loads 独立重解析，rows/实际bytes/SHA与生成器一致。输出 Temp/T19-perf/prepared.json；product_invocations=0，measured=false。脚本当前故意只接受prepare-only；后续生成/实际测量须另GO，不把计划当实测。所有输出路径限制项目Temp，未增加依赖。

## 固定输入及oracle

后续三档最低10/100/300MiB，生成器逐行写，不先建列表；实际bytes可能比目标多最后一行，必须用manifest实际bytes计算速率。分布沿小样：i%97故意坏JSON；其它id、正负value、active、32..144字节payload及周期Unicode tag。所有物理行LF；max raw行长实测并锁定maxLineBytes=4096。无效JSON由callback内部attempt捕获，坏UTF8/超长/CRLF等独立小功能fixture不混入吞吐样本。

生成manifest后，独立Python第二读计算rows/valid/bad/sum_id/sum_value/active/weighted_mod/first_id/last_id/SHA/bytes/max_raw_line_bytes。LFZ forEachLine严格bool返回true，只累计这些标量，不收集对象或行；逐字段比对，最后输出固定小JSON。数据生成、文件hash、源编译都在计时窗外。文件合计约410MiB，不用无限流。

100MiB为主正确性oracle，10/300MiB看伸缩。readFile+split基线仅先在10MiB相同输入/相同汇总运行，与stream输出字节/字段一致；基线不收集所有解析对象。不得为对照故意OOM，扩到100MiB须先评估内存并记录明确终止上限。此对照不是原R304新线。

## Windows OS峰值与正确进程生命周期

每模式/每规模新进程；绑定最终exe及递归source SHA，运行前后核exe。固定LFZ驱动先print小JSON并flush，然后input(空prompt)作有限测量handshake，父host持process handle，在子进程尚存活时调用GetProcessMemoryInfo，记录PeakWorkingSetSize和WorkingSetSize原始bytes；采集后写换行释放input并wait。测试专属handshake不改产品。若CLI数据管道无法可靠读出marker，先小样验证后停，不以退出后取不到的值填0。PeakWorkingSetSize是OS物理工作集峰值，不能叫精确heap/private bytes；保留启动/解析/allocator高水位，不扣基线冒充真实RSS。

父host只取自己的受测pid（OpenProcess query/vm_read），句柄finally关闭，无系统缓存清理/优先级修改。记录环境/CPU/OS/Rust、进程退出码、stderr和采集错误；任何失败作为失败保留，不删除样本择窗。测量工具使用ctypes Windows只读API不代表产品增加FFI或跨平台性能已经验收。

## 吞吐协议

先每档一次预热，之后5次正式；同任务LFZ/Python流式参考交错。wall采用perf_counter_ns，从启动到输出完成marker（含启动/源解析/打开/读取/JSON解析/回调/汇总），释放测量handshake后另记录总exit时间，不把等待host采集算处理吞吐。每次结果须先oracle正确；保留每轮raw wall/峰值/output/hash/exit，报告中位数及min/max、实际MiB/s和rows/s。标注warm-cache，不声称物理磁盘吞吐或cold-cache。

只有固定最终候选且所有并发编译/测试/本地服务结束后才开测；窗口一旦开始不择轮追绿。样本watchdog先按小样校准为有限宽裕时限；仅清自己子进程。峰值伸缩与回调保留/单行上限的关系如实分析，不凭源码O(maxLine)或一档耗时声称复杂度。任何新绝对内存门槛需team-lead批准，不单方改需求。

## 待最终GO

需要最终产品hash、驱动check/run功能绿、无并发测量窗、Windows查询许可。届时再补实际测量runner并核小样OS峰值/handshake负对照，按本计划生成三档输入与独立oracle、实跑原始结果。当前尚无T19最终二进制或成绩；R304保持OPEN，不外推Linux/macOS、T20或正式发布。
## 2026-11-04 runner已准备，未测量
benchmarks/t19/measure.py、stream.lfz、whole.lfz、reference.py已落。prepare-only实际exit0，product_invocations=0/measured=false，协议绑定候选5d2219f…/1041920B但尚未启动其check/run。每新进程打印恒定小JSON→input空prompt活进程握手→OS PeakWorkingSetSize→父释放input→wait；处理wall截至父收到JSON行，握手采集等待另列总exit时间；Python同任务无hash计时参考，各规模stream/Python交错，10MiB另whole基线，一预热五正式共42样本。每样本输出/峰值/时间/失败逐件落盘，any失败停止保全不择轮，候选hash初末锁定，输入生成/独立二次oracle窗外。
当前只prepare-only/AST通过，LFZ新驱动和OS峰值API尚未实际验证；必须root另GO无并发后先小样驱动check/run+live峰值/握手，再生成10/100/300MiB。没有大档/产品启动/网络/编译/Git，不套684成绩当性能。R304OPEN。测量入口python -B benchmarks/t19/measure.py --run --binary target/release/lfz.exe --expected-sha FINAL --root Temp/T19-perf/window01；不要在其它验收进行时执行。