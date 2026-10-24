# 基准 v2 执行器与固定案例 QA

日期：2026-10-24。负责人：test-engineer。结论：固定测评包与执行器自测完成；**没有执行八个学生Agent，没有盲测成绩，也没有最终整题PASS认证。** 源码算法/隔离协议留给独立verifier。

## 交付与固定边界

cases.json为固定JSON，8题共131例；B01/B02/B03/B04/B05/B06/B07/B08分别16/18/15/16/16/17/17/16例，均≥12。用例包含题面上下界、精确大整数、Unicode与NUL键，不读取学生方案，不提供LFZ算法参考解。Python参考仅在run.py供验收数据自校验。

B07已按独立审查意见修为先[A-Za-z]+提词再lower，包含aKB、İx、ſY回归；不能先对全文Unicode.lower，否则把非ASCII分隔符变成ASCII词。LFZ字面量转义$、引号、斜线、LF/CR/tab；额外控制字符用chr(n)拼接，真实IO回读证明含NUL的UTF8字节一致。B06用entries数组核对任意账户键，避免动态键不合法字面量。

## 可复现命令和结果

从项目根目录：

~~~powershell
& 'D:/app/ANACONDA/python.exe' -B docs/validation/agent-benchmark-v2/run.py --self-test --lfz target/release/lfz.exe --output Temp/agent-benchmark-v2-selftest-review.json
~~~

运行前该输出路径须不存在。实际最终自测使用--freeze --self-test，输出QA-final-evidence.json；终端exit0：

~~~json
{"kind":"RUNNER_SELF_TEST_ONLY","ok":true,"cases":131,"work_directory":"Temp/benchmark-v2-selftest-plcvqqyn"}
~~~

最后另调verify_freeze(target/release/lfz.exe)成功，确认TASKS/STUDENT_PROMPT/用例/运行器/四skill/解释器仍与冻结记录相同。运行器语法在项目Temp完整文件上用Python compile(...,"exec")检查后原子替换，最终文件21266B，非空。Python可用，终端有安装路径提示“Failed to find real location”，未阻止运行，实际进程退出0并产生完整JSON。

## 自测观测

| 检查 | 结果与实际证据字段 |
|---|---|
| 131项固定期望、输入边界 | references八项均reference_matches=true |
| B01独立手算向量 | 16项期望逐一一致，包括负数floor与大整数 |
| B02 csv、B04邻居次序、B06自转账、B07 Unicode、B08样例 | 独立witness断言通过 |
| B05小案例 | 穷举所有子集，最佳价值及tie所选下标与二维DP一致；24项边界不做指数穷举 |
| 正确常量stub | controls.positive=PASS，证明断言机制可成功 |
| 错答、显式fail | controls.wrong/nonzero=FAIL，stderr为AssertionError |
| 无solve | controls.missing_function=FAIL，stderr为NameError |
| 缺#42 | controls.invalid_preamble=INVALID_SOURCE |
| 八题程序全缺 | missing_tasks_counted=8；每case MISSING，功能分母仍8；不是实际成绩 |
| 超时 | timeout_detected.timed_out=true、exit=null；自测用0.2s，正式每进程10s |
| 跨solve调用状态泄漏 | state_leak_detected=FAIL、AssertionError |
| BFS输入被改 | grid_mutation_detected=FAIL、AssertionError |
| 文件输出正确 | io_controls.io_positive=PASS，两次真实进程/每次污染旧输出 |
| 文件输出错误 | io_controls.io_wrong=FAIL，均exit0但output_matches=false |
| 输入文件被改 | io_controls.io_input_mutation=FAIL，均exit0但input_unchanged=false |
| NUL账户key及字节编码 | ledger_nul_harness=PASS，literal_bytes_roundtrip.exit=0且外部原字节比较一致 |

所有控制stub只在项目Temp/benchmark-v2-selftest-*/，从未当作学生程序或正式成绩。B08每case两进程、每进程timeout10秒，合计最多20秒，符合最终TASKS口径；每次输出逐字节检查且输入hash对照原数据。

## 冻结与证据哈希

最终manifest.json绑定9个文件，不含自身；README/QA/自测JSON不纳入循环hash。当前冻结使用target/release/lfz.exe（766464B）。详细路径和全部文件hash见manifest.json。

| 文件 | SHA256 |
|---|---|
| run.py | 5a698248db184d091bd0a41e46b861f241e7f8f2a1051cef640640671eb9e862 |
| cases.json | 919cfe6905ef4f4bc003d0321fe4398aab631be0ac8a0873f03c2761af6f22cd |
| manifest.json | f2046eadca7c9f55376cbac000ccd53b25958298d10e2fc176ed0a14867c2a3e |
| QA-final-evidence.json | 22a760f219b8cb14eb37476227f2adf8d416e29bc019374d6ff802140d04a470 |

QA-final-evidence.json是最新稳定运行器的完整自测JSON（21171B），不是正式评测报告。其他self-test-evidence*.json/QA-evidence.json为保留的开发阶段记录，绑定较早manifest，不据其作最终核验。

## 审查与实际限制

自动FUNCTIONAL_PASS仍为source_review/protocol_review=REVIEW_REQUIRED、overall_task_status=NOT_CERTIFIED。最终成绩须独立verifier使用manual-review.schema.json记录源码算法与协议PASS并匹配报告/manifest/源码hash后另行合并；缺失/UNKNOWN不得通过，分母恒8。run.py不自动认证盲测或整题成绩。schema定义结构，八个task各出现一次及证据真实性仍由verifier核对。

本包是遵守隔离协议的程序功能验收器，不是操作系统恶意代码沙箱。无法仅用黑盒证明自写算法、禁止外部程序/硬编码、5次调试/20分钟/未见验收集，这些必须人工证据审查。固定131例也不是所有输入正确性的证明。原tests/源码/spec/guide未改，不执行git操作，不派学生Agent。
