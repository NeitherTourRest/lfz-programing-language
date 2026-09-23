# language-architect — 工作状态
> 最后更新: 2026-09-24 00:20 by language-architect

## 当前状态
**✅ spec v1 已冻结（FROZEN，2026-09-23）；本轮完成 post-v1 变更（v1 补钉）：闭合 runtime-dev 上报的 6 处契约缺口。**
- 交付：`.opencode/team/DECISIONS.md` 追加 ADR「P3.9a 契约缺口闭合（6 项）」+ `docs/spec/{semantics.md, interface-contract.md}` 补充钉死（`syntax.md` 不变）。
- 纪律：**先 ADR 后改文档**；三件套头部「冻结于 2026-09-23」保持；改动均为**补充钉死**，未推翻任何既有冻结规则；不新增错误类（仍 12 类 + 1 基类）、不引入 `E-xxx`。

## 本轮 6 项裁定（速查表）
| # | 缺口 | 裁定 | 落地方式 | 代码变更 |
|---|---|---|---|---|
| 1 | `min`/`max`/`minBy`/`maxBy` 空 | `ValueError`，**新增 `ValueMsg::EmptyExtremum { func }`**，消息 `空数组没有极值（{func}）` | `semantics.md` §8.1 + `interface-contract.md` §8.1/§10.7/§10.8 | **需** `src/error.rs` |
| 2 | `randInt(lo,hi)` 且 `lo >= hi` | `ValueError`，**新增 `ValueMsg::BadRange { lo, hi }`**，消息 `区间非法：{lo} >= {hi}` | 同上 | **需** `src/error.rs` |
| 3 | `pop([])` 的 `idx`/`len` | 复用现有 `Index{idx,len}`，钉死 `idx=-1, len=0` + 通用规则 | `semantics.md` §8.1 + `interface-contract.md` §10.7 | 无（现值一致） |
| 4 | `floor`/`ceil`/`round` 的 `NaN`/`±Inf`/超界 | **复用 `int(float)` 口径**（`NaN`→`ValueError`、`±Inf`/超界→`OverflowError`），写成 §4.5.7 规范 | `semantics.md` §4.5.7 + `interface-contract.md` §10.7 | 无（现值一致） |
| 5 | `del(k,s)` 对方法字段 | **`del` 属数据面、仅作用于数据字段**；方法字段键 → `FieldError`（`del 成功 ⟺ has==true`） | `semantics.md` §4.5.9/§8.1 + `interface-contract.md` §10.7 | **需** `src/builtins.rs`（改判定） |
| 6 | `insert` 负索引 | **不支持负索引**；`i ∈ [0,len]`，`i<0`/`i>len` → `Index{idx:i,len}` | `semantics.md` §8.1 + `interface-contract.md` §10.7 | 无（现值一致） |

## 待执行代码变更清单（交 core-dev / runtime-dev；本轮未写代码）
| # | 文件 | 负责人 | 变更 |
|---|---|---|---|
| 1 | `src/error.rs` | core-dev | 新增 `ValueMsg::EmptyExtremum { func: String }` + `message()`（`空数组没有极值（{func}）`） |
| 2 | `src/error.rs` | core-dev | 新增 `ValueMsg::BadRange { lo: i64, hi: i64 }` + `message()`（`区间非法：{lo} >= {hi}`） |
| 3 | `src/builtins.rs` | runtime-dev | 1/2 改用新变体（`min`/`max`/`minBy`/`maxBy` 空、`randInt` 非法区间） |
| 4 | `src/builtins.rs` | runtime-dev | `del` 判定由 `raw_fields`（存在即删）改为**数据字段集合**（复用 `has`/`keys` 谓词） |
| 5 | `src/builtins.rs` | runtime-dev | 确认 `pop` → `Index{idx:-1,len:0}`；`insert` 越界/负索引 → `Index{idx:i,len}`；`floor`/`ceil`/`round` 复用 `int(float)` 口径 |

## 产出与证据（可命令验证）
| 文件 | 行数 | 说明 |
|---|---|---|
| `docs/spec/syntax.md` | 910（不变） | 6 项均非形式/文法问题 |
| `docs/spec/semantics.md` | 382 → **403** | §4.5.7（+floor/ceil/round 边界）、§4.5.9（+del 数据面）、§8.1（ValueError 新子场景表 + IndexError idx/len 表 + FieldError 行） |
| `docs/spec/interface-contract.md` | 298 → **308** | §8.1（ValueError 模板）、§10.7（内置边界补钉块）、§10.8（ValueMsg 变体） |
| `.opencode/team/DECISIONS.md` | +40 行 | 追加 ADR「P3.9a 契约缺口闭合（6 项）」（L223 起） |

- 三者 UTF-8 无 BOM；错误类计数保持 `共 12 类` + `运行期…10 类`；`E-xxx` 仅存于既有 §13 附录（11 处，本轮未新增）。
- 新增串命中：`EmptyExtremum`、`BadRange`、`空数组没有极值`、`区间非法`、`不支持负索引`。

## 进行中
- （无；待 team-lead 转派 core-dev / runtime-dev 落地代码变更清单）

## 阻塞 / 需要支持
- 无。

## 下一步计划
- team-lead 派发：**core-dev**（`src/error.rs` 加 2 变体）→ **runtime-dev**（`src/builtins.rs` 改用变体 + `del` 改数据面判定）。
- 通知受影响下游：**test-engineer** 解除「暂不 snapshot 缺口 1–5 消息文本」禁令；**docs-writer / ai-dx-engineer** 补 4 点（min/max 空、randInt 区间非法、insert 无负索引、del 仅数据字段）。
- 若落地中发现新歧义 → 继续走 post-v1 变更（先 ADR 后改 spec）。

## 关键经验（写给未来的自己）
- **冻结后变更的最小改动判据**：先问「能否用现有枚举字段表达」——本轮 6 项里 4 项（3/4/5/6）零新增变体，仅 1/2 确需新增；能复用（`Index{idx,len}` / `ValueMsg::Convert`）就绝不新增。
- **消息模板要「语义正确」而非「能塞进去」**：runtime-dev 的 `Convert{src:"array",dst:"min"}` 能跑但语义错（不是转换失败）——此类"能表达但语义不符"必须判为**需新增变体**，不能用"最小改动"为由迁就。
- **A5 数据面是单一口径**：凡 struct-as-dictionary 操作（keys/values/entries/display/==/has/len/**del**）必须共用「数据字段」集合，否则出现 `has(k)=false` 却 `del(k)` 成功的自相矛盾；扩展 A5 时以「同集合不变量」背书。
- **idx/len 不能只钉 pop**：一次性把"通用取值规则（实参原值 / pop 取 -1 / len=越界时容器长度）"补进规范，可避免 removeAt/swap/insert 逐个再报缺口。
- **计数是易腐面**：本轮只加"子场景"不改错误类，故 `共 12 类`/`运行期 10` 保持；增删条目后仍须全文 grep 复核计数。
- **收工用双向 grep**：既查新增串存在（`EmptyExtremum` 等），也查禁用串未新增（`E-[A-Z]{2,3}-[0-9]` 应仅 11 处、全在 §13）。
