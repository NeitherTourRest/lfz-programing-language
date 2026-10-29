# T16-B 冻结设计与实施交接

日期2026-10-29。责任人language-architect。A独立验收738e198后，经team-lead批准冻结三spec §20；源草案作为历史保留。本报告不宣称产品已实现/验收。

## 20. T16-B 静态模块图与非执行检查（2026-10-29 冻结）

### 20.1 范围和语法

B只实现模块声明前端与完整静态图检查，不初始化模块；C前模块执行明确拒绝。A §19所有类型形状不改，T15 Result/error/trace三/五/三字段不改。73内置保持，旧无模块程序行为/错误字节保持。

顶层语法：import IDENT from STRING_LITERAL；export IDENT (, IDENT)*。路径不允许插值（即使常量表达式）、空串、NUL、绝对/根/盘符/UNC路径；必须显式.lfz（ASCII不分大小写），允许..，不搜索或补扩展名。import/from仍用现Reserved词token；不新增全局export关键字。仅语句起始Ident("export")后同一语句紧接Ident才识别导出声明，旧export变量/赋值/调用/字段/下标/表达式均不变。两声明只允许Program直接顶层，不能用仅fn_depth/loop_depth代替完整上下文校验。import须先于任何非import顶层语句（export也是非import）；注释空行不影响。原语句终结NEWLINE/}/EOF规则保持。

StmtKind新增ImportDecl(ImportDecl)、ExportDecl(ExportDecl)，结构derive沿现AST，字段精确为：
- ImportDecl { alias: String, alias_span: Span, path: String, path_span: Span }
- ExportDecl { names: Vec<ExportName> }
- ExportName { name: String, span: Span }

外层Stmt.span指import或上下文export起始；path为解码后字符串，path_span指引号起始。AST不持SourceId，Program旧形状不变，深度visitor把两新节点视作叶。导出名必须对应本模块直接let/var/fn/struct/import绑定，可前向列名；不隐式导出builtin或块内局部，重复导出拒绝。import alias之间、与任何其它顶层直接绑定重名拒绝；不涉及import的普通顶层重声明仍按旧规则。

### 20.2 loader增量接口与raw计数

core-dev新增公开CountedLoaded { pub loaded: Loaded, pub raw_bytes: usize }（derive Clone/Debug），以及：
- pub fn load_file_counted(path: &str) -> R<CountedLoaded>
- pub fn load_source_counted(text: &str) -> R<CountedLoaded>
- pub fn load_bytes(path: &str, bytes: Vec<u8>) -> R<Loaded>

load_bytes按现load_file读取之后的UTF8校验→BOM/换行归一→扩展名/前导逻辑；旧load_file/load_source签名和错误保持，可内部委托复用。raw_bytes在任何归一化和前导消费前按实际读入字节或text.as_bytes().len取得，不事后metadata、不重读。根的counted读取仍无新上限；只有成功parse且确含模块声明才进入图admission。旧无模块大root不被2MiB截断，图预算不是root读取/解析沙箱。

依赖由图层File+Read::take(limit+1)有界读取，再load_bytes，不先无界load_file再检查。limit=min(2MiB,16MiB-当前唯一源码总量)，多读1字节用于超限检测。单文件优先于总量错误，原始计数checked_add；已缓存身份不再读/计数。失败读取沿下述模块IO消息，不修改旧root IO消息。

### 20.3 modules.rs类型、API与来源归属

core-dev新增pub mod modules。下列宿主结构字段均pub，derive Debug（需要克隆的叶结构可Clone；Graph不要求Clone）：
- ParsedEntry { program: Program, loaded: Loaded, display_path: String, identity: Option<PathBuf>, base_dir: PathBuf, raw_bytes: usize }
- ResolvedImport { alias: String, alias_span: Span, path_span: Span, target: ModuleId, span: Span }
- ExportBinding { name: String, span: Span }
- ModuleUnit { id: ModuleId, source_id: SourceId, program: Program, imports: Vec<ResolvedImport>, exports: Vec<ExportBinding> }
- ModuleGraph { sources: Rc<SourceMap>, units: Vec<ModuleUnit>, entry: ModuleId, initialization_order: Vec<ModuleId> }
- GraphFailure { sources: Rc<SourceMap>, failure: LocatedFailure }

