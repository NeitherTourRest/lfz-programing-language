# T12 黑盒测试与进程入口验证

日期：2026-10-25；角色：test-engineer。本报告提供测试证据，最终独立验收由 verifier 执行。

## 结果

- release解释器1.4.0，SHA256 `fb8e5653827de1e1c3f73b81d37f21314ad41adfdc4d99f37cb8ca5f6491f47d`。
- 正式一键黑盒 **166/166 PASS，0 FAIL，0 ERROR，exit 0**。保留原130项，新增4组LFZ正例（64条assert）及32个错误类负例。
- 独立Python进程检查 **91/91 PASS，exit 0**，其中1项为上述正式全量runner调用；不将重叠执行虚增为新的独立语言用例。
- `tests/cases.json` 原99个清单条目经与Git HEAD解码值逐项比较全部原样保留；只追加32条。原有LFZ测试未修改。

## 可复现命令

在项目根目录执行（输出文件必须是项目内尚不存在的新路径）：

```powershell
D:/app/ANACONDA/python.exe -B tests/t12_cli_check.py --lfz target/release/lfz.exe --output Temp/T12/test-engineer/recheck.json --full-suite
```

一键正式测试：`target/release/lfz.exe test --json`。Python只使用标准库，所有临时输入、环境TEMP/TMP及输出都限制项目内，进程超时10秒。JSON错误输入通过UTF-8文件传入，包含NUL时不尝试通过操作系统参数传递。测试内容依据冻结spec设计，没有读取/修改解释器实现。

## 覆盖

| 领域 | LFZ正例/负例 | 进程补充验证 |
| -- | -- | -- |
| JSON映射与数值 | nil/bool/array/struct、i64两端/2^53+1、float类型/负零/下溢、排序/紧凑 | Python独立JSON参考往返，有限/超界数值与类型 |
| Unicode与转义 | 非BMP代理对、NUL、五短转义、任意字符串键、美元字面 | 全32控制字符逐字节编码对比、孤立/错误代理对及UTF8 byte offset |
| 严格文法 | 前导零/单引号/裸键/注释/尾逗号/BOM/裸控制/尾随内容 | 33种解析精确reason与字节偏移，调用点file/line/col |
| 引用与边界 | 深度128通过、129拒绝，struct/array环拒绝，共享非环通过 | 编码错误精确reason、数组函数/模板/NaN/Inf拒绝 |
| struct索引兼容 | 方法过滤不调用、原位更新/新增字段、del新对象后读写、原对象保留 | 键序稳定与别名语义 |
| argv | runner默认空、身份/type/str、别名/闭包/数组/HOF、let/形参遮蔽 | 三入口两拼写、空/Unicode/空格/美元及所有flag字面、返回数组隔离、struct保存函数调用、别名arity |
| CLI入口和通道 | 由进程层检查 | 空-e/stdin、缺参/extraargs、源码token恰--json/--/--help、选项前后、stdout单状态JSON及stderr业务输出、stdin非法UTF8/BOM/CRLF、input EOF与-e input数据、行号/伪路径、无自动print |

LFZ64条assert分布：argv12、JSON values28、Unicode16、references8。32负例置于fixtures并由清单声明错误类，不污染自动发现正例。

## 原始证据

- `T12-blackbox-evidence/release-cli.json`：91次命令、stdin原字节hex、精确期望、退出码、stdout/stderr、检查结果与解释器hash；解析错误另保存JSON输入原字节hex。
- `release-suite.json` / `release-suite.stderr.txt`：正式166项完整结果与程序输出；`summary.json`：计数与脚本/证据hash。`runner-source.txt`保存执行时脚本原字节，目录.gitattributes关闭换行转换以保全hash。
- 脚本SHA256 `e48077dfbb29e19c73bf745a281affbd4539b1a72eb49b01a479657ee4179db2`。
- release-cli.json SHA256 `f4d12a73caa1f035b66371f716016437bcf633edc503c5dc2998670c3d2620e8`。

## 作者纠错与范围

首次新正例错误地把只支持array的insert用于struct，按既有spec改为del所得新struct的字段更新后通过；原始首次运行保留于项目Temp/T12/test-engineer/first-suite.json（165/166，1条作者错误）。首次Python脚本尝试把NUL作为OS参数，Windows拒绝创建进程；修正为项目内UTF-8文件，之后所有精确输入可追溯。不将这两项记为解释器缺陷，也未放宽旧测试。

没有故意触发abort/panic，测试均正常退出；未做性能评价、未修改冻结v2包或历史成绩、未Git提交。不声称该覆盖穷尽所有输入或替代独立验收。
