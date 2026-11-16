# T24 R304 最终候选基线测量

本轮冻结对象为已交付的 Windows T21 候选，SHA-256 `76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e`。T22 与 T23 为验证交付，沿用同一解释器。

使用 T13 已冻结的同路径协议。每次替换后固定等待 5 秒。每个用例和标签有 2 次预热与 7 次正式采样。旧、T12、本轮候选保持协议中的轮转次序。原 19 项数值门槛、输出预期及 8,000 键结构体收益目标均保持。独立记录源码递归指纹及实际日期，历史 runner 原有的 `client_date` 字段保留其原写法，当轮日期以外层回执为准。测量期间停止本项目其他编译和测试。

```powershell
D:/app/ANACONDA/python.exe -B benchmarks/t24/run.py --output Temp/T24-perf/window01.json --preflight
D:/app/ANACONDA/python.exe -B benchmarks/t24/run.py --output Temp/T24-perf/window01.json
D:/app/ANACONDA/python.exe -B benchmarks/t24/assess.py Temp/T24-perf/window01.json Temp/T24-perf/window01.receipt.json Temp/T24-perf/assessment.json
```

将每轮作为独立证据保存。出现环境不确定、失败或中断时保留原数据。固定门槛与同期候选对比均报告。总时间包含启动、载入、解析、准备与求值，短进程的 CPU 计量有量化误差。
