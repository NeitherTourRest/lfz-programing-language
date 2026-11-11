# T20 REPL、开发体验与Agent任务包提案

日期2026-11-04，language-architect。仅提案，未冻§26/源不GO。基线root报告main0d293b4/T19最终5d2219/1041920B；完整启动读AGENTS/STATE/BOARD/自身/NEXT和实际loader/source/call_user/TemplateContext。当前仅T20交付后收口，不automation；R304OPEN，原用户28/1/1保护。

## 1. 可交付范围与关键架构
最小REPL持久变量/闭包/方法/模板、多行、EOF、失败后继续、真实cell诊断、reset与受控元数据。Formatter单独可行性评估，check不新增未定义静态规则；人/AI指南及新版本Agent任务包实测独立交付。不能用重复拼接旧cell源码重新执行冒充持久状态，旧print/IO不得重放。
推荐单ReplSession拥有唯一Interp、一个共享globals和累计source/name表。每个已提交cell独立解析、仅执行一次；旧变量与新声明沿现Env规则（含let可否重定义的原行为），不新造REPL作用域优先级。MVP import/export整cell执行前拒绝，不构ModuleGraph、不IO导入，不把声明忽略；旧run/check模块功能不变。不是永久放弃模块，后续另审sharedglobals与graph session衔接。

## 2. 真实来源与旧定义调用
每cell加载用load_source，不要求#42，行号从1；稳定显示路径<repl:1>、<repl:2>…。不管过去cell行号多长，都不能拿当前cell文本画旧函数span。
建议每个可执行cell建立内部ModuleInstance槽，home ModuleId只映射该cell source和同一个共享globals（不是独立隔离模块）。UserFn整数home不变、每次call_user沿原source映射；graph模板Some TemplateContext在REPL启用，默认表达式定义home/captured/self与独立defaults诊断frame受原Recursion保护。REPL模板默认值因此采用定义上下文，须明确与旧无模块run legacy None/caller默认的区别；若review发现共用映射风险，可另内部home-source表，不许改UserFn公开shape却漏迁全部构造。
模块顶层、旧函数、新函数、模板default每帧source真实；SourceMap保每cell Loaded文本，TracedRun.attach_origins校验不回退到当前cell。编译错误虽没有Interp执行也返回当前cell Loaded/精确LzError，不造运行trace。不同cell source与func_id单调、不复用；输出诊断后可释放outcome，session仍保旧定义需要的source。SourceMap Rc snapshot在后续插入如需clone-on-write必须计量/受限，不称整会话O(当前cell)内存。
历史源保留直到reset（不能以“函数可能不用”提前删）；结果对象保留旧来源snapshot能在reset后独立诊断，不持session Env强引用。

## 3. 多行、命令和EOF
建议core增加结构化probe，不以UnexpectedToken中文message匹配EOF、不写另一套脆弱括号计数器：
    ReplParse { Complete(Program), Incomplete, Failed(Box<LzError>) }
