只使用提供的五份AI文档与lfz.exe。提交LFZ文件及commands.json（实际check/run argv数组）。先check再run；20分钟、40顶层工具调用、最多5个自调周期，每周期最多1check+1run（合计10次产品启动）。不得批量子启动绕预算。不得读仓库、其它题、评分器或历史答案，不得联网公网/外装/调用其它语言解决业务。workspace仅提示隔离，不是OS沙箱。提交后无隐藏验收反馈或继续修复机会。stdout业务JSON一行，除特别模式外不使用CLI --json。所有题maxsteps200000/timeout5000ms，资源错误不得吞。

提交main.lfz。argv=[baseURL]，baseURL仅测评宿主临时127.0.0.1地址。依次GET /ok,/missing,/redirect,/bad,/ok，每次timeout1000、cap16384，使用httpRequest而非shell工具。成功输出{status,body,repeated}，repeated为所有x-repeat值数组保序；失败输出{error:class}。最终stdout为五项JSON数组。404普通响应，302不自动跳转，/bad严格UTF8失败后继续。先check，再无授权run应产生五个ValueError但零网络请求；正式--allow-http限额run得到服务响应。不可读fixture服务实现/日志。无需TLS/真实公网/精确wire字节计量。

允许只为自调启动公开selftest_service.py（宿主Python），读取其stdout baseURL；它不是业务解题语言。保持服务前台可控并在自调结束关闭，调度者可代管同样公开服务。不得读取privatefixture/隐藏日志。服务启动算顶层工具调用，不算LFZ产品进程，自调check/run仍最多5周期。
