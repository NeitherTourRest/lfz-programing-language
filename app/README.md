# LFZ 实用应用

当前应用使用 LFZ 1.4.0。请从仓库根目录使用本项目 `dist/lfz.exe`，避免误用尚未更新的全局安装。开发记录见 `app/DEV_RECORD.md`，本轮实跑报告见 `docs/reports/T12-app.md`。

## jsonflow：配置驱动的 JSON 数据处理

`app/jsonflow.lfz` 是单文件数据脚本：读取配置与 JSON 记录数组，按字段精确相等筛选，再投影指定字段。它将 `argv`、UTF-8 文件 IO、JSON、管道、闭包与 struct 串成实际工作流，不需要生成一份新源码才能改参数。

```powershell
New-Item -ItemType Directory -Force Temp | Out-Null
./dist/lfz.exe run app/jsonflow.lfz -- app/fixtures/t12/config.json app/fixtures/t12/input.json Temp/result.json
```

夹具配置为 `{"where":{"field":"status","equals":"done"},"select":["id","title"]}`，3条记录中保留2条；输出文件与 `app/fixtures/t12/expected.json` 相同，含中文、emoji与精确大整数。正常stdout是单行业务摘要，例如 `{"input":3,"output":2,"path":"Temp/result.json"}`。

配置约定：

- 顶层必须恰含 `where` 和 `select`，拒绝未知字段。
- `where` 为 `null` 时全选；否则必须恰含非空字符串 `field` 与 JSON 标量 `equals`。使用 LFZ 精确相等语义，bool不等于int，int与float按精确数值比较；不做大小写折叠或隐式字符串转换。
- `select` 为至少一项、无重复的非空字符串数组。记录可以含额外字段，但每条记录必须为object且具备全部所选字段和筛选字段；未匹配记录也验证，不会隐藏坏数据。
- 输入是记录数组，空数组合法。输出保留输入记录顺序；输出对象键按 UTF-8 字节序排列，文件为紧凑 JSON 加一个LF换行。
- 参数必须恰为 `CONFIG INPUT OUTPUT` 三条非空路径，路径相对当前工作目录，不相对脚本目录。输出父目录必须已存在。

程序先验证全部配置、全部记录并完成输出编码，再调用 `writeFile`。验证或编码失败时，已有输出保留；不把这个保证扩大为操作系统写入失败时的原子写保证。输出路径与输入/配置路径字面相同时拒绝。LFZ当前没有路径规范化、真实路径或硬链接识别API，**不同拼写、大小写、符号链接和硬链接别名不会被识别**；调用者必须确保OUTPUT指向独立文件。所有数据整批驻留内存，不是流式处理器。

机器状态模式：

```powershell
./dist/lfz.exe --json run app/jsonflow.lfz -- app/fixtures/t12/config.json app/fixtures/t12/input.json Temp/result.json
```

成功时stdout仅为 `{"ok":true}`，业务摘要进入stderr。错误退出2；正常模式诊断进入stderr，机器模式错误对象进入stdout。业务JSON结果始终写入指定文件。

自检（Python仅使用标准库；37项含独立oracle、失败保留输出、通道隔离与输入不变性）：

```powershell
D:/app/ANACONDA/python.exe -B app/verify_jsonflow.py --lfz dist/lfz.exe --output Temp/jsonflow-check.json
```

报告路径必须尚不存在，所有临时材料均在项目 `Temp/T12/`。筛选/投影采用filter/map与字段索引；投影对R条记录、K个所选字段需要平均O(RK)字段访问，JSON编解码及键排序另计，不声称对任意嵌套数据只需常数开销。

## sortviz：排序算法可视化

`app/sortviz.lfz` 保留课程≥200有效行的完整应用：冒泡、选择、插入、归并、快速排序；逐帧字符条形图、结构体计数器、确定性随机数、正确性断言与150次属性自测。默认配置为小规模8、大规模600、种子42、帧宽6、显示逐帧；无参数的stdout与v1.3.0旧应用在旧解释器中的输出逐字节相同。

```powershell
./dist/lfz.exe run app/sortviz.lfz
./dist/lfz.exe run app/sortviz.lfz -- --small 4 --big 30 --seed 7 --no-frames
./dist/lfz.exe --json run app/sortviz.lfz -- --small 4 --big 30 --seed 7 --no-frames
```

- `--small N`：1至100；`--big N`：1至2000；`--seed N`：i64整数。缺值、未知选项、重复选项和越界值拒绝，退出2。
- `--no-frames`：关闭排序过程快照，保留一次原始数据条形图及统计/自测。默认运行不依赖stdin。
- `genData`用 `range |> map` 一次构建数组，保持seed与随机调用顺序，避免循环push复制。手写归并仍保留教学实现的返回新数组push，不能将它宣称为运行时最优归并；大规模更适合内置sort。
- 机器模式stdout仅状态JSON，应用显示全部进入stderr。

结构：配置/参数解析 → Stats计数结构体 → 数据生成 → 字符画函数 → 5个算法 → 校验/selfTest → 3阶段驱动。5项语言特色分别体现在管道数据处理、struct方法计数、富插值、assert/check与 `;;` 可见变量输出。LFZ尚无模块导入，两个应用均为清晰分节的单文件脚本。

默认兼容与参数自检：

```powershell
D:/app/ANACONDA/python.exe -B app/verify_sortviz.py --lfz dist/lfz.exe --before-exe Temp/t12-perf/lfz-before.exe --output Temp/sortviz-check.json
```

旧exe必须为v1.3.0时的1.0.1解释器（SHA `149b184e442ffbf250590431fa0aeffab401eac526faf60b4b435f620725d5ab`），可以从旧首轮原始输入归档恢复；脚本使用只读 `git show v1.3.0:app/sortviz.lfz` 恢复旧应用到项目Temp。自检11项，覆盖默认字节一致、固定参数可复现、150次排序自测、机器通道和8项坏参数。
