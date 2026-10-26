# T12 性能测量与回归验收

日期：2026-10-25。当前最终产物：1.4.0，759808B，批准release opt-level=2，SHA256 `47ed2bfe2a2694b11bd611172cfcf0000d0bfc58e3b9bd809ca95f066dd0261f`。

**最新结论：opt2正式主产物15/19在原固定性能线内，3项环境不确定、1项未通过，R304继续开放。** 数值500k旧104.940→新90.804ms，结构体8k旧93.431→新21.198ms（约77.3%降时）；这些局部收益和包体变化不抵消未通过项。JSON/CLI9组输出正确、每启动核最终hash，峰值内存未测。保存为可审阅候选checkpoint，不据此宣布整体效率验收完成。

以下先给opt2最终产物结果，后面的实现前、FB8首版、路径控制和准备记录均为历史证据，保留失败，不将旧版结果冒充最终产物。旧课程性能报告 `performance.md` 保留其原测量日期与口径。

## 最新opt2正式主产物：15项达线，R304继续开放

本轮只对已批准release opt-level=2的正式主产物测量一次，不新增profile候选。默认未绑核，每copy共同subject后固定等5秒，原19组顺序和旧/新交替次序、2预热+7正式均不变，无额外90秒、不挑选最快读数。每启动核hash，所有旧证据保留，原绝对阈及struct至少30%目标未改。

最终结果：15/19在原绝对线内，3项环境不确定、1项仍超线。**R304继续开放，本轮仅作为功能/优化已验证的可审阅候选checkpoint，不能据此宣布整体效率通过或正式版本验收完成。** 表中paired是同期同路径新版wall/旧版wall，>1表示新版较慢。它必须同时披露，但不能替换原绝对合格线。

| 工作负载 | N | 旧wall ms | 新wall ms | 旧user ms | 新user ms | paired新/旧 | 原wall阈 ms | 判定 |
| -- | -- | -- | -- | -- | -- | -- | -- | -- |
| noop | 0 | 17.481 | 16.287 | 0.000 | 0.000 | 0.932 | 16.307 | PASS |
| numeric_loop | 10000 | 21.274 | 18.762 | 15.625 | 0.000 | 0.882 | 17.968 | 环境不确定 |
| numeric_loop | 100000 | 37.824 | 29.892 | 15.625 | 15.625 | 0.790 | 38.778 | PASS |
| numeric_loop | 500000 | 104.940 | 90.804 | 78.125 | 78.125 | 0.865 | 131.497 | PASS |
| array_builtins | 1000 | 16.267 | 15.250 | 0.000 | 0.000 | 0.937 | 17.210 | PASS |
| array_builtins | 10000 | 22.434 | 21.907 | 0.000 | 0.000 | 0.976 | 22.772 | PASS |
| array_builtins | 50000 | 44.840 | 43.837 | 15.625 | 15.625 | 0.978 | 52.306 | PASS |
| struct_ops | 2000 | 20.864 | 16.422 | 0.000 | 0.000 | 0.787 | 22.445 | PASS |
| struct_ops | 4000 | 37.393 | 19.053 | 15.625 | 0.000 | 0.510 | 37.870 | PASS |
| struct_ops | 8000 | 93.431 | 21.198 | 62.500 | 0.000 | 0.227 | 107.352 | PASS |
| string_ops | 2000 | 33.267 | 35.553 | 15.625 | 0.000 | 1.069 | 28.902 | 环境不确定 |
| string_ops | 4000 | 75.879 | 97.718 | 46.875 | 46.875 | 1.288 | 101.904 | PASS |
| string_ops | 8000 | 216.079 | 223.737 | 109.375 | 109.375 | 1.035 | 236.353 | PASS |
| string_map | 2000 | 17.285 | 15.901 | 0.000 | 15.625 | 0.920 | 17.522 | PASS |
| string_map | 4000 | 23.291 | 18.928 | 0.000 | 0.000 | 0.813 | 23.063 | PASS |
| string_map | 8000 | 21.871 | 22.691 | 0.000 | 0.000 | 1.037 | 22.521 | 未通过 |
| struct_small | 4 | 20.319 | 20.126 | 0.000 | 0.000 | 0.990 | 18.746 | 环境不确定 |
| struct_small | 16 | 31.363 | 28.772 | 15.625 | 15.625 | 0.917 | 34.634 | PASS |
| struct_small | 64 | 69.918 | 64.482 | 46.875 | 31.250 | 0.922 | 78.311 | PASS |

- numeric10k/string2k/small4同期旧版也超参考，按冻结规则环境不确定，不能通过。
- string_map8k新版22.691ms超原22.521ms，旧版21.871ms在线内，仍记录未通过，即使幅度很小也不放宽原线。
- string_ops4k绝对97.718ms在101.904ms内，但paired比旧75.879ms慢约28.8%；报告不隐藏此差异，不把“绝对PASS”说成各负载全面加速。
- numeric500k旧104.940/新90.804ms，结构体8k旧93.431/新21.198ms（约77.3%降时）均达目标；startup新16.287ms在原16.307ms内，余量很小，不能宣称启动稳定优于旧版。
- 小对象16/64键在原线内，4键环境不确定；dist真实交付路径500k探针90.855ms在131.497ms内，限定本次条件。

本产物同时含while空Env优化、struct索引和profile opt-level=2，不能将全部提速单独归因某一个改动。初轮FB8同源读数差异、路径/CPU诊断与候选失败都保留；精确LLVM/环境微因仍未知，不以推测取代证据。包体变小也不能抵消性能未通过项。

