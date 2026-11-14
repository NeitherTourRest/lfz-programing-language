# release-manager — 工作状态
> 最后更新：2026-11-14 by release-manager

## 当前状态
T22 Windows 中文控制台证据及交付文档准备完成。dist 保持 T21 候选，1117184 字节，SHA256 76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e。未重编译、复制或 Git 写入。

## 证据与边界
- 普通 Windows 控制台 CP437/936/65001 的 BMP 中文独立检查 15/15。工具链六组 ConPTY 和管道分通道 UTF-8 证据保留。只写 CONOUT$ 在 CP437/936 是可复现的特殊重定向边界。Emoji 控制台显示本轮未验收，R304 OPEN。
- README 最新段澄清 T20 中文待验项，历史记录保留。三份 T22 manifest 与对应 95 个字节绑定路径的 Git 过滤前后指纹核对通过。

## 提交准备
- T22-release-evidence/prepare-checkpoint.py 沿 T21 schema，只读预检。三组为证据及属性、README/REPL 指南、收口记录。三用户块 28/1/1 仅 Temp 暂存副本过滤。
- 根代理在各角色落盘后生成最终清单并进行 Git 交付。
