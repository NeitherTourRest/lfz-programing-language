import argparse,datetime,hashlib,json,os,subprocess
from pathlib import Path
R=Path.cwd();ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path);ap.add_argument('--expected-sha256');ap.add_argument('--output',type=Path);ap.add_argument('--prepare-only',action='store_true');a=ap.parse_args()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
G=[R/'docs/guide'/n for n in ['README.md','reference.md','tutorial.md','testing.md','errors.md','execution-limits.md']];before={str(p.relative_to(R)):sha(p)for p in G}
W=R/'Temp/T17-docs';W.mkdir(parents=True,exist_ok=True)
if a.prepare_only:print('prepare-only: six guide files present; interpreter launches 0');raise SystemExit
assert a.binary and a.expected_sha256 and a.output
X=a.binary.resolve();O=a.output.resolve();assert X.is_relative_to(R)and O.is_relative_to(R)and not O.exists();assert sha(X)==a.expected_sha256
E=os.environ.copy();E.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1');rows=[]
def run(name,args,stdin=b'',code=0,stdout=None,message=None,error=None):
 assert sha(X)==a.expected_sha256
 p=subprocess.run([str(X),*args],input=stdin,capture_output=True,cwd=R,env=E,timeout=15)
 r={'name':name,'args':args,'input_hex':stdin.hex(),'exit':p.returncode,'stdout':p.stdout.decode('utf8'),'stderr':p.stderr.decode('utf8')};rows.append(r);O.parent.mkdir(parents=True,exist_ok=True);O.with_suffix('.partial.json').write_text(json.dumps({'sha256':a.expected_sha256,'rows':rows},ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 assert p.returncode==code,r
 if error:
  j=json.loads(p.stdout);assert j['error']==error and j['message']==message and not p.stderr,r
 elif message:assert not p.stdout and r['stderr'].splitlines()[0]=='lfz: '+message,r
 elif stdout is not None:assert r['stdout'].replace('\r\n','\n')==stdout and not p.stderr,r
 r['passed']=True;return r
for n in [0,1]:run('expression_budget_'+str(n),['--json','--max-steps',str(n),'-e','1'],code=2,error='ExecutionLimitError',message='执行步数预算已耗尽')
run('expression_two',['--max-steps','2','-e','1'],stdout='')
run('declaration_four',['--max-steps','4','-e','let x=1\nx'],stdout='')
run('declaration_three',['--json','--max-steps','3','-e','let x=1\nx'],code=2,error='ExecutionLimitError',message='执行步数预算已耗尽')
for flag in ['--max-steps','--timeout-ms']:run('empty_'+flag,[flag,'0','-e',''],stdout='')
run('deadline_zero',['--json','--timeout-ms','0','-e','1'],code=2,error='ExecutionLimitError',message='执行时间上限已到')
run('deadline_before_steps',['--json','--max-steps','0','--timeout-ms','0','-e','1'],code=2,error='ExecutionLimitError',message='执行时间上限已到')
run('unlimited',['-e','print(42)'],stdout='42\n')
run('bounded_infinite',['--json','--max-steps','50','-e','while true {}'],code=2,error='ExecutionLimitError',message='执行步数预算已耗尽')
run('attempt_cannot_swallow',['--json','--max-steps','50','-e','attempt(()=>{while true {}})'],code=2,error='ExecutionLimitError',message='执行步数预算已耗尽')
run('check_then_run_check',['check','--json','-e','print(42)'],stdout='{"ok":true,"mode":"check","executed":false}\n')
run('check_then_run_execute',['--max-steps','100','--timeout-ms','1000','-e','print(42)'],stdout='42\n')
run('argv_protected',['--max-steps','100','-e','print(jsonStringify(argv()))','--','--max-steps','9'],stdout='["--max-steps","9"]\n')
run('equals_argv_protected',['-e','print(jsonStringify(argv()))','--','--timeout-ms=0'],stdout='["--timeout-ms=0"]\n')
# Option-like CODE remains source: syntax-valid unknown identifiers are checked, not flags.
run('code_protected',['check','-e','--max-steps'],stdout='')
for flag in ['--max-steps','--timeout-ms']:
 for suffix in ['','1','bad']:
  run('equals_'+flag+'_'+suffix,['--help',flag+'='+suffix],code=2,message=f"选项 '{flag}' 不支持等号写法，请使用 '{flag} N'")
 run('missing_'+flag,[flag],code=2,message=f"选项 '{flag}' 需要一个非负十进制 u64 参数")
 run('invalid_'+flag,[flag,'-1','-e','1'],code=2,message=f"选项 '{flag}' 的值必须是非负十进制 u64")
 run('duplicate_'+flag,[flag,'1',flag,'2','-e','1'],code=2,message=f"选项 '{flag}' 不能重复")
 run('check_reject_'+flag,['check',flag,'1','-e','1'],code=2,message='执行限额选项只适用于运行命令')
# A module callback shares the entry session budget and retains its real source on failure.
(W/'worker.lfz').write_text('#42\nfn spin(){while true {}}\nexport spin\n',encoding='utf8')
(W/'main.lfz').write_text('#42\nimport worker from "./worker.lfz"\nattempt(()=>worker.spin())\n',encoding='utf8')
r=run('module_shared_budget',['--json','--max-steps','100',str(W/'main.lfz')],code=2,error='ExecutionLimitError',message='执行步数预算已耗尽');j=json.loads(r['stdout']);assert j['file'].endswith('worker.lfz') and j['traceback'][-1]['file'].endswith('worker.lfz'),r
assert before=={str(p.relative_to(R)):sha(p)for p in G} and sha(X)==a.expected_sha256
result={'date':'2026-10-30','timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'binary':str(X.relative_to(R)),'sha256':a.expected_sha256,'bytes':X.stat().st_size,'guide_hashes':before,'passed':len(rows),'total':len(rows),'rows':rows,'scope':'affected guide CLI examples; Rust API is normative documentation only; no host token/real-time guarantee tested'}
O.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n');print(json.dumps({'passed':len(rows),'total':len(rows)}))
