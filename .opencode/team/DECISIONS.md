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
