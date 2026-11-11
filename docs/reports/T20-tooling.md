# T20 REPL frontend 作者报告

最终候选target/release/lfz.exe1100800B，SHA256 `a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c`。唯一release no-run/build均0，13harness705/705=528lib+58bin+28cli+12T12+12T14+7B+5C+9T17+6T18A+3T18B+7T20+15runner+15unit，0失败/忽略。每程序90秒外watchdog，正常OS许可同harness，旧A已知默认沙箱Close abort不重复诱发。公开/独立/新版Agent包尚由root协调，不把作者回归当正式发布或R304通过。

## 前端实现

新增显式repl命令，旧无参数/run/check/test/JSON协议保持。按既有scanner支持session权限/argv和每cell限额，repl --json明文拒绝。二进制内部execute的两个writer经root批准收窄为Write+Send，以便安全在现on_eval_stack内捕获；main仍plain Stdout/Stderr，不为Send改变旧锁定路径，不忽略测试writer或unsafe迁移Rc。整个read/probe/session/eval/render/drop在同一大栈。

stdin每次借StdinLock，以fill_buf/consume有界复制一物理行，返回即释放锁再eval；不另BufReader、不先read_to_end。追加前检查pending/raw1MiB，观察超cap立即exit2不drain余行；共享std缓冲未消费bytes仍供input。输入UTF8/IO失败host exit2。typed core probe Incomplete续行，Complete/Err正式交runtime eval_cell，不拼历史源码。空白/注释不占历史；EOF未完成只正式失败一次。

最后值非nil按Display输出，assignment沿原nil不回显。空pending识别help/reset/quit，未知命令计错；reset不清累计exit。主/续提示stderr并flush，pipe不输出提示。session poison后普通输入不能继续probe/eval，仅允许元命令恢复；pipe污染直接受控结束。双diag门禁保持旧文案。runtime API负责session/source/home/预算与状态恢复，前端不触私有Interp。

新增REPL专用compile-error adapter：运行frames空且有真实error_source的编译失败，取该source路径/Loaded，沿现非运行渲染(tracedNone)输出caret；不伪造runtime帧、不改旧graph/run渲染或API outcome。跨旧cell函数错误仍用原逐帧来源。

## 原始失败与改正依据

1. 旧dist5d2219执行repl的真实usage红保red.json。
2. 首定向writer2/2、stdin5/2：本人新oracle误将assignment nil算回显（应两次get的2行），仅改新增测试；另真实REPL接入编译失败缺cell路径/caret，已仅修adapter。首日志保first-*，第二次2/2+7/7通过。旧测试不改。
3. 首全lib527/1：runtime新增测试把普通语句后import期待为REPL禁止消息，而旧parser先报ImportOrder。交runtime原owner，保旧非法顺序预期并增加合法import/export拒绝且前缀无效果见证；只owner改其新oracle。first-full-runtime-failed原件保留；最终528lib均过。
4. 原Temp测试生成器留作历史，含首oracle，不应重新执行覆盖修后tests。

## 真实Windows ConPTY证据

使用Python ctypes本机系统API，不安装依赖。每case创建私有ConPTY、双pipe及自己的cmd/LFZ子进程，输出读线程持续drain；15秒watchdog只终止自身CreateProcess句柄并终止probe，正常结束closePTY/join reader/关闭全部自身句柄。没有产品native代码或全局控制台配置改动。创建/关闭与持续drain依据[Microsoft ConPTY生命周期说明](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session)。

初直接CreateProcess继承Python宿主stdio，LFZ与最小系统cmd echo控制对照均raw0/exit0，不是产品活锁。仅驱动显式CONIN$/CONOUT$绑私有console后，系统CONTROL对照PASS，LFZ出现prompt。ConPTY将末空格编码ESC[1C，初驱动plain-strip误删导致expect失败；又有同chunk结果/prompt匹配游标取整buffer长度漏标记的问题。逐层明确对照后修驱动，不改产品，不将这些失败写成REPL缺陷，全部原件保留。

最终conpty-final-cursor-fixed-result.json PASS/exit0，逐步等主prompt才发fn，等续prompt才发return与闭括号，f()返回17，等data>才发hello，观察下一主prompt，再quit。证明实际终端提示/flush和input交替，不是mock isTerminal。conpty-errors-ascii-result.json PASS/exit2，未知命令后观察错误前缀和下一prompt、reset后再quit仍exit2。两case均绑定相同最终产品hash，watchdog未触发，reader线程完成join。终端合流stdout/stderr，分流正确性由pipe与同writers单测另证。

中文边界：默认console代码页的完整中文expect出现mojibake失败；另私有console chcp65001变体在VT重绘后的尾空格匹配阶段失败，未到中文断言。本轮不再追测，不称完整UTF8中文TTY通过；固定中文文案在真实pipe字节与writer测试已通过。原codepage/65001失败和ASCII范围PASS并存，未删弱旧oracle。实际可见prompt测试不等于通用VT终端模拟器验证。

## 收口与限制

formatter独立评估见T20-formatter-assessment.md：lexer丢comments/trivia、AST管道脱糖/数值原文缺失，本轮有据暂缓，不加format/check伪功能。永久docs/tooling/evidence/T20/含全部阶段、ConPTY raw/decoded/events/driver、递归src+Rusttests清单和精确ZIP。source/exe冻结后无后续产品写/编译；已将仅现rlib的Temp host sidecar单rustc窗口交独立verifier，不产品重建。

未写dist/Git，原用户三块和旧v2保留；无外装/公网/automation。只有Windows实际验收，无跨平台TTY通过声明；R304OPEN，完整中文TTY边界如上。仅T20最终交付收口。
