# tooling-dev — 工作状态
> 最后更新：2026-10-30 02:52 by tooling-dev

## 当前状态
T16-B CLI静态模块图适配及作者验证完成。源码/产物已冻结，停止所有构建和测试，交team-lead独立验证和C接续；未Git/覆盖dist。

## 产出与证据
- src/cli.rs check三来源counted加载；无模块根不构图/不套图预算；有模块根只parse一次后移交ParsedEntry，图构造及析构均在大栈。
- GraphFailure真实source_id取display_path/可选Loaded诊断，依赖错误不绘入口文本；旧JSON/usage保持。
- tests/cli_t16b.rs七组过程测试及旧binary红见证；新增文件无旧tests改动。默认2/7因canonicalize沙箱权限；同harness提权7/7，无产品改动。
- 默认全cargo lib439通过/8权限失败中止，完整保留；同八release harness提权584/584=447+48+28+12+12+7+15+15，0failed/ignored；build exit0。
- 最终target/release/lfz.exe859136B，SHA8EA375DCD08098EF61D040A57A7B1AD28A28C2FDC25A5CA4921A75B728944480。未动dist/profile。
- docs/reports/T16-B-tooling.md，docs/tooling/evidence/T16-B/manifest.json+原始日志；runner-contract §13已同步。

## 阻塞与下一步
无CLI实现阻塞。B仅静态图，run模块仍由runtime在所有副作用前拒绝；C初始化另行冻结实现。R304不因功能全绿关闭。后续独立验收由team-lead协调，缺陷再唤醒。

## 关键经验
来源错误必须携带对应Loaded，不能用root文本画依赖下划线；raw_bytes来自同一次读，legacy大root不能误套图限额。权限失败保原始记录并使用同产物许可验证，不改canonical身份策略凑绿。所有临时项目Temp，仅release复用target。
