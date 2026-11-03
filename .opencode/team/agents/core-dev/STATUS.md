# core-dev — 工作状态
> 最后更新：2026-11-03 by core-dev

## 当前状态
T19 error.rs源码及严格动态字段unit已落盘，停止产品写入；交root/runtime/tooling消费，tooling待全部owner齐后唯一release。只当前T19交付后收口，不T20/automation。

## 产出
冻结§25.3五个ValueMsg及逐字消息：LineLimitRange/LinePathNul/LineNumberOverflow无字段，LineTooLong{line:i64,limit:usize}、LineUtf8{line:i64}。沿ValueError，13错误类不增加；旧所有断言/消息/公开错误shape保留。

## 验证证据/限制
先落line_value_messages_match_contract八case再最小实现：三个固定值；TooLong的(1,1)/(42,16777216)/(i64::MAX,4096)；Utf8的1/i64::MAX。逐字符消息、class、Span(47,31)、Display和value helper等值均锁定。依root未编译，不声称红/绿通过。
git diff --check -- src/error.rs exit0；47新增/1旧注释替换；SHA256 DA2B11D12BF3EE69F8E57AC047CFE0477DBC94FC357FC3A45A313D18A910C621。

## 下一步/阻塞
tooling统一构建运行error::tests，故障由root唤醒本owner修复。无契约疑问；未改runtime/tooling/黑盒/spec，未Git写入/外装/项目外写入，原用户三块保护。R304OPEN。