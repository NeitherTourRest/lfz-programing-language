# Blind-Test 报告：knapsack_dp.lfz（0/1 背包 DP + 回溯 + 表打印）

> 盲测身份：我只读了 `skill\lfz-programming\{SKILL.md, README.md, VERIFICATION.md, prompt-template.md}`，
> 未读 LFZ spec、未读项目仓库、未上网。以下所有「事实」均来自这四个文件 + 实跑 `lfz.exe` 的观察。

【状态】完成

【产出】
- `programs\knapsack_dp.lfz` —— 199 行，首行恰好 `#42`（无 BOM，字节级验证 `first3=#42 bom=False`）
- `programs\run_output.txt` —— 完整运行输出（33 行）证据
- `programs\knapsack_dp.report.md` —— 本报告

运行命令与退出码（workdir = `C:\Users\19170\AppData\Local\Temp\opencode\lfz-skilltest`）：

```powershell
& "C:\Users\19170\AppData\Local\Temp\opencode\lfz-skilltest\lfz.exe" "programs\knapsack_dp.lfz"
# EXIT=0
```

输出前 15 行（完整输出 33 行）：

```
item  0  1  2  3  4  5  6  7  8  9 10
-------------------------------------
none   0  0  0  0  0  0  0  0  0  0  0
A      0  0  0  0  0 10 10 10 10 10 10
B      0  0  0  0  7 10 10 10 10 17 17
C      0  0  0  0  7 10 10 10 10 17 17
D      0  0  0  6  7 10 10 13 16 17 17
== instance 1 (classic) ==
capacity 10: optimum value = 17
chosen: 2 item(s), weight 9 <= 10, value 17
  - A(5,10)
  - B(4,7)
item  0  1  2  3  4  5  6
-------------------------
none   0  0  0  0  0  0  0
```

输出尾部（关键校验）：

```
== instance 2 (slice view) ==
capacity 6: optimum value = 9
chosen: 3 item(s), weight 6 <= 6, value 9
  - X(2,3)
  - Y(3,4)
  - W(1,2)
...
== instance 3 (no item fits) ==
capacity 3: optimum value = 0
chosen: 0 item(s), weight 0 <= 3, value 0
  (no item fits)
all self-checks passed
```

三个实例的 DP 最优值均为手算预期值：实例1=17（A+B）、实例2=9（X+Y+W）、实例3=0（无物品可装）。
10 条 `assert` 全部通过（最优值 ×3、选中重量 ≤ 容量 ×3、选中价值 == 最优值 ×1、空选择 ×1 等），退出码 0。

【变更】
- 新建 `programs\knapsack_dp.lfz`（3 个版本：222 行 → 214 行 → 199 行，均为同一算法）
- 新建 `programs\run_output.txt`（运行输出证据）
- 新建 `programs\knapsack_dp.report.md`

【下一步】
- 无（任务闭环：程序 exit 0、报告齐备）。

【阻塞/需支持】
- 无功能阻塞。仅报告若干 skill 缺口（见下），供维护者回填。

---

## 1. 迭代次数与过程

| 轮次 | 动作 | 结果 |
|---|---|---|
| 1 | 写完第一版（222 行）直接跑 | **一次通过，exit 0**（无任何语法/运行时错误） |
| 2 | 裁剪到 214 行再跑 | exit 0，仍超「~120–200 行」上限 |
| 3 | 再裁到 199 行再跑 | exit 0，满足行数约束 ✅ |

**结论：功能上第 1 次运行即通过（0 次失败迭代）**；第 2、3 轮纯粹是为了满足「~120–200 行」的体量要求而做的减行（删空行、删 `padl` 改用插值格式符 `:>3`、删冗余注释与 2 条冗余 assert）。
全程未触发任何 `SyntaxError`/`TypeError`/运行期错误——这归功于 skill 的「硬纪律：不要凭记忆写 LFZ，没写的语法一律视为不存在」。

## 2. 用到的 SKILL 章节

| 章节 | 用途 |
|---|---|
| §0 | CLI 用法、退出码定义（`0` 成功） |
| §0.5 | 5 步工作流：写 `#42` → 逐条查 §6 内置 → 对照 §4 陷阱 → 立刻跑 → 过 §8 清单 |
| §1 | 首行 `#42` 铁律（字节级核验） |
| §2 | 12 错误类清单（备查；本次未触发任何错误） |
| §3 | 语法骨架：`let`/`var`、`struct`+方法、`while`、`if` 表达式/语句、插值 `"${...}"` 与格式符 `:>N`、注释、`+=` |
| §4 | 陷阱：#1 禁用 `;`、#4 `let` 不锁内容、#10 容器内置返回新值（`push`/`insert` 需接回）、#16 条件必须 bool、#19 无 `..`/`++` 等记号 |
| §4.5 | 错误→修法对照表（备查；未用到） |
| §5 | `assert(cond, msg)` 致命语义（本次 10 条 assert 全通过） |
| §6 | 内置函数逐个核对签名与 data-last 顺序：`len` `push` `insert` `str` `repeat` `print` `assert` |
| §8 | 交付前自检清单（逐项过） |

未使用（skill 未给足信息，主动规避）：`range`、管道 `|>`、`map`/`filter`/`reduce`（本任务用循环更直接）。

## 3. SKILL 不清晰 / 缺失 / 疑似错误之处

> 每条给「原文引用 → 我的猜测 → 结果」。