### opt2 JSON/CLI全组

| 工作负载 | N | 输入字节 | 总wall中位数ms |
| -- | -- | -- | -- |
| json_parse | 2000 | 23781 | 27.271 |
| json_stringify | 2000 | 23781 | 15.898 |
| json_parse | 4000 | 49781 | 42.153 |
| json_stringify | 4000 | 49781 | 21.058 |
| json_parse | 8000 | 101781 | 71.695 |
| json_stringify | 8000 | 101781 | 25.168 |
| cli_eval | 0 | — | 14.758 |
| cli_stdin | 0 | — | 13.140 |
| cli_argv | 0 | — | 15.299 |

每进程JSON重复20次，9单元63次正式输出与Python独立oracle一致，每次启动核47ED hash。输入2k/4k/8k键为23781/49781/101781字节。总时含进程启动、加载、文件读入和准备；本窗startup16.287ms占parse首档约60%，依冻结规则不给纯吞吐/近似net缩放结论。峰值RSS未采，不能声称内存下降；复杂度由契约/源码分析而非这三档总耗时证明。

### opt2证据、复现及最终限制

- `opt2-same-path.json`：19组旧新266次正式值+dist探针7次，逐次hash、stdout、wall、user/kernel CPU及原绝对阈完整保存，全部输出正确。
- `opt2-new-features.json`：9单元63次正式值及每轮hash，输出全部正确。
- `opt2-assessment.json`：绑定上述原始JSON SHA256，记录15 PASS/3环境不确定/1 FAIL及R304 OPEN、同期paired比值，不覆盖任何旧数据。

```powershell
D:/app/ANACONDA/python.exe -B benchmarks/t12/final_compare.py --new-binary dist/lfz.exe --expected-new-sha 47ed2bfe2a2694b11bd611172cfcf0000d0bfc58e3b9bd809ca95f066dd0261f --settle-seconds 5 --output Temp/t12-perf/opt2-independent-same-path.json
D:/app/ANACONDA/python.exe -B benchmarks/t12/new_features.py --lfz Temp/T12/perf-subject.exe --output Temp/t12-perf/opt2-independent-new-features.json
```

输出必须为项目内新路径，两个命令依次执行且第一条成功结束后第二条才测subject当前新版。最终测量已完成，不继续新增profile候选或无限择窗；其它agent可恢复任务。整体效率门槛作为开放项保留，下一轮针对未通过/受环境影响项再决定改进，而非改本轮验收标准。

## 较早固定静默/冷却窗口：整体仍未通过

本次是预先指定的新控制方法，未挑最快读数。所有项目agent停止其它CPU/扫描任务，开始前固定静默90秒；恢复默认未绑核。每次将旧/新字节copy到相同subject后固定等5秒，再预热2+正式7。组顺序不变，旧/新组次序照原交替。窗口完整执行一次，没有因早期不理想重启，没有移除慢样本。

准备检查CPU11.69%、旧noop15.098ms低于原16.307ms，故允许进入正式组。准备成功不代表整段系统始终稳定。全部19组输出精确一致、每次启动核hash，正式273次（旧新266+dist探针7），JSON/CLI另63次输出一致。当前结果是13项在原线内、5项环境不确定、1项仍超线；不能写整体通过。

下表每项均列wall与Windows GetProcessTimes user CPU中位数，单位ms。原接受阈只针对wall，CPU数据没有新合格线。user值较小或为0时受系统进程计数分辨率影响；每轮kernel也保存在JSON，不能以短进程CPU=0推断没有工作。

| 工作负载 | N | 旧wall | 新wall | 旧user | 新user | 原wall阈 | 判定 |
| -- | -- | -- | -- | -- | -- | -- | -- |
| noop | 0 | 15.190 | 17.948 | 0.000 | 0.000 | 16.307 | 未通过 |
| numeric_loop | 10000 | 18.477 | 18.384 | 0.000 | 15.625 | 17.968 | 环境不确定 |
| numeric_loop | 100000 | 35.183 | 30.825 | 15.625 | 15.625 | 38.778 | PASS |
| numeric_loop | 500000 | 107.422 | 90.379 | 78.125 | 62.500 | 131.497 | PASS |
| array_builtins | 1000 | 14.960 | 16.339 | 0.000 | 0.000 | 17.210 | PASS |
| array_builtins | 10000 | 22.736 | 19.925 | 0.000 | 0.000 | 22.772 | PASS |
| array_builtins | 50000 | 43.205 | 44.969 | 15.625 | 31.250 | 52.306 | PASS |
| struct_ops | 2000 | 23.830 | 17.844 | 15.625 | 0.000 | 22.445 | PASS |
| struct_ops | 4000 | 33.835 | 19.797 | 15.625 | 0.000 | 37.870 | PASS |
| struct_ops | 8000 | 92.184 | 21.535 | 46.875 | 15.625 | 107.352 | PASS |
| string_ops | 2000 | 33.021 | 34.436 | 15.625 | 15.625 | 28.902 | 环境不确定 |
| string_ops | 4000 | 81.690 | 70.992 | 31.250 | 31.250 | 101.904 | PASS |
| string_ops | 8000 | 200.101 | 210.324 | 93.750 | 109.375 | 236.353 | PASS |
| string_map | 2000 | 17.621 | 18.388 | 0.000 | 0.000 | 17.522 | 环境不确定 |
| string_map | 4000 | 22.454 | 19.633 | 0.000 | 0.000 | 23.063 | PASS |
| string_map | 8000 | 23.991 | 24.688 | 0.000 | 0.000 | 22.521 | 环境不确定 |
| struct_small | 4 | 20.064 | 19.069 | 0.000 | 0.000 | 18.746 | 环境不确定 |
| struct_small | 16 | 30.917 | 29.559 | 0.000 | 0.000 | 34.634 | PASS |
| struct_small | 64 | 69.818 | 72.566 | 46.875 | 31.250 | 78.311 | PASS |

