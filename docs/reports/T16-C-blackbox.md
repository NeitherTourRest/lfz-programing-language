# T16-C 公开进程黑盒

日期：2026-10-30，责任人 test-engineer。依据三 spec §21、T16-C-design 及 runtime 库阶段交接。最终 C CLI 已完整实际运行：**29/29 PASS，exit 0**。这是公开黑盒结果，独立验收另列。

新增 `tests/t16c_cli_run.py`，29 个独立进程用例，fixtures 仅在项目 `Temp/T16-C-test/` 生成。未改现有测试、manifest、产品、spec、dist、Git；官方 178 项计数保持，公开进程数另列。

| 覆盖 | 用例 |
| -- | -- |
| 依赖后序及一次初始化、canonical diamond、两次进程执行隔离 | diamond_once / fresh_process |
| 独立 globals、home 函数/逃逸闭包、self、统一 argv 新数组 | independent_globals / home_closure / self_methods / argv_snapshot |
| 完成时标量快照、namespace 写不回写绑定、数组/struct 共享、空导出 | namespace_snapshot_shared / empty_namespace |
| 模板定义 globals / captured cell / self、每实例独立可变默认、显式覆盖用 caller、默认全部先执行 | template_definition_globals / template_local_cells / template_self / template_default_order |
| 真正跨文件 JSON 与人类源码、arity 属 caller | cross_error_json / cross_error_human / caller_arity_source |
| attempt 恢复 home，后续错误无旧帧/旧来源 | attempt_home_restore / attempt_new_error_source |
| 默认故障属于定义文件、显式覆盖故障属于 caller | default_error_source / override_error_source |
| 前端坏图先于依赖初始化，文件哨兵保持 | bad_graph_syntax / bad_graph_export / bad_graph_cycle |
| 初始化失败停止后续模块和 entry | init_fail_stops |
| check 不初始化 IO / input / 无限循环 | check_no_init_io / input / loop |
| -e / stdin cwd 基址，JSON 输出通道 | source_eval / source_stdin / json_program_output |

实际准备命令：

```text
D:/app/ANACONDA/python.exe -B -c "import ast,pathlib;ast.parse(pathlib.Path('tests/t16c_cli_run.py').read_text(encoding='utf8'));print('AST PASS')"
D:/app/ANACONDA/python.exe -B tests/t16c_cli_run.py --binary target/release/lfz.exe --expected-sha pending --output Temp/T16-C-test/prepared.json --prepare-only
```

结果：AST PASS；准备命令 exit 0，cases_total=29、executed=false。Python 启动器打印自身 real-location 警告，脚本正常完成；没有调用 LFZ。剩余 D 约 0.17 GiB，未大文件压测。

team-lead 冻结最终 C binary 后，实际按许可使用普通文件权限运行以下命令：

```text
D:/app/ANACONDA/python.exe -B tests/t16c_cli_run.py --binary target/release/lfz.exe --expected-sha cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006 --output docs/reports/T16-C-blackbox-evidence/final-first.json
```

结果：29/29 PASS，exit 0；exe 902656B，SHA 如上。首次完整执行全部通过，没有改产品或放宽断言重跑。原始参数、输入、输出、检查与哨兵在 `T16-C-blackbox-evidence/final-first.json`；manifest 封存原始结果和脚本 hash。验后所有源码 hash 与 tooling 最终冻结 manifest 一致，binary 每进程核 hash；不把验后核对描述为验前源码快照。专属 `.gitattributes` 禁止证据换行转换。

本次未重复制造已知 sandbox canonicalize 权限失败，使用 team-lead 明确批准的普通权限验收；作者原始权限失败与修前 598 候选证据仍在 tooling 证据，不挪作最终 599 或本 29 成绩。未运行官方 runner，178 仍是已有独立基线，不声称本角色重验。

脚本每个进程前后核验 binary SHA，保留参数/stdin/exit/stdout/stderr/逐项检查与哨兵字节，输出路径禁止覆盖；8 秒超时仅检测非预期执行。canonicalize 的已知 sandbox 权限限制需原样保留，由 team-lead 协调同产物许可执行，不改产品身份语义。

边界：宿主手工伪造 ModuleGraph 不能从 CLI 注入，零副作用宿主验证由 runtime unit 单列；安全合成默认递归上限也属 unit，本脚本不造真实 abort/OOM。两次执行用两个进程见证，库 API 同进程 session 隔离由 unit 覆盖。折叠 traceback 与全量 legacy/apps 交独立验收，不计入本 29。R304 OPEN，无性能或正式发布结论。
