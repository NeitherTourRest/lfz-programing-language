# T11-15 v1.1 实现独立复验

验证者：verifier；日期：2026-10-24（Asia/Shanghai）。只验证，不修改实现、正式测试或规范。

## 结论与边界

语言实现定向验证通过：74 个独立进程 witness 全 PASS，15 个新增内置及 range 双参均覆盖；独立 cargo test 491/491；现有工作区 lfz test --json 130/130。没有发现新增语言实现缺陷。

交付结论为**本批实现及测试环境可交付**：初验独立干净 CWD 下黑盒 129 PASS / 1 ERROR，退出 2；测试环境缺陷 bug-20261024-V01 已在下述 13:41 独立复验中确认修复并关闭。历史“同题8盲测”本轮仅核查材料，未重跑，原 2/8 基线保持原值，不能据本报告声称改善。

## 被验产物与复现

使用 target/debug/lfz.exe；字节数、mtime、SHA256 见 evidence/artifact.json。原始证据目录为 docs/reports/T11-v11-reverification-evidence/（下文 evidence/ 均指该目录）。cargo test 无重新编译，确认源码测试与当前构建一致。

- 定向复验：从项目根执行 `& ./docs/reports/T11-v11-reverification-evidence/run.ps1`。脚本仅在项目 Temp/verifier-v11 内写临时文件，在 evidence/witness.json 保存每个进程的源代码、命令、CWD、退出码、stdout、stderr、期望及判定。74/74 PASS。
- 单测：环境 RUSTUP_HOME=项目/Temp/toolchain/rustup，CARGO_HOME=项目/Temp/toolchain/cargo，TEMP/TMP=项目/Temp/toolchain；执行 `Temp/toolchain/cargo/bin/cargo.exe test`，退出 0。388 + 48 + 28 + 12 + 15 = 491，均 0 failed/ignored；原文 evidence/cargo-test.txt。
- 黑盒：`target/debug/lfz.exe test --json`，退出 0，total=130/ passed=130/failed=0/errored=0；stdout evidence/blackbox.json，程序输出 evidence/blackbox.stderr.txt。

## 独立 witness 覆盖

| 方面 | 用例及证据 ID | 实测结论 |
|---|---|---|
| Unicode 标量而非字节/UTF16 | unicode：emoji+中文 indexOf，下标2；substring夹取到标量边界，负数/i64极值夹取；endsWith空串/不匹配 | PASS |
| pad 新值/标量截断 | pad：中文与emoji fill 重复、截断；padStart/padEnd、负宽度、原串保持 | PASS |
| ord/chr 合法域 | codepoints：0、代理区前后55295/57344、0x10FFFF、emoji往返 | PASS |
| ValueMsg 三条 | ord_empty/ord_combining/ord_count 精确0/2/5标量消息；chr负数/代理区两端/上界外/i64最大；fill_empty/fill_start 空串，即使不需填充仍报错 | PASS，类名与中文逐字匹配 |
| range 两参 | range：跨0、同端/反向空、i64 MIN/MAX附近的短区间、负单参 | PASS |
| 容量安全 | range_capacity 使用i64全跨度；pad_capacity 用i64最大宽+四字节fill，容量预检失败 | PASS，OverflowError精确消息、exit2；未尝试分配巨量内存或故意崩溃 |
| contains 深结构/环 | contains：嵌套array+struct、空array、不存在、int/float相等、自环struct对比 | PASS |
| IEEE 与 int 加宽 | ieee：sin/cos/log/exp普通整数，返回float；log(0)=-inf、负数nan、exp上下溢；sin/cos无穷及log(nan) | PASS |
| 参数个数 | 15新增内置逐一0参/超1参，共30进程；range0参/3参，精确中文参数数消息 | PASS |
| 类型错误 | 15新增内置及range各一错误类型，含file contents数字/非string path | PASS，TypeError/exit2 |
| 文件IO正常 | io：Unicode/emoji读回、覆盖截断、append保留前缀、不存在则创建、写追加返回nil | PASS |
| CWD | cwd：脚本仍位于Temp/verifier-v11，用该目录为CWD执行相对路径cwd.txt | PASS，文件实际落该CWD |
| IO错误 | io_invalid非法字节FF FE 80；io_missing；io_parent/io_append_parent无父目录；io_no_tilde不展开~ | PASS，精确IOError消息/exit2 |
| JSON通道 | 每个witness stdout独立JSON可解析；json_channels中print/eprint/独占行dump均在stderr，stdout仅成功JSON | PASS |
| 文本错误通道 | run io_invalid.lfz 非JSON；evidence/plain-io-error.json | PASS：exit2、stdout空、stderr含Traceback和UTF8错误消息 |

