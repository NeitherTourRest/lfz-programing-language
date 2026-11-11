# T20 Agent测评任务包提案

2026-11-04，ai-dx-engineer。状态：**设计提案，未冻结题包、未启动学生、未运行题目**。基线main `0d293b407cc653e4c8249d2f64561c30a0ea5ab3` / T19，76内置/13类错误；正式执行须绑定T20最终实际产物和同步后的指南SHA，不能拿T19字节宣称T20已通过。

依据：NEXT-ITERATIONS.md、三spec §18–25、当前四skill及AI入口、旧v2 RUN_PROTOCOL/run-metadata。§26尚未冻结，本提案不定义REPL语法。旧首轮2/8与v2八题8/8、131case及原始ZIP/manifest只读保留，新题不冒同题对比或提升百分比。R304OPEN，测评不等于效率门槛或正式发布。

## 建议范围与版本

新目录拟用`docs/reports/agent-benchmark-v3/v3-t20-r01/`，学生workspace拟用`Temp/agent-benchmark-v3/v3-t20-r01/C01..C04/`。本阶段不创建正式包。四任务均需新会话、无项目历史；本设计者及参与审题者不能充当学生。题型避开旧v2八道算法题及既有应用逐字重写，检验组合接口开发过程，而非背题。四个学生各一题，不把一个已学会LFZ的学生连续做四题称独立盲测。

每题提供相同五份AI文档快照、同一解释器字节、任务说明、少量公开输入；不提供产品源码/历史应用/报告/参考解/oracle。提供纯任务workspace、相对文件结构及数据argv，不暴露仓库根路径给任务提示。共享机器仍可能访问其它路径，这是提示和审计范围隔离，不是OS沙箱；有越界记录则协议失败或UNKNOWN，不因自述“未访问”就判已强隔离。

REPL采用独立作者/功能验收轨道：§26冻结后更新指南与脚本测试多行、EOF、错误后继续、持久上下文、cell来源，不混入此四题功能分母。若team-lead决定新增REPL学生题，须审题并冻结新revision及分母后才能派发，不能看到学生成绩后增删题。

## 四个候选任务（题面、oracle和源码约束设计）

### C01：分批事件清账

任务输入argv为JSONL路径和输出路径。每行合法业务记录为含非空string account、int delta的JSON object；其它值、缺字段、坏JSON、空行各记rejected，继续后行。输出各成功记录的归一化JSONL（account/delta/原始lineNo），另向stdout打印summary accepted/rejected/sum；不要求按账户聚合，避免隐藏O(账户数)存储。批大小固定32，末批flush，输出预先清空，不把全部记录collect；输入单行cap4096，先check后限额run。reader错误应终止而非伪装坏JSON跳过，已写批次不回滚。

评分oracle：独立Python按原始bytes用LF分行，CRLF仅剥一个CR，尾LF不多行；严格UTF8与cap后解析JSON，再执行明确业务schema（bool不是int）。公开样例仅少数；冻结隐藏输入包含负/零值、转义Unicode、重复account、最后无LF、连续坏JSON后成功、空文件、33/65条跨批、恰cap与cap+1、坏UTF8。对比解析后对象与顺序，不拿JSON键序作业务差异；rejected不包含尚未交付的reader坏行。所有测试输入在派发前生成并固定，不临时随机构造追绿。

源码约束：用forEachLine，禁止readFile整读输入、split整文件、全部记录数组；解析closure在外一次创建，struct cell更新后attempt(parseCurrent)，callback外一次定义，不逐行创建UserFn。批数组最多32，只count/sum保留常量状态；输出可appendFile。源码审计与动态跨批oracle共同评分，不能只凭small input功能正确认证内存约束，也不声称本题已量OS峰值。

### C02：跨模块报价与共享历史

