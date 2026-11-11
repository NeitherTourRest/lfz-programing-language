# T20 冻结契约
## 26. T20 REPL（2026-11-04 冻结）

root已批准主体/stdin/tool评估及ReplCapacity终止方案。仅本轮交付收口，不automation；格式化器本轮只出有依据的评估，check不加未冻静态规则，R304OPEN。旧脚本入口语义/错误JSON/Value/UserFn公开shape不改。

### 26.1 CLI与完整cell边界
新增lfz repl，支持原限额/--allow-process/--allow-http选项及--后argv；仅新模式，旧调用不受影响。repl不带额外来源；CODE/argv保护沿原scanner，--后文本即参数不作选项。--json明确usage“repl 不支持 --json”，不输出新JSONL协议。各能力默认禁用，args快照跨cell；:reset不能增权。
TTY stderr主prompt“lfz> ”续prompt“...> ”，及时flush；pipe无prompt/欢迎语。每行追加pending，结构化probe Complete即执行一次新cell；Incomplete继续读，Failed报错清pending并继续。无中文message猜测，不重复执行旧cell。Complete空白/仅注释静默不占历史。单条完整if块立即执行，下一行else不是上一cell；用户须同cell组织else（闭合花括号与else同一物理行），指南明示。
成功最后值非nil用现Value::Display一行stdout；nil静默，print返回nil不重复。program print/input原stdout行为保持，REPL prompt独立stderr。命令只在空pending识别整行trim为:help/:reset/:quit；unknown冒号命令stderr“repl: 未知命令”计失败继续，pending内冒号是普通源码。:help stdout固定帮助说明命令/多行/ifelse/stdin/limits；:reset静默重置会话；:quit停止读取并按累计错误码退出。
stdin每次物理行读取释放StdinLock后再eval，不另套prefetch BufReader跨eval。input()旧语义不禁：消费同stdin下一物理行，:quit被其读到也只是数据，不再重放到REPL。整个循环在同一个on_eval_stack，Session/Program在该线程持有/析构，不每cell迁移Rc。
EOF无pending正常结束；pending Incomplete用正式lex/parse生成一次真实失败诊断并清pending，exit2，不补括号/执行半cell。读取UTF8/IO错误宿主失败exit2。任一cell/命令失败累计最终exit2（TTY/pipe同），其余0；reset/quit不抹累计失败。无输入EOF0。

### 26.2 core结构化probe（core唯一lexer/parser）
公开parser::ReplInputStatus {Complete,Incomplete}（Clone Copy Debug PartialEq Eq）；
    pub fn probe_repl_input(text:&str)->R<ReplInputStatus>
沿load_source归一化再同lexer/parser探测；Complete指当前输入已完整合法（空亦可），Incomplete只可通过追加物理行完成的EOF状态：尚开合法括号/数组/块/块注释/插值上下文，或parser真正期待EOF后可续的语法单元。非法字符/不匹配关闭符/已出现致命非法token/深度上限直接Err；不能仅因为存在未闭括号就隐藏先前错误。LF结束的裸字符串仍旧非法，不新造多行string；旧规则不许LF续的缺操作数仍Err。EOF Unclosed状态由内部typed标记，不匹配中文或format后token名；旧lex/parse签名和正式错误不变。
probe只看新pending，不保全历史重parse，不执行。返回Complete后session可正式lex/parse一次（probe是判边界，不是第二次执行）；所有AST析构同大栈。core必须实际覆盖fn/if/loop/struct、嵌套注释/插值/字符串/括号、错误关闭及EOF，不实现不可靠探测fallback。若发现无法安全实现，先向root报具体反例，不静默换blankline协议。

### 26.3 runtime owned session API
evaluator公开：
    #[derive(Clone,Debug)]
    pub struct ReplConfig {pub context:EvalContext,pub args:Vec<String>,
                           pub timeout_ms:Option<u64>}
Default等价EvalContext::default()/空args/None。timeout_ms为每cell相对毫秒，0允许；context.limits.deadline是host绝对截止，两个同时存在取较早；token原Arc不reset。max_steps每cell从配置重新建额度。超时从cell完成parse后求值开始建立，输入等待/parse不算相对求值窗。host绝对deadline仍含host经过时间。Instant checked_add失败是InvalidConfig，不伪LFZ业务错。
    pub enum ReplHostError {
        InvalidConfig{reason:String}, Capacity{kind:String,limit:usize},
        Poisoned, Origins(SourceMetadataError)
    }
Debug/Display/Error；Display分别reason、“REPL 会话容量超限：{kind}（上限 {limit}）”、“REPL 会话需要 :reset”、沿来源元数据Debug说明。
    pub struct ReplSession { /* 私有，!Send/!Sync */ }
    pub fn ReplSession::new(config:ReplConfig)->Result<Self,ReplHostError>
    pub fn eval_cell(&mut self,text:&str)->Result<Option<TracedRun>,ReplHostError>
    pub fn reset(&mut self)
