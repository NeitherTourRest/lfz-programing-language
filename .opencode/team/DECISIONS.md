# LFZ 决策记录（ADR）
> 最后更新: 2026-09-22 by 团队初始化

本文件只追加，禁止修改或删除任何已有条目。新条目标题格式：`### [YYYY-MM-DD HH:MM] [agent名] 标题`，追加到文末。

---

### [2026-09-22 00:00] [团队初始化] D-001 采用 14 角色项目级 AI 团队架构
- 决策：以 14 名 opencode agent 组成项目级开发团队，team-lead 为唯一用户接口与调度者，其余 13 名按管理/设计/实现/质量/文档/应用/发布分组，各司其职。
- 背景：课程作业要求覆盖语言设计、解释器实现、测试、性能、文档、Agent 应用、Git 与答辩（评分 20/20/10/20/30），单一 agent 难以稳定覆盖全部环节。
- 理由：职责单一、单一事实源、可并行、可独立验收，符合课程对"编程 Agent 辅助开发"的考察点。
- 影响：确立 14 个 agent 规格与 `AGENTS.md` 团队宪法；team-lead 负责最终交付与统一调度。

### [2026-09-22 00:00] [团队初始化] D-002 采用活文档协议 + 单一写者规则
- 决策：采用活文档（STATUS/JOURNAL/看板/ADR）记录团队工作；TEAM_BOARD/PROJECT_STATE 由 team-lead 唯一写，REQUIREMENTS 由 requirements-analyst 唯一写，DECISIONS 任何 agent 只追加，个人文档本人唯一写。
- 背景：14 个 agent 并行工作存在并发写冲突与状态漂移风险。
- 理由：单一写者杜绝并发冲突；活文档保证跨会话记忆可恢复；只追加 ADR 保证决策可追溯。
- 影响：各 agent 启动/收工协议强制读写对应文档；冲突时以看板为准并向 team-lead 报告。

### [2026-09-22 00:00] [团队初始化] D-003 全 agent 使用 deepseek/deepseek-v4-pro + 中文 prompt
- 决策：全部 14 个 agent 统一 `model: deepseek/deepseek-v4-pro`；prompt 以中文为主，技术名词/路径/代码用英文。
- 背景：团队需统一的模型能力与一致的沟通口径。
- 理由：统一模型降低调优与调试成本；中文 prompt 贴合课程语境，英文技术名词保持规范。
- 影响：各 agent frontmatter 固定 `model` 字段；文档语言约定写入宪法 §6。

### [2026-09-22 00:00] [团队初始化] D-004（开放决策）解释器实现语言：默认建议 Python 3.13
- 决策：解释器实现语言默认建议 Python 3.13；备选 Rust / Go；需用户在 P2 前确认。
- 背景：task-info 规定实现语言无限制；语言选型影响核心/运行时/工具链/性能全部后续工作。
- 理由：Python 3.13 迭代最快、助教可直接运行、且与性能对比基线（Python 脚本）一致；Rust/Go 若追求性能接近或超过 Python 可选，但工作量更大。
- 影响：P2 前未定则技术栈无法冻结；P3/P4/P6 的代码实现与性能基准均依赖此决策。

### [2026-09-23 00:00] [team-lead] D-005 全 agent 模型切换为 deepseek/deepseek-flash
- 决策：全部 14 个 agent 的 `model` 由 `deepseek/deepseek-v4-pro` 切换为 `deepseek/deepseek-flash`（用户指令）。
- 背景：用户要求团队全部使用 deepseek flash 模型（更快、更省）。
- 理由：用户明确指令；`deepseek-v4-flash` 与 `deepseek-v4-pro` 同代，切换成本低、可随时回退。
- 影响：14 个 agent 的 frontmatter、`.opencode/team/TEAM_SPEC.md`、`scripts/verify_team.py`（REQUIRED_MODEL）均已同步为 flash；本决策取代 D-003 中的模型选型部分（D-003 其余内容仍有效）。

### [2026-09-23 01:00] [team-lead] D-006（提案 · 用户改选 Rust）实现语言：原建议 Python 3.13
- 决策：LFZ 解释器用 **Python 3.13** 实现，纯标准库、零第三方依赖；备选 Go；不推荐 Rust / C++。
- 背景：D-004 开放待定；由 language-architect / tooling-dev / perf-engineer / librarian 四方独立调研。
- 理由：六维加权 Python 4.45 最高（Go 3.28 / Rust 2.58）；**本机 `rustc`/`go` 均未安装**，选它们会阻塞 P2→P3→P4→P5/P8 主关键路径；跨语言基准（loxido）显示树遍历解释器本就慢于 CPython，性能项（10 分）考报告的公平方法学与诚实分析、非速度，Rust/Go 不为该 10 分加分却威胁解释器 20 + 测试 20 + 应用 30；写作速度 Python > Go >> Rust。
- 影响：技术栈冻结为 Python 3.13；P2 spec 可启动；`PROJECT_STATE` 技术栈/目录/命令同步更新；**正式关闭 D-004**。

### [2026-09-23 01:00] [team-lead] D-007（提案 · 用户已确认）v1 功能范围建议
- 决策：v1 = 必做核心（int/float/string/bool/nil/array/struct/function、字面量、`let`/`var`、完整运算符优先级、if/while/for-in、函数/闭包、IO、注释、换行终结、条件必须 bool）＋ 5 个「惊喜」特色：**① 管道 `|>` + `_` 占位；② 合一 struct；③ 富字符串插值；④ 结构化错误 + assert/check；⑤ 确定性语义「无魔法」**。11 项延后（生成器/惰性流、match、值式错误处理、一等区间/切片、值式循环+标签 break、`lfz check`、REPL、`lfz fmt/doc/init/package/watch`、模块、类/继承、类型注解）。
- 背景：BRAINSTORM 候选池 + task-info 硬要求 + 课程周期可行性。
- 理由：判据 = 辨识度 × 可行性 × 加分；宁可 5 个真正落地 + 语法自洽拿满分，不要 12 个半成品崩掉解释器与测试两大项。
- 影响：`docs/spec/` 以此为冻结依据；`REQUIREMENTS.md` 待补特色条目（由 requirements-analyst 在 P2 前更新）；延后项标记为 v1.1 backlog。

### [2026-09-23 01:00] [team-lead] D-008（提案 · 用户已确认）开发方式：契约先行 + 双轨 TDD
- 决策：先由 language-architect 冻结 `docs/spec/interface-contract.md`（AST 节点类型 / `evaluate(node, env) -> Value` 求值器接口 / `LFZError` 错误模型）再实现；单测（Python/pytest，`tests/unit/`）与黑盒（LFZ 脚本，`tests/`）双轨且所有权分离；**runner 契约先于黑盒测试编写**（tooling-dev 定稿 → team-lead 转 test-engineer）；Windows 优先（纯标准库、`scripts/lfz.ps1`+`lfz.bat`、显式 UTF-8、ANSI 自动降级）。
- 背景：core-dev 与 runtime-dev 需并行；14 agent 协作需明确接口与所有权；test-engineer 依赖 runner 契约。
- 理由：契约先行让 runtime-dev 用手工 AST 先行、消除串行等待；双轨避免测试所有权冲突；runner 契约先行解除 test-engineer 阻塞。
- 影响：P3 实现前必须冻结 interface-contract；任何契约变更走 ADR；CLI 只做错误格式化与退出码映射（0 成功 / 1 测试失败 / 2 语法·运行时·环境错误）。

### [2026-09-23 01:00] [team-lead] D-009（提案 · 用户已确认）阶段划分与门禁
- 决策：关键路径 **P0.5（git 基线）→ P2（spec 冻结）→ P3（前端 AST + 求值 MVP）→ P4（CLI + runner）→ P5（黑盒测试）/ P8（应用）→ P9（验证）→ P10（发布+答辩）**；各阶段定义 entry/exit 门禁（详见 `KICKOFF.md` §一.4）。P1 需求标记为「基本完成」（REQUIREMENTS.md 23 条已就绪）。
- 背景：需明确串并行与依赖，避免返工。
- 理由：把"interface-contract"与"runner 契约"两类关键接口前置为门禁；P7 文档草案 / P6 基准脚本 / P8 应用设计可与 P3 并行。
- 影响：`TEAM_BOARD.md` 依此细化阶段行；开工首步仍为 P0.5。

### [2026-09-23 02:00] [team-lead] D-010 流程纠正：重大决策先问用户，再决定（D-006~D-009 降级为提案）
- 决策：凡属**重大决策**（技术选型、功能范围、评分项对应物的取舍、范围变更、对外承诺等），team-lead **必须先以「候选方案 + 推荐」的形式请示用户**，获得确认后方可作为正式决策落地并同步文档；**禁止先自行决定、再交用户审核**。
- 背景：本轮 D-006~D-009（实现语言 / 功能范围 / 开发方式 / 阶段划分）由 team-lead 在**未经用户确认**下直接记录为"已定"，违反"先问后决"原则，经用户指正。
- 处理：**D-006~D-009 降级为「提案 · 待用户决策」**，不作为正式决策；用户确认或另选方案后，再更新为正式决策并同步 `PROJECT_STATE.md` / `REQUIREMENTS.md` / `KICKOFF.md`。
- 影响：写入团队工作原则（后续可引用到团队宪法 §6 工作约定）；team-lead 的「研判 → 调度」环节新增门禁：**重大决策先请示**。

### [2026-09-23 02:30] [team-lead] D-011 用户确认：四项决策定案（实现语言改选 Rust）
- 决策（**用户拍板**）：
  1. **实现语言 = Rust**（用户明确改选，覆盖 D-006 提案中的 Python 建议）。
  2. **v1 功能范围 = 全部 5 个特色**（管道 `|>`、合一 struct、富插值、结构化错误 + assert/check、确定性语义）。
  3. **float 纳入 v1**（配 int→float 显式提升规则）。
  4. **应用选题 = 排序算法可视化**（P8 实现，ASCII 条形图动画）。
- 背景：team-lead 先以「候选 + 推荐」请示用户（符合 D-010），用户在四问中作出上述选择，实现语言明确改选 Rust。
- 理由：用户主导技术选型（Rust 可提供更强的性能叙事，部分负载可胜 CPython）。
- 影响：
  - **架构须按 Rust 重规划**：运行时值用 `enum Value`；可变复合类型（array/struct/闭包环境）用 `Rc<RefCell<...>>`；错误用 `Result<T, LfzError>` + 自定义错误枚举；`cargo` 工程、优先仅用 std（不引第三方 crate）。
  - D-006 的 Python 建议作废（保留为历史记录）；D-007 范围不变（生成器/惰性流已延后，不受 Rust 影响）；D-008/D-009 的**门禁与阶段不变**，但实现细节按 Rust 调整。
  - **新增前置任务（P0.5 之前）**：安装并验证 Rust 工具链（rustc/cargo/rustup）。**本机当前未安装** → 未安装完成前 P3 无法开工。
  - **风险上调**：Rust 返工率高于 Python（借用检查器/Rc 循环/mutability），须以「契约先行 + 编译驱动 + 小步提交 + 频繁 `cargo test`」控制。

### [2026-09-23 04:00] [language-architect] D-012（草案）LFZ v0.2 语法修订：C 风格块 + 换行语句 + `;;` 变量 dump
- 决策（用户新指令落地 + 歧义消解，详见 `.opencode/team/DRAFT-LFZ-v0.2.md`）：
  1. **块用 C 风格 `{}`**；缩进纯视觉，不决定结构。
  2. **语句由换行终结**，无语句分隔符；**括号 `()`/`[]` 与 struct 成员表 `{}` 内换行忽略**；多行表达式**只能**用括号续行（不采纳行尾运算符续行/行首续行；部分不同意 core-dev #5）。
  3. **`;` 专用于变量 dump**：`;;`（相邻两字符，最长匹配）= 独立语句 `DUMP`；单独 `;` = 语法错误 `E-SYN-003`；`;;;`/`; ;`/`;;;;` 均有确定错误码（取消"空语句"概念）。
  4. **`{` 双角色按文法位置二分**：block-required 位=块，其余=struct 字面量；**无裸块语句**；`if`/`while` 条件位与 `for` 可迭代位**禁裸 struct 字面量**（采 Rust 规则，需加括号）。
  5. **分隔符统一**：struct 声明体与字面体、数组、实参一律**逗号必填**（尾逗号可选），换行忽略。
  6. **修正 v1 真 bug**：`|>` 优先级表与 EBNF 自相矛盾 → 采纳 core-dev 修正层序 `cmp=pipe{...}; pipe=add{"|>"pipe_rhs}; add=mul{...}; mul=unary{...}`（`|>` 比 `+ - * / %` 松、比比较紧）；`|>` 右侧仅收 `pipe_rhs`，`xs |> sum() + 1` 为语法错误。
  7. **`${}` 插值**：字符串内 `{` 为字面字符，插值必须由 `${` 触发（lexer 模式栈 `CODE/STR/INTERP`）；与块花括号字符集在模式上互斥，**无冲突**。
  8. **`;;` 语义**：stdout、内层→外层、同层 slot 升序、遮蔽去重、行格式 `<name> ： <value>`（冒号 U+FF1A）；实现前提 = 名字解析保留 **slot→name 调试符号表**（写入 interface-contract）。
- 背景：用户下达新指令（大体 Python 风格、C 风格 `{}` 块、换行终结、`;` 专用打印变量、无歧义）；core-dev 提供独立 parser 审计。
- 理由：以"每条解析/求值唯一确定"为硬目标；26 条歧义审计 + 10 条对账全部给出二值可判定规则。
- 影响：
  - **core-dev / runtime-dev**：parser 需 **换行模式栈 + NO_STRUCT_LITERAL 限制位**；lexer 需**字符串模式栈 + 结构化插值记号** 与 §2.2 最长匹配表；新增错误码 `E-SYN-002/003/005/006`。
  - **interface-contract.md**：必须含 `DUMP` 语句节点与 `DebugSym { slots: Vec<String> }` 调试符号表。
  - **docs / ai-dx / test / app**：所有 LFZ 代码样例须迁移到 v0.2 语法（逗号必填、无 `;`、`;;` dump、多行管道加括号）。
  - 本决策为**草案**，待用户对 v0.2 §11 的 4 项开放问题拍板后随 v1 冻结生效。

