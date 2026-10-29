# T14 check 命令冻结设计

日期：2026-10-27。责任人：language-architect。来源：team-lead按用户连续自动推进授权选择下一功能批次。状态：经team-lead批准，2026-10-27 19:33已冻结到三spec §17；本阶段不实现，不混入T13计时候选。

## 目的与保证

新增 `lfz check`，只对一个源码单元执行「加载 → 词法 → 语法及既有AST结构深度检查」，不进入LFZ求值器。成功只说明该输入通过现有前端检查，不能解释为脚本能成功运行、文件IO会成功、名称存在或类型正确。它让Agent在生成脚本后先检错，避免为验证语法触发脚本里的输出、文件写入、交互输入或无限循环。

不新增LFZ关键词/语法/内置，源码内既有 `check(condition)` 断言内置完全不变；CLI子命令与LFZ标识符处于不同层次。

## 命令形式与兼容

接受单输入，选项 `--json` 可在子命令前后、源操作数外出现，重复出现幂等：

```text
lfz check FILE
lfz check -e CODE
lfz check -
lfz --json check FILE
lfz check --json -e CODE
lfz check -e CODE --json
lfz check --json -
```

- `FILE` 接受任意路径，等同显式 `run FILE` 的加载扩展名口径；不用裸文件入口的 `.lfz` 扩展名限制。缺源、多源、未知选项、`-e`缺CODE一律CLI参数错误、exit2。
- CODE紧随 `-e` 的一个原始argv字符串，其内容即使恰为 `--json`、`--`、`check` 或空串也必须完整保留，不能再作为选项解析。
- 本轮check无脚本参数，任何CODE外的 `--`（包括空尾、`-- ARG...`）均拒绝，避免暗示调用argv或执行。CODE内部的 `--` 不受此规则影响。
- `check --help`按未知选项拒绝；帮助只通过 `lfz --help` 文档化check，不顺带扩展旧help解析语义。
- 不能把裸 `lfz check` 再当文件执行；此前无扩展名裸文件本来不会成功运行，因此不破坏已支持的成功入口。名为 `check` 的文件仍能用 `lfz run check` 运行、`lfz check ./check` 检查。`lfz run check.lfz`、旧 `test`、裸脚本、-e/stdin执行及argv解析行为全部保留。
- 带前导 `-` 的实际路径建议使用 `./-name` 或完整路径；本轮不另增通用 `--` 路径终止规则，不改变既有选项分隔的含义。

现 `src/cli.rs` 的 `split_options` 只把顶层和 `run -e` 的下一个字符串识别为CODE，必须按相同方式补上精确的 `check -e`，不能全局删除CODE内的 `--json`。`execution_ready` 的单词通用分支目前会把check当裸文件；新命令须在精确分支先识别并拒绝check脚本参数，而不能借用执行入口的脚本参数行为。应增独立Check来源枚举/模式，不让新模式误落到run/eval函数。

## 加载、解析与非执行边界

1. 文件用既有 `loader::load_file`：先全文件UTF-8检查，跳首个BOM、CRLF/CR归一；扩展名按路径末段ASCII不分大小写判断。`.lfz`/`.LFZ`必须有精确 `#42` 换行前导，消费后 `line_base=1`；其它扩展名不要求也不消费前导。
2. `-e CODE` 用 `loader::load_source`，不要求、不消费 `#42`，`line_base=0`。stdin读到EOF并检查UTF-8，再用同一load_source。stdin读取失败/非UTF8沿用现 `run_source` 的 `IOError: 无法读取标准输入`（无位置）口径，不在本轮另改错误分类。
3. 调用同一 `lexer::lex(text,line_base)` 与 `parser::parse(tokens)`。后者已经包含AST深度检查；不能另复制一套文法或绕过解析上下文。顶层return、顶层break/continue、管道占位符越过lambda、非法lvalue等，取决于现有parser已实现的上下文检查。
4. 此命令不额外做名字解析、重复定义/遮蔽诊断、类型推断、函数arity检查、条件bool检查、let重绑定检查、方法self验证、用户数据JSON解析、IO路径检查、可达性或终止性分析。`unknownName`、`1 + true`、`print(1/0)`、`assert(false)`、`let x=1`后`x=2`、`readFile("不存在的文件")`均可语法通过；语法有效并不保证运行成功。
5. 所有函数体/分支/循环体仍要完整解析，即使从运行角度不可达也不能藏语法错。只能承诺现有parser上下文验证，不能宣传为完整静态语义分析；不趁此强化或修改现有作用域规则。
6. 加载、lex、parse、AST检查与Program析构都必须处于现有 `evaluator::on_eval_stack` 大栈线程机制内（可复用这个通用栈执行包装器）；调用此线程包装器本身不是LFZ求值。成功后就地drop Program再返回标量结果，不把深AST带回主线程析构。禁止调用 `eval_module*`、`run`、`Interp::new` 或执行内置；不需要设置程序输出重定向开关。
7. 读取待检查源码文件或stdin属于宿主加载行为；保证是「不执行LFZ程序及其文件IO」，不能宣传整个命令零IO或完整安全沙箱。读取特殊设备/FIFO或未结束stdin可能等待，大小限制也沿用既有实现，本轮不宣称通用资源隔离。

