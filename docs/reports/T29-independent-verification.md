# T29 独立功能验收

候选 `Temp/T29/candidate.exe`，SHA256 `f0d483f7a0cbf2d9c45da5bd684327f8ac177ac4c2378e23230eba86d29a4ed9`，通过10组独立功能差分。参照为 `Temp/T28/candidate.exe`，SHA256 `326b8ec9ede80883c7ac23217068be278ed607e1b84930df4d7f6af6ef9e73ea`。每例在相同夹具路径运行两份冻结产物，退出码与双通道输出逐字节一致，所有显式断言成立。

覆盖各次回调参数被返回闭包捕获后独立取值、参数与可变局部的跨调用隔离、捕获后当前调用继续修改参数、嵌套map生成两层逃逸闭包、错误发生后已经保存的回调仍可执行且新高阶调用可恢复、短路及共享容器修改、sortBy键回调返回后的闭包、不可捕获执行预算、运行错误完整JSON/栈，以及模块私有全局的定义home查找。模块返回闭包在私有全局修改后仍读取同一home，入口同名全局不会代替私有名。

首轮前4组通过后，自编夹具使用保留字 `fn` 作为对象字段，两份产物都报同一语法错误。保留 `verify-first.py`、`error-first.lfz` 和 `first.json`，修正夹具字段名为 `callback`。第二轮9组通过后，两份产物在默认沙箱读取模块时均报IOError，原始 `second.json` 保留。正常Windows许可运行相同最终脚本，10组全部通过，含模块例。生产源码及旧测试均未修改。

从项目根目录运行以下命令。验证器会自行创建 `Temp/T29-verifier/fixtures/lib` 及夹具。输出路径须尚未存在，已有结果会拒绝覆盖。

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T29-independent-evidence/verify.py --baseline Temp/T28/candidate.exe --baseline-sha 326b8ec9ede80883c7ac23217068be278ed607e1b84930df4d7f6af6ef9e73ea --candidate Temp/T29/candidate.exe --candidate-sha f0d483f7a0cbf2d9c45da5bd684327f8ac177ac4c2378e23230eba86d29a4ed9 --output Temp/T29-verifier/reproduce.json
```

正式窗口和独立性能审计已完成，同一候选通过原R304测量条件。归档正式窗口SHA256 `66aa0e666abcda306369c710bbd32b246b0645a35c2603db6f40452e6c3b916b`。19个旧负载全部在原固定线内，无环境不确定或失败，`struct_ops 8000` 中位15.2285 ms，对原89.4598 ms减少82.9773%，低于原30%降时目标上限62.62186 ms。

独立读取归档JSON及ZIP重建原T13协议、两份原T12基线与各阈值。76组684条记录包括152次预热、532次正式样本，标签轮换、组内序号/阶段/身份/输出/退出码全部匹配。独立重算正式中位、最小最大值、CPU中位及同期倍率。九项JSON/CLI能力输出正确。实际临时候选路径探针 numeric_loop500000 为94.9237 ms，保留在19项之外。候选ZIP中的exe及27项源码/Cargo与窗口前后回执逐一匹配。T28参照和T29候选的ZIP也各自核原始成员、长度与SHA，基准输入ZIP绑定原协议、脚本和原基线资料。

JSON净时按原冻结规则计算，noop为8.96 ms。parse2k/4k/8k总中位19.3508/31.6307/55.4819 ms，近似净时10.3908/22.6707/46.5219 ms，三档启动占比均低于50%，相邻翻倍净时比2.18181、2.05207均低于3.0。stringify总中位13.8780/15.7773/20.0507 ms。2k净时4.918 ms且启动占比64.56%，4k启动占比56.79%，仅8k符合可测资格，故没有相邻可比较净时比。此处按规则保留总时，不解释为函数内部纯吞吐。

独立屏选复核84条样本（12预热、72正式），六例轮换和二候选交替顺序、命令、夹具、原始输入输出及观察时间顺序均核对。array_full中位T28 92.8483 ms、候选49.5180 ms，string_full中位T28 37.43405 ms、候选28.9066 ms，两项均6/6配对轮更快。control与push_self_8000两项回归否决条件均未触发。屏选判定与预先归档条件一致，通过是进入正式窗口的依据。

归档离线入口只读仓库材料，默认输出stdout，可在全新检出上使用任何可用Python 3执行，无需项目Temp基线、原机器Anaconda路径或原时钟。命令如下。需要文件输出时可加 `--output Temp/T29-verifier/offline-reproduce.json`，只允许项目Temp且拒覆盖已有文件。原始功能差分重放仍需上述两个可执行产物，其历史已执行结果已归档。

```powershell
python -B docs/reports/T29-independent-evidence/offline-audit.py
```

离线核验初稿手录正式窗口SHA时多写一个字符，已保留 `offline-first.py`。按不可变原始文件与回执核对正确SHA后，只更正独立读取器，再运行全量检查通过。归档结果 `audit.json`。审核阶段未启动任何LFZ计时或重跑窗口。

结论限定为本机同窗、冻结候选、原协议的整体进程耗时，含启动/解析/I/O/执行。未测峰值RSS，未独立在测量现场观察系统调度。历史T27/T28失败各保留对应产物身份，不能移用本次通过值。
