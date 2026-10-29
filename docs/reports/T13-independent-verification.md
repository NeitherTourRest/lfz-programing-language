# T13 独立功能与语义差分验收

日期：2026-10-27。责任人：verifier。阶段：功能验收完成，性能评估尚待本轮冻结窗口及独立数据审计；不批准正式发布。

## 产物与源码核对

`dist/lfz.exe` 与 `target/release/lfz.exe` 均为761344B，SHA256 `9cda0c6fd5d85178335e8b758a843fc5f66574826bab4a80f2418a318e171cf9`。逐项核对release-manager的source-hashes.json；源码和Cargo指纹一致。生产改动限于参数迭代器与join借用，只有builtins/evaluator及8个新增unit测试发生改动，旧测试、profile、Env和值模型未修改。

源码审阅确认：参数载体消费前仍按原次序检查argv内置/arity/recursion，fresh调用帧、mutable参数、self及错误保帧逻辑保留；直接路径使用标准owned迭代器，高阶路径使用标准slice clone迭代器，无用户代码惰性求值。join持借用的路径只有字符串验证与拼接，无用户回调，非string错误只取type_name；7个高阶内置的源数组快照仍保留。源码理由与进程差分共同支持本次有界合规结论，不冒称形式证明或所有程序已穷举。

## 实际执行结果

| 验证 | 结果 | 原始证据 |
| -- | -- | -- |
| 6个release测试harness | 530/530（412+48+28+12+15+15），0失败/忽略 | [unit-harnesses.json](T13-independent-evidence/unit-harnesses.json)、6份unit-harness原始txt |
| 公开CLI检查 | 91/91，含官方全量166/166 | [cli.json](T13-independent-evidence/cli.json) |
| jsonflow / sortviz应用 | 37/37及11/11；真实IO及旧默认兼容均通过 | [jsonflow.json](T13-independent-evidence/jsonflow.json)、[sortviz.json](T13-independent-evidence/sortviz.json) |
| 独立JSON/argv等见证 | 140/140，原9冻结输入及99旧清单条目仍保持 | [independent-140.json](T13-independent-evidence/independent-140.json) |
| 本轮独立参数/join差分 | 31程序、8类别、human/JSON两模式62组比较，124个进程 | [differential.json](T13-independent-evidence/differential.json)、[differential.py](T13-independent-evidence/differential.py) |

530项由verifier直接执行作者cargo日志列出的6个确切release测试harness，未重新编译，原始输出全部保全；集成harness使用构建时绑定的release解释器。CLI91内部已执行官方166，不能把166与91当互不重复的独立用例数相加。既有CLI脚本的schema/date仍硬编码T12/2026-10-25，本轮实际执行日期为2026-10-27，产物指纹为9cda；没有修改旧脚本或旧证据来改写其历史。

差分使用同一个脚本绝对路径、相同参数和stdin，分别执行保留的T12 47ed与新T13 9cda，核对stdout/stderr原始字节及退出码。成功由脚本内独立assert判定；负例另外校验期望exit2与错误类，避免旧新同时失败被误算通过。覆盖8类：直接/管道/高阶参数流与空源不调用、参数cell逃逸且每次fresh、重绑定与array/struct共享、递归/self/argv遮蔽、7个HOF逐一源快照（含nested alias）、join Unicode/NUL/空项及alias、join首错/自环受控错误、arity与嵌套错误完整trace。自环构造后join返回TypeError，未触发panic/abort；无崩溃压测。

## 复现命令

从项目根目录，证据输出路径要求不存在；要复跑请另用新的输出文件。自编差分脚本当前输出固定且拒绝覆盖旧证据，复跑前仅在项目Temp中复制脚本并调整输出路径。

