from pathlib import Path
import shutil,subprocess
base=Path('Temp/runtime-t12/perf-lab')
for label in ['old-evaluator-new-rest','old-cli-new-lib']:
 root=base/label;(root/'src').mkdir(parents=True,exist_ok=True)
 for p in Path('src').glob('*.rs'):shutil.copyfile(p,root/p)
 for p in ['Cargo.toml','Cargo.lock']:shutil.copyfile(p,root/p)
 if label=='old-cli-new-lib':
  for path in ['src/cli.rs','src/main.rs']: (root/path).write_bytes(subprocess.check_output(['git','show','HEAD:'+path]))
 else:
  p=root/'src/evaluator.rs';s=subprocess.check_output(['git','show','HEAD:src/evaluator.rs']).decode()
  s+='''\n// DIAGNOSTIC ONLY: ignores argv, not a contract implementation or release candidate.\npub fn eval_module_traced_with_args(program: &Program, _args: &[String]) -> TracedRun { eval_module_traced(program) }\npub fn eval_module_traced_on_thread_with_args(program: &Program, _args: &[String]) -> TracedRun { eval_module_traced_on_thread(program) }\n''';p.write_text(s,encoding='utf8')
