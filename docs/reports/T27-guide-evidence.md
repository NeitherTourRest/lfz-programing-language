# T27 findIndex 人指南验证

2026-11-18，team-lead 在最终 Windows 候选上实跑，三项全部通过。候选为 1119744 字节，实跑前后 SHA256 均为 `4816058d402a5d25c0455a0f9c5329febb0dd60629fc27d18654213a947ac636`。指南和两份示例的 SHA256 与实跑记录一致。[运行记录](../../benchmarks/t27/results/guide.json)

| 验收命令 | 退出码 | 观察结果 |
| --- | --- | --- |
| `check docs/guide/examples/find-index.lfz` | 0 | stdout 和 stderr 为空 |
| `run docs/guide/examples/find-index.lfz` | 0 | 所有断言执行通过，stdout 为 `findIndex examples ok` |
| `--json --max-steps 200 run docs/guide/examples/find-index-budget.lfz` | 2 | stdout 为 `ExecutionLimitError` JSON，位置指向回调的循环，stderr 为空 |

原始记录来自 `Temp/T27/guide.json`，SHA256 为 `53d94fc45bd7068854ca89dbd4d1035e77286637637f6aeac65cdd557667e015`。

在仓库根目录用同 SHA 的候选复现。输出文件需采用未占用的项目内路径。

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T27-guide-evidence/verify.py --binary target/release/lfz.exe --sha256 4816058d402a5d25c0455a0f9c5329febb0dd60629fc27d18654213a947ac636 --output Temp/T27/guide-reproduce.json
```
