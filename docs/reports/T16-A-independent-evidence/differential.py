import hashlib,json,os,subprocess
from pathlib import Path
R=Path.cwd();W=R/'Temp/T16-A-verifier';E=R/'docs/reports/T16-A-independent-evidence'
old=W/'lfz-before.exe';new=R/'target/release/lfz.exe';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(old)=='1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c'
assert sha(new)=='65a29eaea54f32d1d2b404f22aed09389c3ffa71e6a81acb8fa6351d32fa42b7'
env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1')
cases=[('plain','print(1+2)',None),('closure','fn outer(x)=>()=>x\nlet f=outer(9)\nprint(f())',None),('method','let o={x:7,f:()=>self.x}\nprint(o.f())',None),('argv','print(jsonStringify(argv()))',None),('pipe','print(range(4) |> map((x)=>x+1) |> sum)',None),('dump','let x=3\n;;',None),('nil','print(jsonStringify(attempt(()=>nil)))',None),('json_error','print(jsonStringify(attempt(()=>jsonParse("{"))))',None),('nested','print(jsonStringify(attempt(()=>attempt(()=>fail("bad")))))',None),('alias','let x=[1]\nlet r=attempt(()=>{x[0]=4\nfail("changed")})\nprint(x[0])',None),('escaped','var f=()=>0\nlet r=attempt(()=>{fn gone()=>1/0\nf=gone\nfail("save")})\nprint(jsonStringify(attempt(f)))',None),('fresh_error','let r=attempt(()=>fail("caught"))\nfn later()=>1/0\nlater()','ZeroDivisionError'),('arity','attempt((x)=>x)','TypeError'),('capacity','attempt(()=>range(9223372036854775807))','OverflowError'),('unknown','unknown','NameError'),('syntax','let =','SyntaxError')]
rows=[]
for name,body,error in cases:
 p=W/'same-source.lfz';p.write_text('#42\n'+body+'\n',encoding='utf8',newline='\n')
 for mode in ['human','json']:
  args=['run']+(['--json'] if mode=='json' else [])+[str(p),'--','x','中文','']
  values=[]
  for label,exe in [('old',old),('new',new)]:
   z=subprocess.run([str(exe),*args],cwd=R,env=env,capture_output=True,timeout=15)
   assert z.returncode==(2 if error else 0),(name,label,z.returncode,z.stdout,z.stderr)
   if mode=='json':
    j=json.loads(z.stdout);assert j['ok']==(error is None)
    if error:assert j['error']==error
   values.append({'label':label,'exit':z.returncode,'stdout_hex':z.stdout.hex(),'stderr_hex':z.stderr.hex()})
  assert {k:v for k,v in values[0].items() if k!='label'}=={k:v for k,v in values[1].items() if k!='label'},(name,mode,values)
  rows.append({'name':name,'mode':mode,'source':body,'expected_error':error,'outputs':values,'identical':True})
start=json.loads((E/'initial-sources.json').read_text());end={p:sha(R/p) for p in start};assert start==end
result={'date':'2026-10-29','old_sha256':sha(old),'new_sha256':sha(new),'programs':len(cases),'comparisons':len(rows),'processes':len(rows)*2,'passed':len(rows),'source_hashes_unchanged':True,'source_hashes':end,'rows':rows}
(E/'differential.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
print(json.dumps({k:v for k,v in result.items() if k not in ('rows','source_hashes')}))