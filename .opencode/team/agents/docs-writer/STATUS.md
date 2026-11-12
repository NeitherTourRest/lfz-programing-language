# docs-writer 状态

2026-11-13，T21 人类格式化指南已完成并通过实跑。新增 formatting.md 与 README 入口，说明 stdout、cmd 安全另存、格式化保真范围、源码位置语义和裸 fmt 兼容。

最终 Windows 候选 76ac4cb5（1117184 bytes）实跑 2/2。直接 stdout 和指南中的 cmd 重定向均与预期字节一致，输入和文档 SHA 核对通过。首轮受限环境路径解析 WinError 5 记录保留，正常 Windows 权限下同验证器通过。证据 docs/reports/T21-guide-evidence.md。

下一步交 team-lead 纳入候选交付。人类指南无阻塞。
