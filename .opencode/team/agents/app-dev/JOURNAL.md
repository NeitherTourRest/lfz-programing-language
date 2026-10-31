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
## [2026-10-29] T15 jsonbatch应用
- 来源：team-lead按§18新attempt契约派发坏JSON混批应用。
- 完成：新增jsonbatch/夹具/Python独立oracle/说明/证据，旧83f新功能缺失红测保留，新1a5a候选38/38实跑通过。
- 变更：仅app新增T15产物与自身记录，旧sortviz/jsonflow不动，不Git。
- 下一步：verifier独立验收；整文件/非原子/路径别名限制已公开；R304仍OPEN。
- 阻塞：无。

## [2026-10-30] T16四模块应用准备
- 来源：team-lead按§21派发实际三文件以上模块应用。
- 完成：common/record/batch/main及33进程Python oracle、说明；AST PASS，未实跑修前C、不编译。
- 下一步：最终hash后实跑、manifest/证据；旧应用和旧oracle不改，不Git。
- 阻塞：待最终candidate，无应用实现阻塞。

## [2026-10-30] T16四模块应用作者最终验证
- 最终C cdedc9/902656B核hash，默认sandbox模块读取拒绝IOError保留；同字节许可提权33/33通过exit0，无产品绕过。
- app/t16-evidence/results/manifest、说明最终节及自身STATUS落盘；旧应用/旧oracle未改，不编译不Git。
- 下一步：verifier独立复跑，R304仍OPEN；无应用阻塞。

## [2026-10-30] T17预算应用验证准备
- 按§22新增专属验证器及实用命令/协作deadline与副作用边界说明，旧应用和oracle不改。
- AST与prepare-only通过，尚未最终产物实跑，无解释器启动/编译/Git；等待root最终hash。

## [2026-10-30] T17应用预算作者验收
- 最终1946ab9/943104B核hash；默认canonicalize权限拒绝保留，同产物许可提权40/40 PASS，exit0。
- callback内部预算28，0..28完整保留；正常等价/早期KEEP/完成IO不回滚覆盖。结果/manifest/最终说明齐，旧app及oracle不改，不编译/Git。
- 下一步：独立验证；R304仍OPEN，无应用阻塞。

## [2026-10-30] T18-A工具编排作者验收
- 新toolplan配置驱动绝对exe argv，attempt局部失败继续、全局资源终止穿透；本地fixture独占复制核hash。
- 最终4093714/984064B默认sandbox7/7 PASS；首份全绿后增加deadline真实exec行/帧断言，最终7/7再过，两个结果和manifest完整保留。
- 禁用不spawn/能力隔离/9任务argv及退出码/三局部错误继续/全局deadline/全体验证/空计划覆盖；无编译Git/旧app修改。
- 交独立verifier与发布收口，R304OPEN，用户边界本轮后停止，无应用阻塞。

## [2026-10-31] T18-B HTTP应用准备
- 任务：root限定B交付，新增JSON任务批处理与stdlib loopback独立oracle，旧app不改。
- 产出：httpplan/verify_httpplan/T18-B说明及属性；AST/prepare-only PASS，0解释器/无server启动。
- 下一步：最终hash后实跑7项，完整保留失败与回收证据；不公网/外装/系统信任改动/编译/Git，R304OPEN。

## [2026-10-31] T18-B HTTPplan最终作者验收
- 轻量启动读STATUS，核冻结18d328d9/1029120B；默认2/3失败保留（WinHTTP12029、零请求），正常OS许可同字节7/7通过。
- 请求显式参数/重复头/404与302正常/三局部失败继续/真实回调全局deadline不可吞/KEEP/前置验证/空计划及server完整关闭均有证据。
- 归档default-permission-failure/results/manifest，更新开发说明；无应用oracle修复、产品/旧app/编译/Git修改。交root及独立verifier，R304OPEN，本轮B后停止。