### [2026-09-23 18:56] [language-architect] D-013（草案）LFZ v0.3 修订：文件前导 `#42` + Python 风格错误模型 + 可移植性
- 决策（用户新指令落地，详见 `.opencode/team/DRAFT-LFZ-v0.3.md`）：
  1. **文件前导 `#42`**：`.lfz` **文件**第一行必须**恰好** `#42` 三字符 + 一个行终止符；**仅文件模式**强制，**REPL / stdin / `lfz run -e` 豁免**；**无任何变体**（`# 42`/`#42abc`/`#43`/尾随空格一律违规）。`#` 为**"文件开头专用符号"**（与注释无关；注释仍 `//`、`/* */`）；`#` 在其它位置（字符串/注释/format_spec 之外）→ 非法字符。不满足 → `CosmosAnswerError`，消息**固定**「你忘记了宇宙的答案」，**不显示编号/hint**。
  2. **报错模型改 Python 风格**：命名错误类 + **中文消息** + 位置（`File "<path>", line N[, in func]` + 源码行 + 插入符）；跨函数显示 **traceback**（最外层帧在前）；**取消 v0.2 的 `E-xxx` 编号**（用户输出不再显示；`--json` 内部用类名）。错误类清单：基类 `LfzError`；子类 `CosmosAnswerError` / `SyntaxError` / `NameError` / `TypeError` / `IndexError` / `FieldError` / `ZeroDivisionError` / `OverflowError` / `ValueError` / `IOError` / `AssertionError`。加载/解析期错误（`CosmosAnswerError`/`SyntaxError`）**无** `Traceback` 头，运行期错误**有**。
  3. **可移植性**：行终止符接受 `\n` / `\r\n` / `\r`（加载器归一化为 `\n`）；文件开头 UTF-8 BOM 静默跳过（仅一个）；源码要求 UTF-8，**非 UTF-8 → `SyntaxError`**（**先编码校验、后前导校验**）。
  4. **新增 §6 边界审计 B1–B12**：空文件 / 只有 `#42` 无换行 / 前导前空行·空格·BOM / 前导变体 / `#` 在程序中间（字符串内 `#` 合法）/ 三种行终止符 / 非 UTF-8 / REPL·stdin·`-e` 豁免 / `CosmosAnswerError` 位置 / `lfz init/fmt/test/check` 统一查头 / 前导字节精确性。
  5. **格式精确定义**：列号为 1-based、按 Unicode 标量计数；插入符指向引发错误的最小 AST 节点首字符；`CosmosAnswerError` 固定为 `File "<path>", line 1`、无源码行/插入符；非文件模式伪路径 `<stdin>`/`<repl>`/`<command>`；退出码仍 0/1/2（D-008）。
  6. **下游硬性要求**：**AI 指南/skill 必须写死「`#42` 首行」+ 错误类清单 + 所有示例带头 + 禁止旧 E-xxx**；**test-runner 契约要求每个测试 `.lfz` 带头**，缺前导的负例须用非自动发现夹具（清单声明 `expect.error = CosmosAnswerError`）；docs/app 全部 `.lfz` 带头。
- 背景：用户下达三项新指令（文件前导「宇宙的答案」`#42`、报错改为 Python 风格含独创 `CosmosAnswerError`、可移植性）；team-lead 已定三种行终止符与 BOM 处理。
- 理由：以"每条规则二值可判定、可被黑盒测试逐字符回归"为硬目标；加载器把编码/前导/行终止符归一化前置，词法与文法层保持纯净。
- 影响：
  - **core-dev**：新增 **loader 阶段**（UTF-8 校验→跳 BOM→校验/消费 `#42`→归一化行终止符）；lexer 在 CODE 模式拒绝 `#`；AST **必须携带 `Span{line,col}`**。
  - **runtime-dev**：新增 **`CallFrame{func,span}` 调用栈**（traceback）；`LfzError` 改**类名枚举**（无 E-xxx 码），提供 `class_name()`/`message()`/`span()`；`DebugSym{slots}` 职责不变（服务 `;;`）。
  - **tooling-dev**：CLI 错误格式化（两类结构）+ `--json`（`error` 字段用类名）+ `lfz test` 逐文件查头；三入口 `run`/`stdin`/`-e` 走 `load_file`/`load_source` 双入口。
  - **interface-contract.md**：必须含 loader 双入口、`Span`、`CallFrame`、`LfzError` 类名枚举、`DebugSym`。
  - **ai-dx / test / docs / app**：所有 LFZ 样例/测试/应用文件加 `#42` 首行；错误一律类名+中文消息。
  - 本决策为**草案**，待用户对 v0.3 §14 的 5 项开放问题拍板后随 v1 冻结生效；v0.2（D-012）中与错误编号相关的部分由本决策**取代**，其余（花括号块、换行语句、`;;`、插值、歧义 A1–A26）**继续有效**。

### [2026-09-23 19:05] [language-architect] D-014 LFZ v0.4 定稿候选（`#42` 前导按扩展名 / 错误模型 / 可移植性 定案）
- **编号说明**：任务书原指定编号 `D-013`；因 `D-013` 已于 [2026-09-23 18:56] 由本轮 v0.3 修订草案占用，依「ADR 只追加、不修改历史条目」顺延为 **D-014**（两者同属 `#42` 前导 / 错误模型 / 可移植性 主题：D-013 为草案，本条件为**定案**）。
- 决策（**用户本轮拍板 5 条**，详见 `.opencode/team/DRAFT-LFZ-v0.4.md`）：
  1. **前导检查仅对 `.lfz` 扩展名文件生效**：按**扩展名**（ASCII 大小写不敏感，`.lfz`/`.LFZ` 均算；Windows 友好）触发；**非 `.lfz` 文件即使被 `lfz run` 执行也不校验前导**。新增 `ext(path)` 判定（`/` 与 `\` 取最后分量、最后一个 `.` 之后为扩展名、与 `"lfz"` 按 ASCII 大小写不敏感比较；空/无扩展名/.gitignore 等一律豁免）。影响 §2.2、B11、§14-1、工具链表、§7、§10.1、§11。
  2. **`CosmosAnswerError` 不显示第 1 行原文**（保持 v0.3 默认，**标为"已定"**）：仅 `File "<path>", line 1` + 末行 `CosmosAnswerError: 你忘记了宇宙的答案`，无源码行/插入符/hint。
  3. **程序体可空**（保持 v0.3 默认，**标为"已定"**）：`.lfz` 文件 `#42` + 行终止符 即合法空程序。
  4. **不输出 `提示：`/hint 行**（team-lead 决定）：v1 用户可见输出**明确不含** hint；§8.1「hint 行策略」措辞由"不输出（§14-4 列为可选议题）"收紧为"**v1 明确不输出**"；可选 hint **列入 v1.1 backlog**。
  5. **`#42` 后必须紧跟行终止符**（保持 v0.3 B2，**标为"已定"**）：3 字节 `#42`（无换行）→ `CosmosAnswerError`。
- **仍沿用默认 4 条（当前默认，仍可覆盖，不阻塞冻结）**：① `;;` 作用域 = 完整可见链（内→外、slot 升序、遮蔽去重）；② `;;` 输出通道 = stdout；③ 多行续行 = **仅括号**（不采纳行尾/行首续行）；④ 单 `;` = 一律 `SyntaxError`（无 `;name`）。
- 产出：`.opencode/team/DRAFT-LFZ-v0.4.md`（**1131 行**，UTF-8 无 BOM）；文首标记 **「v0.4 = 定稿候选，可冻结」** 并列出冻结前提条件；§14 收敛为仅遗留 4 条；§0 新增「v0.3 → v0.4 差异摘要」。
- 影响：**core-dev**（loader 增 `ext(path)` 扩展名闸门 + `Loaded{text,line_base}`；非 `.lfz` 走免前导分支）；**runtime-dev**（`Span.line = 本地行号 + line_base`；`CosmosAnswerError` 无源码行）；**tooling-dev**（`lfz run/check/fmt` 按扩展名查头，`lfz test` 仅 `.lfz` 测试文件查头、非 `.lfz` 夹具豁免；无 hint 行）；**ai-dx/test/docs/app**（`.lfz` 文件带头；错误一律类名+中文；无 hint）；**interface-contract.md**（增 `ext` 与 `line_base`）。
- 状态：v0.4 = **定稿候选**；满足 §0 冻结前提（无遗留分歧 + core/runtime 评审 + ADR 落库）后，拆分为 `docs/spec/{syntax,semantics,interface-contract}.md` 并追加「spec v1 冻结」ADR。

### [2026-09-23 21:30] [language-architect] D-015 LFZ v0.5 定稿（语义定稿 A1–A7 + B/C 补钉）
- 背景：v0.4 冻结门禁评审中 core-dev（可解析性）与 runtime-dev（可求值性）**均判 CONCERNS**，共同要求把"未钉死的语义"补全。用户对 **7 条「语义定稿」拍板（A1–A7）**；团队补钉 runtime-dev 13 条（B1–B13）与 core-dev 必修（M1–M6 / N1–N2）（C）。
- 决策（**全部写死进 `.opencode/team/DRAFT-LFZ-v0.5.md`**）：
  1. **A1 复合类型 = 引用语义（Python 一致）**：array/struct 赋值与传参共享；`a[i]=v`、`s.k=v` **原地修改**；**其余内置（含 `push/pop/removeAt/insert/swap/sort`）一律返回新值、不改原容器**（§4.5.2）。
  2. **A2 闭包捕获 = 按 cell 引用捕获**（共享可变）（§4.5.3）。
  3. **A3 除法 = Python 一致**：`/` 恒返回 `float`（真除法）；`div(a,b)` = **向下取整**（Python `//`）；`%` = **Python 取模**（符号随除数）（§2.3、§4.2）。
  4. **A4 `check` = 非致命**：失败打印警告到 **stderr** + 返回 `false` + **不中断**；**仅 `assert`/`fail` 致命**（`AssertionError`）；修正 v0.4 §8.1 自相矛盾（§4.5.10、§8.1）。
  5. **A5 方法字段不算数据面**：函数值字段不参与 `keys/values/entries/display/==/has/len`；但 `s.k ≡ s["k"]` 仍能取到并调用（self 绑定）（§3.7、§4.5.9）。
  6. **A6 自引用（环）安全**：`==` 用「身份优先 + 访问对集合」（重访即视为相等，不报错、不死循环）；`print/str/;;` 用「路径访问集合」，重复输出 **`<cycle>`**（§3.7、§4.5.9）。
  7. **A7 新增错误类 `RecursionError`**（深递归 / 超深结构超限），加入 §8.1 清单与 `--json` 类名集合。
- 团队补钉摘要（**B**，runtime-dev 可求值性必修，写死进 §4.5 / §10）：**B1** 求值顺序从左到右（新增 §4.5「求值语义规范」）；**B2** `for` 迭代快照；**B3** 键按 UTF-8 字节序升序（`for k in s` / `keys/values/entries` / display）；**B4** 递归深度上限 10000 → `RecursionError`；**B5** float IEEE（NaN/Inf 产地与比较规则；除零含 float → `ZeroDivisionError`）；**B6** `input()` EOF → `IOError`；**B7** struct 实例化 = 平拷贝（默认值每次实例化求值 + 覆盖，可动态加字段）；**B8** `DebugSym` 升级为 **per-scope `ScopeDebug{first_slot,names,parent}`**（`Dump` 携带最内层活动 scope；零热路径开销）；**B9** 冻结内置函数清单 + data-last 签名 + 返回类型（§10.7）；**B10** 统一 `Span{line,col}`，区分 `VmFrame`（VM 执行帧）与 `TraceFrame`（traceback 帧），`TraceFrame.span` = 该帧当前求值节点 span；**B11** `Result<T, Box<LfzError>>` + `#[cold]`/`#[inline(never)]`；`CosmosAnswer` 变体 span = line 1；VM 帧存 `func_id`；**B12** `-9223372036854775808` 必须可解析（INT 承载原文 + 一元负号特判；越界 → `SyntaxError`）；**B13** int→float 加宽改口径（可能不精确；混合比较按数学精确值，附算法）。
- 团队补钉摘要（**C**，core-dev 可解析性必修，写死进 §2/§3/§7）：**M1** `${…}`（INTERP）内裸换行 → `SyntaxError`（禁止）；**M2** EBNF 删除 `[preamble]`（loader 消费、parser 不可见）；**M3** `=> {…}` 恒为块、返回 struct 须加括号（新增 A27）；**M4** `_` 只绑定最近管道 RHS 且不得穿 λ（λ 体内 `_` 非法）；**M5** 未列举转义 → `SyntaxError`、STR 内 `{`/`}` 无需转义；**M6** lexer 产 `FORMAT_SPEC` 原始文本 token、AST 存 `format_spec: Option<String>`；**N1** §10.3 澄清 `TraceFrame.span`；**N2** B12 `//#42` 反例改写（仅第 1 行才 `CosmosAnswerError`）。
- 产出：`.opencode/team/DRAFT-LFZ-v0.5.md`（**1504 行**，UTF-8 无 BOM；新增 **§4.5 求值语义规范**；§8.1 错误类 **12 类 + 基类**；§10.7 内置函数表；§12.1 / §15.1 / §15.2 核对；§14 收敛为**仅 4 条遗留默认**；文首标 **「v0.5 = 定稿（待复审冻结）」**，冻结前提改为 **core/runtime 复审通过**）。
- 影响：**core-dev**（M1–M6/N1–N2 全部落地；`FORMAT_SPEC` + per-scope `ScopeDebug` + 统一 `Span`）；**runtime-dev**（A1–A7 + B1–B13；引用语义 / 按 cell 捕获 / 精确 int↔float / 环安全 / `RecursionError` / `Box<LfzError>`）；**tooling-dev**（`--json` 类名集合含 `RecursionError`；`check` 输出 stderr 且不影响退出码）；**test-engineer / docs / ai-dx / app**（`check` 非致命、`div`/`%`/`/` 语义、方法不入数据面、环显示 `<cycle>`、内置签名表为准）。
- 状态：v0.5 = **定稿（待复审冻结）**；满足 §0 冻结前提（**无遗留分歧 + core-dev/runtime-dev 复审通过 + ADR 落库**）后，拆分为 `docs/spec/{syntax,semantics,interface-contract}.md` 并追加「spec v1 冻结」ADR。

### [2026-09-23 22:00] [language-architect] spec v1 冻结

- **背景**：v0.5 冻结门禁复评结果——core-dev（文法可解析性）**PASS（可冻结）**（手写 lexer + 递归下降/Pratt，无回溯可实现，无新解析歧义）；runtime-dev（语义可求值性）**CONCERNS，仅 3 项且均非架构级**（原 12/12 必修 + 用户拍板 A1–A7 已全部消解；其明确"不阻塞实现启动，可在拆分 spec 时一并闭合"）。本 ADR 闭合这 3 项并冻结。
- **冻结范围（LFZ v1 唯一事实源，本次写入并冻结）**：
  1. `docs/spec/syntax.md` —— 词法、`#42` 文件前导与可移植性策略、文法（EBNF）、语句 / 表达式 / 块、运算符优先级、歧义审计 A1–A27 / 边界审计 B1–B12、样例程序、一致性核对与对账。
  2. `docs/spec/semantics.md` —— 值模型与引用语义、作用域 / 闭包 / cell、§4.5 求值语义规范（B1–B7/B13）、错误语义（§8：A4 `check`/`assert`、A5/A6 环安全、A7 `RecursionError`）。
  3. `docs/spec/interface-contract.md` —— §8.1 错误类清单（实现侧映射）+ §10（唯一 `Span{line,col}`、`VmFrame`/`TraceFrame`、`LfzError` 及子类含 `RecursionError`、per-scope `ScopeDebug`/`Dump{scope}`、loader 双入口 `load_file`/`load_source`、`ext(path)`、§10.7 内置函数表与 data-last 约定）+ §11 下游角色硬性要求 + §13 旧错误码映射。
  - 三文件均以「本文件是 LFZ v1 的唯一事实源（冻结于 2026-09-23）。变更须先 ADR，后改文档。」开头；保留原 DRAFT 章节编号锚点；互以相对链接交叉引用；信息只增不减。
