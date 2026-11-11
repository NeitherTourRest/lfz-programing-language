import pathlib,hashlib,json,subprocess,os
r=pathlib.Path.cwd(); old=r/'docs/reports/T19-independent-evidence'; w=r/'Temp/T20-verifier'; out=w/'regression';out.mkdir(exist_ok=True); h='a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c'; sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest(); exe=r/'target/release/lfz.exe';assert sha(exe)==h
paths=sorted([*r.glob('src/**/*.rs'),*r.glob('tests/**/*.rs'),r/'Cargo.toml',r/'Cargo.lock']); snapshot=lambda:{p.relative_to(r).as_posix():sha(p) for p in paths}; before=snapshot();(out/'sources-before.json').write_text(json.dumps(before,indent=2)); env=dict(os.environ,TEMP=str(w),TMP=str(w),PYTHONDONTWRITEBYTECODE='1',RUSTUP_HOME=str(r/'Temp/toolchain/rustup'),CARGO_HOME=str(r/'Temp/toolchain/cargo'))
rows=[]
sets=[*json.loads((old/'commands.json').read_text(encoding='utf8')),*json.loads((old/'extra-commands.json').read_text(encoding='utf8')),dict(name='stream14-final',args=['D:/app/ANACONDA/python.exe','-B',str(old/'stream14-final.py'),'--lfz',str(exe),'--expected-sha',h,'--output',str(old/'stream14-final.json')])]
for v in sets:
 n=v['name']; cmd=list(v.get('command') or v['args']);cmd=[h if a.startswith('5d2219f8') else a for a in cmd]; cmd[cmd.index('--output')+1]=str(out/(n+'.json'))
 f=pathlib.Path(cmd[2]); source=None
 if f.parent.resolve()==old:
  src=f.read_text(encoding='utf8');source=sha(f);s=out/f.name;s.write_text(src.replace('Temp/T19-verifier','Temp/T20-verifier/regression').replace('Temp/T12/verifier-rerun','Temp/T20-verifier/regression/json140'),encoding='utf8');cmd[2]=str(s)
 p=subprocess.run(cmd,cwd=r,env=env,capture_output=True,timeout=120); row=dict(name=n,command=cmd,exit=p.returncode,stdout=p.stdout.decode('utf8','replace'),stderr=p.stderr.decode('utf8','replace'),driver_original_sha=source);rows.append(row);print(n,p.returncode,flush=True);(out/'commands.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2),encoding='utf8')
assert sha(exe)==h and snapshot()==before
assert all(x['exit']==0 for x in rows)
