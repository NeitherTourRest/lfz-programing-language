# T17 最终候选独立验收（待裁定）

候选 `target/release/lfz.exe`：943104B，SHA256 `1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45`。结论暂为 **CONCERNS / 不予最终通过**：公开121矩阵110通过、11失败，已即时报告team-lead，未改生产、规范或任何旧测试。

独立执行同十release harness全部624通过；旧attempt13、check117、CLI91（含官方178）、jsonflow37、sortviz11、jsonbatch38、独立JSON/argv140，C公开29与四模块应用33全部通过。默认沙箱同lib为463通过/8 canonicalize权限失败；原始日志default-permission.json保留，再以许可运行同产物，不绕过canonical校验。

自编own.py实际14见证通过：单表达式0/1/2步、let共4步、末尾不额外poll、空程序零poll、Deadline优先StepBudget、嵌套attempt穿透、普通失败不补步、有界协作deadline、真实diamond 11成功/10失败、模板默认与函数终止来源、后继独立session。原始逐进程输出、参数和JSON在own.json。

公开失败逐条在public121.json。空for循环100次但无body节点在25步成功，公开期望StepBudget；契约按实际exec_stmt/eval_expr计步，疑oracle误判。CODE保护6项把合法一元负号表达式文本错误预期SyntaxError，实际run为NameError、check成功，疑oracle误判。裸help/version带两项限额4例实际IOError，当作文件处理，契约22.5写明拒绝限额并用指定usage，交architect/tooling裁定。保持原11失败，未经裁定不能写121通过。

独立审阅并实际执行四个新增host单测：cross_thread_cancel_after_session_has_executed_and_run_isolation通过mpsc握手在首语句后另线程Release置位、下一stmt Cancelled；all_seven_hofs_propagate_callback_limits_through_attempt检查七HOF全部callback终止；diamond_counts_shared_dependency_once_and_has_one_session_budget精确11/10；template_default_limit_preserves_definition_source_and_frame验证真实来源及defaults帧。均在同471 lib harness已实际运行。未另编宿主API程序；不把源码审阅等同新增外部宿主注入测试。

实现审阅：poll顺序Acquire cancel→Instant截止→零步，先检查零再减不会wrap；失败调用note_error_span保存当前来源。attempt快照不含limits；Default入口仍不限额。截止仅节点边界，不对native内部中断，不宣称硬实时/CPU或权限沙箱。

源码/Cargo初末SHA冻结，所有证据与脚本在T17-independent-evidence/manifest.json。执行命令为该目录run.py、public121.py、public.py、app.py、own.py，Python为D:/app/ANACONDA/python.exe -B；许可只为原环境canonicalize限制。脚本初次准备遇本机GBK读UTF8失败，发生在产品启动前；仅脚本读取显式utf8后继续，无产品修复。

全程不编译、不改源/spec/旧tests、不Git；不真实abort/OOM、不计时性能，R304 OPEN；本报告不涉及T18实现或正式发布。全部独立测试进程已结束。modules::tests预算夹具可重建，但须root确认无人依赖后清理，本角色未删除。

## 2026-10-30 13:48 裁定后最终独立PASS

前述CONCERNS为真实历史，保留其原始121/110结果和初始独立脚本。architect三spec§22、T17-design与ADR确认：合法帮助/版本入口为--help/-h与--version/-V，裸help/version沿旧文件入口；双负号CODE沿既有表达式语义；空for不伪造节点poll。11项均为新oracle错误，不是产品缺陷，不修改产品。

核作者最终新脚本SHA dd5f8fb1744f92acd2434c365bd5f288c530d1c6f9afd9afbf673ed8c1d52ed0，复制仅更改ROOT和独立Temp目录为public134.py，完整实际复验**134/134 PASS**，原始public134.json。包括原裸词文件错误、合法短长命令拒限额、真实help/version同名文件显式run的0/2步和截止边界。没有省略先前失败。

预算应用另独立**40/40 PASS**，app40-final.json；回调预算28在真实record模块终止，已完成SIDE不回滚。首次迁移探针到更深Temp导致相对导入深度错而IOError，保留app40-initial.py及app40.json；仅修独立副本相对路径为../../../app后完整再跑，未改作者脚本或产品。

只读审阅人指南32/32和AI38/38原始证据，当前六人类指南、五AI文档SHA全部与作者manifest匹配，guide-audit.json记录；这两组是作者成绩审计，不冒称独立复跑。边界说明正确区分host token与CLI、协作deadline与硬实时、默认unlimited和R304。

结合前次同源码624harness、旧全回归/C29/app33/自编14，以及本次134/40，最终独立结论PASS。初末源码/Cargo和产物hash仍一致。无编译、生产修改、Git、性能计时；R304继续OPEN，无正式发布背书。全部独立执行已结束。
