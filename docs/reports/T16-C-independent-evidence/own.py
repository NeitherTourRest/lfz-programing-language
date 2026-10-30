import json,hashlib,os,subprocess
from pathlib import Path
R=Path.cwd();E=R/'docs/reports/T16-C-independent-evidence';W=R/'Temp/T16-C-verifier/own';W.mkdir(parents=True,exist_ok=True);X=R/'target/release/lfz.exe';H='cdedc9b980aa207b28bf895ef95ce76fde18e47ef758c19c187b3f9c2da08006';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert sha(X)==H
env=dict(os.environ,TEMP=str(W),TMP=str(W));rows=[]
def file(home,name,src):
 p=W/home/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text('#42\n'+src+'\n',encoding='utf8',newline='\n');return p

def case(name,files,root,expected='',error_file=None,mode='run',data=None,args=(),json_mode=False):
 for n,s in files.items():file(name,n,s)
 p=file(name,'main.lfz',root);cmd=[mode]+(['--json']if json_mode else [])+(['-e',root] if data=='eval' else ['-']if data=='stdin' else [str(p)])+(['--',*args]if args else [])
 z=subprocess.run([str(X),*cmd],input=root.encode() if data=='stdin' else b'',cwd=p.parent,env=env,capture_output=True,timeout=15)
 row={'name':name,'command':cmd,'exit':z.returncode,'stdout':z.stdout.decode(),'stderr':z.stderr.decode(),'sources':files|{'main.lfz':root}};rows.append(row)
 assert z.returncode==(2 if error_file else 0),row
 if error_file:
  j=json.loads(z.stdout);assert os.path.normcase(j['file'].removeprefix(chr(92)*2+'?'+chr(92)))==os.path.normcase(str((p.parent/error_file).resolve())),row;assert not z.stderr,row
 elif json_mode:
  j=json.loads(z.stdout);assert j==({'ok':True,'mode':'check','executed':False}if mode=='check' else {'ok':True}),row
 else:assert z.stdout.decode()==expected and not z.stderr,row
 return row
files={'leaf.lfz':'var n=0\nlet data=[1]\nfn bump(){n+=1\nn}\nexport data,bump','l.lfz':'import x from "./leaf.lfz"\nexport x','r.lfz':'import x from "./leaf.lfz"\nexport x'}
case('diamond',files,'import l from "./l.lfz"\nimport r from "./r.lfz"\nassert(l.x.bump()==1)\nassert(r.x.bump()==2)\nl.x.data[0]=7\nassert(r.x.data[0]==7)\nprint("same")','same\n')
case('home',{'lib.lfz':'var x=7\nfn get()=>x\nfn escape()=>()=>x\nexport x,get,escape'},'import m from "./lib.lfz"\nvar x=99\nm.x=40\nassert(m.get()==7)\nassert(m.escape()()==7)\nprint(m.x)','40\n')
case('template',{'lib.lfz':'var n=4\nfn tick(){n+=1\nn}\nstruct P{x:tick(),data:[0]}\nfn count()=>n\nexport P,count'},'import m from "./lib.lfz"\nlet P=m.P\nvar n=100\nlet a=P{x:n}\nlet b=P{}\nassert(a.x==100 && b.x==6 && m.count()==6)\na.data[0]=9\nassert(b.data[0]==0)\nprint("defaults")','defaults\n')
case('local_template',{'lib.lfz':'fn make(){var seed=4\nstruct P{x:seed}\nseed=8\nP}\nexport make'},'import m from "./lib.lfz"\nlet P=m.make()\nvar seed=99\nassert(P{}.x==8)\nprint("cell")','cell\n')
case('attempt_home',{'lib.lfz':'let x=7\nfn bad()=>1/0\nfn get()=>x\nexport bad,get'},'import m from "./lib.lfz"\nvar x=100\nlet r=attempt(m.bad)\nassert(!r.ok && r.error["class"]=="ZeroDivisionError")\nassert(m.get()==7)\nprint(x)','100\n')
r=case('after_catch',{'lib.lfz':'fn bad()=>1/0\nexport bad'},'import m from "./lib.lfz"\nlet r=attempt(m.bad)\nunknown_after',error_file='main.lfz',json_mode=True);assert all(f['func']!='bad' for f in json.loads(r['stdout'])['traceback'])
case('default_source',{'lib.lfz':'struct P{x:1/0}\nexport P'},'import m from "./lib.lfz"\nlet P=m.P\nP{}',error_file='lib.lfz',json_mode=True)
case('override_source',{'lib.lfz':'struct P{x:8}\nexport P'},'import m from "./lib.lfz"\nlet P=m.P\nP{x:1/0}',error_file='main.lfz',json_mode=True)
for source in ['eval','stdin']:
 case('argv_'+source,{'lib.lfz':'fn get()=>argv()\nexport get'},'import m from "./lib.lfz"\nassert(m.get()==["中",""])\nprint("args")','args\n',data=source,args=('中',''))
# Multiple process executions of the same source identity must start fresh.
for i in range(2):case('session_fresh',{'lib.lfz':'var n=0\nfn bump(){n+=1\nn}\nexport bump'},'import m from "./lib.lfz"\nprint(m.bump())','1\n')
case('init_failure',{'lib.lfz':'fail("init")\nlet x=1\nexport x'},'import m from "./lib.lfz"\nprint("entry-ran")',error_file='lib.lfz',json_mode=True)
rec={'lib.lfz':'fn dive(n)=>if n==0 {1/0} else {dive(n-1)}\nexport dive'}
r=case('folded',rec,'import m from "./lib.lfz"\nm.dive(45)',error_file='lib.lfz',json_mode=True);assert len(json.loads(r['stdout'])['traceback'])==47,r
p=W/'folded/main.lfz';z=subprocess.run([str(X),'run',str(p)],capture_output=True,cwd=p.parent,env=env,timeout=15);row={'name':'folded_human','exit':z.returncode,'stdout':z.stdout.decode(),'stderr':z.stderr.decode()};rows.append(row);assert z.returncode==2 and not z.stdout and row['stderr'].count('  File ')==40,row
start=json.loads((E/'initial-sources.json').read_text());assert start=={p:sha(R/p)for p in start};assert sha(X)==H
(E/'own.json').write_text(json.dumps({'date':'2026-10-30','hash':H,'passed':len(rows),'source_hashes_unchanged':True,'rows':rows},ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n');print('Independent module witnesses PASS',len(rows))