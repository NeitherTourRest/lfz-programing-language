# verifier — 工作状态
> 最后更新：2026-10-29 by verifier

## 当前状态
T16-A独立API与legacy兼容验收PASS。来源基础不是import/export实现；R304OPEN，未计时/RSS、不批准正式发布。

## 实际证据
- target65a29eaea54f32d1d2b404f22aed09389c3ffa71e6a81acb8fa6351d32fa42b7，783872B；source/Cargo测前后相同。
- 直接7release harness562通过；attempt13/check117/CLI91内官方178/旧app37+11/jsonbatch38/旧JSON140全部实跑PASS。
- 独立Rust API witness校验稳定来源ID/无Loaded/Loaded原文/legacyNone/验证次序/原子不改旧sidecar/多帧来源/公开字段修改后安全访问/home ENTRY，全通过。
- 默认rustc链接release LTO库失败日志保留；匹配-C lto/opt-level2/codegen1后编译运行PASS，无库重建/源码修复。
- 16程序human/JSON两模式32组旧1a5a对新65a29差分、64进程，完整双流与退出码逐字节一致，预期错误类校验通过。
- docs/reports/T16-A-independent-verification.md及evidence含脚本/原始结果/manifest。

## 下一步
交主调度收口A，后继B/C/D须另冻结和验证。UserFn新home_module是明确Rust构造迁移，不能宣传完全无API变化。没有真实模块执行或全跨文件CLI定位证据。

## 纪律
只验证不修复、不Git；全部Temp项目内，未改旧报告/正式测试/源/规范；无真实OOM/abort压力。已停止编译和执行。
