# T19 AI指南作者证据（prepare-only）

2026-11-04：完整启动后按三spec §25、T19-contract及人指南line-processing.md同步五份AI文档。最终产品hash尚未收到，未启动任何LFZ进程，不把旧T18-B产物或旧v2成绩当T19通过。

## 当前产出

- `.opencode/skills/lfz-programming/{SKILL.md,README.md,VERIFICATION.md,prompt-template.md}`与`docs/guide/ai/README.md`。
- `docs/reports/T19-skill-evidence/run_examples.py`：22个小场景、两份逐字提取JSONL例子；数据与脚本全部项目Temp/T19-ai。
- prepare.json / prepare-second.json保留前后准备记录，prepare-final.json为最终文档准备SHA/fixture/source清单。首次提取重复#42仅准备期纠正，不是LFZ执行失败。

## 实际准备命令

`D:/app/ANACONDA/python.exe -B docs/reports/T19-skill-evidence/run_examples.py --prepare-only --output docs/reports/T19-skill-evidence/prepare-final.json`

prepare-only exit0，22小场景，0解释器启动，0网络启动。Python启动器打印“Failed to find real location”但继续执行到完整JSON输出，不把该提示当LFZ错误。

最终收到GO及hash后，同脚本加`--binary target/release/lfz.exe --expected-sha256 HASH --output docs/reports/T19-skill-evidence/results.json`，不带prepare-only。每个小样先check再run，另check-only marker见证；计划最多45次产品调用，不提前宣称45通过。每次启动核hash、15s外watchdog、保stdout/stderr/exit，失败不覆盖或删；五文档初末SHA对照。

## 预设覆盖和边界

提取例子有效JSON/坏JSON/空行连续后成功；仅count不collect。CRLF/EOF末CR、空/尾LF、BOM/NUL/空白/#42文本、严格UTF8跨8KiB、N/N+1含被剥CR、坏UTF8/超长回调前失败及已有count保留、false后坏行不处理、非bool TypeError、空文件错arity、cap范围/目录IOError、共享回调AST预算不可attempt吞。

普通文件pre/post metadata非沙箱/不消灭TOCTOU，OS同步阻塞非硬时界。native只取消/deadline，不按字节行扣AST；RAII统一释放但std Drop不报告OS close错误。真实同宿主关闭、取消/token、注入、跨module来源、规模峰值内存与吞吐由各owner/verifier/perf证据承担，本作者小样不冒称覆盖。未编译/改产品/spec/人指南/Git/旧v2，未占perf窗口。R304OPEN，本轮T19交付后停止，不T20/automation。
