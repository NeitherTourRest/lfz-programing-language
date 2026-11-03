# T19 公开验收计划

2026-11-03，test-engineer。完整启动已读全局/看板/自身记忆、NEXT、T19 proposal及runtime/tooling reviews；root随后确认三spec §25与 `T19-contract.md` 正式冻结。本计划以冻结三参 `forEachLine(callback,maxLineBytes,path)`、返回 `{lines,stopped}` 为准，不沿旧review的eachLine/visited或未采纳close诊断建议。当前只有准备文档，无脚本/产品运行/编译/Git/网络请求。

范围仅本轮T19，完成后收口不T20、不重建automation。旧readFile、旧黑盒、旧v2、原始失败及原用户修改不动；所有fixture/临时证据放项目Temp。官方178、公开进程和库/独立计数另列。R304 OPEN，不能沿用旧最终字节的性能结论。

## 小fixture公开矩阵

后续拟新增 `tests/t19_cli_lines.py`，逐case独立目录与固定raw bytes输入；Python参考按字节LF分割、单独实现CR剥除/UTF8校验生成oracle，不调用待测LFZ生成预期。fixture SHA、raw hex或长度、源、参数、产品SHA、双通道/exit/JSON键序、预期/实际与所有失败保全。

| 维度 | 输入与断言 |
| -- | -- |
| 空与尾部 | 空0行；a、a+LF各1行；LF一空行；LF+LF两空行；无尾换行末行交付；尾LF不再额外造行；序号从1，含空行，返回lines包含false那次 |
| CR/LF | a+CR+LF+b+CR交付a和b+CR；CR+CR+LF只去最后一个CR；纯CR/Unicode行分隔符保持内容；空白不trim、不跳注释/#42 |
| BOM/NUL | UTF8 BOM属于第一行字符串；行内NUL合法；path NUL走LFZ源码文件传入，不在Windows进程argv嵌NUL；数据.lfz不应用loader前导规则 |
| 精确raw cap | a+LF cap1允许；a+CR+LF cap1失败/cap2允许；多字节雪的3bytes cap3/2；EOF末行同N/N+1；CR虽交付前去掉仍计raw cap；固定小cap、跨8KiB cap、16MiB边界另受控单文件，不造OOM |
| UTF8跨块 | 2/3/4字节字符刻意横跨8192边界和LF跨界；合法完整交付；截断/非法continuation在回调前ValueError含真实数据行号；此前计数/写入保留，坏行无回调 |
| false尾毒 | 第一行false，后为非法UTF8/超长；返回lines1/stoppedtrue，无后续回调/输出；承认有限buffer可预读bytes，不把结果称内核完全没读尾部 |
| 类型与优先级 | arity、callback非function、cap类型、path类型、callback形参数、cap0/负/超16MiB、path NUL；空文件也验证callback恰2参；不存在path配错误callback验证先callback而非open。返回nil/int/string/struct为TypeError，不以truthy接受 |
| 文件范围/IO | 缺文件、空path、目录受控IOError固定“无法逐行读取：path”；普通文件符号链接只在Windows可用时实测，否则标环境不适用；不创建设备/FIFO或测试不可中断恶意open，不称metadata消灭TOCTOU |
| callback效果 | true走EOF、false立即停止；第K行业务错误/非bool无第K+1副作用；外包attempt保已有用户效果但不恢复offset；独立再调用从首行开始，不冒称错误后续读同reader |
| JSONL | valid/bad/空白/空行/连续bad/Unicode后valid，callback内部attempt(jsonParse)决定统计；只UTF8合法JSON坏行可逐行恢复；坏UTF8/TooLong只能外层捕获整次流失败；生成与参考重读各算计数/和值/错误行号 |
| home/source | callback定义模块与caller同名globals区分；捕获cell/self/逃逸closure沿旧规则；path相对cwd而非home；callback错指向模块脚本、native错指向调用点，数据lineNo不冒充LFZ源行；先捕获坏JSON再后续新错无旧frame/source |
| 嵌套 | 外流callback调用内流，独立reader/序号/返回；同文件重入用小有限fixture，不依赖读中变更的确定快照；修改/追加/截断仅记录OS观察，不写不可保证的完整顺序oracle |
| 资源 | 小steps下callback无限while受控，嵌套attempt不能吞；多行callback共享remaining、不以行bytes扣AST；deadline有限callback见证，host token取消由库harness同步checkpoint与小step兜底完成；不以CLI不存在的token flag假装Cancelled覆盖 |

新错误固定消息与优先按§25写断言。数据错误lineNo只出现在消息，不把数据path当error.file。旧readFile同raw输入对照仍保全部字节/CRLF；仅安全小数据比较，不偷改其原UTF8错误类。

## Windows真实关闭证据

删除/rename不是关闭oracle。正式资源probe采用Windows `CreateFileW`、GENERIC_READ、OPEN_EXISTING、share_mode=0排他重开，RAII关闭probe自身handle并记录GetLastError；不改产品FFI。测试先做正控制：无人打开时独占成功；再保留明确打开的reader时独占失败（负控制），避免因权限/坏路径把假失败当锁证据。

最强公开进程方案：LFZ callback写ready标记后等待受控go文件，Python在callback确实进入且LFZ PID仍活时排他open必须失败；发go让callback true/false/可恢复业务错误/非bool或后续数据错误，外层attempt收住后写done并保持进程短暂存活。Python在done且PID尚活时同path排他open成功，证明在**调用返回而不是进程退出**时已释放。每个wait有T17 deadline/step兜底，父进程宽容watchdog只清自己PID；控制文件全在独占Temp，证据包括时序、PID生存、前后probe error/handle状态。

EOF、false、回调业务错/非bool、UTF8、TooLong均应走此方案。全局ExecutionLimit不可attempt吞，CLI会直接退出；仅退出后exclusive成功不足以证明eval返回前关闭，因此终止路径需tooling/独立**同进程库host**在eval返回后暂停退出再probe或受控Drop spy，公开报告明确归属不冒自己覆盖。metadata/open/read失败及NumberOverflow需安全注入，由reader作者unit负责，不生成i64天量文件，不虚构std Drop可回传close错误，不引第三diagnostic。

## 规模与性能分工

本角色小fixture功能矩阵不替代规模测量。tooling/perf按冻结10/100/300MiB同分布数据、固定seed/manifest及Python二次参考重读，LFZ callback只累加不collect；至少100MiB安全readFile同结果对照。生成/hash窗与处理窗分开，新PID峰值WorkingSet/RSS及吞吐只报告最终实际hash/硬件/预热/全部样本，未测不写数字；不把Windows PeakWorkingSet与private commit混称RSS。worker warmup后同进程handle plateau需库host probe，不能把每次新PID的归零当无泄漏。

## 下一步

root可先释放本slot给三owner实施。后续唤醒后按冻结§25落新脚本/prepare-only与旧18d328缺builtin红证据，最终hash后实际实跑/独立验证。现在无测试通过计数、无大文件生成，无编译/产品/旧tests/Git操作；本轮交付后收口不T20。
