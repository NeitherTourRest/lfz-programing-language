# T20 tooling：CLI/REPL与formatter只读可行性review

2026-11-04，基线main0d293b4、候选5d2219。已读启动状态/NEXT、CLI/main/parser/lexer/AST/loader/source/evaluator/input接口。本报告无产品修改、无编译或实跑成绩；建议待architect冻结，不将旧接口当持久会话已可用。本轮T20完整交付后收口，不T21/automation，不替原R304验收。

## 最小REPL推荐

新增显式 `lfz repl`，保留无参数现usage、裸file/run/-e/-/check/test/help/version的行为。源码文件repl.lfz仍按文件处理。REPL不冒充旧stdin整批执行，不在check里增加求值或新静态规则。新增命令和选项要进入既有scanner，CODE/--argv保护及扫描错误顺序不能变。

首版推荐显式cell边界，待裁决空物理行提交、EOF提交最后非空cell；不是靠任意括号计数或错误消息猜完整性。当前parser返回R<Program>，IncompleteExpr只是错误变体之一，不是“所有可续输入”信号；lexer未闭字符串/插值/块注释、缺右括号、函数体等均要正确区分。若使用自动续行，应由core新增只供REPL的Complete/Incomplete/Invalid分类，保持旧lex/parse错误协议，CLI不能复制词法状态机。

空行提交方案需要明确：空格行算否、cell内部空白行是否结束、字符串/块注释中物理空行会否提前提交。若要求这些字面量原样可输入，不能同时用无条件空行当任意文本透明分隔；可改显式提交命令/转义传输或实现真正incomplete分类，交architect选定。不要一边声称支持任意多行，一边隐藏空行限制。:quit/:reset建议只在pending buffer为空时识别，精确命令匹配，不拦字符串中的文本；未完成cell先提交产生原语法错误后才能正常发命令，或者另行冻结丢弃pending规则。

EOF的Ok(0)与空物理行不同：无pending直接结束，有pending只提交一次（不要求末LF），语法错只报一次，不无限续行。read错误/invalid输入UTF8等宿主输入失败应有限结束，避免重试坏stream无限循环；普通lex/parse/runtime错误释放本cell资源并允许下一cell。退出码是累计cell失败影响会话还是仅宿主失败影响会话需冻结，不能默认沿一次run错误马上exit。

建议自动显示最后非nil结果（沿既有Value显示）作为新REPL协议，print/;;副作用仍由程序打印；nil和声明的输出应明确。该策略应同时适用于TTY与pipe，避免同输入仅因终端产生不同业务结果。或者冻结只显式print的更小协议；不能实现时临时选。REPL --json建议首版usage拒绝，避免多cell多JSON对象冒充旧唯一对象协议；若未来做JSONL，应独立版本化协议。

## TTY与stdin/input冲突

