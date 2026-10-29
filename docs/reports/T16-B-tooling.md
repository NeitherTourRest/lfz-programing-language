# T16-B 模块图check CLI作者交付

日期2026-10-30。tooling-dev。消费三spec§20（含20.9/20.10）及core CountedLoaded/ParsedEntry/ModuleGraph接口；不改核心、运行时、规范、既有测试或Git。

check文件/-e/stdin同一次counted加载保留原始字节，根仅lex/parse一次。根无直接模块声明时不构图、不启用2MiB/16MiB预算，保持旧前端诊断；根有声明时将Program、Loaded、raw_bytes、显示路径、canonical身份和base移交build_graph，成功后大栈内drop图，不初始化/求值。CODE/stdin在检查开始捕获cwd，文件相对导入基于实际文件父目录。不存在metadata猜测、根重读或二次parse。

GraphFailure按source_id取真实SourceRecord，用其display_path和可选Loaded构建前端诊断；无Loaded不显示入口文本。JSON字段及顺序、check executed:false、CLI usage规则保持。加载、解析、构图及所有Program/图析构都在on_eval_stack内；run的模块执行副作用前拒绝由runtime既有preflight提供，CLI不自行实现初始化。

## 验证证据

- 旧dist模块check红见证：合法模块图预期exit0，旧实现exit2，原始输出red.json保留。
- 新tests/cli_t16b.rs七组覆盖菱形/共享依赖且无写入或循环执行、CODE/stdin cwd、依赖语法的真实路径/源码文本、缺文件与非法UTF8、冲突/导出/环、旧export标识符、无模块>2MiB根例外和有模块根限额、run执行前零副作用拒绝。子进程10秒有界超时，全部fixture项目Temp/T16-B-tooling。
- 默认沙箱定向2/7：依赖canonicalize被拒造成5个IO失败；保持原实现与测试，对同一已编译harness按授权提权运行7/7 exit0。
- cargo test --release默认运行到lib：439通过、8个模块身份测试因同权限失败后中止。保留完整失败日志，不修改身份策略或跳过测试。
- 按授权对同一八个已编译release harness提权全量运行：584/584（447lib+48bin+28cli+12T12+12T14+7T16B+15runner+15unit），0失败/忽略；无重编译、修改源码或替换用例。cargo build --release exit0。

原始日志、red.json及manifest在docs/tooling/evidence/T16-B（-text保留原字节）。manifest绑定全部当前src/Cargo和新增CLI测试hash，记录默认失败与许可环境下的同产物结果；不把默认cargo test说成直接全绿。

最终target/release/lfz.exe859136B，SHA256 `8ea375dcd08098ef61d040a57a7b1ad28a28c2fdc25a5ca4921a75b728944480`。本角色未覆盖dist、变更profile、安装外部资源或Git。完成后不再构建/测试/改源；独立验收、发布复制与C阶段由team-lead接续。B只静态图与链接，不宣称模块执行完成，也不关闭R304性能项。
