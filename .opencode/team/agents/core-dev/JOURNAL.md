# core-dev — 工作日志
> 只追加，最新条目在最上方。
## [2026-09-24 00:35] P3.9a 契约缺口闭合：`ValueMsg` 新增 2 变体（core-dev 侧）
- 来源: team-lead 轻量任务「按 architect 裁定，`src/error.rs` 新增 2 个 `ValueMsg` 变体（含 `message()` 分支与单测）」
- 完成: 先读 `DECISIONS.md` 最新 ADR「[2026-09-24 00:20] [language-architect] P3.9a 契约缺口闭合（6 项）」第 1/2 项 + `docs/spec/semantics.md` §8.1 表与新增细分消息表 → 改 `src/error.rs` → 编译 → 补单测 → 汇报。
- 产出:
  - `src/error.rs`（+152 / -1 行）。
  - 新增变体：`ValueMsg::EmptyExtremum { func: String }`、`ValueMsg::BadRange { lo: i64, hi: i64 }`。
  - `message()` 分支：`format!("空数组没有极值（{func}）")`、`format!("区间非法：{lo} >= {hi}")`。
  - 证据：`git diff --stat -- src/error.rs` → `1 file changed, 152 insertions(+), 1 deletion(-)`；`cargo build` → `Finished dev profile ... in 1.75s`，**WARNCOUNT=0**，exit 0；`cargo test` → `test result: ok. 117 passed; 0 failed; 0 ignored`，exit 0。
  - 新增 4 项测试：`value_msg_covers_all_four_rows`、`empty_extremum_message_char_by_char`、`bad_range_message_char_by_char`、`new_value_variants_still_map_to_value_error`（逐字符断言，含 `min/max/minBy/maxBy` 与 `3 >= 3`、负数、`i64::MIN`）。
- 决策:
  - **不改 `class_name()`**：两新变体仍映射 `"ValueError"`；`LzError` 仍 12 变体；无新增错误类、无 `E-xxx`。
  - **不改 spec / builtins / span / loader / lexer / Cargo.toml**（守边界；`builtins.rs` 由 runtime-dev 按本变体改造）。
  - 枚举文档注释由「两条/双变体」更正为「四条/四变体」（如实反映 §8.1 增补）。
- 下一步: 等 team-lead 派发 P3.3b（`lexer.rs` 字符串/插值第二批）；runtime-dev 消费新变体改造 `min/max/minBy/maxBy`/`randInt`。
- 阻塞: 无。
## [2026-09-23 23:58] P3.3a lexer 第一批（CODE 模式核心记号）
- 来源: team-lead 任务书「P3.3a — `src/lexer.rs` 第一批：CODE 模式核心记号（不含字符串/插值）」
- 完成: **先落盘实现、再补测试**（遵守上轮教训：禁止长时间推演）。
  - 先读 `syntax.md` §2.3–§2.8 → **立刻写 `src/lexer.rs`** → 编译 → 补单测 → 汇报。
  - 一次定义完整 `TokenKind`（含 P3.3b 的 `StrBegin/StrEnd/Text/InterpBegin/InterpEnd/FormatSpec`）+ `Token{kind,span}` + `pub fn lex(text, line_base) -> R<Vec<Token>>`。
  - 实现：空白跳过、`//` 行注释、`/* */` 块注释（等价空格、不产 Newline）、`\n`→`Newline`、标识符/16 关键字/14 保留字/`_`→`Placeholder`、数值（十/`0x`/`0b`/`0o`、浮点 `.digit` 与指数、原文保留、不完整指数回退）、运算符分隔符（§2.3 最大匹配）、`#`→`HashPosition`、其余→`IllegalChar`。
  - 位置：`line=本地行号+line_base`，`col` 按 Unicode 标量计数。
- 产出:
  - `src/lexer.rs`（**27396B / 824 行**，含测试；原桩 143B）
  - 证据：`Get-ChildItem src\lexer.rs`→Length=27396；`cargo build`（强制重编）→ `Finished`，**WARN=0**，exit 0；`cargo test` → `test result: ok. 113 passed; 0 failed; 0 ignored`，exit 0（新增 31 项 lexer 测试）。
- 决策:
  - **回退规则（消歧）**：`1e`→`Int("1")`+`Ident("e")`；`1.`→`Int("1")`+`Dot`（A23）；`0x`（无位）→`Int("0")`+`Ident("x")`。
  - **契约缺口**：未闭合 `/*`（至 EOF）在 `SyntaxMsg` 无对应变体 → 本批「消费至 EOF、不报错、不自造消息」，请 architect 裁定。
- 下一步: P3.3b 字符串/插值（模式栈 CODE/STR/INTERP、转义、未闭合/插值跨行错误）。
- 阻塞: 无
## [2026-09-23 22:02] P3.2 加载器（loader.rs）
- 来源: team-lead 任务书「P3.2 — `src/loader.rs`」（契约 §10.1 / syntax.md §2.2）
- 完成: 实现 `src/loader.rs`（纯加载，不碰 lexer/parser/evaluator）。
  - `Loaded{text,line_base}` + `load_file`（读→UTF-8→BOM→归一化→按扩展名分流，#42 前导）+ `load_source`（不要求前导，line_base=0）+ `ext()`/`is_lfz()`（§2.2.0 四步）+ 私有 `normalize()`/`consume_preamble()`。
  - TDD 红→绿：先写 12 项测试 + `todo!()` 骨架，`cargo test` → 12 失败（not yet implemented）；再实现 → 全绿。
