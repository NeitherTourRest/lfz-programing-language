# T16 本地模块设计草案

日期：2026-10-29。责任人：language-architect。状态：后继草案，未获冻结批准，不改变当前spec/实现。T15仍按原验收流程收口；本草案不是已实现能力。

## 推荐最小语法

```lfz
#42
import math from "./lib/math.lfz"
print(math.add(2, 3))
```

```lfz
#42
var calls = 0
fn add(a, b) { calls += 1
    a + b }
export add, calls
```

import/from使用现已保留词，不引入as、动态import或裸名搜索。语法为顶层`import IDENT from STRING_LITERAL`，路径必须无插值、非空、无NUL；禁止导入表达式、函数/块内导入。所有import必须位于模块其它可执行声明之前；同层别名重复及与其它顶层声明冲突在图检查中报SyntaxError。导入名绑定为不可重绑定的let语义。

`export IDENT (, IDENT)*`是模块顶层上下文声明，不增加全局KwExport：只有语句起始Ident("export")后紧接另一个IDENT时识别。至少一名，不接受export空列表或export表达式；`let export=1`、`export(...)`、名为export的struct字面量仍走既有语法。这个形态在旧文法中是非法的相邻标识符语句，需core-dev补兼容反例验证，不能仅凭推断宣布无破坏。导出声明可放初始化尾部，整模块预扫描收集，无执行动作。导出名必须对应本模块顶层直接let/var/fn/struct/import绑定，允许前向列名；重复或未声明导出为前端SyntaxError。未导出的名字完全私有；无导出则空namespace，不隐式导出所有全局或builtin。首版不做from文件导入单个名字、重命名、export转发表达式或星号导出。

## 路径与静态模块图

文件入口以当前源码文件父目录为base；根脚本可按原run规则为任意扩展名，模块目标只允许.lfz（ASCII大小写不敏感），必须有#42。-e/stdin入口以执行开始时记录的cwd为base（不能随运行变化）；旧无模块库API遇import需报告需要显式base/source上下文，禁止偷偷从调用线程cwd找文件。新模块感知API明确传entry origin/base与args，CLI统一使用它；旧API无import程序保持行为。

导入串只允许相对路径，拒绝盘符/UNC/根路径；允许..（便于兄弟库），因此它不是文件权限沙箱。相对路径按宿主Path语义解析；canonicalize现存文件产生缓存身份（跟随符号链接，Windows宿主等价路径共用身份），打开/编码/前导仍复用loader。不要手写小写化路径来模拟所有文件系统。错误显示名用首次发现的规范绝对路径，后续同身份复用，确定性指固定文件系统/入口/导入声明序，不承诺不同机器绝对路径字节相同。要求路径可表示为UTF8；路径或canonicalize失败按冻结后的IOError消息，缺文件诊断需同时保留导入者位置与目标路径。不是包搜索器，不试探扩展名、不搜索PATH/全局库、不下载远程资源。

ModuleGraph先于任何LFZ求值完整构建：按每个文件import书写序DFS，先load/lex/parse/收集声明，再递归依赖；节点状态Unseen/Visiting/Ready，Visiting回边即报循环依赖，含稳定链A→B→A及当前导入span。图中任意文件前端错误使run在任何初始化前失败；不可达分支也不能隐藏import或语法错。每份源码在本执行加载一次并冻结内容，run使用这份AST快照，不在执行时重新读取产生TOCTOU差异。图深/节点数/总源码字节要在批准前设置明确受控上限；不可让深模块链用宿主递归栈溢出。建议迭代式DFS并在现大栈中解析/析构所有AST，不能把深Program移到主线程drop。

lfz check调用完全相同的模块图构建和静态导出/别名核验，然后丢弃图，绝不初始化模块。因路径和导出名均为源码声明，不需要求值来发现依赖。不额外做一般名称/类型分析；导出声明的顶层名字核验是新增有限静态规则，必须在check文档中明确。模块内writeFile/input/while true、缺少运行时数据文件都不执行/不触发；缺模块文件/模块语法错/循环依赖应失败。check读模块文件仍是宿主IO，不宣传零IO/沙箱。

## 初始化、缓存与namespace

图无前端错误后，按DFS依赖后序初始化；同身份模块在一次执行至多初始化一次，菱形依赖共享实例。入口最后执行。缓存绑定一次eval session，绝不跨两个脚本运行/测试case/库调用共享。缓存不含全局静态HashMap。初始化失败停止整个执行；首版不提供运行时import，因此不存在catch后再导入重试的隐式半初始化缓存。T15 attempt不能捕获模块图阶段或根初始化前的错误。

每模块新独立全局Env；可见内容只有自己的声明和显式namespace绑定。不会把导出名字注入调用者globals。初始化成功后，以导出声明书写序取最终值建立新普通struct namespace，多个导入别名共享同一个namespace Rc。初版为导出值快照，不是live variable binding：导出标量后来在模块内重绑定不刷新namespace；导出容器仍按A1共享，字段写入namespace也不重绑定模块局部。变量导出例中calls是初始化时0的快照；要实时读取请导出getCalls函数。无冻结/只读namespace新类型，namespace字段可按原struct规则改写；需要文档化。keys/entries/JSON继续受A5数据面过滤函数字段，不能将其当“列所有导出函数”的反射API。调用math.add按原字段调用绑定self为namespace，直接/闭包路径按原self规则，不引入另一套绑定魔法。

