# language-architect — 工作状态
> 最后更新: 2026-09-24 00:30 by language-architect

## 当前状态
**✅ spec v1 已冻结（FROZEN，2026-09-23）；本轮完成 post-v1 变更（v1 补钉）：就 P3 独立验收（`docs/reports/P3-verification.md`）的 3 处「规范裁定」缺口（bug-20260924-06 / -08 / -09）逐条裁定并闭合。**
- 交付：`.opencode/team/DECISIONS.md` 追加 ADR「P3.11 验收 3 处规范裁定（let 重绑定 / .self / traceback 截断）」+ `docs/spec/{syntax.md, semantics.md, interface-contract.md}` 补充钉死（三件套均有改动）。
- 纪律：**先 ADR 后改文档**；三件套头部「冻结于 2026-09-23」保持；改动均为**补充钉死**，未推翻任何既有冻结规则；不新增错误类（仍 12 类 + 基类）、不引入 `E-xxx`、未写生产代码。

## 本轮裁定（缺口 → 裁定 → 落地）
| # | 缺陷单 | 缺口 | 裁定 | 变体 / 消息 | 代码变更 |
|---|---|---|---|---|---|
| 1 | bug-20260924-06 | `let` 重绑定无错误类（§4.5.2 称非法，§8.1 无对应） | 归 **`TypeError`**（运行期），**新增子消息变体**，**不新增错误类** | **`TypeMsg::ImmutableRebind { name }`**；消息 `不能重新赋值 let 变量 '{name}'；let 只锁重绑定，不锁内容`；`span` = 赋值目标首字符 | **需** `src/error.rs`(core-dev) + `src/evaluator.rs`(runtime-dev) |
| 2 | bug-20260924-08 | §9.4 样例 `r.self` 与 EBNF（`field="." IDENT` + `self` 是关键字 §2.6）冲突 | **改样例**（`r.self`→`r.me`），**不允许 `.self`**（parser 现行为正确） | — | **无**（仅改 spec） |
| 3 | bug-20260924-09 | `RecursionError` 全帧序列化 → stderr ~10000 帧 | **序列化保持完整、显示层折叠**：`T>40` → 首 10 + `  ... 省略 {T−40} 帧 ...` + 尾 30 | 常量 `TRACEBACK_HEAD=10` / `TRACEBACK_TAIL=30` / 阈值 40；`--json` **不折叠** | **需** `src/cli.rs`(tooling-dev)；**无需** core/runtime |

**裁定 1 理由（4 条）**：① 运行期行为（LFZ 名字运行时解析，parser 无作用域信息）；② 12 类中唯 `TypeError` 具"对某值/绑定执行了不允许操作"兜底语义，与 **JS `const` 重赋值 → `TypeError`** 先例一致；③ `NameError`（名未定义）/`ValueError`（转换）语义均不符；④ 复用现有类 + 新增子消息，守「不新增错误类」红线（同 `UnterminatedBlockComment` / `EmptyExtremum` / `BadRange`）。
**裁定 2 理由（5 条）**：最小改动不碰冻结 EBNF / `self` 保留字是刻意设计 / `s["self"]` 仍可用 / 样例目的是环安全（A6）与字段名无关 / 行业一致。
**裁定 3 理由（3 条）**：显示层不污染数据层（`LfzError`/`TracedRun` 完整可测）/ 头+尾折叠为通行做法 / 阈值 40 使多数浅栈输出逐字节不变。

## 边界钉死（本轮一并写死）
- **仅显式 `let` 声明的绑定不可重绑定**；`var` / 函数·λ 形参 / `for` 循环变量 / `fn` 名 / `struct` 模板名一律**可变**（与现状一致：`src/ast.rs:90`、`src/evaluator.rs:1110/:596/:612/:625`）。
- **关键字不可作裸字段名**（§2.6 新增规范条目）：`.字段` / `member` / `field_init` 均须 `IDENT`；需要保留字作键用 `s["self"]` / `{ "self": v }`。

