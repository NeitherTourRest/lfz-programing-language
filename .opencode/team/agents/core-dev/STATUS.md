# core-dev — 工作状态
> 最后更新：2026-11-13 by core-dev

## 当前状态
T21 格式化词法布局入口与两个单测已落盘，停止源码写入并交 root/tooling。编译和测试由 tooling 统一运行。

## 产出
src/lexer.rs 实现 §27.3 的 crate 内 lex_format_layout。顶层 CODE 的词法分支调用静态观察器，格式化实例收集连续 SP/TAB 的最大 Unicode 标量区间。普通词法分析和 REPL 实例使用静态空闭包。字符串、插值、注释、CR/LF 的识别沿同一扫描器。

## 验证与交接
git diff --check -- src/lexer.rs exit0。SHA256 EB318A3D8458F667EC1917E289CD16932056A2BD62BCB8636440AF1B4AE6A914。新增单测覆盖最大区间、Unicode 标量索引、受保护片段、token/span 等值和正式错误等值。当前仅静态检查，等待 tooling 验证集成。