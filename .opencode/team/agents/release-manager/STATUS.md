# release-manager — 工作状态
> 最后更新：2026-11-11 by release-manager

## 当前状态
T20 候选交付准备完成。项目内 dist 已更新，先在 Temp/T20-release/lfz-T19-5d2219.exe 保全原 T19。target/dist 都是 1100800 字节、SHA256 a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c。未编译或 Git 写入，等待 team-lead 最后生成分组、提交与远端核对。

## 证据与范围
- T20-release-evidence/verify-delivery.py 与 archive-verification.json 核对 760103 字节候选 ZIP（31f34b…）40 项精确字节、v3 ZIP（90932d…）46 项、冻结 manifest 793caa…、五 AI 和六人类指南及最终学生清单 79 项。
- T20 独立 705 / 公开 37 / 旧全回归通过。JSONdesk 6、人指南 16、AI 8 为作者实跑且经独立核证。真实 Windows ConPTY 英文提示、正常退出、错误后重置再退出通过，中文渲染待验。
- v3 首交卷独立认证 4/4、功能 27/27、源码与协议各 4/4、实际 29/29 产品记录匹配。四参考/八负/预算九项通过。formatter 仅完成评估，R304 OPEN、无 T20 性能窗口或正式 tag，旧 v2 未动。

## 分组与保护
- prepare-checkpoint.py --check-only 通过 402 路径预检；七组；11 暂存副本是六源码、两本轮测试、三用户块。工作树三块原文不改，Temp 过滤副本每块恰好 28/1/1 行。
- .gitattributes 限定 T20 SHA 绑定路径。132 路径 `git hash-object` 过滤前后逐个匹配，分组第一组纳入属性。最终团队落盘后由 root 去掉 --check-only 生成 JSON/Temp 副本，提交期间不刷新。
