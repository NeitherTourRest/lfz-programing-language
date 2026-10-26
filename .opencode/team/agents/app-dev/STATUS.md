# app-dev — 工作状态
> 最后更新: 2026-10-25 01:04 by app-dev

## 当前状态
T12应用交付完成，等待verifier独立验收；P8原sortviz保留，新增jsonflow实用数据脚本。未Git。

## 产出与证据
- app/jsonflow.lfz（73行）：配置驱动记录筛选/投影，argv三路径，严格schema/全记录验证后编码并输出。
- app/fixtures/t12三JSON夹具、app/verify_jsonflow.py；release 37/37独立oracle/边界/输出保留/输入不变/机器通道通过。
- app/sortviz.lfz（362行/311有效行）：--small/--big/--seed/--no-frames，参数块局部不改变默认dump，genData range/map避免循环push复制。
- app/verify_sortviz.py：11/11通过，旧v1.3.0源码+旧exe与新release默认stdout逐字节一致；固定参数重复输出相同，150排序自测0失败，坏参数8项受控错误。
- app README/DEV_RECORD已同步；docs/reports/T12-app.md与T12-app-evidence/{jsonflow,sortviz}.json为正式证据。
- 全部实跑使用项目dist/lfz.exe 1.4.0；未改解释器/规范/既有黑盒/冻结v2，临时文件仅Temp/T12。

## 进行中 / 阻塞
无；自检不代替独立验收。

## 下一步
team-lead派verifier验收并由release-manager统一提交；缺陷再唤醒应用角色。

## 关键经验
- 旧“无CLI/文件IO”限制已解除；当前argv与JSON依据spec§16，旧skill描述由ai-dx角色同步。
- 只argv新增一等内置，jsonParse/jsonStringify以直接call/管道调用，若作为map回调需lambda包裹。
- 提前验证/编码可以保留错误输入时已有输出，但writeFile不是原子替换；无法识别别名路径必须公开限制。
- 维护默认dump字节兼容时，参数解析新增名字应局限在块内，避免顶层可见变量变化。
