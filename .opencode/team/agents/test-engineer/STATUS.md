# test-engineer — 工作状态
> 更新：2026-11-18

## 当前状态
T27 三个公开 findIndex LFZ 用例与一个预算夹具已写。冻结 dist 7a144213 的静态检查 4/4。旧运行缺 findIndex 返回 NameError、exit 2，证据 Temp/T27-public/baseline-red.json。

## 产出
`tests/lfz/test_find_index.lfz`、`test_find_index_shadow.lfz`、`test_find_index_shadow_noncallable.lfz` 由标准 runner 自动发现。`tests/fixtures/t27_find_index_budget.lfz` 用 CLI `--max-steps 60` 验证回调终止穿过 attempt。现有 cases.json 与旧测试原样保留。

## 下一步
等待最终候选指纹后验证三公开用例、全 runner 与预算。R304 OPEN。
