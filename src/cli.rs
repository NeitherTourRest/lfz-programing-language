//! 最小命令行界面（P3.10）：`lfz run <file>` + `--help` / `--version`。
//!
//! 单一事实源（本模块**只读引用**，不偏离）：
//! - `docs/spec/semantics.md` §8.2（用户可见输出格式）/ §8.3（逐字符精确示例）。
//! - `docs/spec/interface-contract.md` §8.1（错误类 ↔ 退出码）。
//! - `.opencode/team/DECISIONS.md` D-008（退出码 `0/1/2`）。
//!
//! # 职责边界（契约 §10.6）
//!
//! CLI **只做**两件事：**错误格式化**与**退出码映射**。它不定义语法 / 语义，
//! 不改解释器核心，不复制任何词法 / 语法 / 求值逻辑——只按固定顺序调用稳定库入口：
//! `loader::load_file` → `lexer::lex` → `parser::parse` → `evaluator::eval_module_traced`。
//!
//! # 本批范围
//!
//! **已做**：`lfz <file>`（裸文件调用，P4.4，等价 `lfz run <file>`）、`lfz run <file>`、
//! `lfz run --json <file>`、`lfz <file> --json`、`lfz test [路径...]`
//! （实现见 [`crate::test_runner`]）、`lfz test --json`、`--help` / `-h`、`--version` / `-V`、
//! 错误格式化、退出码、`--json` 机器可读输出（`semantics.md` §8.3 示例 4 / §8.4）。
//! **T14**：`check FILE` / `check -e CODE` / `check -` 仅加载与前端检查，不执行LFZ程序。
//! **不做**：REPL。
//!
//! # 裸文件调用（P4.4，像 Python 那样直接跑）
//!
//! `lfz <file>` 与 `lfz run <file>` 等价；差别仅在**入口校验收紧**：裸文件调用要求 `<file>`
//! 以 `.lfz` 结尾（大小写不敏感），否则给出友好中文错误（退出码 [`EXIT_ERROR`]）。
//! 文件不存在时走既有 `loader` → `IOError: 无法读取：<path>`。`run` 子命令保持旧行为（不校验扩展名）。
//!
//! # 流约定
//!
//! - 程序自身输出（`print` / `eprint` / `;;`）默认由解释器 `builtins` 直接写 **stdout / stderr**，
//!   本模块**不**接管、不缓冲；`--json` 下 `print` / `;;`（dump）/ `input` 提示被重定向到 stderr（见下）。
//!   `;;` 与 `print` **恒同通道**（`semantics.md` §3.6 #7；`builtins::write_dump` 复用同一开关）。
//! - CLI 的**错误诊断**写 **stderr**；`--help` / `--version` 写 **stdout**。
//!   （spec 未规定错误流；采用 Python 约定 → stderr。）
//! - **`--json` 模式**：stdout **只**接收**唯一一行 JSON**（结果对象）；人类可读的报告 / 诊断
//!   一律转 **stderr**（硬性设计约束）。**P4.2-fix（已解决）**：CLI 在运行前把解释器内建
//!   `print` / `;;`（dump）/ `input` 提示重定向到 **stderr**（`lfz::builtins::set_stdout_to_stderr`），
//!   故 stdout 恒为单个 JSON，程序输出不再与之交错（见 `docs/tooling/runner-contract.md` §9.3）。
//!
//! # 退出码（D-008）
//!
//! `0` 成功；`1` 测试有用例失败（`lfz test` 专用，P4）；`2` CLI 参数错误 /
//! LFZ 语法或运行时错误 / 运行环境错误。**`--json` 错误仍退出 `2`**（§8.3 / §8.4）。

use crate::json::{self, Json};
use lfz::error::{LzError, TraceFrame};
use lfz::evaluator::{self, TracedRun};
use lfz::lexer;
use lfz::loader::{self, Loaded};
use lfz::parser;
use lfz::span::Span;
use std::io::{Read, Write};

/// 成功。
pub const EXIT_OK: i32 = 0;
/// 测试有用例失败（`lfz test` 专用；由 [`crate::test_runner`] 使用）。
pub const EXIT_TEST_FAIL: i32 = 1;
/// CLI 参数错误 / LFZ 语法或运行时错误 / 运行环境错误。
pub const EXIT_ERROR: i32 = 2;

/// traceback 折叠（`semantics.md` §8.2 / `interface-contract.md` §10.3）：帧数 `T` **大于**
/// [`TRACEBACK_FOLD_THRESHOLD`] 时，保留的**最外层**帧数（规范性常量 `TRACEBACK_HEAD = 10`）。
pub const TRACEBACK_HEAD: usize = 10;
/// traceback 折叠：保留的**最内层**帧数（规范性常量 `TRACEBACK_TAIL = 30`）。
pub const TRACEBACK_TAIL: usize = 30;
/// traceback 折叠阈值 = `TRACEBACK_HEAD + TRACEBACK_TAIL`（= 40）；`T ≤` 此值时逐帧原样输出。
pub const TRACEBACK_FOLD_THRESHOLD: usize = TRACEBACK_HEAD + TRACEBACK_TAIL;

/// `lfz --help` 文本（写 stdout）。
const HELP: &str = "\
LFZ 解释器

用法:
  lfz <file>           运行 LFZ 脚本（裸调用；等价 lfz run <file>）
  lfz run <file>       运行 LFZ 脚本
  lfz -e CODE [-- ARG...]  直接执行代码（也可 lfz run -e CODE）
  lfz - [-- ARG...]       从标准输入读取源码到 EOF（也可 lfz run -）
  lfz FILE.lfz -- ARG...  原样传入脚本参数，可用 argv() 读取
  lfz check FILE       只检查文件语法，不执行 LFZ 程序
  lfz check -e CODE    只检查直接代码；check - 从 stdin 检查源码
  lfz test [路径...]   运行黑盒测试（默认发现 tests/**/*.lfz，排除 tests/fixtures）
  lfz --help, -h       显示本帮助
  lfz --version, -V    显示版本

文件与前导（#42）:
  裸调用 lfz <file> 等价 lfz run <file>，唯一差别是入口校验：
    lfz <file>       要求 <file> 以 .lfz 结尾（大小写不敏感）；否则报错并退出码 2。
    lfz run <file>   接受任意路径（不校验扩展名）。
  是否要求 #42 前导按扩展名判定（§2.2.0）:
    .lfz 文件        首行必须为 #42，否则 CosmosAnswerError（退出码 2）；
    非 .lfz 文件     豁免 #42 前导（如 .txt / 无扩展名），按普通源加载执行。

选项:
  --max-steps N        限制求值节点步数（非负十进制u64，仅运行命令）
  --timeout-ms N       求值协作截止时间（不含前端，不硬中断阻塞IO）
  --json               以机器可读 JSON 输出（stdout 只写 JSON；程序输出与人类可读诊断转 stderr）
                       源码和 -- 后脚本参数以外的位置可用，以下写法等价：
                         lfz --json <file>       lfz <file> --json
                         lfz run --json <file>   lfz run <file> --json

退出码:
  0  成功（所有用例通过）
  1  测试有用例失败（assert / fail 抛 AssertionError）
  2  CLI 参数错误 / LFZ 语法或运行时错误 / 用例 error / 运行环境错误

代码与参数:
  -e CODE 的 CODE 原样作为源码，不要求 #42；- 源码入口读取 UTF-8 到 EOF。
  脚本参数必须在 -- 后，argv() 不含程序名/脚本名。-- 后的 --json 是普通参数。
  stdin 源码读取后 input() 将遇到 EOF；文件和 -e 模式可继续读取标准输入。
  --json 只输出执行状态对象；需要业务 JSON 时正常模式 print(jsonStringify(value))。
  check 通过语法检查不代表运行成功；不检查名称、类型或文件IO，不执行脚本。
  check 不接受脚本参数或 CODE 外的 --；合法命令的 --json 标记 executed:false。
  CLI 参数错误仍写 stderr 文本；check --help 不受支持，请使用 lfz --help。