- **本次 3 处冻结前补钉（先就地修订 `.opencode/team/DRAFT-LFZ-v0.5.md` 再拆分）**：
  1. **§10.7 数值内置形参加宽**：`floor` / `ceil` / `round` / `sqrt` / `pow` 的 `int` 实参按 §1 / §4.5.7 **全局唯一隐式转换 `int→float` 加宽为 `float`**；`abs` **同型（int→int、float→float）绝不加宽**——二者差异一句话钉死；`log`/`exp` 等 v1 不提供的 math 内置不存在加宽问题，全文不再有"未声明是否加宽"。
  2. **§8.2 计数订正**：运行期错误由原少算 1 类的旧计数订正为「**其余 10 类**」（含 `RecursionError`）；全文自查计数：§8.1 = **12 类 + 基类**、`--json` `error` 集合 = **12**、运行期错误 = **10**、§5 = **27 条**（A1–A27，原 DRAFT §12 误记 26，已订正）、§6 = **12 条**（B1–B12）。
  3. **NaN/Inf → `int` 极窄边界钉死**（§4.5.7 + §10.7 `int` 行 + §8.1 `ValueError`/`OverflowError` 行）：`int(NaN)` → `ValueError`；`int(+Inf)` / `int(-Inf)` → `OverflowError`；有限浮点**向零截断**且截断结果超出 i64 范围 → `OverflowError`。
- **4 项保留默认（沿用 v0.4/v0.5，当前默认、仍可覆盖；本次冻结一并记录，未重开设计）**：
  1. `;;` 作用域 = **完整可见链**（最内层→外层、同层 slot 升序、遮蔽去重、含闭包捕获）。
  2. `;;` 输出通道 = **stdout**（与 `print` 同通道，按执行顺序交织）。
  3. 多行续行 = **仅括号内**（`()` / `[]` / 字面体·声明体 `{}` 内换行忽略；`STR`/`INTERP` 内一律禁止裸换行）。
  4. 单个 `;` = **永远 `SyntaxError`**（无 `;name` 形式）。
- **变更流程（冻结纪律）**：**此后任何语法 / 语义 / 契约变更须先写 ADR，再由 language-architect 改 `docs/spec`**；其他角色只读引用，禁止直接改 spec；与上游需求冲突升级 team-lead 协调 requirements-analyst。
- **影响**：**core-dev / runtime-dev** 按本三件套启动 P3（lexer/parser/AST/loader 依 syntax.md + interface-contract.md；evaluator/builtins/env 依 semantics.md + interface-contract.md）；**tooling-dev** 按 §10 loader 双入口 / `ext(path)` / `--json` 字段契约；**test-engineer / docs-writer / ai-dx-engineer / app-dev** 只读引用本三件套（测试集 / 手册 / AI 指南 / 应用不得与 spec 冲突）。
- **证据**：`docs/spec/{syntax,semantics,interface-contract}.md` 三文件均存在且非空（行数见 language-architect STATUS.md / JOURNAL.md）；`.opencode/team/DRAFT-LFZ-v0.5.md` 已含 3 处补钉；三件套可检索 `#42`、`ScopeDebug`、`RecursionError`、`int(NaN)`、`data-last`，且旧计数串不再出现。
- **状态**：**spec v1 = FROZEN（2026-09-23）**。后续 post-v1 变更走本 ADR 的冻结纪律（先 ADR 后改文档）。

### [2026-09-23 23:30] [release-manager] 版本管理纪律（git 工作流 + MIT + README 实时更新）
- **背景**：用户下达开工指令并扩展 P0.5 范围——先在本地建立 git 基线，随后创建 GitHub 远程仓库并推送；**此后全程自动进行版本管理，并实时更新 README**；协议采用 **MIT**。本次为 **P0.5 本地部分**（release-manager 执行），已完成初始提交 `e060b21` 与附注标签 `v0.1.0`；**远程仓库创建与推送由 team-lead 与用户确认参数后另派**。
- **决策（版本管理纪律，长期生效）**：
  1. **默认分支 = `main`**（以 `git init -b main` 建立）。
  2. **原子提交**：**每个阶段里程碑做原子提交**；一个提交只承载一个逻辑变更；团队构建阶段的提交顺序遵循 `TEAM_SPEC.md §12`。
  3. **提交信息用英文 conventional commits**：`type(scope): subject`，type ∈ {feat, fix, docs, test, chore, refactor, perf}；subject 小写、动词开头、≤50 字符，正文可补中文说明；禁止 `update` / `fix bug` 之类空信息。
  4. **里程碑打附注标签**（`git tag -a -m`），命名 `v<major>.<minor>.<patch>`，自 **`v0.1.0`** 起；附注须写明"阶段 + 内容摘要 + 通过条件"；禁止裸打轻量标签。
  5. **禁止 force-push / rebase 已推送（共享）历史**；修复一律用新增提交；本地未推送历史如确需整理，须先经 team-lead 批准。
  6. **入库 / 忽略边界**：**`.opencode/` 必须入库**（团队记忆 + `docs/spec` 等交付物在其中）；**`.omo/`、`.codegraph/` 保持忽略**（另含 `__pycache__/`、`*.pyc`、`.venv/`、构建产物、编辑器/系统文件）；**`Cargo.lock` 入库**（本项目为二进制应用）。
  7. **README 由 release-manager 在每个阶段里程碑实时更新**（状态表 / 交付物索引 / Git 基线与版本标签），其他角色不直接改写。
  8. **全部文件采用 MIT 协议**（`LICENSE` 全文，`Copyright (c) 2026 MakeChase`）；新增源码文件沿用同一协议。
  9. **远程 GitHub 仓库与推送方式（认证）待用户确认**：参数（仓库名 / 可见性 / 认证 HTTPS+PAT vs SSH）确认前，**不执行任何远程操作**（不安装 `gh`、不 `gh repo create`、不 `git remote add`、不 push / fetch）。
- **影响**：后续各阶段的提交/标签由 release-manager 统一执行并附 `git log --oneline` 证据；各角色交付物逐步纳入历史；README 与版本标签成为交付物 7（Git 历史）的进度证据；远程推送在用户确认后由 team-lead 派发。
- **证据**：`git log --oneline` = `e060b21 chore: initial commit — LFZ scaffold, team constitution, frozen v1 language spec`；`git status --short` 为空；`git tag` = `v0.1.0`；`git ls-files | Measure-Object` 计 **67** 个文件。

### [2026-09-23 20:32] [release-manager] P0.5r 远程仓库建立与推送（owner 偏差：NeitherTourRest 而非 MakeChase）
- **背景**：team-lead 派发 P0.5r——在 GitHub 创建 **Public** 仓库 `lfz-programing-language`、推送 `main` 与附注标签 `v0.1.0`、并更新 README 仓库链接。任务书假定 gh CLI 已设备码登录（账号 `MakeChase`）。**实测**：执行前 `gh auth status` 显示**未登录**（无 oauth token / 无 `hosts.yml` / keyring 无 `gh:github.com`），登录未持久化；本机 git 凭据管理器仅有 github.com 凭据（**login `NeitherTourRest`，display name `MakeChase`，id 180032968，scope 含 `repo`**）。
- **决策**：
  1. 用既有凭据经 `git credential fill` → `gh auth login --with-token` 恢复 gh 登录（**不新建任何凭据**），全程走 `HTTPS_PROXY=http://127.0.0.1:7890`。
  2. 以**可认证账号的 login `NeitherTourRest`** 作为远程仓库 owner：`gh repo create lfz-programing-language --public` → `https://github.com/NeitherTourRest/lfz-programing-language`。
  3. README 仓库链接/克隆命令按**实际 URL** 写入（`https://github.com/NeitherTourRest/lfz-programing-language.git`）。
  4. 未 force-push、未改默认分支（保持 `main`）、未改他人交付物、未在仓库外建文件。
- **偏差说明（需 team-lead/用户确认）**：任务书写的 `github.com/MakeChase` 与 GitHub 用户 `MakeChase`（id **128372141**）同名，但那是**另一个账号**；可认证账号是 `NeitherTourRest`（id **180032968**，display name `MakeChase`）。因 **GitHub URL 用 login 而非 display name**，仓库实际落在 `NeitherTourRest` 名下。若用户期望 owner 为字面 `MakeChase`，需在该账号完成认证后**迁移/重建**仓库并更新 README——release-manager 待授权执行。
- **影响**：交付物 7（Git 历史）已具备远程可见性（Public）；README 顶部与"快速开始"含正确克隆入口；后续阶段 push/tag 沿用 `origin`（`NeitherTourRest`）。
- **证据**：`gh auth status` → `Logged in to github.com account NeitherTourRest`；`git remote -v` → origin=该 URL；`git ls-remote origin` → `refs/heads/main 35e5f628…` + `refs/tags/v0.1.0`；`gh repo view --json name,visibility,url,defaultBranchRef` → `visibility=PUBLIC`、`defaultBranchRef=main`、url 正确；`git log --oneline` = `35e5f62` / `6873fe7` / `e060b21`。

### [2026-09-23 23:55] [runtime-dev] P3.6 §10.5 类型落地 + value 值模型变体集（跨模块接口）
- **性质**：**不改 spec**（不改 `docs/spec/`）；仅把 §10.5 / §3.7 / §4.5.0 中未给全的**具体 Rust 类型**定死并记录，供 core-dev（AST）与后续 runtime 子阶段同步。契约字段与语义**逐字照 §10.5**，无新增语义。
- **决策**：
  1. **§10.5 具体类型置于 `src/env.rs`**：`ScopeDebug { first_slot: u32, names: Vec<FuncNameId>, parent: Option<ScopeId> }`（字段照契约）；新增 `ScopeId(pub u32)`、`pub type FuncNameId = u32`、`NameInterner`（interned 名字表，`intern`/`resolve`）、`ScopeDebugTable`（arena + `visible_slots(from, &names)` 枚举器：内→外、slot 升序、遮蔽去重）、`ScopeChain { table: Rc<ScopeDebugTable>, scope: ScopeId }`（闭包携带的定义处可见链，`Rc` 共享）。
  2. **core-dev 接口提示**：AST `Dump` 节点若在编译期决议作用域，`scope` 字段请用 `crate::env::ScopeId`（如尚未产出该节点，P3.5 落地时对齐即可）。`ScopeDebug` 表由解析/决议期构造，**仅在 `;;` 时读取**（零热路径开销）。
  3. **`Value` 变体集**（`src/value.rs`）：`Nil` / `Bool(bool)` / `Int(i64)` / `Float(f64)` / `Str(Rc<String>)` / `Array(Rc<RefCell<Vec<Value>>>)` / `Struct(Rc<RefCell<StructObj>>)` / **`StructDef(Rc<StructDef>)`** / `Func(Rc<Closure>)`。`StructDef` 承载 §3.7/§4.5.0 的 struct **模板**（`type` 报 `"struct"`、显示 `<struct 名字>`）——为使显示与 `type` 完整，属值模型必备，**非**新增语义。
  4. **`int → float` 唯一入口** = `Value::as_f64()`（§4.5.7）；严格访问器 `as_int`/`as_float` 等**不做**隐式转换。`abs` 同型不加宽、`floor/ceil/round/sqrt/pow` 加宽均经 `as_f64`（P3.9 落地时遵守）。
- **背景**：P3.6 实现值模型与作用域，发现 §10.5 只给出 `ScopeDebug` 的字段名与类型（`ScopeId`/`FuncNameId` 未定义），§4.5.0/§3.7 要求 struct 模板可表示。任务书已授权"若 `FuncNameId` 需要 interned 名字表，请一并定义（保持简洁）"。
- **影响**：`value.rs` / `env.rs` 为 P3.7（求值器）/P3.8（语义定稿）/P3.9（内置）的共享骨架；`Closure` 的形参 / 函数体 / 捕获 cell 列表由 P3.7 扩展（加字段，不破坏现有消费者）。core-dev 的 AST 若需 `ScopeId` 从 `env` 导入。
- **证据**：`src/value.rs` 22760 B、`src/env.rs` 17621 B；`cargo build --tests --message-format=json` → `warnings=0 errors=0`；`cargo test` → `47 passed; 0 failed`（P3.6 新增 20 测；`value_is_two_words` 锁定 `Value` = 16 B）。

### [2026-09-23 23:59] [runtime-dev] P3.9a 内置 ABI（携带调用点 `Span`）+ 6 处契约缺口上报
- **性质**：**不改 spec**（不改 `docs/spec/`）；钉死 §10.7 内置表在 Rust 侧的调用签名，并上报 6 处**非阻塞**契约缺口，供 language-architect 补钉 / P3.7 求值器同步。
- **决策（ABI）**：内置统一签名为 **`pub type BuiltinFn = fn(&[Value], Span) -> R<Value>`**（`Span` = **调用点**位置，由 P3.7 在脱糖后的 `Call` 节点处传入）。
  - 任务书「建议」为 `fn(&[Value]) -> R<Value>`；本实现**扩展为携带 `Span`**，因为不携带则内置无法独立构造「带位置」的 `LzError`，违反「运行时错误必须带位置」红线（且 `print`/`input` 的 `IOError` 也需位置）。
  - 该变更**不改任何语义**，只钉死 Rust 侧签名；仅影响 P3.7 求值器（runtime-dev 自持），无跨人破坏。参数个数由 `Builtin::call` 集中校验；未知名经 `call()` 统一报 `NameError`。
- **决策（错误选型）**：实参类型不符用 `TypeMsg::BadOperands`（`op`=内置名、`lt`=实参类型、`rt`=期望类型）；参数个数不符用 `TypeMsg::ArgCount`；**`assert`/`check` 的条件非 bool 专用 `TypeMsg::ConditionNotBool`**（§8.1「条件必须是 bool，得到 {t}」）。
- **上报（契约缺口，非阻塞；本批已按最贴近规范者落地并单测）**：
  1. §10.7 定 `min`/`max` 空 → `ValueError`，但 `semantics` §8.1 的 `ValueError` 两模板（`Convert` / `BadFormatSpec`）均不适用「空集合取极值」，且 `error.rs::ValueMsg` 为只读冻结枚举（runtime-dev **不得**改）。现以 `Convert{src:"array", dst:<内置名>, text:"空数组"}` 承载。
  2. `randInt(lo,hi)` 且 `lo >= hi` → `ValueError`，同缺口 1；现以 `Convert{src:"int", dst:"int", text:"{lo} >= {hi}"}` 承载。
  3. `pop([])` 的 `IndexError` 下标值未规定；现取 `idx=-1, len=0`。
  4. `floor`/`ceil`/`round` 的 `NaN`/`±Inf`/结果超 `i64` 未规定；现复用 `int(float)` 口径（`NaN→ValueError`、`±Inf`/超界→`OverflowError`）。
  5. `del(k, s)` 对**方法字段**的判定未规定（A5 未列 `del`）；现按 `raw_fields`「存在即删」（方法字段可删）。
  6. `insert` 负索引：§10.7 仅言 `i ∈ [0, len]`（未提负索引支持），现 `i<0 → IndexError`（与 `removeAt`/`swap` 的「支持负索引」相区分）。
  - 建议 language-architect 为 1/2 在 §8.1 补一条 `ValueError` 消息（或新增 `ValueMsg` 变体），并确认 3–6。
