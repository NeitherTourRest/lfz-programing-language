# verifier — 工作状态
> 最后更新：2026-10-29 by verifier

## 当前状态
T15 1a5a候选功能及有界恢复语义独立PASS。R304仍OPEN，无T15性能测量，不批准整体正式发布。

## 独立实证
- dist/target782848B，SHA1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c，source/Cargo测前后相同。
- 直接执行7个release harness549全通过，无编译；新attempt13、旧check117、旧CLI91含官方178、app37+11、新jsonbatch38、旧JSON140均实跑PASS。
- 自编38进程见证，human/JSON双模式覆盖nested、12000次失败后正常递归、逃逸闭包函数ID、error记录独立、alias、argv/pipeline/shadow/self、实参一次求值、外层校验/容量传播、后续错误无旧帧等。
- 直接执行unit合成Recursion/Syntax/Cosmos/Capacity不可捕获分类、depth置上限后的实际attempt调用、完整caller span/depth恢复；不故意abort/OOM，不称真实物理栈压测。
- 只读路径审核variant白名单、记录先于restore、funcID不回滚、旧Invoke及check无求值保持。授权3项内置元数据严格同步，旧行为断言保留。
- docs/reports/T15-independent-verification.md和T15-independent-evidence含原始双流/命令/scripts/manifest。

## 下一步
交主调度完成指南/需求/发布；指南作者结果由各自独立报告提供，不虚称已跑作者全部块。无功能阻塞，正式发布仍受R304开放项限制。

## 纪律
只验证、不修复、不Git、不编译；所有临时项目Temp。旧v2、T13失败数据及旧报告未改。