noop新17.948ms仍高于原16.307ms，同期旧15.190在线内，真实记未通过。numeric10k/string2k/map2k/map8k/small4同期旧也超线，按原规则环境不确定，不放宽。numeric100k/500k与小对象16/64键均达标；结构体8000键旧92.184/新21.535ms（约76.6%降时），满足该样本至少30%目标。dist500k102.878ms在原131.497ms线内，仅限本探针条件。

### 本窗口JSON/CLI（默认未绑核）

| 工作负载 | N | 输入字节 | 总wall中位数ms |
| -- | -- | -- | -- |
| json_parse | 2000 | 23781 | 32.039 |
| json_stringify | 2000 | 23781 | 20.748 |
| json_parse | 4000 | 49781 | 48.014 |
| json_stringify | 4000 | 49781 | 24.987 |
| json_parse | 8000 | 101781 | 81.709 |
| json_stringify | 8000 | 101781 | 28.951 |
| cli_eval | 0 | — | 15.946 |
| cli_stdin | 0 | — | 15.643 |
| cli_argv | 0 | — | 15.752 |

每进程JSON操作20次，全部输出与Python独立oracle一致；每次启动核最终7641 hash。parse总32.039/48.014/81.709ms；本窗startup17.948ms较高，首档占比超过50%，依冻结规则不给纯吞吐/扣启动缩放结论。只报告输入输出字节，不采峰值RSS；复杂度仍由源码/契约论证，不能拿受启动污染的总时证明。

### 本窗口证据与复现

`quiet1.window.json`记录开始/结束、固定90/5秒方法和completed状态；`quiet1.readiness.json`记录先验准入；`quiet1.same-path.json`记录所有19组每轮hash/wall/user/kernel与原固定阈；`quiet1.new-features.json`记录9组所有输出与hash。所有初轮、失败候选、未绑核与CPU0结果原样保留。

```powershell
D:/app/ANACONDA/python.exe -B benchmarks/t12/quiet_window.py --prefix Temp/t12-perf/independent-quiet
```

输出prefix须为项目内新路径；该命令会真实静默90秒，并每次copy后等5秒。准备不达线即记录not-ready停止。复现需其它项目任务静默，但无法保证OS后台无负载。总执行约5分钟，不能用最后结果覆盖既往失败或选取最好一轮。

**后续仍是未通过的效率验收项**：保留原线，处理启动超阈及5项环境不确定；本次固定窗口已完整执行，不继续盲目追加窗口。是否接受当前明确局部收益并保留整体效率限制，由team-lead据真实结果裁定，不能由性能报告自行把门槛改成通过。

## 较早CPU0控制实验：整体仍未通过

按测前指定方法，CPU0不临时换核、不改合格线。两个harness原mask均16777215，active=1，finally恢复16777215且restore_verified=true；结果JSON保存进程ID及mask。所有子进程从创建起继承父mask。19项同路径预热2+正式7，JSON/CLI同CPU0重测，全输出正确，binary仍7641。

19项12项在原固定线内、7项超线。5个超线项同期旧版也超参考，环境不确定；array50k与small4同期旧版在线内而新版略超线，仍按未通过记录，不能全部归因环境。由于本窗口存在广泛旧版超线，依测前规则不自行重复挑窗口，待team-lead另安排真正静默冷却窗口。CPU0控制只减少跨核因素，没有让系统噪声自动消失。

| 工作负载 | N | CPU0旧版ms | CPU0新版ms | 原固定阈ms | 判定 |
| -- | -- | -- | -- | -- | -- |
| noop | 0 | 17.714 | 22.324 | 16.307 | 环境不确定 |
| numeric_loop | 10000 | 14.338 | 16.965 | 17.968 | PASS |
| numeric_loop | 100000 | 35.634 | 31.564 | 38.778 | PASS |
| numeric_loop | 500000 | 115.818 | 94.414 | 131.497 | PASS |
| array_builtins | 1000 | 26.695 | 26.979 | 17.210 | 环境不确定 |
| array_builtins | 10000 | 31.004 | 24.819 | 22.772 | 环境不确定 |
| array_builtins | 50000 | 51.818 | 52.644 | 52.306 | 未通过 |
| struct_ops | 2000 | 32.536 | 28.140 | 22.445 | 环境不确定 |
| struct_ops | 4000 | 45.970 | 23.300 | 37.870 | PASS |
| struct_ops | 8000 | 93.671 | 23.704 | 107.352 | PASS |
| string_ops | 2000 | 30.891 | 39.783 | 28.902 | 环境不确定 |
| string_ops | 4000 | 69.621 | 86.219 | 101.904 | PASS |
| string_ops | 8000 | 184.158 | 205.197 | 236.353 | PASS |
| string_map | 2000 | 16.715 | 16.782 | 17.522 | PASS |
| string_map | 4000 | 22.576 | 18.770 | 23.063 | PASS |
| string_map | 8000 | 22.867 | 22.020 | 22.521 | PASS |
| struct_small | 4 | 18.418 | 19.385 | 18.746 | 未通过 |
| struct_small | 16 | 29.076 | 28.088 | 34.634 | PASS |
| struct_small | 64 | 69.371 | 62.828 | 78.311 | PASS |