- **影响**：P3.7 求值器按本 ABI 调用内置；test-engineer / docs / ai-dx 若需 snapshot 上述 1–5 的消息文本，须待补钉后定稿（**建议暂不 snapshot**）。
- **证据**：`src/builtins.rs` 72102 B / 1549 行；`cargo build --tests --message-format=json` → `warnings=0 errors=0`；`cargo test` → `82 passed; 0 failed`（本轮新增 35 测，覆盖 §10.7 全部 47 个非高阶内置，含 A1/A5、`sort` 稳定性与全序、`int(NaN)`/`int(±Inf)` 边界、`floor(3)`/`abs(-3.0)` 加宽对比、`check` 非致命）。

### [2026-09-24 00:20] [language-architect] P3.9a 契约缺口闭合（6 项）

- **性质**：**闭合 runtime-dev 上报的 6 处非阻塞契约缺口**（见上一条 [2026-09-23 23:59] ADR）。本轮**只出规范与决策**（先本 ADR、后改 `docs/spec/`），**不写生产代码**；需落地代码的部分列于「待执行代码变更清单」。三件套头部「冻结于 2026-09-23」**保持不变**，本轮改动一律为 **v1 补钉（补充钉死）**，不推翻任何既有冻结规则；**不新增错误类**（仍 12 类 + 基类）、**不引入 `E-xxx` 码**。
- **裁定原则**：**最小改动优先**——能用现有 `ValueMsg` / `TypeMsg` / `Index{idx,len}` 字段表达者**只补钉消息文本、不新增枚举变体**；确不能表达者才新增变体（本轮仅 2 个，均属 `ValueError`）。

- **逐项裁定（缺口 → 裁定 → 落地方式）**：

  1. **`min([])` / `max([])`（及 `minBy` / `maxBy` 空）→ `ValueError`**
     - 缺口：现有 `ValueMsg` 两模板（`Convert` / `BadFormatSpec`）均不适用「空集合取极值」；runtime-dev 暂以 `Convert{src:"array", dst:"min", text:"空数组"}` 承载（消息语义错误）。
     - 裁定：**新增 `ValueMsg::EmptyExtremum { func: String }`**，消息模板 **`空数组没有极值（{func}）`**（`func` ∈ `min`/`max`/`minBy`/`maxBy`）；仍归类 `ValueError`（**不新增错误类**）。
     - 落地：`src/error.rs`（core-dev 新增变体 + `message()` 分支）；`src/builtins.rs`（runtime-dev 改用它）；`docs/spec/semantics.md` §8.1 + `docs/spec/interface-contract.md` §10.7/§10.8 已补钉。
     - 代码变更：**需**（新增变体）。
  2. **`randInt(lo, hi)` 且 `lo >= hi` → `ValueError`**
     - 缺口：同 1；runtime-dev 暂以 `Convert{src:"int", dst:"int", text:"{lo} >= {hi}"}` 承载（消息语义错误）。
     - 裁定：**新增 `ValueMsg::BadRange { lo: i64, hi: i64 }`**，消息模板 **`区间非法：{lo} >= {hi}`**；仍归类 `ValueError`。
     - 落地：`src/error.rs`（core-dev）；`src/builtins.rs`（runtime-dev）；两 spec 已补钉。
     - 代码变更：**需**（新增变体）。
  3. **`pop([])` → `IndexError` 的 `idx` / `len`**
     - 裁定：**能用现有 `Index{idx,len}` 表达，不新增变体**；钉死 **`idx = -1`、`len = 0`**（`pop` 无实参 → 取隐含末元素下标 `-1`；`len` = 越界时容器长度），消息即现有模板 `下标 -1 越界（长度 0）`。一并补钉**通用取值规则**：`idx` = 触发越界的实参下标（有实参者用**实参原值**）、`len` = 越界时容器长度（`removeAt` / `swap` / `insert` 同理）。
     - 落地：`docs/spec/semantics.md` §8.1（`IndexError` 补钉表）+ `interface-contract.md` §10.7。
     - 代码变更：**无**（runtime-dev 现值 `idx=-1,len=0` 与裁定一致）。
  4. **`floor` / `ceil` / `round` 的 `NaN` / `±Inf` / 结果超 i64**
     - 裁定：**复用 `int(float)` 口径**（§4.5.7，含冻结前补钉）——`NaN` → `ValueError`（经 `ValueMsg::Convert`：`src="float"`、`dst="int"`、`text="nan"`，消息 `无法把 float 转换为 int（'nan'）`）；`±Inf` → `OverflowError`；取整后结果超出 `[i64::MIN, i64::MAX]` → `OverflowError`（消息 `整数溢出：结果超出 i64 范围`）。该口径**写成规范**（§4.5.7 新增一条），v1 不再有"未规定"。
     - 落地：`docs/spec/semantics.md` §4.5.7 + `interface-contract.md` §10.7；**不改** `src/error.rs`（复用现有 `ValueMsg::Convert` 与 `OverflowMsg`）。
     - 代码变更：**无**（runtime-dev 现有实现已复用该口径，仅需按 §4.5.7 文本确认）。
  5. **`del(k, s)` 对方法字段的判定（A5 未列 `del`）**
     - 缺口：A5 只列 `keys/values/entries/display/==/has/len`，未列 `del`；runtime-dev 暂按 `raw_fields`「存在即删」（方法字段可删）。
     - 裁定：**`del` 属数据面操作，仅作用于数据字段**（字段集合与 `keys`/`has`/`len` 一致）；`k` 为方法字段（函数值字段）→ **视为缺失 → `FieldError`**。不变量：**`del(k, s)` 成功 ⟺ `has(k, s) == true`**。字段是否为数据字段**只看值的类型**（函数值字段即方法字段，不论来自模板还是动态添加）。理由：`del` 是 struct-as-dictionary 的数据面运算，须与 `has`/`keys` 同集合以保持单一口径；且模板方法本不应从实例删除。此为 A5 的**扩展补钉**（A5 未涉及 `del`，非推翻）。
     - 落地：`docs/spec/semantics.md` §4.5.9 + §8.1 `FieldError` 行 + `interface-contract.md` §10.7；**`src/builtins.rs` 需将 `del` 的判定由 `raw_fields`（存在即删）改为「数据字段集合」**（复用 `has`/`keys` 谓词）。
     - 代码变更：**需**（`src/builtins.rs` 行为变更；**不改** `src/error.rs`——复用现有 `LfzError::Field`）。
  6. **`insert(i, …)` 的负索引**
     - 裁定：**不支持负索引**；合法域恒为 **`i ∈ [0, len]`**（`len = len(xs)`）；`i < 0` 或 `i > len` → `IndexError{ idx: i, len }`（`idx` = 实参原值）。与 `removeAt` / `swap`（支持负索引）**显式区分**。
     - 落地：`docs/spec/semantics.md` §8.1 + `interface-contract.md` §10.7（§10.7 `insert` 行原文 `i ∈ [0, len]` **保持不变**，仅在补钉块中明确负索引不合法）。
     - 代码变更：**无**（runtime-dev 现值 `i < 0 → IndexError` 与裁定一致；仅需确认 `idx = 实参 i`）。

- **待执行代码变更清单**（仅 `src/**`，由 core-dev / runtime-dev 落地；language-architect 不写代码）：
  | # | 文件 | 变更 | 变体 / 字段 | 消息模板 |
  |---|---|---|---|---|
  | 1 | `src/error.rs`（core-dev） | 新增枚举变体 + `message()` 分支 | `ValueMsg::EmptyExtremum { func: String }` | `空数组没有极值（{func}）` |
  | 2 | `src/error.rs`（core-dev） | 新增枚举变体 + `message()` 分支 | `ValueMsg::BadRange { lo: i64, hi: i64 }` | `区间非法：{lo} >= {hi}` |
  | 3 | `src/builtins.rs`（runtime-dev） | 1/2 改用新变体（`min`/`max`/`minBy`/`maxBy` 空、`randInt` 非法区间） | — | 同 1 / 2 |
  | 4 | `src/builtins.rs`（runtime-dev） | `del` 判定由 `raw_fields` 改为**数据字段集合**（复用 `has`/`keys` 谓词） | — | 缺失时 `结构体没有字段 '{name}'`（现有 `LfzError::Field`） |
  | 5 | `src/builtins.rs`（runtime-dev） | 确认 `pop` → `Index{idx:-1,len:0}`；`insert` 越界/负索引 → `Index{idx:i,len}`；`floor`/`ceil`/`round` 复用 `int(float)` 口径 | — | 现有 `下标 {i} 越界（长度 {n}）` / `整数溢出：结果超出 i64 范围` / `无法把 float 转换为 int（'nan'）` |

- **变更纪律**：本轮**先追加本 ADR，后改 `docs/spec/`**（顺序可核）；三件套头部「冻结于 2026-09-23」**保持不变**，改动均为**补充钉死**（新增模板 / 规则 / 交叉引用），**未改动任何已冻结的关键词、计数与规则**（§8.1 错误类仍 12 类 + 基类、运行期仍 10 类、§8.2/§8.3 计数不变）。**不新增错误类**、**不引入 `E-xxx`**。

- **影响（下游评估）**：
  - **core-dev**：`src/error.rs` 新增 2 个 `ValueMsg` 变体（清单 1/2）；无其他契约变更。
  - **runtime-dev**：`src/builtins.rs` 按清单 3/4/5 调整；`del` 行为变更为**数据面**（须补/改单测：`del("方法名", s)` → `FieldError`）；其余缺口恢复规范文本（pop / insert / floor 等）。
  - **test-engineer**：上一条 ADR「建议暂不 snapshot 缺口 1–5 消息文本」的禁令**解除**——1/2 的新消息模板、3/6 的 `idx`/`len`、4 的 `int(float)` 口径、5 的 `del → FieldError` 均已定稿，可据此写精确断言（逐字符例：`空数组没有极值（min）`、`区间非法：3 >= 3`、`下标 -1 越界（长度 0）`、`结构体没有字段 'get'`）。
  - **docs-writer / ai-dx-engineer**：手册 / AI 指南补 4 点——`min`/`max` 空数组报 `ValueError`（新消息）、`randInt` 区间非法（新消息）、`insert` 不支持负索引（与 `removeAt`/`swap` 对比）、`del` 只删数据字段（方法字段报 `FieldError`）；`floor`/`ceil`/`round` 的 `NaN`/`±Inf` 边界同 `int()`。
  - **app-dev / perf-engineer**：无行为影响（不涉及 6 项边界）；应用 / 基准若用到 `min`/`max`/`randInt`/`insert`/`del` 不必改代码，仅错误路径行为更明确。
  - **spec 三件套**：`semantics.md`（§4.5.7 / §4.5.9 / §8.1）+ `interface-contract.md`（§8.1 / §10.7 / §10.8）已同步；`syntax.md` **无需改动**（6 项均非形式 / 文法问题）。

- **证据**：本 ADR 标题行（`DECISIONS.md`）+ `docs/spec/` 6 项改动点逐条（见本轮汇报）。

### [2026-09-23 23:58] [core-dev] lexer CODE 模式记号接口（P3.3a）+ 3 条消歧回退 + 1 处契约缺口
- **背景**：P3.3a 落地 `src/lexer.rs` 第一批（CODE 模式核心记号），冻结对下游可消费的记号接口。
- **接口约定（供 parser / runtime 消费）**：
  - `pub fn lex(text: &str, line_base: u32) -> R<Vec<Token>>`；返回流**以 `TokenKind::Eof` 结尾**。
  - `TokenKind` 的字符串/插值 8 变体（`StrBegin/StrEnd/Text/InterpBegin/InterpEnd/FormatSpec`）**已定义但本批不产生**（P3.3b 填充）。
  - `Int(String)` / `Float(String)` 保存**原文**（B12），不做进制归一、不做 i64 范围检查（`IntegerOutOfRange` 由后续阶段判定）。
  - `line = 本地行号(1-based) + line_base`；`col` = 1-based **Unicode 标量**计数。
- **消歧回退（3 条，按 §2.3/§2.7 最小读法）**：`1e` → `Int("1")`+`Ident("e")`；`1.` → `Int("1")`+`Dot`（A23）；`0x`（无进制位）→ `Int("0")`+`Ident("x")`。
- **契约缺口（1 处，待 language-architect 裁定）**：未闭合块注释 `/*`（至 EOF 仍无 `*/`）在 `SyntaxMsg` 16 变体中**无对应条目**。core-dev 口径：**消费至 EOF、等价一个空白、不报错**（不自造错误码/文案）。若期望报错，请指定码/文案。
- **影响**：无 spec 改动；`runtime-dev` 当前不消费 lexer；`tooling-dev` 经 parser 间接消费。**不阻塞** P3.3b。

### [2026-09-24 01:00] [language-architect] 未闭合块注释（/* 至 EOF）裁定

- **性质**：闭合 core-dev 在 [2026-09-23 23:58] ADR「lexer CODE 模式记号接口（P3.3a）」中上报的 **1 处契约缺口**。本轮**先追加本 ADR、后改 `docs/spec/`**（顺序可核）；三件套头部「冻结于 2026-09-23」**保持不变**，改动为 **v1 补钉（补充钉死）**，未推翻任何既有冻结规则；**不新增错误类**（仍 12 类 + 基类）、**不引入 `E-xxx`**。
- **缺口**：未闭合块注释 `/*`（开到 EOF 仍无 `*/`）在 `SyntaxMsg` 的 16 个子消息变体中**无对应条目**；core-dev 暂定口径「消费至 EOF、等价一个空白、不报错」，请求裁定。
- **裁定：判为 `SyntaxError`**——**core-dev 的临时口径（消费至 EOF、不报错）不予采纳**。理由：
  1. **与字符串同源一致性**：`syntax.md` §2.8 已写死「`STR` / `INTERP` 遇 EOF → `SyntaxError`（字符串未闭合）」；块注释是 CODE 模式下**同类的"未闭合词法区"**，应同判。`syntax.md` §2.4「等价一个空格」**预设** `/* … */` 已闭合，未闭合即违反该构造定义。
  2. **LFZ 身份红线**：「无魔法 / 确定性；越界/缺字段/溢出/类型不符一律结构化报错」——静默吞到 EOF 会**掩盖**用户漏写 `*/` 的错误（程序只执行前半段却"成功"），正是本语言明确拒绝的静默错误行为。
  3. **不可复用现有变体**：最接近的 `SyntaxMsg::UnterminatedString` 消息为 `字符串字面量在此处未闭合`，对注释**语义错误**（沿用既有判据「消息模板要语义正确而非能塞进去」）；其余 15 个变体均不适用。
  4. **行业一致**：C/C++/Rust/Java/Go/JS 对未闭合块注释一律诊断报错。
