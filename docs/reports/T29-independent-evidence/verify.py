from pathlib import Path
import argparse,hashlib,json,subprocess,os
root=Path.cwd();base=root/'Temp/T29-verifier';fixtures=base/'fixtures';(fixtures/'lib').mkdir(parents=True,exist_ok=True);p=argparse.ArgumentParser();p.add_argument('--baseline',type=Path,required=True);p.add_argument('--baseline-sha',required=True);p.add_argument('--candidate',type=Path,required=True);p.add_argument('--candidate-sha',required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--prepare-only',action='store_true');a=p.parse_args();h=lambda f:hashlib.sha256(f.read_bytes()).hexdigest();cases=[]
def add(name,text,expected=None,flags=()):
 f=fixtures/(name+'.lfz');f.write_text('#42\n'+text,encoding='utf8');cases.append(dict(name=name,path=f,expected=expected,flags=flags))
add('escape_parameters','''let callbacks=map((x)=>()=>x,[11,22,33])
assert(callbacks[0]()==11)
assert(callbacks[1]()==22)
assert(callbacks[2]()==33)
print(jsonStringify(map((f)=>f(),callbacks)))
''','[11,22,33]\n')
add('escape_mutable_and_mutated_parameter','''let callbacks=map((x)=>{var n=x
 return ()=>{n+=1
 x+=10
 return [n,x]}},[1,2,3])
assert(callbacks[0]()==[2,11])
assert(callbacks[1]()==[3,12])
assert(callbacks[0]()==[3,21])
assert(callbacks[2]()==[4,13])
print("mutable isolated")
''','mutable isolated\n')
add('parameter_cell_escape_before_later_write','''let out=[]
var saved=[]
let values=map((x)=>{saved=push(()=>x,saved)
 x+=100
 return x},[1,2,3])
assert(values==[101,102,103])
assert(map((f)=>f(),saved)==values)
print("same-call captured cells")
''','same-call captured cells\n')
add('nested_map_escaped_environments','''let matrix=map((x)=>map((y)=>()=>x*10+y,[1,2]),[1,2,3])
let a=matrix[0][0]
let b=matrix[1][1]
let c=matrix[2][0]
assert([a(),b(),c()]==[11,22,31])
print("nested")
''','nested\n')
add('error_restoration_escaped_callback','''let saved={callback:nil}
let result=attempt(()=>map((x)=>{saved.callback=()=>x
 if x==2 { fail("stop") }
 return x},[1,2,3]))
assert(!result.ok)
let last=saved.callback
assert(last()==2)
let later=map((x)=>x+1,[7,8])
assert(later==[8,9])
assert(last()==2)
print(jsonStringify(result.error))
''')
add('shortcircuit_and_shared_objects','''var seen=[]
let objs=[{x:1},{x:2},{x:3}]
let index=findIndex((v)=>{seen=push(v.x,seen)
 v.x+=10
 return v.x==12},objs)
assert(index==1)
assert(seen==[1,2])
assert(map((o)=>o.x,objs)==[11,12,3])
print(jsonStringify(objs))
''','[{"x":11},{"x":12},{"x":3}]\n')
add('sortkey_escaped','''var captures=[]
let sorted=sortBy((x)=>{captures=push(()=>x,captures)
 return -x},[3,1,2])
assert(sorted==[3,2,1])
assert(map((f)=>f(),captures)==[3,1,2])
print("sort captures")
''','sort captures\n')
add('resource_terminal','''attempt(()=>map((x)=>{while true {
 x+1
 }},[1]))
print("AFTER_LIMIT")
''',flags=('--json','--max-steps','100'))
add('runtime_stack','''fn failer(x){x/0}
map(failer,[1])
''',flags=('--json',))
add('module_home','''import lib from "./lib/private.lfz"
let offset=900
let callbacks=lib.make()
assert(map((f)=>f(),callbacks)==[41,42,43])
lib.adjust(10)
assert(map((f)=>f(),callbacks)==[51,52,53])
print("home")
''','home\n')
(fixtures/'lib/private.lfz').write_text('''#42
var offset=40
fn make(){map((x)=>()=>offset+x,[1,2,3])}
fn adjust(x){offset+=x}
export make,adjust
''',encoding='utf8')
assert not a.output.exists();a.output.parent.mkdir(parents=True,exist_ok=True)
if a.prepare_only:
 a.output.write_text(json.dumps(dict(prepared=len(cases),executed=False),indent=2),encoding='utf8');print('prepared',len(cases));raise SystemExit
rows=[];passed=[];env=dict(os.environ,TEMP=str(base),TMP=str(base))
for v in cases:
 results=[]
 for binary,digest in [(a.baseline,a.baseline_sha),(a.candidate,a.candidate_sha)]:
  assert h(binary)==digest
  z=subprocess.run([str(binary.resolve()),'run',*v['flags'],str(v['path'])],cwd=root,capture_output=True,env=env,timeout=10)
  results.append((z.returncode,z.stdout,z.stderr));rows.append(dict(name=v['name'],binary_sha256=digest,source_sha256=h(v['path']),exit=z.returncode,stdout=z.stdout.decode('utf8'),stderr=z.stderr.decode('utf8')));a.output.write_text(json.dumps(dict(passed=passed,rows=rows),ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 assert results[0]==results[1],v['name']
 if v['flags']:
  assert results[0][0]==2
  err=json.loads(results[0][1]);assert err['error']==('ExecutionLimitError'if v['name']=='resource_terminal'else 'ZeroDivisionError')
 else:
  assert results[0][0]==0
  if v['expected']is not None:assert results[0][1]==v['expected'].encode() and not results[0][2]
 passed.append(v['name'])
a.output.write_text(json.dumps(dict(passed=passed,rows=rows),ensure_ascii=False,indent=2)+'\n',encoding='utf8');print('PASS',len(passed))

