import argparse,datetime,hashlib,json,os,re,subprocess
from pathlib import Path
R=Path.cwd(); H='1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c'
a=argparse.ArgumentParser();a.add_argument('--binary',type=Path,required=True);a.add_argument('--output',type=Path,required=True);o=a.parse_args();exe=o.binary.resolve();out=o.output.resolve();assert exe.is_relative_to(R) and out.is_relative_to(R) and not out.exists()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(exe)==H
W=R/'Temp/T15-ai';W.mkdir(parents=True,exist_ok=True);env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1');rows=[]
def run(name,args,error=None,oracle=None):
 assert sha(exe)==H
 p=subprocess.run([str(exe),*args],cwd=R,env=env,capture_output=True,timeout=15)
 if error:
  v=json.loads(p.stdout);assert p.returncode==2 and v['error']==error and not p.stderr,(name,p.stdout,p.stderr)
 else:
  assert p.returncode==0 and not p.stderr,(name,p.returncode,p.stdout,p.stderr)
  if oracle:oracle(p.stdout)
 rows.append(dict(name=name,args=args,exit=p.returncode,stdout=p.stdout.decode('utf8'),stderr=p.stderr.decode('utf8'),passed=True))
paths=['.opencode/skills/lfz-programming/'+n for n in ['SKILL.md','README.md','VERIFICATION.md','prompt-template.md']]+['docs/guide/ai/README.md']
start={n:sha(R/n) for n in paths}
for n in [paths[0],paths[-1]]:
 text=(R/n).read_text(encoding='utf8').split('## 6.3 T15：')[1];code=re.search(r'```lfz\n(.*?)\n```',text,re.S)[1];p=W/('skill.lfz' if n==paths[0] else 'ai.lfz');p.write_text(code+'\n',encoding='utf8',newline='\n')
 run(n+'_check',['check','--json',str(p)],oracle=lambda b: b==b'{"ok":true,"mode":"check","executed":false}\n' or (_ for _ in ()).throw(AssertionError(b)))
 def batch(b):
  v=json.loads(b);assert v['values']==[{'n':2},{'n':5}] and len(v['errors'])==1 and v['errors'][0]['class']=='ValueError'
 run(n+'_run',['run',str(p)],oracle=batch)
cases={
'nil':'let r=attempt(()=>nil)\nassert(r.ok && r.value==nil && r.error==nil)',
'alias':'let x=[1]\nlet r=attempt(()=>x)\nr.value[0]=8\nassert(x[0]==8)',
'function':'let r=attempt(()=>()=>42)\nassert(r.value()==42)',
'argv':'let r=attempt(argv)\nassert(r.ok && len(r.value)==0)',
'pipeline':'let r=(()=>8) |> attempt\nassert(r.ok && r.value==8)',
'shadow':'fn f(){let attempt=(x)=>x+1\n attempt(4)}\nassert(f()==5)',
'nested':'let r=attempt(()=>attempt(3))\nassert(!r.ok && r.error["class"]=="TypeError")',
'repeat':'var i=0\nwhile i<100 {assert(!attempt(()=>1/0).ok)\ni+=1}\nassert(attempt(()=>7).value==7)',
'sideeffect':'let x=[0]\nlet r=attempt(()=>{x[0]=9\nfail("bad")})\nassert(!r.ok && x[0]==9)',
'field':'let r=attempt(()=>({}).missing)\nassert(r.error["class"]=="FieldError")',
'integer':'let r=attempt(()=>9223372036854775807+1)\nassert(r.error["class"]=="OverflowError")',
'frames':'fn bad()=>1/0\nlet r=attempt(bad)\nassert(len(r.error.traceback)==2)\nassert(r.error.traceback[1].func=="bad")\nassert(len(keys(r))==3 && len(keys(r.error))==5)',
'json_failure':'let r=attempt(()=>jsonParse("bad"))\nassert(r.error["class"]=="ValueError")',
'baseline':'assert(sum([1,2,3])==6)\nassert(jsonParse(jsonStringify({a:1})).a==1)',
}
for n,c in cases.items():run(n,['-e',c])
for n,c,err in [('arity','attempt()', 'TypeError'),('callback_arity','attempt((x)=>x)','TypeError'),('nonfunc','attempt(3)','TypeError'),('capacity','attempt(()=>range(9223372036854775807))','OverflowError'),('not_first_class','let f=attempt','NameError'),('builtin_needs_wrapper','attempt(jsonParse)','NameError'),('normal_uncaught','1/0','ZeroDivisionError')]:run(n,['--json','-e',c],error=err)
assert start=={n:sha(R/n) for n in paths}
v=dict(date='2026-10-29',timestamp_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),binary=str(exe.relative_to(R)),binary_sha256=H,checks_total=len(rows),checks_passed=len(rows),document_hashes=start,rows=rows,scope='author new T15 snippets and minimal compatibility, not fresh Agent blind benchmark')
out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n');print(json.dumps({k:v[k] for k in ['checks_total','checks_passed','binary_sha256']}))