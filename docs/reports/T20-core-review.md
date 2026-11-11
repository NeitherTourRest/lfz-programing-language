# T20 core review与实现交接

2026-11-04。完整启动读取state/board/本人、T20-design-proposal、tooling-review及冻结§26/T20-contract；root授权后仅lexer/parser/error落盘，未编译。

原lexer先扫描全文件，直接把未闭注释视为待续会遮蔽 `let = /*` 的先前致命语法错。实现复用run_inner，私有typed EOF标记只在block_comment实际走到EOF设置；probe保留前缀tokens并补真实EOF，再交原Parser。parser expect/unexpected在真实Eof记录私有Cell标记，非EOF错误不能续输，试探赋值回滚清标记。无中文消息/格式化token比较，无第二套括号计数器。

公开parser::ReplInputStatus{Complete,Incomplete}五traits，probe_repl_input(&str)->R<ReplInputStatus>，沿load_source归一化和AST深度检查。裸字符串/插值EOF仍正式错误；真实LF处原IncompleteExpr不变。完整if立即Complete，后行else是新cell错误。旧parse/lex/API/Span/LzError形状及脚本消息不改。调用方按§26在大栈创建/析构AST。

error新增SyntaxMsg::ReplModuleUnsupported固定“REPL 暂不支持 import/export”、ExecutionLimitReason::ReplCapacity固定“REPL 会话容量已耗尽”；仍13错误类、旧三个原因测试保留。root批准Rust穷尽迁移，runtime/tooling需补新reason消费。

新增4组probe单测覆盖完整fn/if/loop/struct、括号/数组/调用待续、注释尾致命prefix/错误关闭/非法token、字符串/插值LF与EOF、Unicode/CRLF位置和深度上限；新增error严格消息/位置/Display单测。深度测试在on_eval_stack运行。仅静态diff检查通过，尚未编译/跑测，统一编译由tooling执行，不能把方案/源码当验收通过。

只本轮T20，不恢复automation，R304OPEN。无formatter产品改动，无其它owner源/旧行为断言修改。