数值500k旧115.818/新94.414ms，struct8k旧93.671/新23.704ms（约74.7%降时），两项目标达标。startup旧17.714/新22.324ms，均超原16.307ms线，不能据此给出启动更快结论；小对象4键仍超线，16/64键达标。dist交付路径CPU0的500k探针91.436ms在原线内，限定此探针条件。

| JSON/入口 | N | 输入字节 | CPU0总中位数ms |
| -- | -- | -- | -- |
| json_parse | 2000 | 23781 | 27.265 |
| json_stringify | 2000 | 23781 | 17.875 |
| json_parse | 4000 | 49781 | 37.080 |
| json_stringify | 4000 | 49781 | 18.489 |
| json_parse | 8000 | 101781 | 64.398 |
| json_stringify | 8000 | 101781 | 26.571 |
| cli_eval | 0 | — | 15.205 |
| cli_stdin | 0 | — | 14.357 |
| cli_argv | 0 | — | 13.470 |

全部9单元63正式输出一致，每启动核hash，峰值RSS未测。CPU0 noop 22.324ms已明显波动且parse/stringify前档净时间小或启动占比>=50%，依冻结规则本控制组不报纯吞吐/扣启动缩放结论。仅总耗时可复核，不能把受噪声污染的净值作算法复杂度证据。

证据：`fixed-cpu0-same-path.json`、`fixed-cpu0-new-features.json`，以及 `affinity_control.py`。复现时在前述两个harness命令加 `--cpu 0` 并用新的输出文件名；默认不传--cpu保持原未绑核行为。此方法引用已在测前节记录的微软官方继承说明，未改系统设置。

## 修复版默认亲和性测量与限制（较早窗口）


运行时实现的是架构师批准的while空环境优化：循环体没有直接声明时，避免每轮空Env；源码语义与作用域验证另见runtime/verifier报告。未改变release profile，不绑核。历史路径/文件名和CPU读数显示不稳定现象，但微观LLVM/环境根因仍未知；此处只报告测得效果。

本轮第一次准备检查使用Python -c时cwd解析异常而失败，但PowerShell继续执行了测量。故 `fixed-same-path.json` 和 `fixed-new-features.json` 仅作为诊断批，不能声称经过空闲窗口检查。此失误已报告。随后readiness.py以__file__固定项目根，正确记录GetSystemTimes的0.5秒系统CPU约9.55%、旧noop预热2+7中位数13.232ms，低于原16.307ms上限，才进入下表正式窗口。单次就绪探针不能保证后续全程环境稳定。

正式19项同路径、换bytes后预热2/正式7、每次启动核hash；与原冻结基线相同源码/输入/期望值，copy和hash在计时外。所有原阈未变。下表来自 `fixed-idle-same-path.json`，精确值与每轮输出保存在JSON。

| 工作负载 | N | 同路径旧版ms | 修复版ms | 原固定阈ms | 判定 |
| -- | -- | -- | -- | -- | -- |
| noop | 0 | 11.307 | 13.318 | 16.307 | PASS |
| numeric_loop | 10000 | 13.510 | 14.213 | 17.968 | PASS |
| numeric_loop | 100000 | 30.117 | 27.161 | 38.778 | PASS |
| numeric_loop | 500000 | 105.462 | 91.254 | 131.497 | PASS |
| array_builtins | 1000 | 18.144 | 15.682 | 17.210 | PASS |
| array_builtins | 10000 | 21.957 | 20.557 | 22.772 | PASS |
| array_builtins | 50000 | 44.184 | 45.751 | 52.306 | PASS |
| struct_ops | 2000 | 20.253 | 16.510 | 22.445 | PASS |
| struct_ops | 4000 | 34.643 | 17.953 | 37.870 | PASS |
| struct_ops | 8000 | 94.963 | 22.746 | 107.352 | PASS |
| string_ops | 2000 | 35.494 | 32.387 | 28.902 | 环境不确定 |
| string_ops | 4000 | 84.378 | 78.922 | 101.904 | PASS |
| string_ops | 8000 | 264.786 | 271.104 | 236.353 | 环境不确定 |
| string_map | 2000 | 23.895 | 24.775 | 17.522 | 环境不确定 |
| string_map | 4000 | 27.308 | 22.611 | 23.063 | PASS |
| string_map | 8000 | 27.040 | 26.513 | 22.521 | 环境不确定 |
| struct_small | 4 | 22.516 | 22.611 | 18.746 | 环境不确定 |
| struct_small | 16 | 40.278 | 41.761 | 34.634 | 环境不确定 |
| struct_small | 64 | 77.834 | 77.128 | 78.311 | PASS |

13项在原固定线内；6项超线均有同期旧版也超线，按冻结规则不能通过。数值循环100k/500k及三档大对象都达标，结构体8k也满足至少30%降时目标；小对象4/16键环境不确定，64键达标。startup新13.318ms在16.307ms线内。真实交付dist路径500k探针107.484ms在131.497ms线内，限定该探针而不保证所有路径/时段稳定。

JSON/CLI随后在相同subject路径重新执行，并增加每次启动前SHA检查与每轮hash字段。下表最新hashchecked数据比前一批更完整，旧批不覆盖。

