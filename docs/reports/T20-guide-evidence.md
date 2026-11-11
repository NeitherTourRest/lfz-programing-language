# T20 人类指南验证

2026-11-11，docs-writer。team-lead 使用原验证器在最终 Windows 候选上实跑，16 个 stdin 场景全部通过，退出码为 0。候选大小为 1100800 bytes，运行前后 SHA256 均为 `a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c`。

`docs/guide/repl.md` 及 README、tutorial、reference、errors、testing 五入口在实跑前后具有相同 SHA256，且与当前六份指南一致。独立验证者核对了文档 SHA，并将原始运行记录按字节保存。原始记录及其副本的 SHA256 均为 `594c29d68b612aef7a94f350700266271cd4fc8fe67e2fd36925da9009f5365a`。[运行记录](T20-independent-evidence/human-guide-original.json) · [独立验收](T20-independent-verification.md)

16 个场景覆盖持久会话、多行函数和 if/else、错误后的部分效果和继续执行、struct 默认值的定义来源、input 消费命令文本、reset 保留 argv 和累计退出码、步数和超时、完整及未完成 EOF、print 只执行一次、JSON 模式与 import 拒绝、语法错误后继续，以及 1048577 字节输入超限后尾部代码未执行。真实 ConPTY 和宿主绝对 deadline 见独立验收。

复现命令在仓库根目录运行，输出文件需使用未占用的路径。

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T20-guide-evidence/verify.py --binary target/release/lfz.exe --sha256 a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c --output Temp/T20-guide/reproduce.json
```

2026-11-04 的准备检查记录为 exit 0、prepared=16、executed=false。本次归档依据 2026-11-11 的实跑记录。R304 仍为 OPEN。
