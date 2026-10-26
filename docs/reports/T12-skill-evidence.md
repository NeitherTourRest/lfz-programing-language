# T12 AI 编程指南同步与作者实跑

日期：2026-10-25。责任人：ai-dx-engineer。范围：指南同步与作者示例回归；不是独立产品验收，也不是新Agent盲测成绩。

## 产出与版本

- .opencode/skills/lfz-programming/四文件：SKILL.md、README.md、VERIFICATION.md、prompt-template.md。
- docs/guide/ai/README.md：按team-lead专项追加授权，同步为当前机读skill的AI入口，移除旧54内置/无-e/stdin等陈旧规则。
- 当前项目内dist/lfz.exe：1.4.0，821248 bytes，SHA256 `fb8e5653827de1e1c3f73b81d37f21314ad41adfdc4d99f37cb8ca5f6491f47d`。
- 语法依据：docs/spec/{interface-contract,syntax,semantics}.md §16。未改语法或解释器，未改人类五指南/应用/黑盒/冻结题集。

## 同步内容

Agent可直接复制§6.2参数/JSON模板。明确--后的字面参数保护、-e下一token无条件源码、空串/Unicode/--json保真、业务JSON普通stdout与CLI --json状态对象区别；stdin源码读取至EOF，不能再共用stdin输入数据；-e/stdin不要求也不消费#42，文件仍要求。

argv是唯一新增的一等内置，支持别名/闭包/容器/遮蔽且每次数组独立；jsonParse/jsonStringify及旧69内置没有一等取值能力，map回调用lambda包裹。JSON严格语法、i64精确、有限f64与负零、Unicode/NUL/代理对、解码重复键、UTF-8字节offset、128容器/环与共享、方法字段过滤、确定性键序及精确错误均已写明。

性能部分保留map/预分配构造，禁止把循环push或字符串累积称为O(n)；struct索引是平均查找改善，JSON编码包含展开输出/键排序成本，不声称所有操作O(1)或未测加速。

## 实跑方法与结果

仓库根目录执行：

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T12-skill-evidence/run_examples.py
```

结果：**30 passed**，全部调用项目内dist/lfz.exe，单次15秒上限，临时件及子进程TEMP/TMP均定向Temp/ai-dx-t12。Python运行器只用标准库；环境有Python路径定位提示，命令exit0。

| 覆盖 | 结果与证据 |
| -- | -- |
| 文档完整JSON配置→filter/map→count/total业务结果 | stdout精确`{"count":2,"title":"中文🙂","total":60}`，exit0；源码job.lfz |
| argv中文/空串/--json，机器状态通道，别名/身份/fresh/形参遮蔽 | 5条相关场景通过，精确stdout/stderr |
| -e空CODE、CODE字面--json、stdin源码/EOF/非法UTF8 | 各按期望0或2，JSON错误消息匹配 |
| JSON lambda回调、大整数9007199254740993、负零/1.0、Unicode/emoji/$/NUL | 紧凑输出精确匹配，未被LFZ插值或float整数精度损失 |
| struct方法字段过滤、共享非环 | 正常序列化，键序稳定，不错误拒绝共享 |
| 语法错误UTF8字节偏移、解码重复键、i64溢出、float溢出、BOM、孤立代理、129容器 | 均exit2且ValueError精确reason/offset匹配；128容器成功 |
| 循环、数组函数、非有限float编码 | 均exit2，精确ValueError消息匹配 |
| 既有L1–L5与偶数求和最小基线 | 6条全部exit0，最小基线stdout6 |

完整逐调用args/stdin_hex/exit/stdout/stderr见[results.json](T12-skill-evidence/results.json)。`run_examples.py`内所有示例均来自当前指南或其契约边界；这只是作者实跑，不能替代仅凭skill的新Agent任务验证。

首次测试作者用`s.self = s`构造环，误用了字段关键字，解释器按规范报SyntaxError；探针改用`s.me = s`后正确得到循环引用ValueError。失败保留[author-probe-error.json](T12-skill-evidence/author-probe-error.json)，未改产品实现，未计入30通过。

## 历史基线保护

未改docs/validation/agent-benchmark-v2/manifest.json或任何题目/学生提交/成绩。旧raw-inputs-submissions.zip SHA256仍为`90a6a3e3d894e7a813b06463506d5cc540e0a4570d7b005a3fd27ccc6a776d84`，内含原九项输入。active skill/target exe变化后，旧runner对当前路径的manifest检查不匹配是预期；README明确须在项目Temp还原原输入复现旧轮，新版本测评另建manifest，不能覆盖旧manifest掩盖变化。没有将v2旧成绩改称新skill成绩。