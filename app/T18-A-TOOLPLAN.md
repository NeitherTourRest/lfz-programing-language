# T18-A toolplan：显式能力的本地工具编排

日期2026-10-30，app-dev。仅Windows T18-A，不进入HTTP或其它后继迭代。依据三spec §23；LFZ skill的既有argv/JSON/attempt工作流结合新exec契约使用。旧应用与oracle未修改。

## 实用接口

`app/toolplan.lfz`需要两个脚本参数：CONFIG.json OUTPUT.json。配置是任务array，每项含name、argv、timeoutMs、maxOutputBytes。argv是字符串array，第一项必须是显式绝对.exe路径，不查PATH。例子中的路径要替换为可信本地工具：

```json
[{"name":"example","argv":["D:/project/Temp/fixture.exe","echo","","a b","中文🙂"],"timeoutMs":5000,"maxOutputBytes":4096}]
```

```powershell
./dist/lfz.exe check app/toolplan.lfz
./dist/lfz.exe run --allow-process app/toolplan.lfz -- Temp/T18-app/plan.json Temp/T18-app/report.json
```

命令从仓库根运行，先建输出父目录。只有实际需要exec的任务才会检查能力；非空有效工具计划未带--allow-process时，第一个禁用错误会被应用转为外层AssertionError“toolplan: 需要 --allow-process”，退出2、报告文件不写。空任务列表不spawn，返回空报告。

所有任务结构先检查再执行。每项以attempt捕获exec的局部ValueError/IOError，生成{name,ok,value,error}行。非零exit是成功调用值，包含stdout/stderr/exitCode/signal，调用者不能把ok=true误当exitCode=0。局部timeout/超量/非法UTF8是失败行，后续任务继续。全局deadline/cancel是ExecutionLimitError，不能被attempt吞掉，整个程序退出2；不返回伪成功报告。

OUTPUT含total/succeeded/failed/rows，一次完成后确定性JSON编码并写文件，stdout只打印一行统计。原始child输出只作为value数据，不直接向终端透传。error记录沿T15五字段协议，class需用["class"]读取。写文件前验证/编码不能提供事务保证：已运行的工具、副作用和已写文件不会回滚，writeFile本身也非原子。

能力是会话显式开关，不是可信exe白名单或OS权限沙箱。显式选择shell.exe仍能执行shell；本应用和验证器都不拼接shell命令。继承host cwd/env，stdin为EOF；timeout/maxOutputBytes沿§23，不承诺恶意进程隔离或硬实时。仅作字面配置/输出路径相等检查，真实路径别名须调用者避免。工具也可能自行写输出路径，需调用者信任配置。

## 作者实际验收与开发记录

最终target/release/lfz.exe 984064B，SHA256 `4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86`。使用项目内Rust fixture已构建产物，未编译或安装工具；先核`Temp/T18-A-tooling/fixture.exe` SHA256 `51b83f054f128e381fa034adbe980d6398f24f76621703ba27da4d7e54d27bee`，复制到专属`Temp/T18-app/fixture.exe`再核hash，避免别的验收重建竞态。

7/7进程检查通过，exit0，默认sandbox实际许可全部OS进程，未申请提权。先完成7项全部通过，再加强全局终止必须定位exec匿名回调行/帧的检查，重跑最终7/7；两份原始报告都保留，不伪造失败。原始进程stdout/stderr/exit、任务结果与输出文件hex见app/t18a-evidence/，manifest保全源和结果指纹。

覆盖：
- 禁用不spawn marker，已有报告KEEP保持；随后授权只影响该运行，再次无授权仍拒绝。
- 9任务编排：空串/空格/Unicode/引号/尾反斜线argv精确保真；非零7退出仍成功值；双stream合并恰8字节成功、7字节超量失败；局部20ms timeout；非法UTF8；stdin EOF；marker副作用；三个失败后最后echo继续成功。报告为6成功3失败，两次运行字节一致。
- 全局40ms deadline发生于真实exec闭包调用点（app/toolplan.lfz第7行，最内帧<fn>），ExecutionLimitError退出2，后续marker不执行，已有报告KEEP保持。
- 缺字段任务即使位于合法marker任务之后也在全体验证阶段失败，marker不运行；空计划成功。

```powershell
D:/app/ANACONDA/python.exe -B app/verify_toolplan.py --lfz target/release/lfz.exe --expected-sha 4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86 --output Temp/T18-app/new-results.json
```

输出报告路径须尚不存在；所有临时任务和marker在项目Temp。测试12秒watchdog由Python subprocess.run负责超时kill/wait父解释器，Windows后端Job关闭回收其受控子进程；测试未触发watchdog，不凭此声称任意OS故障都可有界回收。后代/故障注入/孤儿检查由独立后端验收负责，本应用7项不替代它们。

没有公网、系统shell拼接、外部安装、产品源码或旧应用改动，也未执行Git。原R304仍OPEN，不以7功能检查宣称性能达标或正式版本。完成本轮交付后按用户边界停止。
