from pathlib import Path
import re,subprocess,json,hashlib,os
root=Path.cwd();exe=root/'dist/lfz.exe';out=root/'Temp/docs-t12';out.mkdir(parents=True,exist_ok=True)
(root/'Temp/docs-v11').mkdir(parents=True,exist_ok=True)
env=os.environ.copy();env.update(TEMP=str(out),TMP=str(out),PYTHONIOENCODING='utf-8')
rows=[];old=json.loads((Path(__file__).resolve().parent/'prior-guide-runs.json').read_text(encoding='utf-8'))
expected={(r['document'],r['block']):r for r in old}
oldhash=hashlib.sha256(exe.read_bytes()).hexdigest()
for name in ['README.md','tutorial.md','reference.md','errors.md','testing.md']:
 text=(root/'docs/guide'/name).read_text(encoding='utf-8')
 for i,m in enumerate(re.finditer(r'```lfz\s*\n(.*?)\n```',text,re.S),1):
  body=m.group(1)+'\n';f=out/(name[:-3]+'-'+str(i)+'.lfz');f.write_text(body,encoding='utf-8',newline='\n')
  p=subprocess.run([str(exe),'run',str(f.relative_to(root))],cwd=root,env=env,capture_output=True,timeout=15)
  a=p.stdout.decode('utf-8');e=p.stderr.decode('utf-8');orig_i=i
  if name=='tutorial.md' and i>=20: orig_i=i-2
  prior=expected.get((name,orig_i)) if not(name=='tutorial.md' and i in [18,19]) else None
  code=prior['exit'] if prior else 0
  ok=p.returncode==code
  if prior:ok=ok and a.replace('\r\n','\n')==prior['stdout']
  if name=='tutorial.md' and i==18:ok=ok and a=='[{"id":9007199254740993,"title":"中文🙂"}]\n'
  if name=='tutorial.md' and i==19:ok=ok and a=='[]\n'
  rows.append(dict(document=name,block=i,file=str(f.relative_to(root)),source=body,command=[str(exe.relative_to(root)),'run',str(f.relative_to(root))],exit=p.returncode,expected_exit=code,stdout=a,stderr=e,passed=ok))
# Focused documented process forms and precise new diagnostics.
probes=[]
def run(name,args,input=b'',code=0,stdout=None,error=None,message=None,line=None,col=None,file=None):
 p=subprocess.run([str(exe),*args],cwd=root,env=env,input=input,capture_output=True,timeout=20)
 a=p.stdout.decode('utf-8');e=p.stderr.decode('utf-8');checks={'exit':p.returncode==code}
 if stdout is not None:checks['stdout']=a==stdout
 if error:
  obj=json.loads(a);checks.update(error=obj['error']==error,message=obj['message']==message,file=obj['file']==file,line=obj['line']==line,col=obj['col']==col,stderr=e=='')
 probes.append(dict(name=name,args=args,stdin_hex=input.hex(),exit=p.returncode,stdout=a,stderr=e,checks=checks,passed=all(checks.values())))
run('README_argv_literals',['-e','print(jsonStringify(argv()))','--','中文','--json',''],stdout='["中文","--json",""]\n')
run('README_stdin',['-'],b'print(1 + 2)\n',stdout='3\n')
run('tutorial_argv_with_args',['run','Temp/docs-t12/tutorial-19.lfz','--','one','--json'],stdout='["one","--json"]\n')
run('errors_duplicate_utf8_offset',['--json','-e','jsonParse(argv()[0])','--','{"中":1,"中":2}'],code=2,error='ValueError',message='JSON 解析失败（字节偏移 9）：对象键重复',file='<command>',line=1,col=1)
run('errors_nonfinite',['--json','-e','jsonStringify(exp(1000))'],code=2,error='ValueError',message='JSON 编码失败：浮点数必须有限',file='<command>',line=1,col=1)
run('errors_stdin_bad_utf8',['--json','-'],b'\xff',code=2,error='IOError',message='无法读取标准输入',file='<stdin>',line=None,col=None)
run('expression_no_auto_print',['-e','1 + 2'],stdout='')
run('state_json_not_business',['--json','-e','print(jsonStringify({answer:42}))'],stdout='{"ok":true}\n')
probes[-1]['checks']['stderr_business']=probes[-1]['stderr']=='{"answer":42}\n';probes[-1]['passed']=all(probes[-1]['checks'].values())
run('README_jsonflow',['run','app/jsonflow.lfz','--','app/fixtures/t12/config.json','app/fixtures/t12/input.json','Temp/guide-result.json'],stdout='{"input":3,"output":2,"path":"Temp/guide-result.json"}\n')
probes[-1]['checks']['output_bytes']=(root/'Temp/guide-result.json').read_bytes()==(root/'app/fixtures/t12/expected.json').read_bytes();probes[-1]['passed']=all(probes[-1]['checks'].values())
run('sortviz_parameter_command',['run','app/sortviz.lfz','--','--small','4','--big','30','--seed','7','--no-frames'])
assert hashlib.sha256(exe.read_bytes()).hexdigest()==oldhash
report={'date':'2026-10-25','binary':'dist/lfz.exe','binary_sha256':oldhash,'blocks':rows,'probes':probes,'blocks_total':len(rows),'blocks_passed':sum(x['passed'] for x in rows),'probes_total':len(probes),'probes_passed':sum(x['passed'] for x in probes)}
(out/'runs.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
for r in rows:
 print(r['document'],r['block'],r['exit'],r['passed'])
for r in probes:print(r['name'],r['exit'],r['passed'])
print('SUMMARY',report['blocks_passed'],report['blocks_total'],report['probes_passed'],report['probes_total'])
raise SystemExit(0 if report['blocks_passed']==len(rows) and report['probes_passed']==len(probes) else 1)
