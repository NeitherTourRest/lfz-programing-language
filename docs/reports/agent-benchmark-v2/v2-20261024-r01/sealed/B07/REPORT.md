# B07 交付报告

【状态】完成（等待独立验收）

## 产出

- `solution.lfz`：首行 `#42`，全局函数 `solve(text, queries)`；顶层仅定义函数。
- `attempt-01.lfz`：首次调用前保存的源码快照，未覆盖。
- `self-test-01.lfz`：交付函数与自建断言组合的测试文件。
- `attempts.jsonl`：UTC 开始/结束、完整命令、退出码、源码快照与 stdout/stderr 路径。
- `attempt-01.stdout.txt`、`attempt-01.stderr.txt`：原始执行输出。

## 实现

只把 ASCII A-Z/a-z 视为单词字符，逐字符归一为小写，所有其他标量均作分隔符。先完整分词并统计词频，为每个词一次分配位置数组，然后遍历 token 序列填入升序的零起始位置。最后查询索引，不为 query 重扫文本。字符缓冲、token 缓冲和位置数组均预分配，以避免循环追加复制。重复 query 保持输入顺序。

## 验证证据

解释器调用次数：1 / 5。

命令：`.\lfz.exe run self-test-01.lfz`

退出码：0。stderr 为空。stdout 原文：`PASS: 13 assertions`。

自建测试实际含 12 条 `assert`，stdout 标签中的 13 是人工计数笔误；断言均成功。覆盖题目示例、空文本、无字母文本、Unicode/数字/标点分隔、关键字词名、重复 query、空 query 数组、尾部单词、4096 标量的重复词全文及完整位置、4096 字母长词与不存在 query。

只运行了本目录自建测试，未读取或运行隐藏验收集，未声称正式验收通过。

## 剩余问题

未发现功能阻塞。正式结果以独立 verifier 为准。冻结 skill 已足够实现本题，未发现必须依赖额外语言特性的缺口。
