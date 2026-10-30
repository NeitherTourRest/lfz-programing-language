import argparse,datetime,hashlib,json,os,re,subprocess
from pathlib import Path
R=Path.cwd();H='cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006';a=argparse.ArgumentParser();a.add_argument('--binary',type=Path,required=True);a.add_argument('--output',type=Path,required=True);o=a.parse_args();X=o.binary.resolve();O=o.output.resolve();assert X.is_relative_to(R) and O.is_relative_to(R) and not O.exists()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert sha(X)==H
W=R/'Temp/T16-C-ai';W.mkdir(parents=True,exist_ok=True);env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1');rows=[]
names=['.opencode/skills/lfz-programming/'+n for n in ['SKILL.md','README.md','VERIFICATION.md','prompt-template.md']]+['docs/guide/ai/README.md'];docs={n:sha(R/n) for n in names}
def put(n,s):p=W/n;p.write_text(s,encoding='utf8',newline='\n');return p
def save():
 O.parent.mkdir(parents=True,exist_ok=True);O.write_text(json.dumps(dict(date='2026-10-30',timestamp_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),binary=str(X.relative_to(R)),binary_sha256=H,document_hashes=docs,checks_total=len(rows),checks_passed=sum(x['passed'] for x in rows),rows=rows),ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
def run(n,args,expected=None,error=None,stdin=None):
 assert sha(X)==H;p=subprocess.run([str(X),*args],cwd=W,env=env,input=stdin,capture_output=True,timeout=15);out=p.stdout.decode('utf8');err=p.stderr.decode('utf8');ok=p.returncode==(2 if error else 0)
 if error:
  try:v=json.loads(out);ok=ok and v['error']==error and not err
  except Exception:ok=False
 else:ok=ok and not err and (expected is None or out==expected)
 rows.append(dict(name=n,args=args,exit=p.returncode,stdout=out,stderr=err,passed=ok));save();return p
CHECK='{"ok":true,"mode":"check","executed":false}\n'
for i,n in enumerate([names[0],names[-1]]):
 s=(R/n).read_text(encoding='utf8').split('## 6.4 T16-C：')[1];b=re.findall(r'```lfz\n(.*?)\n```',s,re.S);assert len(b)==2
 put('counter.lfz',b[0]+'\n');p=put('main.lfz',b[1]+'\n');run('extracted_check_'+str(i),['check','--json',str(p)],CHECK)
 for j in range(2):run('extracted_run_'+str(i)+'_'+str(j),['run',str(p),'--','中文'],'{"answer":43,"args":["中文"],"shared":[7]}\n')
put('home.lfz','#42\nvar n=1\nlet xs=[3]\nfn inc(){n+=1\n n}\nfn read()=>n\nfn arguments()=>argv()\nexport n,xs,inc,read,arguments\n')
p=put('aliases.lfz','#42\nimport a from "./home.lfz"\nimport b from "./home.lfz"\nvar n=900\nassert(a.inc()==2 && a.n==1)\na.n=7\nassert(b.n==7 && a.read()==2)\na.xs[0]=8\nassert(b.xs[0]==8 && a.arguments()==argv())\n');run('aliases_home_shared_argv',['run',str(p),'--','x','中文'],'')
put('model.lfz','#42\nvar n=5\nstruct Point {x:n, xs:[]}\nexport Point\n');p=put('templates.lfz','#42\nimport lib from "./model.lfz"\nvar n=900\nlet P=lib.Point\nlet a=P{}\nlet b=P{x:n}\nassert(a.x==5 && b.x==900)\na.xs=push(1,a.xs)\nassert(len(b.xs)==0)\n');run('template_definition_defaults_and_caller_override',['run',str(p)],'')
marker=W/'init.txt';marker.unlink(missing_ok=True)
put('leaf.lfz','#42\nappendFile('+json.dumps(marker.as_posix())+',"x")\nlet value=1\nexport value\n')
for n in ['left','right']:put(n+'.lfz','#42\nimport leaf from "./leaf.lfz"\nexport leaf\n')
p=put('diamond.lfz','#42\nimport a from "./left.lfz"\nimport b from "./right.lfz"\nassert(a.leaf.value==1 && b.leaf.value==1)\n');run('diamond_check_no_init',['check','--json',str(p)],CHECK);rows[-1]['passed'] &= not marker.exists();save()
run('diamond_run_once',['run',str(p)],'');rows[-1]['passed'] &= marker.exists() and marker.read_bytes()==b'x';save()
marker.unlink(missing_ok=True);run('new_session_reinitializes',['run',str(p)],'');rows[-1]['passed'] &= marker.exists() and marker.read_bytes()==b'x';save()
put('badlib.lfz','#42\nfn bad()=>1/0\nexport bad\n');p=put('fault.lfz','#42\nimport lib from "./badlib.lfz"\nlib.bad()\n');v=run('cross_file_cli_diagnostic',['run','--json',str(p)],error='ZeroDivisionError')
if v.returncode==2 and v.stdout:
 try:q=json.loads(v.stdout);rows[-1]['passed'] &= q['file'].replace('\\','/').endswith('/badlib.lfz') and q['traceback'][-1]['file'].replace('\\','/').endswith('/badlib.lfz');save()
 except Exception:pass
p=put('recover.lfz','#42\nimport lib from "./badlib.lfz"\nlet r=attempt(()=>lib.bad())\nassert(!r.ok && r.error["class"]=="ZeroDivisionError")\nassert(!has("file",r.error))\nfor f in r.error.traceback {assert(!has("file",f))}\nassert(attempt(()=>42).value==42)\n');run('attempt_no_file_and_context_restore',['run',str(p)],'')
run('code_root_cwd',['-e','import lib from "./counter.lfz"\nassert(lib.add(2)==42)'],'')
run('stdin_root_cwd',['-'],expected='',stdin=b'import lib from "./counter.lfz"\nassert(lib.add(2)==42)\n')
put('cycle.lfz','#42\nimport a from "./cycle.lfz"\n');run('cycle_before_init',['check','--json',str(W/'cycle.lfz')],error='SyntaxError')
p=put('missing.lfz','#42\nimport lib from "./counter.lfz"\nlib.hidden\n');run('unexported_runtime_field',['run','--json',str(p)],error='FieldError')
run('legacy_compatibility',['-e','assert(sum([1,2,3])==6)\nassert(attempt(()=>7).value==7)'],'')
assert docs=={n:sha(R/n) for n in names};save();print(json.dumps({'checks_total':len(rows),'checks_passed':sum(x['passed'] for x in rows),'output':str(O.relative_to(R))}));raise SystemExit(0 if all(x['passed']for x in rows) else 1)