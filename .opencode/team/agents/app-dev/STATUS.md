# app-dev — 工作状态
> 最后更新：2026-10-29 by app-dev

## 当前状态
T15 jsonbatch新增应用已完成，作者独立oracle38/38通过；未Git，等待团队独立验收。现有T12 sortviz/jsonflow未修改。

## 产出/证据
- app/jsonbatch.lfz、fixtures/t15/mixed.jsonl、verify_jsonbatch.py、T15-JSONBATCH.md及t15-evidence原始日志/结果/manifest。
- 旧83f上check通过，运行exit2/NameError attempt缺失，红测保留。
- 新target/release SHA1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c/782848B，Python实跑38/38，所有Temp项目内。
- 空文件/空行/末尾LF/CRLF规则明确；连续200坏行后成功、重复输出确定、失败保留输入/已有输出均通过。

## 阻塞/下一步
无应用阻塞。verifier可复跑；R304仍OPEN，不声称性能达标。只新增T15 app产物及自身记录，未改旧应用/源码/spec/测试/团队看板。

## 关键经验
attempt错误class字段需["class"]访问；输入记录line与error.line的LFZ源码位置不同。整文件驻内存、非原子IO、只能识别字面相同路径必须公开，不称流式或安全沙箱。
