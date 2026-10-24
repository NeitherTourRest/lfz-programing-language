# LFZ Agent 基准 v2 验收包

本包执行 TASKS.md 固定八题功能验收，是新基线，不是历史同题重跑。本轮没有派发八个学生 Agent。固定案例、Python参考和执行器自测均不能记为盲测成绩。

## 固定输入

cases.json 已固定：B01 16、B02 18、B03 15、B04 16、B05 16、B06 17、B07 17、B08 16，共131例。每例保存id、场景名、args、expected；B08另存expected_output精确文本。案例不在看到学生答案后改写。

| 题 | 覆盖 |
|---|---|
| B01 | 优先级/左结合、负数向下整除、连续一元、16层括号、256字符、精确大整数 |
| B02 | 空文本/空行/尾逗号、LF/CRLF、引号转义/字段内换行、Unicode、2048字符 |
| B03 | 同桶/同码点和、覆盖/删除重插、空/Unicode/NUL键、32键/128操作 |
| B04 | 上右下左固定次序、多解/绕墙/无路/单点、20×20、输入不变 |
| B05 | 精确容量、平局不选后项、重复/零价值、回溯、24项/容量80 |
| B06 | 未知账户、拒绝原子性、自转账、空/Unicode/NUL id、守恒、128事件 |
| B07 | ASCII提词后lower、K/İ/ſ分隔、重复query/位置、4096字符/32query |
| B08 | 空/空白、LF/CRLF、符号/前导零、中文空格路径、128行、逐字节/覆盖/输入不变 |

run.py 使用Python标准库独立参考核对固定期望：表达式递归解析（并以手算向量交叉核对）、csv.reader、有序BFS、二维DP（小案例穷举核对tie）、字典事件模型、ASCII正则提词和精确文件文本。它不是LFZ参考解，不能向学生提供。

## 环境与冻结

从项目根目录执行，需Python3.9+及项目内LFZ解释器；本轮使用D:/app/ANACONDA/python.exe、target/release/lfz.exe。推荐-B避免pycache。正式测评前独立审查并显式冻结：

~~~powershell
& 'D:/app/ANACONDA/python.exe' -B docs/validation/agent-benchmark-v2/run.py --freeze --lfz target/release/lfz.exe
~~~

manifest.json保存9个文件SHA256：TASKS.md、STUDENT_PROMPT.md、cases.json、run.py、skill四文件、指定解释器；不包含自身。提示词/skill/用例/运行器/解释器内容或路径变化均拒绝继续，必须先审查并按新轮次/版本显式重冻。README、QA、自测输出不参与循环哈希。不得看到学生答案后改验收条件重算正式成绩。

## 执行

学生目录为<solutions>/B01/solution.lfz至B08/solution.lfz，应遵循TASKS的项目Temp独占目录。报告--output须为项目内尚不存在的.json，保留既有结果。

~~~powershell
& 'D:/app/ANACONDA/python.exe' -B docs/validation/agent-benchmark-v2/run.py --solutions Temp/agent-benchmark-v2/example-run --lfz target/release/lfz.exe --output Temp/agent-benchmark-v2/example-run-functional.json
~~~

每题快照原始源码并hash，每case在项目Temp/benchmark-v2-run-*/Bxx/<case-id>/合并#42、函数源码与独立断言，进程cwd恒为项目根目录。专用LFZ字面量保留Unicode、不使用LFZ不支持的\u；转义$防插值，额外控制字符用chr(n)拼接。B06任意账户键以entries数组比较，避开结构体字面量动态键限制。

B01–B07每case一个进程，同进程重复solve查状态重置，B04还复查grid/start/goal内容。B08每case两进程，每次先污染输出，再外部逐字节比较UTF8输出/原始输入和返回值；第二轮保留上轮输入，再放旧输出以查重复覆盖。每进程10秒，B08每case至多20秒，另有小量文件准备开销。超时终止当前进程并记录证据。

每case含源码/案例hash，每次attempt含命令、exit、stdout/stderr及原始字节hash、耗时、超时标记、合并源码hash/路径；B08另含输入/输出期望和实际hash。原始快照、合并程序及IO夹具留Temp可追查。源代码/案例非法首行或UTF8为INVALID_SOURCE，缺solution逐case为MISSING，题FAIL，分母恒为8。缺solve/错答/非零/超时均FAIL。stdout不必为空，也不按stdout猜答案。

退出码0=八题功能全过；1=有功能失败或缺题；2=配置/冻结/自测错误。题级FUNCTIONAL_PASS仍保留source_review与protocol_review为REVIEW_REQUIRED、overall_task_status=NOT_CERTIFIED、certified_pass_tasks=null。这不是恶意源码沙箱，只运行遵守隔离协议的授权提交；越权/硬编码/外部代算须人工审查。

## 独立人工审查

独立verifier在正式结果旁另存manual-review.json，遵循manual-review.schema.json，绑定functional_report_sha256、manifest_sha256、各题submitted-snapshot源码hash。每题B01..B08恰出现一次；schema之外还须人工核验唯一性和hash一致性。缺源码hash为null，不能判PASS。

源码审查核对：自行解析、固定8碰撞桶及码点和hash、BFS/邻居次序/前驱、二维DP/tie/回溯、账本原子性、先建索引再查、真实LFZ IO，没有硬编码或外部语言/程序代算。协议审查按STUDENT_PROMPT核对隔离、模型/提示词/版本、20分钟/5次自行调用、首跑快照和过程日志；unknown不能当成功。

最终另行合并：只有功能全PASS、source_review=PASS、protocol_review=PASS且绑定hash相符才计该题PASS；缺失/FAIL/UNKNOWN不计，分母恒为8。run.py不自动认证整题或改写功能报告。本包用例/参考对学生保密；模型一旦见过则该轮协议失败。

## 执行器自测

~~~powershell
& 'D:/app/ANACONDA/python.exe' -B docs/validation/agent-benchmark-v2/run.py --self-test --lfz target/release/lfz.exe --output Temp/agent-benchmark-v2-self-test-new.json
~~~

不依赖学生程序。核对131项固定期望/约束、DP穷举与手算/Unicode witness；明确标记的常量stub验证正常断言通过、错答/缺函数/显式非零/非法首行/8题缺失/超时/跨调用状态泄漏/BFS改入参均被识别；IO控制验证exit0下错误输出/输入篡改仍FAIL。另实跑LFZ字面量字节回读、NUL键比较。stub仅是Temp中的执行器控制，不是正式成绩或题目参考解。当前证据见QA.md、QA-final-evidence.json；其他self-test*.json/QA-evidence.json为开发阶段自测记录，以QA指定最终hash为准。