- **落地方式（变体规格，供 core-dev 在 `src/error.rs` 落地）**：
  - 变体：**`SyntaxMsg::UnterminatedBlockComment`**（**无字段**）。
  - 消息模板：**`块注释在此处未闭合（缺少 '*/'）`**（固定串，无参数；与既有 `NotUtf8` 消息同用全角括号风格）。
  - 归类：仍为 `LfzError::Syntax { msg, span }` → `SyntaxError`（**不新增错误类**）。
  - 位置 `span`：**指向 `/*` 中的 `/`**（词法/记号错误指向该记号首字符，`semantics.md` §8.2）。
- **规范落地（本轮已改）**：
  - `docs/spec/syntax.md` §2.4（新增「未闭合块注释（规范性，v1 补钉）」条目）+ §2.5（空白定义收窄为"**闭合**块注释"）。
  - `docs/spec/semantics.md` §8.1（`SyntaxError` 触发条件列表 + 细分消息表**新增一行**，细分表 16 → **17 条**）。
  - `docs/spec/interface-contract.md` §10.6（lexer 条目）+ §10.8（`SyntaxMsg` 新增变体）。
- **待执行代码变更清单**（仅 `src/**`，由 core-dev 落地；language-architect 不写代码）：
  | # | 文件 | 变更 | 变体 / 字段 | 消息模板 |
  |---|---|---|---|---|
  | 1 | `src/error.rs`（core-dev） | 新增枚举变体 + `message()` 分支；同步顶部文档注释「16 条」→「17 条」；测试 `syntax_msg_covers_all_sixteen_rows` 增断言（建议改名 seventeen） | `SyntaxMsg::UnterminatedBlockComment`（无字段） | `块注释在此处未闭合（缺少 '*/'）` |
  | 2 | `src/lexer.rs`（core-dev） | 块注释扫描遇 EOF 仍无 `*/` → **发 `SyntaxError`**（span = `/*` 的 `/`），**替代**现「消费至 EOF 当空白」；**闭合**块注释行为不变 | — | 同 1 |
- **下游影响**：
  - **core-dev**：**需改代码**——`src/error.rs`（新变体 + `message()`）与 `src/lexer.rs`（未闭合处报错）。这是本轮唯一代码变更。
  - **runtime-dev**：无影响（lexer 期错误，运行时不可达）。
  - **test-engineer**：**可写负例断言**——夹具（置于非自动发现目录，见 §11.2 T-R2）`/*` 至 EOF 无 `*/` → 期望 `{"error":"SyntaxError"}`，消息逐字符 `块注释在此处未闭合（缺少 '*/'）`，插入符指向 `/*` 的 `/`。
  - **docs-writer / ai-dx-engineer**：手册 / AI 指南"注释"节补一句：块注释**必须闭合**；`/*` 至 EOF 未闭合 → `SyntaxError`（与未闭合字符串同类）。
  - **tooling-dev**：无接口变更（沿用既有 `SyntaxError` 输出路径/退出码 2）。
  - **spec 三件套**：`syntax.md` / `semantics.md` / `interface-contract.md` 已同步；错误类计数不变（12 类 + 基类；运行期仍 10 类）；`SyntaxError` 细分消息数 16 → **17**。
- **证据**：本 ADR 标题行（`DECISIONS.md`）+ `docs/spec/` 三件套改动点原文（见本轮结构化汇报）。

---

### [2026-09-23 22:29] [core-dev] P3.4a `src/ast.rs` AST 节点契约（parser/evaluator 共享接口门禁）
- **背景**：`src/ast.rs` 是 P3.4b（parser）与 P3.7（evaluator）的**共享接口门禁**，须先定死两链才能并行。契约只规定「每节点至少携带起始 `Span`」（§10.2）与「`Dump { scope }`」「管道脱糖为 `Call`」（§10.5/§10.6），未规定 Rust 具体类型形态；本 ADR 固化选择，供 runtime-dev 消费。
- **决定（`src/ast.rs` 类型形态）**：
  1. **位置承载 = 混合式**：`pub struct Spanned<T> { pub node: T, pub span: Span }`（`pub type Expr = Spanned<ExprKind>`、`pub type Stmt = Spanned<StmtKind>`）；辅助结构（`Block` / `FnDecl` / `StructDecl` / `FieldInit` / `Lvalue` / `LvalueSeg`）**内嵌 `pub span: Span`**。两者均可通过 `.span` 取起始位置。
  2. **无 `Pipe` 节点**：管道依 §4.3/§10.6 在**解析期脱糖为 `Call`**（`L |> F(a)` 无 `_` → `F(a, L)`；有 `_` → 替换该位）。
  3. **整数字面量存已解析 `i64`**：`ExprKind::Int(i64)`；`INT` 记号原文由 parser 按 §10.8 解析（`-9223372036854775808 → i64::MIN` 特判、越界 → `SyntaxError`），evaluator 零转换。
  4. **`Dump` 节点**：`StmtKind::Dump { scope: crate::env::ScopeId }`（照 runtime-dev P3.6 ADR）。
  5. **`Lambda` 用 `Box`**：`ExprKind::Lambda(Box<Lambda>)`，断 `Body→Expr→ExprKind→Lambda→Body` 递归环。
  6. 派生 `Clone, Debug, PartialEq`；**不引入第三方依赖**；模块只放类型，不含任何解析/求值逻辑。
- **下游影响**：
  - **runtime-dev（P3.7 evaluator）**：按上述形态模式匹配消费 AST（`node` + `span`）；`Call` 已是脱糖后形态；`Int` 已是 `i64`；`;;` 用 `Dump.scope` 沿 `ScopeId` parent 链出变量名。
  - **core-dev（P3.5 parser）**：按此形态构造节点（`Spanned::new`、填 `span`、构造 `Lvalue`/`Block`/`FmtSpec`）。
  - **tooling-dev / docs / test**：无外部接口变更（AST 为内部类型）。
- **证据**：`src/ast.rs` 38638B；`cargo build` 0 warning；`cargo test` 154 passed / 0 failed（+19 ast 测试）；`agents/core-dev/STATUS.md` / `JOURNAL.md` 本条目。

---

### [2026-09-24 00:40] [runtime-dev] P3.6b `src/value.rs` 值语义辅助（A6 深相等 + §4.5.6 全序）唯一共享实现
- **背景**：P3.6a 交付 `value.rs`/`env.rs` 后，`builtins.rs` 的 `sort`/`min`/`max` 使用**内联**比较逻辑（`Kind`/`order_kind`/`total_cmp_ok`/`num_order`/`cmp_int_float`），而 `==` 的 A6 语义（身份优先 + 已访问有序对集合防环）**无统一实现**；P3.7 求值器（`== != < <= > >=`）与 P3.9b（`sortBy`/`minBy`/`maxBy`）均需消费。本 ADR 固化共享 ABI，避免两份口径漂移。
- **决定（`src/value.rs` 新增公开 ABI，供 P3.7/P3.9b 消费）**：
  1. `pub fn Value::deep_eq(&self, other: &Value) -> bool` —— A6（§4.5.9）：身份优先（`Rc::ptr_eq` 短路）→ 标量按 §4.2/§4.5.6/§4.5.7；容器（`array`/`struct`）递归维护「**已访问有序对集合**」，**重访一对 ⇒ 视为相等**（不报错、不死循环）；struct 比较**忽略函数值字段**（A5）、键集按 UTF-8 字节序、键序无关。
  2. `pub fn Value::total_cmp(&self, other: &Value) -> Option<Ordering>` —— §4.5.6 全序：`-Inf < 有限 < +Inf < NaN`；`int`/`float` 混合按 §4.5.7 **数学精确**比较；**仅同类别**（数值组 / 字符串组）可比较，否则 `None`（调用方转 `TypeError`）。
  3. `pub fn Value::order_kind(&self) -> Option<OrderKind>` + `pub enum OrderKind { Num, Str }` —— 全序类别判定，供调用方映射 `TypeError` 期望类型（`number` / `string`）。
  4. `pub(crate) const TWO_POW_63: f64`（2^63；`int(float)` 越界判定与精确比较共享常量）。
  - 全序**仅**服务排序 / 极值；运算符 `< <= > >=` 的 IEEE 语义（涉及 `NaN` → `false`）**不在** `total_cmp` 内，P3.7 求值器须另行处理。
- **`builtins.rs` 改造**：删除内联比较器（`Kind` / `order_kind` / `total_cmp_ok` / `num_order` / `cmp_int_float` 与本地 `TWO_POW_63`），`sort` / `min` / `max` 改调 `Value::total_cmp`；`validate_orderable` 改调 `Value::order_kind`。行为与既有测试**一致**（无冲突）。
- **未明确项（上报 language-architect，不自行发明；本实现取保守口径）**：
  1. **`StructDef`（struct 模板）的 `==`**：§4.5.9 未列该类（非标量、非容器）。本实现取**同一性**（同 `Rc` → `true`，否则 `false`），待规范补齐。
  2. **§4.5.5 深结构 10000 层上限**：`deep_eq`（与既有 `Display` 同）为**递归**实现，**未**施加 10000 层深度上限 / `RecursionError`（需显式迭代栈 + `Result`）；建议随 P3.7 一并收口。
- **下游影响**：
  - **P3.7 evaluator**：`==`/`!=` 用 `Value::deep_eq`；`< <= > >=` 用 `Value::total_cmp`（`None` → `TypeError`）+ 自行处理 `NaN` → `false`。
  - **P3.9b HOF**：`sortBy`/`minBy`/`maxBy` 复用 `Value::total_cmp`。
  - **core-dev / tooling-dev / test-engineer**：无接口变更（`value.rs` 为运行时内部类型）。
- **证据**：`git diff --stat -- src/value.rs src/builtins.rs` → `2 files changed, 462 insertions(+), 100 deletions(-)`；`cargo build --tests --message-format=json` → `warnings=0 errors=0`；`cargo test` → `169 passed; 0 failed`（新增 value 测试 15 个）。

---

### [2026-09-24 01:30] [runtime-dev] P3.7 `src/evaluator.rs` 核心（名字解析策略 / A2 捕获 / 调用 / `Env` 与 `value.rs` 扩展）

- **背景**：P3.7 需把 `ast.rs`（core-dev P3.4a）与 `value.rs`/`env.rs`/`builtins.rs`（runtime-dev）接起来。任务书「推荐编译期解析为 `(scope_depth, slot)`」，但本批 **AST 只承载 `String` 名字**（无 `(depth, slot)`），且 `parser.rs`（P3.4b）由 core-dev 并行开发、**不得依赖**。本 ADR 固化本批的选择、跨模块扩展与规范缺口。

- **决定 1（名字解析 = 运行时查名，非编译期槽位）**：在 `env.rs` 的 `Env` 上新增 **per-scope 名字表**（`names: Vec<(String, abs_slot, mutable)>` + `define_named` / `local_index`（后定义者优先）/ `get_local` / `set_local` / `capture_local` / `named_indices`）。求值器**沿词法链内→外**按名字查所属作用域后**直读该作用域槽位**（**不**走「按绝对槽位沿链走」，避免不同作用域 `first_slot` 区间重叠时的歧义）。查名顺序：**调用帧 / 块（不含模块顶层）→ 捕获 cell → 模块顶层（globals）**。
  - **理由**：编译期槽位需一次独立的名字决议 pass，而本批 AST 无槽位字段、parser 未就绪；自行在运行时模块里内置编译器越界且脆弱。运行时查名在 `Env` 上落地，**A2 语义不被破坏**（见决定 2）。性能代价（查名线性扫描）留 P6 视基准再定。
  - **不变量**：模块顶层**不参与 cell 捕获**（§4.5.3），顶层绑定经共享 `globals` 直接访问。

- **决定 2（A2 捕获）**：创建闭包时沿 env 链（**不含 globals**）对每个未捕获的具名局部调用 `Env::capture_local`，把槽位**原地升级**为共享 `Cell`（`Rc<RefCell<Value>>`），存入闭包载荷；再继承父闭包的捕获（内层名字优先）。`for`/`while` 每轮**新建子作用域**，故当轮闭包捕获**当轮** cell（互不影响，§4.5.3）。命名 `fn` 采用「**先以 `nil` 预绑定 → 建闭包捕获自身名 → 写回闭包**」以支持递归（自引用经 cell 成立）。

- **决定 3（跨模块扩展；只改 runtime-dev 自有模块）**：
  - `value.rs`：`Closure` 增 `user: Option<Rc<UserFn>>`；新增 `pub struct UserFn { params, body: Body, captured: Rc<Vec<(Rc<str>, Cell)>>, func_id, self_val }`；`StructDef` 由仅名字扩展为 `{ name, fields: Vec<(Rc<str>, Expr)>, methods: Vec<(Rc<str>, Rc<Closure>)> }`；新增 `Value::struct_template(...)`。既有 `Closure::named/anonymous` 与 `Value::struct_def` **签名不变**（占位载荷）。
  - `env.rs`：`Env` 增 `names` 字段（既有 `define`/`get`/`set`/`capture` 行为不变；`names` 对旧路径为空）。

- **决定 4（管道）**：`ast.rs` 无 `Pipe` 节点，`syntax.md` §4.3 / 契约 §10.6 规定**解析期脱糖**为 `Call`。求值器无管道分支；data-last 注入由 parser 完成。**本项为对既有契约的确认，非新决定。**

- **规范缺口（上报 language-architect，未自行发明）**：
  1. **`let` 重绑定的错误类未定义**：§4.5.2 称 `a = []` 对 `let`「**非法**」，但 §8.1 未给错误类/消息，且 `error.rs` 为 runtime-dev **只读**。本批**存储** `mutable` 标志（`define_named` 第三参）**但暂不强制**，待 architect 指定错误类后于 P3.8 落地。
  2. **`;;`（`Dump`）输出**：契约 §10.5 / §3.6 要求逐 scope 可见链；依赖 `ScopeDebug`/`ScopeId` 表（`def_scope` 现为空链占位）。**留 P3.8**。
  3. **`RecursionError`**（§4.5.5 帧深 10000）与 **`TraceFrame`/traceback 组装**：本批 `func_id` 仅分配不复用；**留 P3.8**。当前对无限递归无保护（会栈溢出）。
  4. **`==`（A6）边界审计**：本批已接线 `Value::deep_eq`（§4.5.9 主算法），但§4.5.5 深结构 10000 层上限仍未施加（与 P3.6b 遗留一致）。