示例:
  lfz -e 'print(1+2)'
  lfz -e 'print(argv())' -- first second
  lfz check --json examples/hello.lfz
  lfz examples/hello.lfz
  lfz run examples/hello.lfz
  lfz --json examples/hello.lfz
  lfz test
  lfz test --json tests/smoke
";

/// `lfz --version` 文本（写 stdout）。
const VERSION: &str = concat!("lfz ", env!("CARGO_PKG_VERSION"));

/// 检查模式的输入来源；与执行入口分离，不携带脚本参数。
#[derive(Debug, PartialEq, Eq)]
enum CheckInput {
    File(String),
    Code(String),
    Stdin,
}

/// 解析后的命令行。
#[derive(Debug, PartialEq, Eq)]
enum Command {
    /// `lfz run <file>`。
    Run { path: String },
    /// `lfz <file>`（裸文件调用，P4.4；等价 `lfz run <file>`，但入口须为 `.lfz`）。
    Script { path: String },
    /// 带显式 `--` 参数的文件入口。
    RunArgs { path: String, args: Vec<String> },
    ScriptArgs { path: String, args: Vec<String> },
    /// `-e CODE` 与 `-` 标准输入源码入口。
    Eval { code: String, args: Vec<String> },
    Stdin { args: Vec<String> },
    /// 只加载、词法、语法检查，不求值。
    Check(CheckInput),
    /// `lfz test [路径...]`。
    Test { paths: Vec<String> },
    /// `--help` / `-h`。
    Help,
    /// `--version` / `-V`。
    Version,
}

/// CLI 入口：解析 `args`（**不含** `argv[0]`），写 `out` / `err`，返回退出码。
///
/// 与真实进程解耦：`out` / `err` 可为任意 [`Write`]（生产用 stdout/stderr，测试用内存缓冲）。
///
/// `--json` 仅在选项位置生效；CODE 与 `--` 后的脚本参数不参与选项解析。
pub fn execute(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let (rest, script_args, json, limits) = match scan_options(args) { Ok(v)=>v,Err(msg)=>return usage_error(&msg,err) };
    if limits.is_set() && rest.first().is_some_and(|s| matches!(s.as_str(),"check"|"test"|"--help"|"-h"|"--version"|"-V")) { return usage_error("执行限额选项只适用于运行命令",err); }
    let parsed=parse_command(&rest,script_args);
    if limits.is_set() {
        match parsed {
            Ok(command) if is_execution(&command) => return run_limited(command,limits,json,out,err),
            Ok(_) => return usage_error("执行限额选项只适用于运行命令",err),
            Err(msg) => return usage_error(&msg,err),
        }
    }
    match parsed {
        Ok(Command::Help) => {
            let _ = write!(out, "{HELP}");
            EXIT_OK
        }
        Ok(Command::Version) => {
            let _ = writeln!(out, "{VERSION}");
            EXIT_OK
        }
        Ok(Command::Run { path }) => run_file(&path, json, out, err),
        Ok(Command::Script { path }) => run_script(&path, json, out, err),
        Ok(Command::RunArgs { path, args }) => run_file_with_args(&path, &args, json, out, err),
        Ok(Command::ScriptArgs { path, args }) => run_script_with_args(&path, &args, json, out, err),
        Ok(Command::Eval { code, args }) => run_source(Some(&code), &args, json, out, err),
        Ok(Command::Stdin { args }) => run_source(None, &args, json, out, err),
        Ok(Command::Check(input)) => check_input(&input, json, out, err),
        Ok(Command::Test { paths }) => crate::test_runner::run(&paths, json, out, err),
        Err(msg) => {
            let _ = writeln!(err, "lfz: {msg}");
            let _ = writeln!(
                err,
                "用法: lfz <file> | lfz run <file> | lfz check FILE | lfz test [路径...]（更多: lfz --help）"
            );
            EXIT_ERROR
        }
    }
}

/// 仅摘除选项区域的 `--json`，保护源码与脚本参数的原始字符串。
#[derive(Clone, Copy, Debug, Default)]
struct CliLimits { max_steps:Option<u64>, timeout_ms:Option<u64> }
impl CliLimits {
    fn is_set(self)->bool {self.max_steps.is_some()||self.timeout_ms.is_some()}
    fn evaluation(self)->Result<evaluator::EvalLimits,String>{
        let deadline=match self.timeout_ms {
            Some(n)=>Some(std::time::Instant::now().checked_add(std::time::Duration::from_millis(n)).ok_or_else(||"选项 '--timeout-ms' 超出当前平台可表示的时间范围".to_string())?),
            None=>None,
        };
        Ok(evaluator::EvalLimits{max_steps:self.max_steps,deadline,cancel:None})
    }
}
fn usage_error(message:&str,err:&mut dyn Write)->i32{
    let _=writeln!(err,"lfz: {message}");
    let _=writeln!(err,"用法: lfz <file> | lfz run <file> | lfz check FILE | lfz test [路径...]（更多: lfz --help）");
    EXIT_ERROR
}
fn is_execution(c:&Command)->bool {matches!(c,Command::Run{..}|Command::Script{..}|Command::RunArgs{..}|Command::ScriptArgs{..}|Command::Eval{..}|Command::Stdin{..})}
fn scan_options(args:&[String])->Result<(Vec<String>,Option<Vec<String>>,bool,CliLimits),String>{
    let mut rest=Vec::new();let mut json=false;let mut limits=CliLimits::default();let mut i=0;
    while i<args.len(){
        let a=&args[i];
        if a=="--json" {json=true;i+=1;}
        else if a=="--" && execution_ready(&rest){return Ok((rest,Some(args[i+1..].to_vec()),json,limits));}
        else if a=="--max-steps"||a=="--timeout-ms" {
            let slot=if a=="--max-steps"{&mut limits.max_steps}else{&mut limits.timeout_ms};
            if slot.is_some(){return Err(format!("选项 '{a}' 不能重复"));}
            let value=args.get(i+1).filter(|s|s.as_str()!="--").ok_or_else(||format!("选项 '{a}' 需要一个非负十进制 u64 参数"))?;
            if value.is_empty()||!value.bytes().all(|b|b.is_ascii_digit()){return Err(format!("选项 '{a}' 的值必须是非负十进制 u64"));}
            *slot=Some(value.parse::<u64>().map_err(|_|format!("选项 '{a}' 的值必须是非负十进制 u64"))?);i+=2;
        }else if a.starts_with("--max-steps=")||a.starts_with("--timeout-ms="){let flag=if a.starts_with("--max-steps="){"--max-steps"}else{"--timeout-ms"};return Err(format!("选项 '{flag}' 不支持等号写法，请使用 '{flag} N'"));}
        else{
            let code=a=="-e"&&(rest.is_empty()||(rest.len()==1&&(rest[0]=="run"||rest[0]=="check")));
            rest.push(a.clone());i+=1;
            if code&&i<args.len(){rest.push(args[i].clone());i+=1;}
        }
    }
    Ok((rest,None,json,limits))
}
#[cfg(test)]
fn split_options(args:&[String])->(Vec<String>,Option<Vec<String>>,bool){let(a,b,c,_)=scan_options(args).expect("旧parse测试无非法限额");(a,b,c)}

