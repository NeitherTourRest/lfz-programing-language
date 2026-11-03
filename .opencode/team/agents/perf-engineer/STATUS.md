# perf-engineer — 工作状态
> 更新：2026-11-04 by perf-engineer

## 当前状态
T19固定最终候选规模测窗完成，42/42样本通过，测量进程全部结束；已通知root释放无并发窗口。停止产品调用/数据生成/测量。R304仍OPEN，不替代原19项门槛。

## 最终证据
- 最终1041920B SHA5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5初末一致。
- docs/reports/T19-performance.md及T19-performance-evidence/manifest.json，111个证据文件/源码驱动SHA；window01单窗7预热35正式共42，10/100/300MiB二次Python oracle全通过，无stderr/非零退出/watchdog/择窗重跑。
- LFZ stream三档median0.345100/3.081447/9.335709s，峰值WS约6.5MiB平稳；10MiB whole约34.9MiB。Python同任务更快，300MiB median6.244406s，LFZ约慢49.5%，不声称吞吐领先。
- 修后单parseCurrent共享cell避免每行新UserFn；初驱动原文/SHA保留，静态修前没有正式测量，不虚构失败。小样stream/whole check/live峰值握手过；64MiB释放负对照OS peak82317312/current15241216通过。

## 边界与下一步
warm-cache/含startup至父观察marker，observer握手等待单列；物理工作集OS peak非heap/private/采样RSS。数据约410MiB仅项目Temp，manifest可重建；所有原样本保全。Get-CimInstance环境查询失败保留，改只读环境与GlobalMemoryStatusEx记录；非产品故障。无产品修改/编译/Git/系统配置/外装/公网。仅T19，等待独立审计或root交付，不T20/automation。