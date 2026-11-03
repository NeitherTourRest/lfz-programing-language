# 有界逐行读取与 JSONL

T19 新增 `forEachLine(callback, maxLineBytes, path)`，适合计数、过滤和逐条 JSONL 处理。它不先把整个文件读进字符串。正式语义见三份规范 §25；原 `readFile` 的整文件读取、编码和错误行为不变。当前共76个内置名字、13类错误；没有新文法、权限开关或 Rust context 字段。普通内置仍不能直接取一等函数值，名称仍可被用户绑定遮蔽。

## 先检查，再处理

在项目根目录创建小数据文件；这里的 UTF-8 不带 BOM，混合两条有效 JSON、一条坏 JSON 和一条空行：

```powershell
New-Item -ItemType Directory -Force Temp/T19-docs | Out-Null
[IO.File]::WriteAllText((Join-Path $PWD 'Temp/T19-docs/input.jsonl'), "{`"n`":1}`nbroken`n`n{`"n`":2}`n", [Text.UTF8Encoding]::new($false))
```

将以下完整代码保存为 `Temp/T19-docs/jsonl.lfz`。回调只保留计数，避免把所有记录积累到数组：

```lfz
#42
var accepted = 0
var rejected = 0
var total = 0
var currentLine = ""
let parseCurrent = () => jsonParse(currentLine)
let result = "Temp/T19-docs/input.jsonl" |> forEachLine((line, lineNo) => {
    currentLine = line
    let parsed = attempt(parseCurrent)
    if parsed.ok {
        accepted = accepted + 1
        if parsed.value != nil {
            total = total + parsed.value.n
        }
    } else {
        rejected = rejected + 1
    }
    return true
}, 4096)
assert(result.lines == accepted + rejected)
assert(!result.stopped)
print(jsonStringify({accepted: accepted, rejected: rejected, sum: total, lines: result.lines}))
```

管道把路径放到第三个参数，等价于 `forEachLine(callback, 4096, path)`。在项目根运行，路径按宿主当前目录解析，不按回调定义模块所在目录。`jsonParse` 只负责解析；真实业务还应校验记录结构，例子的数据中有效记录均有整数 n。

```powershell
./target/release/lfz.exe check Temp/T19-docs/jsonl.lfz
if ($LASTEXITCODE -eq 0) { ./target/release/lfz.exe run --max-steps 10000 --timeout-ms 2000 Temp/T19-docs/jsonl.lfz }
```

最终候选实跑输出（固定解析闭包版本）：

```json
{"accepted":2,"lines":4,"rejected":2,"sum":3}
```
 check 不执行回调或读取数据文件，也不保证名称、类型和业务数据正确。命令中的限额只用于 run；默认仍不限额。文件读取沿既有 IO 边界，无需 `--allow-http` 或 `--allow-process`。

## 回调与结果

回调必须是恰好接收 `(line, lineNo)` 两参的用户函数，`lineNo` 从1开始。每次必须返回严格的 bool：true 继续，false 早停；nil、整数等返回 TypeError。空文件也会检查回调参数数目和所有接口参数，不能靠没有数据绕过校验。

成功返回普通 struct，字段插入顺序 `lines`、`stopped`。lines 是实际调用回调的次数，包含返回 false 的那一行；仅 false 导致的早停令 stopped 为 true，正常 EOF 为 false。false 后不会解析或解码下一行；固定缓冲可以已预读后续字节，不等于继续交付。

任何错误都不返回部分成功结果。已有计数、容器修改或外部副作用不回滚。回调函数沿原 captured/self/home globals 与来源信息调用，跨模块错误保持真实 LFZ 源位置；数据文件行号是实参和消息内容，不冒充 LFZ 源码位置。

## 行与字节上限

maxLineBytes 必须为整数1..16777216。上限计 **LF 前的原始字节**，包含随后剥掉的 CR，不包含 LF。恰好 N 允许，N+1 立即 ValueError，坏行不交给回调。UTF-8 的汉字通常占多个字节，不能按字符数估算。