/// 新限额仅执行路径；默认unlimited保留原流水线和runner入口。
fn run_limited(command:Command,limits:CliLimits,json:bool,out:&mut dyn Write,err:&mut dyn Write)->i32{
    let (input,args,bare)=match command{
        Command::Run{path}=>(CheckInput::File(path),vec![],false),Command::Script{path}=>(CheckInput::File(path),vec![],true),
        Command::RunArgs{path,args}=>(CheckInput::File(path),args,false),Command::ScriptArgs{path,args}=>(CheckInput::File(path),args,true),
        Command::Eval{code,args}=>(CheckInput::Code(code),args,false),Command::Stdin{args}=>(CheckInput::Stdin,args,false),_=>unreachable!(),
    };
    let path=match &input{CheckInput::File(p)=>p.as_str(),CheckInput::Code(_)=>"<command>",CheckInput::Stdin=>"<stdin>"};
    lfz::builtins::set_stdout_to_stderr(json);
    let cwd=std::env::current_dir();
    let result=evaluator::on_eval_stack(||->Result<CaseEval,String>{
        let loaded=match &input {
            CheckInput::File(p) if bare&&!is_lfz_path(p)=>Err(Box::new(LzError::Io{msg:format!("只支持 .lfz 脚本文件：'{p}'"),span:None})),
            CheckInput::File(p)=>loader::load_file_counted(p),CheckInput::Code(s)=>loader::load_source_counted(s),
            CheckInput::Stdin=>{let mut text=String::new();match std::io::stdin().read_to_string(&mut text){Ok(_)=>loader::load_source_counted(&text),Err(_)=>Err(Box::new(LzError::Io{msg:"无法读取标准输入".into(),span:None}))}}
        };
        let failure=|e:Box<LzError>,loaded|CaseEval::Failed{err:*e,loaded,traced:None};
        let counted=match loaded{Ok(c)=>c,Err(e)=>return Ok(failure(e,None))};let loaded=counted.loaded;
        let tokens=match lexer::lex(&loaded.text,loaded.line_base){Ok(t)=>t,Err(e)=>return Ok(failure(e,Some(loaded)))};
        let program=match parser::parse(&tokens){Ok(p)=>p,Err(e)=>return Ok(failure(e,Some(loaded)))};
        if lfz::modules::module_declaration_span(&program).is_some(){
            let cwd=match cwd{Ok(p)=>p,Err(e)=>return Ok(CaseEval::HostFailure(e.to_string()))};
            let identity=if matches!(input,CheckInput::File(_)){std::fs::canonicalize(path).ok()}else{None};
            let base_dir=if matches!(input,CheckInput::File(_)){let p=identity.clone().unwrap_or_else(||cwd.join(path));p.parent().unwrap_or(&cwd).to_path_buf()}else{cwd};
            let graph=match lfz::modules::build_graph(lfz::modules::ParsedEntry{program,loaded,display_path:path.into(),identity,base_dir,raw_bytes:counted.raw_bytes}){
                Ok(g)=>g,Err(g)=>{let source=g.sources.get(g.failure.source_id).expect("图失败来源");return Ok(CaseEval::Located{path:source.display_path.clone(),inner:Box::new(failure(g.failure.error,source.loaded.clone()))});}
            };
            let evaluated=evaluator::eval_graph_traced_on_thread_with_args_and_limits(&graph,&args,limits.evaluation()?);
            return Ok(match evaluated{Ok(r)=>graph_run_result(r),Err(e)=>CaseEval::HostFailure(e.to_string())});
        }
        let traced=evaluator::eval_module_traced_on_thread_with_args_and_limits(&program,&args,limits.evaluation()?);
        if traced.result.is_ok(){Ok(CaseEval::Passed)}else{let e=(**traced.result.as_ref().unwrap_err()).clone();Ok(CaseEval::Failed{err:e,loaded:Some(loaded),traced:Some(traced)})}
    });
    match result{Ok(eval)=>report_eval(path,&eval,json,out,err),Err(message)=>usage_error(&message,err)}
}

fn execution_ready(args: &[String]) -> bool {
    match args {
        [a, _] if a == "-e" => true,
        [a, b, _] if a == "run" && b == "-e" => true,
        [a, b] if a == "run" && b != "-e" => true,
        [a] => a == "-" || (!a.starts_with('-') && a != "run" && a != "test"),
        _ => false,
    }
}

#[cfg(test)]
fn parse_args(args: &[String]) -> Result<Command, String> {
    let (rest, script_args, _) = split_options(args);
    parse_command(&rest, script_args)
}

/// 解析参数为命令；无 `--` 时仍拒绝额外的脚本参数。
fn parse_command(args: &[String], script_args: Option<Vec<String>>) -> Result<Command, String> {
    let first = args.first().ok_or_else(|| "缺少子命令".to_string())?;
    if matches!(first.as_str(), "--help" | "-h") { return Ok(Command::Help); }
    if matches!(first.as_str(), "--version" | "-V") { return Ok(Command::Version); }
    if first == "test" { return Ok(Command::Test { paths: args[1..].to_vec() }); }
    if first == "check" {
        if script_args.is_some() {
            return Err("'check' 不接受脚本参数或 CODE 外的 --".to_string());
        }
        return parse_check_input(&args[1..]).map(Command::Check);
    }
    let explicit = first == "run";
    let input = if explicit { &args[1..] } else { args };
    let target = input.first().ok_or_else(|| "'run' 需要一个 <file> 参数".to_string())?;
    if target == "-e" {
        let code = input.get(1).ok_or_else(|| "'-e' 需要一个 CODE 参数".to_string())?;
        if input.len() != 2 { return Err("'-e' 只接受一个 CODE 参数；脚本参数须放在 -- 后".to_string()); }
        return Ok(Command::Eval { code: code.clone(), args: script_args.unwrap_or_default() });
    }
    if input.len() != 1 {
        return Err(if explicit { "'run' 只接受一个 <file> 参数" } else { "裸文件调用只接受一个 <file> 参数" }.to_string());
    }
    if target == "-" { return Ok(Command::Stdin { args: script_args.unwrap_or_default() }); }
    if !explicit && target.starts_with('-') { return Err(format!("未知选项 '{target}'")); }
    match (explicit, script_args) {
        (true, None) => Ok(Command::Run { path: target.clone() }),
        (false, None) => Ok(Command::Script { path: target.clone() }),
        (true, Some(args)) => Ok(Command::RunArgs { path: target.clone(), args }),
        (false, Some(args)) => Ok(Command::ScriptArgs { path: target.clone(), args }),
    }
}

/// 解析check来源；CODE之外拒绝--和额外操作数。
fn parse_check_input(args: &[String]) -> Result<CheckInput, String> {
    let target = args.first().ok_or_else(|| "'check' 需要 FILE、-e CODE 或 -".to_string())?;
    if target == "-e" {
        let code = args.get(1).ok_or_else(|| "'check -e' 需要一个 CODE 参数".to_string())?;
        if args.len() != 2 {
            return Err("'check -e' 只接受一个 CODE 参数，不接受脚本参数".to_string());
        }
        return Ok(CheckInput::Code(code.clone()));
    }
    if args.len() != 1 {
        return Err("'check' 只接受一个来源，不接受脚本参数或 CODE 外的 --".to_string());
    }
    if target == "-" { return Ok(CheckInput::Stdin); }
    if target.starts_with('-') { return Err(format!("'check' 未知选项 '{target}'")); }
    Ok(CheckInput::File(target.clone()))
}

/// 单个源码流水线结果：供run、test和前端check的错误渲染共用。
///
/// 完成所选模式的全部阶段且无 `LfzError` 为 [`CaseEval::Passed`]；任一阶段报错都进
/// [`CaseEval::Failed`] 并携带渲染所需的上下文（`loaded` / `traced` / `err`）。
pub(crate) enum CaseEval {
    /// run/test完成加载到求值；CLI check仅完成加载到语法，不表示运行成功。
    Passed,
    HostFailure(String),
    Located { path: String, inner: Box<CaseEval> },
    /// 某阶段抛 `LfzError`。
    Failed {
        /// 该错误（已从 `Box` 解出）。
        err: LzError,
        /// 加载成功时的源码视图（加载期失败为 `None`）。
        loaded: Option<Loaded>,
        /// 运行期错误的帧栈（加载 / 解析期失败为 `None`）。
        traced: Option<TracedRun>,
    },
}

impl CaseEval {
    pub(crate) fn host_failure(&self) -> Option<&str> {
        match self { Self::HostFailure(detail) => Some(detail), Self::Located { inner, .. } => inner.host_failure(), _ => None }
    }