probe使用同lexer/parser状态，Complete只表示当前cell已经完整合法，Incomplete仅真正可追加文本修复的EOF上下文（开括号/方括号/花括号、块注释、合法插值嵌套等）。裸字符串遇真实LF仍旧SyntaxError，不自动发明跨行字符串；非法字符/缺失操作数而LF不能续接/深度超限仍Failed。Incomplete不执行、无side effect。
CLI按输入行追加pending，每行后probe；Complete立即提交一次，Failed报告并清pending后继续。若if完整块已经Complete就执行，后行else是新cell非法；用户须在尚未闭合块/括号中组织完整if/else或同一输入行提供闭合及else。不能无限等待“可能未来还有else”，该限制须help/指南示例清楚。空白/仅注释Complete无语句建议静默忽略，不消耗历史source槽；空行在Incomplete内只是输入，不强制执行。单行末尾语法已完整即执行，后续单独一行不能追加旧已执行表达式。
EOF无pending正常退出；有Incomplete则执行同lexer/parser最终失败诊断一次、丢弃pending并exit2，不执行半cell。无“每遇EOF都试图补括号”。I/O非法UTF8/读取失败CLI宿主错误，不作为可继续cell。
仅空pending时识别整行trim后的:help、:reset、:quit；未知冒号命令报命令错误/继续，不送parser。pending内冒号文本按普通LFZ，不能截断字符串/注释。:reset释放旧globals/源/name表/状态，保启动权限/args/预算配置但不reset共享host cancellation token；cell编号建议重置1（新session），输出已脱离session的旧诊断仍自身持来源。:quit忽略后续输入并正常退出。
无需行编辑/历史文件/颜色/第三方依赖，std BufRead+IsTerminal。TTY stderr主prompt“lfz> ”、续prompt“...> ”，stdout只表达式结果/程序输出；pipe无prompt/欢迎语，便于脚本化。成功最后值非nil显示一行Value::Display，nil静默；显式print仍原效果。show value不是JSON编码，用户业务JSON用print(jsonStringify(...))。:help内容stdout仅显式请求，:reset默认静默。
CLI错误后继续且记录本轮是否有未捕获cell/命令错误，EOF最终退出2（任一错误）或0；reset不抹已发生CLI错误退出记录，:quit沿同累计退出码。host cleanup诊断污染立即停止执行后续cell，允许TTY用户reset/quit、pipe建议直接host失败exit2，不能自动吞异常继续运行。
命令入口lfz repl，执行能力/限额选项同现执行区，独立--allow-process/--allow-http默认禁用。首版拒--json（“repl 不支持 --json”）以保持旧单次执行单JSON协议；不默认为JSONL。--后argv允许并跨cell共享同一快照，不消耗REPL输入作为argv；脚本input()会与REPL同stdin竞争，首版建议明确禁用input于REPL或document共享流，需root裁决，不能假保证互不抢行。

## 4. 恢复、预算与session终止
每cell开始建立干净顶层frame/active，保globals/functions/captured cells。正常/未捕获LFZ错误后先将真实fault trace/source/name/诊断转outcome，再清理depth/trace/source/fault/active/globals到session idle，不回滚变量部分写入/容器/IO/RNG/func IDs。失败cell中先定义的函数可被下一cell用到，source必须保留。编译失败无执行状态变化，但可记source/限额消费。session边界恢复是host继续，不等于attempt可捕获全局终止，脚本仍不可吞ExecutionLimit。
推荐每cell max_steps预算独立，CLI timeout每次从cell求值开始建立；不因多行输入等待耗执行timeout。host绝对deadline/token仍外部传入，token已取消后不得偷偷清零。独立cell是不同显式求值任务，不能把“每cell重置”宣称连续脚本总预算；嵌套attempt/forEachLine/exec/http仍共该cell额度。参数能力启动快照，cell不可改权限；:reset不扩大能力。
全局资源终止后下一cell是否允许：建议允许显式新cell并新steps/相对timeout，但host已取消token/过去绝对deadline会持续拒绝。不能因REPL host选择继续而放松attempt白名单。cleanup diagnostics污染则当前session标为不可继续（仅reset/quit），两sidecar标题/双门禁不变，不把unsafe宿主异常当普通可恢复LFZ业务错。
使用单大栈线程包住完整REPL循环、在该线程创建/持有/析构Session与各Program，不每cell把Rc/Env跨线程移交，不扩unsafe Transfer假设。宿主ReplSession公开对象须!Send/!Sync，要求在on_eval_stack作用域内用/析构；不新增自动转移session到worker的隐式API。

