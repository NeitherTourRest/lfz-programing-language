# T21 保真空白格式化契约
## 27. T21 保真空白格式化（2026-11-13 冻结）

### 27.1 命令与输出
lfz fmt FILE读取一个文件，把完整结果原始bytes写stdout，成功exit0。仅当原始argv恰有两个token、首项为fmt且第二项非空并且不以连字符开头时，进入格式化分支。连字符开头的文件名可使用显式相对路径格式化。其余形态走既有解析。fmt、fmt --json、fmt --、fmt --allow-process、fmt --allow-http、--json fmt和fmt -- ARG均保留既有裸文件行为，run fmt仍是原执行入口。格式化只接受一个文件路径。通用选项剥除后的argv不参与格式化分派，因此--json fmt FILE和附带其它选项或脚本参数的格式化调用均拒绝。
沿现loader路径规则处理扩展及前导。I/O、语法、内部一致性或参数失败exit2、诊断stderr。读取和校验全部完成后才写stdout，因此写出前的失败stdout空。实际stdout写入失败可能已写出前缀，exit2，不宣称可撤回OS写入。始终不改输入文件。模块文件只检查当前文件的加载/lex/parse，不构图、读取依赖或执行。

### 27.2 精确变换与位置契约
变换原始UTF8文件bytes，保每个原行结束标记及顺序（LF、CRLF、裸CR可混合），保EOF末行有无行终止符；保BOM与已由loader消费的#42前导整段bytes。沿loader精确接受前导，不先trim修复错误前导。
只识别顶层CODE模式下lexer作为空白消费的ASCII SP/TAB。对每个物理行：
- 行首从该行第一个原字符开始的CODE空白前缀，把每个TAB换成一个SP，保前缀字符数。首行从已保留的可选BOM后判定，不越过前导。已有SP不变，不增减缩进宽度。
- 行尾紧邻行结束或EOF的CODE SP/TAB后缀删除。纯CODE空白行同样处理，重叠时行尾删除优先。若删除导致前一裸CR与本行LF相邻而合并为CRLF，保留该run，连TAB也不改。
- 其余字节原样，包括token间空白、字符串整个字面量（含全部插值及格式说明）、行注释末尾的空格/Tab、块注释内部各行空白、数字拼写、管道、双分号和UTF8文本。行内块注释闭合后独立CODE末尾空白可删；注释闭合后中途空白不属于行首前缀。
基线lex列按Unicode标量计数，TAB与SP各1，因此一对一替换保后续源码token列。增加/减少缩进会改变attempt.error.col和traceback中脚本可观察的列，故不做4空格深度重排。行尾处理只允许Newline/EOF token列变化，所有token行号保留，全部其它token起始Span一致；非空程序运行顶层使用首Stmt.span，空Program的Newline/EOF列无运行错误或脚本可观察位置。若发现某成功语法把此列带入运行可观察位置，收窄相应行的变换并加入回归。
保持LFZ求值及其位置语义。显式读取格式化前后源码文件bytes的程序当然会观察文本变化。诊断展示的源文本允许体现行尾变化、tab一对一后的文本；LFZ错误line/col及函数帧位置按上述契约保持。

### 27.3 核心API与校验
tooling唯一写formatter.rs、lib.rs注册及CLI。公开：
    pub fn lfz::formatter::format_source(path:&str,raw:&[u8])->Result<Vec<u8>,FormatError>
    pub enum FormatError {
        InvalidSource {error:Box<LzError>,loaded:Option<Loaded>},
        InvariantViolation,
    }
FormatError Debug；InvalidSource包装原loader/lexer/parser错误，无新LFZ错误类；loader失败loaded=None，lex/parse失败保原Loaded用于诊断。InvariantViolation是宿主内部一致性失败，CLI stderr“lfz: 格式化结果未通过保真校验”，stdout空、exit2。tooling负责CLI参数、读取、诊断和stdout字节写出。core唯一写lexer专用布局入口：
    pub(crate) fn lex_format_layout(text:&str,line_base:u32)
        ->R<(Vec<Token>,Vec<std::ops::Range<usize>>)>
返回的range是已归一化Loaded.text中的Unicode标量索引，按开始位置排序且不重叠。每段是同一连续顶层CODE SP/TAB的最大范围，排除换行、注释、字符串和插值。tooling把这些range映射回原始文件字节。
先原raw loader/lex/parse均成功，再按formatter专用布局收集可改区间；修改后用同path重过loader/lex/parse。两侧token数量/顺序/完整kind及payload一致（含Newline/EOF），所有token行号、除Newline/EOF外的列完全一致。保真检查失败拒输出，不回退输出误导性成功。调用/析构深AST沿现大栈约束，CLI整路径on_eval_stack；不执行代码。
布局复用lexer已有CODE/STR/INTERP/FormatSpec与注释消费边界，fmt专用入口记录顶层CODE空白，禁止以regex、简单引号/括号计数或AST重新打印代替。普通lex不建布局/原始偏移表，不加可观察运行开销；可用泛型空观察器/专用实例避免普通路径分支。normalized scalar范围对应原raw byte范围，仅fmt建映射；显式处理去BOM/前导和CRLF/裸CR映射，不能用列数当字节索引。构造输出仅复制原raw slices和上述替换/删除，保证注释等被lexer丢弃的文字保真；token比较不替代原字节保护。

### 27.4 验收
精确字节oracle覆盖BOM+#42、混合LF/CRLF/裸CR、CR+空白+LF、末尾无LF、空/仅注释文件、多行块注释、行注释尾白、嵌套插值/格式说明/转义、中文、0xff、pipe、;;、tab等列和行尾删除。断言重复格式化字节相同及无源码覆盖。无效UTF8/前导/词法/语法、缺文件、选项误用的退出码和stdout空应有独立见证。
在安全fixture上分别运行原/格式化文件，覆盖控制流、同逻辑行块注释、可恢复错误中显式读取line/col/traceback、运行错误位置及副作用；等同条件下对照stdout和效果（路径/诊断源码显示差异按测试归一，不能丢位置值）。裸fmt单参数和其它CLI旧形态回归。验证不以parse成功或AST相等单独代保真。
只本轮格式化功能，无新缩进策略/完整formatter/模块图/REPL/HTTP变更。原R304状态不因本契约改变。
