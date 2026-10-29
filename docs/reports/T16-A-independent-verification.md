# T16-A 独立来源基础验收

日期：2026-10-29。责任人：verifier。结论：§19新宿主来源元数据API、legacy语言行为及CLI兼容PASS。本批不提供import/export或真实多模块求值；R304仍OPEN，无性能计时，不批准正式发布。

## 产物及执行

本次执行target/release/lfz.exe：783872B，SHA256 `65a29eaea54f32d1d2b404f22aed09389c3ffa71e6a81acb8fa6351d32fa42b7`。旧T15副本782848B，SHA `1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c`。不依赖尚待发布复制的dist。源码/Cargo测前后指纹一致，证据differential.json及initial-sources.json。

| 独立实际执行 | 结果 | 证据 |
| -- | -- | -- |
| 7个release harness直接运行，无cargo重编译 | 562/562 | T16-A-independent-evidence/harnesses.json |
| attempt公开见证 | 13/13 | attempt13.json |
| check公开见证 | 117/117 | check117.json |
| 原CLI/full-suite | 91/91，内含官方178/178，不重复累加 | cli91.json |
| 旧应用jsonflow/sortviz | 37/37、11/11 | jsonflow.json、sortviz.json |
| jsonbatch | 38/38 | jsonbatch.json |
| 旧JSON/argv独立见证 | 140/140 | independent140.json |
| 新Rust API见证程序 | 编译及执行exit0 | witness.rs、witness-result.txt |
| 新旧同路径字节差分 | 16程序×2模式=32/32组，64进程 | differential.py、differential.json |

公开复用脚本中schema/date可能含T12历史常量；本次实际运行日期以本报告及新manifest为准，产物hash逐脚本绑定当前65a29，不把旧标签当实测日期。

## 新API与有界审查

独立Rust witness直接消费现release liblfz.rlib，验证空map与稳定追加ID、未知ID None、Loaded原文/CRLF及line_base保留、loaded=None、多帧不同来源、legacy无sidecar、安全索引、校验顺序FrameCount→frame UnknownSource→结果error_source条件→error UnknownSource、失败保留旧有效来源、修改公开frames/result后的安全查询、闭包home_module为ENTRY。实际顺序与§19一致，非只凭作者单测。

源码只读核查insert先checked编号再push，SourceMap只持metadata/Loaded；不持AST/Env或用户Rc值、不做canonicalize或文件加载。LocatedFailure保留旧Box错误；core六项单测包括原class/message/span保存及合成编号上限，已在562 harness内真实执行，不为容量边界分配巨大表。runtime七项单测包括active/home ENTRY、attempt状态恢复，已实际执行。

新增私有sidecar不改变旧Span/LzError/TraceFrame/Loaded/Program。UserFn公开home_module字段是明确的Rust构造API迁移，不声称外部手写literal完全不受影响。legacy eval入口origins=None；CLI不读取新sidecar，T15Result固定字段不变。真实home globals切换和跨文件诊断尚属后继，合成两来源元数据不代表已经运行两模块。

## 差分与编译配置记录

同一个临时源码路径与相同argv运行旧、新字节；human和JSON均比较完整stdout/stderr/exit。覆盖普通函数/闭包、self、argv、管道、dump、成功nil、恢复JSON错误、嵌套attempt、共享别名副作用、失败写出逃逸函数、后继未捕获错误、外层arity、受控容量传播、NameError和SyntaxError。所有预期错误类也独立断言。旧新一致是有界回归证据，不是穷举证明。

初次rustc默认配置链接带LTO的release rlib时LNK1107/未解析符号，日志witness-build-initial.txt保留；没有生成或运行失败候选，不属于产品运行缺陷。用相同库加`-C lto -C opt-level=2 -C codegen-units=1 -C debuginfo=0`后链接成功并运行PASS，日志witness-build-lto.txt。没有重建库/新target或改变源码、profile、dist。

复现时只在获主调度编译窗口准许后执行项目工具链rustc，TEMP/TMP置项目Temp，命令：

```text
rustc --edition=2021 -C lto -C opt-level=2 -C codegen-units=1 -C debuginfo=0 docs/reports/T16-A-independent-evidence/witness.rs --extern lfz=target/release/liblfz.rlib -L dependency=target/release/deps -o Temp/T16-A-verifier/witness.exe
```

无真实abort/OOM/大模块链压力测试；没有性能计时/RSS测量、Git或实现修复。
