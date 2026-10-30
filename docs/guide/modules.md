# 本地模块：从检查到执行

本章引用三spec §20–21及T16-C-design，描述C模块执行候选。最终产物与实跑结果见[T16-C指南证据](../reports/T16-C-guide-evidence.md)，R304仍OPEN，不代表性能或正式发布完成。

## 一个三文件程序

同目录保存三个UTF-8文件，每个.lfz首行均为#42。`calc.lfz`：

```lfz
#42
var base = 40
let shared = [1]
fn add(x) => base + x
fn bump() { base += 1
    base }
export base, shared, add, bump
```

`worker.lfz`：

```lfz
#42
import calc from "./calc.lfz"
fn work() => calc.add(2)
export work
```

`main.lfz`：

```lfz
#42
import calc from "./calc.lfz"
import worker from "./worker.lfz"
var base = 900
assert(worker.work() == 42)
assert(calc.bump() == 41)
assert(calc.base == 40)
calc.shared[0] = 7
assert(calc.shared[0] == 7)
print(jsonStringify({answer: calc.add(2), args: argv()}))
```

在三文件目录执行（解释器路径按实际项目位置调整）：

```powershell
lfz check --json main.lfz
if ($LASTEXITCODE -eq 0) { lfz run main.lfz -- '中文' }
```

检查成功JSON为 `{"ok":true,"mode":"check","executed":false}`，不执行任何模块初始化或脚本内IO。运行才输出 `{"answer":43,"args":["中文"]}`。依赖路径相对于导入者文件目录，不是进程工作目录；-e/stdin根使用明确的当前目录基址规则。路径须为无插值的相对.lfz字符串，可含..；不搜索扩展名，不支持绝对/网络/远程包路径。所有import须位于任何其它顶层语句之前，export也算其它语句。

## namespace、缓存与函数上下文

显式export只能导出本模块直接顶层绑定，可用逗号列多个名字；未导出名字不能从namespace读取。每次运行独立session，按依赖后序串行初始化；同一个canonical身份的模块只初始化一次，菱形依赖共享同一个namespace。不同运行不复用缓存。

namespace是普通struct：导出标量为初始化完成时快照，修改namespace字段不会写回原模块变量；数组/struct/函数沿既有引用规则共享。上例bump改变定义模块base，calc.base仍40，calc.add读取的却是定义模块当前base。函数按home globals执行，调用者同名base=900不会污染它。局部闭包cell和self仍遵守原规则。所有模块共享本次脚本argv输入快照，每次argv()仍返回新数组。

namespace的keys/JSON沿A5过滤函数方法，不可用来枚举或完整序列化导出函数。`attempt`结果仍只有既有三/五/三字段，跨文件错误记录没有file；不要把它冒称完整文件定位。最终未捕获错误的CLI人类/JSON诊断会使用各帧真实文件，调用参数校验错误属于调用者，函数体错误属于定义文件。

## 模板默认值与旧行为

导出模板不能写 `lib.Point{}`；先 `let P = lib.Point` 再 `P{}`。graph模块模板的默认表达式在定义模块globals、捕获cells与self上下文中按字段声明序重新求值；每实例可变默认独立。显式覆盖字段在所有默认值之后、恢复调用者上下文后按调用者作用域求值。默认值失败可出现 `Point::<defaults>` 诊断帧和真实定义文件。

无模块legacy模板仍保持原caller scope默认值行为，不增加该默认帧。不要把graph新规则套成旧程序从来如此。attempt恢复后模块上下文/来源也恢复，但副作用、初始化实例、函数ID不回滚。

## 错误与限制

check先构建完整静态图：循环导入、重复alias、与顶层绑定冲突、导出缺失名字、坏路径或依赖语法错误均在初始化前拒绝。依赖初始化运行失败则整个session停止，入口尚未运行的attempt不能捕获它；无隐式重试。读取namespace未导出字段则是实际读取点的FieldError，区别于export声明引用不存在绑定的图错误。

图上限为1024节点、含root128层、单文件2MiB、原始总bytes16MiB，具体admission与legacy根例外以spec§20为准。这些不是OS安全沙箱，读取源码仍是宿主IO。宿主伪造坏ModuleGraph或来源不变量错误由GraphExecutionError拒绝，CLI stdout空、stderr宿主上下文错误并exit2，不伪造成LFZ业务成功。

## Rust嵌入API迁移

旧eval_module*保留无模块行为，遇带模块声明的Program仍guard拒绝，不能把多个Program拼接成隐式共享globals。模块宿主先按§20构建ModuleGraph，再在on_eval_stack内调用 `eval_graph_traced_on_thread_with_args(&graph, &args)`；parse/build/eval/drop graph全部留在大栈，不重读文件或克隆全部Program。外层Result错误为宿主InvalidGraph/Origins，内层TracedRun.result才是LFZ结果。

A阶段UserFn新增home_module，外部手写结构体须按§19补对应字段；C阶段StructDef手写literal须补default_context。旧Value::struct_def/struct_template helper仍提供None，graph定义上下文用新增struct_template_in_context与TemplateContext（home_module/captured/self_val/defaults_func_id）。精确签名以interface-contract §19/21为准，ModuleId与SourceId不可互换。无Loaded来源不能借其它文件文本渲染；GraphExecutionError不是新的LFZ错误类，不被attempt吞掉。
