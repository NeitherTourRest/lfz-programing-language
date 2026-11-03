# T19 独立功能验收（应用oracle待修）

最终5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5/1041920B，递归源/Cargo/测试初末冻结。十二harness684/684实际全过；旧13/117/CLI91含178/apps37+11/jsonbatch38/JSON140、HTTP49/httpplan7、A45/toolplan7、T17公开134/app40、C29/app33全过。新公开47/47独立完整通过。仅原harness内部项目fixture构建获授权，没有产品编译；正常OS许可避免重复已知沙箱C0000008，旧失败不删。

自编own.py实际8通过：跨8KiB UTF8、CRLF/BOM/EOF末CR、false跳后坏UTF8、JSON连续attempt恢复；四个同PID仍活着的EOF/UTF8/TooLong/外attempt完成后排他重开通过，不以退出后打开证明close。deadline/取消与负控、动态行错误、metadata/open/read注入和Drop spy、共享预算/home/source由同684内新增unit与公开47实际覆盖。原native仅cancel-deadline不扣步，统一Outcome finish后返回，RAII不报告OSclose错误，TOCTOU/阻塞非硬保证。

独立见证初稿误用push参数顺序、未赋其新数组和Result错误class字段，保留own-initial.py/own-push-order-only.py/own-class-oracle.py，仅修作者oracle后8通过，无产品修复。小文件不代性能实测。

应用JSONLstream首独立复跑失败保留stream14.json：empty exit0/stdout唯一状态成功，stderr应用显式print汇总JSON，作者新脚本assert not stderr与旧CLI --json输出协议冲突。已即报root/app，不改作者脚本/产品，等待修新oracle后完整14重跑，暂不最终放行。

所有我方功能进程/loopback服务已结束，无大数据生成或性能测量。性能RSS/吞吐待perf独占窗口；R304OPEN，非Windows资源未实测，当前功能不等于性能验收。报告与独立证据manifest可复现；本轮T19收口不T20/automation，无Git/源修改。

## 最终应用复验后功能PASS（2026-11-04）
作者最终oracle按旧CLI协议成功场景用普通run并精确核应用JSON，错误场景保--json状态；独立保留stream14.py及stream14.json首失败，复制最终脚本仅改ROOT/独立Temp为stream14-final.py，完整实际14/14 PASS。app源码SHA2e209be7fd74c2e0a53bbcf6bc651da78fbbbf6faf7930db2ff879a20ce8210e核匹配，产品源仍5d2219冻结。应用使用固定parseCurrent闭包和可变text cell，不以逐行新闭包隐藏函数元数据增长。global0仅证明初始化前KEEP保留，不冒称回调中途终止；callback全局终止和samehost关闭来自同harness/公开47证据。
结合684/旧全回归/公开47/独立8，最终独立功能结论PASS；没有性能测量/大数据生成，RSS吞吐仍待perf独占实测，R304OPEN。全部我方测试进程/服务已结束，可交root协调perf窗；不T20/automation。

## 最后规模结果与指南独立审计（2026-11-04）

规模结果审计PASS：perf最终manifest111个文件实际bytes/SHA全部逐项核对，报告与协议驱动SHA一致。正式只有window01，42原始样本=七组各1预热+5正式（7warm+35formal），全部exit0/ok/无watchdog/无额外stdout；七组中位秒与每样本OS peak数组独立重算匹配。三输入实际字节与SHA流式核对，10/100/300MiB档位真实大小按manifest，不以整数档名代bytes。GetProcessMemoryInfo在marker时子进程仍活着才查询OS PeakWorkingSetSize，处理墙时含启动到父观察结果marker，握手hold等待不入处理窗；总退出另列。warm cache、无清cache，不当物理磁盘速度。

独立自写Python流式再解析100MiB实际输入104857726B/SHA75ddb6254cea3df713ed4d00354bf49ea7de54655b001f2b9ae06985de977ecd：675779行、668812有效、6967坏JSON、sum_id225984482614、sum_value-399120、active222937、weighted_mod717808803，与manifest以及正式所有相应LFZ/Python输出一致，maxraw217B。此次为功能oracle核验，未启动LFZ、未新开性能窗、未拿耗时筛选成绩。scale-audit.py/json完整保留。

64MiB触碰释放负控原始OS peak82317312B/current15241216B差值核到，说明没有把marker当前WS当峰值。固定闭包驱动三档stream中位0.345100/3.081447/9.335709秒、峰值约6.5MiB，10MiB whole约34.9MiB；300MiB Python6.244406秒更快，LFZ约慢49.5%。此分布固定callback的结果不推广为任意脚本恒定内存或吞吐领先，不构成旧19项R304通过。

原stream/whole逐行创建UserFn初稿已保全，正式protocol锁定测窗前静态修订后的currentLine cell/一次parseCurrent闭包SHA，独立核现driver一致。不把未实跑初稿虚构成内存失败对照，不把源码O(line)代实测；UserFn诊断metadata保留仍是任意callback内存边界。环境CIM权限初失败与fallback原证据也保留。根据正式protocol、环境回执、原始全样本及owner/root交接审计无另选窗口证据；不冒称独立观察到了历史每个调度瞬间。

指南仅只读作者证据审计PASS，不是独立产品重跑：人21场景42check/run、AI45全部实际结果与manifest原字节SHA核对，六人类+五AI当前11文档SHA匹配最终证据。人首次新oracle缺换行SyntaxError、第二轮/最终42及原逐行闭包文本、AIprepare快照全部保留。文档正确说明严格bool/rawCRcap/坏行恢复/全局终止/RAII OSclose错误不可见/TOCTOU、固定解析闭包与任意callback可增长边界，无新可信TLS或跨平台/性能达标冒称。guide-audit.json、scale-final-manifest-audit.json记录。

最终功能及本轮规模结果审计完成；R304继续OPEN，不正式发布背书。全部我方工作结束，无产品编译、源修改、Git或活产品/服务进程，不T20/automation。
