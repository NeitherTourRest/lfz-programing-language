# T19：逐条JSONL检查与有界输出

2026-11-03，app-dev。依据§25/T19-contract，使用项目LFZ编程skill已有语法和新契约API。旧jsonbatch及模块应用不改。准备阶段AST/prepare-only通过且解释器启动0；最终冻结产物实跑14/14通过，证据见文末。

## 命令与用途

```powershell
./target/release/lfz.exe run app/jsonlstream.lfz -- Temp/T19-app/input.jsonl Temp/T19-app/output.jsonl Temp/T19-app/summary.json 0 4096
```

五参数是输入、逐条结果JSONL、汇总JSON、TAKE、MAX_LINE_BYTES。TAKE=0处理全部，正数最多交付指定条数；命中该条回调返回false，所以即使刚好到文件末尾也stopped=true。行上限示例4096，可配置1..16777216。参数完整校验后才清空结果文件。只防字面相同路径，不能识别路径别名；调用者必须保证三个实际路径互异并准备输出父目录。

输入由forEachLine逐条读取，不readFile、不collect整文件。每行callback更新外部currentLine.text，随后attempt(parseCurrent)；parseCurrent闭包在读取前只创建一次，空行和坏JSON形成失败结果，后面成功行继续，line保留物理行号；error.line是LFZ源码位置，不能替代数据行号。输出每条含line/ok/value/error，最终summary含total/passed/failed/lines/stopped。有效JSON null和解析失败靠ok区别。

LF是唯一分隔，CRLF去一个CR，孤CR/EOF末CR保留，Unicode严格UTF8，BOM不删除；空文件0条，尾LF不产生虚构记录，真实空行不跳过。maxLineBytes按LF前原始字节含CR计；不是字符数。坏UTF8或超长行不能进入callback，整流失败，不尝试跳过继续。

## 空间和副作用

最多暂存256条编码结果，满批appendFile后清空，EOF或正常早停写最后一批。push复制只有固定256条范围；一批内join一次，无无限字符串拼接。空间由行字节上限、JSON解析/编码和固定批次约束，不随已处理文件长度累积；引擎另有固定8KiB输入缓冲。不据源码宣称实际峰值或性能达标，规模测量由perf负责。

输出非原子，无事务。输入打开失败也可能已清空结果文件；流中错误/全局终止保留已append批次，当前未满批可能丢失，旧summary保留，只有成功流才覆盖summary。summary写失败时结果可能已经完整。重跑会清空输出；请求预算不足也不回滚已完成IO。ExecutionLimitError不能被attempt吞；不能把它作为可恢复坏记录。文件关闭由公开/host unit独立验证，本应用不以rename或删除冒充关闭证据。

## 独立oracle与准备记录

```powershell
D:/app/ANACONDA/python.exe -B app/verify_jsonlstream.py --prepare-only
D:/app/ANACONDA/python.exe -B app/verify_jsonlstream.py --binary target/release/lfz.exe --expected-sha FINAL_SHA --output Temp/T19-app/final.json
```

预设14次小样进程：空文件、坏JSON/空行后Unicode成功、CRLF、无尾LF、连续空行、BOM不删除、早停不解码后续坏UTF8/超长、精确字节限额、CR计入超限、坏UTF8、超长、600条跨批、已写256条在后续错误后保留、全局零步预算在初始化输出前终止。Python独立分行/JSON oracle按业务值核对，失败实时保留stdout/stderr及两输出hex，不调用产品内部实现。小样不占用perf测窗，不生成大文件。

开发选择：保留旧整文件应用供兼容比较；新增小工具展示流式能力，以256条批次限制应用侧积累。当前无实际产品结论；收到最终hash后核验再跑，所有首次失败必须保留。只本轮T19收口，不自动T20/automation；R304仍OPEN。

## 最终功能实跑（2026-11-04）

产物 `target/release/lfz.exe` 1041920 bytes，SHA256 `5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5`。上述命令FINAL_SHA替换该哈希，最终输出 `Temp/T19-app/final-cell.json`，exit0，14/14 PASS，已归档 `app/t19-evidence/results.json`。默认权限运行，无提权/网络/编译；所有测试子进程已结束，无规模测或perf并发。

开发修正和失败全保：初oracle错误对成功调用使用CLI --json，其stdout是运行状态而用户print移stderr；empty实际成功但oracle误判，`first-oracle-failure.json`及first-verifier.py/first-app.lfz原样保留。修正成功调用用普通run、应用print(jsonStringify(result))输出JSON，逐行闭包版本14绿见per-line-closure-results.json及对应源码。

root静态审查指出逐行创建UserFn可能使诊断metadata表随行数增长，因此改为外部struct cell currentLine.text与一次性parseCurrent闭包。第一次调整误把cell写成let字符串而在callback重绑定，触发语言既定TypeError，immutable-binding-failure.json和对应应用保留；改为struct字段赋值后最终完整14绿。以上为新应用/验证器改进，未修改产品、函数身份规则或旧应用。固定闭包避免逐行分配的结论是静态代码依据，不替代规模峰值实测。

最终oracle确认600条跨批结果精确，257条有效行后第258行坏UTF8时仅先前256条批次保留、summary仍KEEP；未满批不提交。early false跳过后续毒尾、CR计入字节cap、JSON错误/空记录后成功的精确物理行号均过。全局零步预算在应用初始化前终止并保留两份KEEP；此小样不宣称覆盖callback中途global终止，后者由公开/host验收及其他预算见证覆盖。资源关闭与性能不由此应用14例独立证明，R304仍OPEN。仅T19收口。
