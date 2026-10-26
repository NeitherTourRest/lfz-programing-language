from pathlib import Path
import shutil,subprocess
base=Path('Temp/runtime-t12/perf-lab')
for label,oldfile in [('old-eval-old-value','src/value.rs'),('old-eval-old-builtins','src/builtins.rs'),('old-source-new-version',None)]:
 root=base/label;(root/'src').mkdir(parents=True,exist_ok=True)
 src=base/('old' if oldfile is None else 'old-evaluator-new-rest')
 for p in (src/'src').glob('*.rs'):shutil.copyfile(p,root/'src'/p.name)
 for p in ['Cargo.toml','Cargo.lock']:shutil.copyfile(p,root/p)
 if oldfile: (root/oldfile).write_bytes(subprocess.check_output(['git','show','HEAD:'+oldfile]))
