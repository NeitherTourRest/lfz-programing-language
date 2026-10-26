import argparse,hashlib,json,os,subprocess
from pathlib import Path
R=Path.cwd();W=R/'Temp/T12/verifier-while';W.mkdir(parents=True,exist_ok=True)
ap=argparse.ArgumentParser();ap.add_argument('--old',type=Path,default=R/'Temp/t12-perf/lfz-before.exe');ap.add_argument('--new',type=Path,default=R/'dist/lfz.exe');ap.add_argument('--output',type=Path,required=True);o=ap.parse_args()
old=o.old.resolve();new=o.new.resolve();out=o.output.resolve()
assert out.is_relative_to(R) and not out.exists()
assert hashlib.sha256(old.read_bytes()).hexdigest()=='149b184e442ffbf250590431fa0aeffab401eac526faf60b4b435f620725d5ab'
assert hashlib.sha256(new.read_bytes()).hexdigest()=='47ed2bfe2a2694b11bd611172cfcf0000d0bfc58e3b9bd809ca95f066dd0261f'
env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1')
cases=[]
def add(name,category,source,code=0):cases.append((name,category,source,code))
add('outer_cell','outer_capture','''fn make() {
 var x=0
 var i=0
 let fs=[nil,nil,nil]
 while i<3 {
  fs[i]=()=>x
  x+=1
  i+=1
 }
 x=99
 print([fs[0](),fs[1](),fs[2]()])
}
make()''')
add('let_fresh','direct_declarations','''var i=0
let fs=[nil,nil,nil]
while i<3 {
 let x=i
 fs[i]=()=>x
 i+=1
}
print([fs[0](),fs[1](),fs[2]()])''')
add('var_fresh','direct_declarations','''var i=0
let fs=[nil,nil,nil]
while i<3 {
 var x=i
 fs[i]=()=>{x+=1
 x}
 i+=1
}
print([fs[0](),fs[1](),fs[2](),fs[0]()])''')
add('fn_fresh','direct_declarations','''var i=0
let fs=[nil,nil]
while i<2 {
 fn f()=>f
 fs[i]=f
 i+=1
}
print([fs[0]()==fs[0],fs[1]()==fs[1],fs[0]==fs[1]])''')
add('struct_fresh','direct_declarations','''var i=0
let fs=[nil,nil,nil]
while i<3 {
 struct S { n:i, fn value()=>self.n }
 fs[i]=S{}
 i+=1
}
print([fs[0].value(),fs[1].value(),fs[2].value()])''')
add('unreachable_decl','unreachable_declarations','''var i=0
while i<3 {
 i+=1
 continue
 let never=999
}
print(i)''')
add('nested_if_shadow','nested_scope','''let x=42
var i=0
let fs=[nil,nil,nil]
while i<3 {
 if true { let x=i
  fs[i]=()=>x }
 i+=1
}
print([x,fs[0](),fs[1](),fs[2]()])''')
add('nested_for_fresh','nested_scope','''var i=0
let fs=[nil,nil,nil,nil]
while i<2 {
 for n in range(2) { fs[i*2+n]=()=>n }
 i+=1
}
print([fs[0](),fs[1](),fs[2](),fs[3]()])''')
add('nested_while','nested_scope','''var i=0
var j=0
let fs=[nil,nil,nil,nil]
while i<2 {
 j=0
 while j<2 {
  let x=i*2+j
  fs[x]=()=>x
  j+=1
 }
 i+=1
}
print([fs[0](),fs[1](),fs[2](),fs[3]()])''')
add('dump_order','dump_visibility','''let alpha=1
fn work(){
 let alpha=2
 let beta=3
 var i=0
 while i<2 {
  ;;
  if true { let beta=9
   ;; }
  i+=1
 }
}
work()''')
add('closure_dump','dump_visibility','''fn work(){
 let alpha=2
 var i=0
 let fs=[nil,nil]
 while i<2 {
  fs[i]=()=>{;;
   alpha+i}
  i+=1
 }
 print([fs[0](),fs[1]()])
}
work()''')
add('break_continue_return','flow','''fn work(){
 var i=0
 var total=0
 while i<9 {
  i+=1
  if i==2 {continue}
  if i==6 {return total}
  total+=i
 }
 -1
}
print(work())
var n=0
while true {
 n+=1
 if n==3 {break}
}
print(n)''')
add('empty_body','empty_body','''var i=0
fn next(){i+=1
 i<4}
while next() {}
while false {}
print(i)''')
add('unknown_name','errors_no_implicit_binding','''while true {
 ghost=1
 break
}''',2)
add('immutable_let','errors_no_implicit_binding','''let x=0
while x<1 {
 x=1
}''',2)
add('error_trace','errors_no_implicit_binding','''fn failLoop(){
 var i=0
 while i<2 {
  i+=1
  if i==2 {fail("expected trace")}
 }
}
failLoop()''',2)
rows=[]
for name,category,source,code in cases:
 f=W/(name+'.lfz');f.write_text('#42\n'+source+'\n',encoding='utf8',newline='')
 results={}
 for label,exe in [('old',old),('new',new)]:
  p=subprocess.run([str(exe),'run',str(f)],cwd=R,env=env,capture_output=True,timeout=15)
  results[label]={'exit':p.returncode,'stdout':p.stdout.decode('utf8'),'stderr':p.stderr.decode('utf8')}
 assert results['old']['exit']==code,(name,results)
 assert results['old']==results['new'],(name,results)
 rows.append({'name':name,'category':category,'source':f.read_text(encoding='utf8'),'expected_exit':code,'equal':True,'runs':results})
result={'scope':'independent old/new semantic differential for spec16.7','date':'2026-10-25','old_sha256':hashlib.sha256(old.read_bytes()).hexdigest(),'new_sha256':hashlib.sha256(new.read_bytes()).hexdigest(),'passed':len(rows),'categories':sorted({x['category'] for x in rows}),'cases':rows}
out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print(json.dumps({'passed':len(rows),'categories':len(result['categories'])}))