Rust std::io::IsTerminal在1.70已稳定，本项目既有Rust工具链足够；可在main检测 stdin/stderr，向CLI传明确交互模式使内存测试无需伪装真实TTY。建议仅stdin和提示目标stderr均TTY时显示固定主/续行提示，提示写stderr并flush，pipe无banner/提示/ANSI。未知终端或检测错误按非TTY。实际PTY提示需独立真实PTY验收，管道测试不能冒充。[IsTerminal](https://doc.rust-lang.org/std/io/trait.IsTerminal.html)

现b_input自行stdin().lock().read_line，并输出可选prompt。REPL不能在主线程持StdinLock跨大栈worker eval：input会等主线程锁，主线程等eval，可能死锁。也不能给REPL另包BufReader私有预读后再让input读底层stdin，否则下一行可能已在REPL缓冲，出现吞行/错序。std Stdin本身共享全局buffer且各handle读取协调；推荐REPL逐行借用同一std stdin读取，返回后释放锁，再执行cell。不将测试用Cursor输入接口误称可被b_input读取的来源。[Stdin](https://doc.rust-lang.org/std/io/struct.Stdin.html)

若保留input，精确规定eval期间它消费同一源后续物理行（包括原本看起来像命令的行），REPL不能预先read_to_end/排队全部cell。pipe测试须cell提交分隔后紧跟input数据行，再继续下一cell，确认只消费一行。TTY中按用户直觉等待程序输入。也可禁止REPL input但需专门runtime契约，不能偷改全局旧input。建议保留input并明确消费语义；EOF沿现input IOError，之后REPL遇EOF结束。Ctrl-D/Ctrl-Z沿OS EOF，Ctrl-C沿现OS默认，不在本轮暗加跨平台handler。

输入读取须在累积前有明确cell/物理行上限，不能先read_line无限长行再判断。使用固定小块从共享Stdin借锁扫描并及时释放，不能额外私有预读缓存越过eval边界；借用std本身共享缓冲中的未消费部分是可行路线。空行cell/超长输入丢弃后的resync与EOF错误策略需冻结。[BufRead语义](https://doc.rust-lang.org/std/io/trait.BufRead.html)

## 持久会话必须runtime实现

现eval_module_traced_on_thread_with_args_and_context每次new Interp，返回时提走trace/func_names/diagnostics；不能直接多次调用获得persistent env。CLI必须消费runtime新session API，不能拼接历史源码重跑（会重写文件、重复HTTP/exec/print与模块初始化），也不能在CLI直接修改私有Interp/Env。

整个REPL/session创建、每cell loader/lex/parse/eval以及历史AST/环境析构应位于同一个已有大栈wrapper中，避免把非Send Rc来回跨线程，避免每次只包eval留下parse/drop在小栈。Stdin交替读/执行也放同线程，锁每次仅覆盖读取，不跨用户eval。跨cell函数体/闭包由runtime稳定持有，不允许引用被CLI释放的临时Program/Loaded。

语言错误不回滚此前副作用/成功绑定/容器mutation；新cell parse失败不能产生求值副作用。运行时错误后清理trace/call_depth/当前source/active module等临时状态，不能通过new Interp清空所有已有绑定来假装继续成功。重复let/var/fn/struct声明的覆盖或拒绝沿明确会话规则，不能擅自做新的shadow/reset语义。

:reset销毁旧运行环境/闭包/模块缓存/来源记录并新建会话；host配置的argv/授权/预算策略建议保留，不因reset自动新增权限。若Rc可构成用户环，不能把drop新session称完整GC/所有任意脚本内存立刻返OS；需要长会话测量和明确限制。

## 来源、模块与预算

loader::load_source可沿REPL输入无#42、归一LF的语义，不能为每cell加文件前导。为各执行cell分配独立display_path（如<repl:3>）与SourceMap Loaded，span本cell从1起；旧cell定义函数在新cell出错应显示旧定义来源和新调用来源，不能全部渲染成当前cell文本。现SourceMap仅存来源元数据，attach_origins/逐帧来源与func_names应由runtime跨cell一致维护；func_id不能每cell重置撞旧函数。错误快照不能取走仍被后续会话需要的全部名字表。

当前单Program eval会preflight拒模块声明；已有graph执行入口面向一次封闭图，不是动态cell增量模块缓存。REPL import/export、相对路径基准、重复初始化、跨cell引用模块及reset缓存策略必须先冻结。首版可以受控拒绝模块声明并明确限制，但不能假装已有graph入口可无改造拼成持久REPL，或执行前面print后才发现整cell模块不支持。

--allow-process/http建议session启动时独立显式授权，旧默认false，各cell不可修改host权限，:reset不放宽；argv建议启动时固定快照，是否接--ARG需scanner冻结，不将REPL元命令混成argv。limits推荐每cell新max_steps计数及前端成功后新deadline，等待用户输入不计时间；同cell callback/HOF/native共享预算，不能用下一cell操作恢复同cell资源错误。ExecutionLimit不可attempt吞仍保持；是否host可继续下一cell属于独立边界，需明文允许。host取消token已置位不能reset清除或复制为新false绕取消；建议结束session或每后续cell继续受控拒绝。

native资源收尾和两diagnostics门禁继续检查；不能因REPL持续运行而放过HTTP/process回收失败，建议host完整性失败有限结束会话。T19文件在callback内结束/错误时仍关闭；EOF/:quit/:reset在大栈释放会话。长会话既有func_names/SourceMap可能按cell/新函数线性累积，需测1000cell及reset前后资源；若source/cell设cap须先冻结，不静默丢旧source造成trace画错。流式输入不代表会话恒定内存。

## formatter独立评估：建议本轮有依据暂缓

现lexer line_comment/block_comment不产comment token，空白被跳过；AST只起始Span，无完整trivia/lexeme范围。管道已脱糖为Call，整数存数值而非原字面拼写，字符串等也会归一；所以从现AST pretty-print不能恢复注释、管道原文和用户风格。重排换行还可能触发TwoStatements/块注释换行等真实语义差异。

建议T20明确暂缓生产formatter，不增伪format命令、不覆盖源文件、不篡改check。若将来实施，应独立定义lossless token/trivia表示、注释附着和稳定布局，再做幂等format(format(s))=format(s)、lex/parse行为等价与真实副作用程序差分；保留#42/BOM/CRLF策略另定。仅AST结构相等或删注释后parse通过不足以证明保程序行为。此取舍不影响最小REPL与新版Agent任务包交付。

## 验收闸门建议

先固定pipe字节级oracle：多cell持久var/function/closure、每cell副作用恰一次、多行、EOF末无LF、错误后续cell、input紧邻cell/EOF、reset/quit、不同能力与限额、资源错误不可attempt吞、旧函数跨celltrace、预flight模块限制、启动usage与旧run/check/JSON完整差分。再同逻辑测真实PTY提示/flush，不拿mock IsTerminal当系统TTY通过。

特殊长会话单独测累计cell/函数/source元数据与reset后资源、HTTP/exec/T19回调错误清理；禁止用一串全成功cell掩盖失败恢复状态问题。宿主输入错误/输出写入失败应有有限失败路径，非恶意超长读无界。全过程外watchdog、项目Temp，测试脚本不公网/外装，旧样本不覆盖。当前仅可行性review完成，无T20产品或性能成绩。

补充：即便每行lex/parse已经Complete，也不代表用户希望立即提交，例如if块后下一行else、函数声明后继续一段同cell代码。若用自动完整性来决定执行，须定义明确提交意图/续行规则；显式cell边界可以保持每cell只parse/eval一次及副作用恰一次，避免任意前缀重复parse带来的长cell二次成本。
