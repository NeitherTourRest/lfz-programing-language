from pathlib import Path
p=Path('src/cli.rs');s=p.read_text(encoding='utf-8-sig');s=s.replace('    Version,','    Version,\n    Repl { args:Vec<String> },')
s=s.replace('out: &mut dyn Write, err: &mut dyn Write) -> i32 {\n    let (rest', 'out: &mut (dyn Write+Send), err: &mut (dyn Write+Send)) -> i32 {\n    let (rest')
s=s.replace('    let parsed=parse_command(&rest,script_args);','''    let parsed=parse_command(&rest,script_args);
    if let Ok(Command::Repl{args})=&parsed {
        if json{return usage_error("repl 不支持 --json",err);}
        let config=evaluator::ReplConfig{context:evaluator::EvalContext{limits:evaluator::EvalLimits{max_steps:limits.max_steps,..Default::default()},capabilities:evaluator::EvalCapabilities{subprocess:limits.allow_process,http:limits.allow_http}},args:args.clone(),timeout_ms:limits.timeout_ms};
        use std::io::IsTerminal;
        let interactive=std::io::stdin().is_terminal();
        return evaluator::on_eval_stack(move ||repl_loop(config,interactive,out,err,read_repl_line));
    }''')
s=s.replace('        Ok(Command::Help) => {','        Ok(Command::Repl{..}) => unreachable!(),\n        Ok(Command::Help) => {')
s=s.replace('    if first == "test" {', '''    if first == "repl" {if args.len()!=1{return Err("repl 不接受额外来源".into());}return Ok(Command::Repl{args:script_args.unwrap_or_default()});}
    if first == "test" {''')
s=s.replace('  lfz --help, -h', '  lfz repl [-- ARG...]  持久REPL，多行自动续读；:help/:reset/:quit\n  lfz --help, -h')
s+='''
const REPL_HELP:&str="命令: :help :reset :quit\\n多行未完成时继续读取；完整输入立即执行。if 的 } 与 else 须在同一物理行。\\ninput() 消费同一stdin下一物理行；限额按cell重新计，能力和argv按session固定。\\n";
const REPL_CELL_CAP:usize=1048576;
fn read_repl_line(remaining:usize)->Result<Option<Vec<u8>>,String>{use std::io::BufRead;let stdin=std::io::stdin();let mut input=stdin.lock();read_repl_physical(&mut input,remaining)}
fn read_repl_physical(input:&mut impl std::io::BufRead,remaining:usize)->Result<Option<Vec<u8>>,String>{let mut line=Vec::new();loop{let block=input.fill_buf().map_err(|e|format!("读取stdin失败：{e}"))?;if block.is_empty(){return Ok(if line.is_empty(){None}else{Some(line)});}let take=block.iter().position(|b|*b==b'\\n').map_or(block.len(),|n|n+1);if take>remaining-line.len(){return Err("REPL 会话容量超限：cell bytes（上限 1048576）".into());}if line.len()+take>line.capacity(){line.reserve_exact(take);}line.extend_from_slice(&block[..take]);let done=block[take-1]==b'\\n';input.consume(take);if done{return Ok(Some(line));}}}
fn repl_loop(config:evaluator::ReplConfig,interactive:bool,out:&mut dyn Write,err:&mut dyn Write,mut read:impl FnMut(usize)->Result<Option<Vec<u8>>,String>)->i32{
 let mut session=match evaluator::ReplSession::new(config){Ok(s)=>s,Err(e)=>{let _=writeln!(err,"repl: {e}");return EXIT_ERROR;}};let mut pending=String::new();let mut failed=false;let mut poisoned=false;
 loop{
  if interactive{if write!(err,"{}",if pending.is_empty(){"lfz> "}else{"...> "}).and_then(|_|err.flush()).is_err(){return EXIT_ERROR;}}
  let line=match read(REPL_CELL_CAP-pending.len()){Ok(v)=>v,Err(e)=>{let _=writeln!(err,"repl: {e}");return EXIT_ERROR;}};
  let eof=line.is_none();let line=match line{Some(bytes)=>match String::from_utf8(bytes){Ok(line)=>line,Err(_)=>{let _=writeln!(err,"repl: stdin 不是合法 UTF-8");return EXIT_ERROR;}},None=>String::new()};
  if eof&&pending.is_empty(){break;}
  if pending.is_empty()&&!eof&&line.trim().starts_with(':'){
   match line.trim(){":quit"=>break,":reset"=>{session.reset();poisoned=false;},":help"=>{if write!(out,"{REPL_HELP}").and_then(|_|out.flush()).is_err(){return EXIT_ERROR;}},_=>{failed=true;let _=writeln!(err,"repl: 未知命令");}}continue;
  }
  pending.push_str(&line);
  if !eof&&matches!(parser::probe_repl_input(&pending),Ok(parser::ReplInputStatus::Incomplete)){continue;}
  if poisoned{failed=true;let _=writeln!(err,"repl: REPL 会话需要 :reset");pending.clear();if !interactive{break;}continue;}
  match session.eval_cell(&pending){
   Ok(None)=>{},Ok(Some(traced))=>{
    poisoned=!traced.host_diagnostics().is_empty()||!traced.http_diagnostics().is_empty()||matches!(&traced.result,Err(e)if matches!(e.as_ref(),LzError::ExecutionLimit{reason:lfz::error::ExecutionLimitReason::ReplCapacity,..}));
    let value=traced.result.as_ref().ok().filter(|v|!matches!(v,lfz::value::Value::Nil)).map(|v|v.to_string());
    let eval=graph_run_result(traced);if report_eval("<repl>",&eval,false,out,err)!=EXIT_OK{failed=true;}else if let Some(value)=value{if writeln!(out,"{value}").and_then(|_|out.flush()).is_err(){return EXIT_ERROR;}}
    if poisoned&&!interactive{let _=writeln!(err,"repl: REPL 会话需要 :reset");return EXIT_ERROR;}
   },Err(e)=>{failed=true;let fatal=matches!(e,evaluator::ReplHostError::Poisoned|evaluator::ReplHostError::Origins(_)|evaluator::ReplHostError::InvalidConfig{..});let _=writeln!(err,"repl: {e}");if fatal&&!interactive{return EXIT_ERROR;}}
  }
  pending.clear();if eof{break;}
 }
 if failed{EXIT_ERROR}else{EXIT_OK}
}
'''
p.write_text(s,encoding='utf-8')
