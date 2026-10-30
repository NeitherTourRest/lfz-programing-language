# T18-A 最终产物独立验收

当前结论：主验收通过，公开45脚本因作者NUL oracle无法启动尚待修正完整复跑，未最终放行。最终candidate SHA256 4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86，984064B。递归src/Cargo初末匹配tooling T18-A-close manifest，不复用修前646成绩。

同十一release harness串行--test-threads=1，649/649全部通过；旧attempt13/check117/CLI91（含官方178）/jsonflow37/sortviz11/jsonbatch38/JSON140全过。另T17公开134/预算app40、C公开29/四模块app33独立完整全过。run.py与所有原始逐测试stdout在独立证据目录。无产品构建；仅按root明确授权，现有lib harness内部rustc构建Rust fixture；RUSTUP_HOME/CARGO_HOME/TEMP/TMP全项目目录，未与其它编译并发。外fixture51b83f不变；内部fixture本轮重建为056679f3036fe32347f6dc41cac196211767e1c35f0cfe45e623b5f3c485b199，详fixture-hashes.json。

默认沙箱同lib在close_diagnostics_reverse_order_and_no_double_close测试，WaitForSingleObject已关闭句柄时exit0xC0000008；原始stdout/stderr十六进制完整保留default-permission.json。默认canonical权限失败亦保留。再以正常OS许可同字节harness649全部通过，未修改测试、句柄校验或产品绕过。此默认环境限制不能称已修复。

Close最终静态重审：每次run创建Handles注册表，创建成功的Job、两pipe两端、NUL、process/thread立即登记；execute任一Result早退之后，外run显式close_all逆序消费slot，并合并关闭诊断；已显式关闭slot置null避免二次close，Drop仅unexpected unwind兜底。关闭注入仍真实CloseHandle但模拟报告失败，不制造泄漏。主资源reason保留，不以cleanup覆盖Cancelled/Deadline/Timeout/OutputLimit；正常成功有diag变Io。正常/启动失败/assign+close/pipe+close及真实活动取消+close三个新增unit已在同649实跑。CLI/runner成功diag拒绝与系统JSON原reason门禁亦实际同harness执行。不能以故障注入代替真实OS无法回收时的绝对保障。

自编own.py 12/12真实进程见证：双流合并0/13/14、200KB并行输出与尾部N+1、非法UTF8回收、无符号4294967295退出、局部timeout可attempt而全局deadline穿透、空串/Unicode/空格/反斜杠/引号argv、后继会话能力禁用、parent退出后后代PID通过OpenProcess+Wait确认已回收。全部有限输出和时间，没有watchdog触发、没有执行任意系统进程清理。

原有host单测实际运行覆盖活动token取消（fixture先marker后host置位）、Job/Assign失败用户代码无副作用、pipe/read/wait/kill/close注入及清理diag。此为同harness实测与独立静态审阅，不冒称新外部RustAPI编译见证。

公开45初次独立副本public45.py在NUL用例将实际NUL嵌入-e的进程argv，Python Popen先报ValueError embedded null character，LFZ该例未启动。extra-commands.json保留完整失败，已即报root/test，等待仅新oracle修正；不改旧测试或产品。

Linux/macOS未实测、不支持；Windows显式受控Job不是权限隔离沙箱。无网络/外装、真实OOM/abort、Git或性能测量。R304OPEN，本轮后停止，不进入HTTP。最终独立证据位于T18-A-independent-evidence。

## 2026-10-30 19:20 最后复验与最终PASS

作者仅修新公开脚本NUL oracle：真实NUL源码写入项目LFZ文件，经FILE入口传递；不再把NUL置于Windows argv。旧初始失败脚本public45.py与extra-commands.json保留。独立复制最新公开脚本为public45-final.py，仅改ROOT和专属Temp目录，使用固定外fixture51b83f，完整实际**45/45 PASS**，public45-final.json。无watchdog异常或进程遗留读数，无产品改动。

独立复验新应用app/toolplan.lfz与verify_toolplan.py，副本toolplan7.py仅改ROOT/独立Temp目录，专属fixture副本校验51b83f，**7/7 PASS**，toolplan7.json。验证默认禁止无副作用、两次九任务编排结果逐字节稳定（六成功/三可恢复错误）、权限不泄漏、真实全局deadline穿透匿名闭包且报告exec真实行号、后继任务不执行/KEEP保留、全计划预校验无前置副作用、空计划成功。应用源码SHA b06a9bc3c00dc0822b7a88f5477a9365144a7883b2f783c918f1944cb13a0d65。

结合前轮同649 harness、完整旧回归、T17公开134/app40、C29/app33和自编12，最终独立功能验收**PASS**。本次未重复已冻结源的649/旧全回归，最后递归src/Cargo、产品4093714/984064B和外fixture再次核一致。所有旧失败与环境限制原样保全；不将作者成绩冒称新独立成绩。

本次不编译产品或fixture，不Git，不改生产/规范/旧测试；全部独立测试已结束。R304仍OPEN，不称性能通过或正式发布。Linux/macOS未实测不支持。本批T18-A后停止，不进入HTTP/后续迭代。

## 2026-10-30 19:24 人/AI指南最终只读审计

此次仅读取作者证据，启动LFZ/fixture零次，不是新的独立重跑。人类指南作者22/22、AI指南作者26/26的逐行passed全部核对；两个manifest中所有证据文件原字节SHA均匹配，六个人类指南与五个AI文档当前SHA共11份全部一致。最终产品4093714/984064B、外fixture及作者两份独占fixture均核51b83f。详细哈希与逐例名称记录guide-audit.json。

人类指南首次keys插入序oracle错误与stderr证据仍在manifest中，现按既有keys排序只修验证器后22全过；AI26为作者正常OS许可首次整套结果，作者明确未制造或借用其它角色默认环境失败。两份报告准确限制为CLI/指南例证，不冒称自己执行Job/cleanup注入、后代回收、Rust token/API或非Windows支持；无新Agent盲测、性能或正式发布声明。能力非沙箱、协作截止非硬实时、原生输出与诊断协议边界说明一致。

只读审计通过，独立功能PASS结论不变；R304继续OPEN。审计后结束全部工作，等待root交付推送与停止自动化；不发起后续迭代。