所有临时写入均在项目 Temp，未执行项目外写操作，未修改产品或正式测试。验证脚本起初使用分号分隔语句，被既有规范拒绝；已修正为换行，以上统计仅计最终合法 witness，不将验证脚本错误登记为产品缺陷。

## 缺陷单 bug-20261024-V01

- 交付物：一键黑盒测试的环境准备；建议 owner：tooling-dev，经team-lead调度。
- 最小复现：在项目 Temp 中新建独立目录，复制 tests/ 到其中；不建立该目录的 Temp 子目录；设置进程 CWD 为该新目录，运行 `target/debug/lfz.exe test --json <独立目录/tests绝对路径>`。
- 期望：新 checkout 的测试无需遗留的被忽略 Temp 目录即可一键执行，全部通过。
- 实际：total=130/ passed=129/failed=0/errored=1、exit2。test_v11_io.lfz 报 `IOError: 无法写入：Temp/v11_io_roundtrip.txt`。
- 证据：evidence/clean-cwd-command.txt（精确命令/CWD/exit/TempExists=False）、clean-cwd.json、clean-cwd.stderr.txt。
- 严重度：中，影响可复现交付。解释器“不自动创建父目录”符合 interface-contract.md §10.7，不能修改该语言语义来规避。
- 初验未移动或删除既有 Temp，未修复缺陷；已立即报告 team-lead。该单初验开放，现经后续独立复验关闭，原失败证据完整保留。

## 8题盲测历史材料核查（未重跑）

对 `.opencode/`、`docs/` 及全仓文件名执行 rg 搜索（含hidden，排除 .git、Temp/toolchain），并核查 team-lead JOURNAL、FEATURE-AUDIT 与 docs/reports/blind-test/。

找到5份历史题目程序：hashmap_chain.lfz、json_mini.lfz、expr_eval.lfz、knapsack_dp.lfz、maze_bfs.lfz；找到2份程序报告：knapsack_dp.report.md、maze_bfs.report.md；另有 run_output.txt 与多份探针。历史文字记8题、2/8通过，但未找到完整8题的原始任务书、其余3题名称/验收条件及完整隔离提示记录。因此“同题8重跑”前置材料在可读仓库内不齐备，不能从5份程序反推为8题，更不能把执行旧程序冒充新的零上下文盲测。建议由 team-lead 恢复原始8题任务书及验收清单，再另立新盲测批次。本轮状态是材料核查完成、盲测重跑未开始。
## V01 修复后独立复验（2026-10-24 13:41）

轻量启动后仅复跑原缺陷场景及冲突路径，未重复74个功能witness。修复由tooling-dev完成，verifier只验证。

被验 `target/debug/lfz.exe` 为1387520字节；SHA256见 evidence/clean-cwd-fixed.json 和 temp-file-conflict.json（分别记录执行命令/CWD/stdout/stderr/exit）。未使用旧release产物。

1. **原clean-cwd原样重跑：PASS**。使用初验原独立目录和tests副本，执行前确认无Temp子目录。运行同一条 `lfz.exe test --json <原副本tests绝对路径>`，exit0，130/130通过，runner自动准备该CWD的Temp目录；脚本相对路径仍按原CWD解析。证据 evidence/clean-cwd-fixed.json。初验失败文件 clean-cwd.json/clean-cwd-command.txt 未覆盖。
2. **Temp是普通文件：PASS**。新独立CWD中预建名为Temp的普通文件，内容sentinel；执行有效非空测试集。exit2，stdout长度0，stderr为 `lfz test: 无法准备测试工作目录 'Temp'：当文件已存在时，无法创建该文件。 (os error 183)`；sentinel文件仍为原内容。证据 evidence/temp-file-conflict.json。

**bug-20261024-V01：已修复 → 关闭。** 本次直接独立复验仅含以上2项；494单测为tooling-dev/team-lead另行测试，本报告不将其冒记为verifier本次重跑。既有491独立单测与74功能witness结果保留，盲测材料不齐备的限制不变。