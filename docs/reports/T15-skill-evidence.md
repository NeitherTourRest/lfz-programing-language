# T15 AI指南同步与作者实跑

日期：2026-10-29。责任人：ai-dx-engineer。三spec §18为事实源；本轮四skill文件与AI入口同步73内置、attempt业务恢复和先check再run工作流。SKILL/AI入口新增§6.3及可提取的坏JSON混批示例，模板要求检查每条ok而非只看exit0。

固定Result三字段、五字段error/三字段traceframe、无file、nil成功、原引用拓扑、白名单与资源终止传播、参数校验边界、完整depth/trace/span恢复、函数ID/用户副作用不回滚均明确。attempt非一等、原普通builtin须closure包装、argv例外、class保留字索引写法已列。并未宣传事务、沙箱、流式内存或性能收益。

## 作者证据

命令：`D:/app/ANACONDA/python.exe -B docs/reports/T15-skill-evidence/run_examples.py --binary target/release/lfz.exe --output docs/reports/T15-skill-evidence/results.json`，exit0，**25/25通过**。candidate782848B，SHA256 `1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c`，每进程核hash，五指南测前后hash一致。

两个指南实际提取的混批LFZ块分别check/run（4项），14项成功/引用/返回函数/argv/管道/遮蔽/嵌套/连续恢复/副作用/字段/整数溢出/trace/JSON及最小旧兼容，7项边界与资源容量传播/非一等/旧普通builtin/未捕获错误负例，总25。真实容量边界仅用既有range容量检查，不做OOM/abort压测。源码、参数、stdout/stderr/exit保存在results.json，未编译或修改dist。

本记录是作者示例回归，不是全新Agent盲测。旧v2的8/8、131/131及原manifest没有改写或重新计分。R304仍OPEN，无T15性能计时，不声明整体提速或正式发布。旧T12/T14记录保持历史语境。
