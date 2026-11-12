# T21 格式化公开测试

2026-11-13，按冻结 §27 准备了 27 个公开用例。`tests/t21_cli_fmt.py` 的 AST 检查通过，`--prepare-only` 退出码为 0，生成计划记录的 `executed` 为 false。

14 个正例逐字节比较输出，并验证输入文件内容和修改时间、二次格式化输出。覆盖 UTF-8 BOM、`#42`、混合 LF/CRLF/裸 CR、末行无换行以及纯空白行。专门检查裸 CR 后接仅含 SP/TAB 的 LF 行，预期保留空白以免把两个物理行合并。其余正例覆盖注释尾白、块注释、插值内部空白、格式说明、转义、中文、管道、双分号和缺失依赖的模块源码。

行为对照运行原文件和格式化后文件，比较退出码、双通道及文件副作用。`attempt` 用例输出并比较完整错误对象，另外验证错误行、列及 traceback 有实际值。9 个负例覆盖坏 UTF-8、前导、词法、语法、缺文件及不合法参数形态，均要求退出码 2、诊断写 stderr、stdout 为空。4 个兼容例覆盖裸 `fmt`、`--json fmt`、`run fmt` 和脚本参数中的 `fmt`，可传入固定 SHA 的 T20 产物做双通道逐字节对比。

冻结 T20 `a545d0c2…` 的真实红测为 0/1，脚本退出码 1。13 个无依赖的合法输入用同一产物 `check` 校验语法，退出码均为 0。准备计划、旧版红测和旧解析结果保存在 `T21-blackbox-evidence`。旧版测试仅验证缺命令与夹具语法，T21 最终功能结果待冻结产物实跑。

最终验收命令使用新证据路径，并传入固定 T20 产物做兼容对照。

```text
D:/app/ANACONDA/python.exe -B tests/t21_cli_fmt.py --binary target/release/lfz.exe --expected-sha FINAL_SHA --baseline dist/lfz.exe --baseline-sha a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c --output docs/reports/T21-blackbox-evidence/final-first.json
```

临时夹具位于项目 `Temp/T21-public`。R304 OPEN。

## 最终公开验收（2026-11-13）

team-lead 在冻结 Windows 候选上运行了原公开脚本，**27/27 通过，退出码 0**。候选大小 1117184 字节，SHA256 `76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e`。T20 对照产物 SHA256 为 `a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c`，四个旧命令用例的输出及退出码逐字节一致。公开结果包含字节精确、幂等、输入文件内容及修改时间、缺失依赖和可观察 `attempt` 位置的通过记录。

本角色核对了候选、脚本、T20 对照和构建清单中的 41 个来源指纹，并逐字节复制了原始结果 `Temp/T21-fmt/public-final.json`。证据 `T21-blackbox-evidence/public-final.json` 的 SHA256 为 `31542d67d30d523fb20f33e0e3cf1883fae60e2a0fc804a71ee736d1039f1b62`，与原件一致。`manifest-final.json` 记录完整校验，原 T20 红测和夹具解析结果仍可查阅。R304 OPEN。
