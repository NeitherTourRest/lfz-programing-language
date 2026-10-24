# tooling-dev — 工作状态
> 最后更新：2026-10-24 13:39 by tooling-dev

## 当前状态
bug-20261024-V01 的 runner 最小修复已完成，等待 verifier 独立复验。未改解释器 IO 语义、正式 LFZ 测试或其他角色产物，未执行 git 操作。

## 产出 / 验证证据
- `src/test_runner.rs`：discover 成功且用例集非空后、首个用例执行前，`create_dir_all("Temp")`；失败 stderr 中文诊断，stdout 空，exit 2，不改变 cwd。
- `tests/test_runner.rs`：新增 3 条 e2e，复用正式 `test_v11_io.lfz`，覆盖无 Temp 的默认一命令与显式目录 JSON、cwd 不变、Temp 同名文件冲突（文本/JSON均不执行脚本且不覆盖文件）、无效/空测试集不创建目录。
- `docs/tooling/runner-contract.md`：v1.3、§1.1 测试工作目录准备与错误通道说明。请 team-lead 转告 test-engineer。
- 先补测试后修复：`cargo test --test test_runner fresh_cwd_io_suite_prepares_temp -- --exact` 修前 FAILED（IOError：无法写入 Temp/v11_io_roundtrip.txt，exit 2）；修后 `cargo test --test test_runner` 15/15 passed。
- 全量 `cargo test` exit 0，494 passed（388 + 48 + 28 + 15 + 15），0 failed / ignored；日志 `Temp/tooling-v01-cargo-test.txt`。cargo 存在增量缓存 hard-link 失败后复制的环境 warning，不声称 0 warning。
- 当前工作区 `target/debug/lfz.exe test --json`：130/130 passed，failed 0，errored 0，exit 0。
- 复制完整 tests 到独立 cwd `Temp/tooling-v01-clean-c3cb31b0f8834cdf8642b2f3d3588816/`，启动前无 Temp，执行 `target/debug/lfz.exe test --json`（exe 用绝对路径）：130/130 passed，exit 0，执行后 Temp 为目录。结果 `Temp/tooling-v01-clean-c3cb31b0f8834cdf8642b2f3d3588816/result.json`。
- 所有测试进程 TEMP/TMP 均指向项目 `Temp/toolchain`；RUSTUP_HOME=`Temp/toolchain/rustup`，CARGO_HOME=`Temp/toolchain/cargo`，cargo 1.90.0。

## 进行中 / 阻塞
无；独立验收由 verifier 负责，未自行关闭 V01。

## 下一步
team-lead 唤醒 verifier 重跑原 clean-cwd witness，并更新全量计数 491 → 494。跨角色契约变更 ADR 请 team-lead 追加（本任务明确仅准许 runner/对应测试/契约/本人状态）。

## 关键经验
- ignored 目录可能隐藏 fresh checkout 的测试环境依赖；必须从无 Temp 的独立 cwd 实跑完整 tests。
- runner 准备约定测试目录，不要求 IO 内置自动创建任意父目录；发现失败或空集不应产生准备副作用。