from pathlib import Path
import shutil,subprocess
base=Path('Temp/runtime-t12/perf-lab')
oldvalue=subprocess.check_output(['git','show','HEAD:src/value.rs']).decode()
oldstruct=oldvalue[oldvalue.index('#[derive(Clone, Debug, Default)]\npub struct StructObj'):oldvalue.index('// ===========================================================================\n// struct 模板')]
for label in ['no-index','rc-index']:
 root=base/label;(root/'src').mkdir(parents=True,exist_ok=True)
 for p in Path('src').glob('*.rs'):shutil.copyfile(p,root/p)
 for p in ['Cargo.toml','Cargo.lock']:shutil.copyfile(p,root/p)
 p=root/'src/value.rs';s=p.read_text(encoding='utf8')
 if label=='no-index':
  a=s.index('#[derive(Clone, Default)]\npub struct StructObj');b=s.index('// ===========================================================================\n// struct 模板');s=s[:a]+oldstruct+s[b:]
 else:
  s=s.replace('index: HashMap<String, usize>,','index: Rc<HashMap<String, usize>>,')
  s=s.replace('Self { fields, index }','Self { fields, index: Rc::new(index) }')
  s=s.replace('self.index.insert(key.to_string(), self.fields.len());','Rc::make_mut(&mut self.index).insert(key.to_string(), self.fields.len());')
 p.write_text(s,encoding='utf8')
