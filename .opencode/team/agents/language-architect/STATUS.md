# language-architect — 工作状态
> 最后更新：2026-11-13 by language-architect

## 当前状态
T21保真空白格式化三spec §27及T21-contract冻结，root可据此给源码GO。
行首CODE TAB一对一换SP、行尾CODE SP/TAB清理。保原换行/BOM/#42、注释/字符串等原字节及所有实质token位置。裸CR+空白+LF保整个run。已指出attempt.col可观察，root裁决不按深度重新缩进。
CLI原argv恰fmt FILE且FILE非空、不以连字符开头才进入；其余原解析，含fmt --json、fmt --、fmt --allow-process/http、--json fmt、fmt -- ARG旧裸文件形态。--json fmt FILE与附选项的新调用拒绝。原/新parse及token kind/行号/实质列一致，幂等/差分由owner实测。

## 分工与证据
core唯一lexer::lex_format_layout(&str,u32)->R<(Vec<Token>,Vec<Range<usize>>)>，归一化标量索引。
tooling唯一formatter.rs/lib.rs/cli；format_source(&str,&[u8])->Result<Vec<u8>,FormatError>，InvalidSource或InvariantViolation。
文档diff检查通过，本角色未写源码/编译。后续由root调度实现验收，R304状态保持。
