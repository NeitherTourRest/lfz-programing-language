# T16-B 静态模块图与非执行检查草案

日期：2026-10-29。责任人：language-architect。状态：仅草案，待T16-A独立验收后由team-lead批准冻结；不修改spec或源码，不把本设计视为已实现模块。A的SourceMap/SourceRecord/SourceId/ModuleId及旧形状契约完全不改。

## 本批结果与明确非目标

B使前端识别顶层模块声明，并让check检查完整本地依赖图及导出/别名约束，始终不初始化任何模块。模块运行属于C；B阶段run或旧eval一旦发现模块声明，须在整个Program的任何LFZ副作用之前明确拒绝，不能走到import才失败、不能忽略export、不能把模块文件当普通脚本成功执行。

旧无模块程序仍走原加载/解析/执行路径，旧输入大小与错误协议不被本批图预算静默收紧。T15 Result/error/trace字段保持；没有新内置或跨session值调用，不预拼源码、不隐式global注入、不多个Interp互调。

## AST与parser推荐接口

保留Program/Stmt包装形状，StmtKind新增两个变体：
- ImportDecl(ImportDecl)
- ExportDecl(ExportDecl)

新增结构：
- ImportDecl { alias: String, alias_span: Span, path: String, path_span: Span }；外层Stmt.span为import关键字位置。
- ExportDecl { names: Vec<ExportName> }
- ExportName { name: String, span: Span }；外层Stmt.span为上下文export位置。

上述结构derive与现AST一致（Clone/Debug等），不含SourceId：单Program仍采用本地Span，来源由ModuleUnit持有。每条导出项分别保留Span，重复/缺名可定位具体标识符，不只指向整句。

语法为import IDENT from STRING_LITERAL及export IDENT (, IDENT)*。路径是解码后的普通字符串，不允许插值（即使插值表达式为常量也拒绝），不能为空或含NUL；路径类型规则不通过LFZ求值。parser完整消费路径后仍走原NEWLINE/}/EOF语句终结检查，不接受尾随表达式或逗号。

lexer可保留现Reserved("import")与Reserved("from")种类，parser仅在模块声明分支接受它们，无需把它们变成一般标识符或批量改旧reserved词测试。其它保留字语义保持。若core选择专用token，必须另列机械测试迁移，不能悄悄弱化reserved断言；推荐保留现token以缩小变更。

export不新增全局关键字：只在语句起始Ident("export")后，同一语句紧接Ident时作为上下文声明。现parse_stmt_seq严格要求语句终结符，因此export x这种相邻标识符旧代码本来不合法。必须实跑兼容反例：let export=1、export = 2、export(1)、export.foo、export["x"]、export + 1、单独export、let x=export；这些仍按旧表达式/赋值解析。export在struct字段/函数参数/函数名中仍是标识符。带逗号却缺首名/尾名应报语法错，不当作表达式悄悄通过。

import与被识别的export只允许Program直接顶层；函数、lambda、if、while、for或表达式块内均拒绝，不能只用fn_depth/loop_depth判断（顶层if块两者也可能为0）。建议parse_stmt_seq显式传顶层上下文或维护块/语句上下文，不改变旧表达式绑定规则。

import须在任何非import顶层语句之前；export也属于非import语句。注释和空行不影响。允许多条export，允许在初始化语句前声明导出，但其名字静态核验针对整个Program。顶层直接let/var/fn/struct/import是可导出绑定，块内声明、builtin和未定义名不是。允许前向列名；导出重复名拒绝。已有普通顶层重复声明/遮蔽规则不额外收紧；但import alias与任一其它顶层直接绑定重名拒绝，import重复alias拒绝，包括alias被后续let/fn覆盖。

新增节点需更新check_ast_depth迭代visitor为叶节点，parse_stmt所有上下文与runtime exhaustive match均显式处理，不删除现检查。B的runtime guard应先扫描全部直接Stmt再决定拒绝，所以print(...);后面export的源码不能先打印。

