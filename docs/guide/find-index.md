# 用 findIndex 找到首个匹配项的位置

`findIndex(predicate, array)` 返回从 0 开始的索引。数组为空或找不到时返回 `nil`。用 `index == nil` 判断未找到，用 `index == 0` 判断命中首项。LFZ 的条件要求布尔值，`if index` 不是有效写法。

```lfz
#42
let numbers = [10, 20, 30]
let first = findIndex((x) => x >= 10, numbers)
assert(first == 0)
let missing = numbers |> findIndex((x) => x == 99)
assert(missing == nil)
```

需要定位或更新第一个匹配项时用 `findIndex`。需要收集所有匹配元素时用 `filter`，它返回数组，并对每个快照元素调用谓词。两者都支持数组放在最后的直接调用与管道写法。`findIndex` 从左到右检查，首次返回 `true` 就结束，后面的回调及其副作用都不会发生。

## 回调中修改数组

回调接收一个元素。调用开始时，LFZ 浅拷贝数组的元素序列。回调替换或增删原数组元素时，本次访问的顺序和索引仍以快照为准。快照里的嵌套数组和结构体仍与原来的对象共享引用，因此修改嵌套字段可以被后续回调看到。回调修改原数组后，返回的索引未必还能定位原来的匹配对象。实际可运行例子见 [find-index.lfz](examples/find-index.lfz)。

## 错误与执行限额

谓词必须返回 `true` 或 `false`。例如，`findIndex((x) => x, [1])` 会在内置调用处报 `TypeError`。空数组仍检查谓词是否为函数以及第二个参数是否为数组，但不调用谓词，因此不会检查回调形参个数。

回调自身抛出的错误保留原来源。符合 `attempt` 捕获规则的错误可用 `attempt(() => findIndex(predicate, items))` 得到错误记录。已完成的副作用仍然生效。回调沿用当前执行的步数、取消和截止限制，`attempt` 无法捕获这些资源终止。浅拷贝本身不另外消耗 LFZ 求值步数。[错误恢复](error-recovery.md) · [执行限额](execution-limits.md)

在仓库根目录运行下列示例。第一个先检查再执行，成功时打印 `findIndex examples ok`。第二个的回调持续循环，达到预算后退出 2 并报告 `ExecutionLimitError`。

```powershell
lfz check docs/guide/examples/find-index.lfz
if ($LASTEXITCODE -eq 0) { lfz run docs/guide/examples/find-index.lfz }
lfz --json --max-steps 200 run docs/guide/examples/find-index-budget.lfz
```

完整调用规则见 [接口契约 §28](../spec/interface-contract.md#28-t27-findindex2026-11-18-冻结)。