| 工作负载 | N | 输入字节 | 含启动总中位数ms |
| -- | -- | -- | -- |
| json_parse | 2000 | 23781 | 24.867 |
| json_stringify | 2000 | 23781 | 16.450 |
| json_parse | 4000 | 49781 | 38.109 |
| json_stringify | 4000 | 49781 | 19.494 |
| json_parse | 8000 | 101781 | 66.919 |
| json_stringify | 8000 | 101781 | 25.828 |
| cli_eval | 0 | — | 12.317 |
| cli_stdin | 0 | — | 12.738 |
| cli_argv | 0 | — | 12.895 |

JSON每进程20次；parse近似扣启动net=11.549/24.791/53.601ms，翻倍比2.15/2.16，未触发3.0排查线。启动扣除是跨组近似而非函数内计时，不将3档当作渐近证明。stringify三档startup占比分别约81%/68%/52%，依冻结规则不给纯吞吐缩放结论。未采峰值RSS，只报告输入/输出字节。

可复现命令（输出必须为新路径）：

```powershell
D:/app/ANACONDA/python.exe -B benchmarks/t12/readiness.py --output Temp/t12-perf/next-readiness.json
D:/app/ANACONDA/python.exe -B benchmarks/t12/final_compare.py --new-binary dist/lfz.exe --expected-new-sha 7641b1ba98551d8726918b320d18501d62e30faa5c8c5c08fe2dc789a4748d81 --output Temp/t12-perf/next-same-path.json
D:/app/ANACONDA/python.exe -B benchmarks/t12/new_features.py --lfz Temp/T12/perf-subject.exe --output Temp/t12-perf/next-new-features.json
```

须在readiness成功后才单独执行后两条，不能用失败后仍继续的无条件shell串行命令。此次已择窗一次，未为追求通过而无限重复测量；后续需team-lead安排稳定静默窗口，再按原门槛测量受环境影响项目，不能据同期旧版较慢来抬高绝对阈值。

证据：`fixed-readiness.json`、`fixed-same-path.json`（诊断）、`fixed-new-features.json`（诊断）、`fixed-idle-same-path.json`、`fixed-idle-new-features.json`、`fixed-hashchecked-new-features.json`。原始FB8源码/二进制还由runtime归档到 `docs/reports/T12-runtime-evidence/perf-diagnostic/initial-t12-snapshot.zip`，其报告保全固定CPU/候选对比；本报告不篡改或替代这些原始证据。

## 历史记录开始

# 历史阶段一：实现前基线

测量日期：2026-10-25。本历史阶段当时仅有实现前数据，尚未进行优化测量。历史 `performance.md` 保留原日期、机器和版本结论，本报告不替换历史测量。

## 方法与复现

仅 Python 标准库，`time.perf_counter_ns` 计整进程 wall time，包含进程启动、源码加载、词法/语法、求值、输出捕获与退出。每个单元预热2次、正式7次；原始逐轮值、源码、命令、输出、SHA256和环境保存在 JSON。每轮要求 exit 0 且输出与 Python 独立计算的精确校验值相同，否则终止，不收纳无效轮。每进程超时15秒。依次运行，各规模2k/4k/8k有界，不做崩溃压测。

```powershell
D:/app/ANACONDA/python.exe -B benchmarks/t12/run.py --lfz Temp/t12-perf/lfz-before.exe --label T12-before --output Temp/t12-perf/recheck-before.json
D:/app/ANACONDA/python.exe -B benchmarks/t12/run.py --lfz Temp/t12-perf/lfz-before.exe --label T12-before-small --suite small --output Temp/t12-perf/recheck-before-small.json
```

输出路径必须是项目内新路径（运行器拒绝覆盖）；所有生成夹具和临时文件在 `Temp/t12-perf/`。旧exe已保全该目录；其原始相同字节也在v2正式结果原始输入ZIP中可恢复。

- 旧版exe：766464 bytes，SHA256 `149b184e442ffbf250590431fa0aeffab401eac526faf60b4b435f620725d5ab`。
- 环境完整字段：见 `benchmarks/t12/results/before.json`。Windows，CPython 3.13.9，24逻辑CPU；未绑核或控制后台负载，不能把小幅差异视为稳定提升。
- 不作本轮LFZ/Python速度对比：Python只提供独立期望值，未计时对照解释器。
- 首次小对象探针误用了LFZ不支持的单分号，退出2后已修正换行；该失败没有计入正式数据。

## 实现前数据

以下均为中位数，单位ms，含进程启动；min/max与七轮原始输出见两个JSON。

| 工作负载 | 规模 | 中位数ms |
| -- | -- | -- |
| noop | 空程序+固定print | 11.307 |
| numeric_loop | 10000 / 100000 / 500000 | 12.968 / 32.315 / 109.581 |
| array_builtins | 1000 / 10000 / 50000 | 12.210 / 17.772 / 43.588 |
| struct_ops | 2000 / 4000 / 8000键 | 17.445 / 31.559 / 89.460 |
| string_ops（循环push） | 2000 / 4000 / 8000项 | 23.902 / 84.920 / 196.961 |
| string_map（相同结果，range/map构造） | 2000 / 4000 / 8000项 | 12.522 / 18.063 / 17.521 |
| struct_small | 4 / 16 / 64键，每种重复1000对象 | 13.746 / 28.862 / 65.259 |

19个测量单元共133次正式运行，全部输出一致。小样本中的string_map中位数不随规模严格递增，说明启动与调度噪声不可忽略；不据此给出纯计算吞吐或精确渐近复杂度。

## 热点建议与证据边界

