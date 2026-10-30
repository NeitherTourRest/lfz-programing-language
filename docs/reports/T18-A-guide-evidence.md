# T18-A 人类指南最终聚焦实跑

日期2026-10-30，docs-writer。最终target/release/lfz.exe **984064B / SHA256 4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86**。五人类指南入口及subprocess.md已完成，默认沙箱实际进程验证 **22/22通过**，exit0；未编译/重建产品或fixture。

## 实际命令与原字节

`D:/app/ANACONDA/python.exe -B docs/reports/T18-A-guide-evidence/verify.py --binary target/release/lfz.exe --expected-sha256 4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86 --output Temp/T18-A-docs/新的结果.json`

在任何执行前，把Temp/T18-A-tooling/fixture.exe独占复制到Temp/T18-A-docs/guide-fixture.exe，并核原件/副本SHA256均为51b83f054f128e381fa034adbe980d6398f24f76621703ba27da4d7e54d27bee；不使用内部测试重建路径。每进程前核产品和独占fixture，结束再次核原fixture/副本/产品及六指南hash一致。

初次shell重定向因Temp目录不存在而未启动Python，创建目录后正常执行。首次实际脚本前2例通过，第三例作者误把keys结果当作struct插入序，AssertionError；first-oracle-failure.json/first-oracle-stderr.txt完整保留。keys按既有A5排序，正确预期为exitCode/signal/stderr/stdout，故只修新验证器该oracle，不改产品/规范。修后完整22通过，runs.json/manifest与脚本按原字节入库。

## 22项见证

教程直接提取tool.lfz，先check唯一executed:false对象再授权run；返回stdout精确保留空串、空格、Unicode参数和换行，父进程只输出显式打印的业务JSON。非零7正常返回四键/signal=nil，stdin EOF、BOM与CRLF原样、非法UTF8整体ValueError、合并0静默/N=6允许/N=5超量。

默认禁用的mark命令未创建目标，启用运行后下一次默认session仍禁用；相对cmd.exe拒绝无PATH查找。局部10ms timeout被attempt捕获，全局deadline执行限额穿透attempt；不声称此短进程见证确定了所有同时事件或硬实时响应。

argv后的--allow-process/等号文本原样保留；无值选项等号空/数字/文本均精确usage拒绝，重复/非执行check拒绝；混合能力与限额的首次类别文案正确。usage stdout空/stderr文本/exit2，所有实际命令、输出及exit在runs逐行保存。

## 承诺边界

文档明确exec三参范围、四键、绝对.exe与CRT argv、无隐式shell（显式选shell仍由用户负责）、raw合并输出与严格UTF8、非零exit正常值、local timeout与global终止区别、会话权限与模块共享、回收diagnostics宿主例外；不得仅凭result Ok断言回收成功。回收故障注入、Job/后代生命周期和host取消由owner/独立验收提供证据，本指南不冒称自己重新穷举。

仅Windows后端本批已验，非WindowsUnsupported，不是恶意进程/权限逃逸沙箱。Rust新context三个owned入口为规范引用，本角色未编译宿主API。未重跑无改历史全部指南块，不把649或公开45/app7当本角色计数。R304仍OPEN，不宣称性能或正式发布通过。

仅人指南/专属报告/自身记忆写入，无AI/spec/产品/dist/Git变更；全部临时项目Temp，仅小fixture独占复制。本轮交付后停止，不HTTP或新迭代。
