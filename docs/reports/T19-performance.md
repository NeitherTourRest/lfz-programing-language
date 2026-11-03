# T19 大文件逐行处理：最终实测

2026-11-04，perf-engineer。单一正式窗口window01，候选1041920B，SHA `5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5`。**42/42样本oracle/exit/hash通过，未择窗重跑。R304仍OPEN，本报告不替代原19项门槛。**

## 输入、窗口与测量口径

10/100/300MiB同周期数据分布，按实际bytes计算：每97行一坏JSON，ASCII/Unicode及不同长度payload；最大raw行218B，callback cap4096。生成与Python二次逐行oracle/hash在处理窗外，LFZ只累计9个标量。stream与Python每档各1预热+5正式；10MiB另同任务readFile+split基线，合42（7预热+35正式）。全部原始结果和输入manifest保留。

|档位|实际bytes|行数|坏JSON|SHA256|
|--|--:|--:|--:|--|
|10MiB|10485812|68018|702|3e4197b3cb2c99d8a2fb463a9a3773be14dc3db376991a8b151021fa2ecd6b3d|
|100MiB|104857726|675779|6967|75ddb6254cea3df713ed4d00354bf49ea7de54655b001f2b9ae06985de977ecd|
|300MiB|314572899|2019323|20818|7f52ab37ded7a89f385d523d30016a906000795fa2a0f71cc2ef335cf87ad995|

每次新进程，print恒定小JSON后input空prompt握手；父进程在子仍存活时GetProcessMemoryInfo取OS PeakWorkingSetSize，再释放stdin并wait。处理wall为启动到父观察JSON marker（包含源解析/文件读取/JSON解析/回调/序列化及观察调度），OS采集等待不计处理wall，另存总exit wall。峰值是物理工作集OS高水位，不是heap/private bytes；不扣启动或allocator基线。

环境：Windows11 10.0.26200，Intel64 Family6 Model183 Stepping1，24逻辑CPU，物理内存16873545728B；Anaconda Python3.13.9。全部输入已生成并oracle/hash读取，warm-cache结果，不称cold-cache或物理磁盘速度。root确认无并发LFZ测试/编译，测量窗串行。

## 全部正式结果

|模式|档位|wall中位秒|min..max秒|中位MiB/s|峰值WS min..max MiB|
|--|--:|--:|--|--:|--|
|stream|10|0.345100|0.333499..0.397043|28.977|6.484..6.500|
|python|10|0.284388|0.265369..0.302399|35.163|14.613..14.766|
|whole|10|0.376632|0.370604..0.378348|26.551|34.863..34.898|
|stream|100|3.081447|2.920035..3.428331|32.452|6.496..6.512|
|python|100|2.039712|2.017833..2.119376|49.027|14.828..14.984|
|stream|300|9.335709|8.775660..9.582360|32.135|6.492..6.500|
|python|300|6.244406|5.904254..6.719328|48.043|14.836..15.078|

固定聚合驱动下，stream三档峰值约6.5MiB、未随输入10→300MiB线性增长；10MiBwhole约34.9MiB。300MiB stream中位9.336s，Python6.244s，LFZ在此任务约慢49.5%，不声称整体吞吐领先。大文件短行仅证明此固定callback分布，不证明任意callback或maxLine无关的常量内存。

## 驱动修订与控制证据

root在正式测窗前静态发现初稿每行attempt(()=>jsonParse(line))创建UserFn，现func_names保留metadata，任意callback可导致线性增长。原stream/whole全文及SHA保stream-initial.txt/whole-initial.txt；它们未跑正式窗口，不虚构旧驱动内存失败。按明确指令改currentLine={text:""}共享对象cell、一次parseCurrent闭包，consume仅赋text并attempt(parseCurrent)，不改产品funcID规则。正式协议锁定修后驱动SHA。不是测量后择绿窗口。

实际小样4174B/37行stream与whole均check/run通过、输出等二次oracle，均liveOS峰值约6.5MiB。负对照Python触碰64MiB后释放：OS peak82317312B/current15241216B，差>32MiB，证明不是把当前RSS采样冒作峰值。原始pilot结果、cmd与stderr齐。Get-CimInstance环境查询权限失败未影响测窗，保null初失败记录，后采用只读环境变量与GlobalMemoryStatusEx；未修改产品/系统配置。

## 证据与限制

benchmarks/t19/{prepare.py,measure.py,stream.lfz,whole.lfz,reference.py}；T19-performance-evidence/window01/{protocol,environment,all-samples,summary,data-*-manifest,每样本JSON/stderr}；pilot01与两初稿。大型数据仅Temp约410MiB，可按已封generator+manifest重建，未纳Git；独立oracle从数据字段二次解析，不从LFZ结果反推。

没有watchdog触发、错误输出、oracle失败或产品重建；候选初末SHA一致。没有Linux/macOS实测、强磁盘IO截止、OS close错误可报告、原19项效率达标或正式发布结论。仅T19本轮，后继由root另行决定。
