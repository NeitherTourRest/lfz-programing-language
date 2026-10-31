import argparse, hashlib, json, math, os, random, struct, subprocess, zipfile
from pathlib import Path
R=Path.cwd(); W=R/'Temp/T12/verifier-rerun'; X=R/'target/release/lfz.exe'
ap=argparse.ArgumentParser();ap.add_argument('--output',type=Path,required=True);o=ap.parse_args()
out=o.output.resolve()
if not out.is_relative_to(R) or out.exists():raise SystemExit('output must be a new project-local path')
W.mkdir(parents=True,exist_ok=True);out.parent.mkdir(parents=True,exist_ok=True)
env=os.environ.copy(); env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1')
rows=[]
def run(name,args,stdin=None,expected=None,error=None,message=None):
 p=subprocess.run([str(X),*args],input=stdin,capture_output=True,cwd=R,env=env,timeout=15)
 row={'name':name,'args':args,'stdin_hex':stdin.hex() if stdin is not None else None,'exit':p.returncode,'stdout':p.stdout.decode('utf8'),'stderr':p.stderr.decode('utf8')}
 if error:
  obj=json.loads(p.stdout); assert p.returncode==2 and not p.stderr and obj['ok'] is False and obj['error']==error,row
  if message: assert obj['message']==message,(row,message)
 else:
  assert p.returncode==0 and not p.stderr,row
  if expected is not None: assert p.stdout.decode('utf8').replace('\r\n','\n')==expected,(row,expected)
 rows.append(row); return p

def equal(a,b):
 if type(a)!=type(b):return False
 if isinstance(a,float):return a.hex()==b.hex()
 if isinstance(a,list):return len(a)==len(b) and all(equal(x,y) for x,y in zip(a,b))
 if isinstance(a,dict):return a.keys()==b.keys() and all(equal(a[k],b[k]) for k in a)
 return a==b

def pairs(ps):
 keys=[k for k,v in ps]; assert keys==sorted(keys,key=lambda k:k.encode('utf8')),(keys,'key order')
 assert len(keys)==len(set(keys));return dict(ps)
rng=random.Random(42261025)
strs=['','${x}','中文🙂','a\0b','\b\f\n\r\t','/\\"','\u2028','𐀀','é','e\u0301','\x1f']
ints=[0,1,-1,-2**63,2**63-1,2**53+1,-2**53-1]
floats=[-0.0,0.0,1.5,1e-300,-1e100,5e-324,1.7976931348623157e308]
def gen(depth=0):
 kinds=6 if depth<4 else 4
 k=rng.randrange(kinds)
 if k==0:return rng.choice([None,True,False])
 if k==1:return rng.choice(ints)
 if k==2:return rng.choice(floats)
 if k==3:return rng.choice(strs)
 if k==4:return [gen(depth+1) for _ in range(rng.randrange(5))]
 keys=rng.sample(strs,rng.randrange(5));return {k:gen(depth+1) for k in keys}
values=ints+floats+strs+[gen() for _ in range(90)]
for i,v in enumerate(values):
 text=json.dumps(v,ensure_ascii=i%2==0,separators=(',',':'),allow_nan=False)
 ip=W/'independent-input.json';ip.write_text(text,encoding='utf8',newline='')
 p=run(f'diff_{i}',['-e','print(jsonStringify(jsonParse(readFile(argv()[0]))))','--',str(ip)])
 got=json.loads(p.stdout,object_pairs_hook=pairs)
 assert equal(got,v),(text,p.stdout,v)
 rows[-1]['input_json']=text;rows[-1]['independent_expected']=v
# Negative cases derive positions independently from UTF8 byte slices, not source helpers.
invalid=[('',0,'不是合法的 JSON'),(' [1,]',4,'不是合法的 JSON'),('{"🙂":1,"x":}',len('{"🙂":1,"x":'.encode()),'不是合法的 JSON'),('{"中":0,"\\u4e2d":1}',len('{"中":0,'.encode()),'对象键重复'),('9223372036854775808',0,'整数超出 i64 范围'),('-9223372036854775809',0,'整数超出 i64 范围'),('1e309',0,'浮点数超出有限范围'),('01',1,'不是合法的 JSON'),('"\\uD800"',7,'不是合法的 JSON'),('true false',5,'不是合法的 JSON'),('\ufeff[]',0,'不是合法的 JSON'),('['*129+'0'+']'*129,128,'嵌套深度超过 128 层')]
for i,(text,offset,reason) in enumerate(invalid):
 ip=W/'invalid.json';ip.write_text(text,encoding='utf8',newline='')
 run(f'invalid_{i}',['--json','-e','jsonParse(readFile(argv()[0]))','--',str(ip)],error='ValueError',message=f'JSON 解析失败（字节偏移 {offset}）：{reason}')
 rows[-1]['input_utf8_hex']=text.encode().hex()
