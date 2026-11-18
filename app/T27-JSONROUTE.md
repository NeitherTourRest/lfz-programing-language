# T27 JSON 规则路由

用 `app/jsonroute.lfz` 把构建通知等事件分配到值班组或处理队列。输入是一条 UTF-8 JSON 消息，可以通过 stdin 的第一行传入，也可以作为 `--` 后的唯一参数。输出是一行 JSON，调用方可据此决定后续动作。

```powershell
$request = '{"event":{"kind":"build","severity":4},"rules":[{"kind":"build","minSeverity":4,"target":"build-oncall"},{"kind":"*","minSeverity":0,"target":"general"}]}'
$request | ./target/release/lfz.exe run app/jsonroute.lfz
```

事件种类是非空字符串，严重级别是非负整数。每条规则包含 `kind`、非负整数 `minSeverity` 和非空 `target`。种类按原字符串精确匹配，规则中的 `*` 匹配任意种类。事件的 `kind` 必须是具体种类。严重级别达到门槛便满足该条件。应用先校验完整规则表，配置错误以退出码 2 和 stderr 诊断报告。

数组顺序决定优先级。在示例中两条规则均满足条件，第一条规则赢得路由。

```json
{"checked":1,"index":0,"matched":true,"target":"build-oncall"}
```

调换两条规则会选中 `general`。没有命中时，`matched` 为 false，`index` 与 `target` 为 null。索引从零开始，`checked` 记录谓词实际调用次数，命中后的规则不再调用谓词。前置配置校验仍检查全部规则。

开发时依据项目 AI 编程指南和 T27 §28 的 `findIndex` 契约。应用先隔离输入校验，再用一次 `findIndex` 选择首条匹配规则。回调只计数并比较，不修改规则数组。Python 验证器用自己的规则循环计算预期，检查两种输入入口、重叠顺序、索引 0 和 1、未命中、空表、Unicode、缺字段、坏 JSON，以及可命中规则之后仍拒绝非法配置。

```powershell
python -B app/verify_jsonroute.py --prepare-only
python -B app/verify_jsonroute.py --binary target/release/lfz.exe --expected-sha 4816058d402a5d25c0455a0f9c5329febb0dd60629fc27d18654213a947ac636 --output Temp/T27-app/results.json
```

2026-11-18 在 1119744 字节的冻结候选上实跑，九组全部通过。两组重叠规则交换顺序后分别路由到 `build-oncall` 与 `general`，两次都只调用一次谓词。第二条规则命中时返回索引 1 并调用两次谓词。未命中和空表均返回 null 索引，调用次数分别为 2 和 0。三组无效输入均退出 2，stdout 为空。候选哈希及各组原始输出见 [验收记录](../benchmarks/t27/results/jsonroute.json)。
