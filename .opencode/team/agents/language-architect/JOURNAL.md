# language-architect — 工作日志
> 只追加，最新条目在最上方。
## [2026-09-24 01:00] 未闭合块注释（/* 至 EOF）裁定（1 处契约缺口闭合，v1 补钉）
- 来源: team-lead 下达「就 core-dev 上报的 1 处契约缺口（未闭合块注释 `/*` 至 EOF）给规范裁定」任务（轻量启动；依据 `syntax.md` §2.4/§2.5、`semantics.md` §8.1、`interface-contract.md` §10.6）
- 完成:
  - **裁定**：未闭合块注释 `/*`（至 EOF 仍无 `*/`）→ **`SyntaxError`**；**否决** core-dev 临时口径「消费至 EOF、等价空白、不报错」。**新增 `SyntaxMsg::UnterminatedBlockComment`**（无字段），消息 `块注释在此处未闭合（缺少 '*/'）`；`span` = `/*` 的 `/`。
  - **理由 4 条**：① 与 §2.8「字符串遇 EOF → SyntaxError」同源（CODE 模式同类未闭合词法区）；② LFZ「无魔法/结构化报错」红线，静默吞 EOF 掩盖漏写 `*/`；③ 最接近的 `UnterminatedString` 消息对注释**语义错误**，不可复用；④ 行业一致（C/C++/Rust/Java/Go/JS 均报错）。
  - **规范落地**：`syntax.md` §2.4（+规范性条目）+ §2.5（空白收窄为"闭合块注释"）；`semantics.md` §8.1（触发列表 + 细分表 +1 行，16→17）；`interface-contract.md` §10.6 + §10.8（变体规格）。
  - **纪律**：先追加 ADR，后改 `docs/spec/`；三件套头部「冻结于 2026-09-23」不变；不新增错误类（仍 12 类 + 基类）、不引入 `E-xxx`。
