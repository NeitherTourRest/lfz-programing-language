# core-dev — 工作状态
> 最后更新：2026-10-30 by core-dev

## 当前状态
T16-B core按§20实现完成，12新unit全部通过；停止编译，tooling接续唯一全量构建。

## 产出
src/ast.rs/error.rs/loader.rs/parser.rs/modules.rs/lib.rs；T16-B-core.md及原字节证据/manifest。AST模块声明、顶层上下文、静态图迭代DFS、canonical缓存、来源与有界读取/预算齐备。B不运行模块，C后继执行。

## 证据
旧dist check import红测exit2。首次release编译成功但sandbox测试4/10，6项helper canonicalize Os5；相同harness提权单1/1和完整10/10通过，未改产品绕过。增加真实byte边界后release no-run成功，最终提权12/12通过/435 filtered。失败/控制/最终全部保留。

## 下一步/阻塞
无core阻塞。tooling集成CLI并独立验收；若canonicalize仍受沙箱限制，应如实使用已授权非沙箱验证，不改变身份契约。不编译、不Git、不调度。

## 经验
旧Span/Loaded/TraceFrame/LzError/Program形状不改，来源外层。判定字节超限仅按限读观测数据，不推测未读实际长度。约18MiB大夹具先保SHA再按核实Temp路径清理，D低空间只release。
