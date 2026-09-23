---
description: 团队状态汇报 — team-lead 汇总全体成员状态并输出结构化报告
agent: team-lead
---

# 团队状态汇报（/standup）

你现在的任务是汇总整个团队的当前状态，并输出一份结构化的状态报告。请按以下步骤执行：

## 第一步：读取共享状态文档

用 Read 工具依次读取：

1. `.opencode/team/PROJECT_STATE.md` —— 项目全局状态、当前阶段、里程碑进度、交付物对照表、风险与开放决策
2. `.opencode/team/TEAM_BOARD.md` —— 任务看板（进行中 / 待办 / 阻塞 / 已完成）

## 第二步：读取全体成员状态文件

用 Read 工具依次读取全部 14 个成员的状态文件（路径固定，缺一不可）：

1. `.opencode/team/agents/team-lead/STATUS.md`
2. `.opencode/team/agents/requirements-analyst/STATUS.md`
3. `.opencode/team/agents/language-architect/STATUS.md`
4. `.opencode/team/agents/core-dev/STATUS.md`
5. `.opencode/team/agents/runtime-dev/STATUS.md`
6. `.opencode/team/agents/tooling-dev/STATUS.md`
7. `.opencode/team/agents/test-engineer/STATUS.md`
8. `.opencode/team/agents/perf-engineer/STATUS.md`
9. `.opencode/team/agents/verifier/STATUS.md`
10. `.opencode/team/agents/docs-writer/STATUS.md`
11. `.opencode/team/agents/ai-dx-engineer/STATUS.md`
12. `.opencode/team/agents/app-dev/STATUS.md`
13. `.opencode/team/agents/release-manager/STATUS.md`
14. `.opencode/team/agents/ppt-presenter/STATUS.md`

## 第三步：输出结构化汇报

汇总以上信息，按以下固定格式输出报告（不要遗漏任何一节）：

```
【项目阶段】
<当前所处阶段 + 一句话说明>

【看板摘要】
<进行中任务数 | 待办任务数 | 阻塞任务数 | 已完成任务数，并点出关键任务>

【各成员状态一览】
| 角色 | 当前状态 | 进行中 | 阻塞 |
|------|---------|--------|------|
| team-lead | ... | ... | ... |
| requirements-analyst | ... | ... | ... |
| ... （14 行，一个成员一行） |

【风险与阻塞】
<从 PROJECT_STATE.md 的风险与开放决策、各成员 STATUS 的阻塞字段汇总；无则写“无”>

【下一步建议】
<基于当前阶段与看板，给出接下来应优先推进的事项>

【需用户决策】
<列出所有需要用户拍板的开放决策；无则写“无”>
```

## 约束

- 只读取、不修改任何文件（本命令是只读汇报，不要更新看板或状态文档）。
- 如实反映各成员 STATUS.md 的内容，不虚构、不美化。
- 若某成员 STATUS.md 缺失或与看板冲突，在【风险与阻塞】中如实标注并向用户说明。
