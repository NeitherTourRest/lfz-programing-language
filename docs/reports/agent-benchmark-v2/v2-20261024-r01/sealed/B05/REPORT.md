# B05 完成报告

## 交付

- `solution.lfz`：首行 `#42`，提供全局 `fn solve(items, capacity)`，返回 `{value:int, indices:array[int]}`。
- 顶层只有函数定义，没有输入、输出、文件操作或提前退出。
- 按原下标顺序构造 `(n+1) × (capacity+1)` 二维 DP；仅候选值严格更大时选取当前物品。随后从最后一项回溯，通过原下标过滤输出升序数组。
- 时间复杂度 O(n × (capacity+1))，空间复杂度 O(n × (capacity+1))。使用预分配与下标写入，没有循环 push 累积。
- 仅使用整数加减及比较；候选来自互不重复的物品，题目给定总价值上限保证候选加法不超 i64。

## 自行验证

解释器调用次数：1 / 5；首次调用退出码 0；无需修正代码。

完整命令：`.\lfz.exe run selftest-01.lfz`（工作目录为本题 B05 目录）。

stdout：`PASS: 15 self-authored cases with interface and consistency assertions`。

stderr：空。

自编 15 例覆盖：空输入两种容量、容量 0、全放不下、单物品不足/恰好/容量富余、重复物品、等值不同重量时不选后项、组合与后项等值时不选后项、全零价值、普通回溯、最大单项价值、恰好 i64 最大总价值、24 项且容量 80。每例同时检查精确返回值、精确下标、升序唯一、总重可行和价值一致性。

证据文件：`attempt-01.lfz`（首跑不可覆盖快照）、`selftest-01.lfz`（测试源码）、`attempt-01.stdout.txt`、`attempt-01.stderr.txt`、`attempts.jsonl`（UTC 开始/结束、完整命令、退出码及输出文件）。

## 限制与剩余问题

未读取或运行隐藏验收集，不对独立正式验收结果作声明。当前无已知实现问题，等待独立 verifier。未访问本题目录以外文件，未联网，未调用其他语言或程序求解，未委派。
