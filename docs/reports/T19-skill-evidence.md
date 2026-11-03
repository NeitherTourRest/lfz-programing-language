# T19 AI指南最终作者证据

2026-11-04：最终产品 `target/release/lfz.exe`，1041920B，SHA256 `5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5`。

最终文档作者聚焦实跑 **45/45，exit0**：22小样逐个check→run，加1个check不执行marker见证。所有启动核同一hash；五文档初末SHA一致，marker未创建。最终数据见`T19-skill-evidence/final-results.json`，含完整参数/stdout/stderr/exit及源码/数据SHA。前次同候选`results.json`也是45/45；随后只补文档状态措辞，故另跑最终45次确认最终SHA，不合计成90个不同用例。没有LFZ失败，原prepare三次JSON与准备快照均保留。

命令：`D:/app/ANACONDA/python.exe -B docs/reports/T19-skill-evidence/run_examples.py --binary target/release/lfz.exe --expected-sha256 5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5 --output docs/reports/T19-skill-evidence/final-results.json`

Python启动器打印Failed to find real location后正常完整执行，不是LFZ失败。单独运行器exit0；后续PowerShell Get-Process lfz无结果（不存在活LFZ进程）导致组合命令状态1，不是验证器失败。

## 文档与覆盖

四skill与AI入口§6.8，prepare原文/原脚本/原报告完整保存在`T19-skill-evidence/prepare-snapshot/`。大文件JSONL示例以外部struct cell及一次parseCurrent closure复用，callback更新cell.text再attempt(parseCurrent)，两份文档逐字提取后实际输出accepted2/rejected2/total3/lines4/stoppedfalse。避免逐行新建UserFn导致func_names诊断元数据增长；reader有界不等于任意callback或总解释器内存恒定。未改变产品funcID语义。

小样实际覆盖CRLF/EOF末CR、空/尾LF、BOM/NUL/空白/#42、UTF8跨8KiB、raw LF前含CR的N/N+1、坏UTF8/超长回调前失败及已有count保留、false后坏行不处理、非bool/空文件arity/cap/目录IOError、全局AST预算不可attempt吞。JSONL复用原jsonParse，坏行/空行继续。无新parser/权限/context/CLI，旧readFile不变。

不把小样当同宿主关闭/注入/取消token/跨源/吞吐峰值内存的独立验收；这些由对应owner承担。reader RAII统一释放但stdDrop不报告OS close错误；普通文件TOCTOU及同步阻塞不构成硬时界/沙箱。未编译/改产品/spec/人指南/Git/旧v2/网络/规模大测；当前无活LFZ进程，交还perf窗口。R304OPEN，仅T19收口不T20/automation。

## 最终五文档SHA256

- `.opencode/skills/lfz-programming/SKILL.md`: `5fa57665fb73f04507494f821ac40cc463a70bf50cae636cfee7d2e808f084db`
- `.opencode/skills/lfz-programming/README.md`: `1f9b4935c41e5aaca7762caf99f24e5be9f5f63c0311136a30d739c22928363a`
- `.opencode/skills/lfz-programming/VERIFICATION.md`: `e59d077998b8254feb5f232c0c54fc2f53b274a5e0f01c07b1835c70609e31bc`
- `.opencode/skills/lfz-programming/prompt-template.md`: `3100b63e9ef00be5381647764fa012384be067814f51b041bf97c8ff187a9c72`
- `docs/guide/ai/README.md`: `49a32b5a2b5135951d565e2695da2a34a732688deef6b1da4f68fd08a1f7a019`