1. **优先结构体字段索引**。源码 `src/value.rs` 的 `StructObj.get/set/contains` 实际逐项扫描 `Vec<(String,Value)>`，不是历史报告中的“疑似”。上述构造+读规模上升与此一致。建议由runtime-dev增加键到slot的HashMap索引，保留fields插入序、重复原始字段first-match行为、别名语义、方法过滤与keys字节序。对JSON大对象也可避免逐字段重复查找。现阶段仅建议，尚未测得优化结果。
2. **应用构造改为map或预分配**。当前push按返回新数组契约复制整个顶层Vec；循环构造具有累计拷贝。8000项相同最终处理的196.961ms与17.521ms说明写法影响明显。这是程序写法对比，不是新解释器加速，也不是一般任务的固定倍数。
3. `BUILTIN_NAMES`及普通表当前线性查找；69项表的收益需专门测量，不将查找改造预先宣称为主要瓶颈。
4. 历史“用Rc唯一时原地push”的建议不能直接照搬：现有A1别名和push返回新容器的语义必须保持；本轮优先改示例，暂不以COW替代契约。

## 后续复测

新release exe就绪后，相同运行器复测19个旧单元，记录二进制hash并报告中小对象回归。再按冻结JSON契约增加JSON parse/stringify吞吐与规模、`-e`/stdin/argv入口总开销测量。旧实现没有JSON，不能虚构JSON前后提升。新的测量结果单独留存，不覆盖before原始数据。
## 阶段二测前冻结（任何新版计时前落盘，2026-10-25）

本节规则冻结后不因结果改线。目标新版SHA256 `fb8e5653827de1e1c3f73b81d37f21314ad41adfdc4d99f37cb8ca5f6491f47d`，821248B。仍预热2+正式7、同进程口径和15秒超时。

- 旧19单元逐项接受线：新版中位数 <= max(旧版原中位数×1.20, 旧版原中位数+5ms)。noop单独宽限为max(旧×1.30,旧+5ms)。small4/16/64均按普通线，不排除小对象。
- 任一超线立即向team-lead报告；保留失败证据，复查最多一次，旧新版交错、同样预热2+正式7。复查仍以冻结原阈值裁定，不能根据新结果放宽；若同期旧版也超原参考范围，标记环境不确定、暂不通过，另择空闲窗口复测。修复后新hash另留新文件，受影响组重测。
- 大对象收益目标：struct_ops 8000键新版中位数 <= 旧值×0.70，即至少30%整进程耗时下降。这是该样本的目标，不是所有任务都快30%。
- JSON无旧版对照：固定2000/4000/8000个ASCII键整数值对象，紧凑UTF8输入；分别parse、stringify各20次/进程。输入字节数、输出长度由Python独立oracle校验。另测-e/stdin/argv固定微脚本，保持相同期望stdout；报告含启动总耗时，无对应旧入口速度提升。
- JSON缩放采用各规模总中位数减新版noop中位数的近似net；若net<=5ms或startup占总数>=50%，只报总耗时和启动占比，不给纯吞吐。足够可测时，每次翻倍net比值<=3.0作为未见严重超线性信号；超线即报告排查，不把3档数据当作复杂度证明。
- 内存仅记录输入/输出字节；当前harness不采跨平台子进程峰值RSS，不宣称峰值内存或内存改善。
## 阶段二首版结果：未通过性能回归门槛

测前规则以上节为准。目标hash `fb8e...f47d` 的新测量已保存 `after.json`、`after-small.json`、`new-features.json`；判定表 `comparison.json`。19个旧单元中16项过线、3项超线，输出全部正确。

| 超阈单元 | 原旧版ms | 初测新版ms | 冻结合格线ms | 交错复查旧版ms | 交错复查新版ms |
| -- | -- | -- | -- | -- | -- |
| numeric_loop 100000 | 32.315 | 46.778 | 38.778 | 28.042 | 45.184 |
| numeric_loop 500000 | 109.581 | 184.662 | 131.497 | 97.865 | 185.514 |
| string_ops 2000 | 23.902 | 34.801 | 28.902 | 28.443 | 29.037 |

按冻结规则仅复查一次，旧新版交错并交替先后，每版预热2/正式7，原始值与输出在 `recheck.json`。三项复查仍超原线；同期旧版都在范围内，因此不能把回归归为环境噪声。已即时交team-lead，由runtime-dev修复；当前首版性能验收结论为不通过，不能只展示索引收益隐瞒循环回归。

其余主要数据：struct_ops 2k/4k/8k为12.081/13.872/25.209ms；8k相对89.460ms下降约71.8%，满足该单元至少30%目标。small4/16/64键重复1000对象15.459/24.404/58.418ms，均过线。收益仅限定当前样本和hash，最终修复后另测。

### JSON与新CLI入口（首版，不与旧版虚构提升）

JSON输入是对象，ASCII键和值为整数，Python独立构造排序紧凑UTF8文本。每进程重复操作20次，每单元预热2/正式7，输出为独立计算的字段值累加或序列化长度累加，全部一致。计时包含文件读入、stringify组一次parse准备、进程启动；不称为孤立JSON函数精确耗时。

| 键数 | 输入/单次编码字节数 | parse总ms | stringify总ms |
| -- | -- | -- | -- |
| 2000 | 23781 | 22.106 | 13.048 |
| 4000 | 49781 | 32.590 | 18.927 |
| 8000 | 101781 | 58.843 | 21.531 |

新版noop中位数9.772ms，近似扣除后的parse为12.334/22.818/49.071ms，翻倍比约1.85/2.15，未触发3.0检查线，但3档并不证明复杂度。stringify前两档启动占比分别约75%/52%，依冻结规则不给纯吞吐或近似net缩放结论。最后档启动仍约45%，不能忽略。

