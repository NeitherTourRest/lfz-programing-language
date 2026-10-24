# LFZ Agent Benchmark v2 — 独立审查

verifier；2026-10-24 17:27（Asia/Shanghai）。只验证，不修复。结论：**本轮可执行测评包通过审查，可交付；正式八学生 Agent 测评未运行。** 这是新基线准备，不与历史2/8作同题比较。没有学生源码的人工PASS，没有八题正式通过率。

## 被审版本与冻结

本报告仅以工程师声明最终就绪后的文件作结论；写入中的瞬时0字节/截断读取不作为稳定版本缺陷。未修改运行器、用例、题目、提示词、manifest或其他角色交付物。

- run.py：21266字节；SHA256 `5a698248db184d091bd0a41e46b861f241e7f8f2a1051cef640640671eb9e862`。
- cases.json：131例，B01..B08分别16/18/15/16/16/17/17/16；SHA256 `919cfe6905ef4f4bc003d0321fe4398aab631be0ac8a0873f03c2761af6f22cd`。
- manifest.json本身SHA256 `f2046eadca7c9f55376cbac000ccd53b25958298d10e2fc176ed0a14867c2a3e`，独立调用verify_freeze(target/release/lfz.exe)通过。
- TASKS.md SHA256 `8d7201425f9c6addab689036bfd2071765ed781f248221e38509c994544c5c42`；STUDENT_PROMPT.md SHA256 `e6edf5debad31fb0efe7e9091bde20f8fb5a17e0dec3adf678fef2556630497f`。
- 被验解释器target/release/lfz.exe SHA256 `149b184e442ffbf250590431fa0aeffab401eac526faf60b4b435f620725d5ab`。
- 人工审查schema SHA256 `0008f2dc43ab89a7437a77c9d9fae4c1566859991997c97ec4df69177ed2f36d`，README SHA256 `e9bec5d58a0a96ba687bd752bac0e527565de0083621a50c18d7298cbe143478`。两者不在manifest九文件内，本报告记录其审查版本；正式归档时与本报告一起保留。

## 题意、可行性及验收一致性

八题均能用当前LFZ已有语法/69内置实现。无需新语言特性；表中“通过”是题目与验收协议审查，绝非学生解答评分。

| 题 | 审查结论 | 依据/边界 |
|---|---|---|
| B01 | 通过 | 非法输入排除；优先级/左结合/一元、负数向下整除与精确i64约束明确；16例覆盖手算边界、16层和256字符 |
| B02 | 通过 | LF/CRLF、包裹字段内原样换行、双引号转义、空输入/空行/尾逗号口径互不冲突；18例含2048字符 |
| B03 | 通过 | 固定8桶和码点和hash是源码约束；仅行为无法证明桶结构。已补单键≤128标量，15例含NUL和碰撞 |
| B04 | 通过 | 上右下左、首次入队标记/前驱、首尾和无路明确，维度1..20；不可变约束有运行器复查和人工检查 |
| B05 | 通过 | 二维DP按原顺序、平局不选当前物品、反向回溯唯一；已补所有DP候选和累计价值不溢出i64；小输入独立穷举最优值/平局mask |
| B06 | 通过 | 未知账户先建立、拒绝不部分扣款、同账户足额/不足、独立调用明确；已补id长度；任意键含NUL用entries结果比较 |
| B07 | 修复后通过 | 必须先按原文ASCII字符切词再小写；非ASCII均分隔。独立原文扫描验证全部17例，含K/İ/ſ反例 |
| B08 | 通过 | UTF8无BOM、固定LF、覆盖旧内容、输入保持、返回count/total明确；父目录已存在且路径不同；中文空格路径真实读写验证 |

独立审查脚本额外核对180个断言：B07全部案例的原文ASCII扫描与query/text边界；B05选择集合重量/价值及小案例穷举tie、规模/容量/i64上界；B08以LF分隔的独立精确输出与边界；CSV期望、B03/B04/B06输入域等。180/180 PASS。这些断言与131例存在交叉，不能把180解释为新增正式验收用例。其余各题的整套期望另经冻结运行器reference自检，131例均一致。

