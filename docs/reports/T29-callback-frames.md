# T29 高阶函数回调与性能验收

T28 已消除循环中反复追加数组的主要复制开销，正式窗口仍有两项明确超线，两项都涉及高阶函数回调。T29 复用一次高阶调用内的参数帧，减少解释器在处理每个元素时重复分配槽位和复制参数名的工作。[T28 诊断](T28-array-append.md)

## 调用与闭包

以 `map((x) => () => x, [11, 22, 33])` 为例，返回的三个函数各自记住当次的 `x`。T29 在回调成功返回后清空参数帧中的值和共享单元引用。逃逸闭包仍持有原单元，下一次调用会为自己的参数建立新绑定。嵌套的高阶函数各自持有缓存，错误路径保留原调用栈。仅在帧独占、没有弱引用、槽位及名字表容量至多各 64 项且参数名缓冲合计至多 4 KiB 时保留空帧。模块回调按定义模块重新确定父作用域和槽位起点。[源码归档与逐项哈希](../../benchmarks/t29/results/candidate-snapshot.json)

## 功能验证

- 726 项 Rust release 测试、182 项 LFZ 黑盒及 17 套既有回归通过。四项新增 Rust 测试覆盖闭包捕获、错误后的恢复、模块调用栈、空帧释放旧值。[Rust 日志](../../benchmarks/t29/results/rust-release.log) · [黑盒](../../benchmarks/t29/results/blackbox.json) · [回归命令与结果](../../benchmarks/t29/results/regression-commands.json)
- 37 项 REPL、27 项格式化、9 项 JSON 路由、6 项交互式 JSON 工具案例通过。人类指南 3 项与 AI 指南 4 项均以候选实跑，包含预算耗尽时的退出状态。[REPL](../../benchmarks/t29/results/repl.json) · [格式化](../../benchmarks/t29/results/formatter.json) · [JSON 路由](../../benchmarks/t29/results/jsonroute.json) · [交互工具](../../benchmarks/t29/results/jsondesk.json) · [人类指南](../../benchmarks/t29/results/guide.json) · [AI 指南](../../benchmarks/t29/results/ai-guide.json)
- 独立验证设计十组程序，对比 T28、T29 的退出码和两个输出通道，十组通过。首次夹具用了保留字，第二次默认环境读取模块失败，两次原始记录均已保存。[独立验证与复现命令](T29-independent-verification.md)

## 固定输入的同机对照

T28 和 T29 在同一个可执行文件路径交替运行。六个负载各预热一次，每个版本再正式运行六次，合计 84 次启动，输出全部正确。完整数组与字符串映射每个进程内各重复 16 遍，耗时包含启动、解释和输出。两项须同时降低中位数、各自至少赢四轮。表中的胜出按同轮配对计数。[预声明](../../benchmarks/t29/results/screen-protocol.json) · [全部 84 个样本](../../benchmarks/t29/results/screen.json)

| 负载 | T28 中位数（ms） | T29 中位数（ms） | T29 胜出 |
| --- | ---: | ---: | ---: |
| 完整数组管道，10,000 项 | 92.85 | 49.52 | 6/6 |
| 其中 map 与 filter | 74.21 | 40.52 | 6/6 |
| 单独测 sort 与 reduce，含一次输入准备 | 32.76 | 20.77 | 6/6 |
| 完整字符串映射，2,000 项 | 37.43 | 28.91 | 6/6 |
| 空循环与打印对照 | 10.53 | 9.38 | 3/6 |
| 数组连续追加，8,000 项 | 14.65 | 11.18 | 4/6 |

两项完整负载中位耗时分别下降 46.7% 和 22.8%。两项回退对照均符合预定条件。这组对照衡量固定程序的整体耗时，未单独测量内存分配数或峰值内存。

本次实测的 Windows 候选为 1,137,664 字节，SHA256 为 `f0d483f7a0cbf2d9c45da5bd684327f8ac177ac4c2378e23230eba86d29a4ed9`。归档内附解释器源码与 Cargo 文件。[下载候选及源码](../../benchmarks/t29/results/candidate-snapshot.zip)

## 原 R304 正式窗口

冻结的 T13 协议以同一份 T29 可执行文件运行一次，共采集 152 个预热样本与 532 个正式样本。19 项原绝对门槛全部通过，所有进程的退出与输出均符合预期。此前 T28 明确超线的两个输入，T29 的中位数为 11.0637 ms（10,000 项数组管道，原上限 22.7717 ms）和 10.0209 ms（2,000 项字符串映射，原上限 17.5219 ms）。8,000 项结构体负载为 15.2285 ms，较原基线 89.4598 ms 降低 82.98%，达到至少降低 30% 的要求。此处每个进程执行正式协议规定的工作量，与上表的重复 16 遍对照分别计量。[原始窗口](../../benchmarks/t29/results/gate.json) · [产物及源码回执](../../benchmarks/t29/results/gate.receipt.json) · [原门槛复算](../../benchmarks/t29/results/gate-assessment.json)

为方便异机复核，性能输入包保存本次实际使用的基准脚本、冻结协议、原始基线数据和筛选预声明的字节，路径按仓库根目录组织。[输入包](../../benchmarks/t29/results/benchmark-inputs.zip) · [逐项 SHA256](../../benchmarks/t29/results/benchmark-inputs.json)

独立复核确认这份候选通过 R304。九项 JSON 与 CLI 能力的输出、退出码符合预期。按原启动占比和净时规则，JSON 解析的两个相邻输入翻倍区间比值为 2.18 和 2.05，均满足 3.0 的上限。JSON 序列化只有 8,000 项满足净时估算条件，因此三档分别报告总时。验收基于本次 Windows 整体进程计时；独立复核读取仓库归档即可重算。[独立报告](T29-independent-verification.md) · [离线复核入口](T29-independent-evidence/offline-audit.py)
