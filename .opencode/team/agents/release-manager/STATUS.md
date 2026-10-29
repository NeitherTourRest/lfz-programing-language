# release-manager — 工作状态
> 最后更新：2026-10-29 by release-manager

## 当前状态
T16-A来源基础及兼容最终独立PASS，README/交付清单和本人记录已收口，尚未Git提交/推送。A仅Rust来源身份/错误sidecar/函数home基础，无import/export解析或模块加载执行，不改CLI/T15 Result。R304OPEN，无本批性能计时，不tag、不Git。

## 本轮产出
- target/dist783872B、SHA65A29EAEA54F32D1D2B404F22AED09389C3FFA71E6A81ACB8FA6351D32FA42B7，核验原样复制，无重编译/源码/profile改动；T16-A-release-evidence保存17src+Cargo/lock及产物指纹。
- README/checklist准确写A阶段边界、最终独立562 harness/attempt13/check117/CLI91含178/旧app37+11/jsonbatch38/JSON140、新Rust API witness编译执行成功，32组64进程逐字节差分及源/Cargo前后一致；UserFn新增home_module需外部Rust literal迁移，旧Span等公开shape保持，不虚称全部Rust API不变。
- prepare-checkpoint.py基线e1368de，五组明确路径，原139case前缀保持；Cargo/v2无差异，三用户块匹配各1，过滤副本仅在Temp/T16-A-release/stage。第5组显式含T16-B-design-draft.md草案路径。

## 下一步
最终实际独立结果已落README/清单，root/analyst所有记录落盘后只在首次提交前运行一次脚本刷新清单/用户快照；主会话代Git，用户原28/1/1行不变。之后不再改dist，释放slot。

## 关键经验
staged快照只排除用户初始28/1/1新增，工作树原文保留，禁止git add .或整文件stage混合记录。新skill/exe不冒用旧v2 manifest；旧9输入从原ZIP恢复，旧成绩不改算。性能门槛不能用局部收益抵消；候选保存与正式发布明确分开。