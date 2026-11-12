# release-manager — 工作状态
> 最后更新：2026-11-13 by release-manager

## 当前状态
T21 候选交付准备完成。Windows target/dist 同为 1117184 字节，SHA256 76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e。旧 T20 字节先保存在 Temp/T21-release/lfz-T20-a545d0.exe，可用于公开测试的旧版对照。未编译或 Git 写入。

## 验收与证据
- T21-release-evidence/verify-delivery.py 核对候选 ZIP 42 项逐字节一致，771423B / a4c2e065…；独立清单 72 件、工具链证据 39 件及两份实测指南指纹一致。
- 独立 715 项 Rust 测试、23 项新见证及全部旧回归通过，公开 27、人类指南 2 通过。192 个库内输入中 173 合法且幂等、19 诊断拒绝。只整理顶层 CODE 空白，保留旧 fmt 调用形态。
- README 与交付清单已更新。R304 OPEN，中文终端渲染待验，T21 无性能窗口或正式标签。旧 v2、冻结 v3 和五份 AI 文档原样保留。

## Git 准备
- prepare-checkpoint.py 沿 T20 schema 分五组；只读预检已验证 114 个 SHA 绑定路径的 Git 字节保真。四源码和两测试使用冻结副本，原用户三块 28/1/1 只从 Temp 暂存副本过滤。
- 待 root 等全员落盘后运行最终分组并代执行提交与推送。
