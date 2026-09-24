# language-architect — 工作状态
> 最后更新: 2026-09-24 05:10 by language-architect

## 当前状态
**✅ spec v1 已冻结（FROZEN，2026-09-23）；本轮完成 post-v1 「规范侧收尾」两项：① spec-20260924-01（§9.4 样例自相矛盾）修正；② bug-20260924-07 / A9（语句首 `{`）裁定。均：先 ADR 后改 spec、不改 `src/**`、不新增错误类、不引入 `E-xxx`。**
- 交付：`.opencode/team/DECISIONS.md` 追加 2 条 ADR「spec-20260924-01 §9.4 样例自相矛盾修正（单 `;` → 换行）」+「bug-20260924-07 裁定：语句首 `{` 按 A9 作匿名 struct 字面量」；`docs/spec/{syntax.md, interface-contract.md}` 补充钉死（`semantics.md` 无需改动）。
- 纪律：三件套头部「冻结于 2026-09-23」保持；本轮改动均为**补充钉死 / 样例自洽化**，未推翻任何既有冻结规则。

## 本轮裁定（缺陷 → 裁定 → 落地）
| # | 缺陷单 | 缺口 | 裁定 | 代码变更 |
|---|---|---|---|---|
| 1 | spec-20260924-01 | §9.4 样例 `fn inc() { n += 1; n }` 用**单个 `;`** 分隔语句，与「单 `;` 永远 `SyntaxError`」（§2.3/§3.2/A11）矛盾；唯一 A1/A2/A6 综合样例按原文**无法运行**（verifier `spec_9_4_refs.lfz` 退出码 2） | **以规则为准，改样例**：`fn inc()` 块体改**换行分隔**；语义与预期输出**逐字符不变**（`1 2 3`） | **无**（纯 spec） |
| 2 | bug-20260924-07 | 语句首 `{` 被 `parse_stmt_seq` 当**裸块语句内联**，与 A9/§3.3「无裸块语句、语句首 `{` 恒为匿名 struct 字面量」不符 | **A9 成立**（spec 正确）；**不允许裸块**；**parser 简化是缺陷，须修** | **需 core-dev**（删 1 分支 + 改 3 测 + 注释） |

**裁定 1 理由（4 条）**：① 单 `;` 是冻结规则、样例是瑕疵；② §9.4 是全规范唯一演示 A1/A2/A6 的综合样例，必须可运行；③ 块体支持换行终结（§3.2 块 `{` push `SIG`），改换行最小且语义等价；④ 与 A5/A11「无语句分隔符/取消空语句」一致。
**裁定 2 理由（4 条）**：① 允许裸块需改**冻结** A9+§3.3+§3.2+§7 EBNF（新增 `block_stmt`/AST `Block`）并制造 `{}`/`{k:v}` 二义，代价远超实现简化；② 按 A9 只删 parser 一处特例 + 更新 3 个自证简化行为的测试，最小且回归冻结设计；③ 「无裸块语句」为有意设计（§3.3/A5/A9 三处重申）；④ 与 Rust 一致。

## 边界钉死（本轮一并写死）
- **语句位 `{ … }` ≡ `expr_stmt`→`expression`→…→`struct_lit`（匿名）**：正例 `{ "k": 1 }`（合法，值被丢弃）、`{ }`（合法，空匿名 struct）；反例 `{ let x = 1 }` / `{ ;; }` → `SyntaxError`（由 `field_init` 解析报「意外记号」，**不新增子消息变体**）；`;;` 在 `fn/if/while/for` 真块体内仍合法。
- **§9 全样例自洽**：§9.1/§9.2/§9.3 无语句分隔 `;`；§9.4 `r.self`→`r.me` 已落地（复核第 810 行 `r.me = r`、输出 `{me: <cycle>}`）。

