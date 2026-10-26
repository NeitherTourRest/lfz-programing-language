# perf-engineer — 工作日志
> 只追加，最新条目在最上方。
## [2026-09-27 12:06] P6 — LFZ vs Python 性能基准与报告交付
- 来源: team-lead 任务书「P6 性能基准与报告（评分项 3/10 分，交付物 4）」
- 完成:
  1. 建 `benchmarks/`：6 基准（数值循环 / 函数调用 / 递归 / 数组内建 / 结构体字典 / 字符串），每个含功能等价 LFZ + Python 实现（同算法同规模，输出经 stdin 传 N）。
  2. 写一键 harness `run_all.py`（仅标准库）：预热 2 轮 + 计时 5 轮取中位数（附 min/max）+ 每轮断言 LFZ 输出 == Python 输出；单列 noop 启动基线 + net 值；写 `results/raw.json`。
  3. 执行测量并产出 `docs/reports/performance.md`（方法/环境/数据表/结论/瓶颈/局限/期望输出）。
  4. 探针定位两处超线性：`push` 整表拷贝 O(N²）、struct 字符串键线性扫描 O(N²）。
- 产出:
  - `benchmarks/{run_all.py,README.md}`、`benchmarks/lfz/*.lfz`、`benchmarks/python/*.py`、`benchmarks/fixtures/noop.*`、`benchmarks/results/raw.json`
  - `docs/reports/performance.md`
  - 证据：`cargo build --release` 全量重编 0 warning/0 error（3.08 s）；`python benchmarks/run_all.py --warmup 2 --runs 5` → 18/18 输出一致；noop 启动 LFZ 11.67 ms vs CPython 42.34 ms。
- 决策: ADR `[2026-09-27 12:06] [perf-engineer] P6 性能基准基线与两处超线性退化上报`（已追加 DECISIONS.md）。
- 下一步: runtime-dev 按报告 §6 优化后，perf-engineer 复测更新数据。
- 阻塞: 无。

## [2026-09-22] 角色初始化
- 来源: 团队初始化（系统构建）
- 完成: 角色定义与活文档建立
- 产出: `.opencode/agents/perf-engineer.md`
- 下一步: 等待 team-lead 调度

## [2026-10-25] T12实现前性能基线
- 来源：team-lead委派T12-03，用户要求特色、实用、Agent友好与高效。
- 完成：完整启动；阅读旧报告和当前源码；新增有界运行器，19测量单元各预热2/正式7，133次正式运行输出全部校验一致。
- 产出：benchmarks/t12/run.py、results/before.json、before-small.json、docs/reports/T12-performance.md；两命令exit0。旧exe原hash保全Temp/t12-perf/lfz-before.exe。
- 建议：优先StructObj字段索引（实际Vec扫描）；保留原始重复键first-match及排序/别名；应用避免循环push。未修改runtime或他人产物，未Git。
- 下一步：新release exe后复测旧负载并增加JSON/CLI入口开销。
- 阻塞：无；第二阶段依赖新实现，按调度释放slot。
## [2026-10-25] T12首版性能回归及JSON/CLI测量
- 先冻结阈值及一次交错复查规则，再测hashfb8e…f47d。19旧单元16过线，numeric100k/500k、string2k初测及复查仍超线，立即报team-lead修复；不调整门槛。
- struct8k降时约71.8%，small4/16/64均过线；只限定样本。6JSON+3CLI单元输出独立oracle一致；JSON三档字节/总耗时/启动占比已记录。
- 新增new_features.py/recheck.py、after/after-small/recheck/comparison/new-features JSON，报告及自身记录更新；未改源码，未Git。
- 下一步：修复最终hash后重测，当前释放slot。

## [2026-10-25] T12最终同执行路径复测仍超阈
- 来源：team-lead要求控制旧Temp/newdist路径条件；原阈不变。
- 完成：final_compare.py将旧新exe先copy同一Temp/T12/perf-subject.exe，启动前核hash；每换bytes预热2+7，case组次序交替。19项16过线3超阈，所有输出正确。
- 数值500k同路径旧104.259/新189.340ms，dist探针171.409ms；路径差异机制未知，不能据另一诊断99ms宣布已解决。struct8k约78.1%局部降时，小对象均过线。
- 产出：final-same-path.json、final-new-features.json、报告及自身记录；全部旧失败证据保留。JSON/入口63次正式输出全一致。
- 下一步：效率门槛未通过，继续诊断超阈；已通知verifier可开始功能验证。释放slot。

## [2026-10-25] 修复版复测脚本准备
- 来源：team-lead预告while空Env候选，等待最终hash。
- 完成：final_compare显式new-binary/expected-new-sha参数、old固定hash、每轮核hash；--help exit0；fb8exe保全副本。原19阈及30%目标未动。
- 未执行候选计时；等正式hash和构建/测试结束后再复测全组，当前释放slot。

## [2026-10-25] 7641修复版全组性能测量及环境限制
- 来源：team-lead确认最终1.4.0/821760B/hash7641，构建测试已停。
- 初始Python-c readiness cwd异常但shell继续测量，fixed-same-path为诊断批，诚实披露。随后新增readiness.py，CPU9.55%/旧noop13.232ms达原线；再择窗全组测量一次。
- fixed-idle-same-path19项13过线，后半6项新旧均超原线，按冻结规则环境不确定不放行；numeric500k105.462→91.254ms、struct8k94.963→22.746ms，dist500k107.484ms。
- 新功能组最终每启动核hash，fixed-hashchecked-new-features九单元63正式输出oracle一致。报告开头最新结论重写，历史失败/路径控制/诊断均保留；原19门槛/30%struct目标未动。
- 下一步：需另安排稳定静默窗口处理环境不确定，当前不自行无限重测；释放slot。未改源码、未Git。

## [2026-10-25] 预指定CPU0控制实验仍未通过整体门槛
- 新增进程内affinity控制，父先mask=1、子创建继承、finally恢复；两harness原16777215/active1/restored16777215均验证。官方说明已核查并引用。测前方法落盘后执行，原阈不变。
- fixed-cpu0-same-path19项12过线、5环境不确定、2略超原线（array50k、小对象4键），未隐藏；JSON/入口9项输出全正确。数值500k94.414ms、struct8k23.704ms。
- 原始JSON全保留，报告最新结论更新；大量old也超原参考，依预定规则不再自选窗口，待root静默冷却安排。未源码/Git，当前收工释放slot。

## [2026-10-25] 固定静默90秒/换bytes等待5秒控制窗口完成
- team-lead安排其它agent静默，测前方法落盘，不挑最快读数；未绑核、90秒静默，每次subject换bytes固定5秒，再2+7。readiness CPU11.69%/旧noop15.098达标；全19组原序完成一次。
- quiet1四份JSON完整保留，GetProcessTimes user/kernel及wall逐轮记录，所有273旧/新/探针+63新功能正式输出正确。
- 13项在原线内、5环境不确定、1startup超线，整体仍未通过。numeric500k107.422→90.379、struct8k92.184→21.535约76.6%降时；阈不放宽，不用局部收益抵消失败。
- 报告开头及完整CPU/wall表已更新，历史失败保留；不自行追加窗口。未源码/Git，收工释放slot。

## [2026-10-25] opt2正式主产物唯一全组测量完成
- 来源：team-lead确认release-manager批准opt-level2产物47ED/759808B，构建测试已停止。测前方法落盘，默认未绑核每copy等5秒，原2+7/顺序/阈不变，只此正式hash一次完整全组。
- opt2-same-path273正式输出正确、opt2-new-features63正式输出oracle一致、逐轮hash核验；opt2-assessment绑定SHA。
- 15项过原线、3环境不确定、1map8k超线，R304 OPEN，不能宣布整体效率通过。numeric500k90.804ms、struct8k21.198ms约77.3%降时；string4kpaired慢28.8%也明确披露。
- 最终报告顶端更新，原FB8/7641/路径/CPU0/quiet1失败证据全保留；不虚构LLVM微因，不以包体/主循环抵消未过项。当前作为可审阅候选checkpoint，测量结束已通知root，其它任务可恢复。未源码/Git，收工释放slot。