## 图层类型与owner建议

新增src/modules.rs由core-dev负责静态模块图，不塞入source.rs或evaluator。source.rs仍只metadata，不持AST/Env；ModuleGraph持Program及图边，C的Session才持运行时Env。

推荐宿主结构（精确可见性在B冻结时定案）：
- ParsedEntry { program: Program, loaded: Loaded, display_path: String, identity: Option<PathBuf>, base_dir: PathBuf, raw_bytes: usize }。
- ResolvedImport { alias: String, alias_span: Span, path_span: Span, target: ModuleId, span: Span }。
- ExportBinding { name: String, span: Span }。
- ModuleUnit { id: ModuleId, source_id: SourceId, program: Program, imports: Vec<ResolvedImport>, exports: Vec<ExportBinding> }。
- ModuleGraph { sources: Rc<SourceMap>, units: Vec<ModuleUnit>, entry: ModuleId, initialization_order: Vec<ModuleId> }。
- GraphFailure { sources: Rc<SourceMap>, failure: LocatedFailure }。
- build_graph(entry: ParsedEntry) -> Result<ModuleGraph, GraphFailure>。

入口Program由CLI已执行的旧前端管线移交，不再重读或再次parse。模块图返回一次性自包含来源表，ModuleId只索引units，SourceId只索引SourceMap；禁止两ID整数相同就互转。ID在首次发现时按DFS书写序分配，entry两域各ENTRY；源表可含失败文件记录，而units只含成功parse的模块，后续ID不必恒等。

A SourceMap append-only：依赖load结果先留局部，成功insert记录Some(Loaded)，加载失败insert记录None并立即返回GraphFailure；不为此给A加setter或拿伪文本填记录。以owned SourceMap累积，图成功/失败返回时才Rc包装；不要每次插入Rc::make_mut克隆全部来源。ModuleGraph独占Program，SourceRecord为诊断需Loaded副本，内存重复必须在性能说明中披露，不能声称只有一份源码内存。

B未启用初始化，initialization_order只是后序依赖计划供C消费；没有全局模块运行缓存或Env。单位内部仍存原ImportDecl/ExportDecl节点；C须显式处理，不把它们当普通表达式执行。

## 路径、读取快照与legacy入口兼容

文件入口base为该文件父目录；CODE/stdin入口base为会话开始时捕获cwd；老无上下文eval API遇模块声明先明确拒绝，不能偷偷从调用线程cwd找模块。CLI根display_path保留原输入/伪路径以保证旧错误文本；依赖显示路径采用首次发现的canonical绝对UTF8路径。identity是canonical PathBuf，仅用于缓存键，不能手写lowercase模拟跨文件系统等价。

仅导入相对路径，拒绝宿主absolute/prefix/root、盘符/UNC及跨平台显式盘符形态；允许..，因此不构成权限沙箱。目标只接受.lfz（ASCII大小写不敏感）且要求#42；root仍按旧显式run规则可非lfz扩展名。不给路径自动补扩展名、不搜索PATH/全局库/网络。符号链接身份跟随canonicalize；同identity只加载解析一次，复用第一份Loaded/Program快照，后续运行不得重读。

root先走原加载/lex/parse，只有出现ImportDecl或ExportDecl后才进入图并应用图预算。旧无模块大文件不能被新2MiB上限截断。该兼容路径意味着root读取/解析不受图内存上限保护，必须明确它不是完整资源沙箱。

为按原始UTF8输入字节计预算，建议core在新CLI前端入口旁保留RawSourceStats（不改Loaded）：文件读取时记录原始bytes.len，CODE/stdin在loader归一化前记录字节数。可增加复用同一解码/归一化/前导逻辑的内部计数入口，旧loader::load_file签名不变；不得用事后fs::metadata猜读取长度，不得为计数重读root造成TOCTOU。未进入模块图时计数不产生限制。

