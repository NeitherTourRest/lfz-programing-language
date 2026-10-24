# B01 整数表达式解释器交付报告

【状态】完成，已冻结交卷源码。

【产出】
- solution.lfz：全局 fn solve(source)，首行 #42，无顶层输入、演示输出或文件操作。
- attempt-01.lfz：首次执行前源码快照，未覆盖。
- test-01.lfz：自编测试，与首次快照相同的函数加 16 条断言。
- attempts.jsonl、attempt-01.stdout.txt、attempt-01.stderr.txt：执行记录。

【实现】
一次 split 将字符串转字符数组；递归下降按 expression / term / unary 解析。循环实现二元左结合；连续一元正负号按奇偶处理；括号递归；除法使用 div 保持向下取整和 int 类型。负数字面量直接负向累积，可表示 i64 最小值。输入有效性按任务书保证，不额外处理无效表达式。时间 O(n)，字符数组 O(n)，解析栈随括号深度增长。

【验证】
仅调用本目录 lfz.exe 1 次，未使用其他解释器或语言代算。
命令：.\lfz.exe run test-01.lfz
退出码：0。
stdout：16 assertions passed。
stderr：空。
自编断言覆盖优先级、乘除与加减左结合、正负除数的向下取整、空格、连续一元、相邻二元与一元、嵌套、零、前导零、i64 最大值和最小值、返回 int、重复调用接口。
UTC 开始/结束详见 attempts.jsonl。

【剩余问题】无已知问题；未读取或运行隐藏验收，正式验收结果待独立 verifier 确认。
【skill/实现缺口】本题所需语法和 div、split、ord 等内置均可从提供的 SKILL.md 获取，首次执行未遇到缺口。
