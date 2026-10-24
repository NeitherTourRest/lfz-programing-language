# test-engineer — 工作状态
> 最后更新: 2026-10-24 17:26 by test-engineer

## 当前状态
新固定八题测评包完成，待独立verifier。没有运行八个学生Agent，不报告盲测成绩。

## 最近完成
- docs/validation/agent-benchmark-v2/cases.json：131固定case，B01..B08分别16/18/15/16/16/17/17/16。
- run.py：--solutions/--lfz/--output、每进程10秒、缺题逐case MISSING、源码/案例/程序/输出hash；B08真实字节、输入不变、两轮污染输出检查覆盖。
- 字面量正确处理Unicode、LF/CR/tab、引号/斜线、美元插值与NUL；B06任意账户键用entries数组比较。
- manifest.json冻结TASKS/STUDENT_PROMPT/cases/run/skill四文件/解释器共9文件，不含自身；最终verify_freeze通过。
- README.md、QA.md、manual-review.schema.json明确自动FUNCTIONAL_PASS仍REVIEW_REQUIRED，最终源码与协议由verifier另行合并。
- QA-final-evidence.json：131参考检查与runner控制自测全部成功，正确识别错答/缺函数/非零/非法前导/缺8题/超时/状态泄漏/BFS改输入/IO错误及输入被改。常量stub仅Temp自测控制，不是LFZ参考解。
- 字节：cases134479/run21266/README6152/QA5288/manifest1231/schema2255/最终自测21171。
- run SHA256=5a698248db184d091bd0a41e46b861f241e7f8f2a1051cef640640671eb9e862；cases=919cfe6905ef4f4bc003d0321fe4398aab631be0ac8a0873f03c2761af6f22cd。
- 原tests/源码/spec/guide未修改，不git、不派agent。
- 保留前批T11-13成果：4正例组63assert+36负例，130/130黑盒连跑两次exit0（已交接）。

## 进行中
无。

## 阻塞
无。Python安装路径有提示但实际执行exit0。run.py最终完整非空，已从Temp完整编译后原子替换。

## 下一步
独立verifier审查固定用例/runner/manifest与manual-review schema。正式盲测须另经用户授权派学生，冻结后不能看到答案再改标准。

## 关键经验
- B07先ASCII regex提词再lower，不可Unicode.lower全文后提词（K/İ等会混入ASCII）。
- LFZ不支持JSON的u转义，美元要转义；任意控制字符需chr拼接，动态struct键比较改entries。
- Windows长命令有32K上限，分批写入；先Temp生成compile再os.replace，避免并发读到truncate文件。
- JS replace替换字符串中美元单引号有特殊含义，应使用回调或切片。
