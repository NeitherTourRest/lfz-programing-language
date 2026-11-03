# T19 tooling/host资源与验收可行性 review

2026-11-03。只读启动状态、角色与现项目约束，查询Rust/Microsoft官方文档；未改产品、未编译、未生成大夹具或宣称实测。基线main c2a3d39，旧readFile/CLI/测试与原用户三块保持。建议只实现本轮callback逐行读取/JSONL，完成交付后收口，不T20/automation。下列语义边界供architect定案，不自行冻结API。

## 实现路线建议

采用标准库File与固定8KiB读取缓冲，行累计Vec在追加前检查长度，按LF扫描分片；或者固定容量BufReader配合fill_buf/consume。不要调用read_line/read_until后才检查maxLine：单个无LF输入可能先累积整文件。标准库说明read_until持续读取至delimiter/EOF且可能持续阻塞，fill_buf/consume可控制逐片消耗。[BufRead](https://doc.rust-lang.org/std/io/trait.BufRead.html)

每个callback只收到已经完整界定且严格UTF8解码的行；多字节字符跨chunk不能逐chunk独立from_utf8。空文件、连续空行、终止LF不多造一行、末行无LF、CRLF/孤立CR、首BOM是否保留、JSONL空行是否跳过、行号从1与坏JSON定位必须由spec明确。建议行限额按剥离分隔符前raw bytes检查（CR/LF/BOM计否需明确），不靠字符数或事后String长度。EOF不能把已超限/非法UTF8半行当成功。N/N+1及分隔符跨8192边界须测试。

读取函数只持有局部File/reader/当前行，不将文件或reader放Value、Env、Rc或callback闭包中。callback通过既有Invoke调度，不另开线程或引入iterator/generator生命周期；callback false/显式停止、业务错误、ExecutionLimit、坏JSON与UTF8、读错误均离开同一资源作用域。每行处理后释放当前数据，不隐式收集结果数组。实际内存界限只能承诺reader内部O(chunk+maxLine)，callback自行保留每行/JSON对象可线性增长，不应称语言整体常量内存。

cancel/deadline在open前、每次阻塞read前后、拼行分片及callback前后协作poll，Interrupted重试也要poll；首reason锁存，主ExecutionLimit不被callback attempt吞掉。native I/O等待不凭空扣AST步；回调按既有求值预算计步。File::open/read/metadata可能阻塞，poll不等于磁盘硬截止。不要为硬时界开线程后detach，也不因本轮有预算就更改旧readFile语义。

建议仅正规文件，opened File metadata确认is_file，拒绝目录/FIFO/设备等输入，减少无EOF源风险；这仍不提供恶意文件系统或网络盘硬返回时界，也不保证并发写/截断的快照一致性。路径symlink/reparse及并发修改政策需spec说明，不承诺沙箱；metadata不能代替实际读取字节计数。是否普通文件限定请architect裁决，不能实现者私自缩scope。

## 资源关闭承诺：标准库足够，但不能假称close可报告

File离开作用域自动关闭，Drop忽略关闭错误。推荐本轮采用标准库RAII并明确这一边界；只读文件不涉及写回持久化承诺，无须为“可释放资源”先扩Windows/POSIX生产FFI。sync_all是同步数据接口，不是一个返回close结果的替代，不应为只读流水线额外调用它来宣称关闭无误。[File](https://doc.rust-lang.org/std/fs/struct.File.html)

若spec要求每次close错误进入diagnostics/HostFailure，std File的Drop不足以履约，必须另审跨平台显式close接口、raw handle唯一所有权、失败后是否可重试与主错误保持；不沿用HTTP/exec文案伪造保证，也不偷偷mem::forget或unsafe close后让File再次Drop。建议本轮冻结为“所有正常Rust返回路径RAII关闭；标准库不暴露关闭失败”，把强close诊断当另一个明确工程选择，不自行扩FFI。panic=abort、进程强杀与OS异常不承诺Rust析构运行。

## 如何证明关闭，避免无效测试

Windows标准库默认共享READ/WRITE/DELETE，打开时也允许rename/delete，因此“回调早停后删除成功”单独不能证明句柄已释放；Unix unlink打开文件也成功。[Windows OpenOptionsExt](https://doc.rust-lang.org/std/os/windows/fs/trait.OpenOptionsExt.html)

Windows强见证建议使用标准库OpenOptionsExt::share_mode(0)排他重开同一个已知正规文件：在读callback内探针应SharingViolation，在读取函数早停/EOF/错误返回后同一探针应成功。必须先做活句柄负对照证明该探针确实能检测泄漏，关闭所有夹具自身句柄，不能把权限失败当预期sharing失败。探针不能修改生产打开共享策略。之后rename/delete只作为夹具清理及辅助信号。

另在同一长期宿主进程预热后跑1000次早停/坏UTF8/超长行/回调错误并分批观测handle count，测平台稳定平台期，不把启动后一次计数比较误称精确泄漏检测。Windows GetProcessHandleCount为进程总句柄数，线程/运行时缓存影响需负对照与重复批次解释；test harness使用Windows FFI取计数不代表生产增加FFI。[GetProcessHandleCount](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesshandlecount)

跨平台用私有generic Read单测包装资源Drop计数，构造第K字节失败Reader和Interrupted→正常流，断言已有callback次数/行序/首错与资源Drop恰一次；这是合成错误路径，不是假称真实磁盘硬故障。系统终止用callback marker后宿主另线程置token，步数宽裕作兜底、reason必须Cancelled；不用sleep猜碰巧发生。早停后不访问后续坏UTF8/超长尾部是否应成功需由spec明确，prefetch取得bytes不等于已交付/验证这些行。

## 有限大文件与独立oracle

全部夹具项目Temp/T19内；不使用外装/公网，不建立无限流。建议主JSONL约100MiB，附10MiB与300MiB同分布扩展作为内存伸缩观察，总磁盘预算低于1GiB；先算准确字节/最大行长，再定行数生成。用Python标准库以固定seed/编号逐条写入，不在生成器里先建百MB列表。字段含id、正负整数、bool、ASCII与Unicode/转义字符串、不同长度payload；每条一行，JSON紧凑编码，明确ensure_ascii和LF策略。

生成时独立累计rows、sum(id)、选定谓词计数、加权模校验、首末id、文件SHA256与实际bytes，写manifest。最终oracle再用Python逐行json.loads重新读独立计算，不能直接拿生成器自报值当唯一oracle；算法依据字段值，不能复制LFZ结果。LFZ只输出固定小汇总结构，逐字段严格比对，不把吞吐测试输出百MB终端。N条早停附正确前缀oracle与未处理尾部毒行，证明不读全文件后才callback。

功能夹具另覆盖空/空行/CRLF/无末LF、UTF8四字节跨chunk、BOM政策、N/N+1、坏UTF8/坏JSON第K行、超长无LF、读错、callback错误、嵌套读取与跨module来源；这些小而有意的夹具不混入大文件吞吐样本。原黑盒与旧readFile差分由test-engineer保全，不能删改原断言。

## 峰值内存与吞吐：测法可行，不先报成绩

Windows建议独立新进程运行每个文件规模，父host保留进程handle并在受测程序完成工作、退出前的有限handshake采集GetProcessMemoryInfo。记录PeakWorkingSetSize（bytes，物理工作集峰值）与当前WorkingSetSize；必要时另采样private/commit指标，不能把它们称同一个RSS。使用OS峰值优于低频采样容易漏短峰；进程结束后取不到计数不能以0补报。[PROCESS_MEMORY_COUNTERS](https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters)

同脚本/同schema/同maxLine的10/100/300MiB：先验证oracle正确，再检查峰值是否随文件规模近似平稳，固定callback仅保计数；若大规模峰值线性上涨先查回调Env/Value/trace是否累积。解释器启动/解析及allocator高水位属于进程峰值，不应扣掉后声称真实RSS；可以旁列空输入基线。大单行场景另测，不能用短行常量内存证明maxLine无关。

吞吐使用单调时钟（Python perf_counter_ns或Rust Instant），明确总wall包含启动/源解析/打开/读取/JSON解析/callback/汇总输出，生成与hash校验在计时外。建议一次预热后5次正式，LFZ/Python参考同任务交错、原始每轮时间保留，报告中位数与范围、actualBytes/(1024^2)/seconds MiB/s及rows/s。warm-cache结果必须标注，不能清系统缓存/把测得速率说成物理磁盘带宽。Python对照只作同任务参考，不替代原19项性能/R304门槛。

每次受测进程有限watchdog（按先小样本校准的宽裕值），读/写输出有界，收尾wait所有测试进程与probe句柄。保存binary/source hash、Rust版本、OS/CPU、输入SHA/bytes/rows、CLI、各轮stdout/exit/耗时/峰值/失败原件。未在Linux实跑则只说标准库路线可移植，不称跨平台性能或资源测试通过。

## 交给architect/root的待定项

建议先冻结callback停止信号/返回值、行号/分隔符/BOM/JSONL空行语义、maxLine raw口径、正规文件范围、total计数与早停尾部行为、读错误时已有副作用、cancel优先级和close标准库边界，再GO源码。当前无必要增加CLI开关、新依赖、生产FFI或HTTP/exec权限联动。review可行性完成，尚无T19产品或运行成绩。
