import hashlib,json,os,subprocess
from pathlib import Path
R=Path.cwd();E=R/'docs/reports/T15-independent-evidence';W=R/'Temp/T15-verifier';X=R/'dist/lfz.exe';H='1a5a3d1c650440929bb599961763573def8b56d33811a8f266688246ac63dd6c'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1')
cases=[
('nil_and_function','let a=attempt(()=>nil)\nassert(a.ok && a.value==nil && a.error==nil)\nlet b=attempt(()=>()=>7)\nassert(b.value()==7)'),
('nested_boundary','let a=attempt(()=>attempt(4))\nassert(!a.ok && a.error["class"]=="TypeError")\nlet b=attempt(()=>attempt(()=>fail("nested")))\nassert(b.ok && !b.value.ok)'),
('alias_and_rebound','let xs=[1]\nlet a=attempt(()=>{xs[0]=8\nfail("changed")})\nassert(!a.ok && xs[0]==8)\nlet b=attempt(()=>xs)\nb.value[0]=9\nassert(xs[0]==9)'),
('closure_ids','var saved=()=>0\nlet a=attempt(()=>{fn escaped()=>1/0\nsaved=escaped\nfail("save")})\nlet b=attempt(saved)\nassert(b.error.traceback[-1].func=="escaped")\nfn later()=>9\nassert(attempt(later).value==9)'),
('trace_data_independent','fn bad()=>missing_probe\nlet a=attempt(bad)\na.error.traceback[0].func="changed"\nlet b=attempt(bad)\nassert(b.error.traceback[0].func=="<module>")\nassert(b.error.traceback[-1].func=="bad")'),
('many_errors_then_recursion','var i=0\nwhile i<12000 {let r=attempt(()=>1/0)\nassert(!r.ok)\ni+=1}\nfn fac(n)=>if n==0 {1} else {n*fac(n-1)}\nassert(attempt(()=>fac(8)).value==40320)'),
('argv_and_pipeline','assert(attempt(argv).value==["x","中文",""])\nlet r=(()=>jsonParse("true")) |> attempt\nassert(r.ok && r.value)'),
('shadow','fn attempt(x)=>x+10\nassert(attempt(2)==12)'),
('captured_self','let obj={n:8,f:()=>attempt(()=>self.n)}\nassert(obj.f().value==8)'),
('input_validation_depth','let text=repeat(129,"[")+"0"+repeat(129,"]")\nassert(attempt(()=>jsonParse(text)).error["class"]=="ValueError")'),
('overflow_business','assert(attempt(()=>9223372036854775807+1).error["class"]=="OverflowError")'),
('left_to_right_once','var n=0\nfn arg(){n+=1\n()=>n}\nassert(attempt(arg()).value==1 && n==1)'),
]
rows=[]
def call(name,code,cls=None):
 for machine in [False,True]:
  assert sha(X)==H
  args=[str(X),*(['--json'] if machine else []),'-e',code,'--','x','中文','']
  p=subprocess.run(args,env=env,cwd=R,capture_output=True,timeout=12)
  expected=2 if cls else 0
  assert p.returncode==expected,(name,p.stdout,p.stderr)
  if machine:
   d=json.loads(p.stdout);assert d['ok']==(cls is None)
   if cls:assert d['error']==cls
  elif cls:assert cls.encode() in p.stderr
  else:assert not p.stdout and not p.stderr,(name,p.stdout,p.stderr)
  rows.append({'name':name,'json':machine,'source':code,'exit':p.returncode,'stdout':p.stdout.decode(),'stderr':p.stderr.decode()})
for n,c in cases:call(n,c)
for n,c,cl in [('nonfunc','attempt(2)','TypeError'),('arg_eval','attempt(1/0)','ZeroDivisionError'),('callback_arity','attempt((x)=>x)','TypeError'),('capacity','attempt(()=>range(9223372036854775807))','OverflowError'),('nested_capacity','attempt(()=>attempt(()=>range(9223372036854775807)))','OverflowError'),('syntax','attempt(()=>{let =})','SyntaxError')]:call(n,c,cl)
# Future uncaught stack must match identical program without the recovered call.
for machine in [False,True]:
 code='fn gone()=>1/0\nlet r=attempt(gone)\nfn fresh()=>missing_probe\nfresh()'
 p=subprocess.run([str(X),*(['--json'] if machine else []),'-e',code],env=env,cwd=R,capture_output=True,timeout=10)
 assert p.returncode==2
 if machine:
  d=json.loads(p.stdout);assert [f['func'] for f in d['traceback']]==['<module>','fresh'],d
 else:assert b'in fresh' in p.stderr and b'in gone' not in p.stderr
 rows.append({'name':'late_error_clean','json':machine,'source':code,'exit':p.returncode,'stdout':p.stdout.decode(),'stderr':p.stderr.decode()})
(E/'witnesses.json').write_text(json.dumps({'date':'2026-10-29','sha256':H,'passed':len(rows),'rows':rows},ensure_ascii=False,indent=2),encoding='utf8')
print('independent witnesses',len(rows),'PASS')
