"""Independent T13 semantic differential. No interpreter mutation or timing claims."""
import hashlib,json,os,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[3]
W=R/'Temp/T13-verifier';W.mkdir(parents=True,exist_ok=True)
OUT=R/'docs/reports/T13-independent-evidence/differential.json'
assert not OUT.exists()
OLD=R/'Temp/t13-perf/lfz-t12.exe';NEW=R/'dist/lfz.exe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(OLD)=='47ed2bfe2a2694b11bd611172cfcf0000d0bfc58e3b9bd809ca95f066dd0261f'
assert sha(NEW)=='9cda0c6fd5d85178335e8b758a843fc5f66574826bab4a80f2418a318e171cf9'
env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1')
cases=[]
def add(name,category,body,error=None):cases.append((name,category,'#42\n'+body+'\n',error))
add('direct_zero_multi_pipeline','argument_flow',r'''fn zero() => 42
fn combine(a,b,c) => a*100+b*10+c
assert(zero()==42)
assert(combine(1,2,3)==123)
assert((3 |> combine(1,2))==123)
var seen = ""
fn mark(x) { seen = seen + str(x)
 x }
assert(combine(mark(4),mark(5),mark(6))==456)
assert(seen=="456")''')
add('all_empty_skip_callbacks','argument_flow',r'''var count=0
fn bad(a,b,c) { count += 1
 0 }
assert(map(bad,[])==[])
assert(filter(bad,[])==[])
assert(reduce(bad,17,[])==17)
assert(sortBy(bad,[])==[])
each(bad,[])
assert(count==0)
assert(map(argv,[])==[])''')
add('reduce_three_alias','argument_flow',r'''let xs=[1,2,3,4]
assert(reduce((acc,x)=>acc*10+x,0,xs)==1234)
assert(xs==[1,2,3,4])
assert(sortBy((x)=>-x,xs)==[4,3,2,1])
assert(minBy((x)=>-x,xs)==4)
assert(maxBy((x)=>-x,xs)==1)''')
add('escaping_parameter_cells','fresh_cells',r'''fn make(n) { n+=10
 () => { n+=2
 n } }
let a=map(make,[1,2,3])
assert(a[2]()==15)
assert(a[0]()==13)
assert(a[2]()==17)
assert(a[1]()==14)
let b=make(1)
assert(b()==13)
assert(a[0]()==15)''')
add('parameter_rebind_vs_alias','rebind_alias',r'''let x=[1]
let o={n:2}
fn edit(a,b) { a[0]+=4
 b.n+=5
 a=[99]
 b={n:99}
 [a,b] }
let r=edit(x,o)
assert(x==[5])
assert(o.n==7)
assert(r[0]==[99])
assert(r[1].n==99)
let rows=[x,o]
let seen=map((v)=>{v},rows)
seen[0][0]=8
seen[1].n=9
assert(x[0]==8)
assert(o.n==9)''')
add('self_nested_recursion_shadow','self_recursion',r'''let o={n:6,work:(xs)=>map((x)=>self.n+x,xs)}
assert(o.work([1,2])==[7,8])
fn fact(n) { if n==0 {return 1}
 n*fact(n-1) }
assert(map(fact,[0,1,5])==[1,1,120])
fn shadow(argv) => map((x)=>argv+x,[1,2])
assert(shadow(9)==[10,11])
let saved=argv
fn invoke(f) => f()
assert(invoke(saved)==["one","中文",""])''')
for hof,expr,expected in [
 ('map','map((x)=>{source[1]=99\n x},source)','[3,1,2]'),
 ('filter','filter((x)=>{source[1]=99\n true},source)','[3,1,2]'),
 ('reduce','reduce((acc,x)=>{source[1]=99\n acc*10+x},0,source)','312'),
 ('sortBy','sortBy((x)=>{source[1]=99\n x},source)','[1,2,3]'),
 ('minBy','minBy((x)=>{source[1]=99\n x},source)','1'),
 ('maxBy','maxBy((x)=>{source[1]=99\n x},source)','3')]:
 add(hof+'_source_snapshot','hof_snapshot','let source=[3,1,2]\nlet result='+expr+'\nassert(result=='+expected+')\nassert(source[1]==99)')
