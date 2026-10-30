# release-manager — 工作状态
> 最后更新：2026-10-30 by release-manager

## 当前状态
T16-C独立功能PASS候选快照/文档/提交准备完成，已先通知root可以GO T17源码。B已推main9d7e1aa，不重做B；C仍候选、R304OPEN、无本轮性能计时、不tag，本角色不Git。

## 精确保全
- C target/dist902656B、SHA CDEDC9B980AA207B28BF895EF95CE76FDE18E47EF758C19C187B3F9C2DA08006。ZIP611909B、SHA8A6F9D0ACADB86FA56CA49BB7E0A82650F7FD00902348DBF46B4BFBE7B8A9532，24成员=18src+2Cargo+3测试+exe，全部回读及冻结前后指纹一致，无T17 spec混入。
- docs/reports/T16-C-release-evidence/build.json/source-hashes.json/candidate-snapshot.zip；Temp/T16-C-release/lfz-C-cdedc9.exe另存精确exe，之后不改dist，不编译/改源码/profile。
- 独立599harness/公开29/新应用33/自编15及旧13/117/91含178/旧app37+11/jsonbatch38/JSON140全PASS。人指南11/AI18分列作者实跑，README/checklist已引用实际独立报告。

## Git防混准备
- prepare-checkpoint.py固定baseline9d7e1aa，所有18源码+cli_t16b/cli_t16c/t16c_cli_run三个测试均从固定SHA的C ZIP生成stagecopy/hash，仅相对B实际改动入组；T17活动src/新测试排除。
- 24项snapshot记录=18src+3tests+3用户，用户原28/1/1块各匹配1，只Temp过滤副本不改工作树。6组清单已生成，root/角色最终记录齐后首次提交前刷新一次，提交过程中不再刷新。
- T17路径仅三个显式规划白名单，§22仅已冻结未实现/未验收，不能作C预算能力；其它T17产物排除，C活跃后不再对当前源码做C hash证明。

## 下一步
主会话按snapshot index代执行C六组提交/推送，活动T17源继续未提交；不得误报只余用户3块。本角色已落记录并释放slot，未Git、不改dist/源码。

## 关键经验
staged快照只排除用户初始28/1/1新增，工作树原文保留，禁止git add .或整文件stage混合记录。新skill/exe不冒用旧v2 manifest；旧9输入从原ZIP恢复，旧成绩不改算。性能门槛不能用局部收益抵消；候选保存与正式发布明确分开。