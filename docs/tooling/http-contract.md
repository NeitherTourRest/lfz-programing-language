# HTTP CLI与runner集成

语义唯一事实源：docs/spec/interface-contract.md §24。`--allow-http`仅本次运行授权，默认关闭，与`--allow-process`独立；CODE/`--`后参数原样保护。check/test/help/version拒绝能力选项，混合选项按首次类别适用性文案，扫描错误按token序优先。

CLI/runner检查子进程host_diagnostics和http_diagnostics。错误保原业务错误和唯一JSON，独立stderr先子进程后HTTP诊断；成功但任一列表非空则stdout空、exit2，标题先子进程（若有）否则HTTP。不得把attempt捕获Io后的非空诊断当PASS。旧子进程诊断前缀逐字保持。

HTTP返回响应值，不自动打印body。超时是协作取消时刻，安全等待关闭不保证异常OS下绝对返回时界。Windows WinHTTP本轮实现，非Windows Unsupported；HTTPS本地自签拒绝已测，未声明可信公网TLS成功。