## 输出、退出码与JSON定案

正常模式通过：stdout、stderr均空，exit0。失败：沿用原错误人类渲染写stderr，stdout空，exit2。帮助中明确「通过语法检查不代表运行成功」。不使用exit1（该码仍为test失败专用）。

对已识别且参数合法的check命令，`--json` stdout恰一行JSON；不产生LFZ程序stdout/stderr。使用新的check专属外层标记精确区别执行与检查，复用现有错误字段构造/转义编码：

```json
{"ok":true,"mode":"check","executed":false}
```

失败对象键序固定为 `ok`、`mode`、`executed`，然后复用现 `push_error_fields` 产生的 `error`、`message`、`file`、`line`、`col`、`traceback`。file使用原路径、`<command>`或`<stdin>`；既有run/test JSON完全不加新键、不改语义。

前端失败不构造TracedRun。复用原错误JSON时，带位置的前端错误会得到已有的单个 `<module>` 源位置条目；这仅是定位记录，不能称实际执行栈，`executed:false`保持明确。无位置加载IO错误仍line/col为null、traceback为空。

CLI参数错误沿用现execute错误分支：stderr用法消息、stdout空、exit2，即使出现 `--json` 也不冒充LFZ错误对象；因此「唯一JSON」承诺限于已识别的合法check命令。本轮不定义CLI参数错误JSON；未来如需扩展，应另定义协议，不能复用不相符的LFZ错误类或更改旧run行为。

## 建议实现分工与验收矩阵

- language-architect：经team-lead定案后，已将命令/结果保证正式落到syntax/semantics/interface-contract §17；该三节为实现冻结契约。
- tooling-dev：唯一实现CLI来源解析、前端模式与JSON包装，复用loader/lexer/parser和大栈；不改核心语法。core-dev仅在发现既有parser缺陷并经另派时介入。
- test-engineer/verifier：过程级黑盒、源码调用图审查及非执行见证；指南由docs/AI角色同步。

| 类别 | 见证 | 期望 |
| -- | -- | -- |
| 三来源 | 文件、-e、stdin同一源码，普通/JSON | 均exit0，JSON mode=check/executed=false |
| 参数保护 | CODE为空、恰为--json、含--、Unicode；--json前后/重复 | 不摘除或拼接CODE；只按源码语法判断 |
| 参数负例 | 缺源、缺CODE、多源、未知option、CODE外--、-- ARG | exit2；无任何执行/写入 |
| 加载规则 | .lfz/.LFZ前导存在/缺失/尾空白、非lfz豁免、BOM/CRLF/非UTF8、缺文件 | 同run前端错误类/消息/位置，行偏移一致 |
| 语法负例 | 顶层return/break/continue、非法管道占位符、函数体内语法错 | 与现parser同结果，不执行之前的语句 |
| 非静态承诺 | 未定义名、类型错、arity错、除零、let重绑定、missing readFile | 只要语法有效就exit0，无运行期错误 |
| 无副作用 | 源码有print/eprint/;;/input/writeFile/appendFile/seed | 无程序输出，无提示，无目标文件创建或修改；已有文件字节保持 |
| 不执行循环 | while true空循环；函数递归入口 | bounded超时内检查返回，不执行循环/调用 |
| 先有副作用后有语法错 | 先writeFile/print后非法token/语法 | 前端exit2且目标不变、无程序输出 |
| 深度与析构 | parser合法边界、超一层、长左偏AST合法/超界 | 沿既有限制受控返回，无主线程AST析构溢出 |
| JSON隔离 | 语法错/加载错/成功且源码含print/dump | stdout一个对象；mode/check和executed/false一致；旧run JSON不变 |
| 兼容 | 既有CLI91、argv保护、runner166、正常执行check内置 | 旧行为全保持，命令层不污染语言名字 |

深度压力不得通过真实panic/abort证明；沿既有受控边界测试并遵守项目崩溃纪律。最终非执行证据必须同时包括实际进程负例（超时/文件字节/输出）和源码路径审阅，不能只凭一个字符串搜索断言。

## 未纳入与推进顺序

不增加多文件批量check、watch、格式化、AST输出、静态类型/未定义名/危险IO审查、自动执行修复、脚本参数或新错误类。T13最终候选与性能数据先冻结，T14另开新源码hash和证据目录；不能用新增check后的二进制冒充T13性能产物。

## 冻结回执

team-lead已明确批准：check --help拒绝、CODE外任何--拒绝、usage错误保持stderr文本/stdout空、合法check JSON成功恰三键、失败接原错误字段。三spec §17已同步；新增ADR时间2026-10-27 19:33。结构自查覆盖三来源、CODE保护、usage例外、非执行/非静态边界、大栈析构与旧协议兼容；这不是产品测试结果。
