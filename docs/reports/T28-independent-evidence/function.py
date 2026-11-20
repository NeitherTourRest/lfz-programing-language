from pathlib import Path
import argparse,hashlib,json,subprocess
root=Path.cwd(); p=argparse.ArgumentParser();p.add_argument('--baseline',type=Path,required=True);p.add_argument('--baseline-sha',required=True);p.add_argument('--candidate',type=Path,required=True);p.add_argument('--candidate-sha',required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();digest=lambda f:hashlib.sha256(f.read_bytes()).hexdigest();cases=[]
def add(n,code,flags=()):cases.append((n,code,flags))
add('alias-and-original-preserved','''var xs=[1]
let alias=xs
xs=push(2,xs)
assert(alias==[1])
assert(xs==[1,2])
xs[0]=9
assert(alias==[1])
print(jsonStringify([alias,xs]))
''')
add('returned-snapshot','''var xs=[]
let values=map((i)=>{xs=push(i,xs)
 return xs},range(4))
assert(values==[[0],[0,1],[0,1,2],[0,1,2,3]])
print(jsonStringify(values))
''')
add('nested-shared-reference','''let inner={n:1}
let old=[inner]
let next=push(inner,old)
next[0].n=7
assert(old[0].n==7)
assert(len(old)==1)
print(jsonStringify([old,next]))
''')
add('self-reference-retains-old','''var xs=[1]
let prior=xs
xs=push(xs,xs)
assert(xs==[1,[1]])
xs[1][0]=8
assert(prior==[8])
assert(xs[0]==1)
print(jsonStringify([prior,xs]))
''')
add('closure-retains-argument','''fn keep(array) {()=>array}
var xs=[1]
let saved=keep(xs)
xs=push(2,xs)
assert(saved()==[1])
print(jsonStringify([saved(),xs]))
''')
add('pipeline-and-captured-assignment','''fn make(){var xs=[]
 fn add(x){xs=xs |> push(x)
 return xs}
 return add}
let add=make()
let first=add(1)
let second=add(2)
assert(first==[1])
assert(second==[1,2])
print(jsonStringify([first,second]))
''')
add('shadowed-and-immutable-error','''fn local(){let push=(v,xs)=>[v+10]
 var xs=[]
 xs=push(1,xs)
 return xs}
assert(local()==[11])
let immutable=[1]
let bad=attempt(()=>{immutable=push(2,immutable)})
assert(!bad.ok)
assert(immutable==[1])
print(bad.error["class"])
''')
add('argument-order-visible-alias','''var xs=[1]
let cell={alias:nil}
fn next(){cell.alias=xs
 xs[0]=9
 return 2}
xs=push(next(),xs)
assert(cell.alias==[9])
assert(xs==[9,2])
print(jsonStringify([cell.alias,xs]))
''')
add('type-error-trace','''var xs=[1]
fn bad(){xs=push(2,0)}
let e=attempt(bad)
assert(!e.ok)
assert(xs==[1])
print(jsonStringify(e.error))
''')
add('budget-propagation','''var xs=[]
attempt(()=>{while true {
 xs=push(1,xs)
}})
''',('--json','--max-steps','100'))
rows=[];output=a.output.resolve();assert not output.exists();output.parent.mkdir(parents=True,exist_ok=True)
for name,code,flags in cases:
 result=[]
 for x,sha in [(a.baseline,a.baseline_sha),(a.candidate,a.candidate_sha)]:
  assert digest(x)==sha
  z=subprocess.run([str(x.resolve()),*flags,'-e',code],cwd=root,capture_output=True,timeout=10);result.append((z.returncode,z.stdout,z.stderr));rows.append(dict(name=name,binary_sha256=sha,source=code,flags=list(flags),exit=z.returncode,stdout=z.stdout.decode('utf8'),stderr=z.stderr.decode('utf8')))
  output.write_text(json.dumps(dict(rows=rows),ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 assert result[0]==result[1],name
 assert result[0][0]==(2 if name=='budget-propagation' else 0),(name,result)
 if name=='budget-propagation':assert json.loads(result[0][1])['error']=='ExecutionLimitError'
output.write_text(json.dumps(dict(passed=len(cases),rows=rows),ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(len(cases))
