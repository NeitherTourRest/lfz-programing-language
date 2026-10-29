# T14 AI指南与skill示例验收

日期：2026-10-29。责任人：ai-dx-engineer。

四skill文件与AI指南入口已同步三spec §17；标准工作流先check再run。新增内容明确三来源、前导与CODE保护、usage的--json文本例外、executed:false、非执行与非静态分析边界、宿主源码IO仍存在，以及CLI check和LFZ断言check的区别。

作者实跑 **27/27通过**。产物 `target/release/lfz.exe` 为774656B，SHA256 `83f56a194e216862c45377f4c3971b085744a7ba2363163e1edac6af5e85ac79`。结果JSON保留每条命令、stdin与stdout/stderr原字节hex、退出码；文件哨兵包含NUL且检查后字节保留，未创建新目标。包含偶数求和最小基线先check再run输出6，以及原运行JSON和非致命断言的关键兼容。

复现：在仓库根目录执行（输出必须为尚不存在的项目内路径）：

```powershell
D:/app/ANACONDA/python.exe -B docs/reports/T14-skill-evidence/run_examples.py --binary target/release/lfz.exe --output Temp/T14-ai/new-results.json
```

脚本以SHA锁定本轮产物，不编译、不改解释器。证据目录原字节入库，manifest绑定五个文档及结果/脚本。未重跑L1–L5全套或旧v2盲测，本轮不是新的Agent成功率；旧8/8、131/131属于原冻结输入，未改旧manifest。T13失败候选已撤回，R304仍OPEN，当前为1.4.0候选，不宣称整体提速或正式发布。
