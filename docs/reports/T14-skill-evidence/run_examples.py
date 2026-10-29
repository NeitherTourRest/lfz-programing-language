import argparse,hashlib,json,os,subprocess,datetime
from pathlib import Path
R=Path(__file__).resolve().parents[3]
ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);a=ap.parse_args()
X=a.binary.resolve();O=a.output.resolve();W=R/'Temp/T14-ai';W.mkdir(parents=True,exist_ok=True)
assert X.is_relative_to(R) and O.is_relative_to(R) and not O.exists()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
H=sha(X);assert H=='83f56a194e216862c45377f4c3971b085744a7ba2363163e1edac6af5e85ac79'
env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1');rows=[]
def run(name,args,stdin=None,exit=0,stdout=None,error=None,usage=False,check=False):
 assert sha(X)==H
 p=subprocess.run([str(X),*args],input=stdin,capture_output=True,cwd=R,env=env,timeout=5)
 row=dict(name=name,args=args,stdin_hex=stdin.hex() if stdin is not None else None,exit=p.returncode,stdout_hex=p.stdout.hex(),stderr_hex=p.stderr.hex())
 assert p.returncode==exit,row
 if stdout is not None:assert p.stdout==stdout,row
 if check:
  if '--json' in args and not usage:
   d=json.loads(p.stdout);assert list(d)[:3]==['ok','mode','executed'] and d['mode']=='check' and d['executed'] is False and d['ok']==(exit==0),row
   if error:assert d['error']==error,row
   else:assert d==dict(ok=True,mode='check',executed=False),row
   assert not p.stderr and p.stdout.count(b'\n')==1,row
  elif exit==0:assert not p.stdout and not p.stderr,row
 if usage:assert not p.stdout and p.stderr,row
 rows.append(row)
job=W/'job.lfz';job.write_text('#42\nfn sum_even(xs) { xs |> filter((x) => x % 2 == 0) |> reduce((a,x) => a+x,0) }\nprint(sum_even([1,2,3,4,5]))\n',encoding='utf8',newline='\n')
run('workflow_check',['check',str(job)],check=True)
run('workflow_run',['run',str(job)],stdout=b'6\n')
for source in ['print(1 / 0)','unknownName','1 + true','readFile("missing-do-not-open")','while true {}','fn f()=>f()\nf()','input("no prompt")',';;','assert(false)','check(false,"not called")']:
 run('nonexecution_'+str(len(rows)),['check','--json','-e',source],check=True)
run('stdin_example',['check','--json','-'],b'print(1 / 0)',check=True)
for source in ['', '--json','--help','check']:
 run('code_preserved_'+source,['check','-e',source,'--json'],check=True)
run('code_dashdash',['check','-e','--','--json'],exit=2,error='SyntaxError',check=True)
run('syntax_error',['check','--json','-e','return 1'],exit=2,error='SyntaxError',check=True)
for args in [['check','--json'],['check','--json','--help'],['check','--json',str(job),'--'],['check','--json','-e','print(1)','--','arg']]:
 run('usage_'+str(len(rows)),args,exit=2,usage=True,check=True)
sentinel=W/'sentinel.txt';sentinel.write_bytes(b'keep\x00bytes');missing=W/'must-not-create.txt'
if missing.exists():raise AssertionError('fresh sentinel path required')
source='writeFile("'+sentinel.as_posix()+'","changed")\nappendFile("'+missing.as_posix()+'","new")\n'
run('io_not_executed',['check','--json','-e',source],check=True)
assert sentinel.read_bytes()==b'keep\x00bytes' and not missing.exists()
run('run_divzero',['--json','-e','print(1 / 0)'],exit=2)
assert json.loads(bytes.fromhex(rows[-1]['stdout_hex']))['error']=='ZeroDivisionError'
run('run_json_unchanged',['--json','-e','print(1)'],stdout=b'{"ok":true}\n')
run('builtin_check_unchanged',['-e','check(false,"soft")\nprint("continue")'],stdout=b'continue\n')
assert bytes.fromhex(rows[-1]['stderr_hex']).decode('utf8').strip()=='check 失败：soft'
files=['.opencode/skills/lfz-programming/'+f for f in ['SKILL.md','README.md','VERIFICATION.md','prompt-template.md']]+['docs/guide/ai/README.md']
result=dict(date='2026-10-29',timestamp_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),binary=str(X.relative_to(R)),binary_sha256=H,binary_bytes=X.stat().st_size,total=len(rows),passed=len(rows),kind='author examples and compatibility; not a new Agent blind benchmark',rows=rows)
O.parent.mkdir(parents=True,exist_ok=True);O.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n');print(json.dumps({k:v for k,v in result.items() if k!='rows'},ensure_ascii=False))
