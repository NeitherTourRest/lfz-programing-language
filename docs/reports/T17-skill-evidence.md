# T17 AI执行限额指南与作者实跑

日期2026-10-30。责任人ai-dx-engineer。四skill与AI入口按三spec§22及T17-design同步新增§6.5，当前73内置、13错误类（新增ExecutionLimitError），旧12错误行为保留。更新范围限所属文档与本作者证据，没有改源、规范、人类指南或旧v2。

## 实际结果

最终target/release/lfz.exe943104B，SHA256 `1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45`。作者**38/38**公开进程见证通过，exit0，五文档测前后hash一致，每次启动前核exe hash。

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T17-skill-evidence/run_examples.py --binary target/release/lfz.exe --output docs/reports/T17-skill-evidence/results.json
```

默认沙箱首次37/38，唯一模块fault用例在求值前canonicalize权限失败为IOError；default-results.json完整保留。同冻结产物提权完整38/38，未更换产物、改路径策略或删去失败样本。所有fixture仅项目Temp/T17-ai，证据JSON保存args/stdout/stderr/exit，无编译/Git或外部写入。

覆盖两个指南实际提取的LFZ块check及限额run（4）；2/4节点精确预算与前导0（3）；0/1/不足预算、含节点无限循环、attempt不吞终止及失败不补步（6）；timeout0、空程序双0、前端先于预算、stdin边界（4）；CODE等号保护/双负号旧NameError、argv flag及空串保真（3）；两flag等号含空/0/bad及--help/--version等号拒绝、负数/重复usage（14）；check/test拒限额（2）；unlimited最小兼容（1）；模块callee真实fault file（1），共38。空for不虚构每轮poll，不以无节点循环作为小预算保证；没有危险OOM/abort/真实深递归压测。

文档明确先check无执行再带两token限额run，不支持等号族，严格u64、CODE/argv保护。取消为Rust宿主token不是CLI Ctrl-C；协作deadline不包含前端且不能在native内部打断，末次native后无新节点可能仍成功。attempt不恢复预算/token/deadline，模块session共享预算，错误来源真实且Result仍无file。

Rust取消并发、poll优先级及宿主API说明为契约引用，作者没有编译新Rust见证，实际对应实现/独立验证由owner负责；本38例不冒称替代全部624测试或性能测量。未增加裸help/version别名。旧v2成绩与manifest不改，不称新Agent盲测。R304OPEN，本轮无性能验收或正式发布声明。
