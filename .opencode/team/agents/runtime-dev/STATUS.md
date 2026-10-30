# runtime-dev 状态
2026-10-30 T18-A运行时源码阶段完成并停止写源/编译。

evaluator三context API、exec校验/映射/严格UTF8、默认能力禁用、diagnostics私有sidecar/accessor及attempt保留；builtins74特殊注册/通用禁用。新增4unit尚未编译，tooling唯一统一release。报告docs/reports/T18-A-runtime.md。生产与测试不得宣称验收。

backend/process_control由tooling拥有，待owner齐源编译及真实fixture/独立验证。未改error/value/spec/CLI/backend/dist/Git。仅T18-A交付后停止，不HTTP；R304OPEN。

最新：平台校验复用backend executable_valid，非Windows允许绝对路径后再Unsupported；增1不spawn unit，旧夹具按平台选有效路径。仅evaluator，SHA D801E604…；停写未编译，tooling统一重建。无Linux实测声明。
