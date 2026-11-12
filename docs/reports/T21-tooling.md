# T21 格式化作者验收

最终 Windows 候选为 1117184 字节，SHA-256 `76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e`。`cargo test --release --no-run` 和 `cargo build --release` 成功。使用正常 Windows 许可运行同一批 14 个测试程序，715 项全部通过。每个测试程序有 90 秒外部超时，实测均正常退出。已有 Windows 句柄测试沿正常许可运行。

`lfz fmt FILE` 在原始参数恰为 `fmt` 和非空、非短横开头的路径时进入格式化。其他形态由原解析器处理。集成测试确认裸 `fmt`、双方向 `--json`、裸 `fmt` 后的脚本参数、`run fmt` 与 `fmt.lfz` 的行为。模块格式化只加载和校验当前文件。完整结果产生后用 `write_all` 和 `flush` 写 stdout，写入错误返回 2。输入路径始终只读。

格式化 API 先按当前 loader、lexer 与 parser 检查输入。格式化专用 lexer 给出顶层 CODE 空白的归一化 Unicode 标量区间。格式化器将区间映射到原文件字节，原样复制其余片段。BOM、`#42`、原换行序列、注释、插值、数字原文、管道和双分号均有精确字节用例。行首 TAB 按 1 比 1 替换，行尾空白删除，保留会导致裸 CR 与 LF 合并的间隔。结果再次加载、词法分析和解析，比较所有 token 内容和行号，以及除 Newline/EOF 外的起始列。

旧候选对有效 `fmt FILE` 的缺功能红测已保存。定向 formatter 3/3、CLI 5/5 通过，覆盖无效 UTF-8、前导、词法、语法、缺文件和参数误用的 stdout 空结果。同一文件路径下，旧候选执行原文件与新候选执行格式化结果的 `attempt.error.line`、`col`、`traceback`、未捕获 NameError JSON 及文件副作用完全一致。原件和运行记录位于 `docs/tooling/evidence/T21/`。同目录包含递归源码及 Rust 测试哈希、编译和测试日志、精确候选快照。

作者验收完成，源文件与构建已冻结，交主调度进行独立验收。R304 仍开放。
