# verifier原始审查证据

这些文件从项目Temp/verifier-benchmark-v2原样复制；b07-fixed.json为冻结版再次执行原反例的记录。保留b07-initial.json初验FAIL和所有故意错误控制的原始FAIL/INVALID_SOURCE，不能将这些控制解释为学生成绩。未运行正式八学生。

- independent-case-review.json：180/180独立案例/输入域检查。
- independent-controls.json：9/9独立执行器控制（含默认10秒超时、真实IO拒错和报告保护）。
- b07-initial.json / b07-fixed.json：原反例修前FAIL→冻结版PASS。
- cli-self-test.json：verifier独立执行冻结版CLI自测，131例，RUNNER_SELF_TEST_ONLY。
- missing/wrong结果与CLI日志、io-*.json、output-preservation.json：完整拒错输出，包含真实退出码和stdout/stderr。
- audit_cases.py / audit_runner.py / review_b07.py / timeout-default.lfz：原验证脚本和超时夹具，未改写历史脚本。

JSON中的Temp路径与命令是执行时原始记录，不要求这些临时目录在新checkout存在。本目录原始JSON可直接阅读。若重放：先从项目根设置PYTHONIOENCODING=utf-8，建立Temp/verifier-benchmark-v2，复制这些脚本到该目录，执行run.py --self-test --lfz target/release/lfz.exe --output Temp/verifier-benchmark-v2/cli-self-test.json（需先按仓库流程重建匹配冻结hash的解释器），然后python -B Temp/verifier-benchmark-v2/audit_cases.py及audit_runner.py。输出文件名已存在时应改用新独立临时目录/新报告名，保留旧证据。review_b07.py在当前已修版本运行应PASS，不能用新输出覆盖本目录的初验FAIL。

未复制临时解释器副本lfz-copy.exe；它只是manifest路径错配控件。未清理任何Temp文件，未更改冻结九文件。