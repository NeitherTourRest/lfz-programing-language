# Agent benchmark v3-t20-r01（准备，未冻结）
四题内容不同于v2，旧2/8与v2 8/8/131case不可直接比较。预算20min/40tool/5check-run周期经root批准。学生仅收到student/<id>/TASK及公开输入、同hash解释器与五份AI文档；private/runner/oracles/控制件一律不交学生。当前未启动学生、未正式评分；manifest含pending_final_binary_and_docs，不构成冻结许可。REPL另功能轨道。本包须独立verifier审题及正负控后root最终GO。R304OPEN。
runner.py在项目Temp创建独立case目录，评分已封存学生源码；每case先check，check marker不执行。C04只loopback并finally关闭服务。所有stdout/stderr/exit/raw命令保存；源码与协议轴独立录入，缺真实trace为UNKNOWN。参考解只供独立预审，不声称已通过。
