"""T19 AI作者聚焦小样。prepare-only不启动LFZ；最终需明确binary/hash。"""
import argparse, datetime, hashlib, json, os, re, subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[3]
W=R/'Temp/T19-ai'
DOCS=['.opencode/skills/lfz-programming/'+n for n in ['SKILL.md','README.md','VERIFICATION.md','prompt-template.md']]+['docs/guide/ai/README.md']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def lit(x):return json.dumps(x,ensure_ascii=False)
def inside(p):
 p=p.resolve()
 if not p.is_relative_to(R):raise ValueError('project paths only')
 return p
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--prepare-only',action='store_true');ap.add_argument('--binary',type=Path);ap.add_argument('--expected-sha256');ap.add_argument('--output',type=Path);a=ap.parse_args()
 assert a.output,'new output path required'
 out=inside(R/a.output);assert not out.exists(),'preserve existing results'
 W.mkdir(parents=True,exist_ok=True);out.parent.mkdir(parents=True,exist_ok=True)
 docs={n:sha(R/n)for n in DOCS};cases=[]
 fixtures={'empty':b'','crlf':b'a\r\nb\r','lf':b'\n\n','tail':b'a\n','bom':b'\xef\xbb\xbfA\n','unicode':'雪\n'.encode(),'nul':b'a\0b\n','space':b'  \n#42\n','bad':b'good\n\xff','over':b'good\n12345\n','exactcr':b'a\r\n','cross':b'a'*8191+'雪\n'.encode(),'input':b'{"n":1}\nbroken\n\n{"n":2}\n'}
 for n,data in fixtures.items():(W/(n+'.jsonl' if n=='input'else n+'.data')).write_bytes(data)
 def path(n):return lit(str(W/(n+'.data')))
 def add(n,code,expected=None,error=None,opts=None):
  p=W/(n+'.lfz');p.write_text((code if code.startswith('#42\n')else '#42\n'+code)+'\n',encoding='utf8',newline='\n');cases.append(dict(name=n,path=p,expected=expected,error=error,opts=opts or []))
 for i,n in enumerate([DOCS[0],DOCS[-1]]):
  text=(R/n).read_text(encoding='utf8').split('## 6.8 T19：')[1]
  code=re.search(r'```lfz\n(.*?)\n```',text,re.S)[1]
  add('extracted_'+str(i),code,{'accepted':2,'rejected':2,'total':3,'result':{'lines':4,'stopped':False}},opts=['--max-steps','10000','--timeout-ms','2000'])
 for n,rows,cap in [('empty',[],1),('crlf',[['a',1],['b\r',2]],2),('lf',[['',1],['',2]],1),('tail',[['a',1]],1),('bom',[['\ufeffA',1]],4),('unicode',[['雪',1]],3),('nul',[['a\0b',1]],3),('space',[['  ',1],['#42',2]],3)]:
  add(n,'var rows=[]\nlet r=forEachLine((s,n)=>{\nrows=push([s,n],rows)\nreturn true\n},'+str(cap)+','+path(n)+')\nprint(jsonStringify({rows:rows,result:r}))',{'rows':rows,'result':{'lines':len(rows),'stopped':False}})
 add('cross','let r=forEachLine((s,n)=>{\nassert(len(s)==8192 && endsWith("雪",s))\nreturn true\n},8194,'+path('cross')+')\nprint(jsonStringify(r))',{'lines':1,'stopped':False})
 add('exactcr','print(jsonStringify(forEachLine((s,n)=>true,2,'+path('exactcr')+')))',{'lines':1,'stopped':False})
 for n,data,cap,msg in [('cr_counted','exactcr',1,'第 1 行超过逐行字节上限 1'),('over','over',4,'第 2 行超过逐行字节上限 4'),('utf8','bad',32,'第 2 行不是合法 UTF-8')]:
  add(n,'var count=0\nlet r=attempt(()=>forEachLine((s,n)=>{\ncount=count+1\nreturn true\n},'+str(cap)+','+path(data)+'))\nassert(!r.ok && r.error["class"]=="ValueError" && r.error.message=='+lit(msg)+')\nprint(jsonStringify(count))',0 if n=='cr_counted'else 1)
 for n in ['bad','over']:
  add('stop_'+n,'print(jsonStringify(forEachLine((s,n)=>false,4,'+path(n)+')))',{'lines':1,'stopped':True})
 for n,expr,kind in [('nonbool','forEachLine((s,n)=>0,32,'+path('tail')+')','TypeError'),('arity_empty','forEachLine((s)=>true,1,'+path('empty')+')','TypeError'),('range','forEachLine((s,n)=>true,0,'+path('empty')+')','ValueError'),('directory','forEachLine((s,n)=>true,1,'+lit(str(W))+')','IOError')]:
  add(n,expr,error=kind)
 add('global_budget','attempt(()=>forEachLine((s,n)=>{\nwhile true {\n1\n}\nreturn true\n},32,'+path('tail')+'))',error='ExecutionLimitError',opts=['--max-steps','100'])
 # check-only witness: execution would create marker then fail opening nonexistent data.
 marker=W/'check-marker.txt';assert not marker.exists()
 witness=W/'check_only.lfz';witness.write_text('#42\nwriteFile('+lit(str(marker))+',"executed")\nforEachLine((s,n)=>true,32,'+lit(str(W/'never-created.data'))+')\n',encoding='utf8')
 evidence=dict(mode='prepare-only'if a.prepare_only else'final',timestamp_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),document_hashes_before=docs,prepared_cases=len(cases),fixture_hashes={n:sha(W/(n+'.jsonl'if n=='input'else n+'.data'))for n in fixtures},source_hashes={c['name']:sha(c['path'])for c in cases},runs=[])
 def save():out.write_text(json.dumps(evidence,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
 if a.prepare_only:
  evidence.update(interpreter_launches=0,network_launches=0);save();print(json.dumps({'prepared_cases':len(cases),'interpreter_launches':0}));return
 assert a.binary and a.expected_sha256
 binary=inside(R/a.binary);H=a.expected_sha256.lower();assert sha(binary)==H
 evidence.update(binary=str(binary.relative_to(R)),binary_sha256=H,binary_bytes=binary.stat().st_size)
 env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1')
 def run(n,args,expected=None,error=None,check=False):
  assert sha(binary)==H
  row=dict(name=n,args=args);evidence['runs'].append(row);save()
  try:
   p=subprocess.run([str(binary),*args],cwd=R,env=env,capture_output=True,timeout=15)
   row.update(exit=p.returncode,stdout=p.stdout.decode('utf8'),stderr=p.stderr.decode('utf8'))
   if check:ok=p.returncode==0 and not p.stdout and not p.stderr
   elif error:ok=p.returncode==2 and not p.stderr and json.loads(row['stdout'])['error']==error
   else:ok=p.returncode==0 and not p.stderr and json.loads(row['stdout'])==expected
   row['passed']=ok
  except Exception as ex:row.update(passed=False,host_exception=repr(ex))
  save();return row['passed']
 run('check_no_execution',['check',str(witness)],check=True)
 evidence['check_marker_absent']=not marker.exists();save()
 for c in cases:
  if not run(c['name']+'/check',['check',str(c['path'])],check=True):continue
  args=['run',*c['opts']]+(['--json']if c['error']else[])+[str(c['path'])]
  if c['name'].startswith('extracted_'):args+=['--','Temp/T19-ai/input.jsonl']
  run(c['name']+'/run',args,c['expected'],c['error'])
 evidence['document_hashes_after']={n:sha(R/n)for n in DOCS};evidence['documents_unchanged']=docs==evidence['document_hashes_after']
 evidence['passed']=sum(x.get('passed',False)for x in evidence['runs']);evidence['total']=len(evidence['runs']);save()
 ok=evidence['documents_unchanged'] and evidence['check_marker_absent'] and evidence['passed']==evidence['total'] and len(evidence['runs'])==1+2*len(cases)
 print(json.dumps({'passed':evidence['passed'],'total':evidence['total'],'complete':ok}));raise SystemExit(0 if ok else 1)
if __name__=='__main__':main()
