# T20 formatter 独立评估与取舍

本轮明确暂缓生产formatter，不新增format命令、不覆盖源码、不改变check。依据是现代码信息缺失，而非把REPL工作冒充格式化已经完成。

- src/lexer.rs的line_comment/block_comment丢弃注释，空白不保trivia；`let x=1 // unit`与无注释版本无法由现AST区分，AST打印会丢有用说明。
- src/ast.rs明确无Pipe节点，parser把`x |> f()`脱糖为Call；AST无法恢复用户管道写法。整数AST存i64，`0xff`与`255`拼写归一，负最小值等也有折叠。
- 节点主要携带起始Span，不是原文完整range/CST；不能可靠将漂浮注释关联到重排节点。块注释换行与LFZ同逻辑行TwoStatements还影响语义，简单插删换行不是纯外观改动。

后续若授权formatter，先定义lossless token/trivia与源码范围、注释附着、#42/BOM/CRLF/末行政策，再规划完整CST/保真token格式化成本。门禁应包括幂等、所有文法样例、注释保留和副作用程序行为差分；仅parse成功或脱糖AST相等不足以证明用户源码可无损重写。

本结论沿docs/spec §26.6；REPL/指南/新版Agent任务包可独立完整交付。未写format产品或假造幂等/行为测试通过。