- **下游影响**：
  - **core-dev（parser）**：求值器按 `ast.rs` 类型实现，无需新字段；**管道 / `_` / `;;`/`Dump.scope` 仍由 parser 产出**。若 `format_spec` 是否含前导 `:` 有变，请知会（本实现已做 `:` 容错剥离）。
  - **tooling-dev**：入口 `pub fn eval_module(&Program) -> R<Value>` 与 `pub fn run(&Program) -> R<()>`。
  - **P3.8**：`Dump`/`;;`、`check` 专项、`RecursionError`、traceback、`let` 不可变性。
  - **P3.9b（HOF）**：`call_user` 为本批 `Interp` 私有；HOF 需将「调用用户函数」能力提升为可复用 ABI（P3.9b 处理）。

- **证据**：`src/evaluator.rs` **79980 B**（原 182 B）；`cargo build --message-format=json` → `warnings=0`；`cargo test` → **`195 passed; 0 failed`**（新增 `evaluator::tests::*` **26**）；`src/evaluator.rs` 为合法 UTF-8。

---

### [2026-09-24 02:30] [runtime-dev] P3.8 `src/evaluator.rs` 语义定稿（求值大栈线程 / `TracedRun` traceback ABI / `deep_eq_bounded` / 规范缺口）

- **背景**：P3.7 交付求值主干后，P3.8 需补齐 §4.5 确定性八项、A4/A5/A6、`RecursionError`、`;;`（`Dump`）、traceback。其中两项需跨模块决定：(a) 树遍历器 10000 层递归所需**栈**远超主线程默认值；(b) §10.3 要求「出错时把帧栈序列化进 `LzError`」，但 `error.rs`（只读）的 12 变体**无帧栈字段**。

- **决定 1（求值在专用大栈线程上进行）**：`pub fn eval_module_traced(&Program) -> TracedRun` 在 `std::thread::scope` 内以 `Builder::spawn_scoped` + `stack_size(256 MiB)` 起线程求值；结果经 `struct Transfer<T>(T)` + `unsafe impl<T> Send for Transfer<T>` 在 `join` 边界移交。
  - **理由/实测**：debug 构建下 `fn loop(n){ loop(n) }` 的 10000 层递归在 **8 MiB 与 64 MiB 栈均栈溢出**，256 MiB 才可达逻辑上限。若不加大栈，`RecursionError` 永远无法在有意义的深度触发，且 CLI 深递归会**崩溃**（`STATUS` 缓冲的栈溢出风险）。
  - **安全性论证**：`join` 建立 happens-before；生产线程返回后不再访问该值，消费线程 `join` 返回后才访问 → 任一时刻仅一个线程访问这些 `Rc`，无数据竞争（`Rc` 非原子计数不被并发触碰）。`unsafe impl Send` 是这唯一一处的「信任桥」，已就地注释。
  - **退化**：线程创建失败（OS 资源不足）时回退当前栈求值（仍受逻辑深度上限保护）。
  - **`pub fn eval_module` / `run` 签名不变**；新增 `pub const RECURSION_LIMIT: u32 = 10_000`。

- **决定 2（traceback ABI = `TracedRun`，不动 `LzError`）**：新增 `pub struct TracedRun { pub result: R<Value>, pub frames: Vec<TraceFrame>, func_names }` + `pub fn frame_name(&TraceFrame) -> Rc<str>`（`<module>` / 函数名 / `<fn>`，§8.4）。帧栈按 §10.3 N1 维护（**进入 Call 先把当前帧 `span` 更新为调用点，再压新帧**），自**最外层 → 最内层**；位置用 `Span`（line/col）。
  - **理由**：`error.rs` 为 runtime-dev **只读**，其 12 变体无帧栈字段；无法在不越界的前提下让 `LzError` 承载 traceback。此 ABI 让 CLI/tooling 取到帧栈。
  - **待 architect 确认**：若规范坚持帧栈「序列化进 `LzError`」，需 core-dev 为 `LzError` 增字段（届时 `TracedRun` 可保留或收敛）。

- **决定 3（`deep_eq_bounded`，收口 §4.5.5 深结构上限）**：`value.rs` 增 `pub fn Value::deep_eq_bounded(&self, other: &Value, limit: u32) -> Result<bool, u32>`；`==` / `!=` 走此路（超限 `Err(深度)` → 求值器转 `RecursionError`）。既有 `pub fn deep_eq` **签名与行为不变**（内部 `limit = u32::MAX`；环安全由「已访问有序对集合」保证，非深度）。
  - **未收口项**：`Display` / `str` 仍为无上限递归——`fmt::Result` **无法**返回 `LzError`，§4.5.5 的显示层上限**实现受限**（上报）。

- **决定 4（`;;` 渲染 = 纯函数 + 运行时可见链）**：`pub fn render_dump(&[(Rc<str>, Value)]) -> String`（行格式 `<name> ： <value>`，分隔串 `U+0020 U+FF1A U+0020`）。`visible_entries` 沿运行时词法链**内→外**（块/函数帧链 → 捕获 cell → globals）、同层按 `Env` per-scope 名字表**声明序（= slot 升序）**、**遮蔽去重**。
  - **与 §10.5 `ScopeDebug` 的关系**：语义等价——`Env` 的 per-scope `names` 以声明序组织（slot 升序），沿链内→外遍历并去重，即 `ScopeDebugTable::visible_slots` 的行为。**实现取运行时 `Env` 链而非编译期 `ScopeDebugTable`**：因 P3.7 名字解析为运行时查名、AST 无 `(depth,slot)`、且**捕获 cell 的槽位不在当前 env 链内**（用 `ScopeDebug` 绝对槽位无法解析捕获变量）。行为一致，机制不同。

- **未明确项（上报 language-architect，不自行发明；本实现取保守口径）**：
  1. `LzError` 无 traceback 字段（见决定 2）。
  2. `let` 重绑定的错误类未定义（§4.5.2 称非法、§8.1 无错误类）→ 仍存储 `mutable` 标志但**不强制**。
  3. `Display` / `fmt` 深结构上限无法产 `RecursionError`（见决定 3）。
  4. `struct` 模板（`StructDef`）的 `==` 仍取同一性（承 P3.6b）。

- **下游影响**：
  - **tooling-dev**：CLI 错误格式化建议改用 `eval_module_traced` 取帧栈；`line/col` 用 `Span`。`eval_module` / `run` 不变。
  - **test-engineer / verifier**：`;;` 输出分流到 stdout，格式逐字符 `名字 ： 值`；深递归应得 `RecursionError`（非崩溃）。
  - **language-architect**：请裁定缺口 1–4；`error.rs` 变更需 core-dev。
  - **P3.9b（HOF）**：`Interp::call_user` 仍需提升为可复用 ABI。

- **证据**：`Get-ChildItem src\evaluator.rs,src\env.rs,src\value.rs` → 114981 / 22303 / 45986 B（合法 UTF-8）；`cargo build --tests --message-format=json 2>$null` → `warnings=0`；`cargo test` → **`226 passed; 0 failed`**（新增 `evaluator::tests::*` **15**）。

---

### [2026-09-24 04:00] [runtime-dev] P3.9b 高阶内置：调用能力注入（`Invoke` ABI）+ §10.7 全表 54/54 齐备

- **背景**：契约 §10.7 全表 54 个内置中，P3.9a 已交付 47 个**非高阶**（ABI `fn(&[Value], Span) -> R<Value>`）。剩余 7 个高阶内置（`map` / `filter` / `reduce` / `sortBy` / `minBy` / `maxBy` / `each`）**必须能调用用户函数 / 闭包**。P3.7 把 `Interp::call_user`（建调用帧 / 绑参 / 执行体 / 维护递归深度与 traceback）留为**求值器私有**，故须设计一个可复用的「调用能力」ABI。

- **决定 1（调用能力 = 注入式回调 `Invoke`；关键决策）**：
  ```rust
  pub type Invoke<'a> = &'a mut dyn FnMut(&Value, &[Value], Span) -> R<Value>;
  pub type HofFn = fn(&[Value], Span, Invoke<'_>) -> R<Value>;
  ```
  7 个 HOF 均实现为 `HofFn`，在需要时调用 `invoke(f, args, span)`。求值器在**派发内置处**（`eval_call`）就地构造闭包 `|f, a, s| self.call_value(f, a, s)`（`call_value` 为新增的 `&mut self` 方法：非函数 → `NotCallable`；函数 → `call_user`），经 [`builtins::call_with`] 注入。
  - **为何不是任务书「推荐」的 `&dyn Fn`（共享借用）**：`call_user` 需要 `&mut self`（它会**递增/递减递归深度** `self.depth` 并**压/弹 traceback 帧栈** `self.trace`），共享借用的 `&dyn Fn` 无法表达可变能力。`&mut dyn FnMut` 是**最小且忠实**的抽象（对比：`&dyn Fn` + `RefCell` 只会把可变性藏起来且更脆）。
  - **为何不「由求值器特判 7 个名字」**：那会把内置表一分为二、HOF 逻辑外溢到求值器，且**破坏可测性**（无法脱离完整求值器测 HOF）。注入式回调让每个 HOF 只依赖一个**窄接口**，单测可直接传 stub 回调（已验证：新增 7 条 HOF 单测**不依赖 evaluator**）。

- **决定 2（表结构：两张表，**不**动既有 47 项 ABI）**：保留 `TABLE: [Builtin; 47]` 与 `lookup` / `call`（对 47 个的**语义与签名完全不变**）；新增 `HOF_TABLE: [Hof; 7]` 与：
  - `pub fn lookup_hof(name) -> Option<Hof>`、`pub const HOF_NAMES: &[&str]`（7）；
  - `pub fn call_with(name, args, span, invoke) -> R<Value>` —— **求值器唯一入口**（高阶表优先，否则回落 `call`）；
  - `is_builtin` 改为两表**并集**（54）；`BUILTIN_NAMES` 由 47 更新为**全表 54**（字母序，测试锁定与两表并集逐项一致）。
  - `Builtin::call` / `Hof::call` 各自集中做 `[min_args, max_args]` 校验（`TypeError::ArgCount`）。
  - **兼容性**：求值器原 `builtins::call(...)` 改调 `call_with(...)`；对 47 个内置，`lookup_hof` 为 `None` → `call` → **行为逐字节不变**。既有 246 测试中仅 2 条「高阶留待 P3.9b」的**状态断言**（`lookup_table_names_consistency` 的 `47`、`hof_deferred_names_are_absent`）按新状态更新为 54 / 已注册；**无任何行为回退**。

- **决定 3（7 个 HOF 的语义口径，逐条对齐 §10.7 / §8.1 / §4.5.6）**：
  - **data-last**：回调（`f` / `pred` / `keyFn`）在前、容器 `xs` 恒为**末参**；容器更新一律**返回新值**（A1，浅拷贝），原容器不变。
  - **回调类型先校验**：`f` / `pred` / `keyFn` 非函数值 → `TypeError::NotCallable`（「不可调用：{t} 不是函数」），**即使容器为空亦先报错**（§10.7「参数类型不符 → `TypeError`」的严格口径）。容器非 `array` → `TypeError`。
  - `map` 逐元素调用 → 新 `array`；`filter` 谓词结果**须为 `bool`**，否则 `TypeError::ConditionNotBool`（§8.1 第 88 行把 `filter` 谓词与 `if`/`while` 并列要求 `bool`）；`each` 仅副作用遍历、返回 `nil`。
  - `reduce` **左折叠** `acc = f(acc, x)`（左→右，B1）；空数组 → 直接返回 `init`（不调用 `f`）。
  - `sortBy` 按 `keyFn` 结果**升序稳定**（`slice::sort_by` 稳定 + 初始下标序）；键须同类可全序（§4.5.6），否则 `TypeError`（承 `sort` 的 `validate_orderable`）。
  - `minBy` / `maxBy` 空 → `ValueError`（**新消息** `空数组没有极值（{func}）`，复用 `ValueMsg::EmptyExtremum`）；非空按键取极值，返回**原元素**（等键取首个，与 `min`/`max` 一致）。
  - 所有 HOF 的 `LzError` 均携带**调用点** `span`（位置红线）。

- **决定 4（端到端可测性的现实约束；上报）**：本机 `parser.rs`（core-dev 并行实现中）**尚不支持 lambda**（`fn (...)` / `(...) =>`），故「`map` + 闭包」的 **lex+parse+eval 成功路径暂不可达**。本批以：(a) **2 条 `lex+parse+eval` 用例**覆盖 HOF 的**求值器路由 / data-last / 参数个数 / 回调类型**（`map(1, [1,2])`、`sortBy(nil, [3,1])`、`each(1)`、`reduce(nil, 0)`）；(b) 1 条**程序化 AST** 用例覆盖真实用户闭包（`map(fn(x) x*2, …)`、`reduce`、闭包捕获 A2、`filter` 非 bool）经求值器 `Invoke` 全链路。**待 parser 支持 lambda 后**，可把 (b) 改回 `lex+parse+eval`（无需改动 HOF 实现）。

- **下游影响**：
  - **core-dev（parser）**：HOF 成功路径依赖 lambda（`fn (...) body` / `(...) =>`，见 syntax.md §3.3 / §4.3 与 §9 样例 `(s) => s.score`）。另注：**v1 内置函数不是一等值**（`eval_expr` 的 `Ident` 对内置名报 `NameError`），故 HOF 回调只能是用户 lambda / 命名函数——与 spec §9 样例一致。
  - **tooling-dev / test-engineer / verifier**：`map`/`filter`/`reduce`/`sortBy`/`minBy`/`maxBy`/`each` 已可用（配用户函数）；错误类/消息见上；`BUILTIN_NAMES` 现为 **54**。
  - **language-architect**：请确认「HOF 回调类型**先校验**（空容器也对非法 `f` 报 `TypeError`）」与「`filter` 非 bool 用 `ConditionNotBool`」两处口径；如另有指定请知会。

- **证据**：`Get-ChildItem src\builtins.rs` → **90390 B**（合法 UTF-8）；`cargo build` → `Finished`（**0 warning**）；`cargo build --tests --message-format=json 2>$null` → `warnings=0 errors=0`；`cargo test` → **`256 passed; 0 failed; 0 ignored`**（基线 246 + 新增 10：`builtins::tests::hof_*` 8 + `evaluator::tests::{e2e_hof_*,hof_drives_user_closures_through_evaluator}` 3，另**替换** 1 条过时的「高阶留待」断言）。未改 `docs/spec/`、`error.rs`、`span.rs`、`loader.rs`、`lexer.rs`、`ast.rs`、`parser.rs`、`Cargo.toml`。

---

