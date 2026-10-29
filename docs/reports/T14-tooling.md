# T14 非执行check CLI实现与作者验证

日期：2026-10-27。责任人：tooling-dev。契约：三spec §17、T14-check-design.md。该报告只给出作者功能证据，不代替独立验收，也不宣布整体性能通过。

## 实现

新增 `lfz check FILE`、`lfz check -e CODE`、`lfz check -`。CheckInput及检查路径与执行入口分离，复用既有loader/lexer/parser和错误JSON编码，不复制文法、不增加公开库API或新错误类。文件/源码/stdin加载、词法、语法与AST深度检查、Program显式析构均发生在 `evaluator::on_eval_stack` 包装内。检查路径不调用eval_module、Interp或内置，也不调整LFZ程序输出重定向。

普通检查通过双流空、exit0；前端错误沿原人类诊断写stderr、exit2。合法命令的机器成功为唯一一行 `{"ok":true,"mode":"check","executed":false}`；失败同前三键，再接现有error/message/file/line/col/traceback。带位置的前端错误仅复用源位置条目，不构造运行时帧。usage错误保留stderr文本/stdout空/exit2，--json也一样；check --help、CODE外任何--、多源和脚本参数均拒绝。CODE本身原样保留，即使为空、--json、--或含Unicode。

文件加载口径与显式run一致；CODE/stdin用load_source且line_base0，伪路径<command>/<stdin>。不声称名称、类型、arity或IO路径静态正确；无限循环、递归、输入/输出及文件写入语句只解析不执行。读取被检查文件/stdin仍属宿主IO，此命令不是通用资源隔离沙箱。

## 作者验证

新增 `tests/cli_t14.rs` 共12组进程集成，每个子进程5秒超时保护，夹具/子进程TEMP和TMP均在项目Temp/T14。旧test文件未修改；T13新增8项回归保留并随最终全套实跑。

| 命令/阶段 | 结果 | 证据 |
| -- | -- | -- |
| 新测试在无check旧实现运行 | 0/12，exit101，证明新命令缺失 | docs/tooling/evidence/T14/red-cli-tests.txt |
| 首次实现后 | 11/12；唯一失败为新测试位置期望不符 | docs/tooling/evidence/T14/green-cli-tests-first.txt |
| 新断言按既有run前端订正后定向测试 | 12/12，exit0 | docs/tooling/evidence/T14/green-cli-tests.txt |
| cargo test --release | 542通过，0失败/忽略；412lib+48bin+28cli+12cli_t12+12cli_t14+15runner+15unit | docs/tooling/evidence/T14/cargo-test-release.txt |
| cargo build --release | exit0 | docs/tooling/evidence/T14/build-release.txt |

位置订正依据：`#42\nlet a =\n` 在check与既有run前端同为SyntaxError“表达式未结束：行尾不能终止表达式；请用括号跨行”，line2,col8；新测试原本误以为EOF line3。只修新测试期望，未改parser或旧测试，中间失败日志保留。Unicode输入测试使用既有ASCII标识符和Unicode字符串，不扩充文法。

12组覆盖三来源/普通和机器模式；空CODE、选项形CODE、Unicode与重复JSON选项；usage拒绝；扩展名/前导/BOM/CRLF/非法UTF8/缺文件和准确位置；parser上下文及不可达语法；名称/类型等非静态承诺；print/eprint/dump/input、无限循环/递归入口无执行；writeFile/appendFile已存在文件字节保持及不存在目标不创建（包括后续出现语法错）；合法/超界解析和左偏AST受控返回；旧run状态JSON、argv字面--json与LFZ check(true)内置兼容。

## 冻结候选与边界

最终 `target/release/lfz.exe`：774656B，SHA256 `83f56a194e216862c45377f4c3971b085744a7ba2363163e1edac6af5e85ac79`。Cargo版本1.4.0候选、release opt-level2/LTO/codegen1/strip均未改变；源中两项T13生产优化已由runtime撤回，8测试保留。原T13 9cda测量与快照不回写，此产物不是T13测窗对象。

本角色未覆盖dist、未改核心/spec/旧报告/旧测试、未外部安装或Git。Rust工具链与全部临时输出均在项目内。原始日志中有环境路径canonicalize警告，不声称零warning。542功能通过不代表性能R304关闭或批准正式发布；后续交付复制、黑盒/非执行独立见证、指南及性能决定由team-lead衔接。

改动：src/cli.rs、src/main.rs说明、tests/cli_t14.rs、docs/tooling/runner-contract.md §11、docs/tooling/evidence/T14及本报告。manifest.json绑定当前源/Cargo、产物和原始日志hash。作者完成后停止所有构建/测试/源码修改。
