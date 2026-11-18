# T27 findIndex 示例验证

2026-11-18，以候选 `Temp/T27/candidate.exe` 实跑两份 AI 指南示例。解释器为 1119744 B，SHA256 为 `4816058d402a5d25c0455a0f9c5329febb0dd60629fc27d18654213a947ac636`。每次调用前核对候选 SHA，指南及两个输入文件的 SHA 在运行前后一致。

| 示例 | check | run |
| --- | --- | --- |
| `05_find_index.lfz` | exit 0，输出为空 | `--max-steps 10000`，exit 0，stdout 为 `findIndex witnesses passed` |
| `06_find_index_budget.lfz` | exit 0，输出为空 | `--json --max-steps 100`，exit 2，stdout 的 `error` 为 `ExecutionLimitError`，故障位置为回调第 4 行第 9 列 |

正常示例的断言覆盖命中 `0`、中间匹配、空或无匹配的 `nil`、首胜后不再回调、槽位替换后的快照索引、嵌套共享、严格布尔条件、`attempt` 恢复及失败前写入保留。预算示例验证 `attempt` 内的回调耗尽步数后，CLI 仍返回资源错误。

在项目根目录运行指南内的四条命令即可重现。验证指定候选时，将命令中的程序路径替换为 `./Temp/T27/candidate.exe`。本次完整 argv、stdout、stderr、退出码和输入 SHA 保存在 `Temp/T27-ai/results.json`。
