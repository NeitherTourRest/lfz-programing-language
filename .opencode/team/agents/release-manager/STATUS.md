# release-manager — 工作状态
> 最后更新：2026-11-04 by release-manager

## 当前状态
T19最终候选功能及单窗口规模审计PASS交付准备完成，仅本轮收口不T20/automation。R304仍OPEN，原19线未改，不tag、不正式发布。本角色不编译产品、不改源/profile、不Git，最后提交推送待root回执。

## 精确保全
- target/dist1041920B、SHA5D2219F88DB8B3196CAC259941FC0A8A3B489EF03519BC7E20415F8EDF6388E5；28项ZIP701750B/SHA8C90CD23ECB72E51590FCD3C4CEDBAB7F7FB2F24D2D2D22EA41B2261A40383BB，递归24src+2Cargo+本轮测试+exe全回读与前后输入一致。
- T19-release-evidence/build/source-hashes/ZIP，Temp/T19-release/lfz-T19-5d2219.exe另存；大输入不归档、不Git，dist复制后不再改。

## 结果与边界
- 独立功能684/全部旧回归/公开47/app14/自编8 PASS；人指南42checkrun/AI45作者实跑，独立报告最后审计规模111文件SHA/bytes、所有42样本及100MiB实际675779行oracle/SHA通过。
- 唯一window01=7warm+35formal，无失败重窗；固定驱动10/100/300MiB peak WS约6.5MiB，10MiBwhole34.9MiB；300MiB LFZ9.335709s/Python6.244406s，LFZ慢49.5%，不夸吞吐领先或R304通过。
- 固定解析闭包/text cell避免逐行函数元数据增长，用户callback仍可分配/积累；普通文件TOCTOU和native阻塞、RAII关闭不能报告OSclose错误如实记录。oracle/driver初版、515-1故障及全部改进/失败证据保留。

## Git准备及停止
- prepare-checkpoint.py baselinec2a3d3946ddd874cf9aeb87bbc33284fc3097a51，7组正确逐行/测量语义，perf单独第6组包含benchmarks/t19五文件（measure/prepare/reference/stream/whole）和performance全部原始证据，第7组收工。
- 源24+测试1只从固定ZIP生成stagecopy/hash，3用户块28/1/1过滤副本只Temp且各1，28snapshot记录。当前311路径，root/所有角色最后落盘后首次提交前刷新，提交中不重跑，禁止宽泛add。
- 已核perf五文件全部在第6组、清单无Temp路径/10MiB以上文件；README/checklist差异检查通过，旧v2/tags/Cargo及原性能门槛不动。
- root代最后Git与推送，本角色未Git、不虚报已push；仅本轮后停止，不启动T20/自动化，其余计划未实施。

## 关键经验
staged快照只排除用户初始28/1/1新增，工作树原文保留，禁止git add .或整文件stage混合记录。新skill/exe不冒用旧v2 manifest；旧9输入从原ZIP恢复，旧成绩不改算。性能门槛不能用局部收益抵消；候选保存与正式发布明确分开。