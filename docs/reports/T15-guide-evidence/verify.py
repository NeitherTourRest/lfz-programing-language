import argparse,hashlib,json,os,re,subprocess,datetime
from pathlib import Path
R=Path.cwd(); a=argparse.ArgumentParser();a.add_argument('--binary',type=Path,required=True);o=a.parse_args();X=o.binary.resolve();H='1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(X)==H
W=R/'Temp/T15-docs';W.mkdir(parents=True,exist_ok=True)
env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1')
rows=[];files=[R/'docs/guide'/n for n in ['README.md','reference.md','tutorial.md','errors.md','testing.md','error-recovery.md']];before={str(p.relative_to(R)):sha(p)for p in files}
def run(name,args,expected='',code=0,error=None):
 assert sha(X)==H
 p=subprocess.run([str(X),*args],capture_output=True,cwd=R,env=env,timeout=15)
 row={'name':name,'args':args,'exit':p.returncode,'stdout':p.stdout.decode('utf8'),'stderr':p.stderr.decode('utf8')};rows.append(row)
 assert p.returncode==code,row
 if error:assert json.loads(p.stdout)['error']==error,row
 else:assert row['stdout'].replace('\r\n','\n')==expected and not row['stderr'],row
 row['passed']=True
lesson=(R/'docs/guide/error-recovery.md').read_text(encoding='utf8');source=re.search(r'```lfz\n(.*?)\n```',lesson,re.S).group(1);job=W/'job.lfz';job.write_text(source+'\n',encoding='utf8',newline='\n')
run('tutorial_check',['check',str(job)])
run('tutorial_run',['run',str(job)],'{"accepted":3,"rejected":1,"values":[{"n":1},null,{"n":2}]}\n')
cases={
 'nil':'let r=attempt(()=>nil)\nassert(r.ok && r.value==nil && r.error==nil)',
 'parse':'let text="bad"\nlet r=attempt(()=>jsonParse(text))\nassert(!r.ok && r.error["class"]=="ValueError")',
 'overflow':'let r=attempt(()=>9223372036854775807+1)\nassert(!r.ok && r.error["class"]=="OverflowError")',
 'type':'let r=attempt(()=>1+true)\nassert(!r.ok && r.error["class"]=="TypeError")',
 'field':'let r=attempt(()=>({}).missing)\nassert(!r.ok && r.error["class"]=="FieldError")',
 'divide':'let r=attempt(()=>1/0)\nassert(!r.ok && r.error["class"]=="ZeroDivisionError")',
 'assertion':'let r=attempt(()=>assert(false))\nassert(!r.ok && r.error["class"]=="AssertionError")',
 'nested':'let r=attempt(()=>attempt(1))\nassert(!r.ok && r.error["class"]=="TypeError")',
 'argv':'let r=attempt(argv)\nassert(r.ok && len(r.value)==0)',
 'pipeline':'let f=()=>7\nlet r=f |> attempt\nassert(r.ok && r.value==7)',
 'alias':'let x=[1]\nlet r=attempt(()=>x)\nr.value[0]=8\nassert(x[0]==8)',
 'side_effect':'let x=[1]\nlet r=attempt(()=>{x[0]=9\n fail("bad")})\nassert(!r.ok && x[0]==9)',
 'function':'let r=attempt(()=>()=>3)\nassert(r.ok && r.value()==3)',
 'repeat':'var i=0\nwhile i<100 {let r=attempt(()=>1/0)\n assert(!r.ok)\n i+=1}\nassert(attempt(()=>42).value==42)',
}
for name,src in cases.items():run(name,['-e',src])
for name,src in {'outer_arity':'attempt()','nonfunc':'attempt(1)','callback_arity':'attempt((x)=>x)','argument_outside':'attempt(jsonParse("bad"))'}.items():run(name,['--json','-e',src],code=2,error='ValueError'if name=='argument_outside' else 'TypeError')
assert before=={str(p.relative_to(R)):sha(p)for p in files}
result={'date':'2026-10-29','timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'binary':str(X.relative_to(R)),'binary_sha256':H,'binary_bytes':X.stat().st_size,'guides':before,'checks_total':len(rows),'checks_passed':sum(x.get('passed',False)for x in rows),'rows':rows}
p=R/'docs/reports/T15-guide-evidence/runs.json';p.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n');print(json.dumps({'total':len(rows),'passed':result['checks_passed']}))
