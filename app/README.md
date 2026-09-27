# sortviz.lfz — LFZ 排序算法可视化（终端字符画应用）

> 交付物 6（评分项 5）｜ 作者：app-dev ｜ 依据 `.opencode/skills/lfz-programming/SKILL.md` 开发
> 源码：`app/sortviz.lfz`（单文件，341 行 / 有效行 290）｜ 开发记录：`app/DEV_RECORD.md`

`sortviz` 是一个纯 LFZ 编写的**终端排序算法可视化器**：生成可复现的随机数组，用 5 种排序算法排序，**逐帧打印字符条形图**（`#` 个数表示数值），统计**比较/交换/步数**，并对结果做正确性校验。LFZ v1 无图形库，故选用「字符画 + 计数」这一最贴合语言能力的展示形态。

---

## 1. 功能列表

| # | 功能 | 说明 |
|---|------|------|
| F1 | **5 种排序算法** | 冒泡、选择、插入、归并、快速（Lomuto 分区） |
| F2 | **逐帧可视化** | 每趟/每轮后用 `repeat` 打印条形图，`#` 个数表示数值大小 |
| F3 | **参数可配** | 规模 `N_SMALL`/`N_BIG`、随机种子 `SEED`、帧宽 `CELLW`、`SHOW_FRAMES` 开关（见 §5） |
| F4 | **可复现随机** | `seed(SEED)` + `randInt` → 同种子必得同一数组 |
| F5 | **统计** | 比较次数 / 交换次数 / 步数（结构体 `Stats` 累加） |
| F6 | **正确性校验** | `assert`（致命）+ 与内置 `sort()` 对比；另有 150 次属性自测（`check` 非致命） |
| F7 | **数据摘要** | 管道 `|>` 串 `min`/`max`/`sum` 输出摘要 |

---

## 2. 运行方法

在仓库根目录（PowerShell）：

```powershell
$env:Path += ";$env:USERPROFILE\.cargo\bin"
cargo run --quiet -- run app/sortviz.lfz
```

- **退出码**：`0` = 全部算法校验通过；`2` = 语法/运行时错误或某个 `assert` 失败。
- 机器可读模式：`cargo run --quiet -- run --json app/sortviz.lfz` → stdout 为 `{"ok":true}`，程序输出转 stderr。

> ⚠️ **无命令行参数**：LFZ CLI v1 只支持 `lfz run <file>` 与 `lfz test`，不支持向脚本传参、不支持读环境变量/文件。因此本项目用**源码顶部常量**配置（文档化的可复现步骤见 §5）。

---

## 3. 交互示例（输入 → 输出）

**输入**（即源码顶部 CONFIG）：`N_SMALL=8, N_BIG=600, SEED=42, CELLW=6, SHOW_FRAMES=true`

**输出节选**：

```
APP_NAME ： LFZ 排序算法可视化      ← ;; 变量 dump（此时全局仅 6 个配置常量）
N_SMALL ： 8
...
== 阶段 1：小规模逐帧可视化（N=8） ==
原始数据: [13, 91, 58, 64, 50, 62, 25, 8]
数据摘要: min=8  max=91  sum=371  n=8
初始条形图:
origin        |      |######|###   |####  |###   |####  |#     |      |

-- 冒泡排序（逐趟）--
bubble t=0    |      |######|###   |####  |###   |####  |#     |      |
bubble pass 1 |      |###   |####  |###   |####  |#     |      |######|
...
  bubble     比较=    28  交换=    19  步数=    28  [校验通过]

== 阶段 2：大规模仅统计对比（N=600，不输出逐帧） ==
数据摘要: min=0  max=99  sum=30660  n=600
  算法         比较次数    交换次数      步数   校验
  bubble     比较=179510  交换= 93045  步数=179510  [校验通过]
  selection  比较=179700  交换=   589  步数=179700  [校验通过]
  insertion  比较= 93640  交换= 93045  步数= 93640  [校验通过]
  merge      比较=  4803  交换=     0  步数=  5576  [校验通过]
  quick      比较=  6473  交换=  2433  步数=  6473  [校验通过]

== 阶段 3：属性自测（随机数组 × 5 算法，check 非致命） ==
  自测 150 次排序，失败 0 次（check 非致命）
== 完成：5 种算法全部通过正确性校验 ==
```

