"""T15 public process witnesses. No interpreter source imports."""
import argparse,datetime,hashlib,json,os,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--expected-sha256',required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
exe=a.binary.resolve();out=a.output.resolve();assert exe.is_relative_to(R) and out.is_relative_to(R) and not out.exists()
sha=lambda q:hashlib.sha256(q.read_bytes()).hexdigest()
assert sha(exe)==a.expected_sha256.lower()
w=R/'Temp/T15-test';w.mkdir(parents=True,exist_ok=True)
env=os.environ.copy();env.update(TEMP=str(w),TMP=str(w))
rows=[]
def run(name,source,expected=None,error=None):
 assert sha(exe)==a.expected_sha256.lower()
 r=subprocess.run([str(exe),'-e',source],capture_output=True,cwd=R,env=env,timeout=10)
 row={'name':name,'source':source,'exit':r.returncode,'stdout':r.stdout.decode('utf8'),'stderr':r.stderr.decode('utf8')};rows.append(row)
 if error:
  assert r.returncode==2 and error in row['stderr'],row
 else:
  assert r.returncode==0,row
  if expected is not None:assert row['stdout'].replace('\r\n','\n')==expected,row
 return row
run('exact_result','print(jsonStringify(attempt(() => nil)))','{"error":null,"ok":true,"value":null}\n')
r=run('caught_named_trace','fn inner() => 1 / 0\nfn middle() => inner()\nlet r = attempt(() => middle())\nprint(jsonStringify(r))')
obj=json.loads(r['stdout']);assert obj['ok'] is False and obj['error']['class']=='ZeroDivisionError'
assert obj['error']['line']==1 and obj['error']['col']==15
assert obj['error']['message']=='除以零'
assert obj['error']['traceback']==[{'func':'<module>','line':3,'col':9},{'func':'<fn>','line':3,'col':23},{'func':'middle','line':2,'col':16},{'func':'inner','line':1,'col':15}]
assert [x['func'] for x in obj['error']['traceback']]==['<module>','<fn>','middle','inner']
assert all(list(sorted(x))==['col','func','line'] for x in obj['error']['traceback'])
r=run('late_uncaught_trace','fn oldbad() => 1 / 0\nlet r = attempt(() => oldbad())\nfn fresh() => missing_t15_trace\nfresh()',error='NameError')
assert 'in fresh' in r['stderr'] and 'in oldbad' not in r['stderr'],r
sentinel=w/'io-existing.txt';created=w/'io-created.txt'
sentinel.write_bytes(b'before\n')
if created.exists():created.unlink()
q=lambda v:json.dumps(str(v),ensure_ascii=False)
source='let r = attempt(() => { writeFile('+q(sentinel)+', "after")\nappendFile('+q(sentinel)+', "!")\nwriteFile('+q(created)+', "new")\nfail("io") })\nprint(jsonStringify(r.ok))'
run('io_not_rolled_back',source,'false\n');assert sentinel.read_bytes()==b'after!' and created.read_bytes()==b'new'
r=run('output_not_rolled_back','let r = attempt(() => { eprint("kept")\n fail("output") })\nprint(r.ok)','false\n');assert r['stderr']=='kept\n',r
r=run('check_false_is_success','let r = attempt(() => check(false))\nprint(jsonStringify(r))','{"error":null,"ok":true,"value":false}\n');assert r['stderr'],r
run('check_intrinsic_compatible','check(true)')
for code,cls in [('attempt()', 'TypeError'),('attempt(() => 1, () => 2)','TypeError'),('attempt(1)','TypeError'),('attempt((x)=>x)','TypeError'),('attempt(1/0)','ZeroDivisionError'),('attempt(() => range(9223372036854775807))','OverflowError')]:
 run('boundary_'+code,code,error=cls)
assert sha(exe)==a.expected_sha256.lower()
result={'date':datetime.datetime.now(datetime.timezone.utc).isoformat(),'binary':str(exe.relative_to(R)),'binary_sha256':sha(exe),'total':len(rows),'passed':len(rows),'rows':rows,'io_witness':{'existing_after_hex':sentinel.read_bytes().hex(),'created_after_hex':created.read_bytes().hex()}}
out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n');print(json.dumps({'total':len(rows),'passed':len(rows),'output':str(out.relative_to(R))}))
