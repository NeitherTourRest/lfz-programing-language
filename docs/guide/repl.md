# 用 REPL 逐段调试 LFZ

依据 [spec §26](../spec/interface-contract.md#26-t20-repl2026-11-04-冻结)。本页示例已准备，最终候选实跑证据见 [T20 人类指南验证](../reports/T20-guide-evidence.md)；实跑前不计通过。

启动 `lfz repl`。REPL 输入不要求文件的 `#42` 前导。终端在 stderr 输出 `lfz> `，未完成片段输出 `...> ` 并刷新；管道输入没有提示或欢迎语。成功结果非 nil 时回显一行；`print` 自己输出且返回 nil，不重复回显。赋值不额外回显。

Windows 下直接在终端运行 `lfz repl` 即可。普通控制台实测在 CP437、CP936 和 CP65001 下均能正确输入和显示常见中文，并显示中文错误诊断。需要捕获 stdout 或 stderr 时，使用普通管道并按 UTF-8 解码。直接重定向到只写的 `CONOUT$` 在 CP437 和 CP936 下可能出现乱码。[Windows 控制台独立验证](../reports/T22-independent-verification.md)

```text
var n=1
fn read()=>n
n=2
read()
```

最后一行得到 `2`。同一会话保留绑定、闭包和容器；重复声明及闭包对全局名字的查找沿原语言规则。只执行刚提交的 cell，不重放旧 print、文件写入或外部请求。

多行函数在闭合后立即提交：

```text
fn add(x) {
return x+1
}
add(2)
```

结果为 `3`。未闭合的合法括号、数组、块、块注释等可等待后续行；非法前缀直接报错，不能靠后面的未闭合注释掩盖。裸字符串不能跨实际换行，缺操作数也不自动获得新续行语义。

`if` 闭块已完整便立即执行。因此 `else` 必须和前面的 `}` 放在同一物理行：

```text
if true {
print("yes")
} else {
print("no")
}
```

输出 `yes`。若把 `else` 放到下一行，上一 cell 已经提交，不会重新拼接执行。EOF 遇到不完整 cell 时报告真实语法错误并退出 2，不自动补括号；空 EOF 正常退出 0。

## 错误后继续与真实来源

```text
var state={n:0}
fn failLater(){
state.n+=1
1/0
}
failLater()
state.n
print("continued")
```

调用报错后仍能读到 `1` 并输出 `continued`：已完成的声明、容器写入、IO 不回滚。错误显示独立 `<repl:N>` 来源，旧函数或 struct 默认值报错时使用定义 cell 的文本/caret，不套用当前调用文本。错误后的求值栈会恢复，但不是事务。

REPL 本轮拒绝 import/export，整 cell 通过语法后会在任何求值效果前拒绝；语法本身非法时保留原诊断。需要模块时使用原 `lfz check file.lfz` 和 `lfz run file.lfz`。

## 命令、stdin 与退出码

只有当前没有待续输入时，整行去空白后 `:help`、`:reset`、`:quit` 才是命令。未知冒号命令报错；待续片段内按普通源码处理。`:reset` 清除会话绑定、历史来源和函数 ID，cell 编号重新从 1 开始；不会清除本次进程已发生错误的记录。

```text
input()
:quit
print("continued")
:quit
```

第一次 `:quit` 是 `input()` 消费的数据，回显 `:quit`，然后输出 `continued`；第二次才退出。输入循环在求值前释放共享 stdin 锁，不抢走 builtin 要读取的下一行。不要在调用 `input()` 后误把下一条代码当作无需输入的数据。

任一 cell 或命令失败，最终退出码累计为 2；后续成功、`:reset`、`:quit` 不抹掉失败。全程成功为 0。REPL 拒绝 CLI `--json`，不产生新的 JSONL 错误协议；业务 JSON 可自行 `jsonStringify`。

## argv、权限与执行预算

`lfz repl --max-steps 100 --timeout-ms 5000 -- --json hello` 的 argv 快照为两个字符串，`--json` 是参数而非 CLI 选项。`:reset` 保留 argv 和启动权限。process/http 能力仍默认禁用，需要启动时显式 `--allow-process` / `--allow-http`；这不是 OS 沙箱。现受控子进程/HTTP 后端的实际支持范围为 Windows，其余平台资源调用 Unsupported。

每个 cell 重建步数和相对超时；超时从 parse 后求值开始，不计等待输入。比如以 `--max-steps 100` 启动，`while true {}` 受控终止后可另输 `1`，仍得到 `1`。`attempt` 不能吞掉系统执行终止。库宿主设置的绝对 deadline/cancel token 不被新 cell 或 reset 清除，不能将 reset 当作绕过取消的方法。大内置和阻塞 IO 的限制见 [执行控制](execution-limits.md)。

## 长会话与容量

单 pending/raw cell 最多 1 MiB（1048576 bytes），累计已提交 raw source 最多 16 MiB（16777216 bytes），最多 4096 个非空提交 cell，含失败；空白/纯注释不占历史。累计 func/default 诊断 ID 上限 65536。原始输入的换行与 UTF-8 字节也计入字节上限。超长输入在追加前有界检查，立即退出 2，不把剩余字节当下一 cell。

source/cell admission 超限在求值前拒绝，旧会话保留；若仍有空间可尝试较小 cell。执行中 func ID 耗尽为不可被 attempt 捕获的 `ExecutionLimitError`，消息“REPL 会话容量已耗尽”，此前效果保留，必须 reset。process/HTTP cleanup 完整性失败也会污染会话；交互模式只允许 help/reset/quit，pipe 模式结束，不继续执行业务。

历史源码/函数名随会话保留，旧错误快照可能使来源表复制；不能宣称长会话恒定内存。reset 重建会话也不等于完整 GC 或任意引用环全部立刻返还 OS。达到限制前分段保存有用脚本，再开新会话。

## 本轮工具边界

formatter 已完成独立评估但暂缓实现：现 lexer 丢弃注释/trivia，AST 不保存管道与数值原始拼写，直接重印会失真。见 [评估报告](../reports/T20-formatter-assessment.md)。没有新增 format 命令；check 沿既有非执行语法/模块图检查，不新增动态查名等未冻结规则。R304 仍 OPEN，功能候选不代表原性能门槛通过或正式发布。