pub fn module_declaration_span(program: &Program) -> Option<Span>返回首个直接模块声明span；pub fn build_graph(entry: ParsedEntry) -> Result<ModuleGraph, GraphFailure>消费已parse的根快照，不再次读/parse。调用者必须在on_eval_stack内构图并析构所有Program，不能把深AST带回主线程drop。无声明root不调用图，不受图限制。

ModuleId只索引units，SourceId只索引SourceMap，两ENTRY各0，不可互转。模块图拥有AST，SourceMap仅metadata/Loaded，不能持Env/用户值。Loaded诊断副本会增加内存，不能声称仅一份源码。图构建期间用owned SourceMap，成功/失败返回时Rc包装，不反复Rc::make_mut克隆全表。先局部load，成功insert Some(Loaded)，失败insert None记录即返回；不修改A增加setter或伪造文本。新依赖身份成功parse后才有ModuleUnit，失败来源也可占SourceId，故两ID不保证同序。

文件root base是当前文件绝对父目录，CODE/stdin base为开始捕获cwd。根display_path保留原CLI路径或<command>/<stdin>以兼容旧诊断；依赖采用首次发现canonical绝对UTF8路径。identity用canonical PathBuf去重/跟随symlink，不手写lowercase。无法canonicalize的依赖用base.join(path)绝对词法路径作失败显示名（保留..允许）；无法表示为UTF8归ModulePath，定位导入者。源码快照每身份只读一次，C必须消费同图，不重读；不保证文件系统多文件原子快照。相对..及symlink不构成权限沙箱。

### 20.4 限额与确定性迭代DFS

冻结常量：
- MAX_MODULES: usize = 1024，含root，唯一canonical身份计一次。
- MAX_MODULE_DEPTH: usize = 128，显式DFS栈长度含root=1；129边在读目标前拒绝。
- MAX_SOURCE_BYTES: usize = 2*1024*1024，单个进入图的原始源码。
- MAX_GRAPH_BYTES: usize = 16*1024*1024，唯一原始源码总和。

这些是保守初版工程限额，不是RSS上限或性能证明；token/AST/副本会放大。root前端已经成功后才图admission，超限不允许自动放宽/重试。边界127/128/129链、1023/1024/1025节点及bytes等于/多1必须有界测试，禁止真实OOM。

显式栈Frame{module_id,next_import_index}及Visiting/Ready颜色，不用Rust递归DFS。先根admission及本地静态核验，然后按每模块import书写序DFS。每条边先路径/canonical身份；Visiting回边报cycle；Ready加共享边，不加深度/节点；Unseen先节点再深度预算，再限读/源码预算、parse、本地静态核验、入栈。全部边完成标Ready、append initialization_order、pop。后序稳定，同身份一次；错误不靠HashMap遍历决定先后。

错误先后：每文件load/lex/parse完整成功后才静态检查；静态检查先收集全文件direct bindings，再按源码声明/名字次序选首个违规（冲突定位第二次发生的名字span，前向导出可用）。在进入依赖前完成当前文件本地核验；root图admission先于其本地核验。节点先于深度、单文件先于总bytes，循环/Ready识别先于新增预算。图错误终止无初始化。

### 20.5 规范性错误和显示路径

外层LzError仍12类；core扩SyntaxMsg以下变体与精确message：
- ModuleDeclTopLevel => "模块声明只能出现在文件顶层"
- ImportOrder => "import 必须位于其它顶层语句之前"
- ModulePath => "模块路径必须是非空的相对 .lfz 字符串（不含插值或 NUL）"
- ImportConflict { name: String } => "模块导入名冲突：{name}"
- DuplicateExport { name: String } => "重复导出：{name}"
- MissingExport { name: String } => "导出名没有顶层声明：{name}"
- ModuleCycle { chain: String } => "模块循环依赖：{chain}"
- ModuleLimit { kind: String, limit: usize } => "模块图超限：{kind}（上限 {limit}）"，kind仅节点数/依赖深度/单文件字节/源码总字节，limit用十进制上述常量
- ModuleExecutionUnavailable => "模块执行尚未启用"

非法声明位置/顺序在关键字span；路径在path_span；冲突/重复/缺导出在对应name span。cycle在导入者当前边span，chain按显式栈从首次重复模块到当前再闭合，用canonical显示路径和" → "分隔。节点/深度/依赖bytes超限定位导入者边span，root自身bytes超限为根Span::START。模板语法不完整（缺IDENT/from/字符串/逗号后缺名等）沿原Expected/TwoStatements等parser错误规则，不伪造成功。

