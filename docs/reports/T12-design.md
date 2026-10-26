# T12 设计冻结与验收清单

日期：2026-10-25。责任人：language-architect。状态：设计已冻结，产品实现与性能结论由后续独立验证提供。

## 方向与特色

本轮选择「Agent 数据脚本」闭环：显式字符串参数、无临时源码文件的执行入口、严格 JSON 数据交换，再通过已有 data-last 管道组合。它强化 LFZ 既有确定性 struct、中文结构化错误和短脚本表达能力，不增加关键词/AST/模块/异常语法。效率改进集中在 JSON 单次构造与 struct 字段索引，避免把接口便利建立在隐蔽 O(n²) 上。

## 唯一事实源

- docs/spec/syntax.md §16：CLI 形式、选项区、stdin、兼容性。
- docs/spec/interface-contract.md §16.1–16.5：argv、两求值API、JSON类型/精确错误/复杂度、内部字段索引。
- docs/spec/semantics.md §16：引用/数据面/往返/确定性解释。
- DECISIONS.md 2026-10-25 两条 language-architect T12 ADR；第一条标题时间误写已由紧随其后的00:50条目追加订正。

## 可执行验收矩阵（交 test-engineer / verifier）

| 编号 | 输入/操作 | 必须观察到 |
| -- | -- | -- |
| A01 | 旧eval API执行argv()；无参数CLI脚本 | []；旧API签名不变 |
| A02 | 文件/run文件/-e/run-e/stdin/run-stdin，均追加 -- 后的空串、中文、--json、--、-e | argv内容/顺序/字节值一致；不含文件名/源码/分隔符 |
| A03 | a=argv(); a[0]="changed"; 再argv() | 新调用保留原参数；a可正常修改 |
| A04 | f=argv；闭包调用f；argv存入容器再取调用 | 普通一等内置行为有效；同一执行上下文 |
| A05 | argv(1)、jsonParse()、jsonStringify(1,2) | TypeError及既有精确arity消息 |
| A06 | 同进程不同模块注入不同args；旧入口再执行 | 相互隔离，无全局泄漏 |
| C01 | lfz run a.lfz b.lfz；lfz -e CODE extra | 仍报CLI参数错误，exit2 |
| C02 | -e的CODE token恰为--json/--/--help | 作为源码解析而非开关/分隔符/帮助 |
| C03 | -e 空字符串；空stdin | 空程序成功，exit0 |
| C04 | 无参数lfz | 缺少子命令，exit2，不等待stdin |
| C05 | -e/stdin解析和运行错误，含CRLF与Unicode | file伪路径<command>/<stdin>；line_base=0及正确行列 |
| C06 | stdin非法UTF8；stdin读失败 | IOError: 无法读取标准输入；无span |
| C07 | stdin源码执行input() | 读完源码后的EOF走原IOError，无挂起/抢读源码 |
| C08 | 原文件前后--json、-e前后--json、--后字面--json、CODE中的--json | 选项只作用于合法选项区；JSON状态stdout恰一对象，程序输出stderr |
| C09 | 原裸.lfz与run非.lfz/#42正负例 | 原扩展名/前导规则不变；REPL仍无实现声明 |
| J01 | null/true/false/空数组/空对象/Unicode/嵌套JSON | nil/bool/array/struct/string映射正确 |
| J02 | i64最小/最大、9007199254740993；再编码 | int精确，绝不转f64丢精度 |
| J03 | 越i64整数、1e999、-0、-0.0、1.0、1e0 | 超界ValueError；-0为int0；其余三者float，-0.0保留符号 |
| J04 | 代理对、孤立高/低代理、NUL、五种短转义、双引号/反斜杠 | 正确Unicode往返/非法项拒绝；错误是ValueError不是SyntaxError |
| J05 | 注释/尾逗号/单引号/裸键/+1/01/空文本/BOM/额外尾项/不合法空白 | 严格拒绝；reason为不是合法的 JSON |
| J06 | 同对象a与转义后等于a的键；不同对象同名键 | 前者拒重复，后者允许；重复offset指第二键开引号 |
| J07 | JSON里$和${x}、编码后的$ | 都是数据；编码不产生非法JSON转义反斜杠美元 |
| J08 | 128/129层数组、对象、混合容器；编码同深度 | 128允许129拒绝；深度错误ValueError且无panic |
| J09 | 未排序字段/函数方法/数组中函数/模板/NaN/Inf | 键按UTF8排序；方法忽略；其余非法项按精确reason拒绝 |
| J10 | 自环/间接环/共享但无环子树 | 前两者拒绝，后者重复编码成功 |
| J11 | 编码包含控制字符/斜杠/中文/非BMP；解码回读 | 紧凑、合法JSON、无尾换行；float类型保留；容器不保引用拓扑 |
| J12 | 非string jsonParse；同一坏值在多排序字段/数组下 | 既有TypeError模板；确定的深度优先首错 |
| P01 | struct同键替换/新增/别名写/复制/删除内置 | 索引与fields一致，数据面和已有外显排序不变 |
| P02 | JSON大型对象批量构造与编码、struct字段多次读写 | 不逐次线性set建对象；计时另由perf给同输入/多轮实证 |
| R01 | 全量Rust、官方黑盒、sortviz和新CLI/JSON黑盒 | 既有494/130基线不能丢；新增计数以实跑为准 |

