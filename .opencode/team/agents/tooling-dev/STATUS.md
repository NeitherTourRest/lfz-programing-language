# tooling-dev — 工作状态
> 最后更新：2026-10-30 18:26 by tooling-dev

## 当前状态
T18-A关闭诊断补丁完成，新649/649作者验收PASS，源码/编译已停止，交root独立验收。仅本轮交付后停止，不HTTP/后续，不覆盖dist/Git。

## 产出与证据
- run独占Handles注册表，必要早关显式close，所有Result路径最终显式逆序close_all，Drop仅异常兜底；消耗槽后单次CloseHandle不doubleclose。关闭失败加入owned diagnostics，正常不能假成功，已有Err/Cancelled主因保持。
- 新2unit覆盖真实关闭确认、正常/启动失败/Assign挂起拒绝/pipe失败诊断、真实marker握手活动取消仍保主因；注入仍真实close，无故意泄漏。原生Job/白名单/双流有界/后代回收/权限session/JSON/runner门禁原实现保留。
- runtime修非Windows绝对路径后进入Unsupported，新1unit同编。唯一cargo test --release --no-run/build0，十一harness649/649=487+53+28+12+12+7+5+9+6+15+15，0失败忽略，watchdog未触发。
- 新984064B/hash4093714B2DC61BB72DBC4F43DB67A798A9BCAF152AD33A11D4D7457D1F7D9C86。证据docs/tooling/evidence/T18-A-close/含递归manifest/精确ZIP/日志；docs/reports/T18-A-tooling.md追加收口。E5F57/646阶段ZIP及日志原样保全。

## 待协调
root接独立/公开/指南验收，任何后续源码改动必须新编/hash。Windows结果不代表Linux支持；非Windows仅Unsupported，未实测。Unsupported dead_code/环境警告保留。R304OPEN，不正式tag，无其他阻塞。
