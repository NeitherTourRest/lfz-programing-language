# release-manager — 工作状态
> 最后更新：2026-10-30 by release-manager

## 当前状态
T17最终独立PASS候选已精确封存并第一时间通知root可GO下一阶段；文档/六组提交准备完成。本角色不Git、不编译、不改源/profile。R304OPEN，无本批性能窗，非正式发布、不打tag。

## 保全与事实
- target/dist943104B SHA1946AB9CA5162D68F83A9D098424BD0BAFDF6FEADC6EA90527E0DB9E1DB12C45；ZIP629305B SHA3D9155399DFDA66A3B9B127A3CC11BF7954DE12354E59B52727C129C88683F83，23成员（18src+2Cargo+相对3639216变化2测试+exe），全回读和输入前后一致。Temp/T17-release/lfz-T17-1946ab.exe另存，之后不改dist。
- T17-independent-verification.md最终追加PASS：624harness/公开134/预算应用40/自编14/旧全回归及C29/app33。人指南32/AI38为作者实跑，经独立SHA审计，不冒称独立重跑。首次121/122与CONCERNS全部保留，没有覆盖失败历史。
- README/checklist明确步数/协作deadline/host token边界、默认unlimited、13类ExecutionLimitError不能attempt吞、副作用不回滚、native/阻塞IO内部不强制poll；不冒称资源或权限沙箱。

## Git准备
- T17-release-evidence/prepare-checkpoint.py固定baseline3639216；源18+两测试只从固定SHA的T17 ZIP产stagecopy/hash，实际变化4src和2测试入组，所有未来活动源/测试排除。
- 23份snapshot=18src+2tests+3用户；用户28/1/1三块过滤副本仅Temp，不改工作树。T18路径除三个显式规划白名单外排除，不能把后继实现当T17验收。
- 六组清单当前162路径（生成时点），全部role/root最后落盘后主会话首次提交前刷新一次，开始提交后不重跑。普通路径按direct_stage_paths，源/测试/用户按staged_snapshot_paths做index，不宽泛add活动T18源码。

## 下一步
主会话按六组快照提交/推送T17；活动后继源码继续未提交，不能误报只剩用户3块。没有收口准备阻塞，本角色已释放slot。

## 关键经验
staged快照只排除用户初始28/1/1新增，工作树原文保留，禁止git add .或整文件stage混合记录。新skill/exe不冒用旧v2 manifest；旧9输入从原ZIP恢复，旧成绩不改算。性能门槛不能用局部收益抵消；候选保存与正式发布明确分开。