    /// 错误的**类名**（`Passed` → `None`）。
    #[must_use]
    pub(crate) fn error_class(&self) -> Option<&'static str> {
        match self {
            CaseEval::HostFailure(_) => None,
            CaseEval::Located { inner, .. } => inner.error_class(),
            CaseEval::Passed => None,
            CaseEval::Failed { err, .. } => Some(err.class_name()),
        }
    }

    /// 错误对象（`Passed` → `None`）。
    #[must_use]
    pub(crate) fn error(&self) -> Option<&LzError> {
        match self {
            CaseEval::HostFailure(_) => None,
            CaseEval::Located { inner, .. } => inner.error(),
            CaseEval::Passed => None,
            CaseEval::Failed { err, .. } => Some(err),
        }
    }

    /// 按 `semantics.md` §8.2 / §8.3 渲染该错误的完整输出块（`Passed` → `None`）。
    #[must_use]
    pub(crate) fn render(&self, path: &str) -> Option<String> {
        match self {
            CaseEval::HostFailure(detail) => Some(format!("lfz: 模块执行上下文无效：{detail}\n")),
            CaseEval::Located { path, inner } => inner.render(path),
            CaseEval::Passed => None,
            CaseEval::Failed { err, loaded, traced } => {
                let origin = traced.as_ref().and_then(|r| r.error_source());
                Some(render_error(origin.map_or(path, |s| s.display_path.as_str()), origin.map_or(loaded.as_ref(), |s| s.loaded.as_ref()), err, traced.as_ref()))
            }
        }
    }
}

/// `run --json` 成功对象：`{"ok":true}`（§8.4：除 `ok` 外其余字段均为错误专有，成功时省略）。
#[must_use]
pub(crate) fn success_json() -> Json {
    Json::Obj(vec![("ok".to_string(), Json::Bool(true))])
}

/// `run --json` 失败对象：字段与顺序**逐字符对齐** `semantics.md` §8.3 示例 4——
/// `ok` / `error` / `message` / `file` / `line` / `col` / `traceback`。
#[must_use]
pub(crate) fn error_json(path: &str, eval: &CaseEval) -> Json {
    let mut fields = vec![("ok".to_string(), Json::Bool(false))];
    push_error_fields(&mut fields, path, eval);
    Json::Obj(fields)
}

/// 追加一个错误的 §8.4 字段（`error` / `message` / `file` / `line` / `col` / `traceback`），
/// **不含** `ok`。供 [`error_json`]（`run`）与 `test_runner`（逐用例）复用。
///
/// - `error` 恒为 [`LzError::class_name`]（12 类名之一，**禁止** `E-xxx`）；
/// - `line` / `col` 取 [`LzError::span`]；`Io` 无 span 时输出 `null`；
/// - `traceback` 为**自最外层到最内层**的完整帧表（**不折叠**，§8.4）。
pub(crate) fn push_error_fields(fields: &mut Vec<(String, Json)>, path: &str, eval: &CaseEval) {
    if let CaseEval::Located { path, inner } = eval { push_error_fields(fields, path, inner); return; }
    let CaseEval::Failed { err, traced, .. } = eval else {
        return;
    };
    let path = traced.as_ref().and_then(|r| r.error_source()).map_or(path, |s| s.display_path.as_str());
    fields.push(("error".to_string(), Json::Str(err.class_name().to_string())));
    fields.push(("message".to_string(), Json::Str(err.message())));
    fields.push(("file".to_string(), Json::Str(path.to_string())));
    match err.span() {
        Some(span) => {
            fields.push(("line".to_string(), Json::Int(i64::from(span.line))));
            fields.push(("col".to_string(), Json::Int(i64::from(span.col))));
        }
        // `Io` 可无源码位置（如读文件失败，loader 不给 span）→ `null`（保持字段存在、类型显式）。
        None => {
            fields.push(("line".to_string(), Json::Null));
            fields.push(("col".to_string(), Json::Null));
        }
    }
    fields.push((
        "traceback".to_string(),
        traceback_json(path, err, traced.as_ref()),
    ));
}

/// 构造 `--json` 的 `traceback` 数组（§8.4：元素 `{file,line,func}`，**不折叠**）。
///
/// - 运行期（`traced = Some`）：逐帧取 `frame.span.line` + [`TracedRun::frame_name`]
///   （`<module>` / 函数名 / `<fn>`），顺序自**最外层→最内层**（与 §8.3 示例 2 一致）。
/// - 加载 / 解析期（`traced = None`）：**合成单帧 `<module>`**，行取错误 span（§8.3 示例 4）；
///   无 span（`Io`）时为空数组 `[]`。
#[must_use]
pub(crate) fn traceback_json(
    path: &str,
    err: &LzError,
    traced: Option<&TracedRun>,
) -> Json {
    match traced {
        Some(run) => Json::Arr(
            run.frames
                .iter()
                .enumerate()
                .map(|(index, frame)| {
                    Json::Obj(vec![
                        ("file".to_string(), Json::Str(run.frame_source(index).map_or(path, |s| s.display_path.as_str()).to_string())),
                        ("line".to_string(), Json::Int(i64::from(frame.span.line))),
                        (
                            "func".to_string(),
                            Json::Str(run.frame_name(frame).to_string()),
                        ),
                    ])
                })
                .collect(),
        ),
        None => match err.span() {
            Some(span) => Json::Arr(vec![Json::Obj(vec![
                ("file".to_string(), Json::Str(path.to_string())),
                ("line".to_string(), Json::Int(i64::from(span.line))),
                ("func".to_string(), Json::Str("<module>".to_string())),
            ])]),
            None => Json::Arr(Vec::new()),
        },
    }
}

/// 加载并执行一个 LFZ 文件：`load_file → lex → parse → eval_module_traced`。
///
/// 与 [`run_file`] 的差别：`run_file` 只关心成败与渲染，本函数把**中间上下文**带出，
/// 供 runner 逐用例判定（类名 ↔ 期望、位置信息）。
///
/// §10.9 R-S2：整条流水线（`load → lex → parse → eval`，含 `Program` 的**析构**）在
/// **≥64 MiB 大栈线程**（复用求值用的 256 MiB [`evaluator::on_eval_stack`]）上运行，
/// 保证 `PARSE_DEPTH_LIMIT`（1000）层可达、深左偏 AST 的递归析构不溢出。
pub(crate) fn eval_case(path: &str) -> CaseEval {
    eval_file_with_args(path, &[])
}

/// [`eval_case`] 的流水线体（在 [`evaluator::on_eval_stack`] 的大栈线程上运行）。
fn eval_file_with_args(path: &str, args: &[String]) -> CaseEval {
    evaluator::on_eval_stack(|| eval_loaded_on_stack(loader::load_file_counted(path), args, path, true, std::env::current_dir()))
}

/// 文件、代码和stdin共用已加载源码流水线；调用者须位于大栈。
fn eval_loaded_on_stack(loaded: lfz::error::R<loader::CountedLoaded>, args: &[String], path: &str, file_input: bool, cwd: std::io::Result<std::path::PathBuf>) -> CaseEval {
    let loaded = match loaded {
        Ok(l) => l,
        Err(e) => {
            return CaseEval::Failed {
                err: *e,
                loaded: None,
                traced: None,
            }
        }
    };
    let raw_bytes = loaded.raw_bytes;
    let loaded = loaded.loaded;
    // 词法期。
    let tokens = match lexer::lex(&loaded.text, loaded.line_base) {
        Ok(t) => t,
        Err(e) => {
            return CaseEval::Failed {
                err: *e,
                loaded: Some(loaded),
                traced: None,
            }
        }
    };
    // 语法期。
    let program = match parser::parse(&tokens) {
        Ok(p) => p,
        Err(e) => {
            return CaseEval::Failed {
                err: *e,
                loaded: Some(loaded),
                traced: None,
            }
        }
    };
    // 运行期：用 `eval_module_traced` 取帧栈（§10.3 / D-008 影响段）；
    // 已在流水线大栈线程上，故用**当前线程**求值体（§10.9 R-S2 同栈），不再嵌套开线程。
    let traced = if lfz::modules::module_declaration_span(&program).is_some() {
        let cwd = match cwd { Ok(p) => p, Err(e) => return CaseEval::HostFailure(e.to_string()) };
        let identity = if file_input { std::fs::canonicalize(path).ok() } else { None };
        let base_dir = if file_input { let absolute = identity.clone().unwrap_or_else(|| cwd.join(path)); absolute.parent().unwrap_or(&cwd).to_path_buf() } else { cwd };
        let graph = match lfz::modules::build_graph(lfz::modules::ParsedEntry { program, loaded, display_path:path.to_string(), identity, base_dir, raw_bytes }) {
            Ok(g) => g,
            Err(g) => { let source=g.sources.get(g.failure.source_id).expect("图失败来源"); return CaseEval::Located { path:source.display_path.clone(), inner:Box::new(CaseEval::Failed { err:*g.failure.error, loaded:source.loaded.clone(), traced:None }) }; }
        };
        match evaluator::eval_graph_traced_on_thread_with_args(&graph,args) {
            Ok(run) => return graph_run_result(run),
            Err(error) => return CaseEval::HostFailure(error.to_string()),
        }
    } else { evaluator::eval_module_traced_on_thread_with_args(&program,args) };
    if traced.result.is_ok() {
        return CaseEval::Passed;
    }
    // 先把错误 clone 出来，再把 `traced`（含完整帧栈）移交出去。
    let err = (**traced.result.as_ref().expect_err("已判定为 Err")).clone();
    CaseEval::Failed {
        err,
        loaded: Some(loaded),
        traced: Some(traced),
    }
}

