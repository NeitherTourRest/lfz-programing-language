# T20：交互式JSON检查台

app-dev，2026-11-04。依据§26/T20-contract，沿项目lfz-programming skill现有表达式/JSON/attempt语法；REPL来源边界以新契约为准。准备阶段通过 AST 与 prepare-only。冻结产物的六组会话均已实跑通过。

## 实用会话

启动 `./target/release/lfz.exe repl`，将 `app/repl_jsondesk/session.repl` 的物理行依次粘贴。该文件是交互输入记录，不是可run的LFZ脚本，不能添加#42，也不能拿普通run冒充持久会话。真正TTY交互prompt在stderr，oracle用管道会话无prompt；本应用不冒称已验证TTY渲染。

先定义desk和多行inspectLine函数，然后逐条输入JSON，inspectLine使用attempt解析，累计成功/失败，保留最近成功值。实际工作中可继续调用inspectLine检查更多数据，每次只处理新输入，无需重跑定义。示例包含合法对象、坏JSON、合法数组，输出逐条JSON与会话统计。命令只在空pending生效；多行函数右花括号后完整即执行，if/else须同一cell组织，不能把独立下一行else拼回上一cell。

会话调用多行函数 markThenFail。函数先增加 desk.failed，再调用 fail，随后查询统计确认字段写入保留。下一cell非法字符后仍可继续；:reset清状态，旧desk访问NameError，后面print仍执行。任何错误累计最终退出2，reset和quit不抹历史失败，不能把可继续输入理解为事务回滚。

## 六组真实REPL重放

```powershell
D:/app/ANACONDA/python.exe -B app/verify_repl_jsondesk.py --prepare-only
D:/app/ANACONDA/python.exe -B app/verify_repl_jsondesk.py --binary target/release/lfz.exe --expected-sha FINAL_SHA --output Temp/T20-app/final.json
```

1. JSON检查台：持久globals、多行函数、解析恢复、容器修改在错误后保留、语法错后继续、reset清状态。stdout逐行严格等值，旧print不能重放。
2. input消费：print(input())下一物理行输入:quit，它是数据被打印；真正后续cell仍执行，然后另一个:quit才退出。未把同stdin的输入回放为源码。
3. 历史函数：首cell多行函数定义，隔一个print再调用fail，诊断须含<repl:1>、定义处fail行和caret；之后继续，reset后旧函数NameError。应用不替代host旧outcome在reset后继续渲染的单测。
4. 每cell步预算：1000步下先建counter，再无限循环耗尽；后两cell仍各有新预算，能读取失败前counter增长并打印标记。oracle不是靠脚本attempt吞系统终止。
5. 每cell零timeout：第一次执行终止，reset后第二次仍终止，未打印业务标记；reset不偷偷移除配置限额。
6. 不完整多行在EOF：真实语法诊断与 EOF 行位置，退出2，不执行半定义。

每次调用argv命令明确是repl，无--json（该模式拒绝），stdin为UTF8会话字节；stdout精确逐行比较，stderr核对诊断片段/无管道prompt，结果实时保留。15秒外层watchdog只防挂，不声明产品硬实时保证。所有Temp在项目内，只小样，无公网/安装/编译/产品或旧app修改。

## 开发边界

选择会话检查台而非重复旧JSONL批处理，展示需要保留定义与统计的交互工作。无需新语法或巨型示例；旧应用及旧oracle保持。普通REPL会话保留源码/函数历史存在容量与复制成本，本示例不声称恒定历史内存。文件IO、模块import和外部能力并非本例业务前提；模块限制/权限/reset不增权、容量和host cleanup由公开及独立验证负责，本应用不代替其证据。

R304 仍为 OPEN。

## 2026-11-11 最终验收

用上面的命令重放会话，将 `FINAL_SHA` 替换为 `a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c`，输出路径为 `Temp/T20-app/app-final-pass.json`。冻结产物为 1100800 字节。驱动退出 0，六组通过。`app/t20-evidence/results.json` 保留每组输入、退出码、stdout 和 stderr。

初版把三条语句写在同一物理行，解释器按语法报 `TwoStatements`。演示改为多行函数 `markThenFail`。随后 `desk.failed` 由 1 变为 2，表明失败前的字段写入保留。坏 JSON 之后，`inspectLine` 仍可检查下一条数据。最近一次成功值只在成功解析时更新。

未完成函数的 EOF 位于空白第 2 行，诊断给出来源和缺失的 `}`。早期 oracle 对空白行要求 caret，随后也误写过诊断中文。这两份失败和对应预期均已归档。历史函数测试要求 `<repl:3>` 调用帧和 `<repl:1>` 第 2 行定义帧，紧接原始 `fail` 源码、caret 与 `AssertionError`。`manifest.json` 记录最终演示、预期和全部报告的哈希。
