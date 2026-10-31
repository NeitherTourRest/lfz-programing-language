# core-dev — 工作状态
> 最后更新：2026-10-31 by core-dev

## 当前状态
T18-B error.rs源码及严格unit已落盘，停止产品写入；交root/runtime/tooling消费，tooling待全部owner停源后唯一统一release。仅当前HTTP收口，不T19、不恢复automation。

## 产出
按冻结§24.3新增11无字段ValueMsg及固定message：HttpDisabled/HttpMethod/HttpUrl/HttpHeaders/HttpBody/HttpTimeoutRange/HttpResponseRange/HttpUnsupported/HttpTimeout/HttpResponseLimit/HttpUtf8。沿既有ValueError，无第14类；旧Process/T17/全部行为断言、LzError/Span/TraceFrame/R不改。

## 验证证据/限制
先落http_value_messages_match_contract十一case严格unit再最小实现，覆盖逐字符消息、class=ValueError、非默认Span(43,29)、Display和value helper等值。按root禁止编译协调指令未运行红/绿，不声称编译或测试通过。
git diff --check -- src/error.rs exit0；diff仅68新增/1旧注释替换。SHA256 590BD6925871176C75C32C24E74F4DB9F6475FE41CC50F7F30185D891FAF804B。

## 下一步/阻塞
tooling统一编译运行error::tests；如本文件故障由root唤醒修复。未改runtime/tooling/黑盒/spec，未Git写入/外装/项目外写入，原用户块保留。R304仍OPEN。