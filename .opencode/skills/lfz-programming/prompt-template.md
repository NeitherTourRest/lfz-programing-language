# LFZ 编程提示模板（prompt skeleton）

> 用途：把本文件 + `SKILL.md`（或 `docs/guide/ai/README.md`）一起交给一个**从未见过 LFZ** 的 Agent，让它产出可运行的 `.lfz` 程序。

---

## 模板 A：写一个新程序

```
你是一名 LFZ 程序员。只允许使用我提供的《LFZ 编程指南》中列出的语法，
严禁联想 Python / C / JS 的语法来补全（**不要凭记忆写 LFZ**；指南没写的语法
一律视为不存在，先查指南 §10 的 spec）。输出必须是**完整可运行**的 `.lfz` 文件，
不要伪代码、不要零散片段。

【任务】
<在此描述你要写的小程序，例如：读取一个整数数组，用管道 filter/map/reduce 求和并打印>

【硬性要求】
1. 每个 `.lfz` 文件第一行必须恰好是 `#42`（三字符 + 换行），否则无法运行。
2. 打印变量用 `;;`（独立一行），不要用 `;`。
3. 整除用 `div(a, b)`；`/` 恒为 float。
4. `let` 不可重绑定，需要重新赋值用 `var`。
5. 条件/逻辑运算必须是 bool（无 truthiness）。
6. 交付前逐条核对《LFZ 编程指南》的「自检清单」。
7. **按指南 §0.5先check再实跑**：先 `lfz check <file.lfz>`；成功只代表前端通过，再用 `cargo run --quiet -- run <file.lfz>` 运行，
   确认**退出码为 `0`** 再交付；若报错，按指南 §4.5「错误类 → 原因 → 修法」修正后重跑。
8. **字符串处理纪律**：不要对 `string` 取下标（`s[i]` 非法 → 先 `split("", s)`，见指南 §4-22）；
   **大 n 时不要在循环里用 `s = s + …` 或 `push` 累积**（O(n²)，用指南 §4.6 的 O(n) 写法）。
9. 当前1.4.0已实现 73 内置及 range(n)/range(lo,hi)，使用指南 §6 已列签名。字符串按 Unicode 标量计数；indexOf 最坏 O(n·m)，contains 线性扫描。文件 IO 为 UTF-8，相对路径基于进程 CWD；本项目只在项目内读写，临时件放 Temp/，父目录需存在。
10. 参数须放CLI的 -- 后，用argv()读取；JSON用jsonParse/jsonStringify，不手写拼接/解析。业务JSON正常模式输出，--json为执行状态；详见指南§6.2。
11. -e/stdin源码无需#42；stdin源码已读到EOF，不能再从同一stdin读数据。JSON两内置作高阶回调要lambda包装，不直接取其函数值。
12. 附 stdout/stderr 与真实退出码，文件 IO 标明路径及副作用。

【输出格式】
- 一个或多个 ```lfz 代码块（含 `#42` 首行）；
- 每个程序的预期 stdout。
```

---

## 模板 B：审查 / 修改已有 LFZ 代码

```
下面是 LFZ 代码。请依据《LFZ 编程指南》检查并修正，只输出修正后的完整文件。

【检查点】
- 首行是否恰好 `#42` + 换行；
- 是否误用 `;` 作分隔或打印变量（应为 `;;`）；
- `/` 是否被误当整除（应为 `div`）；
- 是否对 `let` 重绑定（应改 `var` 或只改容器内容）；
- 是否依赖容器内置就地修改（应接收返回的新值）；
- `${}` 内是否误写 `\"`（应为 `"`）；
- 是否使用了单引号字符串（LFZ 只有双引号）；
- 条件是否都是 bool；
- 是否对 `string` 取了下标（`s[i]` 应改为 `split("", s)` 后对数组索引）；
- 大循环里是否用 `s = s + …` / `push` 累积（应为指南 §4.6 的 O(n) 写法）；
- 退出码是否理解正确（`lfz run` 下 assert 失败 → `2`；`lfz test` 下用例失败 → `1`）；
- 是否按 §6.1 核对 data-last、Unicode 标量、空 fill、ord/chr、IEEE 数学与 IO 的 CWD/UTF-8 边界；
- 参数是否放在 -- 后、argv返回值修改是否不影响下一次调用；业务JSON和CLI --json是否区分；JSON是否符合i64/Unicode/128容器/重复键边界；
- 是否出现/期待了旧的编号错误码（禁止，只有类名 + 中文消息）。

