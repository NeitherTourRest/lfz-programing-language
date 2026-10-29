# T16-B 公开进程黑盒

日期：2026-10-30。责任人：test-engineer。依据三spec §20至§20.10。当前候选target/release/lfz.exe 859136B，SHA256 `8ea375dcd08098ef61d040a57a7b1ad28a28c2fdc25a5ca4921a75b728944480`。

新增 `tests/t16b_cli_check.py`，不导入产品实现，以公开CLI与独立预期检验。所有夹具由脚本在项目Temp/T16-B-test生成，不把模块库加入LFZ runner默认发现目录。本轮不修改任何原LFZ/Rust测试或cases.json；官方runner基线仍178项，本角色本轮不重复执行官方runner或把38进程叠加成官方用例。

## 真实红测和最终实跑

旧dist65a29/783872B执行新增模块check烟雾，预期exit0实际SyntaxError/exit2，0/1：`preimplementation-red.json`保留，说明旧版本缺新语法，不是旧行为回归。

按调度授权，用正常权限执行同一公开脚本，避免Windows受限环境canonicalize项目身份元数据的AccessDenied限制；不改产品绕过canonical路径。命令：

```powershell
D:/app/ANACONDA/python.exe -B tests/t16b_cli_check.py --binary target/release/lfz.exe --expected-sha 8ea375dcd08098ef61d040a57a7b1ad28a28c2fdc25a5ca4921a75b728944480 --output docs/reports/T16-B-blackbox-evidence/final-first.json
```

首次新候选完整实跑 **38/38 PASS，exit0**，无产品修复或放宽预期。每进程前及脚本结束核binary hash；证据manifest当前源码逐一与tooling冻结构建manifest比较一致。不将这个构建指纹对照虚称本脚本记录了源码测前/测后快照。Python AST检查通过；一次辅助AST调用漏encoding导致Windows GBK解码失败已改成UTF8读取，非产品问题。

## 覆盖与限制

- 三来源/普通与JSON、三文件/菱形与./..身份别名、前向导出。
- 顶层/函数/lambda/if/while声明限制、import次序、空/NUL/绝对/非lfz/插值路径。
- import重复与普通绑定冲突、重复/缺失/builtin导出，精确消息和行列；旧export标识符表达式与赋值继续运行。
- 子模块parse/UTF8/前导、缺文件、循环：错误指真实依赖来源（Windows canonical前缀可存在），不是入口文本；JSON字段顺序/单行、mode=check/executed=false保持。
- 依赖含print/eprint/dump/input、文件写入/追加、无限循环均不执行；已有二进制哨兵不变，新文件不创建。B run即使前面有写入/输出也先报“模块执行尚未启用”，仍旧执行JSON协议而非check协议。
- 无模块大于2MiB root仍成功；进入模块图2MiB正好成功、多1字节拒绝；链含root128成功、129拒绝。不制造OOM或abort。

节点1024/1025、全图16MiB/+1及依赖有界尾哨兵由core12项unit覆盖，本38进程不虚计这些范围；symlink权限/身份情况交独立verifier适用性验证。8秒超时只识别意外执行，不是性能指标。源代码仍整文件读取legacy root，不称完整资源沙箱。B只check模块图，模块初始化/执行尚未实现；R304仍OPEN，无性能通过或正式发布声明。

证据目录 `T16-B-blackbox-evidence/` 为原字节属性，含旧红测、完整38行原始结果与manifest。全部Temp在项目内，未编译、改产品/dist/spec、旧用例或执行Git。
