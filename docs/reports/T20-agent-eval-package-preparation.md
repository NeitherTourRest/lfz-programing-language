# T20 Agent v3任务包准备交接

状态：DRAFT_PENDING_REVIEW_AND_FINAL_ARTIFACTS，未冻结/未派学生/未运行LFZ。目录docs/validation/agent-benchmark-v3/v3-t20-r01。4份新题、4份多文件参考解候选、8个负控制候选与矩阵、纯Python oracle/runner、独立审计协议已准备。20min/40顶层工具/5check-run周期由root批准。REPL另轨不增第五题。

Python ast.parse最终通过；首轮新runner非ASCII bytes literal失败保private/prepare-failure.txt，修为UTF8hex，不改产品语义。参考LFZ尚未check/run，不称正控制已过。独立verifier需检查题面/oracle/参考解/八负控并补trace预算负控；签收后补最终T20 exe与五AI文档hash、审计采集wrapper及真实session导出可用性，才可冻结manifest。runner将缺源码/协议审查的认证保持UNKNOWN，不靠学生commands.json自报模型/工具数。

C01包含严格raw cap4096/4097、坏UTF8/坏JSON/跨32批；C02固定三槽共享数组避免越界追加，模块home/namespace/argv/check marker和fault来源；C03业务恢复/成功nil/预算不吞；C04临时loopback重复头/404/302/UTF8/禁用零请求及finally关闭。没有公网/外装/规模窗/编译/旧v2改写。学生可见仅student子目录与后续允许文档/解释器；私有评分件不交学生。workspace提示隔离非OSsandbox。

限制：这是准备交付，当前runner不自动证明完整tooltrace，也未实现受控学生自调wrapper；独立审查记录需绑定source_hashes并人工核原材料。功能通过不替代源码/协议PASS。任务不同于v2，不能直接比较8/8或历史2/8。R304OPEN，本轮T20收口不自动化。