## 5. 长会话上限与推荐API（待review冻结）
推荐固定限制：pending/raw单cell<=1MiB；累计保留源文本<=16MiB；最多4096已提交cell；最多65536累计func/default诊断ID；单次仍沿parse/AST/Recursion上限。空白/注释不占cell；语法失败的非空提交算cell/保留源，防无限错误输入撑表；pending不完整受1MiB。触限拒本次提交、无本cell副作用，提示:reset，不能悄悄删旧源/重置ID。
funcID在cell执行中可能新分配，需安全边界预先保留/分配时检测；触限是宿主session限额而非伪Recursion。具体carrier建议ReplHostError::Limit{kind,limit}，发生midcell则保前面副作用、回收当前frame并锁session至reset，不能要求无所有部分效果。source/ID admission各自必须明确原子性。
推荐runtime公开ReplSession（字段私有）、ReplConfig（args/能力/max_steps/timeout/hostdeadline/token配置，避免暴露未来扩展字段需builder）、new/config/reset/eval_cell；cell评估返回独立ReplOutcome含当前Loaded/源身份、TracedRun或前端错误/HostError；front parse probe由core提供。精确Rust enum/所有权待runtime review，不在提案假冻接口。现script函数/Value/Span/LzError/Result键不改。限额类不必新增第14LFZ class，host limit不得通过attempt伪装业务错误。
core唯一parser/lexer结构化Incomplete API；runtime唯一session/evaluator/source映射及模板；tooling唯一CLI loop/render/prompt/参数/helper测试。SourceMap或错误枚举如需小增量由core单写者，三方错峰。

## 6. Formatter/check/Agent任务包
formatter本轮只独立评估保注释/空白/分号dump/插值/可选换行语义/幂等可行性；lexer当前不保全部comments trivia，不能用AST pretty-print直接宣称等价formatter。若需大改token/CST，输出评估与后续边界、不偷偷加格式化命令；用户T20最小REPL仍可完成。check不扩动态查名/跨cell推断，不加未冻静态规则。
AI新任务包建议独立v3（或从当前已存在版本另选未用标识），先冻结输入/评分/预算/模型条件再实测；旧v2及成绩不覆盖。覆盖REPL多cell持久/错误后继续/跨cell诊断/限额reset、模块/JSONL/http显式能力实用编排；网络测试只loopback或禁用见证，不依赖公网。指南公开持久globals/默认值home和错误恢复不是事务，避免Agent把失败后状态当清空。
实际Agent任务输入/评分由ai-dx owner，要求与旧v2比较只能同条件，新增任务成绩不能追溯填到旧版本。人指南/skill/实际应用演示更新与独立操作实测同最终hash交付。

## 7. 需root裁决及验收
优先裁决：MVP禁import/export/sharedglobals逐cell home；结构化probe及if/else完整cell边界；每cell预算与resource恢复策略；stdio input冲突；限额数值/host锁session；--json拒绝/TTY stderr与pipe格式。review后冻§26与精确owner ABI再GO源。
必测旧cell打印/IO只发生一次；变量/闭包/self/default跨cell与shadow；失败前定义仍可用/容器部分写保留；过去cell函数/default错误从真实旧文本画caret，reset后旧outcome安全；多行所有括号/comment/插值/非法字符串/EOF/错误后继续；命令只idle识别/pipe稳定/argv/权限四组合；steps/timeout/cancel不被attempt重置/hostdiag锁；cell/source/ID上限及reset不真实耗尽内存。
本轮全旧脚本/CLI/check/模块/进程HTTP/JSONL回归，独立验证及新Agent包真实记录；原19项/R304仍OPEN，不能凭T20完成宣称全部正式发布。无源/编译/Git/外装/公网。

## root/tool review裁决补充（2026-11-04，优先于提案旧建议）
保留input()既有语义，不禁用、不新增ReplInput错误。REPL整会话同on_eval_stack，每次用stdin物理行读取并释放StdinLock后再eval，不在eval跨持锁、不另套预读BufReader。执行中input读取同stdin下一物理行，若内容是:quit也仅作为脚本数据；input既有提示仍stdout，REPL自己prompt stderr。脚本消费了多少行不再重放到REPL。库由调用方负责stdin与源输入分离/共享边界。
formatter本轮有依据暂缓并由tooling出独立评估报告：lexer丢comments/trivia，AST脱糖且丢数字原文，不能伪formatter。core结构化probe继续review；若正确性成本过高，可另批准显式blankline cell边界，不可中文错误猜Incomplete。
退出码固定本进程任一cell/命令错误最终2（TTY/pipe同），reset不抹错误累计；仍可继续下一cell。其他批准主体见root消息，精确ABI待review后§26。