## 实际执行证据

全部命令从项目根目录执行，`PYTHONIOENCODING=utf-8`、Python `-B`，临时文件仅项目Temp。Python启动打印本机“Failed to find real location …”诊断，但脚本可实际运行；以真实退出码/断言结果判定，未将此诊断当成功证据。

### 冻结版本CLI自测

~~~powershell
python -B docs/validation/agent-benchmark-v2/run.py --lfz target/release/lfz.exe --self-test --output Temp/verifier-benchmark-v2/cli-self-test.json
~~~

退出0，stdout原文：

~~~json
{"kind":"RUNNER_SELF_TEST_ONLY","ok":true,"cases":131,"work_directory":"Temp/benchmark-v2-selftest-7elwu0np"}
~~~

实际控件结果：positive=PASS；wrong/missing_function/nonzero=FAIL；invalid_preamble=INVALID_SOURCE；8题缺失全部计入MISSING；跨调用状态泄漏和grid输入变更被拒；字面量UTF8真实字节往返含NUL/控制字符成功；B06任意键/NUL结构比较成功；timeout_detected=true；io_positive=PASS，io_wrong/io_input_mutation=FAIL。错误IO两个控件解释器均exit0，仍因外部字节/输入校验被拒，证明不会仅凭进程成功判题。

这是verifier独立执行工具自测，不是采信工程师自述，也不是学生题目答案。

### 额外独立控件

命令：`python -B Temp/verifier-benchmark-v2/audit_runner.py`。退出0，stdout：

~~~json
{"checks":9,"passed":9,"failed":[],"manifest_sha256":"f2046eadca7c9f55376cbac000ccd53b25958298d10e2fc176ed0a14867c2a3e"}
~~~

| 控件 | 实际结果 |
|---|---|
| 空交卷目录调用正式CLI | exit1；8题/131例MISSING；functional_pass_tasks=0，分母8 |
| 仅B01提交solve返回nil，其他缺题 | exit1；B01全部FAIL；certified_pass_tasks=null；每题NOT_CERTIFIED/REVIEW_REQUIRED |
| 默认超时真实无限循环（非0.2秒缩短参数） | 10.013815秒终止；timed_out=true，exit=null；没有故意崩溃 |
| B08真实readFile+writeFile、固定样例窄stub | 两进程PASS，返回值及逐字节输出一致、输入未变；此stub仅控制验收器，不是题解 |
| B08改用appendFile写相同内容 | 两进程FAIL，预污染输出未被覆盖 |
| B08输出对但返回total错误 | FAIL，断言拒绝 |
| B08输出CRLF而非LF | FAIL，外部精确字节拒绝 |
| 复制相同解释器内容到另一项目内路径验manifest | 拒绝Frozen manifest mismatch；未修改冻结文件 |
| 对既有报告重复指定--output | exit2，output exists；拒绝覆盖原证据 |

B08独立控件还核对所有合并程序路径解析后均在项目Temp内；CWD保持项目根目录，传入路径均指项目Temp内、包含中文与空格的夹具。未证明运行器为安全沙箱，也不作这种声称。

独立案例命令：`python -B Temp/verifier-benchmark-v2/audit_cases.py`，退出0，stdout `{"checks":180,"passed":180,"failed":[]}`。

## 初验缺陷与闭环

**REVIEW-V2-01（已关闭）：B07参考先Unicode lower再按ASCII提词。**

初验命令 `python -B Temp/verifier-benchmark-v2/review_b07.py`；输入 `text="aKB"`、queries=["a","b","akb"]。

- 期望 `[["a",1,[0]],["b",1,[1]],["akb",0,[]]]`。
- 初验实际 `[["a",0,[]],["b",0,[]],["akb",1,[0]]]`，FAIL，已立即上报team-lead，修复由test-engineer负责。
- 冻结版先在原文提取ASCII词后再lower，新增B07-17；独立原文扫描及同一反例PASS。