依赖canonicalize/open/read失败用原Io variant，msg精确"无法读取模块：{target}（导入于 {importer}:{line}:{col}）"，span=None，failure.source_id指目标失败来源（loaded=None）；root读取失败保持原IOError字节。依赖UTF8/前导/lex/parse保持原类/消息/span但source_id指真实依赖。SourceMap插入编号理论溢出在图1024限制下不可达，不panic模拟或与LFZ业务错误混为一谈。

tooling从GraphFailure.sources找到failure.source_id，用其display_path/loaded做前端诊断和JSON；无Loaded不显示其它文件的下划线。前端JSON仍现字段序，file为出错文件；traceback带span时为单<module>定位项，IO无span为空。导入链在消息而非伪造运行栈；check success仍{ok:true,mode:"check",executed:false}。

### 20.6 check、run、旧eval与分工

tooling：同一根前端只parse一次，成功后若无模块声明走旧路径；有声明时check build_graph并在大栈drop，无模块初始化、求值/Interp/输出。root CountedLoaded/Program/base移交图，错误使用真实来源。源码内writeFile/input/while true在check完全不执行。

C前run和所有公开旧eval入口对整个root Program的模块声明做执行前preflight，首个模块声明返回ModuleExecutionUnavailable的SyntaxError，任何LFZ输出/IO/初始化之前失败。仅export也拒绝；即使前面有print/writeFile也不能先执行。B run不承诺构图优先检查其它依赖错误，直接unsupported；不要默默忽略新节点。runtime exec_stmt仍对两个新变体防御性拒绝。parser禁止嵌套声明，若宿主手工构造非法AST不扩展旧前端保证。

core拥有AST/parser/error消息/modules/loader计数与bytes复用/lib登记；runtime仅拥有eval preflight和两个新节点防御match，不启用globals/session；tooling拥有CLI根计数、base、check图与GraphFailure渲染/JSON。单写者串行编译：core先，runtime/tooling适配exhaustive match后唯一编译。三spec及所有旧行为测试不得由实现角色私改；仅明确新节点引起的机械元数据测试迁移先报team-lead。

### 20.7 验收与C衔接

必须验证旧A最终基线/549及新增、T15/T14/JSON/apps；contextual export旧变量/表达式不变；三文件/菱形/重复身份/symlink可用性；每次图隔离；前向导出/冲突/缺名；缺文件/非法UTF8/#42/parse/cycle真实路径和位置；全部限额边界/legacy大root例外；check对依赖IO/循环/递归无执行；run任意模块声明零副作用拒绝。所有fixture在项目内，不因runner默认发现把库文件当顶层独立用例。

C需另冻结Session/独立Env/namespace导出快照及函数home_module切换、初始化失败与attempt恢复active module/source状态；必须消费本图一次性快照。B交付不得称模块运行完成，不新增T15Result字段，不以功能测试替代R304性能验收。

### 20.8 B执行预检返回载荷与补充owner
src/error.rs的SyntaxMsg模块变体及message匹配由core-dev唯一写，外层LzError类不增加。runtime所有旧eval入口汇合到eval_module_on_thread_with_args，在Interp::new及任何exec之前调用module_declaration_span。存在声明时立即返回TracedRun：result=Err(Syntax(ModuleExecutionUnavailable,first_decl_span))，frames=[TraceFrame{func_id:MODULE_FUNC_ID,span:first_decl_span}]，func_names=[None]，origins=None；其它旧封装按原result丢弃或暴露trace。exec_stmt两个变体另作同类错误防御。load_bytes精确接owned Vec<u8>，可直接String::from_utf8转移所有权；counted入口在移交bytes前保存len，不额外clone或重读。

### 20.9 冲突定位补钉（旧AST保形）
旧Decl/FnDecl/StructDecl没有独立name_span，禁止从名字长度或关键字列猜算。import-import冲突定位后一个ImportDecl.alias_span；import与普通直接let/var/fn/struct绑定冲突定位该普通声明Stmt.span（关键字），不扩旧AST字段。重复/缺失导出仍用精确ExportName.span。本条细化20.4/20.5名字定位，优先适用；原其它节点形状保持。
