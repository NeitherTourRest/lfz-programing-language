# T18-B 最终独立功能验收：PASS

2026-10-31。最终target/release/lfz.exe 1029120B，SHA256 18d328d9e48780f8e6ae638f9ad24f53186d94f2207ca24da8928cbbba0d0916。递归src、CLItest、Cargo初末逐项匹配tooling manifest，独立证据T18-B-independent-evidence/manifest.json。

## 实际执行

同十二release harness串行667/667全过，内含499lib；旧attempt13/check117/CLI91（内官方178）/apps37+11/jsonbatch38/独立JSON140全过。A公开45/toolplan7、T17公开134/app40、C公开29/app33完整独立全过。HTTP新公开49/49与httpplan应用7/7独立完整通过。原脚本复制只改ROOT/独立Temp，原旧测试无改动。run.py、extra-commands.json及各原始结果可复现。

本次正常OS许可运行，既有A沙箱canonical/已关闭句柄异常历史不改，本批未凭空制造新的默认沙箱测量。仅原lib harness内部构建process fixture明确获root授权，环境RUSTUP_HOME/CARGO_HOME/TEMP/TMP均项目路径，串行无其它cargo/rustc；没有产品编译。内外fixture SHA记录fixture-hashes.json，不能将重建产物伪装冻结未变。

## 独立HTTP请求见证

own.py复用公开的有界loopback服务器，但请求与断言自行编写，实际11/11：404正常值、重复header保留、302未跟随且只一次请求、cookie不跨session自动回发、POST Unicode body与percent query仅一次、环境HTTP_PROXY无效仍直连、API头59bytes+body2精确N61成功/N60失败、local timeout被attempt捕获、滴流全局总deadline穿透attempt、依赖文件真实faultsource、下一进程默认禁用无连接。全部服务器shutdown/join，未触发watchdog、未清理任何他人进程。环境proxy见证不等同穷举系统PAC配置；无应用重发见证不承诺OS网络永不重传。

精确预算来自原生WinHttpQueryHeaders完整API文本/u16证据，公开绑定元数据SHA a47fd2588b8ad0684199637d71d97352ee419e0f3fbdad67655c0ad8523e4e9d；API已重排头，与wire区分。误将未绑定原见证传给脚本时KeyError route在产品启动前发生，记录prelaunch-witness-error.json，后正确bound完整49通过，未修产品或测试oracle。

## 回调关闭与故障门禁

最终源码在run持有Box context/body/scratch直至request close通知HANDLE_CLOSING及active==0，然后关闭父句柄。state锁覆盖active减计和notify，owner等待需同锁；callback只拷事件/error，无LFZ。异常execute catch_unwind仅先安全close再resume_unwind，不伪装业务成功；没有detach/helper。正常错误、local/global取消均保主因，HTTP sidecar不与原process sidecar混淆。runtime先合诊断再处理result，attempt不清空；CLI/runner双诊断拒绝假成功且错误JSON原reason保持。

预审指出的“20轮normal非取消竞态”证据缺口已补：synchronized_real_cancellation_vs_response_completion用真实服务器seen+Barrier，在响应与另线程Release取消间40轮竞态，允许Ok/Cancelled，逐轮closing/active断言；已在本次同499lib实际执行。close诊断注入真实Close并保Deadline、全局token、慢headers/body/drip总timeout、API native N/N-1、双sidecar runner/CLI门禁均同harness实际运行。无需另编Rust API witness，不夸大为独立外部宿主注入。OS异常closing延迟仍无硬返回时限保证。

## TLS与限制

仅只读审计tooling tls-rejection-final.json原字节SHA匹配其manifest：自签TLS客户端WinHTTP12175拒绝、服务端appbytes0、trusted_tls_success_tested=false。tls-audit.json记录，本角色未重跑TLS；不称可信TLS成功/公网HTTPS验收，未修改系统信任。源码无证书忽略选项，直连/禁自动auth-cookie-redirect、默认不启用自动解压；本次独立11不冒称实际压缩响应覆盖，策略另由源审与公开/owner范围承担。

不公网/外装、不改生产/规范/旧测试、不Git、不计时性能、不真实OOM/abort；非Windows未实测Unsupported，R304OPEN，不正式发布。全部独立进程与服务结束，本轮B后收口，不T19、不恢复自动化。

## 最后人/AI指南只读证据审计（2026-10-31）

只读审计通过，此次启动产品/网络/fixture零次，不是独立重跑。六个人类指南和五个AI文档当前SHA共11份全部匹配作者manifest；两manifest所有证据文件原字节SHA也一致。人21/21与AI25/25逐行passed、同最终18d328/1029120B以及服务器关闭回执均核对，guide-audit.json记录。

默认网络权限失败保留：人指南default-failure.json包含WinHTTP12029且hits空，AI default-results.json为17/25（8触网项12029）、server_visits空与server_closed=true。许可最终同产物通过；没有覆盖默认失败或谎称默认环境已修。人指南还明确保留Python keepalive读线程ConnectionResetError工具输出，不将该warning冒称完全无输出；LFZ21例断言与服务关闭回执均通过。

两报告准确区分API头文本字节与wire、协作取消与硬实时、独立HTTP能力与沙箱、作者CLI例证与库token/closing注入/精确N。未虚称可信TLS成功，TLS自签拒绝仍只是此前tooling证据审计；无新Agent盲测或性能结果。独立功能PASS保持，R304OPEN。

最后审计已收工，停止所有工作，不T19、不恢复自动化；由root进行交付推送。
