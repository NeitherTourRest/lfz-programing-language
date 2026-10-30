# tooling-dev — 工作状态
> 最后更新：2026-10-30 08:03 by tooling-dev

## 当前状态
T16-C CLI集成与作者验证已收工，源码/产物冻结，不再改源/编译/测试。等待team-lead独立验收与交付复制；未覆盖dist/未Git。

## 证据
- run三入口counted根一次parse，有模块build→graph API同大栈执行/drop，无模块legacy；check不初始化。
- GraphFailure真实前端path/Loaded，runtime error_source及frame_source原index渲染；折叠尾index不从0重排，JSON全量。
- 经授权迁移B run过渡断言，其余B及旧tests保留；新C五组过程5/5。
- HostFailure经授权扩runner fail-closed，真实合成红0/1→绿1/1，含Located递归。默认canonicalize权限失败保留，最终同九release harness提权599/599=456+49+28+12+12+7+5+15+15，0failed/ignored；build exit0。
- 最终902656B/hashCDEDC9B980AA207B28BF895EF95CE76FDE18E47EF758C19C187B3F9C2DA08006，修前671919/598证据保留。
- docs/reports/T16-C-tooling.md、docs/tooling/evidence/T16-C/manifest.json+原始记录、runner-contract§14。

## 变更/下一步
src/cli.rs、获授权src/test_runner.rs、获授权B末条断言及新tests/cli_t16c.rs；不动core/runtime/spec/profile/dist/Git。独立验收/公开29由team-lead接续；R304仍OPEN。临时项目Temp，release复用target。
