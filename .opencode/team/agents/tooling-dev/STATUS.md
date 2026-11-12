# tooling-dev — 工作状态
> 2026-11-13 00:34

T21 作者实现和验收完成。`src/formatter.rs`、`src/lib.rs`、`src/cli.rs` 与 `tests/cli_t21.rs` 已落。core 停源后统一 release 构建，14 组 715/715 通过。候选 1117184B，SHA256 `76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e`。

保真 oracle 核对原字节、幂等、合法/错误输入、模块独立格式化和旧裸 fmt CLI。同路径原脚本与格式化脚本的可观察行列、traceback、NameError JSON 和文件效果一致。报告 `docs/reports/T21-tooling.md`，递归清单和快照位于 `docs/tooling/evidence/T21/`。

源码与构建已冻结，交 root 独立验收。R304 OPEN。
