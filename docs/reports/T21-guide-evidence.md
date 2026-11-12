# T21 人类格式化指南验证

2026-11-13，docs-writer。最终 Windows 候选 `76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e`（1117184 bytes）实跑通过 2/2，验证器退出码 0。README 和格式化指南的 SHA256 与候选 SHA256 在运行前后保持一致，见 [原始记录](T21-guide-evidence/runs.json)。

- 直接执行 `lfz fmt "source file.lfz"`，stdout 字节精确匹配预期。
- 执行指南中的 cmd 重定向命令，带空格的不同输出路径收到相同字节，stderr 为空。
- 两次均保留原输入字节。输入包含 BOM、#42、CRLF、中文、无换行末行及行注释尾白。代码行首 Tab 等列替换，代码行尾空白删除。测试脚本中的写文件语句未生成文件。

首轮在受限环境解析候选路径时收到 WinError 5，尚未启动解释器。随后在正常 Windows 权限下运行同一验证器和候选，两项均通过。[首轮记录](T21-guide-evidence/first-sandbox-failure.txt)

复现命令在仓库根目录运行，输出文件和同名临时目录须尚未存在。

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T21-guide-evidence/verify.py --binary target/release/lfz.exe --sha256 76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e --output Temp/T21-guide/reproduce.json
```
