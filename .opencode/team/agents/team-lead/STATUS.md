# team-lead — 工作状态
> 最后更新: 2026-09-24 by team-lead

## 当前状态
**P0/P0.5/P2/P3 全部完成，里程碑 `v0.2.0` 已打标签并推送。**
- P0 团队就绪 ✅｜P0.5 版本基线（本地 + 远程）✅｜P2 语言设计冻结（`docs/spec/` v1 三件套，ADR D-016）✅
- **P3 核心实现（20 个子阶段）✅ 收官**：`loader`/`lexer`/`ast`/`parser`/`value`/`env`/`evaluator`/`builtins`（§10.7 **54/54**）/`cli`。
- **P3.11 独立验收三轮**：rev.1 **FAIL（3×🔴）** → rev.2 CONCERNS → **rev.3 PASS（10/10 缺陷闭环、0 回归）**。
- 质量基线：`cargo build` **0 warning**；`cargo test` **361 passed / 0 failed / 0 ignored**（+ bin 9 + cli 7 = **377**）。
- 端到端：`cargo run -- run examples/hello.lfz` → `Hello, LFZ!`；缺 `#42` → 退出码 2 + `CosmosAnswerError`。
- 远程：**https://github.com/NeitherTourRest/lfz-programing-language**（Public）；`main` = `931e2b2`；标签 `v0.1.0` / **`v0.2.0`**。

## 进行中
- （无。等用户下令进入 **P4 工具链** / **P5 黑盒测试集（评分项 2，20 分）**）

## 阻塞 / 需要支持
- （无）
- ⚠️ 待用户知悉（历史遗留，非阻塞）：仓库 owner = 认证账号 login **`NeitherTourRest`**（display name `MakeChase`），非字面账号 `MakeChase`。

## 下一步计划
1. **P4 工具链**（tooling-dev）：REPL、`--json`、`lfz test` 一键 runner（T-R1..T-R4 契约）、打包脚本。
2. **P5 黑盒测试集**（test-engineer，评分项 2 = 20 分）：`tests/*.lfz` 全特性用例（每特性 ≥3）+ 覆盖矩阵；三处目录隔离（`tests/*.rs` Rust 集成 / `tests/lfz/**.lfz` 黑盒 / `tests/fixtures/` 负例）。
3. **P6 性能**（perf-engineer）：LFZ vs Python；**必查项**——P3 采用**运行时查名**（无 resolver），P6 需评估并可能补 resolver/槽位缓存。
4. P7 文档（docs-writer ‖ ai-dx-engineer）→ P8 应用（app-dev，排序算法可视化）→ P9 验收（verifier）→ P10 发布/答辩（release-manager + ppt-presenter）。

## 关键经验（写给未来的自己）
- **"绿了 ≠ 对"**：373 单测全绿时解释器**连多行函数都跑不了**。真正的闸门是**独立跑真实程序**（verifier 的 CLI 端到端），不是"跑测试"。→ 关键交付物必须独立复验。
- **真假完成防线**：把「**必须贴出文件实际字节数**」写进任务书验收项 —— 连续抓出 **3 次假完成**（P3.3/P3.4b/P3.4b1 都是"跑完不落盘"）。
- **失败模式与对策**：某执行者在**大而开放**的任务上会推演到超时；**切成 ≤1 模块的小批次**后 6 批全部一次落地。原负责人连续 2 次不交付即**改派**（parser 改派 Sisyphus-Junior/`unspecified-high`，改派全程写进看板）。
- **执行者不可用时的兜底**：曾发生「2 任务同时超时 + 架构师会话 `Insufficient Balance`」，工作区留下 **2 warnings + 3 failed** 的半成品。我的处置：**先恢复绿灯**（保留写对的部分、回退夹带的高风险改动），再谈新增。**宁可推迟一个 🟡，也不篡改既有测试**（bug-07 即如此）。
- **测试接缝**：parser 单测**手工构造 token 流、绕过 lexer** → `Colon`/`newline` 接缝无人校验。新增回归一律**先 `lexer::lex` 再 `parse`**。
- **规范自相矛盾也算缺陷**：§9.4 样例自身用了被禁止的单个 `;`，使唯一引用语义样例跑不通 → 走「先 ADR 后改 spec」修正。
- **gh 与 git 的代理不通用**：gh 不读 git 的 `http.proxy`，须注入 `HTTPS_PROXY`。
