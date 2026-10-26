# T12 1.4.0 候选工作快照提交计划（非正式发布）

2026-10-25，release-manager。team-lead决定保存可复现的候选checkpoint，允许后续由主会话提交并推送main；本角色尚未stage/commit/tag/push。**非正式发布，不创建v1.4.0正式标签，不移动旧v1.3.0。** R-304开放：最终19原绝对线15达线、3环境不确定、1未通过；不放宽线、不标需求完成。最终47ED功能已获verifier独立PASS：522 Rust、166黑盒/91进程、应用37+11、140witness，另16个旧新差分覆盖8类while范围且stdout/stderr/exit逐字节一致。结论限功能，R-304不改完成。

## 当前功能证据

| 材料 | 已有结果 | 边界 |
| -- | -- | -- |
| `T12-release-evidence/final-opt2/cargo-test-release.txt` | 最终Rust release 522/522：404+48+28+12+15+15 | 已绑定47ED并经verifier最终复验，原517阶段日志保留 |
| `T12-blackbox.md`与原始目录 | 正式166/166、进程91/91 | 91含一次全量黑盒，不虚增独立用例数量 |
| `T12-independent-verification.md`与原始目录 | 阶段独立140 witness及功能/应用结果 | 47ED最终522/166/91/140/应用及16个旧新while差分独立PASS，新增原始材料T12-independent-final/；发布经理只引用实际回执 |
| `T12-app.md` | jsonflow37、sortviz11通过；默认字节一致、150排序自测无失败 | JSON整批内存处理；输出别名/IO非原子写入限制见应用README |
| `T12-guide-evidence.md`、`T12-skill-evidence.md` | 人类35代码块+10探针；AI30探针 | 是作者实跑/指南回归，不是新Agent盲测成绩 |
| `T12-performance.md` | 最终15/19达线，3环境不确定、1map8k未过；struct8k约77.3%降时，numeric500k104.940→90.804ms | R-304开放；可保存候选checkpoint，不作正式版本发布 |

Cargo.toml/Cargo.lock为1.4.0、仅std；最终target/dist均759808B，SHA256 `47ed2bfe2a2694b11bd611172cfcf0000d0bfc58e3b9bd809ca95f066dd0261f`。release profile明确opt-level=2，保留LTO/codegen1/strip；最终构建/测试/Cargo与源文件hash见`T12-release-evidence/final-opt2/`。此前FB8与while优化候选、CPU/静默控制及profile A/B原始失败材料全部保留，不能作为最终版本结论。

README与delivery-checklist已补72内置、argv/-e/stdin、严格确定性JSON、jsonflow及sortviz参数事实和证据链接，明确待发布/效率未过。历史T11 69/494/130和旧hash保留历史语境；旧v1.3.0标签不移动，项目外安装只读。

## 只读预检

旧准备快照`T12-release-evidence/preflight.json`（171路径）保留历史。新的候选快照见`T12-release-evidence/checkpoint-preflight.json`，明确stage建议逐路径列表见`T12-release-evidence/commit-groups.json`。每个文件SHA只绑定生成时点，活跃verifier产物和状态收口后须刷新；禁止照搬旧hash作最终完整性证明。主会话可在开始提交前从项目根执行`python -B docs/reports/T12-release-evidence/preflight-check.py`刷新当前未提交路径，开始提交后不将其误用为全历史审计。证据目录.gitattributes禁止行尾归一，保全日志/快照原始字节。

- 原`tests/cases.json`99条逐项解码比较全部保留，新增32条；原有LFZ/Rust测试未修改。3个测试说明/清单文件变化符合范围；新测试另增。
- 冻结`docs/validation/agent-benchmark-v2/`及历史`v2-20261024-r01/`没有Git内容改动，旧manifest SHA仍`f2046eadca7c9f55376cbac000ccd53b25958298d10e2fc176ed0a14867c2a3e`，旧原始ZIP SHA仍`90a6a3e3d894e7a813b06463506d5cc540e0a4570d7b005a3fd27ccc6a776d84`。
- 开发skill四文件和target二进制已变化，是预期的新版本产物。不能据旧manifest宣称当前开发目录仍冻结；旧复现应从ZIP恢复到项目Temp，后续测评另建输入版本，不修改旧成绩/交卷答案。
- 文本凭据模式扫描未发现私钥头、常见GitHub/OpenAI/AWS令牌模式；这不是全知敏感信息证明。证据中项目绝对路径、工具链路径和本机环境字段为复现信息，未发现项目外文件正文或账号凭据混入本轮变更。
- 未stage、未Git写入；Temp/target/dist均为忽略构建/临时目录，不应强制入库。检查只读取当前项目与Git历史。

