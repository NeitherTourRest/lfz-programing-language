# release-manager — 工作状态
> 最后更新：2026-10-27 by release-manager

## 当前状态
T12 1.4.0候选工作快照已由主会话完成8组原子提交并推送main，远端核对为9fe5e401ad4cfa7d623ec1e872558165027a9e48。非正式发布，无v1.4.0标签；旧v1.3.0保持不动。最终47ED功能独立PASS，R-304仍OPEN（19原线15达线、3环境不确定、1map8k未过）。本角色只记录实际回执，未Git写入。

## 产出与证据
- README.md、delivery-checklist.md、T12-release-plan.md已引用72内置、最终522/166/91、应用37+11、140witness与16旧新差分覆盖8类while范围且stdout/stderr/exit逐字节一致。最终独立报告T12-independent-verification.md及T12-independent-final/；作者指南35块+10/AI30保留各自证据语境，不冒称新盲测。
- 最终target/dist759808B、SHA256 47ED2BFE2A2694B11BD611172CFCF0000D0BFC58E3B9BD809CA95F066DD0261F；Cargo1.4.0、opt-level2、LTO/codegen1/strip。final-opt2/绑定构建/522 release测试/Cargo/源hash。
- 性能只报同期实测局部：struct8k93.431→21.198ms（耗时约-77.3%），numeric500k104.940→90.804ms；startup16.287<=16.307ms余量小。map8k22.691>22.521、old21.871在线，其它3项环境不确定；不把15/19说全部通过。
- T12-release-evidence/checkpoint-preflight.json及commit-groups.json提供逐路径hash/8组清单；preflight-check.py可在开始提交前刷新，纳入最终独立材料/角色记录；开始提交后不可误当全历史审计。
- 原99cases前缀保留+32新增、旧LFZ/Rust测试无修改、v2包/旧报告目录无Git改动；初始用户3保护块各1。文本及新增ZIP成员凭据模式检查不替代全知敏感性证明。

## 实际Git回执（主会话执行）
- 8组提交依次：37bb926 / 91510d3 / 4880d21 / 5df9373 / 6b744f0 / c34d3f5 / f8cb4f1 / 9fe5e40。
- main已推送，远端HEAD核对9fe5e401ad4cfa7d623ec1e872558165027a9e48；旧v1.3.0剥离仍53e6d88aeb2467f04e0118ee408459eaf86f3a5c，无v1.4.0标签。
- 主会话核对提交后工作树仅余用户原3块28/1/1行；本次两份角色记账待主会话最后提交再推送，不自指未产生的最终记账hash。

## 阻塞/下一步
R-304继续开放，后续处理效率项；本轮仅候选checkpoint，8组源码/证据提交与推送已经完成。主会话保存最后角色记账并再推送；本角色仅更新自己的STATUS/JOURNAL，不再修改源码、报告或原始日志。

## 关键经验
staged快照只排除用户初始28/1/1新增，工作树原文保留，禁止git add .或整文件stage混合记录。新skill/exe不冒用旧v2 manifest；旧9输入从原ZIP恢复，旧成绩不改算。性能门槛不能用局部收益抵消；候选保存与正式发布明确分开。