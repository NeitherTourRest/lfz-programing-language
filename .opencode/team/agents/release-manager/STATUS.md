# release-manager — 工作状态
> 最后更新：2026-10-30 by release-manager

## 当前状态
T18-A最后一轮候选功能独立PASS，精确归档/README/交付清单/六组提交准备完成。用户要求本轮后停止，无后继设计或源码；本角色不Git、不编译、不改源/profile。R304OPEN，无性能窗口、不tag，不宣称正式发布。

## 最终保全
- target/dist984064B、SHA4093714B2DC61BB72DBC4F43DB67A798A9BCAF152AD33A11D4D7457D1F7D9C86。ZIP660476B、SHA C37E323CFEA252EC416F85A8052E627C7771E43A6CEE0762E853DADD73C08B38，26项=递归21src+2Cargo+2测试+exe，含process_control/windows.rs与fixture.rs，全回读及冻结前后输入一致。
- T18-A-release-evidence/build/source-hashes/ZIP；Temp/T18-A-release/lfz-T18-A-409371.exe另存。dist复制后不再更动，不额外target或重编译。
- T18-A-independent-verification最终追加PASS：649 harness/公开45/toolplan7/自编12/旧全回归含T17 134app40、C29app33。人指南22/AI26分别作者证据，Windows已验，非Windows Unsupported。
- 默认受限环境C0000008/权限失败、E5 Close修正、NUL新oracle首次失败及全部原始证据保持；README/checklist区分最终功能PASS与未通过效率、未实施其余计划。

## Git准备及停止边界
- prepare-checkpoint.py baseline123230f，21源码/2测试仅从固定SHA ZIP产stagecopy/hash，原3用户块28/1/1各唯一过滤副本仅Temp，工作树原文不动。26份snapshot记录。
- 六组当前182路径（生成时点），root/所有角色最终记录齐后首次提交前再刷新，开始提交后不重跑；不宽泛stage，旧v2/Cargo/profile不动。
- 规划白名单为空，不纳后继T19/T18-B产物；本轮后不进入HTTP等新迭代。root代最后提交/推送并删除automation停止，实际回执未出现前不写已经推送/删除。

## 下一步
本角色材料已齐供root完成最终Git与停止动作，无交付准备阻塞。功能结束不等于R304通过，后续计划未实施；完成本轮停止。

## 关键经验
staged快照只排除用户初始28/1/1新增，工作树原文保留，禁止git add .或整文件stage混合记录。新skill/exe不冒用旧v2 manifest；旧9输入从原ZIP恢复，旧成绩不改算。性能门槛不能用局部收益抵消；候选保存与正式发布明确分开。