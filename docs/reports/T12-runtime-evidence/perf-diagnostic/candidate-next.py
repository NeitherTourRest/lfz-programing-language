from pathlib import Path
import shutil
base=Path('Temp/runtime-t12/perf-lab')
for label in ['inline-hot','boxed-context']:
 root=base/label;(root/'src').mkdir(parents=True,exist_ok=True)
 for p in Path('src').glob('*.rs'):shutil.copyfile(p,root/p)
 for p in ['Cargo.toml','Cargo.lock']:shutil.copyfile(p,root/p)
 p=root/'src/evaluator.rs';s=p.read_text(encoding='utf8')
 if label=='inline-hot':
  for marker in ['    fn lookup(', '    fn assign_name(', '    fn eval_expr(', '    fn exec_stmt(']:
   assert marker in s;s=s.replace(marker,'    #[inline(always)]\n'+marker)
 else:
  marker='struct Interp {';assert marker in s;s=s.replace(marker,'struct ScriptContext { args: Vec<Value>, function: Value }\n\n'+marker)
  marker='    script_args: Vec<Value>,\n    argv_function: Value,';assert marker in s;s=s.replace(marker,'    script_context: Box<ScriptContext>,')
  marker='            script_args: Vec::new(),\n            argv_function: Value::function(argv_closure),';assert marker in s;s=s.replace(marker,'            script_context: Box::new(ScriptContext { args: Vec::new(), function: Value::function(argv_closure) }),')
  s=s.replace('interp.script_args', 'interp.script_context.args').replace('self.script_args','self.script_context.args').replace('self.argv_function','self.script_context.function')
 p.write_text(s,encoding='utf8')