fn graph_run_result(traced: TracedRun) -> CaseEval {
    if (0..traced.frames.len()).any(|i| traced.frame_source(i).is_none()) { return CaseEval::HostFailure("来源元数据缺失".to_string()); }
    if traced.result.is_ok() { return CaseEval::Passed; }
    let Some(origin) = traced.error_source() else { return CaseEval::HostFailure("来源元数据缺失".to_string()); };
    let loaded=origin.loaded.clone();
    let err=(**traced.result.as_ref().unwrap_err()).clone();
    CaseEval::Failed { err, loaded, traced:Some(traced) }
}

/// 只做前端检查。输入加载、解析和深AST析构全部留在大栈线程内。
/// 这里不调用求值入口，也不建立Interp或调整LFZ输出重定向。
fn check_input(input: &CheckInput, json_mode: bool, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let path = match input {
        CheckInput::File(path) => path.as_str(),
        CheckInput::Code(_) => "<command>",
        CheckInput::Stdin => "<stdin>",
    };
    let cwd = std::env::current_dir();
    let (diagnostic_path, checked) = evaluator::on_eval_stack(|| {
        let loaded = match input {
            CheckInput::File(path) => loader::load_file_counted(path),
            CheckInput::Code(code) => loader::load_source_counted(code),
            CheckInput::Stdin => {
                let mut source = String::new();
                match std::io::stdin().read_to_string(&mut source) {
                    Ok(_) => loader::load_source_counted(&source),
                    Err(_) => Err(Box::new(LzError::Io { msg: "无法读取标准输入".to_string(), span: None })),
                }
            }
        };
        check_loaded_on_stack(loaded, path, matches!(input, CheckInput::File(_)), cwd)
    });
    let passed = matches!(checked, CaseEval::Passed);
    if json_mode {
        let mut fields = vec![
            ("ok".to_string(), Json::Bool(passed)),
            ("mode".to_string(), Json::Str("check".to_string())),
            ("executed".to_string(), Json::Bool(false)),
        ];
        push_error_fields(&mut fields, &diagnostic_path, &checked);
        let _ = writeln!(out, "{}", json::encode(&Json::Obj(fields)));
    } else if let Some(block) = checked.render(&diagnostic_path) {
        let _ = write!(err, "{block}");
    }
    if passed { EXIT_OK } else { EXIT_ERROR }
}

/// 不返回Program：成功后在当前大栈显式drop，主线程只接收状态/诊断。
fn check_loaded_on_stack(
    loaded: lfz::error::R<loader::CountedLoaded>,
    path: &str,
    file_input: bool,
    cwd: std::io::Result<std::path::PathBuf>,
) -> (String, CaseEval) {
    let failure = |error: Box<LzError>, loaded| (path.to_string(), CaseEval::Failed { err: *error, loaded, traced: None });
    let counted = match loaded {
        Ok(loaded) => loaded,
        Err(error) => return failure(error, None),
    };
    let loaded = counted.loaded;
    let tokens = match lexer::lex(&loaded.text, loaded.line_base) {
        Ok(tokens) => tokens,
        Err(error) => return failure(error, Some(loaded)),
    };
    let program = match parser::parse(&tokens) {
        Ok(program) => program,
        Err(error) => return failure(error, Some(loaded)),
    };
    if lfz::modules::module_declaration_span(&program).is_none() {
        drop(program);
        return (path.to_string(), CaseEval::Passed);
    }
    let cwd = match cwd {
        Ok(cwd) => cwd,
        Err(_) => return failure(Box::new(LzError::Io { msg: "无法读取当前目录".to_string(), span: None }), Some(loaded)),
    };
    let identity = if file_input { std::fs::canonicalize(path).ok() } else { None };
    let base_dir = if file_input {
        let absolute = identity.clone().unwrap_or_else(|| cwd.join(path));
        absolute.parent().unwrap_or(&cwd).to_path_buf()
    } else { cwd };
    let entry = lfz::modules::ParsedEntry { program, loaded, display_path: path.to_string(), identity, base_dir, raw_bytes: counted.raw_bytes };
    match lfz::modules::build_graph(entry) {
        Ok(graph) => {
            drop(graph);
            (path.to_string(), CaseEval::Passed)
        }
        Err(graph) => {
            let source = graph.sources.get(graph.failure.source_id).expect("模块图失败来源有效");
            (source.display_path.clone(), CaseEval::Failed { err: *graph.failure.error, loaded: source.loaded.clone(), traced: None })
        }
    }
}

/// 运行一个 LFZ 文件（显式 `lfz run <file>`）。
///
/// - `json == false`：失败 → 按 §8.2/§8.3 渲染到 `err` 并返回 [`EXIT_ERROR`]；成功 → [`EXIT_OK`]。
/// - `json == true`：stdout **只写一行 JSON**（成功 `{"ok":true}`；失败为 §8.3 示例 4 形状），
///   人类可读错误块**不**写 stdout；退出码同 D-008（错误 → [`EXIT_ERROR`]）。
fn run_file(path: &str, json: bool, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    run_file_with_args(path, &[], json, out, err)
}

fn run_file_with_args(path: &str, args: &[String], json: bool, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    lfz::builtins::set_stdout_to_stderr(json);
    let eval = eval_file_with_args(path, args);
    report_eval(path, &eval, json, out, err)
}

fn run_source(code: Option<&str>, args: &[String], json: bool, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    lfz::builtins::set_stdout_to_stderr(json);
    let path = if code.is_some() { "<command>" } else { "<stdin>" };
    let cwd = std::env::current_dir();
    let eval = evaluator::on_eval_stack(|| {
        let loaded = match code {
            Some(text) => loader::load_source_counted(text),
            None => {
                let mut text = String::new();
                match std::io::stdin().read_to_string(&mut text) {
                    Ok(_) => loader::load_source_counted(&text),
                    Err(_) => Err(Box::new(LzError::Io { msg: "无法读取标准输入".to_string(), span: None })),
                }
            }
        };
        eval_loaded_on_stack(loaded, args, path, false, cwd)
    });
    report_eval(path, &eval, json, out, err)
}

/// 运行一次**裸文件调用**（`lfz <file>`，P4.4；等价 `lfz run <file>`）。
///
/// 与 [`run_file`] 的唯一差别：**入口校验收紧**——`<file>` 必须以 `.lfz` 结尾
/// （大小写不敏感）；否则不做词法 / 语法 / 求值，直接给出友好中文错误（退出码 [`EXIT_ERROR`]）。
/// 以 `.lfz` 结尾但**不存在**时，交由 [`eval_case`] → `loader` 产出既有
/// `IOError: 无法读取：<path>`（同样退出码 [`EXIT_ERROR`]）。
fn run_script(path: &str, json: bool, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    run_script_with_args(path, &[], json, out, err)
}

