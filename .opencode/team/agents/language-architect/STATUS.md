# language-architect — 工作状态
> 最后更新: 2026-09-23 22:00 by language-architect

## 当前状态
**✅ spec v1 已冻结（FROZEN，2026-09-23）**，交付 `.opencode/team/DRAFT-LFZ-v0.5.md`（**1522 行**，含 3 处冻结前补钉）+ `docs/spec/` 三件套 + ADR **D-016**。
- 冻结门禁复评：core-dev（可解析性）**PASS**；runtime-dev（可求值性）**CONCERNS 仅 3 项、非架构级**（明确不阻塞实现启动）。本轮把这 3 项闭合后冻结。

## 本轮产出（可命令验证）
| 文件 | 行数 | 内容 |
|---|---|---|
| `docs/spec/syntax.md` | 910 | 词法 / `#42` 前导 / 可移植性 / EBNF / 语句·表达式·块 / 优先级 / 歧义审计 A1–A27 / 边界审计 B1–B12 / 样例 §9 / §12 §14 §15 |
| `docs/spec/semantics.md` | 382 | 值模型总览 §4.5.0 / `;;` §3.6 / 显示形式 §3.7 / 类型化运算符 §4.2 / 求值语义 §4.5.1–§4.5.11 / 错误语义 §8 |
| `docs/spec/interface-contract.md` | 298 | §8.1 错误类清单（实现映射）/ §10（`Span`、`VmFrame`/`TraceFrame`、`LfzError`、`ScopeDebug`、loader 双入口 + `ext(path)`、§10.7 内置表 data-last）/ §11 下游要求 / §13 旧码映射 |

- 三文件均以「本文件是 LFZ v1 的唯一事实源（冻结于 2026-09-23）。变更须先 ADR，后改文档。」开头；UTF-8 无 BOM；保留原章节编号；相对链接互指；信息只增不减。
- 关键词证据（三件套合计）：`#42`=101、`ScopeDebug`=12、`RecursionError`=15、`int(NaN)`=2、`data-last`=6；`9 类` 在 docs/spec 为 0。

## 本轮 3 处冻结前补钉（已就地写入 DRAFT 后拆分）
1. **§10.7 数值内置形参加宽**：`floor`/`ceil`/`round`/`sqrt`/`pow` 的 `int` 实参按 §1/§4.5.7 **加宽为 `float`**；`abs` **同型不加宽**（一句话钉死差异）；`log`/`exp` 等 v1 不提供。
2. **§8.2 计数订正**：运行期错误 → **其余 10 类**（含 `RecursionError`）；§12 §5 26 → **27**；§8.1 计数自查一致。
3. **NaN/Inf → `int` 极窄边界**：`int(NaN)` → `ValueError`；`int(±Inf)` → `OverflowError`；有限浮点向零截断超 i64 → `OverflowError`（§4.5.7 + §10.7 + §8.1）。

## 冻结范围与纪律
- 冻结三文件 = LFZ v1 **唯一事实源**；**此后任何语法/语义/契约变更须先写 ADR，再由 language-architect 改 `docs/spec`**；其他角色只读引用。
- 4 项保留默认（README 已记录于 D-016）：`;;` 作用域 = 完整可见链 / `;;` 通道 = stdout / 多行续行 = 仅括号内 / 单 `;` = 永远 `SyntaxError`。

## 进行中
- （无；spec v1 已交付 team-lead）

## 阻塞 / 需要支持
- 无。

## 下一步计划
- 通知 team-lead 派发 **P3**（core-dev/runtime-dev 严格按 `docs/spec/interface-contract.md` + `semantics.md` 实现）。
- docs-writer / ai-dx-engineer / test-engineer / app-dev 只读引用三件套（尤其 `#42` 首行、错误类清单、`check` 非致命、内置 data-last 签名表）。
- 若 runtime-dev 的 3 项 CONCERNS 在实现中暴露残余歧义 → 走 post-v1 变更：**先 ADR，后改 spec**，并通知受影响角色。

## 关键经验（写给未来的自己）
- **冻结前补钉也走"三处齐改"**：`int(NaN)` 边界在 §4.5.7 / §10.7 `int` 行 / §8.1 `ValueError`·`OverflowError` 行**各写一遍**，避免只在语义文档写而死角。
- **计数是易腐面**：错误类共 12 + 基类、运行期 10、`--json` 12、§5 27 条、§6 12 条——**每次增删条目必须全文 grep 计数**；本次发现 §12「26 条」是 v0.5 遗留 stale。
- **拆分三件套的信息完备性**：`syntax` 管形式、`semantics` 管意义、`interface-contract` 管实现契约；跨文件引用一律**保留原章节编号 + 相对链接**，并明确"不在本文件"清单，防读者漏读。
- **验收用"关键词存在 + 禁用串不存在"双向 grep**：仅检查存在会漏掉"旧计数串残留"这类缺陷。
- **`9 类` 事件**：连"订正说明"里引用旧计数都会触发禁用串检查——改写措辞为"少算 1 类"，不复述禁用词。
- v0.2/v0.4/v0.5 教训仍在：优先级表由文法层序机械导出；换行是否有效只能由 parser 上下文栈决定；凡"按直觉写必翻车"处必须给算法级规则。
