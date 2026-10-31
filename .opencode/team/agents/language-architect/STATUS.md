# language-architect — 工作状态
> 最后更新：2026-10-30 by language-architect

## 当前状态
T18-B三spec §24及T18-B-contract.md11024B已冻结，T18-B-design仅历史提案。root已批准API/迁移/限额/双sidecar/async安全closing。源GO由root即将下达。

## 精确接口与owner
runtime evaluator/builtins唯一：六参特殊httpRequest、75名字注册、EvalCapabilities新增pub http默认false、旧函数/EvalLimits不变；Rust literal机械迁移明确ADR。新TracedRun私有http_diagnostics/accessor，保旧process sidecar/文案。双门禁先process后HTTP。
tooling http_control/platform、CLI/main/lib注册唯一：HttpRequest/Control/Response/Failure/Report及run owned ABI；valid_url/headers/method纯helper给runtime共用。Windows async零crate，其它Unsupported。core error.rs唯一11个Http ValueMsg固定消息。
URL ASCII<=16384bytes、request头<=256对/65536bytes、body<=16MiB；response native头<=64KiB/256项，头API文本UTF8+body raw合并<=max，含statusline/末CRLF不含UTF16 NUL，非wire。

## 生命周期
协作截止cancel→globaldeadline→localtimeout→超量→OS错→完成；锁存后safe closing+最后callback退出前不free，不detach。异常OS cleanup不硬时界；loopback慢头/慢体/TLS拒绝/取消watchdog必须实测。禁止重定向/自动认证cookie代理PAC/应用重试/自动解压，TLS不insecure，无可信TLS成功虚报。

## 证据/下一步
文档diff检查exit0；本角色未源改/编译/Git。完整启动与review均已读，一手资料在提案。root可按§24给owner实施GO，独立/旧回归/最终性能另验，R304OPEN。仅本轮HTTP交付后收口，不T19/自动化。
