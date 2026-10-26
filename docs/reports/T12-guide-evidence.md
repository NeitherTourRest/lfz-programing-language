# T12 人类指南与实跑证据

日期：2026-10-25。责任人：docs-writer。范围为 docs/guide 五个人类指南；只读引用 spec §16，不定义新语法。AI入口与skill由ai-dx-engineer单独负责。

## 完成内容

- README：1.4.0、72内置、--原样传参、-e和stdin、状态JSON与业务JSON区别、jsonflow与sortviz入口。
- tutorial：JSON→管道筛选/投影→确定性JSON的完整示例；argv别名、新数组与遮蔽，只有argv新增一等取值能力。
- reference：三内置签名、精确i64/有限f64/负零、Unicode和控制字符、重复解码键、128层容器、循环/共享、方法过滤及平均索引复杂度。
- errors：两个精确ValueError模板、五种parse与四种stringify reason、UTF-8字节偏移与调用点行列的区别、stdin加载期IOError。
- testing：517 Rust/166黑盒/91进程当前功能基线（91含一次全量黑盒，不累加）、可运行进程验收命令与旧v2归档保护。
- 同步旧--json“任意位置”描述的源码/脚本参数例外；纠正文件IO也是外部状态依赖的旧表述。未编写速度提升承诺，性能结论由最终T12性能报告给出。

## 实跑结果

解释器：dist/lfz.exe 1.4.0；SHA256 `fb8e5653827de1e1c3f73b81d37f21314ad41adfdc4d99f37cb8ca5f6491f47d`。本次开始和结束hash一致，功能验证绑定该输入，不冒用后续测量结论。

- 五指南全部35个LFZ代码块逐字提取，**35/35符合预期**：28成功、7教学负例退出2。原33块与前轮真实stdout核对，新增JSON和argv两块另外核对文档精确输出。
- **10/10新增进程探针通过**：中文/空/字面--json参数、stdin代码、argv别名传参、中文重复键字节偏移9/调用点1:1、NaN/Inf拒绝、非法UTF-8 stdin、表达式不自动打印、状态/业务JSON分流、jsonflow结果文件原始字节、sortviz参数命令。
- testing.md所列实际命令运行通过：**91/91**，包含一次官方166/166。输出原始报告为 [cli-check.json](T12-guide-evidence/cli-check.json)。它是已有验收程序重跑结果，不计为新测试数量。
- README的两条PowerShell命令也按原文直接运行，分别得到 `["中文","--json",""]` 与 `3`，退出0。
- 未出现abort/panic，不执行故意崩溃压测；Python主机存在路径定位提示，但全部验证程序exit0。

## 复现与原始材料

从仓库根目录：

```powershell
$env:PYTHONIOENCODING='utf-8'
D:/app/ANACONDA/python.exe -B docs/reports/T12-guide-evidence/verify.py
```

结束输出 `SUMMARY 35 35 10 10`。脚本只在项目Temp准备LFZ片段和IO父目录，原始命令、源码、退出码及完整stdout/stderr见 [runs.json](T12-guide-evidence/runs.json)；前轮指南stdout只读对照保存在 [prior-guide-runs.json](T12-guide-evidence/prior-guide-runs.json)。[verifier-output.txt](T12-guide-evidence/verifier-output.txt) 是公开脚本实跑输出；[guide-sha256.json](T12-guide-evidence/guide-sha256.json) 记录本次五文件工作区字节快照。LFZ块都含文件#42前导；-e示例仅出现在命令块。

旧v2题集、manifest、九项输入归档与首轮成绩均未修改。当前active skill/binary与旧manifest不匹配属于版本边界；按指南在项目Temp恢复旧冻结输入或新建版本化测评，不覆盖旧成绩。

这是指南功能示例证据，不是独立产品总验收，也不替代尚在处理的性能验收。本人未修改src/spec/app/原tests/skill/rootREADME，未执行Git。
