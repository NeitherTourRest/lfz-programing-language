# T12 应用验证（2026-10-25）

本报告记录app-dev自检，不替代verifier独立验收。LFZ 1.4.0 release，821248 bytes，解释器SHA256 `fb8e5653827de1e1c3f73b81d37f21314ad41adfdc4d99f37cb8ca5f6491f47d`。

## 产出

- `app/jsonflow.lfz`（73行）：配置驱动的JSON记录筛选与字段投影，argv/文件IO/JSON/管道/闭包/struct形成完整工作流。SHA256 `3663a59f9563a7c7ab339c5b7389542eae107e94451aa6def86505604dbf89be`。
- `app/fixtures/t12/{config,input,expected}.json`：中文/emoji/精确大整数示例。
- `app/sortviz.lfz`（362行，311有效行）：新增参数配置，genData改range/map；保留5种算法与150自测。有效行剔除空行、纯`//`注释与`#42`前导，仍超过课程200行。SHA256 `f25f3ecf97cf0efd221adc55fa643ee43a8055a93e690760218c005093f95662`。
- `app/verify_jsonflow.py`、`app/verify_sortviz.py`：独立Python标准库oracle及应用级自检；临时文件仅项目Temp/T12。
- `app/README.md`、`app/DEV_RECORD.md`：当前契约、命令、限制与真实迭代记录。

## 实跑证据

```powershell
D:/app/ANACONDA/python.exe -B app/verify_jsonflow.py --lfz dist/lfz.exe --output docs/reports/T12-app-evidence/jsonflow.json
D:/app/ANACONDA/python.exe -B app/verify_sortviz.py --lfz dist/lfz.exe --before-exe Temp/t12-perf/lfz-before.exe --output docs/reports/T12-app-evidence/sortviz.json
```

两命令exit0，分别37/37与11/11通过；证据包含命令结果、完整stderr/关键stdout及hash。报告路径已存在时运行器拒绝覆盖，复跑请使用新的项目内输出路径。

jsonflow覆盖正常夹具、空集、无匹配、全选、Unicode键、bool/int不等、int/float数值比较、null、嵌套投影、机器通道；未知/缺少/坏类型配置、空/重复投影、坏where、非数组输入、非object记录、缺字段（含未匹配记录）、坏JSON、重复JSON键、缺失路径、空/过多/不足参数、输出与输入/配置字面相同。验证失败均exit2、没有业务stdout，既有输出保留，配置和输入字节不变。

sortviz以`git show v1.3.0:app/sortviz.lfz`只读恢复旧源码，搭配SHA149b184e442ffbf250590431fa0aeffab401eac526faf60b4b435f620725d5ab旧exe，对比新release+新源码，默认stdout逐字节一致；参数小4/大30/种子7/不显示过程快照连续两次输出一致，150次排序自测失败0。机器stdout恰状态JSON，stderr与普通stdout逐字节一致。8项无值/越界/非整数/未知/重复参数均受控exit2且无stdout。

首版jsonflow在debug已先跑通过，随后release完成两轮自检（Temp初次证据、正式归档证据）；没有为失败测试修改产品逻辑。Python启动输出“Failed to find real location...”环境提示，但两命令实际exit0，不能将其误报为产品错误。

## 保证与限制

- jsonflow在验证全部配置与记录并编码成功后才写文件；只保证验证/编码失败不触碰已有输出，不承诺IO失败时原子写入。
- 只拒输入/配置与输出的字面相同路径；没有路径规范化/符号链接/硬链接识别，调用者必须确保独立输出文件，README明确限制。
- 数据整批加载驻留内存，不是流式JSON工具。filter/map避免循环push构建；投影平均O(RK)字段访问，JSON及键排序另计。
- sortviz无参数默认保持历史可见行为；`--no-frames`保留原始数据的一次条形图。旧归并教学实现的push保留，没有宣称本轮改进所有算法复杂度。
- 未改src/spec/既有正式tests/冻结v2输入或成绩，未Git提交。
