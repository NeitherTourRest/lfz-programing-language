# T14 人类指南同步与示例验证

日期：2026-10-27。负责人：docs-writer。状态：2026-10-27完成准备，2026-10-29已对最终候选实跑27/27通过。以下准备记录保留原阶段口径，最终结果见末节。

## 范围

更新人类向五指南：README快速入口/命令表、reference的三来源与精确边界、tutorial先check再run的JSON脚本、testing的非执行哨兵设计、errors的check诊断与usage例外。权威为三spec §17；未修改spec、实现、AI指南/skill、根README、测试集、旧T12报告或历史冻结材料。

说明合法命令检查成功不代表运行成功；正常成功双流空exit0；JSON成功严格三个键 `ok/mode/executed`，后者false；前端错误沿原字段，usage错误即使带--json仍stderr文本/stdout空。check不接CODE外--脚本参数且check --help拒绝。没有把宿主读取源码宣传为零IO、静态语义分析或安全沙箱。CLI命令与运行时check内置区分。

T13生产优化已因实测回归拒收，指南明确R304仍开放、1.4.0为候选；未把T13功能通过或源码消分配写成实测提速。T12旧计数明确标为历史指南快照，原证据保留。

## 准备证据

- 五指南修改通过 `git diff --check -- docs/guide`，exit0（只有既有CRLF提示）。
- `T14-guide-evidence/verify.py`通过Python AST解析，exit0。
- `verify.py --prepare-only`完成夹具构造及指南/源码hash快照，解释器启动0次。只写项目内Temp/T14-docs；原字节证据目录设 `* -text`。

## 准备时约定的最终候选实跑

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T14-guide-evidence/verify.py --binary dist/lfz.exe --expected-sha256 <最终hash> --output Temp/T14-docs/final-runs.json
```

验证器仅覆盖新增/受影响的T14示例及一个未改README hello块，不冒称再次全量验证历史35块。包括三来源、精确JSON、源码选项字面值、usage负例、tutorial示例真实提取后check/run、除零对比、未定义名/类型/arity/缺文件/无限循环非执行，以及存在文件字节不变/不存在目标不创建。实际检查总数及最终hash待候选实跑后另记；独立产品验收由verifier负责。

## 最终候选实跑（2026-10-29）

命令：

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T14-guide-evidence/verify.py --binary target/release/lfz.exe --expected-sha256 83f56a194e216862c45377f4c3971b085744a7ba2363163e1edac6af5e85ac79 --output Temp/T14-docs/final-runs-20261029.json
```

exit0，**27/27**检查通过。产物为 **774656 B**，SHA256 `83f56a194e216862c45377f4c3971b085744a7ba2363163e1edac6af5e85ac79`，路径 `target/release/lfz.exe`；没有覆盖dist或编译。27项是聚焦进程检查，不与原35个历史LFZ块合并计数。

覆盖3来源正常/JSON、README检查后运行、从tutorial真实提取的新LFZ块检查和业务JSON运行、检查除零成功与实际运行除零失败、顶层return的JSON错误、空CODE/源码--json保护/选项前后、6项usage负例、未知名/类型/arity/缺运行时文件/无限循环/输出输入文件IO不执行、先副作用后语法错，以及一个未改README hello块。哨兵文件前后字节一致，新目标未创建；每条命令的stdin、退出码、双流及断言保存在[runs.json](T14-guide-evidence/runs.json)。脚本测前测后核二进制及五指南hash一致。

验证器仅将新一轮结果日期从准备阶段的固定2026-10-27订正为实际执行日期2026-10-29，没有改旧T12脚本或历史证据。Python自带路径定位警告来自本机运行环境，验证脚本exit0；不是LFZ输出。

本报告验证指南示例，不替代verifier产品独立验收，不证明效率门槛通过。1.4.0仍是候选，R304仍OPEN。证据目录manifest保全各文件字节hash与五指南hash；未Git、项目外只读、临时件均位于项目Temp。