另两处任务书澄清已在冻结前闭合：B05累计价值不溢出、B03/B06键长有界；超时文案明确每进程10秒，B08两进程每case至多20秒。未降低现有答案标准，正式学生尚未交卷。

## 人工约束、隔离与评分审查

**协议设计审查通过；具体学生源码/过程审查尚未发生。**

- STUDENT_PROMPT只给单题附件/四skill文件/解释器/独占目录，要求fork_turns=none、不传项目历史、禁止委派/联网/父目录读取；明确目录限制只是提示词约束，实际访问须记录审查，不能冒称OS隔离。
- 20分钟和最多5次LFZ调用、不可覆盖首跑快照/attempts.jsonl及输出、模型与过程unknown不得当成功，规则清楚；本运行器不负责强制学生开发预算，正式调度者须执行与留证。
- B03桶结构/hash、B04 BFS、B05 DP/tie、B07先建索引、硬编码/外部代算等不能靠有限输出证明，必须源码审查。功能全PASS仍输出REVIEW_REQUIRED/NOT_CERTIFIED，独立实测已验证。
- manual-review.schema.json绑定功能报告hash、manifest hash及各题源码hash；README要求B01..B08各一次、人工核对唯一性和hash。schema自身不执行这些跨字段规则，README已明确该限制。最终手工合并仅功能全PASS+源码PASS+协议PASS且hash相符才计整题PASS；缺失/UNKNOWN/FAIL不计，分母恒8。不能把schema存在称为学生已通过审查。
- 学生自选首跑测试，首次自行执行成功率仅为过程辅指标，不能替代冻结验收集结果。正式结果应额外保存原始快照/访问记录、提示词/模型/版本、时间/调用预算。

## 原始证据归档与保全

按team-lead最后收口任务，原始JSON和必要脚本已归档至本包 [review-evidence/README.md](review-evidence/README.md)，新checkout可直接读取。原始Temp记录仍保留且未清理。JSON内的Temp路径是历史执行记录；文件本身已随包保存。

| 文件 | SHA256 |
|---|---|
| b07-initial.json | d0c308d1e4fea23e61d2118fc50d5ad1c563cf00e35c831e0ffbe1a512ad43dc |
| independent-case-review.json | 2f1af8b896bdef66c9eb334b0e3447f96816fc7db71d6c3c28010e28eb342a3c |
| independent-controls.json | 43e620c14a5974714dcca4163046cb1aa6224150f34b8438274efce7c438acb3 |
| cli-self-test.json | 453c91f1f3c8649e1a900b9d364ff536142b1891498ac424d216b49426dc6968 |

**最终状态：测评工具功能审查PASS；题目/协议设计人工审查PASS；正式学生提交功能结果未运行、学生源码/隔离预算人工审查未运行，八题正式成绩未产生。** 无本包交付阻塞。

### 收口归档（2026-10-24）

- [180项独立核对](review-evidence/independent-case-review.json)、[9项独立控件](review-evidence/independent-controls.json)、[独立CLI自测](review-evidence/cli-self-test.json)。
- B07原反例：[初验FAIL](review-evidence/b07-initial.json) → [冻结版复验PASS](review-evidence/b07-fixed.json)，原失败未覆盖。
- [案例检查脚本](review-evidence/audit_cases.py)、[运行器检查脚本](review-evidence/audit_runner.py)、[原B07反例脚本](review-evidence/review_b07.py)。
- [空交卷结果](review-evidence/missing-results.json)、[错误交卷结果](review-evidence/wrong-results.json)、[IO正常](review-evidence/io-good.json)、[错误append](review-evidence/io-append.json)、[错误CRLF](review-evidence/io-crlf.json)、[错误返回](review-evidence/io-wrong-return.json)。