【代码】
```lfz
<粘贴代码>
```
```

---

## 最小任务基线（验证指南是否可用）

```
用 LFZ 写一个函数，接收整数数组，返回其中偶数元素之和；用管道 filter / reduce 实现；打印结果。
输入 [1,2,3,4,5] → 输出 6。
```

参考解（实测通过，exit 0）：

```lfz
#42
fn sum_even(xs) {
    xs |> filter((x) => x % 2 == 0) |> reduce((acc, x) => acc + x, 0)
}
print(sum_even([1, 2, 3, 4, 5]))
```

输出：`6`

## 模板 C：Agent 数据脚本调用约定

```text
使用LFZ 1.4.0，读取argv()[0]的JSON配置，按任务完成数据处理。
若交付.lfz文件，首行必须#42；用-e执行的CODE不加该前导。
正常stdout只输出一个jsonStringify业务结果，诊断写eprint。
若调用者要求CLI --json，则业务print改走stderr，stdout仅执行状态。
参数始终放在--之后，包括JSON文本、空串、--json字面数据。
配置先jsonParse并验证，输出先完整jsonStringify后再writeFile。
报告真实命令、stdout/stderr、退出码和文件副作用。
```

## T14模板补充：分开检查和执行证据

```text
先用lfz check FILE / check -e CODE / check -检查生成源码。
合法check --json结果必须有mode:"check",executed:false；usage错误stdout空/stderr文本。
check不执行、不静态检查名称/类型/IO，不代表沙箱，也不保证运行成功。
CODE外不能传--或脚本参数；检查通过后才按任务授权运行，运行参数放--后。
分别报告两步命令、stdout/stderr、退出码和实际文件副作用。
源码check(cond,msg)是独立的非致命断言内置，不是CLI check。
```

## T15模板补充

需逐条恢复业务错误时用attempt(() => ...)，严格按指南§6.3检查ok及error["class"]。先check再run，捕获失败不回滚IO/容器副作用；资源终止不可吞。成功Result含函数/环时不能保证可JSON编码。提交时说明哪些记录成功、哪些失败，而非只报告exit0。

## T16-C本地模块补充（2026-10-30）

模块任务按SKILL.md §6.4先检查完整根图再执行；每文件#42，静态相对import/显式export、独立home globals、session缓存与namespace快照/共享边界。模板、跨源诊断、attempt无file和RustAPI迁移均引用三spec §19–21。作者新增实跑见docs/reports/T16-C-skill-evidence.md，不覆盖旧v2成绩，不宣称性能或正式发布。

## T17执行限额补充（2026-10-30）

Agent工作流：先check前端/模块图，再按任务配置--max-steps N与--timeout-ms N运行。只支持两token，等号族拒绝；CODE及--后argv保护。详情SKILL.md §6.5及三spec§22。当前13错误类、73内置，ExecutionLimitError不被attempt吞掉，业务恢复不补预算。截止时间/native阻塞边界是协作式；库取消token不等于CLI Ctrl-C处理。作者新增例证见docs/reports/T17-skill-evidence.md，旧v2不改，R304OPEN。

## T18-A工具执行补充（2026-10-30）

当前74内置/13错误类，exec需显式--allow-process，先check不执行再按授权限额run。只接受绝对.exe/独立argv，不搜索PATH或隐式shell；非零exitCode仍成功值，严格UTF8及双流合并rawbyte额度，局部错误可恢复而全局资源终止不吞，cleanup宿主异常不保证单JSON。详见SKILL.md §6.6及三spec§23；仅Windows已验、非沙箱。作者记录T18-A-skill-evidence.md，旧v2不改，R304OPEN。本轮交付后停止，不进入HTTP。

## T18-B HTTP补充（2026-10-31，最终候选）

本轮新授权HTTP，交付后收口不T19/不恢复自动化。当前75内置/13错误类，按SKILL.md §6.7和三spec§24先check再显式--allow-http限额run；不以旧A二进制冒称HTTP通过。六参三键、独立能力、重复header/4xx正常、strictUTF8、API头文本UTF8+body非wire限额、无自动跳转/cookie/auth/proxy/retry、局部恢复/全局终止与双sidecar宿主例外清晰；Windows先验非沙箱。作者先prepare-only，最终锁定18d328候选后才实跑，证据报告列实际计数，不覆盖历史准备记录。
