from pathlib import Path
import hashlib,json,subprocess
r=Path.cwd();w=r/'Temp/T27-verifier'; exe=r/'Temp/T27/candidate.exe'; feat=r/'Temp/T27/feature.exe'; h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest(); assert h(exe)=='4816058d402a5d25c0455a0f9c5329febb0dd60629fc27d18654213a947ac636'; assert h(feat)=='d752b9cc7d66622bc1403c591e520e1d0e5df5e9c0e9cc7f65efe56b951179d4';assert h(r/'src/builtins.rs')=='111acf0a4b80d1c094b5c95be49f59ebce1bd37bb161a3cd8091dee85f78e6df'
rows=[];passed=[]
def run(bin,argv):
 p=subprocess.run([str(bin),*argv],cwd=w,capture_output=True,timeout=8);rows.append(dict(sha=h(bin),argv=argv,exit=p.returncode,stdout=p.stdout.decode('utf8'),stderr=p.stderr.decode('utf8')));return p
def pair(name,body,oracle):
 a=run(feat,['-e',body]);b=run(exe,['-e',body]);assert (a.returncode,a.stdout,a.stderr)==(b.returncode,b.stdout,b.stderr)==(0,oracle.encode(),b''),name;passed.append(name)
try:
 pair('argument-order-and-priority','''var log=[]
fn p() { log=push("P",log)
 return 0 }
fn xs() { log=push("A",log)
 return false }
let e=attempt(()=>findIndex(p(),xs()))
let f=attempt(()=>findIndex(0,[]))
print(jsonStringify(log))
print(e.error.message == f.error.message)
''','["P","A"]\ntrue\n')
 pair('nested-reentrant-search','''let xs=[1,2,3]
let nested=(x)=>findIndex((y)=>x+y==5,xs)!=nil
print(findIndex(nested,xs))
''','1\n')
 pair('member-dispatch','''let obj={findIndex:(x)=>x+2}
let key="findIndex"
print(obj.findIndex(3))
print(obj[key](4))
''','5\n6\n')
 pair('callback-nonbool-source','''fn bad(x) { nil }
fn a() { findIndex(bad,[1]) }
let result=attempt(a)
print(result.error.line)
print(result.error.col)
print(result.error["class"])
''','2\n10\nTypeError\n')
 pair('join-empty-delimiter-and-priority','''print(jsonStringify(join("",["", "𠀀", "雪", ""])))
let e=attempt(()=>join(1,[false]))
let f=attempt(()=>join(1,[]))
let p=attempt(()=>join("-",["x",false,17]))
let q=attempt(()=>join("-",[false]))
print(e.error.message == f.error.message)
print(p.error.message == q.error.message)
''','"𠀀雪"\ntrue\ntrue\n')
 b=run(exe,['--json','--max-steps','100','-e','''let bad=attempt(()=>findIndex((x)=>{while true {
  x+1
 }},[1]))
print("continued")
''']); v=json.loads(b.stdout);assert b.returncode==2 and v['error']=='ExecutionLimitError' and 'continued' not in b.stderr.decode();passed.append('resource-error-not-catchable')
finally:
 (w/'function.json').write_text(json.dumps(dict(passed=passed,runs=rows),ensure_ascii=False,indent=2)+'\n',encoding='utf8'); print(passed)
