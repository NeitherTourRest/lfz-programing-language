from pathlib import Path
import hashlib,json,zipfile,shutil
R=Path.cwd();W=R/'Temp/T20-tooling';E=R/'docs/tooling/evidence/T20';E.mkdir(exist_ok=True)
for p in W.iterdir():
 if p.is_file():shutil.copy2(p,E/p.name)
(E/'.gitattributes').write_text('* -text\n',encoding='utf-8');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
rows=json.loads((W/'watched-summary.json').read_text(encoding='utf-8'));assert len(rows)==13 and all(r['exit']==0 for r in rows) and sum(r['passed'] for r in rows)==705
exe=R/'target/release/lfz.exe';expected='a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c';assert sha(exe)==expected
for name,code in [('conpty-final-cursor-fixed-result.json',0),('conpty-errors-ascii-result.json',2)]:
 d=json.loads((W/name).read_text(encoding='utf-8'));assert d['status']=='PASS' and d['exit']==code and d['binary_sha256']==expected
sources=sorted((R/'src').rglob('*.rs'))+sorted((R/'tests').rglob('*.rs'))+[R/'Cargo.toml',R/'Cargo.lock']
with zipfile.ZipFile(E/'candidate-snapshot.zip','x',zipfile.ZIP_DEFLATED) as z:
 for p in sources+[exe]:z.write(p,p.relative_to(R).as_posix())
snapshot={'members':len(sources)+1,'bytes':(E/'candidate-snapshot.zip').stat().st_size,'sha256':sha(E/'candidate-snapshot.zip')};(E/'snapshot.json').write_text(json.dumps(snapshot,indent=2)+'\n',encoding='utf-8')
d={'date':'2026-11-04','status':'author PASS; public/independent/Agent evaluation pending','binary':{'path':'target/release/lfz.exe','bytes':exe.stat().st_size,'sha256':sha(exe)},'tests':{'total':705,'failed':0,'ignored':0,'harnesses':rows,'watchdog_seconds_per_harness':90,'threads':1,'conpty_normal':{'result':'conpty-final-cursor-fixed-result.json','exit':0},'conpty_error_reset':{'result':'conpty-errors-ascii-result.json','exit':2},'conpty_watchdog_seconds':15,'tty_unicode':'full Chinese tty not passed; original codepage and 65001 driver failures retained; exact Chinese pipe/writer passed'},'source_hashes':{p.relative_to(R).as_posix():sha(p) for p in sources},'evidence':{p.name:sha(p) for p in sorted(E.iterdir()) if p.is_file() and p.name!='manifest.json'}}
(E/'manifest.json').write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'binary':d['binary'],'snapshot':snapshot}))