fn run_script_with_args(path: &str, args: &[String], json: bool, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    lfz::builtins::set_stdout_to_stderr(json);
    let eval = if is_lfz_path(path) {
        eval_file_with_args(path, args)
    } else {
        // 与解释器错误**同源**渲染：`--json` 下得到唯一合法 JSON（`IOError` + `line/col:null`），
        // 非 `--json` 下得到 `IOError: 只支持 .lfz 脚本文件：'<path>'`。
        CaseEval::Failed {
            err: LzError::Io {
                msg: format!("只支持 .lfz 脚本文件：'{path}'"),
                span: None,
            },
            loaded: None,
            traced: None,
        }
    };
    report_eval(path, &eval, json, out, err)
}

/// `<path>` 是否以 `.lfz` 结尾（大小写不敏感，如 `.LFZ` 亦接受）。
fn is_lfz_path(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("lfz"))
}

/// 把 [`CaseEval`] 渲染为输出并按 D-008 取退出码（[`run_file`] 与 [`run_script`] 共用）。
///
/// `json == true` → stdout 只写一行 JSON；否则人类可读块写 `err`。
fn report_eval(
    path: &str,
    eval: &CaseEval,
    json: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    if let Some(detail) = eval.host_failure() { let _=writeln!(err,"lfz: 模块执行上下文无效：{detail}"); return EXIT_ERROR; }
    if json {
        let passed = matches!(eval, CaseEval::Passed);
        let doc = if passed {
            success_json()
        } else {
            error_json(path, eval)
        };
        let _ = writeln!(out, "{}", json::encode(&doc));
        return if passed { EXIT_OK } else { EXIT_ERROR };
    }
    match eval.render(path) {
        None => EXIT_OK,
        Some(block) => {
            let _ = write!(err, "{block}");
            EXIT_ERROR
        }
    }
}

/// 按 `semantics.md` §8.2 / §8.3 渲染错误输出（返回值以 `\n` 结尾）。
///
/// - `source`：加载成功后的源码视图；加载失败（如 `IOError`）时为 `None`。
/// - `traced`：运行期错误的帧栈；`None` 表示**加载 / 解析期**（**无** `Traceback` 头）。
///
/// 结构：
/// - `CosmosAnswerError`：仅 `File "<path>", line 1` → 末行（**无源码行 / 无插入符**）。
/// - `SyntaxError`：`  File "<path>", line N` + 源码行 + 插入符 → 末行。
/// - 运行期：`Traceback (most recent call last):` 头 + 逐帧（最外层→最内层）→ 末行。
///   帧数 `T > [`TRACEBACK_FOLD_THRESHOLD`]` 时按 §8.2 **折叠**（首 10 帧 + 省略行 + 尾 30 帧），
///   `T ≤ 阈值` 时逐帧原样输出（浅栈行为逐字节不变）。
pub fn render_error(
    path: &str,
    source: Option<&Loaded>,
    err: &LzError,
    traced: Option<&TracedRun>,
) -> String {
    let mut out = String::new();
    match traced {
        // 加载 / 解析期：无 `Traceback` 头。
        None => render_load_error(&mut out, path, source, err),
        // 运行期：有 `Traceback` 头，逐帧最外层→最内层（深栈折叠，§8.2）。
        Some(run) => {
            out.push_str("Traceback (most recent call last):\n");
            push_frames(&mut out, path, run, source);
        }
    }
    out.push_str(err.class_name());
    out.push_str(": ");
    out.push_str(&err.message());
    out.push('\n');
    out
}

/// 加载 / 解析期错误渲染（§8.2：**无** `Traceback` 头）。
fn render_load_error(out: &mut String, path: &str, source: Option<&Loaded>, err: &LzError) {
    if err.class_name() == "CosmosAnswerError" {
        // §8.3 示例 3：仅 `File "<path>", line 1`，无源码行 / 插入符。
        out.push_str("File \"");
        out.push_str(path);
        out.push_str("\", line 1\n");
        return;
    }
    // SyntaxError 等：有位置则渲染 `File` 帧（含源码行 + 插入符）。
    if let Some(span) = err.span() {
        push_frame(out, path, span, None, source);
    }
}

/// 追加运行期**帧栈**（`semantics.md` §8.2 折叠）：帧数 `T ≤ 40` → 逐帧原样；`T > 40` →
/// **首 10 帧** → **省略行** → **尾 30 帧**。
///
/// 折叠**仅作用于此处的人类可读渲染**；`TracedRun.frames` 本身保持完整（§10.3 / §8.4）。
fn push_frames(out: &mut String, path: &str, run: &TracedRun, source: Option<&Loaded>) {
    let frames = &run.frames;
    let total = frames.len();
    if total <= TRACEBACK_FOLD_THRESHOLD {
        for (index, frame) in frames.iter().enumerate() {
            push_trace_frame(out, path, frame, run, source, index);
        }
        return;
    }
    for (index, frame) in frames[..TRACEBACK_HEAD].iter().enumerate() {
        push_trace_frame(out, path, frame, run, source, index);
    }
    // 省略行（逐字符，§8.2）：2 空格 + `... 省略 {N} 帧 ...`，`N = T − 40`（十进制）。
    out.push_str("  ... 省略 ");
    out.push_str(&(total - TRACEBACK_FOLD_THRESHOLD).to_string());
    out.push_str(" 帧 ...\n");
    for (offset, frame) in frames[total - TRACEBACK_TAIL..].iter().enumerate() {
        push_trace_frame(out, path, frame, run, source, total - TRACEBACK_TAIL + offset);
    }
}

/// 追加单个 traceback 帧（按 `func_id` 反查显示名，§8.4）。
fn push_trace_frame(
    out: &mut String,
    path: &str,
    frame: &TraceFrame,
    run: &TracedRun,
    source: Option<&Loaded>,
    index: usize,
) {
    let name = run.frame_name(frame);
    let origin=run.frame_source(index);
    push_frame(out, origin.map_or(path, |s| s.display_path.as_str()), frame.span, Some(name.as_ref()), origin.map_or(source, |s| s.loaded.as_ref()));
}

/// 追加一个「帧」（§8.2 通用三行格式；源码行 / 插入符仅在可取到源码行时输出）。
///
/// `func = None` → 帧头无 `, in <func>`（加载 / 解析期）。
fn push_frame(
    out: &mut String,
    path: &str,
    span: Span,
    func: Option<&str>,
    source: Option<&Loaded>,
) {
    out.push_str("  File \"");
    out.push_str(path);
    out.push_str("\", line ");
    out.push_str(&span.line.to_string());
    if let Some(name) = func {
        out.push_str(", in ");
        out.push_str(name);
    }
    out.push('\n');
    if let Some(line_text) = source_line(source, span.line) {
        // 源码行原文，固定 4 空格缩进。
        out.push_str("    ");
        out.push_str(line_text);
        out.push('\n');
        // 插入符行 = 4 空格 + (列号 - 1) 空格 + '^'。
        out.push_str("    ");
        for _ in 1..span.col {
            out.push(' ');
        }
        out.push_str("^\n");
    }
}

