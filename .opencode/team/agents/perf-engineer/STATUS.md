# perf-engineer — 工作状态
> 最后更新：2026-10-27 19:46 by perf-engineer

## 当前状态
T13最终9cda候选唯一完整窗口已完成，15/19原线PASS、2环境不确定、2FAIL，R304保持OPEN。全部532正式及152预热输出/hash正确；测前测后src/Cargo/protocol相同。测量任务完成，未批准整体效率或正式发布，不追加择窗重跑。

## 最终产物与证据
- 新dist/target由主调度冻结：1.4.0、761344B、SHA9cda0c6fd5d85178335e8b758a843fc5f66574826bab4a80f2418a318e171cf9。
- benchmarks/t13/protocol.json SHA74934a4ebc1f2eca5a6ea00e8d6e1c39ed5a4de5315a6cb8524d076d84c9f1bb，固定原19输入/线/旧149b和T12 47ed、5秒/2+7、三标签轮转。
- benchmarks/t13/results/final-window.json，399871B，SHA74cdf5052140f47229c5d811c2a92283ebb42b040f3f47325ed1d5c8b09681ea；19组三标签399正式、9能力双标签126正式、真实dist7正式，共532，另152预热。全部warmup/正式原始wall/user/kernel/stdout/stderr/exit/hash保全。
- assessment.json SHA8e68fc322555e0d6904387d97d3ae404a579377ceacaa629939e071f21c214b2；assess.py重算532+152、原线/分类/比值/所有median/min/max全PASS，未启动解释器；用项目Temp新输出复算字节hash完全相同。
- docs/reports/T13-performance.md 12715B：测前协议、零启动预检、最终19组三标签全表/9能力表、局部收益/回归/限制。

## 结果与限制
- FAIL：string_map2k新17.641>原17.522、old14.228；small4新22.960>18.746、old16.221。不能舍入为通过。
- 环境不确定：string_ops2k新32.548/old34.704均超28.902；string_map8k新23.762/old23.032均超22.521，不算PASS。
- string_map三档新/T12为1.162/1.065/1.078；string_ops8k新234.390比T12慢13.7%、比旧慢8.2%仍绝对PASS；small4新比T12慢30.4%。没有证明源码消分配转化为一致总时改善。
- numeric500k旧106.862/T12 90.747/新89.539，dist90.165；struct8k旧95.424/T12 19.180/新19.488，原30%目标通过，主要索引收益属T12不冒充T13。
- JSON stringify4k比T12慢25.5%、CLI eval慢10.4%明确披露；其余有快有慢，无虚构JSON原绝对线。noop12.251，parse2k24.334启动占比50.3%，不报扣启动纯吞吐。
- string_ops主循环是累计push数组O(n²)浅复制，不是字符串拼接。精确LLVM/系统微因未确定；源码中间Vec/快照确实消除不能代替实测。CPU可能量化为0，峰值RSS未测。

## 下一步
独占已释放并通知team-lead。由verifier审计原始JSON/协议；R304与失败继续开放。后续实现方向由team-lead调度，不自行新增候选或窗口。

## 纪律
只写本轮benchmarks/t13、T13-performance、自身记录、项目Temp；旧性能资料/冻结v2/源码/Cargo/profile未改，未Git，项目外只读。