# T27 首条匹配与数组拼接候选

本轮在 T26 上增加 `findIndex(predicate, array)`，返回首个匹配项的零基索引。未命中返回 `nil`，数组放在最后以支持管道写法。回调沿现有求值会话执行，条件必须为布尔值，同名用户绑定优先。进入时复制元素序列，允许回调修改原数组，嵌套对象仍为共享引用。首次命中后停止调用谓词，前置浅拷贝仍需要线性时间和空间。[冻结契约](T27-contract.md) · [人类指南](../guide/find-index.md) · [AI 指南](../guide/ai/findIndex.md)

`join` 在同步检查和拼接期间借用数组，省去整份值向量的克隆。它仍检查分隔符、数组和第一个非法元素。只有新增内置的对照产物及最终候选由同一离线 release 构建流程生成。候选相对于对照的唯一可执行源码差异是 `join`。

## 应用与功能验收

[JSON 规则路由](../../app/T27-JSONROUTE.md)使用规则顺序决定优先级。重叠规则交换后命中目标随之变化，两次都只执行一次谓词。应用、自测、开发记录作为扩展示例交付。课程应用 `sortviz` 也在原回归套件中重跑。

- Rust release 718 项、公开 LFZ 黑盒 181 项，17 套旧功能回归。单独核实空数组、无命中、索引零、短路、浅快照、同名遮蔽及步数预算终止。[Rust 日志](../../benchmarks/t27/results/rust-release.log) · [黑盒](../../benchmarks/t27/results/blackbox.json) · [旧回归](../../benchmarks/t27/results/regression-commands.json) · [旧版红测](../../benchmarks/t27/results/baseline-red.json)
- REPL 37 项、格式化 27 项通过。[REPL](../../benchmarks/t27/results/repl.json) · [格式化](../../benchmarks/t27/results/formatter.json)
- JSON 路由 9 项、人类指南 3 项、AI 指南 4 项实跑符合预期。[应用](../../benchmarks/t27/results/jsonroute.json) · [人类指南](../../benchmarks/t27/results/guide.json) · [AI 指南](../../benchmarks/t27/results/ai-guide.json)

独立验收的 6 组反例全部通过，其中五组还与只新增 findIndex 的产物逐字节比较输出。正式性能与定向记录由同一独立审核复算。[独立审核](T27-independent-verification.md)

旧版回归驱动及运行脚本归档在结果目录。重跑前将 `regression.py` 与 `regression.ps1` 复制至 `Temp/T27/`，创建 `Temp/T27/app40` 与 `Temp/T27/regression/json140`，并确认归档候选与 `target/release/lfz.exe` 的 SHA 一致。本机运行要求包括原 T19 验证夹具与 T21 格式化旧基线。

## 定向性能

16,000 个元素，进程内重复 40 次。两个产物使用同一启动路径，每个测项 1 个热身样本、8 个正式样本，中位数包括启动与数组准备。第一次夹具准备发现 Python 错误解析了 LFZ 插值，修正后四个夹具都通过输出校验，之后才启动定向计时。

| 测项 | 中位耗时 |
| --- | ---: |
| 同一候选命中首项的 `findIndex` | 20.570 ms |
| 同一候选完整 `filter`，然后取唯一匹配项 | 138.337 ms |
| `join` 仅新增内置的基线 → 最终候选 | 33.328 → 29.158 ms |
| 同步运行的数组准备对照 | 25.016 → 22.735 ms |

首项场景中 `findIndex` 的整体进程耗时约为 `filter` 的 15%。两个用例在各自 40 次操作中分别调用 40 和 640,000 次谓词。这个结果针对首项命中。无匹配或末项才命中时仍会遍历全部元素。拼接对照自身也快了 9.1%，因此 12.5% 的原始差值包含测量环境波动。源码明确少一次克隆，现有数据无法独立量化它的耗时收益。[定向原始记录](../../benchmarks/t27/results/focused.json) · [运行脚本](../../benchmarks/t27/focused.py)

## R304 与候选交付

同一最终候选用 T13 双基线原协议完整运行一次。152 个预热和 532 个正式样本均输出正确。原 19 条门槛中 16 通过、2 失败、1 项环境不确定。结构体 `struct_ops 8000` 由原 89.4598 ms 降至 22.1641 ms，降幅约 75.22%，达到原 30% 目标。

| 未通过项 | 同窗旧版（ms） | 原上限（ms） | T27（ms） | 结果 |
| --- | ---: | ---: | ---: | --- |
| `numeric_loop 10000` | 13.8461 | 17.9676 | 18.4975 | 失败 |
| `string_ops 2000` | 28.6862 | 28.9022 | 40.3671 | 失败 |
| `string_map 8000` | 23.5313 | 22.5214 | 23.9812 | 环境不确定 |

R304 对 T27 保持开放，T26 继续作为当前交付版本。未用多次窗口挑选结果。正式数字为本机整体进程耗时，包含启动、解析及 I/O。本轮没有峰值 RSS 数据，也没有把差异归因到新增内置或某次源码改动。[原始窗口](../../benchmarks/t27/results/gate.json) · [运行回执](../../benchmarks/t27/results/gate.receipt.json) · [原门槛复算](../../benchmarks/t27/results/gate-assessment.json)

T27 Windows 候选 1,119,744 字节，SHA256 `4816058d402a5d25c0455a0f9c5329febb0dd60629fc27d18654213a947ac636`。源码、Cargo 文件和实际测量 exe 压缩归档的 SHA256 为 `9aa771a16896f10fd9203bf60be0b854ed21b81390ff17b0cbc7b3fcf7aaad59`。[精确归档](../../benchmarks/t27/results/candidate-snapshot.zip) · [清单](../../benchmarks/t27/results/candidate-snapshot.json)
