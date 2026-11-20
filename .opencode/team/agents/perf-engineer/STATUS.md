# perf-engineer — 工作状态

2026-11-20 T28 定向对照已冻结在 benchmarks/t28/focused.py，SHA256 d9af70d361e799c8542cfdceed07a8068bbabc8822287d56d311d0b450e6e82c。只读载入驱动并生成 8 项负载，原 numeric_loop 10000 与 string_ops 三档源码、期望输出和 stdin 取自 T13 原协议。

两份产物用同一执行路径、每轮交换次序，覆盖 1 轮预热与 6 轮正式样本。自赋值与持有旧数组别名的 push 分别测 4000 与 8000 项。LFZ 窗完整保存后，同一 Python 解释器运行课程原 string_ops.py 做 3 档同题对照。两组分别保留原样本、退出码、双通道字节和哈希。

计时与最终候选由 team-lead 运行，R304 正式门槛单独执行。本角色本次未启动 LFZ 或性能负载。

正式窗后剩余开销诊断驱动已交给 team-lead。benchmarks/t28/diagnose.py 固定 T27/T28 各 5 项、每项进程内 16 次重复，1+6 轮。原 array_builtins 10000、string_map 2000、map/filter 与 sort/reduce 分段和循环/输出对照分别记录。单窗 70 次，各进程 16 行完整输出与独立预期校验。脚本由本角色准备，未运行 LFZ/Python 测量。
