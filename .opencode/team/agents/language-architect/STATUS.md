# language-architect — 工作状态
> 最后更新: 2026-09-27 22:50 by language-architect

## 当前状态
**✅ 已完成 T11-③ 语言侧任务：v1.1 七项补强的 ADR + `docs/spec/` 规范文本落地（先 ADR、后改规范）。** 本轮**只改 `docs/spec/**` + `.opencode/team/DECISIONS.md`**；`src/**` 零改动（实现随后由 core-dev / runtime-dev 做）；spec v1 冻结不变，属 **v1.1 补钉**。**语法零改动**（`syntax.md` 字节数不变 70043）。

## 本轮交付（7 项 1 表）
| # | 特性 | 签名 / 关键裁定 | 错误类 + 精确消息 | 复杂度 | 需追认 |
|---|---|---|---|---|---|
| 1 | `string` 取下标 → `TypeError` | **不加 `s[i]`**，固化既有行为；`[]` 仅 array/struct；读写同源 | `TypeError: 运算符 '[]' 不支持 string 与 array / struct` | O(1) 报错；正解 `split("",s)` O(n)（不引入 O(n²)，反而消除） | 否 |
| 2 | `range(lo,hi)` 半开 | `[lo,hi)`；`hi<=lo→[]`；1/2 参；**3 参 OUT** | 非 int→`TypeError`（`运算符 'range' 不支持 {t} 与 int`）；容量→`OverflowError` | O(hi−lo)，同阶 | 否（提示：旧报错输入变合法） |
| 3 | 字符串方法族 | `indexOf/endsWith/padEnd/padStart/substring`；data-last、返回新值；`substring` 与 `slice` 同夹取 | 非 string/int→`TypeError`；空 fill→`ValueError: 填充串不能为空` | `indexOf` **最坏 O(n·m) 须标注**；其余 O(m)/O(width)/O(子串长) | 否 |
| 4 | 文件 IO | `readFile→string` / `writeFile`·`appendFile→nil`；UTF-8、相对 CWD、不沙箱、不隐式转换 | `IOError`：`无法读取：{path}` / `无法读取：{path}（不是合法的 UTF-8 编码）` / `无法写入：{path}` / `无法追加：{path}` | O(文件大小) | **是**（新外部副作用面） |
| 5 | `ord`/`chr` | `ord(c)→int`（恰 1 标量）；`chr(n)→string`（合法码点、非代理区） | 非 string/int→`TypeError`；`ValueError: ord 的参数必须是单个字符（Unicode 标量数 {n}）` / `chr 的参数不是合法的 Unicode 码点：{n}` | O(1) | 否 |
| 6 | math `sin/cos/log/exp` | `f→float`；int→float 加宽 | IEEE：`log(0)→-Inf`、`log(负)→NaN`、`exp` 溢出→`+Inf`、`sin/cos(±Inf\|NaN)→NaN`（**非报错**） | O(1) | 否（提示：语义口径已定死） |
| 7 | `contains(v,xs)` | 仅 array；用 `==`（深结构、环安全）；struct 用 `has` | 非 array→`TypeError`（`运算符 'contains' 不支持 {t} 与 array`） | **O(n) 须标注** | 否 |

## 规范落点（逐处）
- `docs/spec/semantics.md`：§4.2 新增 `[]` 索引补钉一条；§4.5.6 扩 NaN/Inf 产地；§4.5.7 扩加宽清单；§8.1 `TypeError`/`ValueError`/`IOError` 三触发行 + `ValueError` 细分表 3 行。`(+10/-6)`
- `docs/spec/interface-contract.md`：§8.1 `ValueError`/`IOError` 消息；§10.7 `range` 拆行 + 字符串 5 行 + math 4 行 + `ord`/`chr` 2 行 + `contains` 1 行 + **新增「文件 IO」段** + 更新 math 注 / 加宽清单 + **新增「v1.1 补钉内置的边界与复杂度」段**；§10.8 `ValueMsg` 3 变体。`(+48/-6)`
- `docs/spec/syntax.md`：**零改动**（`git diff --stat` 为空 ⇒ **无文法改动**）。
- `.opencode/team/DECISIONS.md`：+1 条 ADR（标题 `[2026-09-27 22:50]`）。`(+113/-0)`

## 兼容性核对（全绿）
A1 ✅ / A2 ✅ / A3 ✅（`log(0)` 明确非除零）/ A4 ✅ / A5 ✅ / A6 ✅（`contains` 继承环安全）/ A7 ✅ / §4.5 确定性 ✅（8 纯函数 + IO 排除）/ M6 ✅（动态宽度仍 OUT，`pad*` 替代）/ §2.3 ✅（无新记号）/ **12 类错误类 ✅（无第 13 类）**。唯一待核：既有单测中若有「`range` 二元报 `ArgCount`」断言 → 改 1 条（属 core/runtime 实现轮）。

## 进行中
- （无）

## 阻塞 / 需要支持
- 无。**需 team-lead 转派**：core-dev（`builtins.rs` 注册 15 内置 + `range` 2 参 + `error.rs` 3 个 `ValueMsg` 变体）、runtime-dev（15 内置语义 + 文件 IO）、test-engineer（黑盒正/负例 + coverage-matrix 行；文件 IO 夹具用 `Temp/`）、docs-writer / ai-dx-engineer（guide/skill 54→69 + 新消息 + D1–D7）、verifier（复验）、requirements-analyst（R-401 spec 字节数刷新）。

## 关键经验（写给未来的自己）
- **"固化既有行为"也是规范工作**：`s[i]` 补钉不写一行代码，却把「未定义行为」变成「可判定条文」；**先读 `src/evaluator.rs` 的 `BadOperands` 调用点**才拿到精确消息模板（`{lt} 与 {rt}` 中 `rt` 在索引场景是**期望类型描述** `array / struct`，不是右操作数类型）——**规范必须照实现的事实写**。
- **复杂度的"标注"和"隐藏"是两回事**：`indexOf` 最坏 O(n·m)、`contains` O(n) 是**显式标注**；真正的红线是**隐藏** O(n²)（如 `s[i]`、拼接循环）——7 项均不引入，且 #1 消除了一个。
- **新增内置走 `[min,max]` 参数校验约定**：`range` 1/2 参无需新消息——`m<min→n=min`、`m>max→n=max`（源自 `Builtin::call`），照此写即可与既有 `input(prompt?)` 等一致。
- **子消息可增、错误类不可增**：12 类红线用 `ValueMsg`/`IOError` 自由串承载新消息（6 条）全部落**既有类**内。
- **外部 IO 与确定性的边界要写清**：文件 IO 与 `input()` 同属"环境依赖"，**不**动 §4.5「无魔法」纪律，也**不**改写 `seed` 的"唯一非确定源"表述。
- **（沿用）绝不用 PowerShell `Set-Content`/`Out-File` 改源码/文档**（PS 5.1 ANSI 会毁 UTF-8 中文），只用 `edit`/`write`；**临时探针放 `Temp/` 并自清**。
- **（观察）并发写者**：本轮期间 `docs/guide/{README,errors,reference,tutorial}.md` 被**他人在途**修改（非本架构师，疑 T11-04），`git diff --stat` 会一并显示——汇报时须显式区分。
