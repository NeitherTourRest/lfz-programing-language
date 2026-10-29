import hashlib,json,os,subprocess
from pathlib import Path
R=Path.cwd();W=R/'Temp/T16-B-verifier/graph';W.mkdir(parents=True,exist_ok=True);E=R/'docs/reports/T16-B-independent-evidence';X=R/'target/release/lfz.exe';H='8ea375dcd08098ef61d040a57a7b1ad28a28c2fdc25a5ca4921a75b728944480';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert sha(X)==H
env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W));rows=[]
def f(name,body):
 p=W/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(('#42\n'+body).encode());return p
def norm(s):return os.path.normcase(str(s).removeprefix('\\\\?\\'))
def run(name,args,stdin=b'',error=None,message=None,path=None,expected=None,watch=[]):
 before=[(p.read_bytes().hex() if p.exists() else None) for p in watch]
 z=subprocess.run([str(X),*args],input=stdin,cwd=W,env=env,capture_output=True,timeout=15)
 row={'name':name,'args':args,'stdin_hex':stdin.hex(),'exit':z.returncode,'stdout':z.stdout.decode(),'stderr':z.stderr.decode(),'watch_before':before,'watch_after':[(p.read_bytes().hex() if p.exists() else None) for p in watch]};rows.append(row)
 assert z.returncode==(2 if error else 0),row
 if '--json' in args:
  j=json.loads(z.stdout);assert not z.stderr,row;assert j['ok']==(error is None),row
  if error:
   assert j['error']==error,row
   if message:assert message in j['message'],row
   if path:assert norm(j['file'])==norm(path),row
  else:assert j=={'ok':True,'mode':'check','executed':False},row
 else:
  if expected is not None:assert z.stdout==expected,row
  if not error:assert not z.stderr,row
 assert before==row['watch_after'],row
 return row
sent=W/'sentinel.txt';sent.write_bytes(b'keep\x00\r\n');new=W/'must-not-create.txt';assert not new.exists()
leaf=f('leaf.lfz','let value=3\nexport value\nwriteFile('+json.dumps(str(sent)) + ',"changed")\nwriteFile('+json.dumps(str(new))+',"created")\ninput()\nwhile true {}\n')
f('left.lfz','import leaf from "./leaf.lfz"\nexport leaf\n');f('right.lfz','import leaf from "./sub/../leaf.lfz"\nexport leaf\n');(W/'sub').mkdir(exist_ok=True)
body='import left from "./left.lfz"\nimport right from "./right.lfz"\nexport left,right\n';root=f('root.lfz',body)
for mode in ['file','code','stdin']:
 args=['check','--json']+([str(root)] if mode=='file' else ['-e',body] if mode=='code' else ['-'])
 run('diamond_no_init_'+mode,args,body.encode() if mode=='stdin' else b'',watch=[sent,new])
run('human_graph',['check',str(root)],expected=b'',watch=[sent,new])
late=f('late.lfz','writeFile('+json.dumps(str(sent))+',"bad")\nexport nope\n');run('run_preflight',['run','--json',str(late)],error='SyntaxError',message='模块执行尚未启用',watch=[sent])
f('cycle-a.lfz','import b from "./cycle-b.lfz"\n');b=f('cycle-b.lfz','import a from "./cycle-a.lfz"\n');a=W/'cycle-a.lfz';run('cycle',['check','--json',str(a)],error='SyntaxError',message='模块循环依赖',path=b)
bad=f('broken.lfz','let =\n');entry=f('badentry.lfz','import x from "./broken.lfz"\n');r=run('child_syntax',['check','--json',str(entry)],error='SyntaxError',path=bad);assert json.loads(r['stdout'])['line']==2
missing=W/'missing.lfz';entry=f('missingentry.lfz','import x from "./missing.lfz"\n');r=run('missing_child',['check','--json',str(entry)],error='IOError',message='无法读取模块',path=missing);assert json.loads(r['stdout'])['traceback']==[]
entry=f('firsterror.lfz','import x from "./missing.lfz"\nlet x=0\n');run('local_conflict_before_dependency',['check','--json',str(entry)],error='SyntaxError',message='模块导入名冲突',path=entry)
(W/'utf8.lfz').write_bytes(b'\xff');entry=f('utfentry.lfz','import x from "./utf8.lfz"\n');run('child_utf8',['check','--json',str(entry)],error='SyntaxError',path=W/'utf8.lfz')
(W/'nopreamble.lfz').write_bytes(b'let x=1\n');entry=f('nopreentry.lfz','import x from "./nopreamble.lfz"\n');run('child_preamble',['check','--json',str(entry)],error='CosmosAnswerError',path=W/'nopreamble.lfz')
legacy=f('legacy.lfz','let export=4\nexport += 1\nprint(export)\n');run('legacy_export',['run',str(legacy)],expected=b'5\n')
L=2*1024*1024;big=f('huge-legacy.lfz','//'+'x'*(L+1)+'\nprint(42)\n');run('legacy_gt_2m_check',['check','--json',str(big)]);run('legacy_gt_2m_run',['run',str(big)],expected=b'42\n')
lim=W/'limit.lfz';lim.write_bytes(b'#42\n//'+b'x'*(L-6));entry=f('limitentry.lfz','import x from "./limit.lfz"\n');run('dep_exact_2m',['check','--json',str(entry)])
with lim.open('ab') as p:p.write(b'x')
run('dep_over_2m',['check','--json',str(entry)],error='SyntaxError',message='单文件字节',path=entry)
for i in range(128):f('depth/d'+str(i)+'.lfz',('import x from "./d'+str(i+1)+'.lfz"\n') if i<127 else 'let x=0\n')
droot=W/'depth/d0.lfz';run('depth_exact_128',['check','--json',str(droot)])
f('depth/d127.lfz','import x from "./d128.lfz"\n');f('depth/d128.lfz','let x=0\n');run('depth_over_128',['check','--json',str(droot)],error='SyntaxError',message='依赖深度',path=W/'depth/d127.lfz')
start=json.loads((E/'initial-sources.json').read_text());assert start=={p:sha(R/p) for p in start};assert sha(X)==H
result={'date':'2026-10-30','binary_sha256':H,'checks':len(rows),'passed':len(rows),'source_hashes_unchanged':True,'rows':rows};(E/'graph-witnesses.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n');print('graph independent witnesses PASS',len(rows))