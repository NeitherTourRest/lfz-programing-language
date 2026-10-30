# T16-C 单session模块运行时：库阶段交接

日期2026-10-30。责任人runtime-dev，依据三spec §21；源码已交接给tooling，尚非最终CLI验收。

## 实现与API

新增公开GraphExecutionError（InvalidGraph/Origins，Debug+Display+Error）及：

```rust
pub fn eval_graph_traced_on_thread_with_args(
    graph: &crate::modules::ModuleGraph, args: &[String]
) -> Result<TracedRun, GraphExecutionError>
```

调用者必须已在on_eval_stack闭包内部完成根前端、build_graph，再借graph调用本API；消费完成后在同一大栈drop graph。不要把包含Rc<SourceMap>的图从调用线程捕获进要求Send的on_eval_stack闭包。本API不重读、不重复parse、不克隆全部Program。外层Err是坏宿主图或origin元数据不变量；内层TracedRun.result是LFZ业务结果，不能把宿主错误包成attempt成功记录。

完整预检在任何初始化之前检查entry/units编号、唯一来源与Loaded、初始化全排列且entry最后、每依赖先初始化、AST声明及alias/span与import metadata顺序一致、alias唯一且不与直接绑定冲突、export名称/span/顺序/绑定及重复校验。它不证明宿主伪造AST与文本一致，不是安全沙箱。

单Interp持会话内模块实例/Env/namespace状态，按依赖后序串行初始化；import为不可重绑定alias并共享Ready命名空间，export构造完成时值快照，容器/函数保留原共享语义。初始化失败立即停止，entry未执行。无全局缓存、多Interp互调或隐式全局注入。旧eval_module*根模块声明guard保持。

UserFn用整数home_module选择定义Env，call_user明确收集结果，成功和错误均恢复caller globals/module；错误保留故障trace/depth。同步trace_sources与首次fault_source保证恢复caller后仍保留实际定义文件。attempt恢复全部调用状态/来源，保持旧Result/error/frame字段，不回滚funcID、模块实例或用户副作用。

TemplateContext保存定义模块/captured cells/self/defaults_func_id；graph模板用新struct_template_in_context，旧struct_def/struct_template填None并保持旧行为。默认字段在定义上下文依声明序重新计算，显式覆盖在恢复caller后求值。默认表达式诊断帧使用TemplateName::<defaults>、定义SourceId与受RECURSION_LIMIT保护的depth，不吞Return/Break/Continue。无默认字段不加帧，方法调用沿既有self规则。新增StructDef.default_context是明确公开Rust构造字段迁移。

## 实际测试与失败记录

新增9项C unit：模块全局/两session隔离；模板定义全局/局部cell及caller覆盖；跨模块attempt、逃逸闭包及随后错误来源；函数/默认帧真实来源；快照alias/self/argv；7类坏宿主图预拒绝；依赖初始化失败不执行entry；安全合成默认深度上限；默认表达式return向外传播。

真实红测为新API缺失的E0425，red.txt保留。作者中间错误也保留：首次test把Rc图跨Send闭包，修为闭包内构图；两fixture误用struct字段x=，按旧语法改x:；资源测试对非Debug Flow使用unwrap_err，改显式match，不改产品类型迁就测试。

定向7项通过后增加2项。完整cargo test --release --lib默认执行为448通过/8失败，全部8失败来自既有modules.rs测试canonicalize路径PermissionDenied，非C断言失败；保留lib-corrected.txt。主会话将**同一已构建release lib harness**提升环境串行复跑，最终456/456，0failed/ignored，日志lib-elevated-parent.txt保全。没有改旧模块测试来追绿，没有再次编译替换该harness。新增C9项均真实运行通过。

本阶段未编译CLI最终产物、未运行完整release各harness或公开CLI验收；tooling将唯一负责后续完整编译。库harness SHA256 2b7e947e60a4e57afd2fdbf7142cd360c75a333ae1e982d86ffe38db1436ce90。src/value.rs SHA63967cb79996c7fedfe7285378e58c2774825c73680a81e50c12a185bbe438a0；evaluator.rs SHAa5cf87f647d7c3840c6eb75cd625ce457173e9db592eff5107a781437caa4095。manifest记录精确字节与所有原始日志。

## 交接限制

只修改value/evaluator、新增C单测及作者证据/自己的记忆。不改B AST/loader/modules/source、CLI、spec、旧测试、profile、dist或Git。源码停止改动，tooling可按§21.6对真实来源渲染和run graph分流。库测试仅合成图及有界状态，后续须真实文件/菱形/CLI诊断/折叠trace/公开和独立验收。全部临时项目Temp，仅复用release；磁盘约196MiB。不制造真实abort/OOM；R304OPEN，无性能计时或正式发布批准。
