# T16-C 人类指南最终聚焦验证

日期：2026-10-30。责任人docs-writer。最终候选target/release/lfz.exe，902656B，SHA256 `cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006`。人类五指南入口及modules.md三文件教程已完成，实际候选聚焦见证**11/11通过**，exit0；不使用B或C修前exe冒称完成。

## 实际执行与权限记录

命令：`D:/app/ANACONDA/python.exe -B docs/reports/T16-C-guide-evidence/verify.py --binary target/release/lfz.exe --expected-sha256 cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006 --output Temp/T16-C-docs/新的结果.json`。

默认沙箱首个tutorial_check返回IOError/exit2，原因为本环境模块canonicalize权限；首次工具输出保留，随后为补部分结果持久化而对同一默认首例重跑一次，结果仍失败，default-failure.json/default-stderr.txt保存原始实际输出。未改解释器、关闭模块校验或把该失败当作产品已通过。

按主调度授权，同字节提权执行11/11通过，保存initial-elevated-runs.json；随后把可变默认值见证从“重新绑定数组”增强为“原地修改[0]后另一实例[0]仍0”，同一候选重跑11/11通过，最终runs.json保全。两次成功记录分别绑定自身验证器阶段，不是性能择窗，不增加性能通过结论。

## 见证范围

- 从modules.md直接提取calc/worker/main三文件，先check精确executed:false JSON、再两次独立run，业务JSON完全一致。
- 多alias namespace同一性；标量快照不回写模块绑定；共享数组原地修改；函数home globals不读caller同名全局；所有模块argv与入口一致。
- graph模板默认定义模块上下文、显式覆盖caller上下文；两个实例默认数组通过原地修改验证独立。
- 菱形check不产生初始化IO，run只初始化leaf一次（marker字节为x）。
- 缺导出直接绑定与循环导入均为SyntaxError图错误；依赖运行错误ZeroDivisionError的file和末帧file均指真实fail.lfz。
- attempt跨模块恢复后继续成功，错误记录仍五字段且无file，不夸称Result提供完整文件路径。

每进程前及结束核binary SHA，六指南测前测后hash一致。verify脚本/原始结果/失败记录与manifest按原字节保存；每行记录实际命令、退出码、stdout/stderr。新产物源/CLI后续若变化，本报告只证明上述C字节，不能外推T17。

RustAPI迁移依据interface-contract §19/21：模块graph入口与大栈所有权、外层GraphExecutionError和内层TracedRun区分、SourceId/ModuleId分域、UserFn home_module、StructDef default_context及旧helper None。此部分是规范引用说明，本角色未编译Rust，也不宣称RustAPI独立验收。

未重跑无改历史全部指南块；未改源码、规格、AI skill、dist或Git。所有临时只在Temp/T16-C-docs，未复制大产物。R304仍OPEN，不宣称模块效率或正式发布通过。
