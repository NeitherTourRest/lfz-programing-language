# T26 用户函数 globals 切换原型

2026-11-17，runtime-dev。仅修改 evaluator.rs 中的 call_user。普通脚本沿 modules=None 分支直接使用当前 globals，省去入口 Rc::clone 与退出归还。模块与 REPL 用 mem::replace 保存调用者 module/globals，函数仍在 home 环境中执行，body_result 确定后在返回成功或传播错误前恢复调用者状态。trace/source/depth 的退出路径保持原顺序。

静态核对发现，普通脚本运行期只有 attempt 恢复原 globals 指针。图初始化与 REPL 会话清理属于各自入口，模板默认值切换仅有 TemplateContext 时进入，普通脚本模板为 None。用户对 globals 内容的写入仍通过同一 Env 对象生效。

基线文件 Temp/T26-runtime/evaluator.baseline.rs 的 SHA256 为 049FF34AF33BD90EFFEBF1ABAA9AA4BE9FA2FFFC7F06B2B862CF9D70C5EF96CF，候选为 FC8509362CF8F29271106A098EFB6DFD9362DB504FE40416B4F610F6790EF616。git diff --check -- src/evaluator.rs 通过，差异仅 call_user 两处。主调度统一构建与对照。现有 t16_c_attempt_restores_module_source_and_keeps_escaped_function_id、t16_c_module_globals_and_session_isolation、old_function_and_default_sources_and_snapshot_survive_reset 与 legacy attempt 单测覆盖关键调用路径，尚待重跑。性能结论以实际候选和对照测量为准。

## 独立原型 B

2026-11-17，主调度回报 A 的 715 项测试通过。全量样本耗时约增加 2.6%，回调样本约下降 3.1%，A 退出候选。A 差异留在 Temp/T26-runtime/evaluator.prototype-a.diff。B 从 SHA256 为 049FF34A… 的同一基线生成，保留基线 globals 逻辑。

B 的内部参数表示区分移动 Vec 和借用 slice。直接用户调用把值移交参数槽。回调逐个克隆 Value 到参数槽，省去每次调用时建立临时 Vec，原引用语义不变。实参数量错误与缺函数体错误仍先于绑定。参数按原顺序绑定，原生函数派发 ABI 沿基线。

候选 SHA256 982f54cd78860fa109aa30d837fc1333428de63b16338746f4dfac8b3200db0b。代码已停止写入，交主调度构建和性能筛查。当前证据为基线与候选的静态检查和哈希，等待运行确认。
