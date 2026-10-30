# T17 人类指南最终聚焦实跑

日期2026-10-30，docs-writer。最终target/release/lfz.exe **943104B**，SHA256 **1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45**。五指南T17入口与execution-limits.md已完成，同字节授权提权实跑 **32/32通过**，exit0。未使用620/F487修前成绩。

## 复现及权限记录

`D:/app/ANACONDA/python.exe -B docs/reports/T17-guide-evidence/verify.py --binary target/release/lfz.exe --expected-sha256 1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45 --output Temp/T17-docs/新的结果.json`

默认沙箱共执行32条，前31条通过，末条模块共享预算见证因本环境canonicalize权限返回IOError/exit2；原始default-partial.json/default-stderr.txt保留。没有改解释器、改预期成IOError或关闭模块验证。按主调度授权，同一产物提权完成32/32，原始runs.json与manifest保留。提权仅补运行条件，不作为性能结果。

## 覆盖范围

- 单表达式N0/1终止、N2成功；声明再读N3终止/N4成功；空程序max-steps0和timeout0成功；有节点timeout0终止，deadline先于steps。
- 默认unlimited、50步空无限循环终止、attempt不能吞系统限额；先check精确executed:false再按步数/时间双限额运行。
- argv同名限额/等号文本原样传递、check -e CODE形如限额名仍受保护。
- 两flag各自缺值、非法、重复、非执行命令拒绝；等号后空/数字/任意文本在--help前缀情况下都精确拒绝，使用既有--help拼写，不新增裸help/version别名。usage字段与原stderr风格精确检查。
- 模块函数无限循环耗尽共享预算，error与末帧file均是真实worker.lfz；T15 Result协议未新增键。

每进程前及结束核产物SHA，六指南测前测后hash一致。脚本按行保存实际命令、输入hex、exit和stdout/stderr，失败也持久化partial。未穷举全部CLI组合，不替代公开121矩阵或独立验收，不重跑无改历史全部指南。

## 文档承诺边界

严格两token、默认unlimited、参数区CODE/argv保护、0与空程序的区别、模块会话共享及attempt余量不可恢复均明确。取消仅host token（Release/Acquire），CLI无Ctrl-C结构化Cancelled承诺；deadline只在stmt/expr边界协作观测，native/阻塞IO不保证硬中断，末次native后无节点可成功。不是沙箱、CPU配额或硬实时响应，前端时间不计CLI超时窗口。

Rust EvalLimits与三个owned参数入口、旧入口Default兼容、13类穷尽match迁移为spec§22引用说明；本角色未编译Rust或实测host取消，不把该说明当作并发token验收。当前32见证也不声称毫秒级实时性能。

仅写人类指南、专属报告/验证器与自身记忆；未改产品、spec、AI、dist，未编译或Git。所有临时项目Temp/T17-docs；R304OPEN，unlimited分支效率仍须对最终字节单独测量。
