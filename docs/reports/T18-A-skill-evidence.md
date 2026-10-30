# T18-A AI工具执行指南与作者实跑

日期2026-10-30。责任人ai-dx-engineer。四skill及AI入口按三spec§23/T18-A-design更新§6.6，74内置/13错误类。Agent流程为先check不执行、按任务授权显式--allow-process并带限额run，再核业务返回与非零exitCode。绝对.exe/noPATH/不隐式shell、会话权限、双流合并rawbyte、严格UTF8、局部与全局终止区别、cleanup宿主异常JSON例外、Windows已验其它Unsupported均明确。Rustcontext/API说明引用契约，不冒称本作者编译验证。

## 实际结果

最终target/release/lfz.exe984064B，SHA256 `4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86`。正常OS权限下作者**26/26**，exit0，单次运行未改预期/重跑筛选。fixture由Temp/T18-A-tooling/fixture.exe独占复制至Temp/T18-A-ai/fixture.exe，SHA256 `51b83f054f128e381fa034adbe980d6398f24f76621703ba27da4d7e54d27bee`，每个进程前核两个hash，五文档测前后hash一致。没有编译产品或fixture。

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T18-A-skill-evidence/run_examples.py --binary target/release/lfz.exe --output docs/reports/T18-A-skill-evidence/results.json
```

该命令本次以许可正常OS权限执行，避免已知sandbox句柄/canonicalize干扰；本作者没有新建默认沙箱失败记录，也未将其它角色沙箱失败说成自己的结果。原始每例参数/stdout/stderr/exit、fixture和文档hash保存在results.json。项目外只读，所有fixture/输出位于项目Temp与本证据目录，外层进程15秒watchdog，局部sleep200ms受Job控制，不访问网络/外部安装或shell拼接。

26项：两指南实际提取脚本check+run（4）；默认禁用、空/空格/Unicode argv、后继上下文禁用（3）；非零退出成功、stdin EOF、BOM/CRLF保持（3）；双流合并N等于及N+1、坏UTF8、局部timeout（4）；全局deadline不被attempt捕获（1）；子进程输出不转发、父JSON唯一状态（1）；相对/.cmd/空argv参数校验先于权限（3）；能力等号/重复、混合首类别、扫描错误优先、CODE/argv保护（7）。

本作者未重新执行Job故障注入、后代回收及cleanup诊断注入、平台Unsupported或Rust并发取消测试；这些指南内容准确引用契约，并由对应owner/独立验收负责，不能以本26例冒称全部覆盖。不宣传沙箱、硬实时或绝对可回收保证。没有新Agent盲测，不覆盖旧v2。R304OPEN，无本轮性能验收或正式发布；按用户新边界仅完成T18-A后停止，不推进HTTP。
