import argparse,datetime,hashlib,json,os,re,subprocess
from pathlib import Path
R=Path.cwd();H='1946ab9ca5162d68f83a9d098424bd0bafdf6feadc6ea90527e0db9e1db12c45';a=argparse.ArgumentParser();a.add_argument('--binary',type=Path,required=True);a.add_argument('--output',type=Path,required=True);o=a.parse_args();X=o.binary.resolve();O=o.output.resolve();assert X.is_relative_to(R) and O.is_relative_to(R) and not O.exists();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert sha(X)==H
W=R/'Temp/T17-ai';W.mkdir(parents=True,exist_ok=True);env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1');names=['.opencode/skills/lfz-programming/'+n for n in ['SKILL.md','README.md','VERIFICATION.md','prompt-template.md']]+['docs/guide/ai/README.md'];docs={n:sha(R/n) for n in names};rows=[]
def save():O.parent.mkdir(parents=True,exist_ok=True);O.write_text(json.dumps(dict(date='2026-10-30',timestamp_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),binary=str(X.relative_to(R)),binary_sha256=H,document_hashes=docs,checks_total=len(rows),checks_passed=sum(r['passed'] for r in rows),rows=rows),ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
def run(n,args,expected='',error=None,message=None,usage=None,stdin=None,faultfile=None):
 assert sha(X)==H;p=subprocess.run([str(X),*args],cwd=W,env=env,input=stdin,capture_output=True,timeout=15);s=p.stdout.decode('utf8');e=p.stderr.decode('utf8');ok=p.returncode==(2 if error or usage else 0)
 if usage:ok=ok and not s and usage in e
 elif error:
  try:v=json.loads(s);ok=ok and v['error']==error and not e and (message is None or v['message']==message) and (faultfile is None or v['file'].replace('\\','/').endswith('/'+faultfile))
  except Exception:ok=False
 else:ok=ok and s==expected and not e
 rows.append(dict(name=n,args=args,exit=p.returncode,stdout=s,stderr=e,passed=ok));save()
for i,n in enumerate([names[0],names[-1]]):
 s=(R/n).read_text(encoding='utf8').split('## 6.5 T17：')[1];code=re.search(r'```lfz\n(.*?)\n```',s,re.S)[1];p=W/('skill.lfz' if i==0 else 'ai.lfz');p.write_text(code+'\n',encoding='utf8',newline='\n');run('extracted_check_'+str(i),['check','--json',str(p)],'{"ok":true,"mode":"check","executed":false}\n');run('extracted_limited_run_'+str(i),['--max-steps','200','--timeout-ms','1000','run',str(p)],'6\n')
for n,b,code in [('expr_two',2,'1'),('four_steps',4,'let x=1\nx'),('leading_zeros',2,'1')]:run(n,['--max-steps',str(b).zfill(3) if n=='leading_zeros' else str(b),'-e',code])
for n,b,code in [('expr_zero',0,'1'),('expr_one',1,'1'),('four_short',3,'let x=1\nx'),('infinite',20,'while true {1}'),('attempt_no_catch',20,'attempt(()=>{while true {1}})'),('recovery_no_refund',50,'while true {attempt(()=>1/0)}')]:run(n,['--json','--max-steps',str(b),'-e',code],error='ExecutionLimitError',message='执行步数预算已耗尽')
run('deadline_zero',['--json','--timeout-ms','0','-e','1'],error='ExecutionLimitError',message='执行时间上限已到')
run('empty_zero_both',['--max-steps','0','--timeout-ms','0','-e',''])
run('frontend_before_limits',['--json','--max-steps','0','-e','let ='],error='SyntaxError')
run('stdin_exact',['--max-steps','2','-'],stdin=b'1')
run('code_equals_protected',['--json','-e','--max-steps=1'],error='SyntaxError')
run('code_double_minus_old_name',['--json','-e','--max-steps'],error='NameError')
run('argv_flags_protected',['--max-steps','200','-e','print(jsonStringify(argv()))','--','--max-steps=0','--timeout-ms',''], '["--max-steps=0","--timeout-ms",""]\n')
for flag in ['--max-steps','--timeout-ms']:
 msg=f"选项 '{flag}' 不支持等号写法，请使用 '{flag} N'"
 for suffix in ['','0','bad']:run(flag+'_equals_'+suffix,[flag+'='+suffix,'-e','1'],usage=msg)
 run(flag+'_help_equals',['--help',flag+'=1'],usage=msg)
 run(flag+'_version_equals',['--version',flag+'=1'],usage=msg)
 run(flag+'_invalid',[flag,'-1','-e','1'],usage='选项')
 run(flag+'_duplicate',[flag,'1',flag,'2','-e','1'],usage='重复')
run('check_rejects',['check','--max-steps','10','-e','1'],usage='执行限额选项只适用于运行命令')
run('test_rejects',['test','--timeout-ms','10'],usage='执行限额选项只适用于运行命令')
run('unlimited_compatibility',['-e','assert(sum([1,2,3])==6)\nassert(attempt(()=>7).value==7)'])
(W/'leaf.lfz').write_text('#42\nfn work(){while true {1}}\nexport work\n',encoding='utf8');p=W/'main.lfz';p.write_text('#42\nimport lib from "./leaf.lfz"\nlib.work()\n',encoding='utf8');run('module_fault_source',['run','--json','--max-steps','30',str(p)],error='ExecutionLimitError',message='执行步数预算已耗尽',faultfile='leaf.lfz')
assert docs=={n:sha(R/n) for n in names};save();print(json.dumps({'checks_total':len(rows),'checks_passed':sum(r['passed'] for r in rows)}));raise SystemExit(0 if all(r['passed']for r in rows) else 1)