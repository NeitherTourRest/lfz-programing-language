# T15 受控错误恢复黑盒准备矩阵

日期：2026-10-29。责任人：test-engineer。状态：准备，仅依据主调度批准方案，等待三spec §18最终冻结；尚未编写正式LFZ用例或宣称实跑通过。

基线T14：542 Rust、官方166、check进程117；R304开放。原LFZ/Rust用例和manifest条目保留，新需求仅新增文件/追加条目。不会将新增语义测试的通过数混入历史计数。

## 待验收矩阵

| 类别 | 正常与边界见证 | 错误/反例见证 |
| -- | -- | -- |
| 结果数据结构 | success值含nil/bool/int/float/string/array/struct；固定ok/value/error三个键；成功原容器别名不复制；JSON可表示结果可序列化 | nil成功必须区别error；结果/trace容器各次调用不共享可变状态；function值成功仍按既有JSON不可编码规则 |
| 可恢复错误 | callback内Name/Type/Index/Field/ZeroDivision/Value/IO/Assertion/整数Overflow，每类至少正常与恢复后成功 | 精确class/message/line/col；IO有无位置按规范，字段值不伪造 |
| 连续与嵌套 | 多次失败后再次成功；外attempt包内attempt失败值；callback调用map/reduce内失败 | 内callback参数错误属于内业务TypeError与外attempt边界校验明确区分；外callback后续错误仍可捕获 |
| 参数与调用形态 | direct/pipeline/用户闭包别名/捕获self/特殊argv零参函数值；空数组map不执行callback | attempt自身arity、非函数、非零参callback在边界外抛TypeError；不把旧builtin误当一等函数值 |
| 副作用保留 | callback写数组/struct字段、外部可变cell后失败，恢复后可见已发生修改 | 重绑定let错误依旧，不承诺事务回滚；局部参数重新绑定不改外部绑定 |
| 文件与输出 | callback writeFile/appendFile/eprint后失败，捕获后哨兵保留既有写入和输出 | 正式文件IO探针全用项目Temp及进程脚本，避免LFZ runner隐含工作目录影响；不以caught错误掩盖文件检查 |
| traceback恢复 | named多层调用捕获trace；嵌套attempt；恢复后另一个未捕获错误正常完整trace | 对照相同源码未捕获路径的class/message/line/col及frames；无前一次失败frame/span残留；无伪造file字段 |
| 非可恢复资源错误 | 既有range容量上限受控失败、现有递归限制路径 | CapacityOverflow/RecursionError须继续传播，外层attempt也不能吞；不用真实OOM/host panic或abort测试 |
| 加载解析优先 | 合法源码中attempt才可运行；check仅解析而不调用 | callback文本含语法错仍是加载/解析失败，Cosmos前导错误不能被attempt捕获 |
| 批处理端到端 | good/bad/good JSON记录处理，输出成功值+失败记录，按输入序继续 | Unicode/空记录/超i64JSON/重复键/非法输入均按原jsonParse ValueError捕获；不改变原JSON规则 |

## 冻结后执行方式

- 正向与捕获业务错误使用新增 `tests/lfz/test_t15_*.lfz`，用LFZ assert/check验证结果，原166项全量回归。
- 边界外和非可恢复错误用新增 `tests/fixtures/t15_*.lfz`，仅向cases.json追加类名预期；精确消息/位置/trace和文件字节由独立公开进程脚本与原始JSON验证。
- 等最终接口冻结后才写语法/字段精确断言；等runtime指定candidate hash后新输出路径实跑，保留所有失败，不为测试修改解释器。
- 所有临时夹具项目Temp；D空间0.31GB，不编译、不新target、不复制大产物。证据按原字节属性保全hash。

本矩阵只表达测试计划。接口名称、字段、错误捕获范围均以最终spec §18为唯一事实源；不替代语言架构师定义或verifier独立验收。

## 冻结后用例准备
三spec §18冻结后新增4组正向LFZ和8个负例，cases原131条值前缀完整保留，只追加8项。新增tests/t15_cli_check.py公开进程见证（精确Result/trace、失败后新trace、IO/输出不回滚、check(false)成功、边界传播）；Python AST检查通过。4正例在旧83f的check语法预检4/4通过，但未宣称attempt行为通过。旧83f运行results文件NameError/exit2，preimplementation-red.json保全，明确为缺新增内置的真实红测。
- 预期官方LFZ总数166+4+8=178，待新candidate实跑才可确认。
- 受控递归错误仅交runtime构造或verifier安全边界见证，不做host abort；本角色不编译。

## T15最终候选实际结果（2026-10-29）

指定 `target/release/lfz.exe` 782848B，SHA256 `1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c`。官方runner **178/178通过、0失败、0错误、exit0**（旧166 +4正向LFZ +8清单负例）；公开进程 **13/13通过、exit0**，两种计数不叠加。

- `candidate-suite.json`、stderr及meta保全官方原始结果和指定字节。
- `final-process.json`核精确Result、错误消息“除以零”及1:15、四层完整trace(func/line/col)；随后未捕获NameError只保新函数frame，无旧失败frame；IO已有/新文件、输出不回滚；check(false)仍成功false且有原警告；外层校验和容量错误传播。
- `candidate-process.json`是第一次13/13成功的较宽位置断言；随后加强新脚本为逐字段精确line/col/完整frame并以新输出再跑13/13。没有解释器修复、隐藏失败或替换原证据。
- 4正例覆盖nil/标量/共享返回容器/函数、argv/管道/遮蔽、9类可恢复错误、嵌套与连续100失败后成功、正常递归、逃逸闭包、HOF及坏JSON混批。8负例覆盖0/2参数、非函数、回调arity、先求参错误、容量及嵌套容量传播、前端语法错。
- RecursionError不可捕获资源边界由runtime单元合成及verifier安全独立见证负责；本角色未制造真实abort/OOM，不将未实跑的该项计入13公开进程。

复现（输出须不存在，cwd仓库根）：

```powershell
target/release/lfz.exe test --json
D:/app/ANACONDA/python.exe -B tests/t15_cli_check.py --binary target/release/lfz.exe --expected-sha256 1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c --output Temp/T15-test/new-process-run.json
```

原131manifest条目值前缀保持，仅追加8；旧LFZ/Rust用例未改。本角色未编译、改产品源码、dist或Git。性能R304仍OPEN，此报告仅功能黑盒，不批准整体正式发布。
