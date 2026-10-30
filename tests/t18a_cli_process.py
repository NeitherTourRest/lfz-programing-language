"""T18-A public process tests. No builds, shell oracle, network or product imports."""
import argparse,datetime,hashlib,json,os,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser()
 for name in ('binary','fixture','output'):ap.add_argument('--'+name,type=Path,required=True)
 ap.add_argument('--expected-sha',required=True);ap.add_argument('--fixture-sha')
 ap.add_argument('--prepare-only',action='store_true');ap.add_argument('--red-only',action='store_true');a=ap.parse_args()
 exe,fixture,out=a.binary.resolve(),a.fixture.resolve(),a.output.resolve()
 assert all(p.is_relative_to(ROOT) for p in (exe,fixture,out)) and not out.exists()
 w=ROOT/'Temp/T18-A-public';w.mkdir(parents=True,exist_ok=True);out.parent.mkdir(parents=True,exist_ok=True)
 q=lambda s:json.dumps(str(s),ensure_ascii=False)
 f=q(fixture.as_posix());cases=[]
 def add(name,code,args=(),cls=None,msg=None,usage=None,allow=True,watch=False,where=None):
  cases.append(dict(name=name,src=code,args=list(args),cls=cls,msg=msg,usage=usage,allow=allow,watch=watch,where=where))
 def call(mode,extra='',timeout=1000,limit=10000):return 'exec(['+f+','+q(mode)+extra+'],'+str(timeout)+','+str(limit)+')'
 add('feature_red','let r='+call('exit',',"0"')+'\nassert(r.exitCode==0)')
 marker=w/'marker.txt'
 add('default_disabled','exec(['+f+',"mark",'+q(marker.as_posix())+'],1000,0)',cls='ValueError',msg='子进程能力未启用',allow=False,watch=True)
 for name,src,cls,msg in [
  ('arity','exec([])','TypeError',None),('argv_type','exec(1,100,0)','TypeError',None),
  ('element_type','exec([1],100,0)','TypeError',None),('empty','exec([],100,0)','ValueError','子进程 argv 不能为空'),
  ('relative','exec(["relative.exe"],100,0)','ValueError','子进程程序必须是绝对可执行文件路径'),
  ('noPATH','exec(["cmd.exe"],100,0)','ValueError','子进程程序必须是绝对可执行文件路径'),
  ('bat','exec(["C:/test.bat"],100,0)','ValueError','子进程程序必须是绝对可执行文件路径'),
  ('nul','exec(['+f+',"a\x00b"],100,0)','ValueError','子进程参数不能包含 NUL'),
  ('timeout_type','exec(['+f+'],"1",0)','TypeError',None),('limit_type','exec(['+f+'],1,"0")','TypeError',None)]:
  add(name,src,cls=cls,msg=msg)
 for n in (0,-1,600001):add('timeout_range'+str(n),'exec(['+f+'],'+str(n)+',0)',cls='ValueError',msg='子进程 timeoutMs 必须在 1..600000 范围内')
 for n in (-1,16777217):add('output_range'+str(n),'exec(['+f+'],1000,'+str(n)+')',cls='ValueError',msg='子进程 maxOutputBytes 必须在 0..16777216 范围内')
 add('exit_nonzero','let r='+call('exit',',"7"')+'\nassert(r.exitCode==7)\nassert(r.signal==nil)\nassert(r.stdout=="")\nassert(r.stderr=="")')
 add('stdin_EOF','let r='+call('stdin')+'\nassert(r.stdout=="0")')
 add('argv_unicode_empty','let r='+call('echo',',"","a b","雪"')+'\nassert(r.stdout=="0:\\n3:a b\\n3:雪\\n")')
 add('UTF8_exact','let r='+call('utf8')+'\nassert(r.stdout=="\ufeffa\\r\\n")')
 add('invalid_UTF8',call('invalid'),cls='ValueError',msg='子进程输出不是合法 UTF-8')
 add('zero_output',call('exit',',"0"',limit=0))
 for n in (0,5,6):
  add('combined_'+str(n),call('dual',',"3"',limit=n),cls='ValueError' if n<6 else None,msg='子进程输出超过字节上限' if n<6 else None)
 add('dual_large','let r='+call('dual',',"70000"',limit=140000)+'\nassert(len(r.stdout)==70000)\nassert(len(r.stderr)==70000)')
 add('tail_overflow',call('dual',',"70000"',limit=139999),cls='ValueError',msg='子进程输出超过字节上限')
 add('local_timeout_attempt','let r=attempt(()=>'+call('sleep',',"500"',timeout=10)+')\nassert(!r.ok)\nassert(r.error.message=="子进程执行时间上限已到")')
 add('global_deadline_attempt','attempt(()=>'+call('sleep',',"1000"',timeout=2000)+')',args=['--timeout-ms','50'],cls='ExecutionLimitError',msg='执行时间上限已到')
 add('context_after_enabled','exec(['+f+',"mark",'+q(marker.as_posix())+'],1000,0)',allow=False,cls='ValueError',msg='子进程能力未启用',watch=True)
 add('argv_protection','assert(argv()==["--allow-process","--allow-process=1"])',args=['--','--allow-process','--allow-process=1'])
 add('CODE_protection','--allow-process',allow=False,cls='NameError')
 add('duplicate','1',args=['--allow-process'],usage="选项 '--allow-process' 不能重复")
 for suffix in ('','1','bad'):
  add('equals_'+suffix,'1',args=['--allow-process='+suffix],allow=False,usage="选项 '--allow-process' 不接受值")
 for cmd in ('check','test','--help','-h','--version','-V'):
  add('inapplicable_'+cmd,'1',args=[cmd],usage='子进程能力选项只适用于运行命令')
 # Dependency runtime errors must render the real dependency file.
 lib=w/'lib.lfz';lib.write_text('#42\n'+call('sleep',',"500"',timeout=10)+'\n',encoding='utf8')
 mainfile=w/'main.lfz';mainfile.write_text('#42\nimport m from "./lib.lfz"\n',encoding='utf8')
 add('module_fault','',args=[str(mainfile)],cls='ValueError',msg='子进程执行时间上限已到',where=lib)
 # Parent emits descendant PID and leaves inherited pipes open; backend must close its Job.
 pidfile=w/'descendant.pid'
 add('descendant_pipe','let r='+call('parent',','+q(pidfile.as_posix()),timeout=3000)+'\nassert(r.exitCode==0)')
 selected=cases[:1] if a.red_only else cases
 if a.prepare_only:result={'schema':'LFZ-T18A-PREPARED-1','cases_total':len(cases),'executed':False,'cases':[c['name'] for c in cases]}
 else:
  digest=sha(exe);assert digest==a.expected_sha.lower()
  fixture_digest=sha(fixture)
  if a.fixture_sha:assert fixture_digest==a.fixture_sha.lower()
  env=os.environ.copy();env.update(TEMP=str(w),TMP=str(w),PYTHONDONTWRITEBYTECODE='1');rows=[]
  for c in selected:
   assert sha(exe)==digest and sha(fixture)==fixture_digest
   if marker.exists():marker.unlink()
   if pidfile.exists():pidfile.unlink()
   opts=c['args'];special=bool(opts and (opts[0] in ('check','test','--help','-h','--version','-V') or opts[0]==str(mainfile)))
   args=['--json']+(['--allow-process'] if c['allow'] else [])+(opts if special else ['-e',c['src'],*opts])
   if '\x00' in c['src']:
    nulsource=w/'nul-case.lfz';nulsource.write_text('#42\n'+c['src']+'\n',encoding='utf8')
    args=['--json']+(['--allow-process'] if c['allow'] else [])+[str(nulsource),*opts]
   row={'name':c['name'],'args':args};checks={}
   p=subprocess.Popen([str(exe),*args],stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=w,env=env)
   try:
    so,se=p.communicate(timeout=7);so=so.decode('utf8');se=se.decode('utf8');row.update(exit=p.returncode,stdout=so,stderr=se)
    checks['exit']=p.returncode==(2 if c['cls'] or c['usage'] else 0)
    if c['usage']:checks['usage']=not so and se.splitlines()[0]=='lfz: '+c['usage']
    else:
     obj=json.loads(so);checks['protocol']=list(obj)==(['ok','error','message','file','line','col','traceback'] if c['cls'] else ['ok']) and not se
     checks['ok']=obj.get('ok')==(not c['cls'])
     if c['cls']:checks['class']=obj.get('error')==c['cls']
     if c['msg']:checks['message']=obj.get('message')==c['msg']
     if c['where']:checks['source']=Path(obj.get('file','')).name==c['where'].name
    if c['watch']:checks['no_spawn']=not marker.exists()
   except (subprocess.TimeoutExpired,ValueError,UnicodeError) as err:
    row['exception']=repr(err);checks['complete']=False
    # Cleanup utility only, never used as the LFZ behavioral oracle.
    if isinstance(err,subprocess.TimeoutExpired):
     cleanup=subprocess.run(['taskkill','/PID',str(p.pid),'/T','/F'],capture_output=True,timeout=3)
     row['watchdog_cleanup']={'exit':cleanup.returncode,'stdout_hex':cleanup.stdout.hex(),'stderr_hex':cleanup.stderr.hex()}
     if p.poll() is None:p.kill()
     p.communicate(timeout=3)
   if pidfile.exists():
    pid=pidfile.read_text().strip();probe=subprocess.run(['tasklist','/FI','PID eq '+pid,'/FO','CSV','/NH'],capture_output=True,timeout=3)
    alive=('"'+pid+'"').encode() in probe.stdout;checks['descendant_reaped']=not alive
    row['descendant']={'pid':pid,'probe_stdout_hex':probe.stdout.hex()}
    if alive:subprocess.run(['taskkill','/PID',pid,'/T','/F'],capture_output=True,timeout=3)
   row.update(checks=checks,passed=all(checks.values()));rows.append(row)
  assert sha(exe)==digest and sha(fixture)==fixture_digest
  result={'schema':'LFZ-T18A-CLI-1','binary_sha256':digest,'fixture_sha256':fixture_digest,'script_sha256':sha(Path(__file__)),'checks_total':len(rows),'checks_passed':sum(r['passed'] for r in rows),'red_only':a.red_only,'rows':rows}
 result['timestamp_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 print(json.dumps({k:v for k,v in result.items() if k not in ('rows','cases')},ensure_ascii=False))
 return 0 if a.prepare_only or result['checks_total']==result['checks_passed'] else 1
if __name__=='__main__':sys.exit(main())
