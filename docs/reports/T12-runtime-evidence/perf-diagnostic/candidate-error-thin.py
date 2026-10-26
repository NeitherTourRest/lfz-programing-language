from pathlib import Path
import shutil
base=Path('Temp/runtime-t12/perf-lab')
for label in ['old-only-error','new-thin']:
 root=base/label;(root/'src').mkdir(parents=True,exist_ok=True)
 if label=='old-only-error':
  for p in (base/'old/src').glob('*.rs'):shutil.copyfile(p,root/'src'/p.name)
  shutil.copyfile('src/error.rs',root/'src/error.rs')
 else:
  for p in Path('src').glob('*.rs'):shutil.copyfile(p,root/p)
 for p in ['Cargo.toml','Cargo.lock']:shutil.copyfile(p,root/p)
 if label=='new-thin':
  p=root/'Cargo.toml';s=p.read_text(encoding='utf8').replace('\nlto = true','\nlto = "thin"');p.write_text(s,encoding='utf8')
