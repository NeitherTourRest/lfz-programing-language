# tooling-dev — 工作状态
> 最后更新：2026-11-04 T20首完整回归等待runtime交接

## 当前状态
T20 CLI/REPL前端已实现，修后同writers2/2与真实stdin7/7通过；完整lib527通过/1新增runtimeunit失败，已报root，不越权改runtime/core。本人源/编译暂停等待owner修复后GO，尚无最终hash/全套成功。

## 本轮产出
- cli::execute仅二进制内部Write+Send（root批准），main plain Stdout/Stderr不改旧锁策略；整个REPL driver同on_eval_stack，sharedstdin fill_buf/consume追加前1MiB限制、无另BufReader、每行返回释放锁再eval。
- 新repl扫描、原flags/argv保护/--json拒绝、typedprobe、多行/EOF/错误继续、元命令/累计exit、autoecho非nil、双diag/容量poison仅reset/help/quit；独立REPL compile-error adapter按真实source与Loaded、tracedNone渲染，不改旧graph/run。
- tests/cli_t20.rs七项进程测试及同writers/读取cap两unit。首2/2、进程5/2失败保留；一项新oracle把assignment nil多算回显（改新test为2次），另一项真实compile失败缺source渲染已修adapter；第二次2/2+7/7通过。
- docs/reports/T20-formatter-assessment.md明确lexer丢comments/trivia、AST管道脱糖/数值拼写归一，formatter本轮有据暂缓，无format/check伪功能。

## 当前待协调
Temp/T20-tooling/first-full-runtime-failed.txt及summary：evaluator.rs5068新增unit输入n=9后import，原parser报import必须先于普通语句，unit期待ReplModuleUnsupported。旧parser优先级不能为此随便改，交runtime/root确认新oracle。当前无cargo，待owner停源后我唯一release重建/完整13harness。真实PTY提示尚未实跑（同writers模拟TTY不冒称PTY）。源未终验不公开套旧字节。

所有失败/红证据在Temp/T20-tooling，原旧tests/用户块/dist/Git保持。仅T20收口，无外装/公网/automation，R304OPEN。