-e/stdin/argv微脚本总中位数9.640/9.074/9.508ms。argv包含中文、空串和字面--json/-e参数；结果与Python JSON期望一致。入口开销处于启动噪声尺度，不宣称谁更快。未采峰值RSS，仅有输入输出字节。

### 复现新功能组

```powershell
D:/app/ANACONDA/python.exe -B benchmarks/t12/new_features.py --lfz dist/lfz.exe --output Temp/t12-perf/new-features-recheck.json
```

最终修复hash出现后重跑受影响组并另存JSON，原有首版失败证据保留。测前阈值不变；如binary路径内容改变，必须以每次记录的hash辨别结果，不能把首版结果冒充最终版本。

## 最终同执行路径复测：效率门槛仍未通过

runtime进一步诊断曾观察同字节exe在不同路径/文件名时产生显著耗时差异，但机制未知。首轮旧版位于Temp、新版位于dist，未控制完整执行路径，这限制了把超阈直接归因源码的结论。初轮失败及全部原始值保留，不删除或重算。不能据此声称已找到缓存缺陷，也不能简单归因噪声。

在team-lead明确要求下，新增 `final_compare.py`：固定共同执行路径 `Temp/T12/perf-subject.exe`，旧新版源码exe均只读，依次copy到同一路径；每次换字节后预热2次，再正式7次。每次启动前计算SHA256确认bytes；copy及hash耗时不计入进程运行时间。各case旧/新组次序交替。全部旧19项仍用原先冻结阈值，没有放宽。JSON/CLI组也用相同subject路径重测。工具环境未控制后台负载，峰值内存未测。

**结论：本轮最终效率验收仍未通过。** 19项16项过线，numeric100k/500k仍超阈，string_ops2k也超阈；后者同期旧版也超原参考范围，须标记环境不确定。数值循环同期旧版在原范围内，同路径仍存在新旧差异，需继续诊断而不能声称路径因素已解决。

| 工作负载 | N | 同路径旧版ms | 同路径新版ms | 原固定阈ms | 判定 |
| -- | -- | -- | -- | -- | -- |
| noop | 0 | 10.817 | 11.165 | 16.307 | PASS |
| numeric_loop | 10000 | 13.822 | 15.328 | 17.968 | PASS |
| numeric_loop | 100000 | 31.301 | 47.374 | 38.778 | 超阈 |
| numeric_loop | 500000 | 104.259 | 189.340 | 131.497 | 超阈 |
| array_builtins | 1000 | 10.034 | 10.722 | 17.210 | PASS |
| array_builtins | 10000 | 15.764 | 15.337 | 22.772 | PASS |
| array_builtins | 50000 | 37.784 | 36.730 | 52.306 | PASS |
| struct_ops | 2000 | 15.314 | 12.005 | 22.445 | PASS |
| struct_ops | 4000 | 35.438 | 13.989 | 37.870 | PASS |
| struct_ops | 8000 | 99.979 | 21.932 | 107.352 | PASS |
| string_ops | 2000 | 31.588 | 33.869 | 28.902 | 超阈 |
| string_ops | 4000 | 82.119 | 64.726 | 101.904 | PASS |
| string_ops | 8000 | 180.521 | 196.353 | 236.353 | PASS |
| string_map | 2000 | 11.454 | 11.331 | 17.522 | PASS |
| string_map | 4000 | 17.096 | 13.189 | 23.063 | PASS |
| string_map | 8000 | 16.788 | 16.984 | 22.521 | PASS |
| struct_small | 4 | 13.267 | 13.991 | 18.746 | PASS |
| struct_small | 16 | 26.466 | 23.720 | 34.634 | PASS |
| struct_small | 64 | 68.690 | 63.110 | 78.311 | PASS |

结构体8000键同路径99.979→21.932ms，约78.1%降时，满足该样本至少30%目标；三档小对象均过线。这是明确的局部收益，不能抵消其它负载未过门槛。push语义未改变，map构造更快仍是程序写法效果。

真实交付路径 `dist/lfz.exe` 的500k循环单独预热2/正式7探针中位数171.409ms，也高于原131.497ms阈值；不能把另一次诊断观察到约99ms直接移植为当前交付路径稳定性能。该环境/路径相关现象仍开放，报告不承诺target/dist所有执行路径性能一致。最终与首版均为相同hash `fb8e5653827de1e1c3f73b81d37f21314ad41adfdc4d99f37cb8ca5f6491f47d`，没有用新构建冒充已修复。

### 最终JSON/入口组

以下均在共同subject路径，预热2+正式7且独立oracle输出一致。JSON每进程20次操作，输入字节和算法与首版完全相同。

| 工作负载 | N | 输入字节 | 总中位数ms |
| -- | -- | -- | -- |
| json_parse | 2000 | 23781 | 24.722 |
| json_stringify | 2000 | 23781 | 13.325 |
| json_parse | 4000 | 49781 | 34.471 |
| json_stringify | 4000 | 49781 | 15.952 |
| json_parse | 8000 | 101781 | 58.489 |
| json_stringify | 8000 | 101781 | 23.576 |
| cli_eval | 0 | — | 13.973 |
| cli_stdin | 0 | — | 9.635 |
| cli_argv | 0 | — | 9.508 |

同路径新版noop=11.165ms；parse近似net=13.556/23.305/47.323ms，翻倍比1.72/2.03，未触发3.0排查线，但不是复杂度证明。stringify前两档仍有>=50%启动占比，不报纯吞吐缩放；不采峰值RSS。

### 最终证据与复现