| 数据字节 | 交付内容 | 行数 |
| -- | -- | -- |
| 空文件 | 无回调 | 0 |
| `a` 或 `a\n` | `a` | 1 |
| `\n` | 空字符串 | 1 |
| `\n\n` | 两个空字符串 | 2 |
| `a\r\nb\r` | `a`、`b\r` | 2 |

只有 LF 分行；带 LF 的 CRLF 去掉一个末 CR，EOF 孤 CR 保留。因此 `a\r\n` 需预算2，`a\n` 需预算1。尾 LF 不凭空生成最后空行。BOM、空白、空行、注释样文本及内容 NUL 均保留，不应用 LFZ 源码 loader 的 `#42` 或 ext 规则。路径 NUL 则是参数错误。

每行严格 UTF-8，跨读取块的合法多字节字符仍可用；非法编码在回调前报 ValueError，不作有损替换。BOM 保留意味着带 BOM 的首条 JSON 也会原样交给 jsonParse，是否接受沿其原行为，不自动修理数据。

## 错误恢复与资源边界

JSON 语法坏行已是合法字符串，可在回调里 `attempt(() => jsonParse(line))` 后继续。空行是否跳过由脚本决定，本例将它计入 rejected。超长行和非法 UTF-8 尚未交付，当前回调不能恢复；外层 attempt 只能捕获整次失败，不能恢复文件偏移继续扫描。访问错误字段 class 要用 `r.error["class"]`。

| 情况 | 错误与处理 |
| -- | -- |
| cap不在范围、路径NUL、超长、坏UTF-8、行号溢出 | ValueError，可由外层attempt捕获整次读取失败 |
| 非普通文件、metadata/open/read失败 | IOError，消息 `无法逐行读取：{path}` |
| 回调参数或返回类型不符 | TypeError；参数表达式仍先从左到右求值 |
| 步数耗尽、全局取消或deadline | ExecutionLimitError，attempt不能吞或补回预算 |

原 readFile 的坏 UTF-8 行为保持不变，不能把新接口的 ValueError 推广到旧接口。

参数表达式和回调的真实 AST 语句/表达式共享单会话步数。native 读取只协作检查取消和 deadline，不按字节、块或行额外扣 AST 步数；回调需要执行 AST 时仍受剩余预算限制。回调返回 false 后也先检查全局终止，观测到取消时不会伪装早停成功。metadata/open/read 是同步 OS 调用，检查不能硬打断阻塞，不承诺毫秒级返回；取消仅由宿主 token 控制，不新增 CLI Ctrl-C 保证。

只接受普通文件：预 metadata 和打开后 metadata 都检查，可跟随普通文件符号链接。这不是安全沙箱，检查与 open 之间仍有 TOCTOU 竞态，不能保证绝不碰到被替换的设备或消除阻塞。读中修改、追加、截断沿 OS 观测，不是快照、锁或事务。

reader只持当前有界行和固定8KiB输入缓冲，空间 O(当前最长行+8KiB)，不保留历史行。当前解释器的函数诊断元数据 func_names 可随新建 UserFn 数量增长；每行创建 `() => jsonParse(line)` 不能据此宣称整个脚本内存有界。教程把 currentLine 放在外层，仅创建一次 parseCurrent 闭包，在回调赋值后复用 `attempt(parseCurrent)`。这是同步顺序处理写法，不用共享该 cell 实现重入解析。用户自己收集数据仍可占 O(文件)；嵌套调用各有缓冲，受原回调递归深度限制。这是实现口径，吞吐和峰值内存必须另有测量，R304仍OPEN。

EOF、早停、回调错误、非bool、资源终止及读取/解码失败均统一释放 reader/File，RAII也覆盖提前退出，外层 attempt 返回时已释放。但标准库 Drop 不回传 OS close 错误，不能声称报告了关闭异常；没有新增 diagnostic sidecar。没有可逃逸的文件 handle。

本轮只收口 T19，不自动进入 T20 或恢复自动化。最终命令与原始输出将记于 [指南证据](../reports/T19-guide-evidence.md)。
