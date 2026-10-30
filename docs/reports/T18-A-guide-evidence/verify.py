import argparse,datetime,hashlib,json,os,re,shutil,subprocess
from pathlib import Path
R=Path.cwd();ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--expected-sha256',required=True);ap.add_argument('--output',type=Path,required=True);a=ap.parse_args()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
X=a.binary.resolve();O=a.output.resolve();assert X.is_relative_to(R)and O.is_relative_to(R)and not O.exists();assert sha(X)==a.expected_sha256
W=R/'Temp/T18-A-docs';W.mkdir(parents=True,exist_ok=True);F=W/'guide-fixture.exe';original=R/'Temp/T18-A-tooling/fixture.exe';FH='51b83f054f128e381fa034adbe980d6398f24f76621703ba27da4d7e54d27bee';assert sha(original)==FH;shutil.copyfile(original,F);assert sha(F)==FH
q=lambda s:json.dumps(str(s),ensure_ascii=False)
fs=q(F.as_posix());rows=[];env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1');guides=[R/'docs/guide'/n for n in ['README.md','reference.md','tutorial.md','errors.md','testing.md','subprocess.md']];before={str(p.relative_to(R)):sha(p)for p in guides}
def run(name,args,code=0,stdout=None,error=None,message=None):
 assert sha(X)==a.expected_sha256 and sha(F)==FH
 p=subprocess.run([str(X),*args],capture_output=True,cwd=R,env=env,timeout=15)
 row={'name':name,'args':args,'exit':p.returncode,'stdout':p.stdout.decode('utf8'),'stderr':p.stderr.decode('utf8')};rows.append(row);O.parent.mkdir(parents=True,exist_ok=True);O.with_suffix('.partial.json').write_text(json.dumps({'binary_sha256':a.expected_sha256,'fixture_sha256':FH,'rows':rows},ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 assert p.returncode==code,row
 if error:
  j=json.loads(p.stdout);assert j['error']==error and not p.stderr,row
  if message:assert j['message']==message,row
 elif message:assert not p.stdout and row['stderr'].splitlines()[0]=='lfz: '+message,row
 elif stdout is not None:assert row['stdout'].replace('\r\n','\n')==stdout and not p.stderr,row
 row['passed']=True
source=re.search(r'```lfz\n(.*?)\n```',(R/'docs/guide/subprocess.md').read_text(encoding='utf8'),re.S).group(1);P=W/'tool.lfz';P.write_text(source+'\n',encoding='utf8',newline='\n')
run('tutorial_check',['check','--json',str(P)],stdout='{"ok":true,"mode":"check","executed":false}\n')
expected=json.dumps({'exitCode':0,'signal':None,'stderr':'','stdout':'0:\n3:a b\n3:雪\n'},ensure_ascii=False,separators=(',',':'))+'\n'
run('tutorial_run',['--allow-process',str(P),'--',str(F)],stdout=expected)
def call(mode,extra='',timeout=1000,limit=10000):return 'exec(['+fs+','+q(mode)+extra+'],'+str(timeout)+','+str(limit)+')'
def execsrc(n,s,allow=True,**kwargs):run(n,['--json']+(['--allow-process']if allow else[])+['-e',s],**kwargs)
execsrc('four_keys_nonzero','let r='+call('exit',',"7"')+'\nassert(keys(r)==["exitCode","signal","stderr","stdout"])\nassert(r.exitCode==7 && r.signal==nil && r.stdout=="" && r.stderr=="")',stdout='{"ok":true}\n')
marker=W/'marker.txt';marker.unlink(missing_ok=True)
execsrc('default_disabled',call('mark',','+q(marker.as_posix()),limit=0),allow=False,code=2,error='ValueError',message='子进程能力未启用');assert not marker.exists()
execsrc('stdin_EOF','assert('+call('stdin')+'.stdout=="0")',stdout='{"ok":true}\n')
execsrc('raw_utf8','assert('+call('utf8')+'.stdout=="\ufeffa\\r\\n")',stdout='{"ok":true}\n')
execsrc('bad_utf8',call('invalid'),code=2,error='ValueError',message='子进程输出不是合法 UTF-8')
execsrc('zero_quiet',call('exit',',"0"',limit=0),stdout='{"ok":true}\n')
execsrc('combined_equal',call('dual',',"3"',limit=6),stdout='{"ok":true}\n')
execsrc('combined_over',call('dual',',"3"',limit=5),code=2,error='ValueError',message='子进程输出超过字节上限')
execsrc('local_timeout_caught','let r=attempt(()=>'+call('sleep',',"500"',timeout=10)+')\nassert(!r.ok && r.error.message=="子进程执行时间上限已到")',stdout='{"ok":true}\n')
run('global_deadline_not_caught',['--json','--allow-process','--timeout-ms','30','-e','attempt(()=>'+call('sleep',',"500"',timeout=1000)+')'],code=2,error='ExecutionLimitError',message='执行时间上限已到')
execsrc('permission_next_session',call('exit',',"0"'),allow=False,code=2,error='ValueError',message='子进程能力未启用')
execsrc('absolute_only','exec(["cmd.exe"],1000,0)',code=2,error='ValueError',message='子进程程序必须是绝对可执行文件路径')
run('argv_protected',['-e','assert(argv()==["--allow-process","--allow-process=1"])','--','--allow-process','--allow-process=1'],stdout='')
for text in ['','1','bad']:run('no_value_'+text,['--allow-process='+text,'-e','1'],code=2,message="选项 '--allow-process' 不接受值")
run('duplicate',['--allow-process','--allow-process','-e','1'],code=2,message="选项 '--allow-process' 不能重复")
run('check_no_permission',['check','--allow-process',str(P)],code=2,message='子进程能力选项只适用于运行命令')
run('mixed_first_permission',['check','--allow-process','--max-steps','1',str(P)],code=2,message='子进程能力选项只适用于运行命令')
run('mixed_first_limits',['check','--max-steps','1','--allow-process',str(P)],code=2,message='执行限额选项只适用于运行命令')
assert before=={str(p.relative_to(R)):sha(p)for p in guides} and sha(X)==a.expected_sha256 and sha(F)==FH and sha(original)==FH
result={'date':'2026-10-30','timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'binary_sha256':a.expected_sha256,'binary_bytes':X.stat().st_size,'fixture':str(F.relative_to(R)),'fixture_sha256':FH,'guides':before,'passed':len(rows),'total':len(rows),'rows':rows};O.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n');print(json.dumps({'passed':len(rows),'total':len(rows)}))