## 待执行代码变更清单（交 core-dev；本轮未写代码）
| # | 文件 | 负责人 | 变更 |
|---|---|---|---|
| 1 | `src/parser.rs` `parse_stmt_seq`（`:265-269`） | core-dev | **删除** `TokenKind::LBrace => { parse_block 内联 }` 分支，落 `_ => parse_stmt()`；经 `primary` LBrace（`:1127`）→ `parse_struct_lit(None, span)`（分支已存在） |
| 2 | `src/parser.rs` 头注释「# 本批已知简化」#1（`:52-56`）+ `parse_stmt_seq` 注（`:255-256`） | core-dev | 删除/改写简化说明 |
| 3 | `src/parser.rs` 测 `block_statement_inlines_contents`（`:1676`） | core-dev | 改为断言 `{ let x = 1 }` → `Err(SyntaxError)` |
| 4 | `src/parser.rs` 测 `no_brace_literal_statement_start_is_block`（`:2226`） | core-dev | 改为负例 `Err` + 正例 `{ "k": 1 }` → `StructLit` |
| 5 | `src/parser.rs` 测 `dump_inside_block_is_legal`（`:4424`） | core-dev | 改为**真块体**（`fn f() { ;; }`）内 `Dump` 合法 |

## 产出与证据（可命令验证）
| 文件 | 行数（LF） | 说明 |
|---|---|---|
| `docs/spec/syntax.md` | 913 → **920** | §3.3（语句位 `{` 规范性钉死）、§5 A9（补推论）、§9.4（样例改换行 + 「语句分隔注」） |
| `docs/spec/interface-contract.md` | 311 → **311**（行数不变，单行扩写） | §10.6（parser：语句首 `{` 不得按裸块；无 `Block` 变体） |
| `docs/spec/semantics.md` | 415（**未改动**） | 本轮未新增错误类/消息 |
| `.opencode/team/DECISIONS.md` | 543 → **602** | 追加 2 条 ADR（标题行 **L547** / **L566**） |

- 字节级：`syntax.md` bytes=62389 LF=920 CR=0 BOM=False last=LF(10)；`semantics.md` bytes=33931 LF=415 CR=0；`interface-contract.md` bytes=30592 LF=311 CR=0；三者均 UTF-8 无 BOM。
- `DECISIONS.md` bytes=99390 LF=602 CR=602（该文件**固有 CRLF**，非本轮引入）。
- 串命中：新增 `语句分隔注`、`无 block_stmt`、`须落表达式路径`；`n += 1;` 已清零（§9.4）；`r.self` 仅存于 §2.6 **反例**（预期）。
- 错误类计数保持：`12 类 + 基类`、`运行期 10 类`；**未新增 `E-xxx`**。

## 进行中
- （无；待 team-lead 转派 core-dev 落地 parser 变更清单 #1–#5）

## 阻塞 / 需要支持
- 无。

## 下一步计划
- team-lead 派发：core-dev → `src/parser.rs`（清单 #1–#5）。
- 通知下游：test-engineer / verifier（`spec_9_4_refs.lfz` 恢复为正向夹具，期望 5 行输出；`bug07_stmt_brace.lfz` 修复后应 EXIT=0）；docs-writer / ai-dx-engineer（§9.4 换行版、无裸块语句）；app-dev 无影响。
- verifier 复验 spec-20260924-01 / bug-07 后更新 `docs/reports/P3-verification.md` §5 / §7。

## 关键经验（写给未来的自己）
- **样例必须可运行**：规范唯一综合样例若含违反冻结规则的字面（`;` / `.self`），会让 verifier 夹具与所有下游引用一起崩；冻结后仍须逐字符自检样例。
- **实现「简化」不能变成规范「放宽」**：parser 自述的「本批简化」若与冻结 A9 冲突，默认答案是**改实现**（最小、回归冻结设计），不是改 spec；改 spec 需评估连锁（EBNF/AST/二义）。
- **裁定要给出「规范原文位置 + 推论 + 可抄写文本」**：让 core-dev 能直接引用（§3.3 规则 2 / §5-A9 / EBNF `statement` 无 `block_stmt`）并把「哪些测试是有意识地改」列清。
