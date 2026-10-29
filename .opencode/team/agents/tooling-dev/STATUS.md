# tooling-dev — 工作状态
> 最后更新：2026-10-27 19:59 by tooling-dev

## 当前状态
T14非执行check CLI实现与作者验证完成，等待team-lead独立验收/交付复制。现在停止源码修改、构建和测试；未覆盖dist或Git。

## 产出与证据
- src/cli.rs新增CheckInput/parse_check_input/check_input/check_loaded_on_stack：file/-e/stdin仅load/lex/parse/drop，全部位于既有大栈，不求值、不建Interp、不重定向LFZ输出。
- 合法check JSON严格ok/mode/check/executed/false及既有错误字段；普通成功空双流，usage仍stderr文本且stdout空。保护CODE；CODE外--/脚本参数/check --help拒绝。
- tests/cli_t14.rs新增12组进程回归；旧测试未改。旧版红测0/12；候选首次11/12，按既有run同源line2,col8订正新位置期望后12/12。中间失败日志保留。
- cargo test --release 542通过（412+48+28+12+12+15+15），0failed/ignored；cargo build --release exit0。T13新增8回归包含在412lib内。
- 最终target/release/lfz.exe774656B，SHA83F56A194E216862C45377F4C3971B085744A7BA2363163E1EDAC6AF5E85AC79。Cargo1.4.0/opt2/profile未改，dist交release。
- docs/tooling/runner-contract.md §11、main说明已同步；原始日志和manifest在docs/tooling/evidence/T14；作者报告docs/reports/T14-tooling.md。

## 阻塞/限制
无实现阻塞。语法通过不保证运行成功，不做名称/类型/arity/IO路径静态验证，不是通用沙箱；T13性能R304保持OPEN，不据功能全绿宣布正式发布。

## 下一步
team-lead衔接独立非执行验证/黑盒、文档、交付复制与Git；若发现缺陷再由team-lead唤醒修复。原T13测窗/快照保留不混成新产物。

## 关键经验
检查模式须独立于run；Program不能带回主线程析构。CODE紧随-e无条件保护，usage错误JSON例外须明确。位置断言按既有前端契约核实，不能为通过新测试改旧parser。所有临时件在项目Temp，cargo只release。