- 产出: `.opencode/team/DECISIONS.md` 追加 ADR「未闭合块注释（/* 至 EOF）裁定」（标题行 L290）；`docs/spec/syntax.md`（910→**911** 行）、`semantics.md`（403→**404** 行）、`interface-contract.md`（308→**309** 行）
- 证据: 字节级 LF=911/404/309、CR=**0**、BOM=False、末字节=LF(10)；错误类计数 `共 12 类 + 1 基类` / `运行期 10 类` 不变；`E-xxx` 仍 11 处（仅 §13）；新增串 `UnterminatedBlockComment` / `块注释在此处未闭合` / `未闭合块注释` 均命中
- 决策: ADR「未闭合块注释（/* 至 EOF）裁定」（缺口 → 裁定 → 落地 + 待执行代码变更清单 + 下游影响）
- 待执行代码变更（本轮未写代码）: `src/error.rs`（core-dev：加 `SyntaxMsg::UnterminatedBlockComment` + message + 更新「16 条」注释/测试）；`src/lexer.rs`（core-dev：未闭合块注释改报 SyntaxError，替代静默吞 EOF）
- 下一步: 通知 team-lead 转 core-dev 落地 1/2；test-engineer 可写负例断言；docs/ai-dx 补「块注释必须闭合」
- 阻塞: 无

## [2026-09-24 00:20] P3.9a 契约缺口闭合（6 项，v1 补钉）
- 来源: team-lead 下达「闭合 runtime-dev 上报的 6 处契约缺口」任务（先读 DECISIONS 末条 ADR + `semantics.md` §8.1 + `interface-contract.md` §8.1/§10.7）
- 完成:
  - **逐项裁定**（最小改动优先；6 项中 4 项零代码变更）：
    1. `min`/`max`/`minBy`/`maxBy` 空 → `ValueError`：**新增 `ValueMsg::EmptyExtremum { func }`**，消息 `空数组没有极值（{func}）`（现有 `Convert`/`BadFormatSpec` 两模板不适用）。
    2. `randInt(lo,hi)` 且 `lo >= hi` → `ValueError`：**新增 `ValueMsg::BadRange { lo, hi }`**，消息 `区间非法：{lo} >= {hi}`。
    3. `pop([])` → `IndexError`：**复用现有 `Index{idx,len}`、不新增变体**，钉死 `idx = -1`、`len = 0`（消息 `下标 -1 越界（长度 0）`），并补通用 idx/len 取值规则。
    4. `floor`/`ceil`/`round` 的 `NaN`/`±Inf`/超界：**复用 `int(float)` 口径**（`NaN`→`ValueError`、`±Inf`/取整后超 i64→`OverflowError`），写成 §4.5.7 规范（新增一条）。
    5. `del(k,s)` 对方法字段：裁定 **`del` 属数据面、仅作用于数据字段**（与 `keys`/`has`/`len` 同集合），方法字段键 → `FieldError`（不变量 `del(k,s) 成功 ⟺ has(k,s)==true`）；**runtime-dev 需改 `del` 判定**（`raw_fields`「存在即删」→ 数据字段集合）。
    6. `insert` 负索引：裁定 **不支持负索引**，合法域 `i ∈ [0, len]`，`i < 0` 或 `i > len` → `Index{idx:i,len}`；与 `removeAt`/`swap`（支持负索引）显式区分。
  - **纪律**：先追加 ADR，后改 `docs/spec/`；三件套头部「冻结于 2026-09-23」不变，改动均为**补充钉死**；不新增错误类（仍 12 类 + 基类）、不引入 `E-xxx`。
  - 只改 `semantics.md`（§4.5.7 / §4.5.9 / §8.1）与 `interface-contract.md`（§8.1 / §10.7 / §10.8）；`syntax.md` **无需改动**。
- 产出: `.opencode/team/DECISIONS.md` 追加 ADR「P3.9a 契约缺口闭合（6 项）」；`docs/spec/semantics.md`（382→**403 行**）、`docs/spec/interface-contract.md`（298→**308 行**）、`docs/spec/syntax.md` 不变（910 行）。三者 UTF-8 无 BOM。
- 证据: 行数 910/403/308、BOM=False；错误类计数 `共 12 类` 与 `运行期…10 类` 保持；`E-xxx` 仅存于既有 §13 附录（11 处，未新增）；新增串 `EmptyExtremum`/`BadRange`/`空数组没有极值`/`区间非法`/`不支持负索引` 均命中；ADR 标题行位于 `DECISIONS.md` L223。
- 决策: ADR「P3.9a 契约缺口闭合（6 项）」（逐项裁定 + 待执行代码变更清单 + 下游影响）
- 待执行代码变更（本轮未写代码）: `src/error.rs` 加 2 变体（`EmptyExtremum`/`BadRange`）；`src/builtins.rs` 改用新变体 + `del` 改数据面判定 + 确认 `pop`/`insert`/`floor` 口径。
- 下一步: 通知 team-lead 转 core-dev（`src/error.rs`）+ runtime-dev（`src/builtins.rs`）落地；test-engineer 可解除「暂不 snapshot 缺口 1–5」禁令。
- 阻塞: 无

## [2026-09-23 22:00] spec v1 冻结（3 处补钉闭合 + docs/spec 三件套 + ADR D-016）
- 来源: team-lead 下达「收尾定稿」任务（core-dev 可解析性 PASS；runtime-dev 可求值性 CONCERNS 仅 3 项且非架构级，要求拆分 spec 时一并闭合）
- 完成:
  - **补钉 1（§10.7 数值内置形参加宽）**：`floor`/`ceil`/`round`/`sqrt`/`pow` 的 `int` 实参按 §1 / §4.5.7「全局唯一隐式转换 `int→float`」加宽为 `float`；`abs` **同型不加宽**；二者差异一句话钉死；`log`/`exp` 等 v1 不提供故无加宽问题。同步 `abs` 行（同型）与 `int` 行（NaN/Inf 边界）。
  - **补钉 2（§8.2 计数订正）**：运行期错误旧计数（少算 1 类）订正为 **其余 10 类**；§12 的 §5 条目数 26 → **27**（A1–A27）；§8.1 计数自查一致（12 类 + 基类、`--json` 12、运行期 10）。
  - **补钉 3（NaN/Inf → `int` 极窄边界）**：§4.5.7 + §10.7 `int` 行 + §8.1 `ValueError`/`OverflowError` 行：`int(NaN)` → `ValueError`、`int(±Inf)` → `OverflowError`、有限浮点向零截断且超 i64 → `OverflowError`。
  - **拆分冻结**：`docs/spec/syntax.md`（**910 行**）/ `semantics.md`（**382 行**）/ `interface-contract.md`（**298 行**），共 **1590 行**；三文件均以「本文件是 LFZ v1 的唯一事实源（冻结于 2026-09-23）。变更须先 ADR，后改文档。」开头；保留原 DRAFT 章节编号锚点；相对链接互指（semantics ↔ syntax ↔ interface-contract）；信息只增不减。
  - **DRAFT 就地修订**：3 处补钉 + 文首「冻结前补钉」记录；§4.5.7 / §10.7 / §8.1 / §8.2 / §12 更新。
- 产出: `docs/spec/{syntax,semantics,interface-contract}.md`（UTF-8 无 BOM，首 3 字节 `23 20 4C`；行数 910/382/298）；`.opencode/team/DRAFT-LFZ-v0.5.md`（1522 行，已含 3 补钉）；`.opencode/team/DECISIONS.md` 追加 **D-016**
- 证据: 三件套关键词合计 `#42`=101、`ScopeDebug`=12、`RecursionError`=15、`int(NaN)`=2、`data-last`=6；`9 类` 在 `docs/spec`、DRAFT-v0.5、DECISIONS 中检索均为 0（仅存于历史 v0.3/v0.4 草案）
- 决策: ADR **D-016「spec v1 冻结」**（冻结范围 + 3 补钉 + 4 项保留默认 + 「先 ADR 后改文档」纪律）
- 下一步: 通知 team-lead 派发 P3（core-dev/runtime-dev 依本三件套实现）；docs/ai-dx/test/app 只读引用
- 阻塞: 无

## [2026-09-23 21:30] DRAFT-LFZ-v0.5 定稿（A1–A7 用户拍板 + B1–B13 / M1–M6 / N1–N2 全部落地）
- 来源: team-lead 下达的 v0.5 任务（v0.4 冻结门禁评审 core-dev/runtime-dev 均判 CONCERNS；用户 7 条语义拍板）
- 完成:
  - **A1–A7**：§4.5.2 引用语义；§4.5.3 按 cell 捕获；§2.3/§4.2 `/` 真除法 + `div` 向下取整 + `%` Python 取模；§4.5.10/§8.1 `check` 非致命（stderr + false + 继续）；§3.7/§4.5.9 方法不入数据面；§3.7/§4.5.9 环安全（`==` 访问对集合、display `<cycle>`）；§8.1/§10.4 新增 `RecursionError`。
  - **B1–B13**：新增 **§4.5 求值语义规范**（11 个子节：求值顺序/快照/键序/递归深度/IEEE/int↔float 精确算法/struct 平拷贝/相等显示/check/字符串不可变）；**§10.7 内置函数表**（data-last 签名 + 返回类型）；**per-scope `ScopeDebug`**（§10.5）；统一 `Span{line,col}` + `VmFrame`/`TraceFrame`（§10.2/§10.3）；`Result<T,Box<LfzError>>` + cold 构造 + `i64::MIN`（§10.8）。
  - **M1–M6/N1–N2**：§2.8（M1 INTERP 禁裸换行、M5 未列举转义、M6 `FORMAT_SPEC` token）；§7（M2 去 `[preamble]`、M3 `=> {` 恒为块）；§3.3（M3）；§4.3 规则 6（M4 `_` 不穿 λ）；§10.3（N1）；§6-B12（N2）。
  - §8.1 错误类 **12 类 + 基类**；§8.3 增示例 5（check stderr）与 `--json` 集合；§9.4 新语义演示；§12.1/§15.1/§15.2 核对；§14 收敛为仅 4 条遗留默认；文首标「**v0.5 = 定稿（待复审冻结）**」。
- 产出: `.opencode/team/DRAFT-LFZ-v0.5.md`（**1504 行**，UTF-8 无 BOM；字节级验证：首 3 字节 `23 20 4C`=`# L`、CR=0、无 BOM、代码围栏 60 行平衡）
- 决策: 追加 ADR **D-015**（DECISIONS.md）
- 下一步: team-lead 组织 core-dev/runtime-dev 对 v0.5 **复审**；通过后拆 `docs/spec/` 三件套并追加「spec v1 冻结」ADR
- 阻塞: 无

## [2026-09-23 19:05] DRAFT-LFZ-v0.4 定稿候选（用户拍板 5 条落地；`#42` 前导按扩展名）
- 来源: team-lead 下达的 v0.4 任务（用户对本轮 5 项开放问题拍板）
- 完成:
  - **拍板 1 前导按扩展名**：新增 §2.2.0 `ext(path)` 判定（按 `/` 与 `\` 取最后分量，最后一个 `.` 之后为扩展名，与 `"lfz"` 按 **ASCII 大小写不敏感**比较；空/无扩展名/`.gitignore`/`foo.lfz.bak` 一律豁免）；§2.2.2 加载器 `is_lfz` 分流 + `line_base`；**非 `.lfz` 文件即使被 `lfz run` 执行也不校验前导**；§1、§2.1、§2.3、§7、B1/B9/B10/B11/B12、§10.1、§11.1/11.2/11.5、§12 同步。
  - **拍板 2** `CosmosAnswerError` 不显示第 1 行原文 → 保持（标"已定"）；**拍板 3** 程序体可空 → 保持（标"已定"）；**拍板 5** `#42` 后必须紧跟行终止符（3 字节 `#42` → `CosmosAnswerError`）→ 保持（B2 标"已定"）。
  - **拍板 4 无 hint 行**：§8.1「hint 行策略」收紧为"**v1 明确不输出**"，可选 hint 列入 **v1.1 backlog**。
  - 文首加「**v0.4 = 定稿候选，可冻结**」标记 + **冻结前提条件**（5 条）与 `docs/spec/` 三件套拆分映射。
  - §0 新增「v0.3 → v0.4 差异摘要」（F–K 六行）；§14 由 5 项开放收敛为**仅 4 条遗留默认**（`;;` 作用域/通道、多行续行、单 `;`），并注明"新增未决项：无"。
- 产出: `.opencode/team/DRAFT-LFZ-v0.4.md`（**1131 行**，UTF-8 无 BOM；字节级验证：首 3 字节 `23 20 4C`=`# L`、无 BOM、LF=1131、CR=0）
- 决策: 追加 ADR **D-014**（**编号更正**：任务书原指定 D-013，但 D-013 已被 [2026-09-23 18:56] 的 v0.3 草案占用，依"ADR 只追加、不修改历史"顺延为 D-014，正文已注明）
- 下一步: team-lead/用户无异议 + core-dev/runtime-dev 可实现性评审通过 → 拆 `docs/spec/` 三件套并追加「spec v1 冻结」ADR
- 阻塞: 无

## [2026-09-23 18:56] DRAFT-LFZ-v0.3 修订草案（`#42` 前导 + Python 风格错误模型 + 可移植性）
- 来源: team-lead 下达的 v0.3 任务（用户三项新指令：文件前导 `#42` / Python 风格报错 / 可移植性）
- 完成:
  - **A 文件前导 `#42`**：仅文件模式要求、第一行恰好 `#42`+行终止符、无任何变体；`#` 为"文件开头专用符号"（与注释无关）；失败抛 `CosmosAnswerError`（消息固定「你忘记了宇宙的答案」、不显示编号/hint）；REPL/stdin/`-e` 豁免。给出加载器逐字节算法（先编码校验、后前导校验、跳 BOM、归一化行终止符）。
  - **B Python 风格错误模型**：11 类错误（`CosmosAnswerError`/`SyntaxError`/`NameError`/`TypeError`/`IndexError`/`FieldError`/`ZeroDivisionError`/`OverflowError`/`ValueError`/`IOError`/`AssertionError` + 基类 `.LfzError`）；中文消息模板；`File "<path>", line N[, in func]` + 源码行 + 插入符；跨函数 traceback（最外层帧在前）；**取消 E-xxx 编号**（`--json` 用类名）；输出示例 4 个（语法错 / 运行时错含栈 / CosmosAnswerError / `--json`）。
  - **C 可移植性**：行终止符 `\n/\r\n/\r` 归一化；BOM 静默跳过；UTF-8 强制（非 UTF-8 → `SyntaxError`）。
  - **新增 §6「新增规则的歧义/边界审计」B1–B12**（规则+正例+反例）：空文件 / 仅 `#42` 无换行 / 前导前空行空格 BOM / `# 42`·`#42abc`·`#43`·尾随空格 / `#` 在程序中间（字符串内合法）/ CRLF·LF·CR / 非 UTF-8 / REPL·stdin·`-e` 豁免 / CosmosAnswerError 位置 / `init/fmt/test/check` 统一查头 / 前导字节精确性。
  - EBNF 增 `program = [preamble] …`；§8 错误模型全量改写；§9 三个样例全部加 `#42` 首行；§10 Rust 影响（loader / `Span` / `CallFrame` / `LfzError` 枚举 / `DebugSym` 职责不变）；§11 下游硬性要求（**AI 指南写死 `#42` 首行 + 错误类清单**；**test-runner 契约要求测试文件带头**）；§12 与 D-007/D-011/用户指令一致性核对；§13 旧码→新类映射表（仅追溯）。
- 产出: `.opencode/team/DRAFT-LFZ-v0.3.md`（1059 行，UTF-8 无 BOM；字节级验证：前 3 字节 `23 20 4C`=`# L`、无 BOM、LF=1059、CR=0）
- 决策: 追加 ADR `D-013`（DECISIONS.md）
- 下一步: 等用户确认 §14 的 5 项开放问题 → 拆 `docs/spec/` 三件套并冻结 v1
- 阻塞: 无

## [2026-09-23] DRAFT-LFZ-v0.2 修订草案（用户新指令 + 歧义审计 + core-dev 对账）
- 来源: team-lead 下达的 REVISION 任务（用户新指令覆盖 v1 的 Q1 及相关选择）
- 完成:
  - 落地指令：块用 C 风格 `{}`；语句换行终结（无 `;` 终结符）；`;` 专用 → `;;` dump 变量；`${}` 插值保留并说明与块花括号无冲突。
  - 输出 **26 条歧义审计**（规则 + 正反例）：换行模式栈、`{` 双角色、条件位禁裸字面量、一元负号、管道换行、`;;` 词法/语义/作用域、插值 lexer 模式栈、分隔符统一等。
  - 修正 **v1 真 bug**：`|>` 优先级表和 EBNF 矛盾 → 采用 core-dev 的修正层序。
  - 对 core-dev 10 条独立审计逐条对账（同意 8 / 部分同意 1 / …），写入 §9。
  - 更新 EBNF、2 个完整样例（含 `;;` 与多行管道）、错误码（E-SYN-002/003/005/006）。
- 产出: `.opencode/team/DRAFT-LFZ-v0.2.md`（833 行，UTF-8 无 BOM；字节级验证首字节 `# L`、无 BOM、LF=833）
- 决策: 见本文件同批追加的 ADR（DECISIONS.md）
- 下一步: 等用户确认 §11 的 4 项开放问题 → 拆 `docs/spec/` 三件套并冻结 v1
- 阻塞: 无

## [2026-09-22] 角色初始化
- 来源: 团队初始化（系统构建）
- 完成: 角色定义与活文档建立
- 产出: `.opencode/agents/language-architect.md`
- 下一步: 等待 team-lead 调度
