# T20 独立预审（静态，2026-11-04）

【状态】部分完成：v3 包暂不准冻结/派学生。未运行 LFZ、未构建、未启动服务或学生；本报告不是功能验收。只读取新 v3、§26 与 REPL 相关源码；旧 v2 未修改。R304 OPEN。

## 必须在冻结前解决

1. **P2 C01 错误后的批次保留没有可达见证。** runner.py:37-47 的坏 UTF8/超 cap 数据均在第一次有效批次之前，且 error case 不检查输出文件。反例：学生在读取出错时清空已写文件再 rethrow，现有所有功能 case 仍可能通过，但违反题面“已写批不回滚”。加入 32/33 条有效行后坏 UTF8 与超 cap 两类，绑定前32条完整输出；题面未要求失败时 flush 未满批，不应把该部分强加为必需。同步正负控制和冻结 revision。
2. **P2 C04 学生缺可用的自调服务。** 私有 runner 有宿主服务，但 README 禁止学生读 runner；目前学生包只含 TASK，无法在相同条件下完成先 check 后本地授权 run。须提供只暴露公共协议的自调 wrapper/服务入口与用法，不泄露私有 oracle；所有 wrapper 子产品启动计入既有周期预算。应先确认本机正常 OS 许可路径，不把沙箱网络拒绝归咎学生。
3. **P2 工具协议尚未可认证。** AUDIT_PROTOCOL:3/11 仍是“准备 wrapper”；完整真实 session trace、启动源码 snapshot 与预算边界控制未实证。先验证 40/41 tool、5/6 cycle、1200 秒边界、check-before-run、缺 trace 为 UNKNOWN；冻结实际 exe 与五文档 SHA 后才派学生。学生 commands.json 与 wrapper 均不能单独证明无旁路调用。当前三轴 UNKNOWN 默认原则正确，未知 model/effort 不得捏造。
4. **P2 收尾门禁未影响功能判定。** runner.py:73 保存 server_closed，但 :74 functional_axis 未纳入该值。建议 C04 显式 require server_closed is True，并保存 thread/server 收尾证据；不能记录失败同时认证全绿。此为评分器收尾门禁缺口，不是已观察到服务泄漏。

## 派题前澄清

- C02 TASK:3 开头导出清单为 rate/history/quote/setRate，后文要求 rates 导出 crash；不是完全不可解，但应把 crash 统一列入清单，避免正常实现者误判前句为穷尽清单。
- flagText **并非无用**：view.args() 须返回全部原 argv，runner quantity case 使用 `--allow-http=数据` 作为第三参数，能检验 -- 后参数保护。题面宜明示它须原样出现在 args 数组，不应被识别为 host 能力 flag。
- C02 共享三槽 + 私有索引与参考解一致，不要求越界 append；namespace rate 快照99与内部 home rate11、共享 history 三条值被现 oracle 区分。
- C03 nilOk 只看汇总字段，真实 attempt/不可伪造结果须 source 独立审阅；C01 真有界批/固定解析 closure 也须 source 审阅，不能仅凭小输入功能 PASS。
- C04 依次访问、404普通响应、302不跟随、UTF8失败后继续及无授权零 visits 现 runner 均有功能见证。尚未执行，不称通过。
- 新四题与旧八题难度不同，不做 8/8、2/8 的提升比例。

## REPL 静态范围

已对照 §26 读取 evaluator ReplSession 与 cli repl_loop：单 Interp/shared Env、per-cell home/source admission、错误来源 snapshot 后清 trace/depth、绝对 deadline 与 host token 保留、动态 ReplCapacity poison、stdin 锁不跨 eval、pipe 错误累计退出、compile 无 frames 时来源 adapter 均可见。未发现足以直接拒收的新生产缺陷；此结论仅静态，不能替代历史来源/default、真实 input 共享缓冲、TTY prompt/flush、host token/容量/poison 与旧完整回归实测。真实 TTY 尚未执行，不可用 mock interactive 单测冒称。

## 证据与下一步

只读命令：Get-Content 三启动文件/契约/包交接/四 TASK/AUDIT_PROTOCOL/README/参考解/负控 matrix/runner/oracles；rg 对 runner 及 REPL 对应行定位。无产品执行命令、无 cargo、无 Git、无网络。

题包 owner 先处理以上冻结门禁；最终候选 hash 与五文档冻结后，独立 verifier 实跑四正控制、八负控制、预算/trace 控制、最终公开36及自编 REPL/旧回归。正控制尚未实跑，因此不接受当前草案 manifest 为最终冻结。