argv在所有模块内取同一session显式脚本参数快照（每次调用仍新数组）；模块没有单独argv，check不求值argv。随机状态、IO和attempt副作用沿原进程/会话约定；初始化顺序可观察，不能并行初始化。函数必须持有定义模块身份，任何调用都使用定义模块globals及source定位，而不是调用者globals；模块内闭包/方法/逃逸函数同理。

## 当前源码约束与建议接口改造

现Loaded只有text/line_base；Span只有line/col，LzError无file，TraceFrame只有func_id/span，CLI给所有帧附入口path；Interp仅一个globals，UserFn没有home module。直接在原Interp里轮换globals然后把函数导出，会使导出函数调用时查错全局，不能这样最小拼接。

建议新增模块图层（独立模块文件，不塞进loader词法职责）：SourceId/ModuleId、SourceRecord{identity,display_path,Loaded}、ModuleUnit{Program,imports,exports}、ModuleGraph、Session模块Env缓存。AST增加ImportDecl/ExportDecl，parser上下文识别顶层；图层完成路径/静态联结。保留Span及LzError现公开形状，另加LocatedFailure{error,source_id}或等价外层载荷；TracedRun扩可选source map/逐帧source标识，以旧无模块入口保留原显示行为。CLI错误渲染必须查最深错误的Loaded，不可拿入口文本标下划线。

UserFn应携带home_module身份，call_user在home环境查globals；统一Interp/session分配func_id并记录定义source，模块顶层帧也需独立source身份。不要每模块独立func_id又拼进一个trace导致名称冲突。优先通过ModuleId查session持有的Env，避免每个UserFn强Rc持home_globals形成globals→函数→globals引用环。函数/容器本来可有用户环，但新增模块机制不应为普通无环模块制造必然泄漏。跨session调用来自旧session的函数值应明确拒绝或由运行时设计session所有权，不可把旧ModuleId误解释成新session同序号；现公开API没有通用跨session函数调用功能，本轮不新增它。

模块调用退出（成功/失败）须正确恢复active module/global指针，失败trace来源已单独保存；T15 attempt快照需同时维护新增active source/module状态，依然不回滚func_names/模块对象/用户副作用。这是运行时接口的实质改造，不是仅加两个parser节点。

## 错误位置与T15兼容

现有LfzError类保持，新增SyntaxMsg仅用于非法模块声明、导出冲突及循环等前端错误，具体中文模板在冻结时逐项定案。IO错误仍沿既有类，但图错误应带发生文件/导入span。CLI run/check JSON保留旧字段序及成功协议，每个trace frame的file取实际source；原无模块程序输出字节不得变化。

T15 Result协议已冻结无file，不能在T16悄悄增加error/trace字段。首版保留attempt error与frame原五/三字段（frame的line/col是各自真实文件行列）；跨模块时不能声称其记录含完整文件定位，开发指南明确此限制。CLI未捕获错误的file定位先完整实现。若希望attempt也含file，需另立协议版本/专用接口批准，不能以兼容名义静默增键。函数名不临时拼路径以绕过协议。

## 拆批成本与交付依赖

1. T16-A：先冻结source identity/LocatedFailure及函数home module契约和旧无模块兼容矩阵；这是最高风险依赖，涉及core/runtime/tooling。
2. T16-B：parser声明+图构建/循环/静态检查，只做check的无执行多文件验证；不冒称模块运行已实现。
3. T16-C：session缓存、独立Env、home globals与初始化；三文件应用及attempt交互验收。
4. T16-D：CLI跨文件精确诊断、人/AI指南、独立复验、冻结候选性能。源码扫描与功能回归不能代替性能门槛，R304仍开放。

整体成本中高，主要在原全局查名/错误定位基础改造；不用隐式global注入、预拼源码文本或多Interp互调来伪装模块。暂不做远程包/动态import/循环live绑定/热重载/并行初始化/包管理。

## 必测矩阵

三文件与菱形单次初始化；导入顺序副作用；重复路径./..与符号链接身份；每次执行缓存隔离；同名私有全局不串；导出函数在调用者同名变量存在时仍查home globals；逃逸闭包/模块方法/self/argv；导出快照与共享容器/namespace改写；export上下文不破坏旧export标识符/模板语法；缺文件/非法UTF8/#42/导出缺名/重复/循环及完整跨文件错误下划线；-e/stdin cwd base；check遍历模块但没有任何初始化输出/写入/等待；attempt捕获跨模块运行错后继续且后续未捕获错误trace无污染；深模块链受控拒绝；549基线及T15/T14全部旧用例保持。

所有测试临时文件在项目Temp；D盘仅0.31GB，本草案不运行构建/大任务。无实现、测试通过率或性能提升声明。
