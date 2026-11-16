# T26 回调实现单变量实验

先针对普通脚本函数调用中多余的全局环境保存与恢复做候选。模块和 REPL 仍须切换到函数的定义环境，并在返回或错误时恢复调用方。

比较采用 T25 相同的五个用例、8000 元素、进程内 12 次，同路径交替。一轮预热、六轮正式，全部原始样本保留。比较对象是已交付的 T21 二进制与实验候选。完整链路与映射分段的中位数均至少低于基线 3%，且完整链路至少五轮更快，才进入原 R304 全窗口。这个筛选只约束是否接纳优化，R304 原门槛继续适用。

功能检查覆盖模块、闭包、错误调用栈、REPL、执行预算及原回归。若实验没有足够性能收益，保留补丁和数据，撤回实验源改动。

## 可复现入口

以项目根目录为当前目录。两份实验源差异见 `results/a.patch.txt` 和 `results/b.patch.txt`。`results/candidate-snapshot.zip` 保存正式验收的 B 原字节。A 和 B 只用于各自的原型对照；正式窗口只启动了 B。

```powershell
D:/app/ANACONDA/python.exe -B benchmarks/t26/assess.py benchmarks/t26/results/a-window.json Temp/T26-perf/a-recheck.json
D:/app/ANACONDA/python.exe -B benchmarks/t26/assess.py benchmarks/t26/results/b-window.json Temp/T26-perf/b-recheck.json
D:/app/ANACONDA/python.exe -B benchmarks/t26/assess-gate.py benchmarks/t26/results/gate-b.json benchmarks/t26/results/gate-b.receipt.json Temp/T26-perf/gate-recheck.json
```

`compare.py CANDIDATE SHA WORK_DIR` 的基线是 T21 的原二进制，实跑前位于 dist。交付更新后，如需重放，应在其他项目内快照恢复原基线并以 `--baseline Temp/T26-perf/t21-delivered.exe` 显式传入，避免覆盖当前 dist。将冻结的 B 提取到 `Temp/T26-perf/prototype-b.exe` 后，`run-gate.py --output Temp/T26-perf/new-window.json --preflight` 可做零启动预检。正式窗口数据已归档，日常复核只需运行上面的复算命令。
