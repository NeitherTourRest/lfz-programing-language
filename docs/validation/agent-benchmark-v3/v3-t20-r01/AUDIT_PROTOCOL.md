# 独立审计与冻结（待verifier预审）

主调度派学生前记录UTC start/deadline(1200s)、session/parent/agent path、dispatch SHA、exe/五docs/TASK/fixture hash、允许工具和workspace；fresh context、相同配置。20min、40顶层tool调用、5周期每周期check≤1/run≤1。shell中隐藏的子启动也须进程日志计数，顶层batch计1但所有实际产品启动计数不豁免。准备学生wrapper记录启动源码快照/命令/UTC/stdout/stderr/exit；wrapper不能证明学生没有旁路启动，须完整宿主trace交叉核对。缺此证据协议UNKNOWN。

复用旧v2机制：由调度/审计者读取匹配session的宿主真实工具调用及返回、turn_context，不让学生自报数量。允许的源只读范围为该run关联session、parent/agent_path匹配记录；禁止扫描无关聊天或导出私有推理/系统/developer/密钥。原session只读hash锚定，可分享导出仅含model/effort事实、时间和工具调用/结果及文件快照。若宿主接口不可取得完整trace，先记录缺口，不能造model或PASS。

协议审核记录须有task、reviewer、source_hashes、verdict PASS/FAIL/UNKNOWN、source_trace_sha256、export_sha256、session_id、actual_model、actual_effort、UTC start/deadline/received、top_level_calls、product_calls、cycles、每次snapshot_sha256、check_before_run、输入hash核对、访问范围审查结果。未提供model/effort写unknown。actual_model未知不自动功能FAIL；完整工具trace缺失必须协议UNKNOWN。不能从学生commands.json或wrapper输出单独推断工具完整性。

source-review单独包含绑定源码hash、独立reviewer和每条约束判定。文字搜索只能提示，不足以认证“未collect/无每行closure/未伪造Result”，需人工源码审阅与功能负控制。PASS要求协议/源码/功能三轴都PASS；runner接受独立审查记录不等于自动证明记录真实，verifier须核原材料及签收。

正式冻结前必须补齐：T20最终exe+五docs hash；TASK/source约束/oracle所有歧义裁定；verifier实跑参考解与负控制（预期失败规则与真实失败对应）；wrapper与真实trace来源可用性；manifest审核SHA与raw字节ZIP。manifest当前status DRAFT_PENDING_REVIEW_AND_FINAL_ARTIFACTS；没有其余手续不得用本包启动学生。改输入/oracle须新revision，不能评分后改原oracle追绿。

最终评分目录非学生目录；学生收到student/<id>内容+五docs+exe。shared filesystem不代表OS访问控制，不宣称sandbox。C04环境许可在学生前统一确认，失败保全并一致处理；不公网、不TLS信任改动、不外装。REPL另轨、旧v2不动，四题结果不得和旧8题计算提升比例。
