# T23 Windows 终端 Unicode 输出对照

2026-11-15，以已交付的 `dist/lfz.exe` 实测，SHA-256 `76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e`。

| 观测路径 | 对照方式 | 结果 |
| --- | --- | --- |
| 私有 ConPTY 的原始输出 | CP437、CP936、CP65001 各运行 `WriteConsoleW`、LFZ `-e` 和 LFZ REPL | 九次均退出 0，输出是合法 UTF-8。五个标记各完整出现一次，共 45/45。 |
| 私有传统控制台屏幕缓冲 | 同三种代码页分别运行 LFZ 脚本与直接调用 `WriteConsoleW`，用 `ReadConsoleOutputCharacterW` 读取 | 三对 UTF-16 转储逐字节一致。两条写入路径均成功，读取结果中的 😀 和 𠀀 均为替代字符，BMP 中文保持原样。 |

ConPTY 样本包含 BMP 中文、单个 emoji、扩展区汉字、组合重音和 emoji ZWJ 序列。REPL 通过 argv 传值，伪终端输入只含 ASCII 表达式，因此标记来自 LFZ 输出。系统对照写入报告了完整的 UTF-16 单元数。本轮验收了本机常规可读写句柄的输出传输。终端字体、图形显示效果与补充平面字符键入不在本轮实测范围。T22 的只写 `CONOUT$` 边界另有历史证据。

原始流、屏幕转储、代码页、句柄属性、退出码和实际脚本保存在 [T23-terminal-evidence](T23-terminal-evidence/)。从项目根目录回读：

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T23-terminal-evidence/verify-evidence.py
```

重放实际探针时先创建 `Temp/T23-emoji`，再分别用 `conpty-unicode.py` 传入代码页和 `wide`、`run` 或 `repl`，用 `native-unicode.py` 重放传统控制台。两者均从同目录的 `dist` 读取已交付产物，写入项目 Temp。探针创建私有 Windows 控制台，代码页变化限定在该控制台。

本轮无需修改解释器或重编译。R304 保持开放。
