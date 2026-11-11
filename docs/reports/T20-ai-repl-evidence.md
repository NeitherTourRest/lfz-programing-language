# T20 AI REPL与v3审计准备

五AI文档新增§26 REPL说明及formatter有据暂缓；8个聚焦pipe场景（两份提取例、multiline、错误继续、旧source、EOF、input、argv）prepare-only完成，0LFZ启动。待最终hash GO：python -B docs/reports/T20-ai-repl-evidence/verify.py --binary target/release/lfz.exe --sha256 HASH --output docs/reports/T20-ai-repl-evidence/results.json。

v3补student_cycle.py：每周期源码快照/核exe SHA/check后run/UTC/deadline/最多5周期；不证明无旁路，完整宿主trace仍必需。audit_budget.py纯逻辑9/9控制通过：40/41、5/6、10/11、1200/1200.001、顺序/访问违规、缺trace/缺process核对UNKNOWN。不是学生测评或独立审计通过。

trace_export.py复用旧v2思路，只读显式关联session、核session/parent，导出工具调用/返回与实际model/effort，不导出私有推理/系统/developer/无关会话。实际本人关联session只metadata探针可读取，但当前302call/300result、匹配不完整（运行中/长期session），因此不能声称当前已取得完整学生trace或协议PASS。未来正式学生结束后必须核全部call_id/results、函数exec内子启动与wrapper日志交叉核对；实际model见probe原字段，无法取得则UNKNOWN。

按独立预审修：C02完整导出crash与flag argv原样；C01新增33有效后UTF8/超cap，验32已flush保留、未flush不输出；C04公开selftest_service.py供学生自调例外、runner server_closed必须True才functional PASS。原draft manifest保manifest.pre-review.json，新manifest仍DRAFT，待verifier独立实跑参考解/负控/服务关闭false负控及trace审计。未派学生/未运行产品，未编译/Git/改产品/人指南/旧v2。R304OPEN，仅T20不自动化。

## 2026-11-11 最终 AI 指南实测

最终解释器 1100800 B，SHA256 `a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c`。8/8 pipe 场景通过，五份文档的 SHA 在运行前后相同。证据为 `T20-ai-repl-evidence/results.json` 和 `manifest.json`。

```powershell
python -B docs/reports/T20-ai-repl-evidence/verify.py --binary target/release/lfz.exe --sha256 a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c --output docs/reports/T20-ai-repl-evidence/results.json
```

v3 C04 首轮失败为 oracle 名称覆盖 Python http 模块，原日志 `Temp/T20-verifier/default-ref-C04.log`，原脚本 `Temp/T20-ai/runner-before-http-fix.py`。现以 expected_http 别名消除冲突。新增工作区准备脚本和派发模板，待独立控制签收后由主调度冻结。当前草稿仍保留待签收状态。
