# T12 数字循环性能异常诊断与作用域分配优化

> 更新：2026-10-25 22:52，runtime-dev。运行时小优化已交付；正式全负载性能复验由perf-engineer负责，不在本报告提前判合格。

## 当前结论

首版T12的数字循环回归不能仅用路径、名字或P/E调度解释。恢复后固定同一个logical CPU、同subject路径、相同进程优先级，首版新程序的CPU时间仍显著增加。早期同字节换路径偶尔较快的读数已保留，但不足以证明问题解决。

本轮采用可独立说明的通用优化：**while块没有直接Decl/FnDecl/StructDecl时，不为每轮创建没有绑定的空Env**。直接语句检查仅在进入while前进行一次。只要存在任一直接声明，仍每轮创建fresh child；for、函数帧、其它exec_block均不改。language-architect已批准窄边界并补契约§16.7。变量cell、闭包捕获、可见变量顺序及控制流保持原语义。

没有应用失效的cold/inline/Box/Rc索引候选，没有改主Cargo的release参数、测试规模或预设阈值。微观LLVM/codegen根因尚未确认；不声称此次已证明某个编译器bug或缓存bug。

## 恢复后的控制实验

脚本`affinity-control.py`先读取Windows处理器拓扑，选P类logical CPU0和E类CPU16。Popen启动后、发送stdin数字数据前，用SetProcessAffinityMask设置子进程亲和性；记录GetProcessAffinityMask、GetPriorityClass、GetProcessTimes。仅修改子进程，不改系统设置。各版本轮换相同subject路径，预热2轮+7正式轮、正反顺序交错、全部原始输出核对。CPU时间受Windows计时粒度约束，表中是中位数。

原`benchmarks/lfz/numeric_loop.lfz`500k任务，固定CPU0的一组：

| 版本 | wall ms | process CPU ms |
| -- | --: | --: |
| 旧149b | 115.6683 | 93.75 |
| 首版新FB8E | 196.6697 | 187.5 |
| 同源重新构建新版本 | 200.0153 | 171.875 |

CPU16同组旧277.8945/CPU171.875ms，新442.7261/CPU343.75ms。因此异构核调度有影响，但不能解释新旧主要差距。

隔离候选均保留源码与原始结果，且未作为最终实现：

- cold/noinline argv fallback、强制inline热路径薄包装、Box参数上下文：未明显改善。
- 旧evaluator+新rest、旧CLI+新lib：仍慢，argv/CLI改动不是回归必要条件。
- 撤StructObj索引、改Rc<HashMap>索引：仍慢；未据此回退大对象索引收益。
- 旧eval+旧value或旧eval+旧builtins：仍慢；单模块替换未给出唯一根因。
- 全旧源码仅升Cargo版本1.4：与旧版本近似，版本号本身不是必要原因。
- 全旧源码仅新error两变体：与旧版本近似；thin LTO完整新版对照未改善，不建议改发布profile。

真实尺寸控制（旧/only-error一致）：Value16B、Flow24B、R<Value>16B、LzError88B、ValueMsg72B、Interp64B。历史error.rs头注释把R<Value>称为指针大小不准确，不能作为ABI推理依据。lib汇编中旧新lookup/assign_name/exec_stmt_inner/apply_binary的指令行数和栈大小相同；这不等于证明最终全程序LTO机器码完全相同。

## 有效候选与功能验收

仅窄while作用域优化的固定CPU0同subject7轮500k结果：

| 版本 | wall ms | process CPU ms |
| -- | --: | --: |
| 旧149b | 113.1171 | 93.75 |
| 首版新FB8E | 197.7244 | 171.875 |
| scope-elision候选 | 101.8117 | 78.125 |

无绑核同subject的另一组，100k旧47.9147/首版66.0952/候选46.1712ms；500k旧121.7894/首版203.1403/候选108.3014ms。后台负载未严格控制，100k连旧版也高于原绝对阈值；本组只能证明同组相对恢复，不能提前判正式性能全部合格。

主源码应用后：

- 新增5项有意义语义回归：无声明循环的外层cell闭包共享；直接let/fn/struct声明保留fresh绑定；嵌套shadow与for迭代fresh cell；break/continue/return；空作用域的dump可见顺序和值等价。
- `cargo test`：**522 passed / 0 failed / 0 ignored**（404+48+28+12+15+15）。
- 标准`cargo build --release`成功，主profile不变；`lfz test --json`：**166/166 PASS**。
- 实际含`;;`的无声明while程序文本模式stdout和--json模式stderr逐字核对相同两轮变量顺序；JSON stdout仅`{"ok":true}`。
- target/release与dist最终字节一致：**821760B**，SHA256 **7641b1ba98551d8726918b320d18501d62e30faa5c8c5c08fe2dc789a4748d81**。正式全工作负载性能复验待perf报告。

## 原始证据与保全

目录[T12-runtime-evidence/perf-diagnostic/](T12-runtime-evidence/perf-diagnostic/)：

- 首版FB8E exactexe与对应src/Cargo存于`initial-t12-snapshot.zip`（548437B，SHA256 `fc6b5a47ba423b4bbebba45c17ab547ad0c507c34e074aff0b3b60da2340c4b4`），逐项回读核对，附SHA清单。
- `diagnostic-variant-sources.zip`保留所有隔离variant源码/Cargo及清单；诊断组合有故意省略argv/JSON的版本，不能作为发布候选。没有将冗余对照exe逐个入库。
- 所有affinity/unpinned控制JSON与脚本、构建日志、尺寸控制日志、正式522/166日志、dump witness与结果均保留。
- 前一阶段control/copied-controls/path-controls/path-name-controls/material-check仍保留。它们展示路径/名字读数差异，不作为已解决证据。早期HEAD旧源码/同源新源码/独立CARGO_TARGET_DIR构建也保留日志。

磁盘空间告警后，确认没有活跃构建，验证解析路径仍在项目Temp/runtime-t12/perf-lab内，保留所有可执行产物、asm、源码与原始证据，清理已完成variant构建中间件约247916368B。未清根target、未删唯一证据、未写项目外目录。所有后续临时文件仍在项目Temp内。
