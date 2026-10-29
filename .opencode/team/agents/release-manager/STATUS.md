# release-manager — 工作状态
> 最后更新：2026-10-29 by release-manager

## 当前状态
T14候选最终交付文档收口完成，独立check及兼容功能PASS，R304仍OPEN，无T14性能窗口，不正式发布或打v1.4.0标签。本角色不Git；主会话已完成前5组提交，第6组记账及push待主会话实际执行，不提前声称已推送。

## 最终证据
- dist/target774656B、SHA83F56A194E216862C45377F4C3971B085744A7BA2363163E1EDAC6AF5E85AC79，T14-release-evidence/保全构建来源与16src/Cargo指纹。本任务未重编译、改源码/profile/dist。
- T14-independent-verification.md及原始目录：542 release、公开check117、旧CLI91含166官方、app37+11、旧独立140、新增57见证全部实际独立PASS。人指南27/AI27作者实跑分别引用，不虚增独立计数。
- README与delivery-checklist已改最终独立验收完成，保留check非执行/非静态分析边界、T13拒收及R304开放。公开首次116/117作者UTF8 oracle订正和原始失败保留，不虚称产品修复。
- 本次只读git diff确认Cargo/lock、原tests/lfz与fixtures、冻结v2包和旧首轮目录无未提交改动；自己两文档diff --check通过。
- T13两生产优化拒收并恢复原生产行为，8新增回归测试保留；9cda源码/exe ZIP及失败性能记录保留，不能当83f性能结果。

## Git实际/待执行边界
- 主会话回执前5组86acd9e/c5aa671/625dc69/8589a3c/e1ed520已提交。
- 按主调度最新指令不再刷新prepare-groups，避免已提交路径消失；第6组继续使用原30 direct+3 snapshot清单。
- 用户原三块28/1/1行须过滤stage快照保留工作树，不宽泛stage；旧v1.3.0不动，v1.4.0不创建。第6组及推送由主会话执行后另记实际回执。

## 下一步/阻塞
本角色文档/记录已落盘可供第6组提交；R304保持OPEN，后续效率另批，不以T13性能或check功能通过抵消。没有最终提交记账准备阻塞。

## 关键经验
staged快照只排除用户初始28/1/1新增，工作树原文保留，禁止git add .或整文件stage混合记录。新skill/exe不冒用旧v2 manifest；旧9输入从原ZIP恢复，旧成绩不改算。性能门槛不能用局部收益抵消；候选保存与正式发布明确分开。