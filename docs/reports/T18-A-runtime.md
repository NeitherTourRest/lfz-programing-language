# T18-A运行时源码交接
2026-10-30。仅evaluator/builtins生产修改；未执行cargo，tooling唯一编译者。

三context owned API、EvalCapabilities/EvalContext默认false，旧EvalLimits三字段及所有旧入口保留。exec特殊派发在名字遮蔽后，与attempt同级，不作为一等内置。按契约先参数数量、全部argv元素类型、值约束、两int类型、范围、权限，再交owned backend处理平台及native资源预检。成功输出四键，UTF8严格解码；失败映射ValueMsg/Io/全局ExecutionLimit。后端diagnostics合入Interp并转TracedRun私有sidecar/accessor，不入attempt快照；权限同样不恢复且各session独立。

注册74名字；通用builtins::call/call_with无上下文exec固定禁用，ABI不改。注册表测试只机械新增exec及计数74，旧表72断言不改。

新增4项unit：校验先后/默认禁用不spawn、合成report UTF8/退出码、global终止不可recover+cleanup诊断保留、attempt后诊断与默认能力。均未编译运行，不能声称通过。backend尚未落文件时process_control引用静态缺接口；按调度禁止编译，未制造编译红证据。真实进程/Job/pipe/超时/回收需tooling和独立fixture验收。

源码diff --check通过；停止改源交tooling统一完整release。CLI须调用context入口并检查host_diagnostics：Err保原错误，诊断独立stderr；Ok但非空走宿主失败exit2，不能只看result。R304OPEN，无性能承诺；本轮完成后停止，不进入HTTP。

## 平台校验最小修正
作者646/E5F57候选已由tooling保留。runtime validate_exec删除重复Windows路径逻辑，复用pub(crate) process_control::executable_valid单一事实源：Windows盘符/UNC.exe与设备拒绝不变，非Windows仅Path::is_absolute，平台未实现仍后端Unsupported。旧无spawn测试的有效路径夹具按平台选择；新增不spawn测试核对runtime/backend同规则、盘符相对/设备/批处理拒绝与非Windows绝对路径接受。仅evaluator生产/测试改动，无Linux执行实现，无Linux实际验收声明；未编译，交tooling统一最终构建。最终SHA D801E6043ABF229C763366A0771E5C14B83BA7CC9F0BD2B8C689CA1C54D36B99，diff check PASS。