## 待执行代码变更清单（交对应开发者；本轮未写代码）
| # | 文件 | 负责人 | 变更 |
|---|---|---|---|
| 1 | `src/error.rs` | core-dev | `TypeMsg` 增 `ImmutableRebind { name: String }` + `message()` 分支；顶部注释「6 条」→「7 条」；单测增断言 |
| 2 | `src/evaluator.rs`（+ 视需要 `env.rs`/`value.rs`） | runtime-dev | `assign_name`（:330）查绑定 `mutable`（`Env::local_mutable`）；`false` → `TypeError::ImmutableRebind`（`span=target.span`）；返回值需可区分「未找到/不可变/成功」；**捕获 cell 须携带可变性**；`a[i]=`/`s.k=` 路径不变 |
| 3 | `src/cli.rs` `render_error`（:171） | tooling-dev | 运行期 traceback 折叠：`T>40` → 首 10 + `  ... 省略 {T−40} 帧 ...` + 尾 30；`T≤40` 原样；`TracedRun`/`LfzError` 不变 |

## 产出与证据（可命令验证）
| 文件 | 行数（LF） | 说明 |
|---|---|---|
| `docs/spec/syntax.md` | 911 → **913** | §2.6（+关键字不可作裸字段名）、§9.4（样例 `r.self`→`r.me`、输出 `{me: <cycle>}`、字段名注） |
| `docs/spec/semantics.md` | 404 → **415** | §4.5.2（重绑定错误类 + 可变性边界）、§8.1（TypeError 触发行 + 细分表 6→7）、§8.2（traceback 折叠）、§8.4（JSON 不折叠） |
| `docs/spec/interface-contract.md` | 309 → **311** | §10.3（折叠指针 + 常量）、§10.6（parser：`.self` 为正确 SyntaxError，勿改）、§10.8（`TypeMsg::ImmutableRebind`） |
| `.opencode/team/DECISIONS.md` | 479 → **543** | 追加 ADR「P3.11 验收 3 处规范裁定」（标题行 L483） |

- 三件套 UTF-8 无 BOM；字节级：`syntax.md` bytes=61290 LF=913 CR=0、`semantics.md` bytes=33931 LF=415 CR=0、`interface-contract.md` bytes=30190 LF=311 CR=0，末字节均 = LF(10)。
- 错误类计数保持：`共 12 类 + 1 基类`、`运行期错误 = 10 类`（未变）；`E-xxx` 仅存于既有 §13 附录（14 处命中行，**本轮未新增**）。
- 新增串命中：`ImmutableRebind`、`不能重新赋值 let 变量`、`TRACEBACK_HEAD`、`... 省略`、`r.me`、`{me: <cycle>}`；旧串 `r.self` 已清零。

## 进行中
- （无；待 team-lead 转派 core-dev（#1）/ runtime-dev（#2）/ tooling-dev（#3）落地代码变更清单）

## 阻塞 / 需要支持
- 无。

## 下一步计划
- team-lead 派发：core-dev → `src/error.rs`；runtime-dev → `src/evaluator.rs`（`mutable` 强制）；tooling-dev → `src/cli.rs`（traceback 折叠）。
- 通知下游：test-engineer（负例 `let a=1; a=2` → `TypeError` + 深递归折叠快照）、docs-writer / ai-dx-engineer（`let` 不可重绑定、`self` 不可作字段名、traceback 折叠）。
- verifier 复验 bug-06/-08/-09 后更新 `docs/reports/P3-verification.md` §5 回归表。

## 关键经验（写给未来的自己）
- **动态语言的"静态外观"规则必落运行期**：`let` 不可变看似编译期，但 LFZ 名字运行时解析 → 只能归运行期类；并**优先复用现有错误类 + 新增子消息**（对标 JS `const` 重赋值走 `TypeError`）。
- **样例与文法冲突以文法为准**：§9.4 是样例瑕疵（非设计问题），改样例（最小改动）优于特例化文法；"关键字当名字"是高频手误点，需在关键字节补规范条目。
- **"全部序列化"要分层看**：数据层（`LfzError`/`TracedRun`）保持完整、显示层（CLI）折叠——既满足 §10.3，又解决实用性；阈值设大（40）以保浅栈快照逐字节不变。
- **冻结后每处改动都要能被"先 ADR 后改"顺序核验**：本轮先追加 ADR 标题行，再动三件套。
