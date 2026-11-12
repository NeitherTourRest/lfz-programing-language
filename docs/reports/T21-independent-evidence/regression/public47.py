"""T19 public CLI and live-PID file-close tests. All fixtures remain project-local."""
import argparse,ctypes,datetime,hashlib,json,os,subprocess,sys,time
from pathlib import Path
ROOT=Path.cwd()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def q(s):return json.dumps(str(s),ensure_ascii=False).replace('\\u0000','\x00')
def exclusive(path):
 k=ctypes.WinDLL('kernel32',use_last_error=True)
 k.CreateFileW.argtypes=[ctypes.c_wchar_p,ctypes.c_uint32,ctypes.c_uint32,ctypes.c_void_p,ctypes.c_uint32,ctypes.c_uint32,ctypes.c_void_p];k.CreateFileW.restype=ctypes.c_void_p
 k.CloseHandle.argtypes=[ctypes.c_void_p];k.CloseHandle.restype=ctypes.c_int
 h=k.CreateFileW(str(path),0x80000000,0,None,3,0x80,None);err=ctypes.get_last_error()
 ok=h!=ctypes.c_void_p(-1).value
 if ok:assert k.CloseHandle(h)
 return {'ok':ok,'error':0 if ok else err}
def waitfile(path,p,seconds=2):
 end=time.monotonic()+seconds
 while time.monotonic()<end:
  if path.exists() and path.read_bytes():return
  if p.poll() is not None:raise RuntimeError('LFZ exited before handshake '+path.name)
  time.sleep(.005)
 raise TimeoutError('handshake '+path.name)
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--expected-sha',required=True);ap.add_argument('--output',type=Path,required=True);ap.add_argument('--prepare-only',action='store_true');ap.add_argument('--red-only',action='store_true');a=ap.parse_args()
 exe,out=a.binary.resolve(),a.output.resolve();assert exe.is_relative_to(ROOT) and out.is_relative_to(ROOT) and not out.exists()
 w=ROOT/'Temp/T21-verifier/regression/public47';w.mkdir(parents=True,exist_ok=True);out.parent.mkdir(parents=True,exist_ok=True);cases=[]
 def file(name,raw):
  p=w/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(raw);return p
 def add(name,src,cls=None,msg=None,args=(),probe=None,where=None):
  source=file(name+'.lfz',('#42\n'+src+'\n').encode('utf8'));cases.append(dict(name=name,source=source,cls=cls,msg=msg,args=list(args),probe=probe,where=where))
 def call(path,cb='(s,n)=>true',cap=10000):return 'forEachLine('+cb+','+str(cap)+','+q(str(path))+')'
 def collect(name,raw,expected,cap=10000):
  p=file(name+'.data',raw)
  src='var seen=[]\nlet r='+call(p,'(s,n)=>{\nseen=push([n,s],seen)\ntrue\n}',cap)+'\nassert(seen=='+json.dumps([[i+1,s] for i,s in enumerate(expected)],ensure_ascii=False).replace('\\u0000','\x00')+')\nassert(r.lines=='+str(len(expected))+')\nassert(!r.stopped)\nassert(keys(r)==["lines","stopped"])'
  add(name,src)
 collect('feature_red',b'a\n',['a'])
 for name,raw,expected in [('empty',b'',[]),('single',b'a',['a']),('tailLF',b'a\n',['a']),('oneLF',b'\n',['']),('twoLF',b'\n\n',['','']),('CRLF_EOFCR',b'a\r\nb\r',['a','b\r']),('doubleCR',b'a\r\r\n',['a\r']),('BOM',b'\xef\xbb\xbfa\n',['\ufeffa']),('NUL',b'a\0b\n',['a\0b']),('whitespace',b' \t\n#42\n',[' \t','#42'])]:collect(name,raw,expected)
 collect('cross_UTF8',b'a'*8191+'雪'.encode()+b'\n',['a'*8191+'雪'])
 for name,raw,cap in [('capN',b'a\n',1),('CR_capN',b'a\r\n',2),('UTF8capN','雪\n'.encode(),3)]:collect(name,raw,[raw.rstrip(b'\n').removesuffix(b'\r').decode()],cap)
 for name,raw,cap,line in [('capPlus',b'ab\n',1,1),('CRcapPlus',b'a\r\n',1,1),('UTF8capPlus','雪'.encode(),2,1),('secondTooLong',b'a\nxxxx',3,2)]:
  add(name,call(file(name+'.data',raw),cap=cap),cls='ValueError',msg='第 '+str(line)+' 行超过逐行字节上限 '+str(cap))
 for name,raw,line in [('badUTF8',b'\xff',1),('laterUTF8',b'a\n\xff',2),('crossBadUTF8',b'a'*8191+b'\xe9x',1)]:add(name,call(file(name+'.data',raw)),cls='ValueError',msg='第 '+str(line)+' 行不是合法 UTF-8')
 for name,raw in [('earlyUTF8',b'a\n\xff'),('earlyLong',b'a\n'+b'x'*1000)]:
  add(name,'var count=0\nlet r='+call(file(name+'.data',raw),'(s,n)=>{\ncount+=1\nfalse\n}',2)+'\nassert(count==1)\nassert(r.lines==1)\nassert(r.stopped)')
 empty=file('empty-validation.data',b'');valid=file('valid.data',b'a\nb\n')
 for name,src,cls,msg in [('arity','forEachLine()','TypeError',None),('cbType','forEachLine(1,1,'+q(empty)+')','TypeError',None),('capType','forEachLine((s,n)=>true,"1",'+q(empty)+')','TypeError',None),('pathType','forEachLine((s,n)=>true,1,1)','TypeError',None),('callbackArity',call(empty,'()=>true'),'TypeError',None),('argvArity',call(empty,'argv'),'TypeError',None),('nonbool',call(valid,'(s,n)=>1'),'TypeError',None),('pathNUL',call(str(valid)+'\x00'),'ValueError','逐行读取路径不能包含 NUL')]:add(name,src,cls,msg)
 for cap in (0,-1,16777217):add('range'+str(cap),call(empty,cap=cap),cls='ValueError',msg='逐行读取 maxLineBytes 必须在 1..16777216 范围内')
 add('missing',call(w/'missing.data'),cls='IOError',msg='无法逐行读取：'+str(w/'missing.data'))
 add('directory',call(w),cls='IOError',msg='无法逐行读取：'+str(w))
 jp=file('mixed.jsonl',b'{"x":2}\nbad\n\n{"x":3}\n')
 add('JSONL_attempt','var good=0\nvar bad=0\nvar sum=0\nlet r='+call(jp,'(s,n)=>{\nlet r=attempt(()=>jsonParse(s))\nif r.ok {\ngood+=1\nsum+=r.value.x\n} else {\nbad+=1\n}\ntrue\n}')+'\nassert(good==2)\nassert(bad==2)\nassert(sum==5)\nassert(r.lines==4)')
 add('steps_shared',call(valid,'(s,n)=>{\nwhile true {}\n}'),cls='ExecutionLimitError',msg='执行步数预算已耗尽',args=['--max-steps','100'])
 lib=file('home/lib.lfz',('#42\nlet value=7\nfn cb(s,n){\nassert(value==7)\ntrue\n}\nfn bad(s,n){\n1/0\n}\nexport cb,bad\n').encode())
 add('home','import m from "./home/lib.lfz"\nlet value=99\nlet r='+call(valid,'m.cb')+'\nassert(r.lines==2)')
 add('callback_source','import m from "./home/lib.lfz"\n'+call(valid,'m.bad'),cls='ZeroDivisionError',where=lib.name)
 # All recoverable outcomes are checked before this same LFZ PID is allowed to exit.
 for mode,raw,outcome in [('EOF',b'a\n','true'),('false',b'a\n','false'),('callbackError',b'a\n','1/0'),('nonbool',b'a\n','1'),('UTF8',b'a\n\xff','true'),('tooLong',b'a\nxxxx','true')]:
  d=w/('close_'+mode);d.mkdir(exist_ok=True);p=file('close_'+mode+'/input',raw)
  go=file('close_'+mode+'/go',b'');release=file('close_'+mode+'/release',b'');ready=d/'ready';done=d/'done'
  for old in (ready,done):
   if old.exists():old.unlink()
  cb='(s,n)=>{\nwriteFile('+q(ready)+',"ready")\nwhile readFile('+q(go)+')=="" {}\n'+outcome+'\n}'
  src='let r=attempt(()=>'+call(p,cb,2)+')\nwriteFile('+q(done)+',"done")\nwhile readFile('+q(release)+')=="" {}'
  add('close_'+mode,src,args=['--timeout-ms','6000'],probe=dict(path=p,go=go,release=release,ready=ready,done=done))
 if a.prepare_only:result={'schema':'LFZ-T19-PREPARED-1','cases_total':len(cases),'executed':False,'cases':[c['name'] for c in cases]}
 else:
  digest=sha(exe);assert digest==a.expected_sha.lower();rows=[]
  for c in (cases[:1] if a.red_only else cases):
   assert sha(exe)==digest;args=['--json',str(c['source']),*c['args']];row={'name':c['name'],'args':args};checks={};probe=c['probe']
   if probe:
    assert os.name=='nt';row['unheld_control']=exclusive(probe['path']);assert row['unheld_control']['ok']
   p=subprocess.Popen([str(exe),*args],cwd=w,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
   try:
    if probe:
     waitfile(probe['ready'],p);row['held_probe']=exclusive(probe['path']);checks['held_negative']=not row['held_probe']['ok'] and row['held_probe']['error']==32
     probe['go'].write_bytes(b'go');waitfile(probe['done'],p);row['after_probe']=exclusive(probe['path']);checks['same_pid_alive']=p.poll() is None;checks['returned_closed']=row['after_probe']['ok'];probe['release'].write_bytes(b'exit')
    so,se=p.communicate(timeout=8);so,se=so.decode('utf8'),se.decode('utf8');row.update(exit=p.returncode,stdout=so,stderr=se);obj=json.loads(so)
    checks['exit']=p.returncode==(2 if c['cls'] else 0);checks['protocol']=not se and list(obj)==(['ok','error','message','file','line','col','traceback'] if c['cls'] else ['ok']);checks['ok']=obj.get('ok')==(not c['cls'])
    if c['cls']:checks['class']=obj.get('error')==c['cls']
    if c['msg']:checks['message']=obj.get('message')==c['msg']
    if c['where']:checks['source']=Path(obj.get('file','')).name==c['where']
   except (ValueError,UnicodeError,RuntimeError,TimeoutError,subprocess.TimeoutExpired) as e:
    row['exception']=repr(e);checks['completed']=False
    if p.poll() is None:p.kill();p.communicate(timeout=2)
   row.update(checks=checks,passed=all(checks.values()));rows.append(row)
  assert sha(exe)==digest
  result={'schema':'LFZ-T19-CLI-1','binary_sha256':digest,'script_sha256':sha(Path(__file__)),'checks_total':len(rows),'checks_passed':sum(r['passed'] for r in rows),'rows':rows,'scope':'small public fixtures; global terminal close requires same-host unit evidence, not process-exit reopen'}
 result['timestamp_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps({k:v for k,v in result.items() if k not in ('rows','cases')},ensure_ascii=False))
 return 0 if a.prepare_only or result['checks_total']==result['checks_passed'] else 1
if __name__=='__main__':sys.exit(main())
