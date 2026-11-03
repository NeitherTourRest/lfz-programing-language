# T19 独立静态预审

2026-11-03，verifier。按启动协议读取全局状态/看板/自身记忆，审§25/T19-contract及活动源。未编译、未运行产品，不构成最终功能或性能验收。活动源指纹在T19-independent-evidence/preaudit-sources.json，最终须新hash核对。

当前未发现确定产品缺陷。LineReader使用容量8KiB BufReader，fill_buf/consume只按当前块查LF；追加前以max减当前长度检查take，超过立即TooLong不复制坏行尾，reserve_exact按需要增长而非几何扩容。LF前rawbytes含CR，只有实际遇LF才剥一个终止CR；EOF末CR保留。空文件/尾LF不伪造空行；BOM/NUL数据保留；跨块UTF8拼成有界完整行才严格解码。last==i64MAX只在确有非空后续块时Overflow，真正EOF允许正常None。测试注入last而非天量文件。

普通文件metadata前检及open后handle.metadata复检均在，符号链接跟随；不消灭TOCTOU，不保证OS同步open/read能硬中断。poll在块前与成功行/EOF前，仅Acquire cancel→deadline，不扣或检查AST预算。读/解码已确定错误不在清理时追加poll覆盖。所有reader/File私有owner，不逃逸LFZ值。

runtime数量→callback Function→max int→path string→callback两参→范围→pathNUL→native预检→open的顺序符合契约，空文件亦预检。call_value调用沿原home/captured/self/source/trace，不持Env RefCell借用，不新Interp。回调前及正常返回后poll（包括false），严格bool，无truthy转换。lines包含false那行；早停不再next_line，后坏UTF8不被解码。读取错误消息调用点note_error_span，数据lineNo不伪装LFZ源码文件；callback故障沿原调用源。参数/真实回调节点用同会话预算，native无额外StepBudget，attempt不会补步。

open成功后循环位于outcome闭包，任何?、EOF、false、非bool、业务/资源/读取/UTF8/TooLong都先离开闭包，再统一reader.finish消费owner，最后返回Outcome；意外路径仍RAII兜底。finish不假装回传OSclose错误，不添加第三诊断列表或平台关闭FFI。无sync_all替代close声明。

已读现有unit：reader同host存活share_mode(0)负控在reader持有时Win32 sharing violation，finish/drop后排他重开；callback活跃marker后另一host线程取消，再同进程排他重开；runtime false跳坏尾、JSON连续恢复、nonbool/业务/Step终止释放；warmed handle count平台抽样。上述此时仅源码审阅，不冒称已独立执行。最终须明确逐Outcome验收EOF、UTF8、TooLong、Deadline及外attempt返回后同host排他重开；metadata/open/read失败路径和全局优先/主因不覆盖仍需实际证据。删文件/rename/产品进程退出不是关闭证据。

最终计划：待统一最终exe与递归source/Cargo/test hash、owner停止编译后实际同harness和旧全回归；新公开、独立行byte边界/跨块/末行/JSONL/严格bool/早停与嵌套/模块source与home/self、exact AST预算和全局终止见证；以同host存活资源探针证明finish，不制造真实abort/OOM。Windows真实实测不能推广为未验平台。

至少10/100/300MiB同分布大文件oracle和100MiB readFile对照，由perf-engineer在最终hash、无并发测量窗实测原始peakWS/RSS、walltime/吞吐；当前源码空间结构不能代替实际内存/吞吐证据。用户主动collect仍可O(file)，不宣称任意程序恒定内存；T19测量不等于原R304达标。R304OPEN，无性能结论/正式发布背书。只本轮T19交付后收口，不T20、不恢复automation。
