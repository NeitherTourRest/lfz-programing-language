# T16-A core 来源元数据基础

日期：2026-10-29。责任人：core-dev。依据三spec §19；这是模块前置元数据，不代表import/export或模块运行已实现。

新增src/source.rs与lib.rs公开声明，提供分域SourceId/ModuleId及ENTRY、SourceRecord、SourceMap、SourceMetadataError与LocatedFailure。SourceMap只持元数据及可选Loaded，插入前checked转换编号，未知ID返回None；不加载、规范化或合并路径，不持Program/Env/用户Value，不修改Span、Loaded、TraceFrame、LzError、Program与旧parse形状。

先用项目Temp中的外部Rust消费见证调用旧liblfz.rlib：明确E0432找不到lfz::source、rustc exit1。它是新API缺失的真实红测，不是在产品里放编译错误。随后新增模块及6项单测，cargo test --release --lib source::tests结果6 passed、0 failed/ignored、419 filtered，exit0。并未宣称同一外部见证已在新发行rlib重新链接；新source单测本身已编译并调用全部本批API。后继runtime最终构建会统一刷新rlib。

六项覆盖：两个ID类型/ENTRY及Hash；插入顺序稳定与越界None不回落入口；Loaded含BOM/CRLF/Unicode原字节与line_base完全保留、虚构路径无IO；加载失败缺源码记录；私有checked编号转换在u32最大值及可表示的超界usize处验证（不分配数十亿记录）；LocatedFailure完整保留旧错误class/message/span以及旧struct literal可编译。

源码和日志SHA/字节绑定见T16-A-core-evidence/manifest.json，原始红绿日志保留。工具链、TEMP/TMP和所有临时均在项目内；只复用release target，未改profile、旧测试、AST/parser/loader/error或runtime/tooling，未Git。环境存在canonicalize警告，不宣称零warning。完整功能、runtime来源sidecar和性能验收由后继角色执行，R304仍OPEN。
