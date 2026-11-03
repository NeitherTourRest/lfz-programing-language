# tooling-dev — 工作状态
> 最后更新：2026-11-04 01:23 by tooling-dev

## 当前状态
T19 reader/lib作者实现及最终684/684回归完成，源码/编译已停止，交root公开/独立与perf。仅T19，不T20/automation；未dist/Git。

## 产出/证据
- src/line_reader.rs精确owned ABI，8KiB固定缓冲/追加前raw cap+capacity限制、LF/CRLF/EOF/BOM/NUL/跨块UTF8/overflow、普通file双metadata、nativecancel/deadline-only，finish/RAII，无生产FFI/close diag/sync_all。
- reader11unit+runtime5unit同编实际通过：活callback share0错误32负控，宿主/traced仍存活所有Outcome后排他成功；故障注入metadata/open/postopen/read、Drop spy恰一次，暖机50+1000次handle plateau；非Windows资源实测未声明。
- 首reader7/7保留；首次最终lib515/1因本人新test while/true同行TwoStatements，保完整失败，仅修新增test换行，未改parser/runtime/旧tests。随后全12harness684/684，0失败忽略，正常OS许可串行；watchdog未触发，build0。
- 最终1041920B/SHA5D2219F88DB8B3196CAC259941FC0A8A3B489EF03519BC7E20415F8EDF6388E5。旧dist有效输入红→同source新产物绿0真实记录。docs/tooling/evidence/T19/递归manifest+精确ZIP，docs/reports/T19-tooling.md齐。

## 下一步/限制
root接公开/独立，perf统一最终hash/no并发窗口，tooling不重复10/100/300MiB大fixture。std Drop不报告OS close失败，磁盘阻塞不硬截止；R304OPEN，无性能通过声明。任何源改须新编/hash。无其它阻塞。
