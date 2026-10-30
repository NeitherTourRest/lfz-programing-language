# T16-C CLI模块执行与真实跨文件诊断

日期2026-10-30，tooling-dev。消费冻结§21及runtime图API；不改core/runtime/spec。

run文件/-e/stdin以counted根加载和一次lex/parse为入口。无声明保持legacy；有声明build_graph后同大栈直接调用eval_graph_traced_on_thread_with_args，使用一次性图快照，执行/图析构都在栈内。check仍只构图。GraphFailure用内部Located载荷保留真实前端文件和可选Loaded；runtime错误取error_source，JSON帧及人类帧取frame_source原始index，首10尾30折叠尾index使用total-tail+offset，legacy None保持旧渲染。

宿主GraphExecutionError转内部HostFailure，不伪造LFZ错误类；run及runner用递归host_failure检查，固定stderr“lfz: 模块执行上下文无效：{detail}”并exit2。runner在judge前直接中止且judge也防御，避免classNone误PASS。Located包HostFailure也fail-closed。此runner扩展已获team-lead明确授权。

B测试最后一条阶段性run拒绝断言经授权迁移为C真实成功/print1/文件写入，其余六条不改，旧library模块guard原样。新cli_t16c五组验证三入口home globals、依赖初始化失败无entry副作用、运行图前端依赖路径、51函数帧JSON全量及人类折叠、attempt恢复后的新root错误来源。

作者结果：修前671919/902144B候选598通过保留；新增宿主合成测试在旧judge编译后红0/1，修复后绿1/1，Located断言合并同测试。一次作者编译误用private origins字段已改为公开frame_source，失败编译日志保留，不称产品红测。C新helper unused警告已去。

默认cargo test --release因8个canonicalize沙箱权限失败在lib阶段448通过后中止，保留默认日志。按授权对最终九个已编译release harness提权执行599/599：456lib+49bin+28cli+12T12+12T14+7B+5C+15runner+15unit，0失败/忽略。无改变canonical身份策略、跳过旧用例或删除测试。build --release exit0。新五个C进程测试提权5/5通过；旧B执行基线exit2证据在red-b-baseline.json，明确为实现后记录的旧版对照，不冒称测试先行。

最终target/release/lfz.exe902656B，SHA256 `cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006`。全部当前src/Cargo及B/C测试hash、原始日志hash在docs/tooling/evidence/T16-C/manifest.json；目录-text保全原字节。dist、Git、profile未动。收工停止改源/构建/测试，后续公开29、独立验收、文档/交付由team-lead接续；R304不因功能全绿关闭。