学生提交main.lfz、lib/rates.lfz、lib/view.lfz。rates导出更新/报价函数、标量rate快照和共享历史array；view经不同相对路径复用rates，入口也导入同一rates。任务先冻结操作序列：入口同名rate不污染定义模块函数，namespace标量赋值不改模块binding，函数更新改变后续报价但旧namespace标量仍快照，共享历史可见，菱形模块初始化仅一次。argv文本含Unicode和flag样式并在模块内读取；check不执行初始化marker写入。

oracle：以题面完整状态转移表计算每步报价/快照/共享历史/argv，匹配JSON业务输出与marker次数；从不同cwd运行，import仍相对文件而数据路径按cwd。增加一个合法导出函数故意在定义文件fail的输入，核CLI真实file/trace，恢复后另场景核无旧故障帧。不要求attempt error含file。封存时必须把操作顺序、输入整数范围、导出名称及错误预期逐字落TASK，避免“实现任意等价库”与具体测试冲突。

源码约束：真实多文件import/export，禁止单文件硬编码预期、复制两份rates模拟缓存；方法体不能用caller数据伪装home globals。无模板/未冻结语法要求。独立审核真实模块结构与指令读取范围。

### C03：可恢复记录与不可恢复预算

给小型JSON字符串数组数据文件，编写runner.lfz，逐条attempt解析/业务检查，产生固定ok/value/error摘要，连续失败后仍接受好记录，处理前修改共享cell并证明错误不回滚。数据使用C01不同schema（job/id/enabled），不复用其学生解。再提供受控模式进入真实有AST的无限while，入口外包attempt；需交commands.json记录check和两种run命令，限额严格两token、数据flag经--保护。

oracle：成功nil与失败区分、错误class/message与已改cell正确，不能只看exit0。预算模式必须在规定--max-steps下exit2/ExecutionLimitError，不产伪成功summary；自调允许学生观察自身结果。检查CLI JSON stdout单对象与print业务stderr通道分别，不要求机器JSON含预算统计。外watchdog是测评宿主兜底，不算语言预算成功；取消token/硬实时不在本题宣称范围。

源码约束：attempt真实执行，不能直接构造所有Result伪造错误；至少一次连续错误后成功。只捕业务白名单，不能改产品吞资源错误；不能拿空for制造预算耗尽假设。无需学生实现宿主取消API。

### C04：本地响应采集器

测评宿主启动127.0.0.1临时端口服务，argv传baseURL与资源名数组文件。学生顺序GET三类资源，输出每项status/body/repeated header列表；404是普通响应，302仅记录不跟随，坏UTF8为可恢复错误并继续下一项。先无授权运行证明默认禁用，再独立--allow-http带局部timeout/response cap与全局限额执行；所有请求由LFZ httpRequest完成。端口、header可变值/测试序列在服务fixture中确定，动态port在dispatch前绑定审计，而不是冻结一个可能被占用的固定端口。

oracle：业务输出之外独立请求日志校验method/path/顺序/次数，重复头保序，redirect target不被访问、无额外重试；禁用阶段服务日志零请求。API响应cap给宽裕固定值，不猜wire精确字节N；本题不测TLS/网络公网/timeout毫秒性能。服务finally shutdown/server_close/join，关闭状态记录。需要正常OS网络许可时在学生前统一验证环境，默认权限失败保存，禁止只给失败学生事后额外自调次数。

源码约束：不能借exec/curl/PowerShell/Python发请求，不能读服务日志/fixture实现/oracle。URL仅所给loopback；单线程服务策略/动态字段与大小边界须审题落实，避免环境偶发失败当学生错。Windows功能范围，非Windows未验不外推。

## 相同执行预算与反馈

建议每题20分钟wall、最多40次顶层工具调用、最多5个自调周期，每周期最多1次LFZ check加1次LFZ run，合计至多10次产品进程。读/写/工具发现均计顶层调用；一次batch也要按内部实际LFZ启动计进程数，禁止循环启动绕预算。所有题有相同shell/读写接口，既能写多文件也能跑允许命令。固定one-shot dispatch输入，每题开始前记UTC deadline，再创建/派发；同并发上限建议串行或最多2，记录实际排队不静默补时。

