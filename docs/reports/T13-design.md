# T13 内部分配优化设计审查

日期：2026-10-27。责任人：language-architect。审查起点：T12候选提交 `cbc974b`。本报告仅给出语义合规边界和待验证项，不宣称实现已落地或性能已通过。

## 结论与事实源

允许两项范围受限的内部实现优化：用户函数绑定参数时直接消费标准库迭代器；无回调的 `join` 在只读借用数组时完成验证与拼接。两项不新增语言行为，不变更文法、AST、公开API、内置签名或错误类；本轮无需扩展 `docs/spec/`。

事实源为 `docs/spec/semantics.md` §4.5.1–4.5.5、§4.5.11及 `docs/spec/interface-contract.md` §10.3、§10.7、§16.6。源码审查包括 `src/evaluator.rs` 的 `eval_args`、`call_func`、`call_value`、`call_user`、`lookup`、`assign_name`、`make_closure`、`visible_entries`，`src/env.rs` 的 `child_after`、`define_named`、`capture_local`，以及 `src/builtins.rs` 的 `b_join`、`need_str`、`need_array`、`array_snapshot` 和高阶内置实现。

## 参数迭代器替代临时 Vec

原高阶路径 `call_value` 先 `args.to_vec()`，再交 `call_user` 逐项移动绑定。许可将私有 `call_user` 的参数载体改为 `ExactSizeIterator<Item=Value>`，直接调用传 `argv.into_iter()`，高阶回调传 `args.iter().cloned()`；每个高阶参数仍只克隆一次，但不再为参数列表分配额外 Vec。必须遵守：

1. iterator只能来自已经求值完毕的参数容器，不能在 `next()` 内再执行LFZ表达式或用户回调。标准 `Vec::IntoIter` 与切片 `Cloned<Iter>` 满足此条件。
2. 在消费iterator前读取准确长度；保持原顺序：显式 `argv` 内置标签检查与arity、占位闭包检查、用户函数arity、递归深度检查，然后建调用环境和绑定。`argv()` 返回新数组、不产生普通用户调用帧的原行为不变。
3. 每次用户调用仍分配新的逻辑调用环境。参数按声明序绑定、均可重绑定；相同参数名的既有后者遮蔽规则不变。不得复用逃逸闭包已经捕获的cell，不得把上次调用值留入下次调用。
4. 数组/struct参数克隆只共享原Rc；修改容器仍对别名可见，形参重绑定只影响该次形参cell。字符串仍按不可变值语义；延迟一次Rc计数增减不属于LFZ可见副作用。
5. 保持 `self_override` 优先于定义处self，以及原captured列表。每调用深度递增、调用者span更新、callee帧初始化和压帧顺序不变；仅成功返回才退深度/弹帧，错误保留原traceback。
6. `map`、`filter`、`reduce`、`sortBy`、`minBy`、`maxBy`、`each` 的源数组快照不可删除。用户回调能修改原数组及别名，不能在回调期间持源数组的Ref借用。

此改动消除的是临时参数容器，不能被描述为消除调用帧或改变每调用绑定语义。克隆移至参数绑定时发生，不影响源程序表达式求值顺序，因为 `eval_args` 仍先完整左到右求值。

## join 只读借用替代 Value 快照

原 `b_join` 在读取 `sep` 和数组后 `xs.borrow().to_vec()`，再验证字符串元素、建立 `Vec<&str>` 并拼接。许可直接持 `xs.borrow()`，在该借用下完成元素验证及 `strs.join(sep)`，省去Value序列快照及其中字符串的Rc计数往返。理由是整个操作同步、不执行LFZ代码、不调用用户函数，Rc容器也不跨线程并发；在该时间段不存在可观察到的数组修改点。

边界：

- 保持参数检查顺序 `sep` → 数组 → 从左到右首个非string元素，错误类、中文消息和调用span不变；不可静默把非string转字符串。
- 空数组、空分隔符、空元素、多字节Unicode、NUL均按现有字节拼接行为；返回不可变新字符串，输入数组及元素保持原值。
- 持Ref期间只读元素，不能进入任何回调、递归求值、用户格式化或可修改该数组的路径。非string元素报错只需类型名，不得为格式化该值再次借用输入数组；自引用数组应受控TypeError，不得RefCell panic。
- 此许可不泛化到高阶内置。保留 `Vec<&str>` 与原拼接算法即可；不因本次审查擅自扩大为容量处理或其它字符串算法变更。

## 交付给独立验收的差分矩阵

| 类别 | 最少见证 | 对照要求 |
| -- | -- | -- |
| 直接/高阶参数传递 | 0、1、多参数及map/reduce回调；空源数组 | 返回值/输出一致，空源不额外调用 |
| arity与错误 | 直接函数、map错误回调arity、占位非函数、argv别名错误arity | 错误类/消息/span/完整traceback及退出码一致 |
| 每调用fresh | map回调返回捕获可变形参的闭包，之后逐个调用 | 各闭包保存当次cell，相互独立 |
| 重绑定与共享 | 回调重绑定形参；array/struct参数原地修改 | 重绑定隔离，容器共享按A1保留 |
| 递归/self/遮蔽 | 方法内高阶回调捕获self；全局/局部同名及argv遮蔽 | 捕获和解析顺序不变，错误调用帧不丢失 |
| 高阶快照 | 回调改源元素、改源绑定、修改嵌套容器 | 原顶层元素序列快照与嵌套共享各自保留 |
| join成功 | 空数组/空sep/空元素/Unicode/NUL/重复共享字符串 | 输出字节和输入后续值一致 |
| join失败与环 | sep非string、第二参非array、首个非string、含自身的array | 原首错TypeError、受控exit2，无panic |

本表是测试设计输入，不是已经执行的验收结果。运行时作者单测、verifier旧新差分与perf按原门槛的实测各自提供证据；本报告不替代它们。

## 后续功能方向（本轮未纳入）

优先考虑结构化数据契约检查：让Agent在管道边界显式验证字段、类型与路径，并得到确定性机器可读错误。它可减少应用手写验证与失败重试。但这属于新语义/接口，应另立需求与契约，不能塞进本轮内部优化；在当前性能门槛复验后再由team-lead选择。
