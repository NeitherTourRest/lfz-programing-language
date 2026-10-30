# app-dev — 工作状态
> 最后更新：2026-10-30 by app-dev

## 当前状态
T16四模块jsonbatch作者验收33/33通过，锁C最终cdedc9/902656B；独立验收由verifier负责。T15应用38/38及全部旧应用保持不变。

## 产出
app/jsonbatch_modules/{common,record,batch,main}.lfz、verify_jsonbatch_modules.py、T16-JSONBATCH-MODULES.md、t16-evidence/.gitattributes。
Python AST PASS。菱形共享计数、caller同名全局不串、两次执行cache隔离、坏JSON恢复/行号/错误数据及输入输出保留都有oracle。

## 下一步/阻塞
默认sandbox首个check因模块访问权限IOError失败已保全；同一产物获许可提权原33进程全过，结果/manifest/报告已落盘。没有编译/Git/改产品，所有Temp项目内，D约105MiB只写小文件；无应用阻塞，可交独立verifier。

## 经验
模块路径与IO路径基准不同；attempt记录仍无file；共享state.calls须等于总记录数。应用非流式、非原子，保持字面路径检测限制。

## T17预算应用准备（2026-10-30）
新增verify_jsonbatch_budgets.py/T17-BUDGETS.md及t17-evidence属性；AST/prepare-only通过，解释器0启动。正常结果等价、早期终止KEEP输出、已完成IO不回滚、跨模块attempt不吞系统终止均准备；等待最终hash实跑，旧app与oracle未改，不编译/Git。

## T17最终实跑（2026-10-30）
锁1946ab9/943104B，同字节许可提权40/40 PASS；callback预算28见证，0..28逐次保留，默认sandbox拒绝失败也保存。正常结果字节等价/早期KEEP/已完成SIDE不回滚/check拒预算均过。app/t17-evidence/results/manifest和T17-BUDGETS最终节齐，源码/旧app/旧oracle未改，不编译/Git。R304OPEN，无应用阻塞，交独立verifier。

## T18-A工具编排应用完成（2026-10-30）
新增toolplan.lfz/verify_toolplan.py/T18-A-TOOLPLAN.md/t18a-evidence。锁4093714/984064B、独占fixture51b83f…27bee；默认sandbox实际7/7通过，两份7项报告保留，最终加强deadline必须在exec匿名闭包行7。9任务6成功3局部失败、结果字节确定；禁用/能力不泄漏/global deadline/全体验证无副作用/空计划均过。旧应用未改，无编译Git，R304OPEN。仅本轮交付后停止，不进入HTTP/T19/T20。
