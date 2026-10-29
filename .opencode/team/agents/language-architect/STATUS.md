# language-architect — 工作状态
> 最后更新：2026-10-30 by language-architect

## 当前状态
T16-C已按team-lead批准冻结三spec §21与T16-C-design.md，B独立PASS后方GO源码；本角色收工腾slot。T17资源类仍待单独ADR，不改当前错误类。

## 已冻结接口与补钉
- AST ImportDecl/ExportDecl、contextual export兼容、CountedLoaded和owned load_bytes、ParsedEntry/ModuleGraph/GraphFailure、迭代DFS和1024/128/2MiB/16MiB限额。
- 旧无模块root无新图限额；root admission前已加载/解析，不称内存沙箱。B check图不初始化，C前run整Program先无副作用拒绝模块。
- core唯一error.rs SyntaxMsg新增变体及消息；runtime汇合入口在Interp::new前guard，单MODULE帧、originsNone。
- §20.9：import-import后alias_span；对普通声明用普通Stmt.span，不猜列或扩旧AST。导出用ExportName.span。
- §20.10：优先级仅已观测bytes；总剩余较少时哨兵证明总量超限，不能推断未读尾部也超单file，更不能补读越界。常量/消息均不变。

## 后继草案
docs/reports/T16-C-design-draft.md与T17-design-draft.md已落盘，仅草案未冻结。C须同时处理StructDef默认字段Expr定义模块上下文，不能只改UserFn；active恢复与故障source保存分离，T15 Result键不变。T17建议步数/token/deadline拆批，资源终止不被attempt吞、预算不restore；ExecutionLimitError新类需后续明确批准。

## 证据及边界
本次仅读自身状态和当前§20尾部，追加§20.10/ADR及本人记录；未改源码、重编译、测试、看板、C/T17草案或Git。规范diff检查通过；实现/产品验收由对应owner及verifier负责，R304仍OPEN。

## 下一步
B独立过后按team-lead后续派发冻结C；用户已授权全自动继续，无需等用户说继续。本角色不调度其它agent。项目Temp，D盘低空间，不运行大任务。

## C接口提案与T17类决策（2026-10-30）
T16-C-interface-proposal.md10395B：单Interp Session，graph新入口外层GraphExecutionError；复用图快照；home globals与fault source分别恢复/保存；attempt同步source/active但不加Result键。重点StructDef默认Expr目前在caller scope执行，建议legacy None保旧，graph Some TemplateContext记录home/captured/self及默认值诊断帧；实例化用let P=lib.Point; P{}，不扩字段路径文法。CLI折叠trace必须保实际frame index查source，不能切片后重新编号。
T17-resource-error-proposal.md2163B：ExecutionLimitError新增类待单独ADR，绝不伪装Recursion/Overflow，不被attempt吞且预算不restore。文档diff检查通过，无spec/源/编译/Git变更。下一步team-lead裁决后冻结C，B独立通过才改源。

## C正式冻结（2026-10-30）
T16-C-design.md11172B；新eval_graph_traced_on_thread_with_args(&ModuleGraph,&[String])->Result<TracedRun,GraphExecutionError>。宿主结构验证零初始化拒绝，单Interp模块表/namespace缓存/home globals；faultSource独立记录、attempt同步新来源/active快照但固定Result不增键。StructDef.default_context:Option<TemplateContext>，新增struct_template_in_context，旧helper None；graph默认定义home/captured/self，以独立defaults_func_id诊断帧在同Recursion上限保护下执行，Ok Flow清理后传播、Err恢复active但保trace/depth。runtime唯一value/evaluator，tooling唯一CLI/main。
文档diff检查exit0，ADR先追加；未改B冻结段、源码、测试、看板或Git，未编译。B当前584作者测试过不代替独立。下一步由team-lead按B独立结果给GO；R304OPEN。
