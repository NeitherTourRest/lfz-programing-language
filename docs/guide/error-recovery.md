# 错误恢复：逐条处理JSON

保存下列程序为job.lfz（文件首行必须#42）：

```lfz
#42
let lines = ["{\"n\":1}", "broken", "null", "{\"n\":2}"]
var values = []
var failures = []
for text in lines {
    let r = attempt(() => jsonParse(text))
    if r.ok {
        values = push(r.value, values)
    } else {
        failures = push({kind: r.error["class"], message: r.error.message}, failures)
    }
}
assert(len(values) == 3)
assert(values[1] == nil)
assert(len(failures) == 1)
print(jsonStringify({accepted: len(values), rejected: len(failures), values: values}))
```

先 `./target/release/lfz.exe check job.lfz`，成功后 `./target/release/lfz.exe run job.lfz`。输出：

```json
{"accepted":3,"rejected":1,"values":[{"n":1},null,{"n":2}]}
```

结果中的ok区分成功nil与失败nil。失败JSON记录不会中断后续输入。这里输入是一个内存数组，不是流式文件读取；读取整文件后split也不会自动变成流式处理。

错误数据和捕获白名单见[错误指南](errors.md)，字段与调用边界见[参考](reference.md)。恢复不回滚文件/容器等副作用。通用内置不是一等函数值，因此必须使用零参数闭包包装操作。R304仍开放，此教程不宣称性能提升或正式发布。
