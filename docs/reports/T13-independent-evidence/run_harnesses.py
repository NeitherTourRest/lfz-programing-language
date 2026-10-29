import hashlib,json,os,re,subprocess
from pathlib import Path
R=Path.cwd();E=R/'docs/reports/T13-independent-evidence';W=R/'Temp/T13-verifier'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
log=(R/'docs/reports/T13-runtime-evidence/cargo-test-release.txt').read_text(encoding='utf8')
paths=re.findall(r'Running [^\n]+\((target\\release\\deps\\[^)]+\.exe)\)',log)
assert len(paths)==6,paths
expected=[412,48,28,12,15,15]
env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1')
rows=[]
for index,(name,count) in enumerate(zip(paths,expected)):
 p=R/Path(name.replace('\\','/'))
 r=subprocess.run([str(p)],cwd=R,env=env,capture_output=True,timeout=120)
 text=(r.stdout+r.stderr).decode('utf8')
 outfile=E/f'unit-harness-{index+1}.txt';outfile.write_bytes(r.stdout+r.stderr)
 assert r.returncode==0,(p,r.returncode,text[-2000:])
 assert f'{count} passed; 0 failed; 0 ignored' in text,(p,text[-1000:])
 rows.append({'binary':p.relative_to(R).as_posix(),'binary_sha256':sha(p),'passed':count,'exit':r.returncode,'raw_output':outfile.relative_to(R).as_posix(),'output_sha256':sha(outfile)})
meta={'date':'2026-10-27','method':'execute six exact release test harnesses named by author cargo-test log; no compilation; integration harnesses use baked release executable','passed':sum(expected),'groups':rows}
(E/'unit-harnesses.json').write_text(json.dumps(meta,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
print(json.dumps({'passed':sum(expected),'groups':expected,'compiled':False}))
