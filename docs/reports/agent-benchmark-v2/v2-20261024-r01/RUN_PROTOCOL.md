# v2-20261024-r01 正式执行记录与审计边界

- 固定输入：提交 e77b88c 的八题任务、131 case、run.py，以及 manifest SHA256 f2046eadca7c9f55376cbac000ccd53b25958298d10e2fc176ed0a14867c2a3e；正式执行期间不修改冻结件。
- 学生：B01..B08 各自 fork_turns=none 创建，未传本项目历史；仅本题TASK.md、四skill文件和解释器副本。完整调度提示按题归档，所有允许输入绑定hash。
- 模型：派发时继承配置且工具未返回型号，初记unknown；交卷后从八个匹配的独立session turn_context确认均为 gpt-6.1-sol、reasoning effort medium。原unknown字段保留，温度未知。不将本次8题结果泛化为该模型普遍能力。
- 预算：每题20分钟、最多5次学生自调；保留每次solution源码快照、命令、UTC起止、stdout/stderr与退出码。首次自调成功只是学生自选测试的过程指标，不等于冻结验收集的首次通过率。
- 时间：首批B01..B03以工作区创建时间作为保守预算起点，真实派发位于created_utc与dispatch_recorded_utc之间；后续题在派发前记时。元数据中初始DateTime本地化解析被纠正为DateTimeOffset解析，保留修正说明；没有基于错误deadline中断学生，也没有给学生延长预算。完成时间取调度者收到最终答复后的保守上界。
- 隔离强度：提示词限定读取范围，共享文件系统；没有声称操作系统访问控制。父Agent不向学生提供任何验收反馈，不向学生暴露验收代码/参考解；交卷后封存，评分后不回写本轮答案。
- 审计依据：调度提示/派发元信息、文件快照/hash、自调日志与报告，以及tool-traces/中的八个独立session完整记录工具调用/返回。原session文件只读保留在本机，导出排除私有推理、系统/developer消息及无关会话；以原文件hash和session/parent/agent_path绑定来源。仍没有OS文件访问轨迹，不把记录内合规表述为已经证明进程或文件系统强隔离。
- 评分：冻结运行器给功能结果；独立verifier另核源码算法约束与协议，绑定functional报告/manifest/学生源码hash。FAIL/UNKNOWN不得自动改为PASS。分母恒8，旧2/8不作同题比较。
- 原始字节：最终归档除可读源码/日志外，保留ZIP原始字节副本及SHA256，避免Git换行归一影响原始输入/输出hash追溯；副本与本轮记录一起保存。
- 本次不是旧题重跑；不得据新8题结果计算相对历史2/8的提升比例。