预算是待team-lead确认的提案，正式冻结后不随难题调整。学生可以看自己的公开样例/自调输出，不能获取隐藏验收反馈；父agent不提供语法修复或解题提示。交卷立即快照提交目录及commands.json，验收在新目录运行封存源码，评分失败不回发本轮继续修。越时/超工具预算/缺交卷保持分母4并分类；API平台故障记INFRA/UNKNOWN，不自动PASS或偷偷换学生取最好成绩。补测须新run-id，旧run保留，不替换首轮。

同模型/effort使用当前获准配置，不为本提案自行改模型。dispatch请求值、宿主返回值、session turn_context实际值分别记录；若不提供真实model/effort就写unknown，不凭继承猜。缺完整工具trace不能宣称协议完全认证。温度/token用量仅有实际provider数据才填数值。20分钟/40调用不是“相同推理token数”。

## 冻结与独立审题闸门

1. team-lead接受范围/预算后，AI只准备TASK/公开输入/提交协议；隐藏oracle与独立负控制放评分目录，不在学生可见包。冻结所有题面、fixture原始bytes、generator固定seed、oracle、源码约束规则、harness、依赖运行时版本、dispatch模板、工具白名单及预算。
2. verifier在正式学生前独立审题：核题面与实际契约、独立Python重算期望；正控制参考实现须全过但不交学生；负控制应有“最简总返回/硬编码公开例/全文件collect/跳坏行offset/伪attempt/单文件模拟module/跟随302/无授权仍触网”等故意错误提交，分别命中对应功能或源码约束失败。负控制不能由oracle作者自称已认证。默认禁用和check不执行必须有服务/marker外部见证。
3. 负控制矩阵记录每错误variant应失败哪个case或审核规则，保留实际输出。全部通过才冻结manifest SHA256并由独立复核者签收；有歧义先新revision，不允许开学生后修改期望。当前仅提案，无任何负控制已经执行。
4. 正式绑定T20最终exe bytes/SHA、五AI文档SHA、四task SHA、私有评分件SHA、公开输入/dispatch SHA、runner SHA和raw ZIP SHA；每次启动与交卷评分重核。逐文件二进制原始hash与ZIP原字节副本保留，防Git换行归一。若exe/文档变化整轮停止并记录invalidated，不把两套字节混一个总分。
5. 单题独立workspace仅复制准许输入；评分目录与学生目录分离，不给学生oracle。对话无历史fork，记录session/parent/path关联；完成后保留实际源码、自调每版hash、全部工具输入/返回、UTC与exit/output，不只留最终自述。导出不含私有推理、系统/developer内容、密钥及无关会话，原session只读hash绑定。若宿主缺导出接口，先证明可取得审计数据或降低结果认证级别，不正式宣称盲测协议PASS。

## 评分与报告

分母冻结4个任务，单题PASS要求功能全部通过、源码约束PASS、协议审计PASS。三轴分列：functional case count、source constraints、protocol certification，UNKNOWN不折成PASS。所有任务等权，不用case多的题主导总分；报告每题0/1与全体x/4，另报首次提交（封存后唯一评分）与最多5次自调过程统计，首次自调exit0不等于隐藏集首过。

主要结论只适用于本次四题/该模型配置/指南与产物hash；不由新题结果计算对旧2/8、v2 8/8的改进比例。记录失败类别（指南误导/任务歧义/产品缺陷/学生逻辑/环境/协议不可审计），指南改进在首轮封存后新revision复测，不覆写原失败。REPL和formatter功能验收另表，R304保持OPEN。

## 当前交接

本阶段唯一产出为该提案及AI角色记忆；无正式题包/hash、无学生、无参考解/负控运行、无产品/人指南/旧v2/Git变更。待team-lead确认四题范围和预算，再由其调度verifier预审与后续正式冻结/实测。§26后续只据架构师最终契约同步指南，本轮T20交付后停止，不恢复自动化。
