import hashlib,json,os,re,subprocess
from pathlib import Path
R=Path.cwd();E=R/'docs/reports/T17-independent-evidence';W=R/'Temp/T17-verifier';X=R/'target/release/lfz.exe';H='1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(X)==sha(R/'target/release/lfz.exe')==H
start={str(p.relative_to(R)):sha(p) for p in list((R/'src').glob('*.rs'))+[R/'Cargo.toml',R/'Cargo.lock']}
E.mkdir(parents=True,exist_ok=True);W.mkdir(parents=True,exist_ok=True)
env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1')
def proc(args):
 p=subprocess.run(args,cwd=R,env=env,capture_output=True,timeout=40);return p
log=(R/'docs/tooling/evidence/T17/compile.txt')
if not log.exists():log=next((R/'docs/reports/T16-A-runtime-evidence').glob('*full*'))
s=log.read_text(encoding='utf8');paths=re.findall(r'Running [^\n]+\((target\\release\\deps\\[^)]+\.exe)\)',s)
paths=['target/release/deps/'+p.name[7:-4] for p in sorted((R/'docs/tooling/evidence/T17').glob('final2-*.exe.txt'))]
assert len(paths)==10,paths
hs=[]
for name in paths:
 p=proc([str(R/name)]);out=p.stdout.decode();assert p.returncode==0,(name,out,p.stderr);n=int(re.search(r'test result: ok\. (\d+) passed',out)[1]);hs.append({'path':name,'sha256':sha(R/name),'passed':n,'stdout':out,'stderr':p.stderr.decode()})
assert sum(r['passed'] for r in hs)==624
(E/'harnesses.json').write_text(json.dumps(hs,ensure_ascii=False,indent=2),encoding='utf8')
py='D:/app/ANACONDA/python.exe'
commands=[('attempt13',[py,'-B','tests/t15_cli_check.py','--binary',str(X),'--expected-sha256',H,'--output',str(E/'attempt13.json')]),('check117',[py,'-B','tests/t14_cli_check.py','--binary',str(X),'--output',str(E/'check117.json')]),('cli91',[py,'-B','tests/t12_cli_check.py','--lfz',str(X),'--output',str(E/'cli91.json'),'--full-suite']),('jsonflow',[py,'-B','app/verify_jsonflow.py','--lfz',str(X),'--output',str(E/'jsonflow.json')]),('sortviz',[py,'-B','app/verify_sortviz.py','--lfz',str(X),'--before-exe','Temp/t12-perf/lfz-before.exe','--output',str(E/'sortviz.json')]),('jsonbatch',[py,'-B','app/verify_jsonbatch.py','--lfz',str(X),'--expected-sha',H,'--output',str(E/'jsonbatch.json')]),('independent140',[py,'-B','docs/reports/T17-independent-evidence/check140.py','--output',str(E/'independent140.json')])]
rs=[]
for name,args in commands:
 p=proc(args);rs.append({'name':name,'command':args,'exit':p.returncode,'stdout':p.stdout.decode(),'stderr':p.stderr.decode()});assert p.returncode==0,rs[-1];print(name,'PASS',flush=True)
(E/'commands.json').write_text(json.dumps(rs,ensure_ascii=False,indent=2),encoding='utf8')
(E/'initial-sources.json').write_text(json.dumps(start,indent=2),encoding='utf8')
print('624 harness and all public suites PASS')
