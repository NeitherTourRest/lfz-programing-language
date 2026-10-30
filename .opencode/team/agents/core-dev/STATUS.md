# core-dev — 工作状态
> 最后更新：2026-10-30 by core-dev

## 当前状态
T18-A error.rs源码与新unit已落盘，停止源码写入，交root/tooling统一编译。遵守用户边界仅当前T18-A，交付后停止，不进入HTTP/后续。

## 产出
src/error.rs按§23.2新增十个无字段ValueMsg：ProcessDisabled/ProcessArgvEmpty/ProcessArgNul/ProcessExecutable/ProcessTimeoutRange/ProcessOutputRange/ProcessUnsupported/ProcessTimeout/ProcessOutputLimit/ProcessUtf8及逐字固定中文消息。沿既有ValueError，不增加第14类；LzError/Span/TraceFrame/R保持，旧全部断言保留。

## 证据/验证限制
新增process_value_messages_match_contract十case严格unit，逐字符message、class=ValueError、非默认Span(41,23)、Display和value helper等值全部覆盖。先落测试后最小实现；依禁止编译协调要求未运行红/绿，不声称测试通过。
git diff --check -- src/error.rs exit0；diff仅64新增/1旧注释替换；SHA256 0AB3C6112BFD01AAE0F52D1D9338C7950E10BBBCA975A68A7013A61BF6C3DBA6。

## 下一步/阻塞
tooling等runtime停源后唯一cargo编译并运行error::tests。无设计疑问；如本文件编译/测试失败由root唤醒修复。不改runtime/tooling/黑盒/spec，不Git/安装/外部写入，R304仍OPEN。