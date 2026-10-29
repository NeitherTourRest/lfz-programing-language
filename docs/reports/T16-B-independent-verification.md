# T16-B 静态模块图独立验收

日期：2026-10-30。责任人：verifier。结论：B静态声明/模块图、非执行check、run执行前拒绝及legacy兼容PASS。B不提供模块初始化/执行；C仍须独立实现与验收。R304OPEN，无性能测窗，不批准正式发布。

## 固定产物和执行证据

本次target/release/lfz.exe 859136B，SHA256 `8ea375dcd08098ef61d040a57a7b1ad28a28c2fdc25a5ca4921a75b728944480`。没有编译或修改产品。初始源码/Cargo在initial-sources.json，自编图见证末尾复核一致；全部流程在C改源之前完成。后继C工作树与本次冻结指纹不相同不表示本窗污染，不能用C二进制冒充本报告对象。

| 实际独立执行 | 结果 | 原始证据 |
| -- | -- | -- |
| 同产物8个release harness，安全提权实际执行 | 584/584 | harnesses.json |
| B公开模块图过程矩阵，独占Temp复跑 | 38/38 | public.json、public.py |
| 自编独立图/非初始化/错误路径与边界 | 18/18 | graph-witnesses.json、graphs.py |
| 既有attempt | 13/13 | attempt13.json |
| 既有check | 117/117 | check117.json |
| 既有CLI full-suite | 91/91，内含官方178/178，不重复累计 | cli91.json |
| jsonflow / sortviz / jsonbatch | 37/37、11/11、38/38 | 同名JSON |
| 旧JSON/argv独立见证 | 140/140 | independent140.json |

证据目录T16-B-independent-evidence，manifest绑定原始字节。复用旧脚本内历史date/schema不是本轮执行日期，当前产物hash与本报告日期为准。

## 独立图见证

以不同于公开矩阵的独占项目Temp目录构造三来源file/-e/stdin菱形依赖，依赖源码含writeFile(existing)、writeFile(missing)、input和while true。check成功、双流按协议，已有哨兵含NUL/CRLF原字节保留，新目标不创建；human模式也双流空。菱形成功是check行为见证，单个check结果本身不用于证明缓存节点数，图内部唯一实例不冒称模块运行。

循环A→B→A定位真实B文件；子文件语法错定位真实子文件第2行；非法UTF8、前导错误及缺文件IO错误均报告真实依赖路径，不拿root文本下划线冒充。无span IO traceback为空。root本地import冲突先于缺依赖，符合当前本地检查优先。run中writeFile在late export前仍零副作用拒绝“模块执行尚未启用”，没有执行前面的写入。

legacy var export表达式依旧运行输出5。超过2MiB且无模块声明的旧源码同时check及run成功；依赖原始字节恰2MiB通过，多1字节受控SyntaxError；128模块深度通过，129层在导入者真实文件受控拒绝。此处是真实有限字节/深度，不用超大OOM/abort证明；节点1024、总量16MiB及§20.10优先级另在实际执行的core单位harness内覆盖，不声称18进程穷举所有组合。

## 权限控制和验证器修正

本环境默认沙箱canonicalize会AccessDenied，作者已保留默认439/447及同产物许可控制；本独立验收使用require_escalated执行同一8个冻结harness和图进程，仅写项目Temp，不编译或放宽产品路径身份规则。独立584全通过不代表默认权限环境限制消失。

验证器初版从中止的cargo full-tests日志提取harness，实际只读到1个路径，在启动前因期望8个而失败；run-initial.py保留，后按作者已保存的8个.exe.txt名称列出同产物harness，逐个hash和执行，不改测试内容。自编初版误用let export后重新赋值，旧语义正确抛TypeError；graphs-initial.py保留，仅将该兼容例改var后重跑18全过。这两项是验证作者错误，不是产品缺陷，不将其计入产品失败或隐藏。

## 范围限制

A类型和T15Result固定字段未扩展；B静态图不初始化，run及旧eval接口执行前拒绝。没有模块执行、缓存初始化次数、跨模块home globals运行或C attempt恢复身份的验收结论。没有RSS/计时、真实资源耗尽、Git或实现修复。最终交付与R304由主调度处理。