依赖在图预算启用后采用File+Read::take(limit+1)有界读取，再复用相同UTF8/归一化/前导逻辑（可抽取load_bytes内部helper）；不要先旧load_file无界读完再说有2MiB上限。root与每个依赖原始快照逐一计数，别名复用不重复计。检测超限多读的1字节是哨兵，不算成功装入图的内容。文件在读取过程中改变不提供跨文件原子快照，只保证每个已读模块之后不再重读。

## 实际候选预算与迭代DFS

推荐固定默认上限（批准前仍为草案）：
- MAX_MODULES=1024，含root；每个canonical身份算一次。
- MAX_MODULE_DEPTH=128，显式DFS栈长度含root=1；128允许，尝试进入第129层时在读取/解析目标前拒绝。127/128/129必须分别测试。
- MAX_SOURCE_BYTES=2*1024*1024，单个进入图的原始源码字节上限。
- MAX_GRAPH_BYTES=16*1024*1024，已接纳唯一模块原始源码字节总和。

理由：课程和Agent本地库规模远小于1024/128，2MiB远高于现脚本，16MiB给多库空间并在当前D盘约0.31GB条件下保持有界测试；源码限额不是AST/RSS等量上限，token/AST/Loaded副本会放大内存。它们是初版保守工程限额，不是性能测试证明或通用多租户隔离；超限错误必须可复现，默认不得自动放宽重试。

显式工作栈Frame { module_id, next_import_index }加颜色Unseen/Visiting/Ready，禁止Rust递归DFS：
1. root前端成功并通过图admission，提取direct bindings/imports/exports作本地静态检查，分配ID、标Visiting、push。
2. 处理栈顶下一条import，按书写序验证路径并canonicalize。Ready目标只加边；Visiting目标为cycle，给当前边位置及稳定A→B→A链；Unseen目标先检查节点/栈深，再有界读/解析/本地静态检查，标Visiting并push。
3. 每个模块全部边完成后标Ready，append到initialization_order，pop；直到空栈。菱形依赖顺序及重复模块只出现一次。
4. root所有静态导出检查在读其依赖前进行；同节点多个错误选择源码声明顺序第一项。明确错误优先级后冻结，不能HashMap随机遍历决定先错。

依赖读取limit取单文件上限与总剩余上限的min，再加1读哨兵。超限判断固定先单文件、后总量（如都超过，单文件优先）。源码admission相同规则；总字节算加用checked_add，不允许整数溢出或超限后继续parse。节点/深度失败定位在导入者当前边；字节超限也优先定位该边并在消息给目标路径，root自身超限定位root Span::START。全图构建及Program析构在现on_eval_stack大栈内；迭代DFS不等于Program drop无递归，不能把深AST带回主线程drop。

## 错误类、路径与建议消息

仍用现LzError外层，不新增第13类；SyntaxMsg可增加模块相关变体。以下是待批准模板，非当前规范：
- 嵌套声明：SyntaxError，import/export span，“模块声明只能出现在文件顶层”。
- import次序：SyntaxError，import span，“import 必须位于其它顶层语句之前”。
- 路径非静态/空/NUL/绝对或非lfz：SyntaxError，path_span，“模块路径必须是非空的相对 .lfz 字符串（不含插值或 NUL）”。
- 重复/冲突alias：SyntaxError，发生冲突的声明名span，“模块导入名冲突：{name}”。
- 重复导出：SyntaxError，重复ExportName.span，“重复导出：{name}”；缺顶层绑定：“导出名没有顶层声明：{name}”。
- cycle：SyntaxError，导入者边span，“模块循环依赖：{path chain}”；路径链按显式栈/首次canonical显示名稳定。
- 超限：SyntaxError（图前端受控限制），当前导入span或root START，“模块图超限：{kind}（上限 {limit}）”，kind固定节点数/依赖深度/单文件字节/源码总字节。这些不是可由attempt恢复的业务错误。
- B执行guard：SyntaxError，root首次模块声明span，“模块执行尚未启用”；包括只有export的程序，在任何语句执行前失败。
- 依赖canonicalize/open/read失败：IOError，记录目标显示路径且Loaded=None，“无法读取模块：{target}（导入于 {importer}:{line}:{col}）”，span=None；不改旧root IOError消息。
- 依赖UTF8/前导/lex/parse：沿既有错误类和消息，failure.source_id指依赖自己的SourceRecord，span沿其line_base，不拿入口文本画线。

