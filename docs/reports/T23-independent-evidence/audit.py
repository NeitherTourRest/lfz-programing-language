from pathlib import Path
from hashlib import sha256
import json,re
r=Path.cwd(); d=r/'docs/reports/T23-terminal-evidence'; t=r/'Temp/T23-emoji'; out=r/'docs/reports/T23-independent-evidence'
h=lambda p:sha256(p.read_bytes()).hexdigest(); read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
exe=h(r/'dist/lfz.exe'); assert exe=='76ac4cb51fe8ff1a67cd697540e483f50759e4fd67dce00636781481d492f11e'
markers=[f'CASE{i}<{s}>' for i,s in enumerate(['\u4e2d\u6587\u96ea','\U0001f600','\U00020000','e\u0301','\U0001f469\u200d\U0001f4bb'])]
esc=re.compile(r'\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07]*\x07)'); source=[]; con=[]; native=[]
for cp in (437,936,65001):
 for kind in ('wide','run','repl'):
  tag=f'{cp}-{kind}'; raw=d/f'{tag}.raw.bin'; ans=read(d/f'{tag}.result.json'); modes=read(d/f'{tag}.handles.json'); text=esc.sub('',raw.read_bytes().decode('utf8',errors='strict')); lines=[q for q in text.splitlines() if q]
  assert ans['binary_sha256']==exe and ans['exit']==0
  assert modes['supported'] and all(modes[k]==cp for k in ('input_cp','output_cp','requested_cp')) and len(modes['handles'])==3
  assert all(x['mode_ok'] and x['access']=='0xc0000000' and x['error']==0 for x in modes['handles'])
  assert [s for s in lines if s.startswith('CASE')]==markers and all(text.count(m)==1 for m in markers),lines
  item=dict(cp=cp,kind=kind,raw_sha256=h(raw),captured_sample_lines=5)
  if kind=='repl':
   payload=d/f'{tag}.input.bin'; q=payload.read_bytes().decode('ascii',errors='strict'); exprs=q.split('\r'); assert exprs[-2:]==[':quit',''] and len(exprs)==7; assert all(exprs[i]==f'print("CASE{i}<"+argv()[{i}]+">")' for i in range(5)); assert all(exprs[i] in text for i in range(5)); assert all(m not in q for m in markers); item['input_sha256']=h(payload)
  con.append(item)
 a=d/f'{cp}-ordinary-out.utf16le'; b=d/f'{cp}-wide.utf16le'; s=a.read_bytes(); assert s==b.read_bytes() and len(s)==948
 assert s.decode('utf-16-le',errors='strict').rstrip()=='EMOJI<\ufffd> CJK<\ufffd> BMP<\u4e2d\u6587\u96ea>'
 native.append(dict(cp=cp,bytes=len(s),identical_sha256=h(a)))
m=read(d/'ordinary-console.json'); assert m['binary_sha256']==exe and len(m['results'])==6; assert all(x['type']==2 and x['console_mode'] for x in m['handles']['ordinary']); assert all(x['actual_cp']==x['cp'] and x['exit']==0 and h(d/f"{x['cp']}-{x['variant']}.utf16le")==x['raw_sha256'] for x in m['results'])
for p in sorted(d.iterdir()):
 if (t/p.name).is_file():assert h(p)==h(t/p.name),p.name;source.append(dict(name=p.name,bytes=p.stat().st_size,sha256=h(p)))
assert len(source)==50,len(source)
(out/'audit.json').write_text(json.dumps(dict(binary_sha256=exe,archived_inputs=source,conpty=con,native=native),ensure_ascii=False,indent=2)+'\n',encoding='utf8');print('45 exact terminal lines, 3 equal UTF-16 pairs, 50 matching archived inputs')

