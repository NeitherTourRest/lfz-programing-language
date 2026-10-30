import hashlib,json,os,re,subprocess
from pathlib import Path
R=Path.cwd();E=R/'docs/reports/T18-A-independent-evidence';W=R/'Temp/T18-A-verifier';X=R/'target/release/lfz.exe';H='4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(X)==sha(R/'target/release/lfz.exe')==H
start={str(p.relative_to(R)):sha(p) for p in list((R/'src').glob('*.rs'))+[R/'Cargo.toml',R/'Cargo.lock']}
E.mkdir(parents=True,exist_ok=True);W.mkdir(parents=True,exist_ok=True)
env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1',RUSTUP_HOME=str(R/'Temp/toolchain/rustup'),CARGO_HOME=str(R/'Temp/toolchain/cargo'))
def proc(args):
 p=subprocess.run(args,cwd=R,env=env,capture_output=True,timeout=90);return p
manifest=json.loads((R/'docs/tooling/evidence/T18-A-close/manifest.json').read_text(encoding='utf8'))
paths=['target/release/deps/'+v['name'] for v in manifest['tests']['permitted_harnesses']]
hs=[]
for name in paths:
 p=proc([str(R/name),"--test-threads=1"]);out=p.stdout.decode();assert p.returncode==0,(name,out,p.stderr);n=int(re.search(r'test result: ok\. (\d+) passed',out)[1]);hs.append({'path':name,'sha256':sha(R/name),'passed':n,'stdout':out,'stderr':p.stderr.decode()})
assert sum(r['passed'] for r in hs)==649
(E/'harnesses.json').write_text(json.dumps(hs,ensure_ascii=False,indent=2),encoding='utf8')
py='D:/app/ANACONDA/python.exe'
commands=[('attempt13',[py,'-B','tests/t15_cli_check.py','--binary',str(X),'--expected-sha256',H,'--output',str(E/'attempt13.json')]),('check117',[py,'-B','tests/t14_cli_check.py','--binary',str(X),'--output',str(E/'check117.json')]),('cli91',[py,'-B','tests/t12_cli_check.py','--lfz',str(X),'--output',str(E/'cli91.json'),'--full-suite']),('jsonflow',[py,'-B','app/verify_jsonflow.py','--lfz',str(X),'--output',str(E/'jsonflow.json')]),('sortviz',[py,'-B','app/verify_sortviz.py','--lfz',str(X),'--before-exe','Temp/t12-perf/lfz-before.exe','--output',str(E/'sortviz.json')]),('jsonbatch',[py,'-B','app/verify_jsonbatch.py','--lfz',str(X),'--expected-sha',H,'--output',str(E/'jsonbatch.json')]),('independent140',[py,'-B','docs/reports/T18-A-independent-evidence/check140.py','--output',str(E/'independent140.json')])]
rs=[]
for name,args in commands:
 p=proc(args);rs.append({'name':name,'command':args,'exit':p.returncode,'stdout':p.stdout.decode(),'stderr':p.stderr.decode()});assert p.returncode==0,rs[-1];print(name,'PASS',flush=True)
(E/'commands.json').write_text(json.dumps(rs,ensure_ascii=False,indent=2),encoding='utf8')
assert all(sha(R/p)==h for p,h in json.loads((E/'initial-sources.json').read_text()).items())
print('649 harness and all public suites PASS')
