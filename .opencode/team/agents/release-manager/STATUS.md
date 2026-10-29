# release-manager — 工作状态
> 最后更新：2026-10-30 by release-manager

## 当前状态
T16-B来源快照与最终候选文档/分组准备完成，已落盘T16-B-independent-verification.md确认独立584harness/38公开/18额外模块进程及旧13/117/91含178/apps37+11/38/140全PASS。B仅graph check不初始化执行，C文档仅规划；R304OPEN，无计时，不正式发布不tag，本角色不Git。

## 产出
- B target/dist859136B SHA8EA375DCD08098EF61D040A57A7B1AD28A28C2FDC25A5CA4921A75B728944480；ZIP582683B SHA DB4AE08E66DD43D7A90C9DDD33706B170EE36C014114840922431C36292556CA，21成员=18src+2Cargo+exe，全回读且快照后源/Cargo一致。无C spec混入。Temp/T16-B-release/lfz-B-8ea375.exe保留旧B字节，之后不改dist。
- T16-B-release-evidence/build.json/source-hashes.json、candidate-snapshot.zip、cli_t16b.rs.txt；README/checklist准确B静态图与C执行尚未验收边界。
- prepare-checkpoint.py基线738e198生成5组清单；src始终从B ZIP生成18份stagecopy/hash，仅B实际变化8src入组，未来C新src排除。cli_t16b从冻结txt生成test快照。原3用户块各1，仅Temp过滤快照，工作树28/1/1不动。
- tests路径仅白名单BCLI/公开脚本/REPORT/coverage/.gitattributes，未来C测试排除。最终脚本显式过滤所有含T16-C路径且非3个C规划白名单的产物，记录excluded_C_nonplanning_paths；包括C-runtime报告/证据及C-tooling产物均排除。B ZIP先核固定SHA，再逐源码核T16-B指纹，仅生成冻结stagecopy，从不核活动C源作B验证。C及T17设计草案仅作为明确规划文件入记账，不视验收。默认canonicalize失败/控制原日志保留，不改字节凑diff-check。

## 下一步
主会话按5组清单代Git；source及BCLI测试必须按snapshot path/hash-object/update-index，不git add活动C源码。C活跃改动保持工作树未提交，故提交后不声称只剩用户3块。所有角色/root落盘后首次提交前刷新一次；开始提交后不再重跑。无编译/profile/Git写入，释放slot。

## 关键经验
staged快照只排除用户初始28/1/1新增，工作树原文保留，禁止git add .或整文件stage混合记录。新skill/exe不冒用旧v2 manifest；旧9输入从原ZIP恢复，旧成绩不改算。性能门槛不能用局部收益抵消；候选保存与正式发布明确分开。