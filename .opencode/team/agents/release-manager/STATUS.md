# release-manager — 工作状态
> 最后更新：2026-10-29 by release-manager

## 当前状态
T15最终功能独立回执已齐，README/交付清单已收口；人指南20/AI25作者实跑亦通过，需求最终状态待analyst关闭，不正式发布、不Git。R304OPEN，无T15性能计时。

## 本轮产出
- 核target后复制dist均782848B，SHA1A5A3D1C650440929BB599961763573DEF8B56D33811A8F266688246AC63DD6C；T15-release-evidence/build.json及18项源码/Cargo指纹保全。不编译不改源/profile，之后不改dist。
- README/checklist更新73内置attempt、业务白名单/资源不捕获/无副作用回滚/参数边界与jsonbatch。最终独立PASS由verifier经主调度回执确认：549 harness、178官方（旧CLI91full内）、attempt13、check117、旧app37+11、新app38、旧JSON140及额外38独立进程；人20/AI25仍分列作者证据。
- T15-release-evidence/commit-groups.json六组明确路径，prepare-checkpoint.py可在提交前刷新并在Temp/T15-release/stage生成三份过滤快照；工作树用户原28/1/1行不动，匹配各1。
- 基线2c0e079，原131cases前缀保留，Cargo/lock及旧v2无差异；NEXT-ITERATIONS及上轮root未提交记录纳入第6组；按主调度追加，仅显式容纳T16-design-draft.md草案路径，不扩其它匹配规则。原3项注册元数据72→73为专项授权，不扩大旧测试改动。

## 下一步
本角色最终文档/记录已完成，释放slot供analyst关闭功能需求；没有Git写入，不虚报已提交/推送。最终角色及root记录落盘后主会话刷新明确路径/快照并代Git，禁止宽泛stage，旧tags不动。

## 关键经验
staged快照只排除用户初始28/1/1新增，工作树原文保留，禁止git add .或整文件stage混合记录。新skill/exe不冒用旧v2 manifest；旧9输入从原ZIP恢复，旧成绩不改算。性能门槛不能用局部收益抵消；候选保存与正式发布明确分开。