None仅正式lex/parse后无语句的空白/注释cell；不消耗历史。TracedRun含独立来源snapshot/func_names及原双diagnostic，compile失败为Err结果/空运行frames/当前cell error_source；runtime失败为真实frames。host error无伪LzError。CLI host errorstderr前缀“repl: ”exit计2；session poison后只准reset/quit/help，不执行后续cell，pipe遇poison直接host失败结束。
Session要求整个生命/析构在on_eval_stack作用域，不新增worker迁移/unsafe Transfer。旧函数入口保持。ReplConfig公开新类型无历史literal，能力仍启动固定；reset重建Interp/元数据保配置（含host deadline/token，不偷偷清取消）。
单Interp/sharedglobals，let/var/fn同名声明/闭包globals查询完全沿现Env，成功/失败前新声明与容器/IO/RNG效果都保留。每cell只执行新Program；MVP import/export在整Program任何效果前SyntaxError，新增SyntaxMsg::ReplModuleUnsupported消息“REPL 暂不支持 import/export”。不构graph或忽略声明，run/check原模块不变。

### 26.4 source、home与恢复
每非空正式提交cell显示<repl:N>，N从1、reset重1；每cell Loaded用load_source、line_base0，不要求#42。source记录保至reset，compile/runtime失败cell也保留，不能用当前cell文本画旧函数caret。
内部每cell唯一home ModuleId/来源映射，所有槽globals指同一共享Env；复用现call_user切home/source机制，不假建独立隔离模块。不新增UserFn公开字段；REPL模板default_context=Some，定义home/captured/self/独立default frame受原Recursion，旧run legacy None语义不变。来源身份数字与ModuleId分域，不猜相等；函数/方法/闭包及default出错各用真source。
每cell开始新顶层frame，结尾先保存真实fault/name/source/双diag到outcome，再恢复depth/trace/trace_sources/fault/active/globals为session idle；保next_func_id及用户状态。未捕获ExecutionLimit也可由host开始新显式cell，脚本attempt仍不能吞；已取消token/过期host deadline会继续终止。禁止把恢复宣传成事务回滚。
双cleanup诊断污染锁session只reset/quit/help；旧process/HTTP文案与门禁保持，不因新cell清空历史污染后继续。旧outcome自己持来源snapshot，reset后可诊断；不要强持Env或已移除Session。SourceMap clone-on-write/func_names snapshot的复制成本要如实测，不宣称恒定历史开销。

### 26.5 长会话限制与第4资源原因（core/runtime）
单pending/raw cell最多1048576bytes；累计raw已提交源码最多16777216bytes；最多4096非空提交cell（含失败）；累计func/default诊断ID最多65536个（不计MODULE0）。空白/注释不占历史，但pending原始bytes仍受1MiB限制。source/数量admission在执行前原子检查，超限ReplHostError::Capacity kind为"cell bytes"/"source bytes"/"cells"，本cell无效果、不清旧源/不重放。source表与home槽只在admission成功后追加。
funcID分配时可midcell达到上限，不能无限表/回绕/以可attempt ValueError吞掉。ExecutionLimitReason新增ReplCapacity第4原因，class仍ExecutionLimitError（13类），固定message“REPL 会话容量已耗尽”，调用/定义当前span，仍不可attempt；仅REPL启用，旧三个reason/message保持。穷尽Rust match机械补此variant、strict注册新增原因断言而不弱化旧测试。触限后session poison直到reset；此前cell部分副作用保留、fault保存后清执行状态。新cell是否覆盖旧next ID不得使其复用，不回滚ID。
source/cell admission失败可输入更小cell若仍能容纳，不一定poison；midcell func容量与cleanup必poison。pending超过1MiB必须追加前有界检查：CLI用共享stdin.lock().fill_buf/consume小块，只累积到物理LF或剩余cap，不std read_line先吸无限行，也不另BufReader；eval前释放锁，未consume的共享缓冲仍供input。观察到超cap即前端受控Capacity并结束REPL exit2，不无限drain、不把余字节当下一cell；未进入正式cell不占source槽。宿主eval_cell对已提供str也先检查raw cap。

### 26.6 owner、评估和验收
core唯一parser/lexer typed probe与error.rs ReplModuleUnsupported/ReplCapacity消息；runtime唯一evaluator ReplSession/配置/恢复/source/home/func容量及任何必要value消费（不改UserFn shape）；tooling唯一CLI行循环/扫描/render/TTY/EOF/exit/formatter可行性报告。SourceMap若需append API core单写者；避免两个owner改同文件，root指定唯一编译。
formatter本轮有据暂缓：lexer丢comment/trivia、AST脱糖和数字原文缺失，评估报告列失真例子/后续CST成本，不做伪formatter/不加未批准命令。check不加动态查名/未冻静态规则。
AI新任务包独立新版本（选择未占v3或后继标识），先冻结输入/评分/资源/模型条件后实跑，旧v2输入成绩不覆盖；覆盖cell持久、错误后继续、历史来源/limits/reset及现模块/JSONL/HTTP显式能力编排。新任务不同条件不冒旧成绩直接可比；人指南/skill/应用演示与最终hash记录owner实测。
必须测旧cell print/IO只一次、let/var shadow/closureglobals沿旧、失败前定义仍可用、部分容器写保留、早cell函数/default真实文件caret、reset后旧outcome；多行每合法与非法EOF/注释/字符串/插值/ifelse限制、命令仅idle/pipe无prompt/input取下一行/argv/权限；每cellsteps/timeout/hosttoken不reset、attempt不可吞ReplCapacity、双diag poison；1MiB/16MiB/4096/65536真实或安全注入边界（不真OOM），reset清表不增权。
同最终hash旧全部run/check/CLI/module/exec/http/JSONL/apps回归、独立REPL/新Agent包实测；无性能结果不得关闭原19项/R304，不宣称T20完成即正式发布。
