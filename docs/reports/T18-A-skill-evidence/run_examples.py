import argparse,datetime,hashlib,json,os,re,shutil,subprocess
from pathlib import Path
R=Path.cwd();H='4093714b2dc61bb72dbc4f43db67a798a9bcaf152ad33a11d4d7457d1f7d9c86';F='51b83f054f128e381fa034adbe980d6398f24f76621703ba27da4d7e54d27bee';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
a=argparse.ArgumentParser();a.add_argument('--binary',type=Path,required=True);a.add_argument('--output',type=Path,required=True);o=a.parse_args();X=o.binary.resolve();O=o.output.resolve();assert X.is_relative_to(R) and O.is_relative_to(R) and not O.exists() and sha(X)==H
W=R/'Temp/T18-A-ai';W.mkdir(parents=True,exist_ok=True);original=R/'Temp/T18-A-tooling/fixture.exe';assert sha(original)==F;fx=W/'fixture.exe';shutil.copyfile(original,fx);assert sha(fx)==F
names=['.opencode/skills/lfz-programming/'+n for n in ['SKILL.md','README.md','VERIFICATION.md','prompt-template.md']]+['docs/guide/ai/README.md'];docs={n:sha(R/n)for n in names};env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1');rows=[]
def save():O.parent.mkdir(parents=True,exist_ok=True);O.write_text(json.dumps(dict(date='2026-10-30',timestamp_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),binary=str(X.relative_to(R)),binary_sha256=H,fixture_sha256=F,fixture_copy=str(fx.relative_to(R)),document_hashes=docs,checks_total=len(rows),checks_passed=sum(r['passed']for r in rows),rows=rows),ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
def run(n,args,expected='',error=None,message=None,usage=None):
 assert sha(X)==H and sha(fx)==F
 p=subprocess.run([str(X),*args],cwd=W,env=env,capture_output=True,timeout=15);s=p.stdout.decode('utf8');e=p.stderr.decode('utf8');ok=p.returncode==(2 if error or usage else 0)
 if usage:ok=ok and not s and usage in e
 elif error:
  try:v=json.loads(s);ok=ok and v['error']==error and (message is None or v['message']==message) and not e
  except Exception:ok=False
 else:ok=ok and s==expected and not e
 rows.append(dict(name=n,args=args,exit=p.returncode,stdout=s,stderr=e,passed=ok));save()
def call(mode,args=None,ms=5000,limit=1024):return 'exec('+json.dumps([fx.as_posix(),mode,*(args or [])],ensure_ascii=False)+','+str(ms)+','+str(limit)+')'
for i,n in enumerate([names[0],names[-1]]):
 b=re.search(r'```lfz\n(.*?)\n```',(R/n).read_text(encoding='utf8').split('## 6.6 T18-A：')[1],re.S)[1];p=W/('skill.lfz'if i==0 else'ai.lfz');p.write_text(b+'\n',encoding='utf8',newline='\n');run('extracted_check_'+str(i),['check','--json',str(p)],'{"ok":true,"mode":"check","executed":false}\n');expected=json.dumps({'exitCode':0,'signal':None,'stderr':'','stdout':'3:a b\n6:中文\n'},ensure_ascii=False,separators=(',',':'))+'\n';run('extracted_run_'+str(i),['--allow-process','--max-steps','1000','--timeout-ms','10000','run',str(p),'--',str(fx)],expected)
c=call('echo',['','a b','中文🙂']);run('default_disabled',['--json','-e',c],error='ValueError',message='子进程能力未启用');run('enabled_argv',['--allow-process','-e','assert('+c+'.stdout == '+json.dumps('0:\n3:a b\n10:中文🙂\n',ensure_ascii=False)+')'])
run('next_context_disabled',['--json','-e',c],error='ValueError',message='子进程能力未启用')
run('nonzero_success',['--allow-process','-e','let r='+call('exit',['7'],limit=0)+'\nassert(r.exitCode==7 && r.signal==nil && r.stdout=="" && r.stderr=="")'])
run('stdin_eof',['--allow-process','-e','assert('+call('stdin',limit=1)+'.stdout=="0")'])
run('strict_preserved_bom_crlf',['--allow-process','-e','assert('+call('utf8',limit=6)+'.stdout=='+json.dumps('\ufeffa\r\n',ensure_ascii=False)+')'])
run('combined_exact',['--allow-process','-e','let r='+call('dual',['32'],limit=64)+'\nassert(len(r.stdout)==32 && len(r.stderr)==32)'])
for n,c,m in [('combined_over',call('dual',['32'],limit=63),'子进程输出超过字节上限'),('invalid_utf8',call('invalid',limit=1),'子进程输出不是合法 UTF-8'),('local_timeout',call('sleep',['200'],ms=10,limit=0),'子进程执行时间上限已到')]:run(n,['--allow-process','-e','let r=attempt(()=>'+c+')\nassert(!r.ok && r.error["class"]=="ValueError")\nassert(r.error.message=='+json.dumps(m,ensure_ascii=False)+')'])
run('global_deadline_not_caught',['--allow-process','--timeout-ms','10','--json','-e','attempt(()=>'+call('sleep',['200'],ms=1000,limit=0)+')'],error='ExecutionLimitError',message='执行时间上限已到')
run('captured_output_single_status',['--allow-process','--json','-e',call('echo',['captured'])],'{"ok":true}\n')
for n,c,m in [('relative','exec(["tool.exe"],1,0)','子进程程序必须是绝对可执行文件路径'),('batch','exec(["C:/tool.cmd"],1,0)','子进程程序必须是绝对可执行文件路径'),('empty','exec([],1,0)','子进程 argv 不能为空')]:run(n,['--json','-e',c],error='ValueError',message=m)
run('capability_equals',['--allow-process=1','--help'],usage="选项 '--allow-process' 不接受值")
run('duplicate',['--allow-process','--allow-process','-e',''],usage="选项 '--allow-process' 不能重复")
run('mixed_first_capability',['check','--allow-process','--max-steps','1','-e','1'],usage='子进程能力选项只适用于运行命令')
run('mixed_first_limit',['check','--max-steps','1','--allow-process','-e','1'],usage='执行限额选项只适用于运行命令')
run('scanner_error_precedes_applicability',['check','--allow-process','--timeout-ms=1','-e','1'],usage="选项 '--timeout-ms' 不支持等号写法，请使用 '--timeout-ms N'")
run('code_protected',['check','-e','--allow-process','--json'],'{"ok":true,"mode":"check","executed":false}\n')
run('argv_protected',['-e','print(jsonStringify(argv()))','--','--allow-process','--allow-process=1'],'["--allow-process","--allow-process=1"]\n')
assert docs=={n:sha(R/n)for n in names};save();print(json.dumps({'checks_total':len(rows),'checks_passed':sum(r['passed']for r in rows)}));raise SystemExit(0 if all(r['passed']for r in rows) else 1)