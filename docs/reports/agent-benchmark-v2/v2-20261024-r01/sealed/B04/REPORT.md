# B04 交付报告

- 交付：`solution.lfz`，首行 `#42`，全局接口 `fn solve(grid, start, goal)`，顶层无执行语句。
- 算法：BFS，预分配 FIFO 队列；邻居按上、右、下、左处理；首次入队设置前驱并标记，之后不覆盖。到达目标后沿前驱重建包含首尾的路径，无路返回 `[]`。
- 不修改传入的 grid、start、goal；字符通过 split 读取。
- 时间与额外空间均为 O(rows × cols)，没有队列头删除或重复全图扫描。

## 自行验证

- LFZ 解释器调用次数：1 / 5，首次执行成功。
- 完整命令：`.\lfz.exe run selftest.lfz`
- 退出码：0。
- stdout：`PASS: own BFS interface, boundary, ordering, detour, no-path, immutability, 20x20 tests`
- stderr：空。
- 自编测试覆盖：单点；起终点相同；单行、单列；上优先于右、右优先于下、下优先于左；无路；障碍绕行；输入内容不变；20×20 开放网格最短长度及逐坐标顺序。
- 源码快照：`attempt-01.lfz`（执行前保存，未覆盖）。
- 测试入口：`selftest.lfz`（包含与交付相同的函数源码及自编断言）。
- UTC 起止、完整命令、退出码与输出文件名：见 `attempts.jsonl`。
- 输出文件：`attempt-01.stdout.txt`、`attempt-01.stderr.txt`。

## 限制与剩余问题

未读取或运行隐藏验收集，不声称通过正式验收。当前自行测试未发现剩余问题。实现只使用所提供 SKILL.md 中列出的语法和内置函数，本题未遇到指南缺口。
