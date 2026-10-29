"""Focused T14 human-guide examples; no compiler or historical report mutation."""
import argparse, datetime, hashlib, json, os, re, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
ap=argparse.ArgumentParser()
ap.add_argument('--binary',type=Path,required=True)
ap.add_argument('--expected-sha256',required=True)
ap.add_argument('--output',type=Path,required=True)
ap.add_argument('--prepare-only',action='store_true')
a=ap.parse_args()
exe=a.binary.resolve(); out=a.output.resolve()
assert exe.is_relative_to(ROOT) and out.is_relative_to(ROOT)
work=ROOT/'Temp/T14-docs'; work.mkdir(parents=True,exist_ok=True)
guide_names=['README.md','reference.md','tutorial.md','testing.md','errors.md']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
start_guides={n:sha(ROOT/'docs/guide'/n) for n in guide_names}
text=(ROOT/'docs/guide/tutorial.md').read_text(encoding='utf8')
m=re.search(r'<!-- T14-check-example -->\s*```lfz\s*\n(.*?)\n```',text,re.S); assert m
job=work/'job.lfz';job.write_text(m.group(1)+'\n',encoding='utf8',newline='\n')
hello=ROOT/'examples/hello.lfz'
# Prepare source files without launching an interpreter.
sentinel=work/'sentinel.txt'; sentinel.write_bytes('原字节\x00keep\n'.encode('utf8'))
new=work/'must-not-create.txt'
assert not new.exists(), 'non-execution witness target already exists; choose a clean scratch path'
quoted=lambda p:json.dumps(p.relative_to(ROOT).as_posix(),ensure_ascii=False)
side='print("NO")\neprint("NO")\n;;\ninput("NO")\nwriteFile('+quoted(new)+', "NO")\nappendFile('+quoted(sentinel)+', "NO")\n'
# File-IO signatures are data-last (path, content), as documented in the human guide.
sources={'job':sha(job),'hello':sha(hello)}
if a.prepare_only:
 print(json.dumps({'status':'prepared; interpreter launches=0','guide_hashes':start_guides,'source_hashes':sources},ensure_ascii=False))
 raise SystemExit(0)
assert not out.exists(), 'output must be a new project-local path'
expected=a.expected_sha256.lower();assert len(expected)==64 and sha(exe)==expected
before=sentinel.read_bytes();rows=[]
env=os.environ.copy();env.update(TEMP=str(work),TMP=str(work),PYTHONDONTWRITEBYTECODE='1')
SUCCESS={'ok':True,'mode':'check','executed':False}
def run(name,args,code=0,stdin=b'',stdout=None,json_success=False,error=None,usage=False,stderr=None):
 assert sha(exe)==expected,'binary changed'
 p=subprocess.run([str(exe),*args],cwd=ROOT,env=env,input=stdin,capture_output=True,timeout=10)
 actual=p.stdout.decode('utf8').replace('\r\n','\n');err=p.stderr.decode('utf8').replace('\r\n','\n')
 checks={'exit':p.returncode==code}
 if stdout is not None:checks['stdout']=actual==stdout
 if stderr is not None:checks['stderr']=err==stderr
 if json_success:
  checks['success_exact_bytes']=actual=='{"ok":true,"mode":"check","executed":false}\n'
  checks['success_keys']=json.loads(actual)==SUCCESS
  checks['stderr_empty']=err==''
 if error:
  data=json.loads(actual);checks.update(error=data.get('error')==error,mode=data.get('mode')=='check',executed=data.get('executed') is False,ok=data.get('ok') is False,stderr_empty=err=='',field_order=list(data)==['ok','mode','executed','error','message','file','line','col','traceback'])
 if usage:checks.update(stdout_empty=actual=='',stderr_usage=bool(err) and 'lfz:' in err)
 checks['sentinel_preserved']=sentinel.read_bytes()==before
 checks['new_target_absent']=not new.exists()
 rows.append({'name':name,'args':args,'stdin_hex':stdin.hex(),'exit':p.returncode,'stdout':actual,'stderr':err,'checks':checks,'passed':all(checks.values())})
 assert rows[-1]['passed'],rows[-1]
 return p