GraphFailure必须携来源表，调用者先定位failure.source_id再取display_path/Loaded。module graph错误的front-end traceback仍复用原单<module>位置记录（IO无span为空），不要伪造已经执行的模块调用栈。导入链若只在消息中出现则指南明确，不偷偷扩JSON字段。T15 Result完全不涉及B graph失败，因为图先于任何求值。后继C运行错再用逐帧真实来源，不能依据active状态在冒泡末端猜file。

## CLI与旧eval迁移及owner

core-dev：AST/parser上下文/深度visitor、新modules.rs静态图、loader内部计数/有界bytes复用（原loader签名和行为保持）、lib模块登记；不改runtime或CLI。
runtime-dev：旧eval所有入口在实际exec前完整扫描root模块声明并拒绝，新增StmtKind match防御，B不实现namespace/Env缓存或导入执行。T15路径不变。
tooling-dev：check调用新图检查分支并使用GraphFailure的真实文件/Loaded渲染；run对模块声明明确unsupported guard并保持零LFZ副作用。无模块分支的原CLI JSON/stdout/stderr/error字节保持；不将图metadata当实际执行trace。
test-engineer/verifier：语法与图边界的公开fixtures/独立程序；不得为新功能删除旧reserved/旧export标识符测试。多源码fixture只在项目Temp或版本化tests新目录，别使runner误把模块library当顶层用例（默认发现需有明确布局/清单策略）。

依赖顺序：A独立过→批准B契约→core节点和图→runtime guard与tooling接入（源码单写者、唯一编译）→check全图/旧行为独立验收。B终验明确只check模块图，C再实现执行；不打含“模块运行已完成”的版本说明。

## 最少验收矩阵

- 所有旧549及后续A新增tests、T15/T14公开回归、JSON/apps结果保持。
- import字面路径/插值/空/NUL/绝对/前导/UTF8；顶层/if块/循环/函数/lambda位置；export上下文旧表达式/变量/字段兼容。
- 前向导出、重复导出、builtin不隐式导出、私有块局部不导出、import alias与后续let/fn/struct重名。
- 三文件、菱形、重复./..、可用符号链接、同文件别名只读一次；导入顺序稳定、每次check图隔离。
- 缺模块、子模块语法/UTF8/前导错、循环链精确路径/位置/JSON，加载失败无源码时不错误画线。
- 节点1023/1024/1025，链127/128/129，单文件与总bytes恰好/多1；稀疏或有界生成fixture，别用巨量进程/真实OOM，证明依赖先限读。
- 没有模块声明的大root保留旧行为；进入图的大root按图admission拒绝且说明此前已加载解析。
- check依赖中含writeFile/input/print/while true仍不执行；run含模块声明即使前面有print/writeFile也必须任何副作用前拒绝。
- -e/stdin base明确；宿主无base旧eval明确拒绝；旧T15Result字段不加file。
- 图Program析构仍在大栈内，无真实panic/abort压测，不把构建与check通过当性能R304通过。

## 待冻结时需一起定案

上述具体限额/消息模板/GraphFailure呈现、ParsedEntry raw byte计数接口与AST新变体必须同批冻结，不能让core/runtime/tooling各发明一套。A基础载荷签名完全不改；T15的来源缺项限制保持。B图并不提供文件沙箱或运行缓存，C初始化必须消费已检查快照，不再次读文件。

本草案只读核对当前source.rs、loader.rs、ast.rs、lexer.rs、parser.rs、evaluator/CLI接口；没有构建、源码修改、产品测试或Git。R304仍OPEN。下一步由team-lead在A独立验收后批准B冻结。
