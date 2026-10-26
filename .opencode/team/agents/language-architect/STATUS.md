# language-architect — 工作状态
> 最后更新：2026-10-25 by language-architect

## 当前状态
T12-01设计契约及argv最小一等载荷补钉已冻结；已通知team-lead可安排runtime/tooling实现。没有声称新功能已实现或已加速。

## 产出与证据
- docs/spec/syntax.md §16：argv分隔符、-e/run-e、-/run-、--json选项边界、stdin UTF8/EOF、原入口兼容。
- docs/spec/interface-contract.md §16.1–16.5：72内置目标、两with_args求值API、一等argv上下文、严格JSON/i64/Unicode/重复键/深度128/环/精确ValueMsg、内部struct字段索引许可。
- docs/spec/semantics.md §16：显式环境输入、数据面与引用拓扑往返边界。
- docs/reports/T12-design.md可执行验收矩阵；T12-design-checks.json：自动检查结果如文件所载，不代替产品验收。
- DECISIONS追加两ADR；首条标题时间误写已紧随后续条目订正，历史条目未改。旧v2 9输入与manifest未修改。

## 进行中 / 阻塞
无设计阻塞。实现/测试/性能由team-lead后续调度；本人不等待产品实跑，不改源码、测试、看板或他人状态，无git操作。

## 紧急补钉证据
- 源码核查发现旧内置非一等值且可词法遮蔽，撤回初稿保留名假设；追加ADR、契约§16.6、三spec及报告同步。仅argv支持一等值，Closure.builtin显式标记，每Interp同一Rc、type=function、str=<fn argv>；别名保存/形参遮蔽探针已纳入报告。旧69及JSON二内置不泛化，JSON map须用户fn包装。
- T12-design-checks.json新增三项边界结构检查PASS，共20项；未实现、未运行新产品测试。

## 空环境优化审核
- 按轻量任务复核exec_while/exec_block/赋值/闭包捕获/;;/first_slot，许可仅while无直接Decl/FnDecl/StructDecl体一次扫描、每轮不物化空Env。for与其它exec_block及任何声明体保持原fresh环境。追加ADR、契约§16.7及报告8类差分witness；不实现、不声称性能恢复。

## 下一步
按spec实现并独立验证。开发skill/二进制更新后，旧v2校验仅在项目Temp恢复9输入的工作区复现，不能覆盖历史manifest；新轮另建输入版本。

## 关键经验
JSON codec必须独立于历史宽松runner codec；对象构造不反复线性set；序列化只按当前递归路径判环。保留既有A5方法过滤，不能擅自将函数方法字段拒绝。效率要计真实输出展开规模和字符串键比较成本，避免只看唯一图节点数。