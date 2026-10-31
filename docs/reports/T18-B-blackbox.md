# T18-B 公开脚本准备

最终公开结论：2026-10-31，Windows最终产物18d328d9/1029120B，**49/49 PASS，exit0**；首次完整许可运行，无测试oracle修订。准备历史如下，最终证据见末节。

2026-10-31，test-engineer。依据冻结§24。新增 `tests/t18b_cli_http.py`，内置Python stdlib raw socket loopback fixture，**47例准备完成，executed:false**；无最终HTTP产物，未实跑新功能，不称通过。

fixture只bind 127.0.0.1:0；固定raw响应避免http.server自动Date/Server。单server非daemon worker、短accept/read超时、请求头64KiB/body1MiB上界、stop事件、finally关闭socket并2秒join；上一次慢响应必须idle后才进入下例，防计数污染。逐请求保存method/path/header/body原字节hex及单调时刻；产品PID独占，6秒外watchdog只kill自己的LFZ，server在finally收工，不碰其他进程。HTTP callback/WinHTTP内部是否安全回收仍需tooling白盒，不因父进程watchdog触发而称成功。

当前例涵盖200/204/HEAD/404/500、POST UTF8回显/query percent、重复header/Set-Cookie pair、BOM/CRLF/非法UTF8、redirect目标未请求、零cap、慢头/慢body/持续滴流总timeout及attempt、globaldeadline不可吞、module真实source、arity/类型/pair/非法method/URL/保留header/body/范围无请求、默认/仅process/后续session仍禁用、strict flag/非执行命令/CODE和argv保护。

实际命令：

```text
D:/app/ANACONDA/python.exe -B -c "import ast,pathlib;ast.parse(pathlib.Path('tests/t18b_cli_http.py').read_text(encoding='utf8'));print('AST PASS')"
D:/app/ANACONDA/python.exe -B tests/t18b_cli_http.py --binary dist/lfz.exe --expected-sha 4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86 --output Temp/T18-B-public/prepared.json --prepare-only
```

AST PASS、prepare-only exit0/47/executed:false。prepare不打开listener、不启动LFZ。脚本补充header witness身份验证后再次AST检查；例数不变。现有tooling红 `Temp/T18-B-tooling/red.json`复制为专属 `tooling-red-reference.json`，仅引用：命令 `lfz --allow-http -e 1`期待0，旧版exit2/usage缺flag、无网络动作；不是本角色执行，不伪称绑定新产物或HTTP测试结果。

## exactN证据注入口

可选 `--header-witness PATH`，JSON需含route=/、body_hex=6f6b、raw_headers_crlf（实际WinHttpQueryHeaders完整API头文本）、fixture_wire_sha256。后者须匹配本fixture固定返回：

```text
HTTP/1.1 200 Test\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok
```

只有tooling独立实际API文本见证与固定fixture身份匹配才开启exactN/N-1两例，N=API文本UTF8字节+2；结果保存witness文件SHA。脚本不从返回pairs/wire重建头、不择阈值追绿。当前未提供见证，pending明确列出，不把47冒称含精确边界；加入后49例。

## 尚待补足与角色边界

TLS本地自签拒绝fixture仍待tooling提供，不访问公网/修改信任库；host cancel、callback closing、cleanup双诊断故障注入归库harness/独立，不算CLI黑盒。Cookie跨请求不回发、早断重试/压缩及完整输入大小边界尚在规划表，本次47未覆盖，不计为完成；最终GO时按root范围补齐或用独立证据明确补足。六参输入形状和响应header API文本策略只按§24，不采用旧review接口。

正式实跑须root冻结最终SHA、新output且普通loopback/进程权限明确；脚本逐进程及末尾核产品SHA，保stdout/stderr/exit/异常/请求数。当前只新脚本与专属报告/记忆，没有产品/旧测试/编译/Git/网络请求，R304 OPEN。本轮HTTP后收口不T19。

## 最终49进程实际验收（2026-10-31）

root冻结产品target/release/lfz.exe1029120B/SHA18d328d9e48780f8e6ae638f9ad24f53186d94f2207ca24da8928cbbba0d0916后，实际普通OS权限执行：

```text
D:/app/ANACONDA/python.exe -B tests/t18b_cli_http.py --binary target/release/lfz.exe --expected-sha 18d328d9e48780f8e6ae638f9ad24f53186d94f2207ca24da8928cbbba0d0916 --header-witness docs/reports/T18-B-blackbox-evidence/public-header-witness-bound.json --output docs/reports/T18-B-blackbox-evidence/final-first-permitted.json
```

结果49/49 PASS、exit0、server_errors=[]；script SHA744bbd9ffe46ecc96ef11ec039b8e0f5fc94cce1bce0c853e896564ce0b18831。程序逐case与末尾核hash，验后产品不变；源码验后逐一匹配tooling冻结manifest。单次执行与审批使用同cell，没有并发重复启动；server finally关闭listener并join非daemon worker。无watchdog触发、没有产品修改或测试oracle追绿。

exactN来自tooling真实独立WinHTTP API见证：原始public-header-witness.json SHA08bf660de17c469a9e20b44d8044cf03ce461eece83b2c871935a78074166750，raw API文本UTF8=59、native无NUL118B、body2、N61。API重排Connection/Content-Length，未把wire作为计数。原见证缺route/body_hex这两个测试绑定元数据，本角色另存bound文件，仅补route=/及body_hex=6f6b和原见证SHA，原raw/count不变，并验证固定wire SHA685f7bfbaa0613d701d83e0ffe7b42567be633933ee26cefdb17df29866ca221。bound SHAa47fd2588b8ad0684199637d71d97352ee419e0f3fbdad67655c0ad8523e4e9d，N61成功、N60受控ResponseLimit；没有择阈值反推oracle。

TLS拒绝独立引用作者docs/tooling/evidence/T18-B/tls-rejection-final.json：exit2/IOError WinHTTP12175，server application_bytes=0，trusted_tls_success_tested=false。本角色没有复跑TLS，也不把它计入49。host cancel/callback/双diagnostic故障注入仍属作者harness与独立验证；公开结果pending保留这两类未由本角色执行的边界。

所有原始参数、输出、请求记录、计数和检查在final-first-permitted.json；manifest-final封存产品/source/script/头见证及TLS引用指纹。667作者harness与49公开另计，旧178不修改。未编译/改产品/旧测试/Git，不公网/系统信任修改，无性能结论，R304 OPEN。本轮HTTP后收口不T19。
