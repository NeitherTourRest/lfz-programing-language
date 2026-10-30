import hashlib,json,os,subprocess
from pathlib import Path
R=Path.cwd();E=R/'docs/reports/T17-independent-evidence';W=R/'Temp/T17-verifier/own';W.mkdir(parents=True,exist_ok=True);X=R/'target/release/lfz.exe'
H='1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert sha(X)==H
rows=[];env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W))
def run(name,args,code=0,message=None,file=None):
 p=subprocess.run([str(X),'--json',*args],capture_output=True,env=env,timeout=10);d=json.loads(p.stdout);r=dict(name=name,args=list(map(str,args)),exit=p.returncode,stdout=p.stdout.decode('utf8'),stderr_hex=p.stderr.hex(),document=d);rows.append(r)
 assert p.returncode==code,r
 if message:assert d['error']=='ExecutionLimitError' and d['message']==message,r
 if file:assert d['file'].removeprefix('\\\\?\\').lower()==str(file.resolve()).lower(),r
 r['passed']=True
step='执行步数预算已耗尽';dead='执行时间上限已到'
for n in [0,1,2]:run('exact_'+str(n),['--max-steps',str(n),'-e','1'],2 if n<2 else 0,step if n<2 else None)
run('no_final_poll',['--max-steps','4','-e','let x=1\nx'])
run('empty_no_poll',['--max-steps','0','--timeout-ms','0','-e',''])
run('priority',['--max-steps','0','--timeout-ms','0','-e','1'],2,dead)
run('nested_attempt',['--max-steps','35','-e','attempt(()=>attempt(()=>{while true {}}))'],2,step)
run('recoverable_no_refund',['--max-steps','60','-e','while true {attempt(()=>1/0)}'],2,step)
run('cooperative_deadline',['--timeout-ms','10','-e','while true {}'],2,dead)
files={'leaf':'let x=1\nexport x','left':'import leaf from "./leaf.lfz"\nexport leaf','right':'import leaf from "./leaf.lfz"\nexport leaf','main':'import left from "./left.lfz"\nimport right from "./right.lfz"\n1'}
for name,text in files.items():(W/(name+'.lfz')).write_text('#42\n'+text+'\n',encoding='utf8')
run('diamond11',['--max-steps','11',str(W/'main.lfz')])
run('diamond10',['--max-steps','10',str(W/'main.lfz')],2,step,W/'main.lfz')
(W/'lib.lfz').write_text('#42\nstruct P{x:if true {while true {}}}\nexport P\n',encoding='utf8');(W/'default.lfz').write_text('#42\nimport lib from "./lib.lfz"\nlet P=lib.P\nattempt(()=>P{})\n',encoding='utf8')
run('default_source',['--max-steps','50',str(W/'default.lfz')],2,step,W/'lib.lfz');assert '<defaults>' in str(rows[-1]['document']['traceback'])
(W/'lib.lfz').write_text('#42\nfn f(){while true {}}\nexport f\n',encoding='utf8');(W/'call.lfz').write_text('#42\nimport lib from "./lib.lfz"\nattempt(()=>lib.f())\n',encoding='utf8')
run('function_source',['--max-steps','40',str(W/'call.lfz')],2,step,W/'lib.lfz')
run('fresh_session',['--max-steps','2','-e','1'])
(E/'own.json').write_text(json.dumps(dict(binary_sha256=sha(X),total=len(rows),passed=sum(x['passed'] for x in rows),rows=rows),ensure_ascii=False,indent=2),encoding='utf8');print(len(rows),'independent witnesses PASS')
