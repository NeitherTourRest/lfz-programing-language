# language-architect — 工作状态
> 最后更新：2026-10-29 by language-architect

## 当前状态
T15 attempt受控错误恢复契约已按team-lead确认冻结于三spec §18及T15-design.md，runtime可实现。未修改源码、测试、看板或Git，未宣称功能/效率通过，R304仍OPEN。

## 产出/关键决定
- attempt(f)零实参调用，固定Result ok/value/error；error class/message/line/col/traceback，帧func/line/col完整外→内，无file。
- 外层/回调arity与函数值校验在边界外；普通求参一次左→右。argv可调用，attempt不一等，变量遮蔽保持。
- 可恢复Name/Type/Index/Field/DivZero/Value/Io/Assert和整数Overflow；前端错误、Recursion与容量Overflow不捕获，禁止宿主catch_unwind伪装业务结果。
- 仅成功/可恢复失败恢复depth和完整trace/span；错误数据先取。不可捕获原样保帧传播。副作用、func_names和next_func_id不回滚，防逃逸闭包错配。
- Ident上下文特殊派发、BUILTIN_NAMES73/is_builtin登记；lookup/lookup_hof不登记attempt，无Interp call/call_with为NameError，旧Invoke ABI保持。
- DECISIONS仅追加21:03 ADR；用户初始块/旧记录未改。文档检查不等于实现验收。

## 下一步/阻塞
runtime按§18实现，test/verifier按矩阵验证；无设计阻塞。仅team-lead统一调度，本角色不再调度。

## 关键经验
错误恢复不能只truncate帧：调用者span已被改写，必须完整恢复。函数标识表回滚会破坏失败回调写出的逃逸闭包。资源错误分类按variant，不按中文message。

## T16后继草案（2026-10-29）
已完成docs/reports/T16-design-draft.md（10619B），只读审查loader/AST/lexer/parser/Interp/UserFn/CLI当前单source限制；未改spec或源码。建议import name from静态路径，非全局关键字export名列表，执行内缓存/循环拒绝/独立globals与home module；拆T16-A source定位基础→B无执行图检查→C运行时→D验收。T15 Result原固定字段不加file，CLI跨文件定位新增外层source map；需另批协议才扩Result。当前仅草案，待team-lead批准，不宣称实现。