# Six entry forms, exact argument fidelity and saved argv / local-shadow behavior.
body='let saved=argv\nlet a=saved()\na[0]="changed"\nfn f(argv) => saved()\nassert(f(42)==saved())\nlet argv=fn(){["local"]}\nassert(argv()==["local"])\nprint(jsonStringify(saved()))\n'
file=W/'entry.lfz';file.write_text('#42\n'+body,encoding='utf8',newline='')
args=['中文','','--json','--','-e','--help']; expect=json.dumps(args,ensure_ascii=False,separators=(',',':'))+'\n'
for name,cmd,stdin in [('bare',[str(file)],None),('run',['run',str(file)],None),('e',['-e',body],None),('rune',['run','-e',body],None),('stdin',['-'],body.encode()),('runstdin',['run','-'],body.encode())]:
 run('entry_'+name,cmd+['--',*args],stdin,expected=expect)
run('shared_acyclic',['-e','let x={"z":1}\nprint(jsonStringify([x,x]))'],expected='[{"z":1},{"z":1}]\n')
run('cycle',['--json','-e','let x={"a":nil}\nx.a=x\njsonStringify(x)'],error='ValueError',message='JSON 编码失败：循环引用')
run('index_alias_data',['-e','let x={"b":2,"a":1}\nlet alias=x\nx.z=3\nlet y=del("b",x)\ny.a=7\nassert(alias.a==1)\nassert(alias.z==3)\nassert(!has("b",y))\nprint(jsonStringify([x,y]))'],expected='[{"a":1,"b":2,"z":3},{"a":7,"z":3}]\n')
run('depth_128',['-e','let x=jsonParse(repeat(128,"[")+"0"+repeat(128,"]"))\nassert(len(jsonStringify(x))==257)\nprint("ok")'],expected='ok\n')
run('encode_depth_129',['--json','-e','var x=0\nfor i in range(129){x=[x]}\njsonStringify(x)'],error='ValueError',message='JSON 编码失败：嵌套深度超过 128 层')
run('method_filter',['-e','struct S { x:1, fn boom(){fail("must not call")} }\nprint(jsonStringify(S{}))'],expected='{"x":1}\n')
run('data_channel',['--json','-e','print(jsonStringify(argv()))','--','--json']) if False else None
p=subprocess.run([str(X),'--json','-e','print(jsonStringify(argv()))','--','--json'],capture_output=True,cwd=R,env=env,timeout=15)
assert p.returncode==0 and json.loads(p.stdout)=={'ok':True} and json.loads(p.stderr)==['--json']
rows.append({'name':'data_channel','exit':p.returncode,'stdout':p.stdout.decode(),'stderr':p.stderr.decode()})
# Freeze verification includes every original manifest entry restored inside the ZIP.
old=R/'docs/reports/agent-benchmark-v2/v2-20261024-r01/raw-inputs-submissions.zip'
assert hashlib.sha256(old.read_bytes()).hexdigest()=='90a6a3e3d894e7a813b06463506d5cc540e0a4570d7b005a3fd27ccc6a776d84'
with zipfile.ZipFile(old) as z:
 manifest=json.loads((R/'docs/validation/agent-benchmark-v2/manifest.json').read_text())
 matches={}
 for path,expected in manifest['files'].items():
  found=[n for n in z.namelist() if n.endswith(path) and hashlib.sha256(z.read(n)).hexdigest()==expected]
  assert found,(path,'old input absent'); matches[path]=found
 frozen={'zip_sha256':hashlib.sha256(old.read_bytes()).hexdigest(),'matched_original_inputs':matches}
# All old official manifest cases survive structurally, all old source test files byte-identical.
old_cases=json.loads(subprocess.check_output(['git','show','v1.3.0:tests/cases.json'],cwd=R))['cases']
new_cases=json.loads((R/'tests/cases.json').read_text())['cases']
assert new_cases[:len(old_cases)]==old_cases
changed_old=[]
for p in subprocess.check_output(['git','ls-tree','-r','--name-only','v1.3.0','tests'],cwd=R,text=True).splitlines():
 if p.endswith('.lfz'):
  assert (R/p).read_bytes()==subprocess.check_output(['git','show','v1.3.0:'+p],cwd=R),p
for path in ['docs/validation/agent-benchmark-v2','docs/reports/agent-benchmark-v2/v2-20261024-r01']:
 assert not subprocess.check_output(['git','diff','v1.3.0','--',path],cwd=R),path
result={'date':'2026-10-25','binary_sha256':hashlib.sha256(X.read_bytes()).hexdigest(),'seed':42261025,'passed':len(rows),'cases':rows,'frozen_v2':frozen,'old_manifest_cases_preserved':len(old_cases),'old_lfz_files_byte_identical':True}
out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print(json.dumps({'passed':len(rows),'frozen_inputs':len(frozen['matched_original_inputs']),'old_cases':len(old_cases)}))