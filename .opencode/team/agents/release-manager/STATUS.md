# release-manager — 工作状态
> 最后更新：2026-10-24 17:31 by release-manager

## 当前状态
Agent benchmark v2 测评包通过独立QA，可按两组提交；正式八学生Agent测评未运行。已只读复核交付清单、冻结哈希与根README入口，等待主会话统一提交/推送。本角色不执行git/网络，不改冻结包。

## 产出 / 证据
- 根README入口已指向TASKS.md，明确v2新题集、不与旧2/8直接比较、正式测评未开始。
- 包内13个顶层文件：TASKS.md、STUDENT_PROMPT.md、cases.json、run.py、manifest.json、manual-review.schema.json、README.md、QA.md、QA-evidence.json、QA-final-evidence.json、self-test-evidence.json、self-test-evidence-final.json、REVIEW.md；review-evidence/19个原始证据文件。
- REVIEW.md结论可交付；131固定case、180独立断言、9控制PASS。此为测评工具与协议QA，不是学生成绩。
- 本角色独立用Get-FileHash逐项核对manifest的9个冻结条目，均吻合；manifest SHA256=f2046eadca7c9f55376cbac000ccd53b25958298d10e2fc176ed0a14867c2a3e；根README入口存在。
- 未改Cargo/dist/tag；前批解释器仍1.0.1、dist766464B，既有历史2/8保留。

## 建议提交分组（由主会话执行）
1. test(benchmark): add reviewed agent benchmark v2 — docs/validation/agent-benchmark-v2/全包，含review-evidence/，保留早期及最终QA证据。
2. docs(team): link benchmark v2 and record readiness — README.md、TEAM_BOARD.md、PROJECT_STATE.md、DECISIONS.md及lead/test-engineer/verifier/release-manager本轮STATUS/JOURNAL；仅暂存本轮改动。

## 边界 / 下一步
- 第二组对初始3处旧事故改动仍需采用安全staged快照：DECISIONS去除09-27 23:55/23:58两段；PROJECT_STATE去除历史崩溃转储bullet；team-lead STATUS去除历史崩溃压测bullet。工作树原文保留，核对剩余diff应恰这3处。
- 主会话完成提交/推送后记录最终哈希；不新增tag。后续按冻结协议另行启动正式八Agent，不能把runner自测或QA通过当作正式通过率。

## 关键经验
- 新题集单独建基线，历史2/8不用于计算跨题集改善比例。
- 冻结manifest覆盖9件；README/schema/REVIEW等额外文件仍随包归档，不擅改冻结清单。
- 早期缺包内README的准备态已解决，正式入口与完整包均已核对。