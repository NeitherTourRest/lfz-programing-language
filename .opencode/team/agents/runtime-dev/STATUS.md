# runtime-dev — 工作状态
> 最后更新：2026-10-27 19:51 by runtime-dev

## 当前状态
T13性能候选9CDA实测未形成验收改善证据，team-lead已拒收；经verifier冻结指纹复核与明确GO，两处生产优化已准确撤回到cbc974b原实现，保留全部8新增回归测试及失败证据。未在撤回后编译/测试，交tooling协调T14统一release验证。

## 产出与证据
- src/evaluator.rs与src/builtins.rs完整生产前缀与cbc974b Git对象逐字节相同；仅追加7+1个新增test mods，原有测试无修改/删除。
- docs/reports/T13-runtime-evidence/retraction-source-check.json保全前缀原/新SHA及8测试静态核对；生产前缀hash分别59981d01a98d4601ae74b9e56d0ea96234c6e9fb609e6fba44cf4c80816a4ee3、529bffd4927ea479529f93f68a0d00f26dfa370cd1a9262d56cc9f5281606213。
- docs/reports/T13-runtime.md只追加拒收/撤回节，原9CDA候选历史作者530/530（412+48+28+12+15+15）、8/8回归日志及manifest保持，不冒充撤回后的验证。
- 同期性能相对T12：map三档慢6.5%–16.2%、small4约30.4%、string8k约13.7%；完整原19线与限制见T13-performance.md。未定位微观原因，不继续择窗重测同候选。
- 不改Cargo/profile/spec/dist或Git；dist47ED保持。当前target旧候选exe不能当作恢复源码产物。

## 进行中/阻塞
无源码撤回阻塞。保留8项测试的恢复后实跑由tooling在T14最终release统一完成；性能R-304仍开放。

## 下一步
T14以恢复后的T12运行时推进check能力，等待协调验证。仅有实际缺陷再改运行时；不因源码少分配坚持发布未被测量支持的性能候选。

## 关键经验
减少分配的机制成立不代表实测更快；保留失败候选源码/二进制/原始结果，准确撤回生产实现而保留新增语义回归测试。撤回后新源码必须重新构建验证，历史9CDA功能通过不可替代。
