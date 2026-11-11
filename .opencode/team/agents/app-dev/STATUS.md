# app-dev — 工作状态
> 最后更新：2026-11-11 by app-dev

## 当前状态
T18-B HTTPplan作者验收7/7通过，最终18d328d9/1029120B；独立验收由verifier负责。所有旧应用/oracle保持不变，本轮B完成后停止，无应用阻塞。

## 产出
app/jsonbatch_modules/{common,record,batch,main}.lfz、verify_jsonbatch_modules.py、T16-JSONBATCH-MODULES.md、t16-evidence/.gitattributes。
Python AST PASS。菱形共享计数、caller同名全局不串、两次执行cache隔离、坏JSON恢复/行号/错误数据及输入输出保留都有oracle。

## 下一步/阻塞
默认sandbox首个check因模块访问权限IOError失败已保全；同一产物获许可提权原33进程全过，结果/manifest/报告已落盘。没有编译/Git/改产品，所有Temp项目内，D约105MiB只写小文件；无应用阻塞，可交独立verifier。

## 经验
模块路径与IO路径基准不同；attempt记录仍无file；共享state.calls须等于总记录数。应用非流式、非原子，保持字面路径检测限制。

## T17预算应用准备（2026-10-30）
新增verify_jsonbatch_budgets.py/T17-BUDGETS.md及t17-evidence属性；AST/prepare-only通过，解释器0启动。正常结果等价、早期终止KEEP输出、已完成IO不回滚、跨模块attempt不吞系统终止均准备；等待最终hash实跑，旧app与oracle未改，不编译/Git。

## T17最终实跑（2026-10-30）
锁1946ab9/943104B，同字节许可提权40/40 PASS；callback预算28见证，0..28逐次保留，默认sandbox拒绝失败也保存。正常结果字节等价/早期KEEP/已完成SIDE不回滚/check拒预算均过。app/t17-evidence/results/manifest和T17-BUDGETS最终节齐，源码/旧app/旧oracle未改，不编译/Git。R304OPEN，无应用阻塞，交独立verifier。

## T18-A工具编排应用完成（2026-10-30）
新增toolplan.lfz/verify_toolplan.py/T18-A-TOOLPLAN.md/t18a-evidence。锁4093714/984064B、独占fixture51b83f…27bee；默认sandbox实际7/7通过，两份7项报告保留，最终加强deadline必须在exec匿名闭包行7。9任务6成功3局部失败、结果字节确定；禁用/能力不泄漏/global deadline/全体验证无副作用/空计划均过。旧应用未改，无编译Git，R304OPEN。仅本轮交付后停止，不进入HTTP/T19/T20。

## T18-B HTTP应用准备（2026-10-31）
新增httpplan.lfz/verify_httpplan.py/T18-B-HTTPPLAN.md/t18b-evidence属性，AST与prepare-only PASS：解释器0进程/服务器未启动。预设7进程回环oracle，显式六参/重复响应头/404正常/302不跟随/三个局部失败后继续/global deadline穿透/禁用无连接。最终hash待root，未实跑未编译Git，旧应用oracle未改。仅B本轮交付后收口，不自动T19。

## T18-B最终实跑（2026-10-31）
冻结18d328d9/1029120B核hash。默认sandbox 2/3，首批8项WinHTTP12029且server零请求，完整失败保留；同一应用/oracle及产物获正常OS权限7/7 PASS，server_shutdown_complete=true。两批5成功3局部失败报告字节一致，全局deadline真实回调/KEEP/无后续请求、禁用、前置验证、空计划均过。app/t18b-evidence两报告及manifest、T18-B说明齐。应用源码77187a…45fdc未改，无产品/旧app/编译/Git，所有Temp在项目内。下一步交独立验证/指南复用；R304OPEN，仅B收口不T19，无阻塞。

## T19准备（2026-11-03）
jsonlstream.lfz/verify_jsonlstream.py/T19说明已落：forEachLine逐条attempt解析，256条固定批输出，精确物理行号，TAKE早停与字节cap。AST/prepare-only PASS、0解释器，待最终hash。14小样oracle覆盖分行/坏JSON继续/局部读取失败/批次副作用/global零预算。旧app不改，无编译Git；本轮T19后停止，R304OPEN。

## T19恢复检查（2026-11-04）
确认昨日三份新产物已完整落盘，无重做；重新AST/prepare-only PASS（0解释器），preparation.json保三个文件指纹。待最终hash GO实跑14进程，不干扰perf窗；源码8436f850…71a0b，R304OPEN。

## T19最终功能验收（2026-11-04）
最终5d2219f8/1041920B，默认权限14/14 PASS。固定struct currentLine.text+单个parseCurrent闭包，256条有界批；首oracle CLI--json错误/逐行closure14绿/误let重绑定失败及全部源码、最终14绿保留app/t19-evidence，manifest齐。全局0步初始化前KEEP；不冒称callback中途资源/关闭/规模性能已由应用证明。所有测试进程结束，无perf并发/产品/旧app/编译/Git。交独立verifier，R304OPEN，仅T19后停止，无阻塞。

## T20 REPL准备（2026-11-04）
新增repl_jsondesk/session.repl和cases.json、verify_repl_jsondesk.py/T20说明，AST/prepare-only PASS，6组/0解释器。真实repl持久JSON检查台、多行/错误后容器与声明保留/input消费/history source-caret/reset/每cellsteps与timeout，等最终hash实跑。旧app不改，无产品/编译Git/外装公网。仅T20交付，不automation，R304OPEN。

## T20最终验收（2026-11-11）
a545d0c2/1100800B 默认权限实际lfz repl重放6/6 PASS，全部子进程结束。多行markThenFail核失败后desk.failed=2，input消费、历史真实来源及caret、reset、每cell预算和EOF均过。首分号、EOF空白行caret预期和EOF中文措辞失败保留于app/t20-evidence及manifest。产品/旧应用未改，未编译/Git。交root收口，R304OPEN，无应用阻塞。
