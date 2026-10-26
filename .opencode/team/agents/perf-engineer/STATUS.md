# perf-engineer — 工作状态
> 最后更新：2026-10-25 by perf-engineer

## 当前状态
T12最终opt2主产物全组只测一次完成，15/19在原绝对线，3环境不确定、1未通过，R304继续开放。按可审阅候选checkpoint结论保存，不宣布整体效率完成；不再新增profile/无限择窗。已通知root全部测量结束，当前收工释放slot。

## 最终binary
1.4.0，759808B，release opt-level=2，SHA47ed2bfe2a2694b11bd611172cfcf0000d0bfc58e3b9bd809ca95f066dd0261f。binary由release-manager构建，本角色只测不改代码或profile。

## 本轮方法与证据
- 测前落盘方法后，默认未绑核、共同subject每copy静默5秒，原19顺序/旧新交替/2预热+7正式不变，无额外90秒。
- benchmarks/t12/results/opt2-same-path.json：19旧新266正式值+dist7次，逐次hash/wall/user/kernel/stdout完整，输出全部正确。
- opt2-new-features.json：9JSON/CLI×7正式=63输出独立oracle一致，每启动核最终hash。
- opt2-assessment.json：原始两JSON SHA绑定，15PASS/3环境不确定/1FAIL，R304 OPEN及同期paired比值。
- docs/reports/T12-performance.md开头为最终准确结论、每组old/new user/wall/paired/原线全表，所有早期失败/路径/CPU0/quiet1原始值和限制保留。

## 核心结果（wall含启动）
- noop新16.287在原16.307内但余量小，不能宣称启动稳定改善。
- numeric500k104.940→90.804ms；struct8k93.431→21.198ms（约77.3%降时，满足>=30%目标）；dist500k90.855ms。
- numeric10k/string2k/small4同期old也超原线，环境不确定；map8k新22.691>原22.521，old21.871在线内，仍未通过。
- string4k虽然绝对97.718<101.904达标，paired比旧75.879慢约28.8%，报告明确披露，不称全面加速。small16/64达标。
- JSON parse2k/4k/8k27.271/42.153/71.695ms；stringify15.898/21.058/25.168；-e/stdin/argv14.758/13.140/15.299。所有输出正确，峰值RSS未采，启动占比高不报纯吞吐。

## 下一步 / 限制
R304开放，当前作为可审阅checkpoint；不打正式标签由team-lead/release-manager按整体情况处理。未通过项不得用包体缩小或局部主循环收益抵消。精确LLVM/环境微因仍未知。

## 纪律
性能脚本/报告/自身记录唯一写范围，未改源码/测试/冻结v2，未Git。Temp均项目内。旧before/FB8/7641/default/CPU0/quiet1原始证据完整保留，未覆盖或挑最好成绩。