/// 取绝对行号 `line` 对应的源码行文本。
///
/// `loaded.text` 的行号与绝对行号相差 `line_base`：`.lfz`（`line_base = 1`）的 `text`
/// 首行 = 文件第 2 行 → 下标 = `line - 2`；非 `.lfz`（`line_base = 0`）→ 下标 = `line - 1`。
/// 行号越界或 `source` 缺失 → `None`（此时帧只输出 `File` 行）。
fn source_line<'a>(source: Option<&'a Loaded>, line: u32) -> Option<&'a str> {
    let loaded = source?;
    let idx = line.checked_sub(loaded.line_base + 1)?;
    loaded.text.lines().nth(idx as usize)
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use lfz::error::SyntaxMsg;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn parse_help_and_version() {
        assert_eq!(parse_args(&args(&["--help"])), Ok(Command::Help));
        assert_eq!(parse_args(&args(&["-h"])), Ok(Command::Help));
        assert_eq!(parse_args(&args(&["--version"])), Ok(Command::Version));
        assert_eq!(parse_args(&args(&["-V"])), Ok(Command::Version));
    }

    #[test]
    fn parse_run_requires_exactly_one_file() {
        assert_eq!(
            parse_args(&args(&["run", "a.lfz"])),
            Ok(Command::Run {
                path: "a.lfz".to_string()
            })
        );
        assert!(parse_args(&args(&["run"])).is_err());
        assert!(parse_args(&args(&["run", "a.lfz", "b.lfz"])).is_err());
        assert!(parse_args(&args(&[])).is_err());
        // P4.4：非 flag 的首个参数 = 裸文件调用（不再是「未知命令」）。
        assert_eq!(
            parse_args(&args(&["frobnicate"])),
            Ok(Command::Script {
                path: "frobnicate".to_string()
            })
        );
        // 以 `-` 开头者仍是未知选项 / 命令。
        assert!(parse_args(&args(&["--frobnicate"])).is_err());
    }

    /// P4.4：`lfz <file>` 裸文件调用解析为 [`Command::Script`]（须恰一个参数）。
    #[test]
    fn parse_bare_file_is_script() {
        assert_eq!(
            parse_args(&args(&["examples/hello.lfz"])),
            Ok(Command::Script {
                path: "examples/hello.lfz".to_string()
            })
        );
        assert_eq!(
            parse_args(&args(&["hello.lfz"])),
            Ok(Command::Script {
                path: "hello.lfz".to_string()
            })
        );
        assert!(parse_args(&args(&["a.lfz", "b.lfz"])).is_err());
    }

    /// `.lfz` 扩展名判定（大小写不敏感；非 `.lfz` / 无扩展名为假）。
    #[test]
    fn is_lfz_path_checks_extension() {
        assert!(is_lfz_path("a.lfz"));
        assert!(is_lfz_path("dir/a.lfz"));
        assert!(is_lfz_path("A.LFZ"));
        assert!(!is_lfz_path("a.txt"));
        assert!(!is_lfz_path("a"));
        assert!(!is_lfz_path("dir.with.dot/a"));
    }

    #[test]
    fn parse_test_collects_paths() {
        assert_eq!(
            parse_args(&args(&["test"])),
            Ok(Command::Test { paths: Vec::new() })
        );
        assert_eq!(
            parse_args(&args(&["test", "a", "b"])),
            Ok(Command::Test {
                paths: vec!["a".to_string(), "b".to_string()]
            })
        );
    }

    #[test]
    fn help_and_version_exit_zero_on_stdout() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(execute(&args(&["--help"]), &mut out, &mut err), EXIT_OK);
        let text = String::from_utf8(out).expect("UTF-8");
        assert!(text.contains("lfz <file>"), "help 应把裸文件调用放最前: {text}");
        assert!(text.contains("lfz run <file>"), "help: {text}");
        assert!(text.contains("lfz test"), "help 应列出 test 子命令: {text}");
        assert!(err.is_empty());

        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(execute(&args(&["--version"]), &mut out, &mut err), EXIT_OK);
        assert_eq!(String::from_utf8(out).expect("UTF-8"), format!("{VERSION}\n"));
        assert!(err.is_empty());
    }

    /// obs-A-02：`--help` 文案须与 `syntax.md` §2.2.0 扩展名豁免一致，并明示
    /// 「裸调用 `lfz <file>` 等价 `lfz run <file>`」；不得再保留把豁免说死的一刀切表述。
    #[test]
    fn help_text_matches_extension_exemption() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(execute(&args(&["--help"]), &mut out, &mut err), EXIT_OK);
        let text = String::from_utf8(out).expect("UTF-8");
        // 明示裸调用与 `run` 的等价关系（任务要求）。
        assert!(text.contains("等价 lfz run <file>"), "应说明裸调用等价：{text}");
        // 明示 §2.2.0 扩展名豁免（非 `.lfz` 文件不要求 `#42`）。
        assert!(text.contains("非 .lfz 文件"), "应说明非 .lfz 豁免：{text}");
        assert!(text.contains("豁免 #42 前导"), "应说明前导豁免：{text}");
        // 裸调用的入口校验（须 `.lfz`）仍须如实记录。
        assert!(text.contains("要求 <file> 以 .lfz 结尾"), "应记录裸调用入口校验：{text}");
        // 旧的一刀切误导文案（对 `run` 亦不成立）必须移除。
        assert!(
            !text.contains("<file> 须以 .lfz 结尾"),
            "不得保留旧误导文案：{text}"
        );
    }

    #[test]
    fn bad_args_exit_two_on_stderr() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(execute(&args(&[]), &mut out, &mut err), EXIT_ERROR);
        assert!(out.is_empty());
        assert!(String::from_utf8(err).expect("UTF-8").contains("缺少子命令"));
    }

    /// §8.3 示例 3：`CosmosAnswerError` 仅 `File` 行，无源码行 / 插入符 / Traceback 头。
    #[test]
    fn cosmos_answer_has_only_file_line() {
        let e = LzError::CosmosAnswer { span: Span::START };
        let rendered = render_error("forgot.lfz", None, &e, None);
        assert_eq!(
            rendered,
            "File \"forgot.lfz\", line 1\nCosmosAnswerError: 你忘记了宇宙的答案\n"
        );
    }

    /// §8.3 示例 1：`SyntaxError` 的 `File` 帧 + 源码行 + 插入符，无 Traceback 头。
    #[test]
    fn syntax_error_renders_frame_with_caret() {
        let loaded = Loaded {
            text: "let x = 1 $ 2\n".to_string(),
            line_base: 1,
        };
        let e = LzError::Syntax {
            msg: SyntaxMsg::IllegalChar { c: '$' },
            span: Span::new(2, 11),
        };
        let rendered = render_error("demo.lfz", Some(&loaded), &e, None);
        assert_eq!(
            rendered,
            "  File \"demo.lfz\", line 2\n    let x = 1 $ 2\n              ^\nSyntaxError: 非法字符 '$'\n"
        );
    }

    /// 运行期错误：含 `Traceback` 头，且末行为 `类名: 中文消息`。
    #[test]
    fn runtime_error_has_traceback_header() {
        let src = "1 / 0\n";
        let tokens = lexer::lex(src, 0).expect("lex");
        let program = parser::parse(&tokens).expect("parse");
        let traced = evaluator::eval_module_traced(&program);
        let err = traced.result.as_ref().expect_err("除以零应报错");
        let loaded = Loaded {
            text: src.to_string(),
            line_base: 0,
        };
        let rendered = render_error("demo.lfz", Some(&loaded), err, Some(&traced));
        assert!(
            rendered.starts_with("Traceback (most recent call last):\n"),
            "运行期错误必须有 Traceback 头：{rendered}"
        );
        assert!(
            rendered.contains("File \"demo.lfz\", line 1, in <module>"),
            "{rendered}"
        );
        assert!(
            rendered.ends_with("ZeroDivisionError: 除以零\n"),
            "{rendered}"
        );
    }

    /// bug-20260924-09 回归：深递归（`RecursionError`）的**人类可读** traceback 按 §8.2 折叠：
    /// 首 10 帧 → `  ... 省略 {T−40} 帧 ...` → 尾 30 帧；`TracedRun.frames` 本身保持**完整**。
    #[test]
    fn deep_recursion_traceback_is_folded() {
        // `fn loop(n) { loop(n) }` 无限递归 → 命中 10000 层上限 → RecursionError。
        let src = "fn loop(n) { loop(n) }\nloop(1)\n";
        let tokens = lexer::lex(src, 0).expect("lex");
        let program = parser::parse(&tokens).expect("parse");
        let traced = evaluator::eval_module_traced(&program);
        let err = traced
            .result
            .as_ref()
            .expect_err("深递归应报 RecursionError");
        assert_eq!(err.class_name(), "RecursionError");
        // 数据层不折叠：完整帧栈（模块帧 + 10000 个用户帧）。
        assert!(
            traced.frames.len() >= 10_000,
            "TracedRun.frames 应完整保留，实得 {}",
            traced.frames.len()
        );

        let loaded = Loaded {
            text: src.to_string(),
            line_base: 0,
        };
        let rendered = render_error("deep.lfz", Some(&loaded), err, Some(&traced));

        // 人类可读渲染：帧行数 = HEAD + TAIL（省略行不以 `  File "` 开头）。
        let frame_lines = rendered
            .lines()
            .filter(|l| l.starts_with("  File \""))
            .count();
        assert_eq!(frame_lines, TRACEBACK_HEAD + TRACEBACK_TAIL, "{rendered}");

        // 省略行逐字符：`  ... 省略 {T−40} 帧 ...`。
        let omitted = traced.frames.len() - TRACEBACK_FOLD_THRESHOLD;
        assert!(
            rendered.contains(&format!("  ... 省略 {omitted} 帧 ...\n")),
            "缺少逐字符省略行：{rendered}"
        );
        assert!(
            rendered.ends_with("RecursionError: 递归深度超限（超过 10000 层）\n"),
            "{rendered}"
        );
    }

    /// `source_line` 的 `line_base` 偏移（`.lfz` 的 text 首行 = 文件第 2 行）。
    #[test]
    fn source_line_respects_line_base() {
        let loaded = Loaded {
            text: "first\nsecond\n".to_string(),
            line_base: 1,
        };
        assert_eq!(source_line(Some(&loaded), 2), Some("first"));
        assert_eq!(source_line(Some(&loaded), 3), Some("second"));
        assert_eq!(source_line(Some(&loaded), 1), None, "前导行不显示源码");
        assert_eq!(source_line(Some(&loaded), 99), None);
        assert_eq!(source_line(None, 2), None);
    }

    // ---- `--json`（§8.3 示例 4 / §8.4） ----------------------------------

    fn failed(err: LzError) -> CaseEval {
        CaseEval::Failed {
            err,
            loaded: None,
            traced: None,
        }
    }

    /// `CosmosAnswerError` 的成功失败对象**逐字符**等于 §8.3 示例 4。
    #[test]
    fn error_json_matches_example4_byte_for_byte() {
        let eval = failed(LzError::CosmosAnswer { span: Span::START });
        assert_eq!(
            json::encode(&error_json("forgot.lfz", &eval)),
            concat!(
                r#"{"ok":false,"error":"CosmosAnswerError","message":"你忘记了宇宙的答案","#,
                r#""file":"forgot.lfz","line":1,"col":1,"#,
                r#""traceback":[{"file":"forgot.lfz","line":1,"func":"<module>"}]}"#
            )
        );
    }

    /// 加载/解析期错误 → 合成单帧 `<module>`，位置取错误 span。
    #[test]
    fn error_json_syntax_error_uses_module_frame_and_position() {
        let eval = failed(LzError::Syntax {
            msg: SyntaxMsg::IllegalChar { c: '$' },
            span: Span::new(2, 11),
        });
        let s = json::encode(&error_json("demo.lfz", &eval));
        assert!(s.contains(r#""error":"SyntaxError""#), "{s}");
        assert!(s.contains(r#""message":"非法字符 '$'""#), "{s}");
        assert!(s.contains(r#""line":2"#), "{s}");
        assert!(s.contains(r#""col":11"#), "{s}");
        assert!(
            s.contains(r#""traceback":[{"file":"demo.lfz","line":2,"func":"<module>"}]"#),
            "{s}"
        );
    }

    /// `Io` 无 span → `line`/`col` 为 `null`、`traceback` 为 `[]`（不伪造型别）。
    #[test]
    fn error_json_io_without_span_is_null_and_empty_traceback() {
        let eval = failed(LzError::Io {
            msg: "无法读取：x.lfz".to_string(),
            span: None,
        });
        let s = json::encode(&error_json("x.lfz", &eval));
        assert!(s.contains(r#""error":"IOError""#), "{s}");
        assert!(s.contains(r#""message":"无法读取：x.lfz""#), "{s}");
        assert!(s.contains(r#""line":null"#), "{s}");
        assert!(s.contains(r#""col":null"#), "{s}");
        assert!(s.contains(r#""traceback":[]"#), "{s}");
    }

    #[test]
    fn success_json_is_ok_true_only() {
        assert_eq!(json::encode(&success_json()), r#"{"ok":true}"#);
    }

    /// `--json` 被摘除后 `run` 正常解析；失败的 `run --json` 只向 stdout 写 JSON（IOError）、
    /// stderr 为空、退出码 `2`。
    #[test]
    fn execute_run_json_writes_only_json_to_stdout() {
        let missing = std::env::temp_dir().join("lfz_cli_json_missing_98765.lfz");
        let _ = std::fs::remove_file(&missing);
        let p = missing.to_string_lossy().into_owned();
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = execute(&args(&["run", "--json", &p]), &mut out, &mut err);
        assert_eq!(code, EXIT_ERROR);
        let s = String::from_utf8(out).expect("UTF-8");
        assert_eq!(s.lines().count(), 1, "stdout 应只有一行 JSON：{s}");
        assert!(s.starts_with(r#"{"ok":false,"error":"IOError""#), "{s}");
        assert!(!s.contains("Traceback"), "JSON 模式 stdout 不得有人类可读块：{s}");
        assert!(err.is_empty(), "JSON 模式错误诊断不进 stderr（run）：{err:?}");
    }

    /// `--json` 同时出现在子命令前后均被识别；成功路径 stdout 为 `{"ok":true}`。
    #[test]
    fn execute_run_json_success_prints_ok_true() {
        let dir = std::env::temp_dir().join(format!("lfz_cli_json_ok_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("建临时目录");
        let file = dir.join("ok.lfz");
        std::fs::write(&file, "#42\nlet x = 1\n").expect("写文件");
        let p = file.to_string_lossy().into_owned();

        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = execute(&args(&["run", &p, "--json"]), &mut out, &mut err);
        assert_eq!(code, EXIT_OK, "err={err:?}");
        assert_eq!(
            String::from_utf8(out).expect("UTF-8"),
            "{\"ok\":true}\n"
        );
        assert!(err.is_empty(), "err={err:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---- 裸文件调用（P4.4，`lfz <file>`） --------------------------------

    /// 裸文件调用：非 `.lfz` → 友好中文错误（`IOError`），退出码 2，不读文件。
    #[test]
    fn execute_script_rejects_non_lfz() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = execute(&args(&["notes.txt"]), &mut out, &mut err);
        assert_eq!(code, EXIT_ERROR);
        assert!(out.is_empty());
        let e = String::from_utf8(err).expect("UTF-8");
        assert!(e.contains("只支持 .lfz 脚本文件：'notes.txt'"), "{e}");
    }

    /// 裸文件调用 + `--json`：非 `.lfz` → stdout 唯一一行合法 JSON（`IOError`），退出码 2。
    #[test]
    fn execute_script_json_rejects_non_lfz() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = execute(&args(&["--json", "notes.txt"]), &mut out, &mut err);
        assert_eq!(code, EXIT_ERROR);
        let s = String::from_utf8(out).expect("UTF-8");
        assert_eq!(s.lines().count(), 1, "stdout={s:?}");
        assert!(s.starts_with(r#"{"ok":false,"error":"IOError""#), "{s}");
        assert!(s.contains(r#""message":"只支持 .lfz 脚本文件：'notes.txt'""#), "{s}");
        assert!(err.is_empty(), "{err:?}");
    }

    /// 裸文件调用：`.lfz` 成功 → 退出码 0（与 `run` 同源渲染）。
    #[test]
    fn execute_script_lfz_success() {
        let dir = std::env::temp_dir().join(format!("lfz_cli_script_ok_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("建临时目录");
        let file = dir.join("ok.lfz");
        std::fs::write(&file, "#42\nlet x = 1\n").expect("写文件");
        let p = file.to_string_lossy().into_owned();

        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = execute(&args(&[&p]), &mut out, &mut err);
        assert_eq!(code, EXIT_OK, "err={err:?}");
        assert!(out.is_empty(), "非 json 成功不写 stdout: {out:?}");
        assert!(err.is_empty(), "err={err:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
