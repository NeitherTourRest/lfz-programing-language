# T20 公开REPL验收准备

2026-11-04，test-engineer。完整启动已读全局/看板/自身记录及冻结§26/T20-contract。新 `tests/t20_cli_repl.py` **36例准备完成，尚未T20实际验收**；只新脚本/小stdin fixture/专属报告，无产品/旧测试/编译/Git/网络操作。

覆盖pipe无prompt、EOF空/空白注释、非nil echo/nil静默/print一次、共享globals、多行fn/ifelse与else跨cell限制、array/嵌套注释/插值、裸string LF/致命prefix不被未闭括号隐藏、pending EOF错误、失败后继续与旧effects不重放/失败前写入保留、旧函数/模板default真实repl来源caret、reset清globals与cell编号但不清最终error exit、unknown命令、pending命令作源码、input消费下一物理行(:quit作为数据)、argv/reset保持快照、HTTP/exec默认禁用、每cellsteps/timeout重新预算与资源后新cell继续、--json usage、模块拒绝、:help，以及真实>1MiB无LF单cell pipe超限/退出2/余尾print禁止执行。

输入通过communicate的stdin bytes直接写入，父进程10秒watchdog仅回收自己的LFZ。raw stdin独占Temp/T20-public，不创建网络/后台server。长行fixture为1048577字节首pending加LF和毒tail；避免用生成无限流、OOM或先吞无界物理行当测试。公开验收能证明此输入终止且未执行尾部，不替代源码/白盒对“追加前有界”检查的证明。

实际准备命令：

```text
D:/app/ANACONDA/python.exe -B -c "import ast,pathlib;ast.parse(pathlib.Path('tests/t20_cli_repl.py').read_text(encoding='utf8'));print('AST PASS')"
D:/app/ANACONDA/python.exe -B tests/t20_cli_repl.py --binary dist/lfz.exe --expected-sha 5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5 --output Temp/T20-public/prepared.json --prepare-only
D:/app/ANACONDA/python.exe -B tests/t20_cli_repl.py --binary dist/lfz.exe --expected-sha 5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5 --output Temp/T20-public/T19-red.json --red-only
```

AST PASS；prepare-only exit0/cases_total36/executed:false。冻结T19最终5d2219真实缺repl红0/1、脚本exit1，LFZ exit2、stderr按文件路径repl旧IOError。旧红输出/参数/输入hash与准备计划、script hash已封专属evidence/manifest；不拿旧产物成绩称T20通过。

正式T20最终hash后完整实跑，保存首脚本/首失败和新oracle修正依据。对error stderr检查类/固定消息及真实旧source文本，TTY prompt/flush必须owner PTY/host另测，pipe不冒充TTY。host绝对deadline/token跨reset、旧outcome snapshot、双diag poison、func65536/ReplCapacity不可attempt吞与16MiB/4096 admission安全边界归owner host unit/独立，不把公开36计为全部§26已覆盖；:reset不能增权与真实同源重复声明仍需最终补例/独立证据。模块same-Program前置拒绝由session unit验证，公开仅校验合法早cell effects与后cell声明错误不重放旧cell。

本轮T20交付后收口，无automation；格式化器只有owner评估，不新增未批准命令/静态检查。R304 OPEN，不声称性能原线通过或正式发布。

## 最终公开验收（2026-11-11）

冻结 Windows 候选 `target/release/lfz.exe` 为 1100800 字节，SHA256 `a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c`。首轮公开矩阵通过 34/36，原始结果 `final-first-default.json` 和原脚本 `script-first.txt` 已保留。

首轮两项测试预期有误。`x+=1` 赋值语句静默，后一行读取 `x` 输出 `2`。现有语法规定块注释不可嵌套，因此原用例应报 SyntaxError，随后仍输出 `8`。经 team-lead 确认修正这两项，并新增普通跨行块注释成功用例。

```text
D:/app/ANACONDA/python.exe -B tests/t20_cli_repl.py --binary target/release/lfz.exe --expected-sha a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c --output docs/reports/T20-blackbox-evidence/final-corrected-default.json
```

同一产物默认环境完整复验 **37/37 PASS，exit 0**。公开证据覆盖旧 cell 来源、输入消费、历史效果只执行一次及超长 pending 的尾部无副作用。`manifest-final.json` 封存两轮证据、脚本与产品初末 SHA。验后 39 个来源指纹匹配构建者冻结清单。TTY、宿主容量与性能由对应验收记录报告，R304 OPEN。
