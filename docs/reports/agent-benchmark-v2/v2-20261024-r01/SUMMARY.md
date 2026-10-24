# LFZ Agent v2 首轮正式测评总结

Run: `v2-20261024-r01`；冻结任务/工具版本提交 `e77b88c`。

## 结果

**8/8 整题通过；131/131 功能用例通过。** 每题均通过独立源码约束审查及本轮已记录工具动作的协议审查。

| 题目 | 功能用例 | 源码 | 协议 | 学生LFZ调用 | 学生会话耗时 |
|---|---:|---|---|---:|---:|
| B01 整数表达式解释器 | 16/16 | PASS | PASS | 1 | 175.3s |
| B02 CSV 解析器 | 18/18 | PASS | PASS | 1 | 170.5s |
| B03 碰撞桶字符串哈希表 | 15/15 | PASS | PASS | 1 | 165.7s |
| B04 网格最短路径 BFS | 16/16 | PASS | PASS | 1 | 163.7s |
| B05 0/1 背包及回溯 | 16/16 | PASS | PASS | 1 | 168.3s |
| B06 账户事件账本 | 17/17 | PASS | PASS | 1 | 162.8s |
| B07 文本倒排索引 | 17/17 | PASS | PASS | 1 | 193.3s |
| B08 文件流水账转换 | 16/16 | PASS | PASS | 2 | 182.5s |

学生共9次LFZ自行执行；8题首个记录均exit 0。此项使用学生自选测试，只是过程辅指标，不替代冻结验收集的成绩。学生交卷后未得到验收反馈并重写本轮答案。

## 输入与审计范围

- 八个独立 `fork_turns=none` 学生会话；实际会话元数据确认 `gpt-6.1-sol`、reasoning effort `medium`，温度未记录。
- 每题20分钟、最多5次LFZ自行调用；完成时间与日志/会话记录均在预算内，源码首跑快照和交卷hash绑定。
- verifier核对33次记录工具调用及33份对应返回：未见读取父目录/spec/验收/其他题、联网、外部语言求解或再次委派。
- 这是共享文件系统上的提示词隔离与记录工具审计；不是OS沙箱，也不证明未记录的系统活动不存在。私有推理及系统/developer提示未被导出。
- 本次是新v2题集的第一条基线，**不与旧2/8计算提升，也不泛化为所有工程任务上的成功率**。

## 证据入口

- [独立审查报告](REVIEW.md)、[功能报告](functional-results.json)、[人工审查](manual-review.json)、[最终汇总](certified-results.json)。
- [执行协议与限制](RUN_PROTOCOL.md)、[派发元信息](run-metadata.json)、[完整记录工具索引](tool-traces/index.json)。
- `sealed/B01..B08/` 保存交卷源码、首次快照、自写测试、stdout/stderr及日志；`runner-evidence/` 保存147个验收进程证据。
- [原始字节归档](raw-inputs-submissions.zip)及[ZIP校验](raw-archive.json)保留117项输入/源码/过程记录的原始字节；无需依赖Git换行设置。
- [总文件校验清单](artifact-manifest.json)覆盖本报告目录所有其他文件；`.gitattributes`禁用本目录文本换行归一。

解释器保持1.0.1，SHA256 `149b184e442ffbf250590431fa0aeffab401eac526faf60b4b435f620725d5ab`；未修改冻结验收或根据成绩放宽标准。
