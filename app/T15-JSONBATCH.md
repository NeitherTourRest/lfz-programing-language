# T15 jsonbatch：坏记录不中断的JSONL批处理

日期：2026-10-29。作者：app-dev。依据三spec §18，使用项目LFZ编程skill及新attempt契约；skill尚未同步attempt时，以spec扩展为准。现有sortviz仍承担课程大应用，本例不凑行数、不替换原应用。

## 运行

从仓库根目录（最终产物复制dist后可将target路径改为dist）：

```powershell
./target/release/lfz.exe check app/jsonbatch.lfz
./target/release/lfz.exe run app/jsonbatch.lfz -- app/fixtures/t15/mixed.jsonl Temp/T15-app/result.json
```

必须恰好INPUT OUTPUT两参数，均非空白路径，禁止字面相同的输入/输出路径。正常模式stdout是一行JSON统计，结果写OUTPUT；业务坏记录不导致进程失败，读取/参数/写入错误仍按原错误退出2。CLI --json模式应按原规则解释状态对象与程序输出，不能把程序统计当执行状态。

OUTPUT是单个JSON文档：`total/passed/failed/rows`。rows与输入行顺序一致，每项含`line/ok/value/error`。成功null为ok=true/value=null/error=null；失败为ok=false/value=null/error={class,message,line,col,traceback}。error的line/col是LFZ调用点位置，外层line才是输入记录物理行号。字段class是保留字键，如需读取应写`row.error["class"]`。编码由jsonStringify完成，键排序/错误记录均按规范，末尾追加一个LF。

## 行策略

- 全文件按UTF-8读取，先只把CRLF替换为LF，再按LF切分；裸CR不另作为分隔符，作为JSON文本处理。
- 空文件0条；非空文件仅移除结尾LF产生的一个虚拟尾片段。`1\n`为1条，`\n`为1条空记录，`\n\n`为2条空记录。
- 所有真实空行/仅空白行都送jsonParse并得到失败记录，不静默跳过。末行无换行照常处理。
- 原样严格JSON：重复键、i64超界、垃圾尾随、BOM等按jsonParse失败。处理一行时用`attempt(() => jsonParse(text))`捕获业务错误，后续行继续。

## 验证与开发记录

1. 先读LFZ skill和spec，选用range/map预构造结果而非循环push，lambda返回记录用括号明确struct，避免`=> {}`块体歧义。
2. 项目83f旧候选语法检查通过，但运行混合夹具exit2/NameError：未定义attempt；[原始新功能缺失证据](t15-evidence/preimplementation-red.txt)保留，不把它称为旧产品回归。
3. 用最终T15候选`1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c`（782848B）实际运行Python独立JSON oracle，**38/38通过**，exit0。
4. 覆盖混合夹具、空文件、末尾LF、连续空行、中间空行、CRLF、裸CR、nil/false/0/空字符串、Unicode、整数边界、重复键、尾随垃圾、连续200失败后成功；15种成功场景各重复一次核对输出字节相同。另8项参数/缺文件/同路径/写失败/非法UTF8失败路径核对输入及既有输出保留。
5. Python自检首次未显式UTF8读取源导致GBK解码错误，已改自检命令显式encoding=utf8；未修改语言实现。正式应用验证首次即38全过。

```powershell
D:/app/ANACONDA/python.exe -B app/verify_jsonbatch.py --lfz target/release/lfz.exe --expected-sha 1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c --output Temp/T15-app/new-validation.json
```

输出报告路径必须尚不存在，所有夹具只在项目Temp。原始结果与manifest在`app/t15-evidence/`。Python用独立json.loads及重复键/i64/有限浮点钩子，不调用LFZ实现内部函数；这里只验证选定数据而非所有JSON文法，完整codec验收由独立测试负责。

## 限制

整文件、分行数组、结果数组和最终JSON同时驻内存，非流式、非低内存声明。坏记录的原文本没有写入报告，仅保留结构化错误和记录行号。实际IO/编码完成后才覆盖输出；writeFile不是原子替换，写入中途失败可能破坏输出。仅比较字面路径，不能识别`./a`与`a`、大小写/链接等别名；调用者须确保输入输出实际不同。不会创建父目录；测试先建Temp目录。

本例不提供事务回滚/重试/资源隔离。容量/递归等不可捕获错误仍终止；R304仍OPEN，不以38功能通过声称性能达标或正式发布。未改旧应用、解释器或规范，未执行Git。
