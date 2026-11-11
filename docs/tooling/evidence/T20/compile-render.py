from pathlib import Path
p=Path('src/cli.rs');s=p.read_text(encoding='utf-8');s=s.replace('{use std::io::BufRead;let stdin=std::io::stdin();','{let stdin=std::io::stdin();')
s=s.replace('  pending.push_str(&line);','  if poisoned{failed=true;let _=writeln!(err,"repl: REPL 会话需要 :reset");if !interactive{break;}continue;}\n  pending.push_str(&line);')
s=s.replace('let eval=graph_run_result(traced);if report_eval("<repl>"','let eval=repl_run_result(traced);if report_eval("<repl>"')
s+='''
fn repl_run_result(traced:TracedRun)->CaseEval{
 if traced.frames.is_empty()&&traced.result.is_err(){
  let Some(origin)=traced.error_source()else{return CaseEval::HostFailure("REPL来源元数据缺失".into());};
  let path=origin.display_path.clone();let loaded=origin.loaded.clone();let err=(**traced.result.as_ref().unwrap_err()).clone();
  let inner=with_http_diagnostics(with_diagnostics(CaseEval::Failed{err,loaded,traced:None},traced.host_diagnostics().to_vec()),traced.http_diagnostics().to_vec());
  CaseEval::Located{path,inner:Box::new(inner)}
 }else{graph_run_result(traced)}
}
''';p.write_text(s,encoding='utf-8')
p=Path('tests/cli_t20.rs');s=p.read_text(encoding='utf-8').replace('.filter(|l|*l=="2").count(),3)', '.filter(|l|*l=="2").count(),2)');s='\n'.join(l for l in s.split('\n') if not l.startswith(' fn file('));p.write_text(s,encoding='utf-8')
