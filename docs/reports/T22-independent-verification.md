# T22 Windows 普通控制台独立验证

2026-11-14，实测交付文件 `dist/lfz.exe`。SHA256 `76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e`，1117184 字节。用 Python ctypes 启动隐藏的独立 `CREATE_NEW_CONSOLE`，父控制台与系统代码页保持原值。子控制台逐轮调整输出代码页，并在结束前复原。普通 stdin、stdout、stderr 均为 `FILE_TYPE_CHAR`，`GetConsoleMode` 均成功。通过 `WriteConsoleInputW` 输入实际按键，并用 `ReadConsoleOutputCharacterW` 读取真实 UTF-16 屏幕缓冲。

在 CP437、CP936、CP65001 各实测五组普通控制台行为，共 15/15。普通执行打印 `中文雪` 返回 0，未定义名字的中文诊断返回 2。真实 REPL 支持中文 argv 回显、输入包含中文的表达式计算 `3字符`，均返回 0。输入无效元命令后显示 `repl: 未知命令`，再输入 `:quit`，退出 2。三种代码页的输出与诊断预期一致。

同一次运行以显式 `GENERIC_WRITE` 打开的 `CONOUT$` 替换 stdout 或 stderr。该句柄仍为 `FILE_TYPE_CHAR`，但 `GetConsoleMode` 返回拒绝访问 5。CP437/936 各两组真实显示乱码，退出分别仍为 0/2。CP65001 两组正常。持有读写权限的 `CONOUT$` 控制句柄 `GetConsoleMode` 成功。故意构造的只写重定向失败与普通控制台模式结果分别记录。测试没有测 ConPTY。

初次探索夹具混入补充平面 emoji，屏幕缓冲读出替代字符，相关原始输出保留。独立管道对照确认同一程序的 UTF-8 stdout 能完整输出 emoji。最终控制台矩阵的断言限定为 BMP 中文，不认证 emoji 显示。

按仓库根目录复现，临时输出位于项目 `Temp/T22-verifier`：

```powershell
New-Item -ItemType Directory -Force Temp/T22-verifier/input-final | Out-Null
D:/app/ANACONDA/python.exe -B docs/reports/T22-independent-evidence/native-console-final.py
D:/app/ANACONDA/python.exe -B docs/reports/T22-independent-evidence/verify-native-final.py
```

原始 UTF-16LE 屏幕缓冲、各项退出码、字节哈希、句柄属性、当前代码页与输入均保存在 `T22-independent-evidence/input-final/`。候选文件未修改，也未重新构建。
