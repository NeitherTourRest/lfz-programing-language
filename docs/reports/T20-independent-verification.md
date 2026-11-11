# T20 独立验收

Windows 候选 1100800 bytes，SHA256 `a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c`。独立复跑 13 个现有 release harness，共 705/705。公开 REPL 37/37。现源码哈希与构建者 manifest 一致，旧回归运行前后源码及候选哈希一致。正常 OS 许可覆盖已知沙箱句柄与本地文件 canonicalize 限制。原 harness 内部项目测试 fixture 构建与产品构建分开。

旧 CLI 与应用套件全部复验通过。attempt 13、check 117、CLI 91（含官方178）、jsonflow 37、sortviz 11、jsonbatch 38、独立 JSON 140、T17 134 与应用40、模块29与应用33、子进程45与应用7、HTTP49与应用7、逐行47与应用14。独立旧回归首轮应用40因我方 Temp 目录多一层导致 fixture 相对导入错误。逐行应用14因我方把 `--binary` 写成 `--lfz` 未执行。两项首失败原样保留，纠正独立驱动后分别40/40与14/14，产品和预期未修改。命令与原始结果见 evidence/regression，复验参数为相同 SHA 与 `app40-rerun.py --lfz`、`stream14-final.py --binary`。

独立 `host.rs` 用现有 release rlib 与匹配的 fat LTO 链接，实测 host 绝对截止时间早于相对5000ms、到期后跨 cell/reset 保留，并实际执行1000小cell后核对编号及 reset 清定义。第一次普通链接 LNK1107 保留，成功重试的退出为0。公开测试实际提交超 1MiB pending。Rust 1MiB 是实际字符串输入，4096/16MiB 通过计数注入，65536 函数门限由生产配置值与降低门限的单元测试共同覆盖。host token 跨 reset、双诊断 poison、旧来源快照由同批单元实际通过。没有执行任意长会话内存或 GC 测量。

作者 ConPTY 正常会话及错误后 reset 退出码有真实驱动记录，中文渲染仍待验。人指南16、AI指南8、JSONdesk应用6的作者原始结果同候选哈希，11份当前文档和应用 manifest 已独立只读核对。保留人指南原字节副本 `human-guide-original.json`。审计不重复计作本人执行次数。JSONdesk 首失败已保留，作者最终报告含合法多行部分副作用见证和 EOF。

v3 控制签收 `v3-controls-receipt.json` 绑定38件草案和同一候选。四参考功能通过，八负控制预期拒绝，预算判定9/9。C01 两类后段读错均核实已刷32条，预读反例另有实际源码审计 FAIL。C04 首次因 Python 模块同名覆盖无法启动 fixture，修后参考功能通过且服务关闭。独立控制真实关闭服务后才注入 false，所有产品行成功但总判 FAIL。首注入未命中及原评分器失败已保留。学生真实 trace 的预算与范围审计另交后续签收。

T20 功能独立通过，中文终端渲染及学生协议按各自证据记账。R304 OPEN。