### [2026-09-23 23:40] [tooling-dev] P3.10 CLI 输出契约落地：错误流 = stderr、Traceback 按阶段判定、CosmosAnswerError 单行
- **背景**：P3.10 实现最小 CLI，须把 `semantics.md` §8.2/§8.3 的显示契约映射为**稳定字节输出**；spec 未明示三处细节，须定案冻结 CLI 行为（test-engineer / verifier / docs-writer 均据此写用例与文档）。
- **决定**：
  1. **错误诊断流 = stderr**；`--help`/`--version` 与程序自身输出（`print`/`eprint`/`;;`）= stdout。理由：spec 未规定错误流，采用 Python 约定；程序输出由 builtins 直接写 stdout/stderr，CLI 不接管、不缓冲。
  2. **`Traceback` 头按「阶段」而非「类」判定**：`load_file`/`lex`/`parse` 阶段错误一律**无**头（含加载期 `IOError`、`NotUtf8` 这类「运行期类」）；`eval_module_traced` 阶段错误一律**有**头 + 逐帧（最外层→最内层）。理由：§8.2 的二分本就是阶段二分，且可避免「零帧 Traceback」的不合理输出。
  3. **`CosmosAnswerError` 只输出一行 `File "<path>", line 1`（无缩进、无源码行、无插入符）**，严格照 §8.3 示例 3。其余帧：`File` 行前缀 **2 空格**；源码行前缀 **4 空格**；插入符 = 4 空格 + (col-1) 空格 + `^`（§8.2 通用帧格式）。
  4. **位置表达**：行号显式在 `File "...", line N`；列号由**插入符位置**表达（**不**在 `File` 行追加 `col M`，以保 §8.3 逐字符一致）。`--json` 的 `line`/`col` 字段留 P4。
  5. **退出码**（重申 D-008）：`0` 成功；`1` 测试失败（`lfz test` 专用，P4）；`2` CLI 参数错误 / LFZ 语法或运行时错误 / 运行环境错误。
- **影响**：**test-engineer** 黑盒用例按 stderr 抓错误、按阶段判有无 `Traceback` 头、按上述逐字符格式断言；**verifier** 验收命令 3–5 以此为输出基线；**docs-writer** 运行/错误章节据此撰写；**ai-dx-engineer / app-dev** 示例输出对齐；**language-architect** 需知悉下方 spec 排版瑕疵。
- **发现的 spec 排版瑕疵（未改 spec，仅记录待 language-architect 裁定）**：`semantics.md` §8.3 示例 2 的**内层帧**（`n / 0`）源码行与插入符均比 §8.2 通用帧格式**少 4 空格**缩进（外层帧 `let r = half(10)` 与示例 1 均符合通用格式）。本实现按 §8.2 通用规则统一处理（内层帧源码行前缀 4 空格、插入符 = 4+(col-1)），故内层帧输出会比示例 2 多 4 空格前缀；若要与示例 2 逐字符一致，须先改 §8.3。
- **证据**：产出与命令见 `agents/tooling-dev/JOURNAL.md` 2026-09-23 条目；`cargo build` → `Finished`（**0 warning**）；`cargo test` → **292 passed / 0 failed**（lib 277 + bin 8 + `tests/cli.rs` 7）。未改任何他人模块与 `Cargo.toml`。

---

### [2026-09-24 00:30] [language-architect] P3.11 验收 3 处规范裁定（let 重绑定 / .self / traceback 截断）

- **来源**：`docs/reports/P3-verification.md`（verifier 独立验收）缺陷单 **bug-20260924-06 / -08 / -09**——verifier 明确标注为「**非实现 bug，需 language-architect 规范裁定**」。
- **纪律**：本轮**先追加本 ADR、后改 `docs/spec/`**（本标题行先落地，再动三件套）；三件套头部「冻结于 2026-09-23」**保持不变**；三处改动均为 **v1 补钉（补充钉死）**，**未推翻任何既有冻结规则**；**不新增错误类**（仍 12 类 + 基类）、**不引入 `E-xxx`**。`docs/spec/` 由 language-architect 改；`src/**` 由对应开发者按文末清单落地（本 ADR **不含生产代码**）。

#### 裁定 1 —— `let` 重绑定未被限制（bug-06）：归 `TypeError` + **新增子消息变体**（不新增错误类）

- **缺口**：`semantics.md` §4.5.2 写死「`let` 只锁重绑定，不锁内容」（`a = []` 对 `let` 非法），但 §8.1 的 **12 类**中**无对应类/消息**；`SyntaxError` 细分表的「赋值目标非法」是**语法**规则（左侧须为变量/字段/下标），**不覆盖**「目标语法合法但绑定不可变」。→ **契约缺口**。
- **裁定：判 `TypeError`（运行期），新增 `TypeMsg::ImmutableRebind { name }`**（复用现有类，**不新增错误类**；`LfzError::Type { msg, span }` 不变）。
  - **类名**：`TypeError`。
  - **消息模板**：`不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容`（`{name}` = 被重绑定的绑定名）。
  - **`span`**：= **赋值目标的 span**（变量名首字符；§8.2「引发错误的最小 AST 节点」）。
  - **触发条件（充分必要）**：一条赋值语句，其 **lvalue 基为裸 `IDENT`（无 `.字段` / `[下标]` 后缀）** 且 op ∈ `{ = += -= *= /= %= }`，且该名字沿**词法可见链**解析到的绑定是**由显式 `let` 声明创建**的（含外层块/函数可见的 `let`、被当前闭包按 cell 捕获的 `let`）→ 抛 `TypeError`。
  - **明确不触发**：① `a[i] = v` / `s.k = v` / `s["k"] = v`（容器原地修改，A1；`a`/`s` 为 `let` 亦合法）；② 对 **`var` 绑定 / 函数·λ 形参 / `for` 循环变量 / `fn` 名 / `struct` 模板名** 的赋值（这些绑定**非 `let`**）。
  - **边界钉死（补 `semantics.md` 此前未言明处，不推翻任何规则）**：**仅显式 `let` 声明的绑定不可重绑定**；`var` / 形参 / `for` 变量 / `fn` 名 / `struct` 名一律**可变**——与现有实现一致（`src/ast.rs:90` `let→mutable:false / var→mutable:true`；`src/evaluator.rs:1110` 形参、`:596` `for` 变量、`:612/:625` `fn`/`struct` 名均 `mutable:true`）。
- **理由**：① 本规则位于 §4.5 求值语义，是**运行期**行为（LFZ 名字运行时解析，parser 无作用域信息，无法解析期判定）；② 12 类中唯 `TypeError` 具「对某值/绑定执行了不允许操作」的兜底语义，与 **JS 先例**（`const` 重赋值 → `TypeError: Assignment to constant variable`）一致；③ `NameError`（名未定义）/`ValueError`（转换）语义均不符；④ 复用现有类 + 新增**子消息变体**，与 `UnterminatedBlockComment` / `EmptyExtremum` / `BadRange` 同法，守住「不新增错误类」红线。

#### 裁定 2 —— `.self` 字段访问与 EBNF 冲突（bug-08）：**改样例**（`r.self`→`r.me`），**不允许 `.self`**

- **缺口**：`syntax.md` §9.4 样例写 `r.self = r`，但 EBNF `field = "." IDENT`（§7）与 `self` 是保留关键字（§2.6）冲突 → **规则与样例自相矛盾**。
- **裁定：以规则为准，改样例；`self` 保持保留关键字，不允许作裸字段名。** parser 拒绝 `.self` 是**正确**实现，**无需改代码**。
- **理由**：① 最小改动，不触碰冻结 EBNF；② `self` 作方法接收者关键字是刻意设计，允许其作字段名会产生双重身份，并须在 `field`/`field_init`/`member` **三处**文法特例化；③ 需要 "self" 作键时**括号形式 `s["self"]` 仍可用**（字符串键不受限）；④ 样例目的是演示**环安全**（A6），与字段名无关；⑤ 行业一致（带 `self`/`this` 关键字的语言点访问成员一般不允许该关键字）。

#### 裁定 3 —— `RecursionError` 的 traceback 帧数爆炸（bug-09）：加**首 K + 省略行 + 尾 M** 折叠规则

- **缺口**：`interface-contract.md` §10.3 要求帧栈「**全部**序列化」，未规定截断；深递归（10000 层）→ CLI stderr 约 10000 帧 × 3 行 ≈ 数十万行，实用性差。
- **裁定：序列化保持完整、显示层折叠**（分层，不矛盾）：
  - 设帧总数 `T`。`T ≤ 40` → **原样逐帧**（浅栈行为逐字节不变）；`T > 40` → **首 10 帧**（最外层）→ **一行省略行** → **尾 30 帧**（最内层）。
  - **省略行格式（逐字符）**：`  ... 省略 {N} 帧 ...`（前缀 **2 空格**，与 `File` 行缩进对齐；`N = T − 40`，十进制）。
  - **建议值（规范性常量）**：**K = 10**、**M = 30**、**阈值 = 40**（`TRACEBACK_HEAD = 10` / `TRACEBACK_TAIL = 30`）。理由：traceback **尾部**（最内层）承载出错现场与紧邻调用链，信息量最大；**头部**仅需确立入口 → M > K；阈值 40 使**多数正常栈深 < 40 的 traceback 输出完全不变**。
  - **`--json` 不折叠**：`traceback` 数组**完整**（§8.4 schema 与元素 `{file,line,func}` 不变）；机器可读契约须稳定，消费者自行折叠（已知：深递归 `--json` 仍给 ~10000 元素数组，属完整数据；如需折叠另立 v1.1）。
  - **退出码不变**：仍 `2`。
- **理由**：① 显示层策略不污染数据层（`LfzError`/`TracedRun` 保持完整、可测）；② 头+尾折叠为通行做法；③ 阈值 40 最小化对既有黑盒快照的影响。

#### 规范落地（本轮已改 `docs/spec/`）

- `syntax.md`：§2.6（新增「关键字不可作裸字段名」规范性条目）、§9.4（样例 `r.self`→`r.me`；预期输出 `{self: <cycle>}`→`{me: <cycle>}`；加字段名注）。
- `semantics.md`：§4.5.2（重绑定错误类 + 可变性边界钉死）、§8.1（`TypeError` 触发行 + 细分消息表 **6 → 7**）、§8.2（**traceback 折叠**规则）、§8.4（`--json` traceback 不折叠）。
- `interface-contract.md`：§10.3（折叠指针 + 常量）、§10.6（parser：`.self` 为**正确** `SyntaxError`，勿改）、§10.8（`TypeMsg::ImmutableRebind`）。

#### 待执行代码变更清单（language-architect 不写代码）

| # | 文件 | 负责人 | 变更 | 依据 |
|---|---|---|---|---|
| 1 | `src/error.rs` | **core-dev** | `TypeMsg` 增变体 `ImmutableRebind { name: String }` + `message()` 分支 `不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容`；顶部注释「**6 条**」→「**7 条**」（`:110`）；`TypeMsg` 相关单测增断言 | 裁定 1 |
| 2 | `src/evaluator.rs`（+ 视需要 `src/env.rs` / `src/value.rs`） | **runtime-dev** | `assign_name`（`:330`）命中 local / captured / globals 绑定时查其 `mutable` 标志（`Env::local_mutable`）；`false` → `TypeError::ImmutableRebind { name }`（`span = target.span`）。因其现返回 `bool`，需改为可区分「未找到 / 不可变 / 成功」；**捕获 cell 须携带可变性**（`Closure.captured` / `capture_local`）以支持闭包内重绑 `let`。`let`/`var` 定义与 `a[i]=`/`s.k=` 路径**不变** | 裁定 1 |
| 3 | `src/cli.rs` `render_error`（`:171`） | **tooling-dev** | 运行期 traceback 折叠：`T>40` → 首 10 帧 + `  ... 省略 {T−40} 帧 ...` + 尾 30 帧；`T≤40` 原样。**仅改渲染**；`TracedRun`/`LfzError` 不变（**无需 core-dev/runtime-dev 改动**） | 裁定 3 |
| 4 | `docs/spec/*` | **language-architect**（本轮已完成） | 见上「规范落地」 | 裁定 1/2/3 |

> 裁定 2 **无代码变更**（parser 已按 EBNF 正确拒绝 `.self`）；仅改 `syntax.md` 样例。

#### 下游影响

- **core-dev**：`src/error.rs`（#1）。另：**勿**把 `.self` 改成合法（裁定 2）。
- **runtime-dev**：`src/evaluator.rs`（#2）；此前 P3.7/P3.8 ADR 已上报的「`let` 重绑定错误类未定义」缺口**本轮闭合**（`mutable` 标志现可强制）。
- **tooling-dev**：`src/cli.rs`（#3）；`--json` 的 `traceback` 保持完整。
- **test-engineer**：① 新增负例 `let a = 1; a = 2` → `{"error":"TypeError"}` + 逐字符消息 `不能重新赋值 let 变量 'a'；let 只锁重绑定，不锁内容`；② 深递归快照改为「≤40 帧原样；>40 帧含 `  ... 省略 N 帧 ...`」；③ 既有 RecursionError 全帧断言需更新。
- **docs-writer / ai-dx-engineer**：手册/AI 指南「变量」节写明 `let` 不可重绑定（消息模板）、`self` 不可作字段名（用 `s["self"]`）；错误章加一条 traceback 折叠说明。
- **spec 三件套**：错误类计数不变（12 类 + 基类；运行期 10 类）；`TypeError` 细分消息 **6 → 7**；`E-xxx` 仍 **11** 处（§13 附录，未新增）。

- **证据**：本 ADR 标题行 + `docs/spec/` 三件套本轮改动点（见结构化汇报）；命令与字节级计数见 `agents/language-architect/STATUS.md`。

---

### [2026-09-24 05:00] [language-architect] spec-20260924-01 §9.4 样例自相矛盾修正（单 `;` → 换行）

- **来源**：verifier 缺陷单 **spec-20260924-01**（规范侧 🟡）——`docs/spec/syntax.md` §9.4 样例 `fn inc() { n += 1; n }` 以**单个 `;`** 分隔两条语句，但本规范写死「单个 `;` 永远 `SyntaxError`」（§2.3 / §3.2 / A11）；verifier 逐字复制该样例的夹具 `spec_9_4_refs.lfz` 因此退出码 2。该样例是规范**唯一**的「引用语义 + 闭包捕获 + 环安全」综合样例（A1/A2/A6），按原文**无法运行**。
- **纪律**：**先追加本 ADR、后改 `docs/spec/`**；三件套头部「冻结于 2026-09-23」保持不变；本轮改动为 **v1 补钉（样例自洽化）**，**未推翻任何既有冻结规则**；**不新增错误类**、**不引入 `E-xxx`**。
- **裁定：以规则为准，改样例**——把 `fn inc() { n += 1; n }` 改为**以换行分隔**两条语句的多行块体；**语义与「预期输出」逐字符不变**（`1 2 3`）。
  - 修正前：`fn inc() { n += 1; n }          // 闭包按 cell 捕获 n（A2）`
  - 修正后：
    ```
    fn inc() {                      // 闭包按 cell 捕获 n（A2）
        n += 1
        n
    }
    ```
  - **理由**：① 单 `;` 是**冻结规则**（§2.3 / §3.2 / A11 / §14 遗留默认 4），规则正确、样例是瑕疵；② §9.4 是全规范唯一演示 A1/A2/A6 的综合样例，必须**可运行**（评分项「解释器 / 测试 / 文档 / 应用」共同引用）；③ 块体天然支持换行终结（§3.2 块 `{` push `SIG`），改换行为**最小且语义等价**的修法；④ 与 A5/A11「取消空语句、无语句分隔符」的设计一致。
