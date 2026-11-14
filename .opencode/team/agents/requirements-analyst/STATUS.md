# requirements-analyst — 工作状态
> 2026-11-14 T22 验证记录完成

新增 R217。交付版 dist/lfz.exe SHA256 76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e，1117184 字节。普通 Windows 控制台 CP437/936/65001 的 BMP 中文输入、回显、诊断与退出码 15/15，私有 ConPTY 六组及 UTF-8 管道原始双通道互证。只写 CONOUT$ 在 CP437/936 仍乱码，CP65001 对照正常。emoji 的终端渲染待验。需求侧无阻塞，R304 OPEN。
