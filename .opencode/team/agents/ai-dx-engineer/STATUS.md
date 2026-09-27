# ai-dx-engineer — 工作状态
> 最后更新: 2026-09-27 23:40 by ai-dx-engineer

## 当前状态
**T11-③ P0 完成**：`lfz-programming` skill 的 **7 条盲测缺口（D1–D7）逐条实跑验证后落地**，另新增「O(n) 字符串 / 数组构建」性能红线（§4.6）。**每条改动均有 `lfz` 实跑证据**（`VERIFICATION.md` §8）。skill 已从「盲测 2/8」时的版本升级为「缺口已补」版本。

## T11-③ 交付（本次）
- **改动文件（仅 skill 包 4 个）**：
  - `SKILL.md`：28748 → **37925 B**（§0 退出码；§3 对齐 `<>^`/fill、`else if`、多 `${}`、零参 `print()`、`&&`·`||` 短路、链式下标赋值；§4 新增陷阱 **22–24**；§4.5 两行；**新增 §4.6 O(n) 构建性能红线**；§5 退出码语境；§6 `len`/`range`/`split` 签名；§7 新增 **L5**；§8 自检 +6 项；§9/§10）
  - `VERIFICATION.md`：15028 → **23837 B**（新增 §1.4 需求追溯 + §8 全部实测证据 / 探针源码）
  - `README.md`：5596 → **5932 B**（文件表、退出码语境、证据指针）
  - `prompt-template.md`：2721 → **3265 B**（模板 A/B 增字符串与 O(n) 纪律）
- **逐条证据**（`dist\lfz.exe`，详见 `VERIFICATION.md` §8.1）：
  - D1 `s[0]` → `TypeError`（exit 2）/ `split("",s)` → exit 0
  - D2 `range(3)`→`[0,1,2]`、`range(-1)`→`[]`、`range(1,4)`→`TypeError`（exit 2）
  - D3 循环 `let` 闭包 → `0/10/20`（证明每轮新 cell）
  - D4 `lfz run` assert 失败 → exit **2**；`lfz test` 失败用例 → exit **1**
  - D5 对齐 8 例 exit 0；动态宽度 → `ValueError`（exit 2）
  - D6 `len("日本語")==3`、`len("😀")==1`、`len("")==0`
  - D7 五构造一次跑通 exit 0
- **新发现（须 team-lead 知悉）**：`FEATURE-AUDIT.md` §7.1 建议的「`push`+`join` = O(n)」**实测不成立**——`push` 遵 A1 返回新数组（`to_vec()` 克隆），循环累积**也是 O(n²)** 且更慢。真 O(n) 写法为 `range |> map |> join` 或「预分配 + 下标写」。已写入 §4.6 并追加 ADR 订正（DECISIONS）。

## 进行中
- （无）

## 阻塞 / 需要支持
- （无）
- ⚠️ 提请 team-lead：① 把 §7.1 订正转 **language-architect**（spec 是否需注记 O(n) 惯用法）；② 转 **runtime-dev** 评估「`Rc` 强计数为 1 时就地追加」的可选优化；③ **T11-06 同题盲测重跑**以量测 2/8 → ? 的改进。
- ⚠️ 前置事实：`docs/spec/**` 已含 **v1.1 补钉**（`range(lo,hi)`/`indexOf`/`padEnd`/文件 IO/`ord`/`chr`/math/`contains`），但**实现尚未落地**（`src/builtins.rs` 仍 54 个）。SKILL 已加**版本对齐声明**：**只写当前解释器已实现行为**，不写 v1.1 未落地特性。

## 下一步计划
1. 若 T11-05（语言 v1.1 七项）落地 → **立即同步** §6 签名与 §4.6 复杂度标注，并重跑最小基线 + L1–L5。
2. 与 **app-dev** 闭环：收集 P8+ 卡点，回填 §4 / §4.5。
3. 请 team-lead 派全新 Agent 做「D1–D7 后」的同题盲测重跑（T11-06），用证据量测改进。

## 关键经验（写给未来的自己）
- **「建议」也要实跑**：`FEATURE-AUDIT §7.1` 的 `push+join` 建议看似合理，实测却是 O(n²)——`push` 遵 A1 返回新值会整体克隆。**A1（容器返回新值）是性能陷阱的根源**：任何「循环里 `push`/`insert`/`removeAt`」都是 O(n²)。
- **小 n 会骗人**：`s=s+c` 在 N=2 万时仅 ~30 ms（循环开销主导），到 N=32 万才暴露 O(n²)（1.6 s）。**判复杂度必须跨数量级测**（40k/80k/160k/320k）。
- **复杂度结论要源码 + 实测双证**：`src/evaluator.rs:1496`（`format!` 复制）+ `src/builtins.rs:385`（`to_vec`）解释了两条 O(n²) 的成因。
- **PS 5.1 中文输出**：`chcp 65001` + `[Console]::OutputEncoding=UTF8` 才能干净显示中文错误消息；直接跑会 GBK 乱码。
- 事实源边界：`docs/spec/**` 只读；skill 只转述**已实现**行为；**文档只写「当前解释器已实现」**，避免领先于实现。