## 兼容边界与效率声明

三个新增内置名仍可被用户词法变量/函数遮蔽，不新增保留名。仅argv本轮支持独立一等取值；其它内置不泛化。已有函数数据面省略规则、错误类12、AST和词法不变。JSON坏数据统一ValueError，不默默截断/null化。编码对象使用数据面，因此不应宣称它是任意LFZ值/引用图的无损序列化。

HashMap内部索引仅影响平均查找成本，不允许改变遍历顺序，也不声称最坏O(1)。JSON解析复杂度 O(n+Σk log k)，编码按实际展开输出规模加对象排序计算，键比较成本另计。设计的渐近约束不是已测加速；性能工程师需测前固定样本、预热、重复次数和验收阈值后再测新版。

## 冻结v2保护

不改 docs/validation/agent-benchmark-v2 的既有manifest/题目/运行器；旧首轮raw-inputs-submissions.zip已保全当时9项输入。开发版skill/目标二进制变化后，旧manifest对开发目录校验失败是预期，不能重写旧manifest掩盖变化。旧基线复现应在项目Temp中还原原9项路径再运行旧验证；后续新版测评另建版本化输入目录和manifest，旧成绩/交卷答案不可改。

## 冻结自查

未新增非终结符，原EBNF无需变更；三个新增名均不与旧内置名/关键字冲突；每项入口和API都有输入/返回/错误/副作用约束，JSON限制覆盖成功和失败清理。自动文本/源扫描及9输入hash核对见 T12-design-checks.json；该检查只证明设计结构与冻结输入，不代替产品验收。
## argv 实现边界复核补钉

初稿把既有内置误认为已有一等函数值并不可遮蔽；源码复核后由追加ADR订正，并同步删除spec的保留名限制。允许Closure显式builtin载荷，仅argv创建；每Interp稳定一个Rc值，用户闭包/占位构造器保留None，统一调用路径先处理builtin，再按原用户函数规则执行。

新增验收：未遮蔽argv==argv、别名同一性为true；type(argv)为function、str(argv)为<fn argv>；let/var/形参/用户fn argv均可遮蔽；遮蔽前保存别名仍读正确脚本参数；数组/struct取调用与高阶回调一致；同名用户函数及无user占位不被误判；旧69内置独立取值仍保持历史行为。argv内置不消耗用户递归帧，错误span采用调用点；不得冒称所有内置已经一等化。
### 精确兼容探针（由后续测试角色执行）

```lfz
#42
let argv = fn() { "shadow" }
assert(argv() == "shadow")
```

```lfz
#42
let saved = argv
fn probe(argv) { saved() }
assert(probe("local shadow") == saved())
assert(saved == argv)
assert(type(saved) == "function")
assert(str(saved) == "<fn argv>")
```

上述第二程序应通过带非空CLI参数执行，并断言saved返回原参数，不能只在空参数场景自比。JSON批量解析不能误写成未支持的一等内置 `map(jsonParse, xs)`；应包装用户函数 `map(fn(x) { jsonParse(x) }, xs)`。本说明只是验收输入，尚未声称在新解释器通过。
## 无直接声明 while 空环境消除审核（2026-10-25）

裁定仅允许契约§16.7窄方案，不泛化exec_block。当前AST直接写当前env的语句恰为Decl/FnDecl/StructDecl；未知名字赋值不会定义。跳过空env对查名/写名/捕获/;;集合和顺序无贡献；空层无槽位，后代child_after的first_slot仍为同一父末槽；self/captured不变。逻辑scope和调试链规范仍保留，物理Env可不分配。

须由runtime/test/verifier构造以下旧版/候选版差分witness；这是一份设计审核，不是已执行结果：

1. 外层var i循环内创建函数表达式存入数组、循环后调用：均观察同一外层cell最终值；函数值身份各自独立。while直接体无声明，应走优化。
2. while直接体仅嵌套if与赋值；if内let k=i，保存fn(){k}：逐轮返回0/1，分支let不泄漏。优化外层但保留内层child与独立cell。
3. while体直接let/var声明、直接fn声明、直接struct声明分别构造三组：保持fresh child；未执行声明也保守回退；循环后同名局部不可见，声明闭包逐轮cell互不串。
4. 函数局部内while的;;、嵌套if遮蔽后的;;、闭包内;;：旧版与新版stdout/stderr逐字一致，包括声明序、内→外、同名最后有效绑定、空作用域无额外行、--json程序输出转stderr。
5. for置于无声明while体内：for迭代变量捕获各自0/1，快照规则不变；函数调用帧/参数与self保留，保存的argv别名仍读上下文。
6. continue/break、if分支return、while条件副作用与假条件零次执行：结果、次数、末值和Flow一致；不丢最后一次条件求值。
7. 外层let重绑定错误、未知名字赋值、循环中用户函数报错：错误类/中文消息/span/traceback逐字一致；内置错误亦不多出/少掉用户帧。
8. 构造空while层→if局部→闭包捕获，并与同父直接if局部对应；槽位首号/同一cell关系保持；必要时runtime单测断言Env::child_after的first_slot关系，不能只比数值结果。

numeric500k固定工作量与门禁阈值保持原样。架构许可不等于已恢复性能，所有计时结论由perf同条件测量；不改源码、不跑新产品、不替代独立验收。