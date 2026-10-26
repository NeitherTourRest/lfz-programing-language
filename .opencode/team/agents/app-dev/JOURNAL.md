# app-dev — 工作日志
> 只追加，最新条目在最上方。
## [2026-09-27] P8 交付：LFZ 应用「排序算法可视化」（sortviz.lfz）
- 来源: team-lead P8 任务书（交付物 6 / 评分项 5，30 分）
- 完成: 先读 SKILL.md + docs/spec + tests 样例；确认 LFZ v1 无模块/无 CLI 参数/无文件 IO；据此定单文件架构；用 LFZ 从零实现 5 算法 + 字符画逐帧 + Stats 统计 + assert/check 校验（341 行/有效 290）；写 README + 开发记录（6 轮迭代、踩坑、验证）
- 产出: `app/sortviz.lfz`（9705 B）、`app/README.md`、`app/DEV_RECORD.md`；实测 3 命令退出码 0；`cargo build` 0 warning；`cargo run --quiet -- test` 82/82；`git status` 仅 `?? app/`
- 决策: 单文件架构（无 import）；参数用源码常量（无 CLI 参数）；默认零 `input()`（EOF→IOError）；取整用 `div`
- 下一步: 配合 ppt-presenter 演示；等待 verifier 独立验收；已向 ai-dx-engineer 提 1 条指南补充建议
- 阻塞: 无
## [2026-09-22] 角色初始化
- 来源: 团队初始化（系统构建）
- 完成: 角色定义与活文档建立
- 产出: `.opencode/agents/app-dev.md`
- 下一步: 等待 team-lead 调度

## [2026-10-25 01:04] T12实用应用与sortviz参数化
- 来源: team-lead T12任务书，用户自主方向授权。
- 完成: 阅读LFZ编程skill与spec§16；jsonflow配置筛选/投影、严格验证后输出；sortviz四参数与range/map生成；README/开发记录同步。
- 产出: app/jsonflow.lfz、fixtures/t12三文件、verify_jsonflow.py、sortviz.lfz、verify_sortviz.py；docs/reports/T12-app.md与T12-app-evidence。
- 证据: release自检37/37+11/11，均exit0；默认sortviz与旧源码+旧exe stdout逐字节一致，150自测0失败；sortviz362行/311有效行。
- 决策: 整批JSON数据流，不凑新应用200行；输入输出别名识别受API限制，README公开，验证失败保留输出不等于原子IO。
- 下一步: 独立验收与统一发布。
- 阻塞: 无。