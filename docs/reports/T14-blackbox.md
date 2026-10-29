# T14 非执行check公开进程黑盒验收

日期：准备2026-10-27，最终实跑2026-10-29。责任人：test-engineer。当前状态：最终公开进程黑盒117/117通过；下方准备记录按时点保留。

## 依据与范围

只按三spec §17和冻结设计设计公开进程用例，不读取解释器实现。新增 `tests/t14_cli_check.py`，不修改既有LFZ用例、fixtures、manifest或Rust测试。T14是CLI模式新增，官方LFZ runner保持原166项；本脚本检查CLI行为，不能把其进程检查数叠加成LFZ runner计数。

覆盖三来源和普通/JSON模式、CODE保护、usage错误、加载前导/UTF8/BOM/CRLF、准确行列、前端错误、非静态保证、真实文件字节哨兵、输出/提示隔离、无限循环/递归不执行及原run/check内置兼容。进程超时5秒用于识别意外执行，不作为性能指标。每个进程前核binary hash；失败、超时和文件前后hex全部保留。

`--json`成功严格逐字节检查三键一行，失败检查字段顺序、mode/executed、错误类、位置和单个定位trace；usage依现契约为stderr文本、stdout空。非执行文件见证组合write/append已有和不存在路径、print/eprint/dump/input/循环，并分别在完整合法源码、随后语法错误、随后非法字符三种情况下测试file/-e/stdin和两输出模式。

## 实现前证据

运行：

```powershell
D:/app/ANACONDA/python.exe -B tests/t14_cli_check.py --binary Temp/t13-perf/lfz-t13-9cda.exe --output docs/reports/T14-blackbox-evidence/preimplementation-red.json --smoke-only
```

结果exit1：新增命令烟雾例0/1，旧9cda进程返回exit2和usage诊断。这证明该版本尚未支持 `check` 命令，是新增能力的实现前红测，不是旧产品回归。

另外通过旧公开 `run -e --json` 入口确认语法基准：CODE字面 `--json`/`--help`在语法阶段合法而运行到NameError，`--`为SyntaxError；末尾换行的 `let x =` 错误位置为行尾1:8（不是下一行EOF）。原始公开输出保全在 `preimplementation-public-oracles.json`，据此纠正作者最初的测试行号预期，不将测试草拟错误归为产品问题。红测运行时脚本hash与纠正后的当前脚本hash分别由JSON和实际脚本提供，保留其时点差异。

## 原准备运行指令（2026-10-27历史）

```powershell
D:/app/ANACONDA/python.exe -B tests/t14_cli_check.py --binary target/release/lfz.exe --output Temp/T14-test/final-cli-new.json
```

待新候选哈希下发后只运行一次全量，保全实际总数/失败详情并更新本报告、REPORT和覆盖矩阵。脚本可用 `--binary` 指定项目内路径，输出必须新建在项目内，不覆盖既有证据；不编译、不改dist、不自行挑选候选。

## 最终产物实跑（2026-10-29）

主调度指定并逐进程核对：`target/release/lfz.exe` **774656B**，SHA256 **83f56a194e216862c45377f4c3971b085744a7ba2363163e1edac6af5e85ac79**。没有编译、改dist或选择其它候选。

```powershell
D:/app/ANACONDA/python.exe -B tests/t14_cli_check.py --binary target/release/lfz.exe --output docs/reports/T14-blackbox-evidence/final-cli-corrected-20261029.json
```

结果 **117/117 PASS，0 FAIL，exit0**。首次运行116/117，唯一差异是作者把源码文件非法UTF8的预期误写为IOError。规范interface-contract既有加载契约明确为SyntaxError；IOError属于运行时readFile或stdin读取的另一个契约。只纠正本轮新脚本这一错误oracle，并将输出date改为实际2026-10-29，新文件重跑全量，产品和旧测试均未改。首次JSON完整保留为 `final-cli-20261029.json`，其date历史硬编码为27，实际timestamp_utc为29；不覆盖失败、不把测试作者错误包装成产品修复。

| 公开行为 | 进程用例数 | 结果 |
| -- | -- | -- |
| 新命令烟雾、三来源普通/JSON | 7 | PASS |
| CODE保护和选项位置 | 11 | PASS |
| CLI参数负例，普通/JSON请求均保持usage文本 | 28 | PASS |
| 非静态承诺，名称/类型/除零/IO/循环/递归等 | 24 | PASS |
| 加载正负例、无文件、非法stdin、准确位置 | 17 | PASS |
| 语法/上下文错误 | 7 | PASS |
| 文件字节哨兵：三来源×两输出×三合法/语法错误状态 | 18 | PASS |
| 原run、内置check、命名check文件、帮助兼容 | 5 | PASS |

合计117个进程用例。18哨兵组合中已有文件二进制字节保持，新目标不创建，输入/输出/循环均未执行；5秒超时只用于识别意外执行，不是性能基准。JSON成功逐字节验证三键；失败检查键序及定位trace，usage例外明确。

官方LFZ runner仍166，本脚本不改其用例/manifest、不将117加到166中。542 Rust、旧91进程及其它独立验证由各自负责人提供证据，本报告不冒称本角色重新跑过。新增 `tests/.gitattributes` 仅一条`t14_cli_check.py -text`，保护新脚本SHA；证据目录原字节保护。当前脚本SHA256 `44e1c5d9639b5306da9779994e0acb30f62f4943aebbc116de735e4a86a42106`。