- 产出:
  - `src/loader.rs`（15873B / 384 行，含测试）
  - 证据：`cargo build`（改 mtime 强制重编）→ `Finished dev profile ... in 0.86s`，WARNINGS=0，exit 0；`cargo test` → `test result: ok. 27 passed; 0 failed; 0 ignored`（15 旧 + 12 新），exit 0
  - §10.1 四要点落点：职责 L62–84/L98–109；双入口+Loaded L20–26/L62/L89–95；ext 四步 L37–44/L50–53；前导不产 token + line_base L116–126/L82/L93。
- 决策: 3 处「契约待确认」按最小合理读法实现，列 STATUS【阻塞/需支持】请 architect 裁定（不阻塞）：①`NotUtf8.span`=`Span::START`（精确偏移由消息承载）②`#42` 消费后 `text` 自 `#42\n` 之后起（依 §2.2.2 7d）③`ext` 返回 `Option<&str>` 原始子串 + 独立 `is_lfz()`（步骤 4 折叠）。
- 下一步: 待 team-lead 派 P3.3 `lexer.rs`。
- 阻塞: 无
## [2026-09-23 23:40] P3.1 基座类型（span + error）
- 来源: team-lead 任务书「P3.1 — 基座类型」（PLAN-P3 §3 串行门禁子阶段）
- 完成: 实现 `src/span.rs` 与 `src/error.rs`（本项目错误与位置的唯一事实源），纯实现 + 单测，不涉词法/语法/求值。
  - `Span{line,col}`（1-based、Unicode 标量列）+ 常用 trait + `Display`（`line 3, col 5`）+ `Span::START`。
  - `LzError` 12 变体（严格照抄 §10.4 字段与顺序）+ `class_name()`/`message()`/`span()` + `Display`。
  - 子消息枚举 `SyntaxMsg`(16) / `TypeMsg`(6) / `OverflowMsg`(1) / `ValueMsg`(2)，逐条对应 `semantics.md` §8.1 细分表。
  - `R<T> = Result<T, Box<LzError>>` + 12 个 `#[cold] #[inline(never)]` 构造器 + `TraceFrame`；`VmFrame` 按树遍历路线本阶段跳过并已说明。
- 产出:
  - `src/span.rs`（2305B）、`src/error.rs`（32211B）
  - 证据：`cargo build`（强制重编）→ `Finished ... in 0.63s`，WARNING_LINES=0，exit 0；`cargo test` → `test result: ok. 15 passed; 0 failed`，exit 0；`cargo tree` → 仅 `lfz v0.1.0`（零依赖）
  - 15 测试覆盖：12 个 `class_name()` 逐字断言 + 数量/去重/基类不直接抛 + `E-` 前缀禁止；各细分表逐行 `message()` 断言；`CosmosAnswer` 固定消息；`span()` 的 `None`/`Some`；`size_of::<R<()>>() == size_of::<usize>()`。
- 决策: `Assert.msg` / `Io.msg` 视为**已组装完成的最终消息**（由构造点决定包装），`message()` 原样返回 —— 这是单字段对齐 §10.4 的唯一可行口径；已在 STATUS 列为待 architect 确认项，不阻塞。
- 下一步: 待 team-lead 派 P3.2 `loader.rs`。
- 阻塞: 无（1 处歧义已最小化落定，见 STATUS）。
## [2026-09-23 21:55] P3.0 Cargo 工程骨架
- 来源: team-lead 任务书「P3.0 — Cargo 工程骨架」（PLAN-P3 §3 门禁子阶段）
- 完成: 手写 `Cargo.toml`（lib+bin 均名 `lfz`、edition 2021、`[dependencies]` 空）+ `src/lib.rs`（声明 10 模块）+ `src/main.rs`（占位）+ 10 个模块占位文件；`cargo build` / `cargo test` 双绿。
- 产出:
  - `Cargo.toml`、`Cargo.lock`（147B，仅 `lfz`）、`src/{lib,main}.rs`、`src/{span,error,loader,lexer,ast,parser,value,env,evaluator,builtins}.rs`（共 12 个 .rs）
  - 证据：`cargo build` → `Finished dev profile ... in 2.87s`（exit 0，无 warning）；`cargo test` → 3 harness 全 `test result: ok`（0 tests，exit 0）；`cargo tree` → 仅 `lfz v0.1.0`（零第三方依赖）
- 决策: 采用**手写 Cargo 工程**而非 `cargo init`，以规避 cargo 自动覆盖/追加 `.gitignore`、`README.md`、`LICENSE`；未改这三个已有文件。
- 下一步: 待 team-lead 派发 P3.1（`span.rs` + `error.rs`，core-dev 一次写全）。
- 阻塞: 无
## [2026-09-22] 角色初始化
- 来源: 团队初始化（系统构建）
- 完成: 角色定义与活文档建立
- 产出: `.opencode/agents/core-dev.md`
- 下一步: 等待 team-lead 调度
