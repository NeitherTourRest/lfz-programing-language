# tooling-dev — 工作状态
> 最后更新：2026-10-30 13:30 by tooling-dev

## 当前状态
T17 CLI及最终完整release作者验收已收工冻结。停止源码修改、编译与测试，交root公开121/独立验收；未覆盖dist或Git。

## 最终证据
- 两flag严格单次scanner、CODE/argv保护、非执行拒绝、精确等号族usage；旧Command和unlimited流水线保持。
- 新run_limited Result<CaseEval,String>，前端/静态图成功后checked_add建EvalLimits；旧eval_case/runner默认unlimited，第13类runner注册原12保留。
- 合runtime四unit后最终完整no-run编译；十个本次新release harness提权624/624=471lib+50bin+28cli+12T12+12T14+7B+5C+9T17+15runner+15unit，0failed/ignored。新增实际跨线程取消/7HOF/diamond11对10/default faultsource全通过。
- 最终target/release/lfz.exe943104B，SHA1946AB9CA5162D68F83A9D098424BD0BAFDF6FEADC6EA90527E0DB9E1DB12C45，build exit0；evaluator仍交接a673cca7cf009ba19ac9897737aa6f75db2dc24941ef31112fe6728c5b2e551d。
- docs/tooling/evidence/T17/final-manifest.json绑定当前源/Cargo/newCLI测试及final2原始日志；F487/620/first默认6/8及精确快照保留不覆盖。docs/reports/T17-tooling.md末节为最终结果，runner-contract§15已同步。

## 边界/下一步
作者624不代替独立验收或R304性能门禁。root接续公开121、独立、交付复制。只改CLI/授权runner/新增测试与本人文档，未改runtime/core/spec/旧tests/profile/dist/Git。Temp项目内、release复用，无Ctrl-C依赖或硬中断native承诺。
