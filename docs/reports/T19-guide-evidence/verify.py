"""T19 人指南小样；prepare-only不启动产品，所有文件限定项目内。"""
import argparse, hashlib, json, re, subprocess, sys, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
GUIDES=[ROOT/'docs/guide'/n for n in ['README.md','reference.md','tutorial.md','errors.md','testing.md','line-processing.md']]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def inside(p):
 p=p.resolve()
 if not p.is_relative_to(ROOT):raise ValueError('path must stay inside project')
 return p
def lit(x):return json.dumps(x,ensure_ascii=False)
def main():
 ap=argparse.ArgumentParser()
 ap.add_argument('--prepare-only',action='store_true')
 ap.add_argument('--binary');ap.add_argument('--expected-sha256');ap.add_argument('--output')
 a=ap.parse_args()
 work=ROOT/'Temp/T19-docs';work.mkdir(parents=True,exist_ok=True)
 text=(ROOT/'docs/guide/line-processing.md').read_text(encoding='utf-8')
 blocks=re.findall(r'```lfz\n(.*?)```',text,re.S)
 if len(blocks)!=1:raise ValueError('expected exactly one extracted LFZ tutorial')
 tutorial=work/'jsonl.lfz';tutorial.write_text(blocks[0],encoding='utf-8')
 (work/'input.jsonl').write_bytes(b'{"n":1}\nbroken\n\n{"n":2}\n')
 fixtures={'empty':b'', 'basic':b'a\r\nb\r', 'lf':b'\n\n', 'tail':b'a\n', 'bom':b'\xef\xbb\xbfA\n', 'bad':b'good\n\xff', 'over':b'good\n12345\n', 'crlf':b'a\r\n', 'unicode':'雪\n'.encode(), 'nul':b'a\x00b\n', 'spaced':b'  \n#42\n', 'cross':b'a'*8191+'雪\n'.encode()}
 paths={}
 for name,data in fixtures.items():
  path=work/(name+'.data');path.write_bytes(data);paths[name]=lit(str(path))
 if a.prepare_only:
  print(json.dumps({'prepared':str(tutorial.relative_to(ROOT)), 'fixtures':len(fixtures),'executed':False},ensure_ascii=False));return
 if not all([a.binary,a.expected_sha256,a.output]):ap.error('binary/expected-sha256/output required')
 binary=inside(ROOT/a.binary);out=inside(ROOT/a.output)
 if out.exists() or out.with_suffix('.partial.json').exists():raise ValueError('output must be new')
 if sha(binary)!=a.expected_sha256:raise ValueError('binary hash mismatch')
 guide_hash={str(p.relative_to(ROOT)):sha(p) for p in GUIDES}
 evidence={'binary':str(binary),'sha256':sha(binary),'bytes':binary.stat().st_size,'guide_hashes':guide_hash,'runs':[]}
 out.parent.mkdir(parents=True,exist_ok=True)
 def save():out.with_suffix('.partial.json').write_text(json.dumps(evidence,ensure_ascii=False,indent=2),encoding='utf-8')
 def invoke(name,args,exit_code=0,expected=None,error=None,check=False):
  if sha(binary)!=a.expected_sha256:raise ValueError('binary changed')
  start=time.monotonic();r=subprocess.run([str(binary),*args],cwd=ROOT,capture_output=True,timeout=15)
  row={'name':name,'argv':args,'exit':r.returncode,'stdout':r.stdout.decode('utf-8',errors='strict'),'stderr':r.stderr.decode('utf-8',errors='strict'),'seconds':time.monotonic()-start}
  evidence['runs'].append(row);save()
  assert r.returncode==exit_code,(name,row)
  if check:assert not row['stdout'] and not row['stderr'],row
  elif error:
   obj=json.loads(row['stdout']);assert obj['error']==error,(name,obj)
  else:
   assert not row['stderr'],row
   if expected is not None:assert json.loads(row['stdout'])==expected,(name,row,expected)
  row['pass']=True;save()
 def case(name,code,expected=None,error=None,opts=()):
  code=code.replace(';','\n')
  invoke(name+'/check',['check','-e',code],check=True)
  invoke(name+'/run',['run',*opts,'--json','-e',code] if error else ['run',*opts,'-e',code],exit_code=2 if error else 0,expected=expected,error=error)
 invoke('tutorial/check',['check',str(tutorial)],check=True)
 invoke('tutorial/run',['run','--max-steps','10000','--timeout-ms','2000',str(tutorial)],expected={'accepted':2,'rejected':2,'sum':3,'lines':4})
 def collect(key,cap=32):return 'var rows=[]; let r=forEachLine((s,n)=>{rows=push([s,n],rows); return true},'+str(cap)+','+paths[key]+'); print(jsonStringify({rows:rows,result:r}))'
 for key,rows,cap in [('empty',[],1),('basic',[['a',1],['b\r',2]],2),('lf',[['',1],['',2]],1),('tail',[['a',1]],1),('bom',[['\ufeffA',1]],4),('unicode',[['雪',1]],3),('nul',[['a\x00b',1]],3),('spaced',[['  ',1],['#42',2]],3)]:
  case(key,collect(key,cap),{'rows':rows,'result':{'lines':len(rows),'stopped':False}})
 case('cross8KiB','var n=0;let r=forEachLine((s,i)=>{n=len(s);return true},8194,'+paths['cross']+'); print(jsonStringify([n,r.lines]))',[8192,1])
 case('exactCRLF',collect('crlf',2),{'rows':[['a',1]],'result':{'lines':1,'stopped':False}})
 case('crlfNplus1','forEachLine((s,n)=>true,1,'+paths['crlf']+')',error='ValueError')
 case('utf8ByteCap','forEachLine((s,n)=>true,2,'+paths['unicode']+')',error='ValueError')
 case('badUTF8','forEachLine((s,n)=>true,32,'+paths['bad']+')',error='ValueError')
 for key in ['bad','over']:
  case('stopBefore/'+key,'print(jsonStringify(forEachLine((s,n)=>false,4,'+paths[key]+'))) ',{'lines':1,'stopped':True})
 case('strictBool','forEachLine((s,n)=>nil,32,'+paths['tail']+')',error='TypeError')
 case('emptyArity','forEachLine((s)=>true,32,'+paths['empty']+')',error='TypeError')
 case('range','forEachLine((s,n)=>true,0,'+paths['empty']+')',error='ValueError')
 case('outerAttemptEffects','var count=0;let r=attempt(()=>forEachLine((s,n)=>{count=count+1;return true},4,'+paths['over']+')); print(jsonStringify([r.ok,r.error["class"],count]))',[False,'ValueError',1])
 case('budgetNotCaught','attempt(()=>forEachLine((s,n)=>{while true {}\nreturn true},32,'+paths['tail']+'))',error='ExecutionLimitError',opts=['--max-steps','100'])
 if sha(binary)!=a.expected_sha256 or guide_hash!={str(p.relative_to(ROOT)):sha(p) for p in GUIDES}:raise ValueError('input changed')
 evidence['passed']=len(evidence['runs']);evidence['complete']=True
 out.write_text(json.dumps(evidence,ensure_ascii=False,indent=2),encoding='utf-8')
 print(json.dumps({'passed_processes':evidence['passed'],'sha256':sha(binary),'output':str(out)},ensure_ascii=False))
if __name__=='__main__':main()