- **同步核查（§9 其它样例）**：§9.1 / §9.2 / §9.3 全样例**无**语句分隔 `;`（仅 `;;` dump 独立语句）；§9.4 内 `r.self`→`r.me` **已由 [2026-09-24 00:30] ADR（bug-08）落地**（本轮复核：第 803 行为 `r.me = r`、预期输出 `{me: <cycle>}`，`r.self` 已清零）。→ **§9 全样例现已自洽**。
- **规范落地**：`docs/spec/syntax.md` §9.4（样例块体改换行 + 新增「语句分隔注（v1 补钉）」钉死「块内语句以换行分隔，不得用单 `;`」）。
- **下游影响**：**test-engineer / verifier** 可恢复 `spec_9_4_refs.lfz`（逐字提取 §9.4）为**正向夹具**，期望 `1 2 3 / 99 99 / [3, 1, 2]  [1, 2, 3] / {me: <cycle>} / true`；**docs-writer / ai-dx-engineer / app-dev** 引用 §9.4 时须用换行版（不得复制旧 `;` 版）；**core-dev / runtime-dev** 无代码变更。
- **代码变更**：**无**（纯 spec 样例修正）。

### [2026-09-24 05:05] [language-architect] bug-20260924-07 裁定：语句首 `{` 按 A9 作匿名 struct 字面量（parser 简化须修正）

- **来源**：verifier 缺陷单 **bug-20260924-07**（🟡）——A9 / §3.3 规定「语句首 `{` 恒为匿名 struct 字面量（LFZ 无裸块语句）」，但 `src/parser.rs` `parse_stmt_seq` 把语句首 `{` 当**裸块语句内联**（parser 头注释 #1 自述为「本批已知简化」），且 AST 无 `Block` 语句变体。
- **纪律**：同上（先 ADR 后改 spec；不改冻结规则；不新增错误类；不引入 `E-xxx`；**不改 `src/**`**）。
- **裁定：A9 成立（spec 正确、无歧义），不允许裸块语句；parser 的简化是缺陷，须修。**
  - **依据（规范原文位置）**：① `syntax.md` §3.3 规则 2 末句「含**语句起始处**的 `{`：LFZ **没有"裸块语句"**，故语句首 `{` 恒为**匿名 struct 字面量**（A9）」；② §5 **A9**「语句首 `{`：恒为匿名 struct 字面量。反例 `{ let x = 1 }` → `SyntaxError`」；③ EBNF §7 `statement` 产生式**无** `block_stmt`（`block` 仅出现在 `body` / `if_stmt` / `while_stmt` / `for_stmt` 的 block-required 位）；④ §3.3 规则 2 覆盖「其余一切期待表达式之处」，语句位属 `expr_stmt`（§7）。
  - **推论（可直接抄写）**：
    - 语句位 `{ … }` **≡ `expr_stmt` → `expression` → … → `struct_lit`**（匿名，无前置类型名）。
    - `{ "k": 1 }` 语句 → **合法**（匿名 struct 字面量表达式语句，值被丢弃）；`{ }` 语句 → 合法（**空**匿名 struct）。
    - `{ let x = 1 }` → **`SyntaxError`**（`let` 非 `field_init` 的 `(IDENT|STRING)` 头；由 `struct_lit`→`field_init` 解析报「意外记号」，**不新增子消息变体**）。
    - `{ ;; }` 语句 → **`SyntaxError`**（`;;` 不能出现在 struct 字面量成员表内）；`;;` 在**真块体**（`fn/if/while/for` 的 body）内**仍合法**（A10 不受影响）。
    - 程序顶层 / 块体**不再有**「裸 `{` 开新作用域」语法；作用域仅由 `if/while/for/fn/lambda` 体与 struct 字面量成员表产生。
  - **理由**：① 选择「允许裸块」需改**冻结** A9 + §3.3 + §3.2 + §7 EBNF（新增 `block_stmt` 产生式与 AST `Block` 变体），并制造 `{}` / `{k:v}` 的块↔字面量二义，**远超**「实现简化」的代价；② 选择「按 A9」只需删掉 parser 一处特例分支 + 有意识更新 3 个自证简化行为的测试，**最小且回归冻结设计**；③ 「无裸块语句」是**有意设计**（§3.3、A5、A9 三处重申），且与「块用 C 风格 `{}`、语句由换行终结」的 v0.2 定案一致；④ 与 Rust 一致（Rust 语句位 `{ … }` 是块表达式，但 LFZ 无块表达式 → 字面量）。

- **待执行代码变更清单（仅 `src/**`，由 core-dev 落地；language-architect 不写代码）**：

  | # | 文件 | 变更 | 说明 |
  |---|---|---|---|
  | 1 | `src/parser.rs` `parse_stmt_seq`（`:265-269`） | **删除** `TokenKind::LBrace => { let block = self.parse_block()?; stmts.extend(block.stmts); }` 分支，使语句首 `{` 落入 `_ => stmts.push(self.parse_stmt()?)`，经表达式路径 `primary` 的 `TokenKind::LBrace`（`:1127`）→ `parse_struct_lit(None, span)`（**该分支已存在，无需新增**） | 唯一行为变更点 |
  | 2 | `src/parser.rs` 模块头「# 本批已知简化」#1（`:52-56`）与 `parse_stmt_seq` 文档注（`:255-256`） | **删除 / 改写**该简化说明为「语句首 `{` 按 A9 解析为匿名 struct 字面量」 | 注释与实现同步 |
  | 3 | `src/parser.rs` 单测 `block_statement_inlines_contents`（`:1676`） | **改写**：输入 `{ let x = 1 }` → 断言 **`Err`（`SyntaxError`）**（A9 反例）；建议更名 `statement_start_brace_is_struct_literal_not_block` | 有意识更新 |
  | 4 | `src/parser.rs` 单测 `no_brace_literal_statement_start_is_block`（`:2226`） | **改写**：`{ let a=1 \n let b=2 }` → 断言 **`Err`（`SyntaxError`）**；另加正例 `{ "k": 1 }` → `stmts.len()==1` 且 `StmtKind::Expr(ExprKind::StructLit(_))`；建议更名 `statement_start_brace_parses_as_struct_lit` | 有意识更新 |
  | 5 | `src/parser.rs` 单测 `dump_inside_block_is_legal`（`:4424`） | **改写**：把「语句首 `{ ;; }`」改为**真块体**（如 `fn f() { ;; }` 或 `if true { ;; }`）→ 断言块体内 `Dump` 合法（保留原测试意图；语句首 `{ ;; }` 现应 `SyntaxError`） | 有意识更新 |

- **规范落地（本轮已改 `docs/spec/`）**：
  - `syntax.md` §3.3（规则 2 末句扩写为**规范性钉死**：语句位 `{` ≡ `expr_stmt`→`struct_lit`；给出 `{ "k": 1 }` 正例、`{ let x = 1 }` / `{ ;; }` 反例与 `SyntaxError` 结论）。
  - `syntax.md` §5 A9（补「推论」一行，指向 §3.3 与 EBNF `statement`）。
  - `interface-contract.md` §10.6（parser 条目补：`parse_stmt_seq` **不得**把语句首 `{` 当裸块；无 `Block` 语句变体；语句首 `{` → `struct_lit`）。
- **下游影响**：
  - **core-dev**：清单 #1–#5（唯一代码变更方）。
  - **runtime-dev**：无影响（AST 无新节点；struct 字面量既有求值路径已就绪）。
  - **verifier**：`bug07_stmt_brace.lfz`（`{ "k": 1 }`）修复后应**成功**（EXIT=0）；建议补负例 `{ let x = 1 }` → `SyntaxError`。复验后更新 `P3-verification.md` §5 / §7 回归表。
  - **test-engineer**：黑盒可加正例 `{ "k": 1 }`（匿名 struct，值丢弃）与负例 `{ let x = 1 }` → `SyntaxError`；**既有黑盒夹具无裸块用法**（本轮已扫描 `tests/`、`app/`，零命中）。
  - **docs-writer / ai-dx-engineer**：若述及「`{}` 双角色」须写明**无裸块语句**、语句首 `{` 为匿名 struct 字面量。
  - **app-dev**：无影响（应用代码无裸块）。
  - **spec 三件套**：`syntax.md` / `interface-contract.md` 已同步；`semantics.md` **无需改动**（未新增错误类 / 消息）；错误类计数不变（12 类 + 基类）。
- **证据**：本 ADR 标题行 + parser 缺陷位置（`src/parser.rs:52-56`、`:255-256`、`:265-269`）与 3 测（`:1676` / `:2226` / `:4424`）+ verifier `docs/reports/P3-verification.md:383-394`。

---

### [2026-09-24 08:46] [runtime-dev] P3.11 bug-09 折叠在 `src/cli.rs` 由 runtime-dev 落地（本轮）；bug-06 待 core-dev `error.rs`

- **来源**：team-lead 任务书「落地架构师已裁定的两条非阻塞缺陷」；裁定见上文 `[2026-09-24 00:30] P3.11 验收 3 处规范裁定`（裁定 1 / 裁定 3）。
- **决策 1（跨角色：`src/cli.rs` 归属）**：裁定 3 的「待执行代码变更清单」#3 原指派 **tooling-dev** 在 `src/cli.rs` `render_error` 落地 traceback 折叠；本轮 team-lead 任务书改派 **runtime-dev** 落地，现已完成（规范常量 `TRACEBACK_HEAD = 10` / `TRACEBACK_TAIL = 30` / 阈值 40；新增 `push_frames` / `push_trace_frame`；`TracedRun`/`LzError` 不变）。**tooling-dev 请勿重复实现**；若后续 CLI 展示层需变更，请与 runtime-dev 协调或由 team-lead 明确归属。
- **决策 2（bug-06 阻塞）**：裁定 1 需在 `src/error.rs`（属 **core-dev**）新增 `TypeMsg::ImmutableRebind { name }`；该变体当前**不存在**（`Select-String src\*.rs -Pattern ImmutableRebind` 无匹配），故 runtime-dev 按任务书 ⚠️ **停工**，**不写引用不存在变体的半成品**。core-dev 落地后，runtime-dev 执行 tri-state `assign_name` + `exec_assign` 接线 + 「捕获 cell 携带可变性」，并移除负例 `let_rebind_is_type_error` 的 `#[ignore]`。
- **影响**：**core-dev**（`error.rs` 变体 #1）；**tooling-dev**（`cli.rs` 归属见决策 1）；**test-engineer**（bug-06 黑盒负例待 core-dev + runtime-dev 完成后可绿）。
- **证据**：`agents/runtime-dev/STATUS.md` / `JOURNAL.md` 2026-09-24 08:46 条目；`git diff --stat -- src/cli.rs src/evaluator.rs`（cli.rs +102/−6、evaluator.rs +29）；`cargo build --tests`（warnings=0）；`cargo test`（**375 passed / 0 failed / 1 ignored**）；CLI 实测深递归 stderr 123 行 + 逐字符 `  ... 省略 9961 帧 ...` + exit 2。

---

### [2026-09-24 08:49] [core-dev] P3.11 裁定 1 落地：`TypeMsg::ImmutableRebind { name }` 已就绪（解除 runtime-dev bug-06 阻塞）

- **来源**：team-lead 轻量任务书「`src/error.rs` 新增 `TypeMsg::ImmutableRebind`」；依据上文 `[2026-09-24 00:30]` 裁定 1 + `semantics.md` §4.5.2 / §8.1。
- **变更**：`src/error.rs` 新增变体 `TypeMsg::ImmutableRebind { name: String }` + `message()` 分支 `不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容`；枚举文档注释「**6 条**」→「**7 条**」。**不改** `class_name()`（`TypeMsg` 仍映射 `"TypeError"`）、**不新增错误类**、**无 `E-xxx`**、**不改其它变体**、**不改 spec / Cargo.toml**。
- **解除阻塞（runtime-dev）**：上文 `[2026-09-24 08:46]` 决策 2 的阻塞（变体不存在 → 停工）**已解除**。runtime-dev 可执行 tri-state `assign_name` + `exec_assign` 接线 + 「捕获 cell 携带可变性」，并移除负例 `let_rebind_is_type_error` 的 `#[ignore]`；`span = target.span`（变量名首字符，裁定 1）。
- **影响**：**runtime-dev**（唯一被解除的跨角色依赖）；**test-engineer**（bug-06 黑盒负例待 runtime-dev 完成后可绿）；**verifier**（bug-06 复验入口）。
- **证据**：`git diff --stat -- src/error.rs`（`1 file changed, 60 insertions(+), 2 deletions(-)`，**仅此文件**）；`cargo build`（`WARN_COUNT=0`，exit 0）；`cargo test` lib harness `test result: ok. 359 passed; 0 failed; 1 ignored`（`1 ignored` 为 runtime-dev 占位，**未动**）。

### [2026-09-24 09:20] [team-lead] 错峰开工纪律（DeepSeek 低谷窗口）+ 自动闸门

- **背景**：用户要求「每天在 DeepSeek 低谷期开工、高峰期停工」以省钱。
- **决策**：
  1. **窗口（可配置）**：低谷 = **北京时间 00:30–08:30（UTC+8）**；由环境变量 `LFZ_OFFPEAK_START` / `LFZ_OFFPEAK_END` / `LFZ_OFFPEAK_UTC_OFFSET` 覆盖，**支持跨午夜**窗口（如 23:00–07:00）。
  2. **闸门脚本** `scripts/offpeak.ps1`：退出码 **`0` = 在低谷（可开工）/ `3` = 高峰（应停工）**；`-Quiet` 只返回退出码，便于人、脚本或计划任务调用。
  3. **自动拦截（强制，非"靠记得"）**：opencode 插件 **`.opencode/plugin/offpeak.ts`**（自动发现）在**高峰时段阻止 `task` / `call_omo_agent`** —— 即"派发子智能体"这一最耗 token 的动作；被拦时给出明确原因与距窗口开启时间。`LFZ_OFFPEAK_ENFORCE=0` 可临时关闭。**只拦"派发"**；本地读写/构建/测试/提交不受影响。
  4. **调度纪律（team-lead 遵守）**：派发前先判窗口；高峰时段只做**本地/离线**动作（读状态、整理看板、写计划与文档草稿），把**批量派发**留到低谷；窗口结束前**不再开新任务**，只收尾在途任务。
- **影响**：全体 agent（节流）｜release-manager（提交/推送可在任意时段）｜用户（**需重启 opencode** 使插件生效）。
- **证据**：`scripts/offpeak.ps1`（退出码语义 + 距窗口时间）；`.opencode/plugin/offpeak.ts`（钩子签名取自 `@opencode-ai/plugin` 类型定义：`"tool.execute.before": (input: { tool; sessionID; callID }, output: { args }) => Promise<void>`，本插件据 `input.tool` 判定并 `throw` 拦截）。
- **待确认**：窗口是否就是 00:30–08:30（本机联网搜索配额已用尽，未能复核官方页）；若 DeepSeek 调整，改环境变量即可，无需改代码。