```powershell
D:/app/ANACONDA/python.exe -B tests/t12_cli_check.py --lfz dist/lfz.exe --output Temp/T13-verifier/cli-new-run.json --full-suite
D:/app/ANACONDA/python.exe -B app/verify_jsonflow.py --lfz dist/lfz.exe --output Temp/T13-verifier/jsonflow-new-run.json
D:/app/ANACONDA/python.exe -B app/verify_sortviz.py --lfz dist/lfz.exe --before-exe Temp/t12-perf/lfz-before.exe --output Temp/T13-verifier/sortviz-new-run.json
D:/app/ANACONDA/python.exe -B docs/reports/T12-independent-evidence/check.py --output Temp/T13-verifier/independent-new-run.json
```

本轮证据目录用`* -text`保留原始行尾字节并附manifest。未更改src/spec/官方测试或T12历史材料，未执行Git。性能未在本角色功能阶段计时；R304仍开放，等待perf冻结窗口结束后独立审计原19门槛、三标签数据和全部532正式值。T14 check尚未在此候选实现，不将新增契约文档误算为产品能力。

## 性能冻结窗口独立审计（2026-10-27 19:49）

本角色自编[audit_performance.py](T13-independent-evidence/audit_performance.py)，未导入perf的assess.py，也未启动解释器或制造新样本。首次审计在源码撤回前核实测前、测后、当时工作树、release元数据四者完全一致，结果保全为[performance-audit-initial-current.json](T13-independent-evidence/performance-audit-initial-current.json)。随后审计增加封存ZIP支持，使工作树进入后继T14后仍能按T13原字节复核；[performance-audit.json](T13-independent-evidence/performance-audit.json)绑定19成员候选封存包SHA256 `8f36bf82a0dfb9a10d74c7594ca452294212ac90a352a5570cde09af0d03d165`，逐成员核对16个src、两Cargo及9cda exe。

核查项全部通过：原始final-window.json SHA256 `74cdf5052140f47229c5d811c2a92283ebb42b040f3f47325ed1d5c8b09681ea`；测前协议SHA74934a4e…不变；19原baseline输入/源码/stdin/oracle/固定阈公式不变；三标签每次hash/预热与正式计数、固定标签轮转、默认affinity未改；逐项重算wall min/max/中位数、user/kernel中位数及全部paired比值；532次正式及152预热输出由独立数学/Python JSON oracle检查，exit0、stderr空；最终实际路径7正式探针也核对。只可由harness源码确认每copy固定等待5秒；原始记录不含每个sleep的独立时钟轨迹，不能把代码审阅冒称OS级独立监测。

原19线重算为 **15 PASS、2 ENVIRONMENT_INDETERMINATE、2 FAIL**，R304保持OPEN：

- string_map2k：新17.641ms > 原17.522ms，旧14.228ms在线内，FAIL。
- struct_small4：新22.960ms > 原18.746ms，旧16.221ms在线内，FAIL。
- string_ops2k及string_map8k：同期旧、新均超线，环境不确定而非通过；其中map8k新23.762ms、旧23.032ms均超22.521ms。
- 当前总时未证明T13回调消分配提速：map三档新/T12约1.162、1.065、1.078；small4新/T12约1.304。string_ops8k在线内但比T12慢约13.7%，array1k在线内但比旧慢约33.0%；JSON stringify4k新/T12约1.255。这些回归必须保留，源码省分配不能替代实测通过。
- struct8k相对旧约79.6%降时且原至少30%目标通过，主要收益来自T12索引；新比同期T12略慢约1.6%，不能算T13新增收益。CPU短进程量化及未测RSS限制保留。

**独立结论：功能与有界语义差分PASS，效率门槛未通过，不批准T13正式发布或宣称全面提速。** 主调度据此拒收该性能候选并决定在后继阶段撤回两项生产优化，保留8项新增回归测试、原始失败样本和精确封存包。源码撤回/后继T14不改变T13历史测量，也不属于本报告功能验证对象；后继产物应另建hash与验收记录。

再次复核冻结数据可执行：

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T13-independent-evidence/audit_performance.py
```

该命令只重算保留数据并更新本轮派生审计JSON，不启动解释器；`current_worktree_still_matches_measured_sources`可随后继开发变为false，归档源码校验仍必须true。最初功能目录manifest保留T13冻结source hash，新增审计文件与状态一并更新证据清单。