## 建议原子提交顺序

最终功能独立回执到位后，主会话可按明确路径保存候选checkpoint，性能R-304仍开放不妨碍如实记录工作快照，但禁止正式v1.4.0标签。避免`git add .`；每组stage前审实际diff与索引。表中的目录是关注点分组，实际路径列表取commit-groups.json并核对所有新增产物。

| 顺序 | 建议提交信息 | 范围 |
| -- | -- | -- |
| 1 | `docs(spec): define agent data workflow contracts` | docs/spec三件套、T12-design及design-checks、REQUIREMENTS中本轮验收定义；ADR由第8组过滤快照处理 |
| 2 | `feat(runtime): add json and argument contexts` | builtins/error/evaluator/value/lib、新json_codec、对应runtime证据；含最终性能修复但须在说明中写清 |
| 3 | `feat(cli): add inline and stdin script execution` | cli/main、cli_t12、runner-contract、docs/tooling/evidence/T12；旧测试未改 |
| 4 | `test(agent): cover json and script entry points` | tests新增LFZ组/fixtures、cases追加、coverage/REPORT、t12_cli_check与黑盒及独立功能证据、T12-cargo-full |
| 5 | `feat(app): add configurable json data workflow` | app/jsonflow、fixtures、两个verify脚本、sortviz、app README/DEV_RECORD及T12-app证据 |
| 6 | `docs(guide): teach agent json script workflows` | 人类五指南、docs/guide/ai/README、skill四文件及guide/skill实跑证据 |
| 7 | `perf(runtime): record bounded workflow benchmarks` | benchmarks/t12、T12-performance与runtime-performance-diagnostic及最终性能原始证据；失败样本保留 |
| 8 | `chore(checkpoint): save 1.4.0 candidate evidence` | Cargo版本/lock、README、delivery-checklist、T12-release-plan与构建/最终验收证据、全部团队本轮记录；共享3文件用过滤快照 |

若实现修复跨第2/7组，应让第2组含实现及必要回归，第7组只承载测量，不拆成无法构建的提交。第1组需求状态在最终收口前须由requirements-analyst核对；不要提交“完成”与证据冲突的版本。最终独立报告归属第4或8组，按实际产物落盘后选定。

## 三处用户初始修改保护

本轮开始前就存在DECISIONS 28行、PROJECT_STATE 1行、team-lead STATUS 1行事故记录。保留工作树全文，不以`checkout/restore`覆盖它们，不整文件add这3个文件。

仅在项目Temp生成stage专用UTF-8快照，移除以下**唯一匹配**的初始段落后，`git hash-object -w --path=...`及`git update-index --cacheinfo`写入索引；工作树原文不变。preflight确认各正则恰1次，匹配数变化必须先查明原因。

```text
DECISIONS: (?ms)^### \[2026-09-27 23:55\].*?(?=^### \[2026-10-24)
PROJECT_STATE: (?m)^- \*\*⚠️ 崩溃转储（2026-09-27 事故，ADR 已记）\*\*：[^\n]*\n
team-lead STATUS: (?m)^- \*\*🚨 崩溃压测会吃磁盘（2026-09-27 事故，已立 ADR）\*\*：[^\n]*\n
```

全部提交后，`git diff --numstat`应只剩上述初始28/1/1新增，任何其它残留都须核对，不以“干净”目标删除用户原改动。所有Git写操作由主会话代执行，避免子agent审批悬挂；本任务没有申请或执行Git写入。

## 候选checkpoint与正式发布的界限

1. 当前19原绝对线15达线、3环境不确定、1map8k未过，R-304保持开放。struct8k93.431→21.198ms、numeric500k104.940→90.804ms、startup16.287<=16.307ms是局部条件实测；不能用这些收益抵消失败，也不无限择窗继续追绿。
2. verifier已完成最终47ED的522/166/91、应用、140witness及16个旧新while作用域差分；README/清单引用最终功能PASS。原早期FB8结论保留原阶段语境，不冒用为当前产物。
3. 主会话随后按8组明确列表创建候选源码/证据提交并推送main，核对远程HEAD。允许保存工作快照不等于需求R-304已完成；不把候选commit称为正式版本发布。
4. 本轮不打v1.4.0正式标签，不移动旧v1.3.0；后续若满足开放门槛，另行批准正式发布。实际commit/push回执出现前本文不虚构哈希或推送成功。
5. 初始用户三处28/1/1行仍留工作树，不入本轮提交；新增/变更路径以最后刷新清单为准，禁止丢掉新证据或提交项目外内容。