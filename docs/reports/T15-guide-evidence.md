# T15 人类指南聚焦实跑

日期：2026-10-29。责任人：docs-writer。

五指南已同步73内置与attempt，新增error-recovery.md可复制混合好坏JSON教程。所有行为只引用三spec §18：固定Result、可恢复白名单、非捕获资源错误、外部校验与求参边界、trace字段、无file、副作用不回滚、普通内置仍非一等值。class保留字使用索引，lambda空struct采用括号。

运行：`D:/app/ANACONDA/python.exe -B docs/reports/T15-guide-evidence/verify.py --binary target/release/lfz.exe`，exit0，**20/20通过**。实际产物782848B，SHA256 `1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c`。每进程前与结束核hash，指南测前后hash一致，原始runs与manifest按原字节保存。

见证：直接从教程提取程序先check再run，输出精确accepted3/rejected1；成功nil、JSON/类型/字段/除零/整数越界/断言恢复；嵌套、argv、管道、引用alias、副作用、函数返回、连续100次失败后成功；4个外层校验/求参错误仍受控退出2。资源不可捕获规则来自spec和运行时单测，本指南不做真实OOM/abort压测，也不声称自己穷举所有错误或回归。

这是新增/受影响示例的聚焦作者验证，不是历史35块全量重跑，不替代独立验收。未编译、覆盖dist、改源/spec/AI或Git。R304仍OPEN，候选非正式发布，不宣称性能改善。