add('each_source_snapshot','hof_snapshot',r'''let source=[3,1,2]
var order=0
each((x)=>{source[1]=99
 order=order*10+x},source)
assert(order==312)
assert(source[1]==99)''')
add('nested_snapshot_shared_values','hof_snapshot',r'''let a=[1]
let b=[2]
let rows=[a,b]
let out=map((x)=>{rows[1]=[9]
 x[0]+=10
 x},rows)
assert(out==[[11],[12]])
assert(a==[11])
assert(b==[12])
assert(rows[1]==[9])''')
add('join_unicode_nul_alias','join_success',r'''let nul=chr(0)
let a=["", "中", "🙂", nul, ""]
let alias=a
let result=join("|",a)
assert(result=="|中|🙂|"+nul+"|")
assert(join("",a)=="中🙂"+nul)
assert(join("x",[])=="")
assert(join("x",[""])=="")
alias[1]="changed"
assert(result=="|中|🙂|"+nul+"|")
assert(a[1]=="changed")''')
for name,body,err in [
 ('direct_arity','fn f(a,b)=>a+b\nf(1)','TypeError'),
 ('map_arity','fn f(a,b)=>a+b\nmap(f,[1])','TypeError'),
 ('reduce_arity','reduce((x)=>x,0,[1])','TypeError'),
 ('argv_alias_arity','let saved=argv\nfn f() => saved(1)\nf()','TypeError'),
 ('argv_hof_arity','map(argv,[1])','TypeError'),
 ('not_function','map(1,[1])','TypeError'),
 ('empty_min','minBy((x)=>x,[])','ValueError'),
 ('empty_max','maxBy((x)=>x,[])','ValueError'),
 ('nested_body_trace','fn inner(x)=>x/0\nfn middle(xs)=>map(inner,xs)\nfn outer()=>middle([3])\nouter()','ZeroDivisionError'),
 ('bad_bool','filter((x)=>x,[1])','TypeError'),
 ('join_sep_first','join(1,nil)','TypeError'),
 ('join_array_second','join(",",nil)','TypeError'),
 ('join_leftmost_type','join(",",["a",1,true])','TypeError'),
 ('join_self_cycle','let x=[]\nx[0]=x\njoin(",",x)','TypeError'),
 ('join_struct_method','join("/",[{f:()=>1}])','TypeError'),
 ('argument_effect_before_arity','fn mark(x) {eprint(x)\n x}\nfn f(a)=>a\nf(mark(1),mark(2))','TypeError'),
]:add(name,'join_error' if name.startswith('join') else 'error_trace',body,err)
# Construct actual self-cycle using push is immutable: allocate one mutable slot then point it back.
for index,(name,cat,src,err) in enumerate(cases):
 if name=='join_self_cycle':cases[index]=(name,cat,src.replace('let x=[]','let x=[nil]'),err)
rows=[]
for name,category,source,error in cases:
 p=W/'same-input.lfz';p.write_text(source,encoding='utf8',newline='\n')
 for mode in ['human','json']:
  args=['run']+(['--json'] if mode=='json' else [])+[str(p),'--','one','中文','']
  outputs=[]
  for label,exe in [('t12',OLD),('t13',NEW)]:
   r=subprocess.run([str(exe),*args],capture_output=True,cwd=R,env=env,timeout=10)
   outputs.append({'label':label,'exit':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()})
   expected=2 if error else 0
   assert r.returncode==expected,(name,label,expected,r.returncode,r.stdout,r.stderr)
   if mode=='json':
    data=json.loads(r.stdout)
    assert data['ok']==(error is None),(name,label,data)
    if error:assert data['error']==error,(name,label,error,data)
   elif not error:assert not r.stdout and not r.stderr,(name,label,r.stdout,r.stderr)
  assert {k:v for k,v in outputs[0].items() if k!='label'}=={k:v for k,v in outputs[1].items() if k!='label'},(name,mode,outputs)
  rows.append({'name':name,'category':category,'mode':mode,'source':source,'source_sha256':sha(p),'args':args,'expected_error':error,'outputs':outputs,'identical':True})
result={'date':'2026-10-27','old':{'path':str(OLD.relative_to(R)),'sha256':sha(OLD)},'new':{'path':str(NEW.relative_to(R)),'sha256':sha(NEW)},'programs':len(cases),'categories':sorted(set(x[1] for x in cases)),'comparisons':len(rows),'processes':len(rows)*2,'passed':len(rows),'rows':rows}
OUT.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
print(json.dumps({k:v for k,v in result.items() if k not in ('rows','old','new')},ensure_ascii=False))
