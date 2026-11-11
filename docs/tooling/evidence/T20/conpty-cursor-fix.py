from pathlib import Path
D=Path('Temp/T20-tooling')
for name in ['conpty-final.py','conpty-errors.py']:
 s=(D/name).read_text(encoding='utf-8').replace("return len(value)","return value.index(fragment,start)+len(fragment)")
 new=name.replace('.py','-cursor-fixed.py');s=s.replace(name[:-3],new[:-3]);(D/new).write_text(s,encoding='utf-8')
