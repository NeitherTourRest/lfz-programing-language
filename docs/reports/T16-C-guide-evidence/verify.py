import argparse,datetime,hashlib,json,os,re,subprocess
from pathlib import Path
R=Path.cwd();a=argparse.ArgumentParser();a.add_argument('--binary',type=Path);a.add_argument('--expected-sha256');a.add_argument('--output',type=Path);a.add_argument('--prepare-only',action='store_true');o=a.parse_args()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
files=[R/'docs/guide'/n for n in ['README.md','reference.md','tutorial.md','errors.md','testing.md','modules.md']];before={str(p.relative_to(R)):sha(p)for p in files}
W=R/'Temp/T16-C-docs';W.mkdir(parents=True,exist_ok=True)
blocks=re.findall(r'```lfz\n(.*?)\n```',(R/'docs/guide/modules.md').read_text(encoding='utf8'),re.S);assert len(blocks)==3
for n,b in zip(['calc.lfz','worker.lfz','main.lfz'],blocks):(W/n).write_text(b+'\n',encoding='utf8',newline='\n')
if o.prepare_only:print('prepared 3 extracted tutorial files; interpreter launches 0');raise SystemExit
assert o.binary and o.expected_sha256 and o.output
X=o.binary.resolve();out=o.output.resolve();assert X.is_relative_to(R) and out.is_relative_to(R) and not out.exists();assert sha(X)==o.expected_sha256
rows=[];env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1')
def run(name,args,expected=None,code=0,error=None):
 assert sha(X)==o.expected_sha256
 p=subprocess.run([str(X),*args],cwd=R,env=env,capture_output=True,timeout=15)
 row={'name':name,'args':args,'exit':p.returncode,'stdout':p.stdout.decode('utf8'),'stderr':p.stderr.decode('utf8')};rows.append(row);out.parent.mkdir(parents=True,exist_ok=True);out.with_suffix('.partial.json').write_text(json.dumps({'binary_sha256':o.expected_sha256,'rows':rows},ensure_ascii=False,indent=2)+'\n',encoding='utf8');assert p.returncode==code,row
 if error:
  j=json.loads(p.stdout);assert j['error']==error and not p.stderr,row
 elif expected is not None:assert row['stdout'].replace('\r\n','\n')==expected and not p.stderr,row
 row['passed']=True;return row
main=str(W/'main.lfz')
run('tutorial_check',['check','--json',main],'{"ok":true,"mode":"check","executed":false}\n')
for i in range(2):run('tutorial_run_'+str(i),['run',main,'--','中文'],'{"answer":43,"args":["中文"]}\n')
# Namespace alias, snapshot, home globals and captured module argv.
(W/'home.lfz').write_text('#42\nvar n=1\nlet xs=[3]\nfn inc(){n+=1\n n}\nfn read()=>n\nfn arguments()=>argv()\nexport n,xs,inc,read,arguments\n',encoding='utf8')
(W/'aliases.lfz').write_text('#42\nimport a from "./home.lfz"\nimport b from "./home.lfz"\nvar n=900\nassert(a.inc()==2)\nassert(a.n==1)\na.n=7\nassert(b.n==7 && a.read()==2)\na.xs[0]=8\nassert(b.xs[0]==8)\nassert(a.arguments()==argv())\n',encoding='utf8')
run('namespace_home_argv',['run',str(W/'aliases.lfz'),'--','x','中文'],'')
# Exported template defaults use definition module; explicit override uses caller.
(W/'model.lfz').write_text('#42\nvar n=5\nstruct Point {x:n, xs:[0]}\nexport Point\n',encoding='utf8')
(W/'templates.lfz').write_text('#42\nimport lib from "./model.lfz"\nvar n=900\nlet P=lib.Point\nlet first=P{}\nlet second=P{x:n}\nassert(first.x==5 && second.x==900)\nfirst.xs[0]=7\nassert(second.xs[0]==0)\n',encoding='utf8')
run('template_context',['run',str(W/'templates.lfz')],'')
# Check must not run module initialization IO; run initializes diamond once.
marker=W/'marker.txt';marker.unlink(missing_ok=True)
(W/'leaf.lfz').write_text('#42\nappendFile('+json.dumps(marker.as_posix())+',"x")\nlet answer=1\nexport answer\n',encoding='utf8')
for n in ['left','right']:(W/(n+'.lfz')).write_text('#42\nimport leaf from "./leaf.lfz"\nexport leaf\n',encoding='utf8')
(W/'diamond.lfz').write_text('#42\nimport left from "./left.lfz"\nimport right from "./right.lfz"\nassert(left.leaf.answer==right.leaf.answer)\n',encoding='utf8')
run('diamond_check',['check',str(W/'diamond.lfz')],'');assert not marker.exists()
run('diamond_run',['run',str(W/'diamond.lfz')],'');assert marker.read_bytes()==b'x'
# Bad export binding and cycle are graph failures before initialization.
(W/'missing-export.lfz').write_text('#42\nexport missing\n',encoding='utf8')
run('missing_export',['check','--json',str(W/'missing-export.lfz')],code=2,error='SyntaxError')
(W/'a.lfz').write_text('#42\nimport b from "./b.lfz"\n',encoding='utf8');(W/'b.lfz').write_text('#42\nimport a from "./a.lfz"\n',encoding='utf8')
run('cycle',['check','--json',str(W/'a.lfz')],code=2,error='SyntaxError')
# Real dependency runtime error source; attempt data intentionally has no file.
(W/'fail.lfz').write_text('#42\nfn bad()=>1/0\nexport bad\n',encoding='utf8')
(W/'failed.lfz').write_text('#42\nimport lib from "./fail.lfz"\nlib.bad()\n',encoding='utf8')
r=run('cross_file_fault',['run','--json',str(W/'failed.lfz')],code=2,error='ZeroDivisionError');j=json.loads(r['stdout']);assert j['file'].endswith('fail.lfz') and j['traceback'][-1]['file'].endswith('fail.lfz')
(W/'recover.lfz').write_text('#42\nimport lib from "./fail.lfz"\nlet r=attempt(()=>lib.bad())\nassert(!r.ok && !has("file",r.error))\nassert(len(keys(r.error))==5)\nassert(attempt(()=>3).value==3)\n',encoding='utf8')
run('attempt_cross_module',['run',str(W/'recover.lfz')],'')
assert before=={str(p.relative_to(R)):sha(p)for p in files};assert sha(X)==o.expected_sha256
result={'date':'2026-10-30','timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'binary':str(X.relative_to(R)),'sha256':o.expected_sha256,'bytes':X.stat().st_size,'guides':before,'passed':len(rows),'total':len(rows),'rows':rows,'marker_hex':marker.read_bytes().hex()}
out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n');print(json.dumps({'passed':len(rows),'output':str(out.relative_to(R))}))