1. **`range` 只列名、无签名**（缺失）
   - 原文（§6）：「**核心 / 数组（21）**：`len` `range` `push` `pop` `removeAt` `insert` `swap` `slice` `min` `max` `sum` …」
   - 猜测：`range(n)`？`range(lo, hi)`？包不包右端？——完全无法确定。
   - 结果：**直接不用**。改用 `while` + `push(0, row)` 手写「零行生成器」`make_row(n)`。这是盲测中最大的"想用却用不了"的功能。

2. **`len` 能否作用于 string 未说明**（缺失/模糊）
   - 原文（§6）只给 `len(xs)` 语境；VERIFICATION 只证过 `len(struct)`；没有任何一行说 `len("abc")` 合法。
   - 猜测：支持（`split("", s)` 按字符拆分的语义暗示字符串有长度概念）。
   - 结果：`padr` 用 `len(r)` 判断补空格数、`repeat(len(head), "-")` 打分隔线——**实跑通过**，猜测正确。

3. **退出码自相矛盾**（错误/冲突）
   - §0：「**退出码**：`0` 成功；`1` 测试失败（`assert` / `fail`）；`2` CLI 参数错误 / 语法或运行时错误」
   - §5：「`assert(cond, msg?)` | 抛 **`AssertionError`（致命）**，终止，退出码 **`2`**」
   - 两条互相打架（assert 失败到底是 1 还是 2）。我无法仲裁，只能保证所有 assert 成立、exit 0。建议维护者统一。

4. **格式说明符只给了 4 种、无左对齐、无动态宽度**（缺失）
   - 原文（§3）：「`"${x:>6}"`、`"${f:.2f}"`、`"${n:05d}"`、`"${n:x}"`」
   - 没有 `<`（左对齐）、没有 `*`/嵌套动态宽度。做「对齐 ASCII 网格」时，列宽只能固定。
   - 结果：数值格用固定的 `${j:>3}` / `${table[i][j]:>3}`（右对齐宽度 3，靠 §3 的 `:>6` 推断 `:>3` 合法）；行标签左对齐则手写 `padr` 补空格。全通过，但 `padr` 本可以是一个 `<` 格式符。

5. **`let` 在循环体内的作用域未定义**（缺失）
   - 原文（§4-4）只讲「`let` 不可重绑定」；没说 `while` 体内 `let x = ...` 每次迭代是「新绑定」还是「重复绑定报错」。
   - 猜测：不确定，不敢赌。
   - 结果：**所有循环局部量一律用 `var` 提升到循环外声明 + 循环内赋值**（如 `var it = nil`、`var skip = 0`、`var take = 0`），彻底绕开。

6. **链式下标赋值未提及**（模糊）
   - 原文（§4-10）只给 `a[i] = v` / `s.k = v`。
   - 猜测：`table[i][j] = v` 应合法（`[]` 是 postfix，链式 lvalue 顺推）。
   - 结果：DP 填表核心语句 `table[i][j] = take` **实跑通过**，猜测正确。

7. **`else if` 链未展示**（模糊）
   - 原文（§3）只给 `if/else` 两分支。`else if` 是否被解析为「`else` 后跟一个 if 表达式语句」不确定。
   - 结果：**一律写嵌套 `if { } else { if { } }`**，不冒险。

8. **一个字符串里多个 `${}` 未显式展示**（模糊）
   - 原文示例每串基本只有一个插值。
   - 猜测：多个插值顺序求值应合法。
   - 结果：`"${self.name}(${self.weight},${self.value})"` 与 report 长行（一行 3 个 `${}`）**实跑通过**。

9. **空 `print()` 未说明**（缺失）
   - 原文（§6）「`print(...)`（空格连接 + 换行）」，没说零参数是否合法。
   - 结果：首版用过 `print("")` 代替空行，后来裁行时删掉，未依赖零参 `print`。

## 4. 错误语法猜测与发现方式

**诚实结论：我没有向解释器提交过任何错误语法——所有「可能猜错」的地方都在写码前主动规避了**（skill 顶部硬纪律：「本文件没写的语法一律视为不存在」）。所以没有 SyntaxError 排错过程可报告。

唯一的真实失败来自环境而非 LFZ：PowerShell 5.1 的 `Tee-Object` 不支持 `-Encoding` 参数（报错 `NamedParameterNotFound`），改用 `> file 2>&1` 重定向收集证据。

若一定要列「我猜过但没写进终稿」的写法：
- `for i in range(n)` → 弃用（`range` 签名不明）
- `a = if c { x } else if d { y } ...` → 弃用（`else if` 不确定）
- `"${v:<5}"` 左对齐格式符 → 弃用（skill 未列出 `<`）
- `let x = 1; let y = 2` 同行 → 已知非法（§4-1），一开始就没写
- `a.push(v)` 原地改容器 → 已知必须 `a = push(v, a)`（§4-10），一开始就没写

## 5. 交付数据

- 最终行数：**199 行**（满足 ~120–200）
- 首行：字节级确认为恰好 `#42`（`EF BB BF` BOM 不存在，`first3=#42`）
- 运行命令：`& "C:\Users\19170\AppData\Local\Temp\opencode\lfz-skilltest\lfz.exe" "programs\knapsack_dp.lfz"`（在 `lfz-skilltest` 目录下）
- 退出码：`0`
- 程序结构：`struct Item`（name/weight/value + `tag()` 方法）、`struct Solution`（table/chosen/value）、`make_row`、`padr`、`total_weight`、`total_value`、`solve`（DP 填表 + 回溯）、`print_table`（支持 step 切片视图 + 对齐 ASCII 网格）、`report`、3 个实例 + 10 条 assert。