条形图读法：每个数值占一个单元，单元内 `#` 数量 = `数值 × CELLW ÷ 当前最大值`（`div` 向下取整），故最宽为 `CELLW` 个 `#`。

---

## 4. 设计结构（文字版模块图）

LFZ v1 **没有模块/import 机制**（`import` 是保留字，使用即 `SyntaxError`），因此整个应用是**单文件 + 清晰分节**：

```
app/sortviz.lfz
├─ §0 CONFIG          配置常量（N_SMALL/N_BIG/SEED/CELLW/SHOW_FRAMES）+ `;;` dump
├─ §1 Stats           统计结构体（cmp/swaps/steps 字段 + 4 个方法）
├─ §2 genData(n)      seed + randInt 数据生成
├─ §3 可视化           padRight / cellOf / barLine / hline / summarize
├─ §4 排序算法         bubbleSort / selectionSort / insertionSort
│                      mergeSort(+msort,mergeParts) / quickSort(+qsort,partition)
├─ §5 校验             isSorted / runSort / selfTest
└─ §6 驱动             阶段1 逐帧（小 N）/ 阶段2 统计（大 N）/ 阶段3 属性自测
```

关键设计点：
- **引用语义**：`array`/`struct` 是引用类型，传参按引用；排序在 `slice()` 出的副本上**原地交换**，不污染调用方数据。
- **结构体作计数器**：`Stats` 实例经方法 `self.cmp += 1` 原地累加（`self` 引用调用方实例）。
- **快照回调**：排序函数接收 `snap` 回调（lambda），在关键时刻输出条形图；`SHOW_FRAMES=false` 时回调为空操作。

---

## 5. 配置方式（可复现步骤）

因 CLI 不支持参数，改配置 = 改源码常量。示例：**切到「大规模仅统计」模式**：

1. 打开 `app/sortviz.lfz`，把第 17 行 `let SHOW_FRAMES = true` 改为 `false`；
2. `cargo run --quiet -- run app/sortviz.lfz` → 阶段 1 不再输出逐帧条形图，只余统计行（退出码 0）；
3. 复原为 `true`。

改 `SEED`（如 42 → 7）可复现另一组数据；改 `N_BIG` 可调规模（越大越耗时，见 §7）。

---

## 6. 5 大语言特色对照

| 特色 | 代码位置 | 说明 |
|------|---------|------|
| **管道 `|>`** | L55, L66–68, L256, L258, L263, L308 | `xs |> map(...) |> join("|")`；`data |> sort()`；`xs |> min()/max()/sum()`（data-last） |
| **结构体** | L22–30（`struct Stats`） | 数据字段 + 方法（`self.cmp += 1` 原地累加） |
| **富插值 / 格式说明符** | L29、L55、L265、全文件 | `"比较=${self.cmp:>6}"`（右对齐 6 宽）；`"${name} ${label}"` |
| **`check` / `assert`** | L265（assert）、L292（check） | `assert` 失败致命（退出码 2）；`check` 非致命（stderr 提示、继续） |
| **`;;`** | L19 | 变量 dump，输出 `<名> ： <值>`（内→外、遮蔽去重） |

---

## 7. 已知限制（LFZ v1 语言/实现能力）

1. **无模块系统** → 应用只能单文件（`import` 保留字，用即报错）。
2. **无 CLI 参数 / 环境变量 / 文件读取** → 参数只能用源码常量（§5）。
3. **无 `input()` 交互默认路径** → `input()` 在无 stdin 环境抛 `IOError: 输入结束（EOF）`，故默认运行不调用 `input`。
4. **容器内置返回新值** → `push` 等每次返回新数组（O(n) 拷贝），归并排序用 `push` 建结果的开销随规模上升，故大规模 N 取 600（`N_BIG` 调大需注意耗时）。
5. **浮点显示**：`/` 恒为真除法返回 `float`；本应用所有取整一律用内置 `div(a, b)`。

---

## 8. 相关文件

- 开发记录（选题/需求/迭代/踩坑/验证）：`app/DEV_RECORD.md`
- AI 编程指南：`.opencode/skills/lfz-programming/SKILL.md`
- 语法/语义事实源：`docs/spec/{syntax,semantics,interface-contract}.md`
