# test-engineer — 工作状态
> 最后更新: 2026-10-25 by test-engineer

## 当前状态
T12黑盒与CLI进程验证完成，待verifier独立验收。

## 产出与证据
- 4组LFZ正例64 assert，32 fixtures负例；tests/cases.json只追加，原99条清单解码内容逐项保留，原130测试保持。
- release1.4.0正式166/166 PASS、0 FAIL/ERROR；tests/t12_cli_check.py独立进程91/91 PASS（含一次正式全量，不重复计数）。
- docs/reports/T12-blackbox.md与T12-blackbox-evidence/完整原始证据；tests/REPORT.md与coverage-matrix.md当前快照已同步，历史保持。
- release hash fb8e5653827de1e1c3f73b81d37f21314ad41adfdc4d99f37cb8ca5f6491f47d。脚本e48077dfbb29e19c73bf745a281affbd4539b1a72eb49b01a479657ee4179db2。
- 命令：D:/app/ANACONDA/python.exe -B tests/t12_cli_check.py --lfz target/release/lfz.exe --output Temp/T12/test-engineer/final-cli.json --full-suite；exit0。

## 进行中/阻塞
无。未改实现/spec/指南/冻结v2材料，不Git，不委派。

## 下一步
verifier独立复验；如后续binary变化，由team-lead通知同脚本复跑，不覆盖既有证据。

## 关键经验
- 旧内置不是一等函数，JSON映射要lambda包装；仅argv本轮新增一等取值。
- 操作系统参数无法承载NUL，严格JSON错误输入应经UTF8文件传入并归档原字节。
- 首次作者把insert当struct操作，依据spec改为del所得新struct更新；不属于产品缺陷。原始失败留Temp且报告透明记录。
- stdout/stderr须由Python直接捕获UTF8字节，避免PowerShell解码影响中文证据。
