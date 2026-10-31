# release-manager — 工作状态
> 最后更新：2026-10-31 by release-manager

## 当前状态
T18-B HTTP最终独立PASS交付准备完成，用户只授权本轮B收口，不T19，不重建已删除自动化。R304OPEN，无性能窗、不正式发布不tag。本角色未Git/编译/修改产品或profile，最终push待root实际回执。

## 精确归档
- target/dist1029120B、SHA18D328D9E48780F8E6AE638F9AD24F53186D94F2207CA24DA8928CBBBA0D0916；递归23src含http_control/windows.rs及旧process模块，2Cargo+2本轮tests+exe共28成员ZIP691881B，SHA1333CFCF3990E7B92AAB05ADB6746BC48A1A7D44524CA1915C07FEB9B65CB8FA，全回读与冻结前后输入一致。
- T18-B-release-evidence/build/source-hashes/candidate-snapshot.zip；Temp/T18-B-release/lfz-T18-B-18d328.exe另存，不再改dist。没有临时TLS私钥入ZIP。新增项目文本私钥header扫描0命中；docs/tooling/evidence/T18-B/tls-cert.pem是公开CERTIFICATE审计材料，不是私钥，保留。

## 最终事实/边界
- T18-B-independent-verification.md：667harness、旧全回归、HTTP49/app7/独立11全PASS；首4normal和补40真实race证据保留。人指南21/AI25分列作者成绩，AI默认17/25有8 WinHTTP12029、同脚本同hash正常OS许可25/25，server_closed=true，五文档SHA一致。
- README/checklist准确75内置/13错误类、Windows已验而非Windows Unsupported、API完整头文本UTF8+body非wire限额、仅系统自签拒绝审计无可信TLS成功，不宣传网络/权限内存沙箱。
- 所有失败、首normal/race和TLS首config错误、默认12029保留原文，没有因最终PASS改写历史。其余计划未实施，只B收口。

## Git准备/下一步
- prepare-checkpoint.py baseline740fd32371584da1d5afc9b12d262a434b1c87f8，六组HTTP语义message，28snapshot=23src+2tests+3用户，源/测试只从固定ZIP生成stagecopy/hash。用户原28/1/1块各1，副本只Temp，禁止整文件add混合记录。
- 六组当前190路径：6/11/104/8/28/33（含snapshot），root/所有角色最后落盘后首次提交前刷新一次，开始提交后不重跑。旧v2/Cargo/profile保持，后继T19/T18-C排除，无新增规划白名单。
- root代最终提交/推送，本角色没有Git写入，不提前宣称已push；文档差异检查通过，材料齐可收口停止。

## 关键经验
staged快照只排除用户初始28/1/1新增，工作树原文保留，禁止git add .或整文件stage混合记录。新skill/exe不冒用旧v2 manifest；旧9输入从原ZIP恢复，旧成绩不改算。性能门槛不能用局部收益抵消；候选保存与正式发布明确分开。