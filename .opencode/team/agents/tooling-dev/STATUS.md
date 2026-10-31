# tooling-dev — 工作状态
> 最后更新：2026-10-31 00:15 by tooling-dev

## 当前状态
T18-B HTTP作者实现与验收完成，源码/编译停止，交root公开49/独立验收。仅本轮，不T19/automation，不dist/Git。

## 产出
- http_control.rs纯owned ABI+共用验证helper、私有Windows async单owner/有界completion/稳定buffer与context/安全close最后回调握手；异常展开也先回收。非Windows Unsupported。
- CLI独立allow-http与首类别、CODEargv保护、双diagnostic宿主门禁；获批runner最小双列表fail-closed。旧process文案/旧测试不改。
- 先旧binary红，第一编译生命周期4/4保留；第二最终7/7，含原20normal及新增40Barrier真实cancel完成race，active0/closing断言；close注入仍真实回收、保Deadline。
- 新完整release12harness667/667，0失败忽略，build0。exe1029120B/SHA18D328D9E48780F8E6AE638F9AD24F53186D94F2207CA24DA8928CBBBA0D0916。
- API原始头UTF8/u16le及public-header-witness.json（固定wire头59+body2=N61，实际原文）；TLS真实自签拒绝12175、server应用bytes0，无trust改动/可信TLS成功声明。首openssl config失败保留。
- docs/reports/T18-B-tooling.md、docs/tooling/http-contract.md；docs/tooling/evidence/T18-B/递归manifest/精确ZIP/全部阶段日志。旧A证据保持，临时测试私钥未入ZIP。

## 下一步/限制
root接公开/独立矩阵（方法body/redirect/请求次数/坏协议等），作者7项不冒称全公开覆盖；任何产品修源须重编新hash。Windows不代表Linux支持，OS异常回收不硬实时，R304OPEN。无额外阻塞。
