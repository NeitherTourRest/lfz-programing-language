# T16-C AI指南同步与作者示例实跑

日期2026-10-30，责任人ai-dx-engineer。事实源三spec§19–21、T16-C-design及人类modules.md。四skill文件与AI指南入口已同步新增§6.4：静态import/export、先check图再run、多文件相对路径、home globals、namespace快照/共享、session缓存、argv、跨源CLI诊断、attempt无file、模板默认上下文与RustAPI迁移。

## 实际证据

最终target/release/lfz.exe902656B，SHA256 `cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006`。作者公开进程检查**18/18**，exit0。五个文档测前后hash一致，每启动前核binary hash。命令：

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T16-C-skill-evidence/run_examples.py --binary target/release/lfz.exe --output docs/reports/T16-C-skill-evidence/results.json
```

该结果在同产物提权复核获得；默认沙箱结果default-results.json为1/18，模块canonicalize导致IOError的17项权限失败完整保留。未改变产品路径/源码以绕过权限，也未排除失败样本。提权运行只读同exe并写项目Temp/T16-C-ai及证据目录，不编译不改dist。

18项包括：两个AI指南实际提取的双文件示例分别check及两次run（6），namespace多alias/标量快照/home函数与容器共享/argv（1），模板定义默认值与caller显式覆盖（1），菱形check无初始化、单次初始化及新session重新初始化（3），真实定义文件JSON诊断（1），attempt无file及恢复上下文（1），-e/stdin相对cwd根（2），循环图拒绝（1），未导出字段运行时失败（1），legacy最小兼容（1）。脚本与原始args/stdout/stderr/exit均保全。

RustAPI迁移内容为准确引用契约的指南说明，作者未编译Rust见证；宿主InvalidGraph/Origins/HostFailure及资源边界由对应实现/独立验收负责，不以本18例冒称覆盖。未新增内置，当前73；没有新Agent盲测，旧v2输入/成绩未改。R304OPEN，未测C性能，不宣称正式发布或吞吐改善。项目外资源只读，所有夹具位于项目Temp。
