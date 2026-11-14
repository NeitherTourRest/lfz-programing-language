# tooling-dev — 工作状态
> 2026-11-14 23:41

T22 私有 Windows ConPTY 六组对照完成。可读写控制台在 CP437/936/65001 均正确处理中文输入、回显与诊断。仅写 CONOUT 句柄在 CP437/936 复现乱码，CP65001 正确。当前交付版 76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e 用于全部验证。与独立原生屏幕缓冲区验证一致。本轮为验证交付，产品源与构建保持冻结。

报告 docs/reports/T22-tooling.md，六组原始记录及 SHA 索引 docs/tooling/evidence/T22/manifest.json。可重放脚本输出至 Temp。R304 OPEN。