- `benchmarks/t12/results/final-same-path.json`：19旧负载×旧新×7正式值，共266次，另dist探针7次；全部精确输出一致，记录每轮hash、命令输入、源码、原固定阈值与判定。
- `benchmarks/t12/results/final-new-features.json`：9单元×7次=63次正式值，全部输出一致。
- 原始`before/after/recheck/comparison/new-features` JSON全部保留，失败信息不被最终数据覆盖。

```powershell
D:/app/ANACONDA/python.exe -B benchmarks/t12/final_compare.py --new-binary Temp/T12/lfz-fb8-before-while.exe --expected-new-sha fb8e5653827de1e1c3f73b81d37f21314ad41adfdc4d99f37cb8ca5f6491f47d --output Temp/t12-perf/final-independent-recheck.json
D:/app/ANACONDA/python.exe -B benchmarks/t12/new_features.py --lfz Temp/T12/perf-subject.exe --output Temp/t12-perf/final-new-independent-recheck.json
```

第一条结束会在subject保留新版，第二条因此测新版；独立复跑时须避免其它进程改写subject。同路径脚本为旧新hash设固定校验，不匹配就拒绝；后续解释器变化必须新增版本化测量，不篡改本轮结果。

后续工作：运行时/环境联合诊断数值循环超阈来源，确认后按原阈复测受影响组；在此之前，R304不能仅凭结构体收益判为整体效率通过。

## 修复版复测准备（尚未开始新hash计时）

runtime候选为while体无直接Decl/FnDecl/StructDecl时复用外层作用域、避免每轮空Env；契约及语义验收由architect/runtime/verifier负责。当前不预先声称优化成功或解释根因；LLVM布局/路径/CPU现象均不能凭读数虚构微观机制。

`final_compare.py`已改为显式 `--new-binary` 和 `--expected-new-sha`，旧hash149b固定不变，每次启动仍校验实际bytes。既有fb8原始JSON不修改，其exe保全 `Temp/T12/lfz-fb8-before-while.exe`；上面的历史复现命令已补参数指向该副本。最终修复版使用新文件名另存结果，19原固定阈及struct至少30%降时目标原样不动。若新版超阈且同期旧版也超原参考，则按既定规则标环境不确定，再择空闲窗口；不把当时旧版更慢作为放宽绝对线依据。

脚本准备仅执行--help确认参数解析成功，没有运行候选新版；须收到team-lead最终hash及构建/测试结束通知后才开始19项、JSON及CLI全组新计时。
## CPU0控制实验：测前固定方法（尚未计时）

team-lead预先指定CPU0，所有既往raw保留、原19阈及struct30%目标完全不变。harness在创建任何受测子进程前，通过Windows GetProcessAffinityMask保存自身mask，再SetProcessAffinityMask为1（logical CPU0）；子进程从创建起继承，避免启动后改mask的竞争。每个harness finally恢复原mask并核验，结果JSON记录原/active/restored mask。只影响本次Python与其子进程，不改系统设置。

此法用于控制跨核迁移/核心类别差异，仍不能控制全部系统噪声；原参考值来自未绑核环境，故单核控制下满足原线是更明确的有限实验结论，不能冒称默认部署环境恒定性能。微软官方明确子进程继承进程亲和性，见[SetProcessAffinityMask](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-setprocessaffinitymask)。

执行方案：--cpu 0，旧新同subject路径，每次换bytes后预热2+7；19组和9JSON/CLI各一次正式测量；若超线最多原规则一次复查。若旧版同期仍大面积超参考，判环境受限，不反复挑最快窗口。不改profile，不改解释器binary，不把控制变量测量当源码修复。
## 静默/冷却控制窗口：测前固定方法

本窗口由team-lead安排，所有其它agent已收工，root只睡眠/简短消息。不挑选最快读数；原19阈值及struct30%目标不变。恢复未绑核（与原baseline环境形式一致），开始前静默90秒；每次共同subject换字节后固定静默5秒，再预热2+正式7。19组顺序不变、各组旧新次序照原交替，完整保留所有值。初始90秒后先查系统CPU及旧noop，若不达原准备线就记录not-ready并停止，不强行进入正式组。

新方法仅为控制文件写入/加载及后台活动，不预先假定其成因或声称能消除干扰。新增GetProcessTimes逐进程user/kernel CPU时间与wall并列；短进程CPU计数可能量化为0，不能用0推断没有运算。进程创建至退出wall仍为主原阈口径；CPU计数不另立合格线。峰值RSS仍不采。

执行一次固定静默窗口，必要最多遵守原规则复查，不无限重试追通过。所有先前default/CPU0失败原始值保留。JSON/CLI同最终hash重测，不与旧版虚构不存在功能的前后提升。
## opt2正式主产物全组：测前固定方法（一次）

team-lead确认release-manager按批准profile opt-level=2构建，1.4.0、759808B、SHA256 `47ed2bfe2a2694b11bd611172cfcf0000d0bfc58e3b9bd809ca95f066dd0261f`，构建/测试已停，其它agent暂不运行CPU任务。该profile变更属于本次产物条件，不能把全部收益单独归因while优化。

本次只测该正式hash一轮完整19旧组和9JSON/CLI组：默认未绑核、组顺序及旧新交替次序不变，每copy共同subject后固定等5秒，再2预热+7正式；不额外90秒、不循环选窗。原19绝对阈、struct至少30%目标完全不变，同时报告同期旧版是否在线及paired比值。所有旧原始数据完整保留，新文件用opt2-*。若仍有未过项，R304保持开放，不能以主循环或包体缩小抵消；不无限继续试profile/择窗。