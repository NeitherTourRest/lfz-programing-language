# T16-C 模块执行最终独立验收
日期：2026-10-30。责任人：verifier。结论：§21模块运行、跨文件诊断及legacy兼容PASS；本报告不包含§22/T17预算实现验收。R304OPEN，无性能计时，不批准正式发布。

## 冻结与实际执行
候选target/release/lfz.exe 902656B，SHA256 `cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006`。无cargo/rustc编译，直接执行作者manifest列出的同九个final release harness；安全提权用于既知canonicalize限制，未修改身份规则或掩盖默认沙箱失败。

| 独立实际执行 | 结果 | 原始证据（T16-C-independent-evidence/） |
| -- | -- | -- |
| 同9个release harness | 599/599 | harnesses.json |
| 模块公开过程 | 29/29 | public29.json / public.py |
| 四文件JSONL应用 | 33/33 | app33.json / app.py |
| 自编真实模块见证 | 15/15 | own.json / own.py |
| 旧attempt/check/CLI | 13/13、117/117、91/91（内含官方178/178） | attempt13/check117/cli91.json |
| 旧jsonflow/sortviz/jsonbatch | 37/37、11/11、38/38 | 同名JSON |
| 旧JSON/argv独立见证 | 140/140 | independent140.json |

旧脚本date/schema历史常量不是本轮时间，实际日期及hash以本报告/manifest为准。源码/Cargo初始指纹initial-sources.json在自编见证末尾重新核对完全一致；后继T17改源不属于本冻结窗口。

## 新的独立行为见证
自编15进程使用真实文件与独占Temp目录，非仅合成来源：菱形导入共享函数私有计数和可变容器；调用者同名全局不影响定义模块函数及逃逸闭包；namespace标量字段改写不回写模块私有绑定；模板默认值在定义模块依次求值，显式覆盖在caller求值，两个实例可变默认不共享；局部模板捕获cell后续改值仍可见。

attempt捕获模块函数错误后caller全局恢复，后继未捕获错误file属于entry且trace不残留已捕bad函数；默认表达式错误file与最后帧为lib/P::<defaults>，显式覆盖表达式错误file为entry。-e/stdin都按cwd解析模块且argv含Unicode/空串原样共享。相同脚本独立两次执行计数均从1开始，不跨session缓存；依赖初始化失败定位lib且没有打印entry-ran。

模块函数递归45层的JSON保留47帧，human折叠保留40个File记录，二者最深file均为真实lib。公开29矩阵另外检查方法self、默认值self与原有check无初始化等契约；应用33独立oracle确认四模块混合坏行继续处理。

## 宿主失败与legacy guard审计
只读核查CaseEval::host_failure递归穿透Located，runner在评判前遇host failure写stderr并exit2；judge无论expected为空或负例均Verdict::Error，不因error_class=None伪判成功。实际599 harness内运行test_runner::t16c_host_tests::host_failure_is_not_a_passing_case_even_without_lfz_error_class，包含直接HostFailure及Located wrapper合成断言通过。此为已执行内部合成边界，不虚称CLI能输入任意坏宿主图。

旧eval入口模块Program guard仍由runtime单元实际执行验证；B阶段run unsupported断言仅按主调度批准迁移为C初始化前失败/执行语义，不删除原无副作用与旧错误检查。资源/default递归边界依赖安全合成state测试而非真实stack abort/OOM。本验收不修改规范、源码或旧正式测试。

## 验证器作者修正及限制
初次自编模板例将空数组写索引0，正确触发IndexError；own-initial.py保留，改为单元素默认数组检验实例隔离。第二次路径比较未去Windows canonical扩展前缀，正确lib路径被作者oracle误拒；own-path-initial.py保留，仅修比较规范化，未修改产品返回路径。最终15/15通过。这两项是验证器错误，不作为产品缺陷，也不隐藏初版。

所有临时输出项目Temp。无编译、性能测量、RSS结论、真实资源耗尽或Git写入。599功能通过与模块能力不意味着效率门槛关闭。全部进程已结束，后继可开始T17，但须另绑定新源码及二进制验收。
