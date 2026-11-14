# T22 Windows REPL 中文终端验证

对 `dist/lfz.exe` SHA-256 `76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e` 完成六组私有 ConPTY 对照。代码页 437、936、65001 均实际应用于输入和输出。每组预送同一组 UTF-8 字节，触发未知元命令、在 `input()` 中读取 `你好LFZ`、用 `print` 加 `RESULT<…>` 边界输出、触发带 `中文源码` 的语法诊断，然后 reset 和 quit。数据和诊断的匹配与键盘 echo 区分开，整个序列不依赖提示符尾空格或重绘布局。

| 实际代码页 | stdout/stderr 可读写，三项中文见证 | stdout/stderr 仅写，三项中文见证 |
| --- | --- | --- |
| 437 | 正确 | 乱码 |
| 936 | 正确 | 乱码 |
| 65001 | 正确 | 正确 |

三项见证分别是完整 `repl: 未知命令`、`RESULT<你好LFZ>`、真实 cell 文件帧下的 `let = "中文源码"`。六组退出码均为 2，原始 ConPTY 输出均可严格按 UTF-8 解码。可读写三组的三个流均通过 `GetConsoleMode`。仅写三组的 stdin 仍通过，stdout 和 stderr 均失败并返回 Windows 错误 5。解释与 Rust 1.90 stdio 分流吻合。它用 `GetConsoleMode` 判断控制台句柄，检测失败时写原字节。Windows 要求查询句柄具有 GENERIC_READ。在 437 或 936 的控制台中，仅写设备路径收到的 UTF-8 字节由控制台当作其本地代码页解释，再由 ConPTY 编为 UTF-8。T20 历史乱码与这一条件一致。 [Rust 1.90 Windows stdio](https://raw.githubusercontent.com/rust-lang/rust/1.90.0/library/std/src/sys/stdio/windows.rs) [GetConsoleMode](https://learn.microsoft.com/en-us/windows/console/getconsolemode)

独立验证使用原生控制台屏幕缓冲区的 UTF-16 观测，覆盖普通 stdin/stdout/stderr 与仅写输出对照。本报告的 ConPTY UTF-8 结果与其一致。主调度另保存了 pipe 的 stdout/stderr 独立字节和历史 cmd 重定向复核。 [独立验证](T22-independent-verification.md) [管道及历史重定向](T22-pipe-evidence/)

在仓库根目录执行 `python -B docs/tooling/evidence/T22/conpty_compare.py 936 rw` 可重放。参数为 `437`、`936` 或 `65001`，以及 `rw` 或 `write`。脚本默认写 `Temp/T22-tooling-replay`，环境变量 `LFZ_T22_OUTPUT` 可指定项目 Temp 中的其他目录。脚本用私有伪控制台、实际句柄访问权限以及外部 watchdog。`conpty_compare.executed.py` 保存正式运行版本，`conpty_compare.py` 只调整结果目录以供重放。SHA-256 索引绑定两份脚本和六组输入、原始输出、归一转录、判断结果、句柄和实际代码页记录。 [封存索引](../tooling/evidence/T22/manifest.json)

ConPTY 合并输出流，通道区分由 pipe 证据提供。范围覆盖本机 Windows 上的 BMP 中文输入、诊断和回显。普通控制台通过，历史仅写设备重定向在非 UTF-8 代码页仍可复现乱码。本次验证使用交付版二进制，结论支持保持现有产品实现。