run('README_file_plain',['check','examples/hello.lfz'],stdout='',stderr='')
run('README_eval_no_division',['check','-e','print(1 / 0)'],stdout='',stderr='')
run('README_stdin_unicode',['check','-'],stdin='print("中文")\n'.encode('utf8'),stdout='',stderr='')
run('README_file_json',['check','--json','examples/hello.lfz'],json_success=True)
run('README_check_then_run_check',['check','examples/hello.lfz'],stdout='',stderr='')
run('README_check_then_run_execute',['run','examples/hello.lfz'],stdout='Hello, LFZ!\n',stderr='')
run('tutorial_job_check',['check','--json',str(job.relative_to(ROOT))],json_success=True)
run('tutorial_job_run',['run',str(job.relative_to(ROOT)),'--','输入'],stdout='{"args":["输入"],"message":"中文"}\n',stderr='')
run('tutorial_runtime_division',['-e','print(1 / 0)'],code=2,stdout='')
run('errors_return_json',['check','--json','-e','return 1'],code=2,error='SyntaxError')
run('reference_empty_CODE',['check','-e',''],stdout='',stderr='')
run('reference_CODE_literal_json',['check','-e','--json','--json'],json_success=True)
run('reference_options_before_after',['--json','check','--json','-e','print("--")','--json'],json_success=True)
for name,args in [('missing_source',['check','--json']),('check_help',['check','--json','--help']),('separator_tail',['check','examples/hello.lfz','--']),('separator_args',['check','examples/hello.lfz','--','one']),('multiple_sources',['check','examples/hello.lfz','examples/hello.lfz']),('missing_CODE',['check','-e'])]:
 run('usage_'+name,args,code=2,usage=True)
for name,src in [('unknown_name','unknownName'),('wrong_type','1 + true'),('wrong_arity','len()'),('missing_runtime_file','readFile("Temp/T14-docs/no-such-input.txt")'),('infinite_loop','while true {}'),('output_input_file_IO',side)]:
 run('nonstatic_'+name,['check','--json','-e',src],json_success=True)
run('syntax_after_side_effects',['check','--json','-e',side+'let =\n'],code=2,error='SyntaxError')
# One unchanged human-guide LFZ example, extracted and executed rather than copied by hand.
readme=(ROOT/'docs/guide/README.md').read_text(encoding='utf8')
old=re.search(r'```lfz\s*\n(.*?)\n```',readme,re.S);assert old
oldpath=work/'unchanged-hello.lfz';oldpath.write_text(old.group(1)+'\n',encoding='utf8',newline='\n')
run('unchanged_README_first_LFZ_block',['run',str(oldpath.relative_to(ROOT))],stdout='Hello, LFZ!\n',stderr='')
assert sha(exe)==expected
assert start_guides=={n:sha(ROOT/'docs/guide'/n) for n in guide_names}
result={'date':'2026-10-29','timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'binary':str(exe.relative_to(ROOT)),'binary_sha256':expected,'guide_hashes':start_guides,'source_hashes':sources,'scope':'new/affected T14 guide examples and one unchanged hello block; not historical 35-block full rerun','checks_total':len(rows),'checks_passed':sum(r['passed'] for r in rows),'rows':rows,'witnesses':{'existing_path':str(sentinel.relative_to(ROOT)),'before_hex':before.hex(),'after_hex':sentinel.read_bytes().hex(),'new_path':str(new.relative_to(ROOT)),'new_exists':new.exists()}}
out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
print(json.dumps({'checks_total':len(rows),'checks_passed':result['checks_passed'],'binary_sha256':expected,'output':str(out.relative_to(ROOT))},ensure_ascii=False))