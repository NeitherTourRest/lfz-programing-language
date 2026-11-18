# T27 为 Agent 查找第一个匹配项

`findIndex(predicate, array)` 返回首个匹配元素的索引。索引从 `0` 开始，空数组或未命中返回 `nil`。用 `index != nil` 判断命中，然后再取下标。数组是末参，因此 `[1, 2, 3] |> findIndex((x) => x == 2)` 返回 `1`。

谓词接收当前元素这一个实参，返回严格的 `bool`。`true` 立即结束查找，`false` 继续，整数和 `nil` 都会触发 `TypeError`。空数组仍校验函数和数组参数类型。由于没有回调调用，空数组不会触发回调形参个数校验。

访问从数组的浅快照左向右进行。回调内替换原数组的槽位不会改写本次访问序列，返回的索引属于快照。嵌套容器保持共享引用，回调对共享字段的修改可由后续元素观察到。需要用索引读取原数组时，应避免在查找期间改变槽位。

回调错误保留原来源并向外传播。用 `attempt(() => findIndex(predicate, xs))` 捕获允许恢复的错误，按 `r.ok` 区分调用成功与失败。未命中属于 `{ok: true, value: nil, error: nil}`，错误类别读 `r.error["class"]`。已经发生的副作用保留。步数上限、取消和截止时间沿当前求值会话传播，`attempt` 无法捕获 `ExecutionLimitError`。回调执行正常计步，快照和内置遍历不额外计 LFZ 步。

同名用户绑定优先。需要把查找传为函数时，可定义自己的函数并在函数体中调用 `findIndex`。T27 共有 77 个内置名称，其中 8 个为高阶内置。完整规则见 `docs/spec/interface-contract.md` §28 和 `docs/reports/T27-contract.md`。

在项目根目录先检查示例，再运行。正常示例完成断言后输出 `findIndex witnesses passed`。

```powershell
./target/release/lfz.exe check docs/guide/ai/examples/05_find_index.lfz
./target/release/lfz.exe run --max-steps 10000 docs/guide/ai/examples/05_find_index.lfz
```

对预算示例先运行 `check`，再用以下命令执行。预期退出码为 `2`，stdout 的 JSON 对象中 `error` 为 `ExecutionLimitError`。

```powershell
./target/release/lfz.exe check docs/guide/ai/examples/06_find_index_budget.lfz
./target/release/lfz.exe run --json --max-steps 100 docs/guide/ai/examples/06_find_index_budget.lfz
```
