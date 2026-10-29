# T16-B core静态模块图实现

日期：2026-10-30。责任人core-dev。依据三spec §20及T16-B-design；本批是模块前端与非执行图检查，模块初始化/执行仍待C。

新增AST ImportDecl/ExportDecl/ExportName；import/from保留词token不变，export仅语句起始紧邻标识符才识别；Program与block显式top_level上下文，嵌套声明拒绝，import顺序检查，旧export表达式语法保留。深度visitor新增两种叶节点。error.rs新增九个模块SyntaxMsg与精确消息，原LzError类与旧消息不变。

loader新增CountedLoaded/load_file_counted/load_source_counted/load_bytes；原load_file委托同一bytes解码逻辑。原始字节在归一化之前计数，不重读root。新modules.rs持AST/来源外层，显式DFS栈与canonical身份缓存，静态导出核验、循环诊断、节点/深度/字节上限；依赖take(min单文件上限,总剩余)+1限读，按已观测bytes先单后总，不猜测未读取内容。源缓存不持Env，图不做LFZ求值；后继C消费同快照。

旧dist的check import真实缺功能红测SyntaxError/exit2保留。首次cargo test --release --lib modules::tests产品编译成功，4/10通过、6项因Windows受限环境std::fs::canonicalize项目绝对路径Os code5失败，均在测试helper进入图之前。相同已编译harness获提权执行后单diamond1/1、完整10/10通过，产品未为绕权限而修改。然后增加真实byte边界两项，唯一release --no-run重编译成功，最终提权同harness modules::tests 12/12通过、435 filtered、exit0。全部失败/控制/最终日志保留，不把环境差异称产品修复或择窗追绿。

12项覆盖：声明与legacy export标识符；嵌套/次序/路径；BOM/CRLF原字节计数及旧loader等价；菱形共享/依赖后序/快照不重读/while与input不执行；静态首错与前向导出；循环/丢失目标真实来源；依赖解析行号；127/128/129深度；1023/1024/1025节点；合成字节优先级；真实依赖2MiB正好/+1与全图16MiB正好/+1。超限追加0xFF验证预算先于UTF8解析，不制造OOM/abort。

原始manifest绑定core六源码和日志SHA。最终测试前D可用295292928B，约18MiB可重建大夹具在记录各路径/字节/SHA后只按已验证项目Temp子路径清理；小夹具保留，源码中的生成配方可复现。未改runtime/CLI/spec/旧行为测试/Cargo/profile或Git，没有运行CLI全量。完整集成验收由tooling/verifier后续完成，R304OPEN、不批准正式发布。
