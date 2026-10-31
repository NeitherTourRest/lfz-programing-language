"""Public T18-B HTTP process tests; project-local, loopback only, no compilation."""
import argparse,datetime,hashlib,json,os,socket,subprocess,sys,threading,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()

class Loopback:
 def __init__(self):
  self.stop=threading.Event();self.records=[];self.errors=[];self.lock=threading.Lock()
  self.idle=threading.Event();self.idle.set()
  self.sock=socket.socket();self.sock.bind(('127.0.0.1',0));self.sock.listen(8);self.sock.settimeout(.05)
  self.base='http://127.0.0.1:'+str(self.sock.getsockname()[1]);self.worker=threading.Thread(target=self.serve)
 def start(self):self.worker.start()
 def snapshot(self):
  with self.lock:return list(self.records)
 def serve(self):
  while not self.stop.is_set():
   try:c,_=self.sock.accept()
   except socket.timeout:continue
   except OSError:break
   self.idle.clear()
   try:
    with c:
     c.settimeout(.15);raw=b'';end=time.monotonic()+1
     while b'\r\n\r\n' not in raw:
      if self.stop.is_set() or time.monotonic()>end:break
      raw+=c.recv(4096)
      if len(raw)>65536:raise ValueError('fixture request headers exceeded bound')
     if b'\r\n\r\n' not in raw:continue
     head,body=raw.split(b'\r\n\r\n',1);lines=head.split(b'\r\n');method,path,_=lines[0].decode('ascii').split(' ',2)
     headers=[line.decode('ascii').split(':',1) for line in lines[1:]]
     length=next((int(v.strip()) for k,v in headers if k.lower()=='content-length'),0)
     if length>1024*1024:raise ValueError('fixture request body exceeded bound')
     while len(body)<length:
      block=c.recv(min(4096,length-len(body)))
      if not block:break
      body+=block
     with self.lock:self.records.append({'method':method,'path':path,'headers':headers,'body_hex':body.hex(),'head_hex':head.hex(),'time':time.monotonic()})
     route=path.split('?',1)[0]
     if route=='/slowheaders' and self.stop.wait(.4):continue
     status={'/404':404,'/500':500,'/redirect':302,'/empty':204}.get(route,200)
     payload=b'ok'
     if route=='/echo':payload=body
     if route=='/utf8':payload=b'\xef\xbb\xbfa\r\n'
     if route=='/invalid':payload=b'\xff'
     if route=='/empty' or method=='HEAD':payload=b''
     extra=b'X-Dup: one\r\nX-Dup: two\r\nSet-Cookie: a=1\r\nSet-Cookie: b=2\r\n' if route=='/duplicate' else b''
     if route=='/redirect':extra+=('Location: '+self.base+'/target\r\n').encode()
     if route=='/cookie':extra+=b'Set-Cookie: hidden=secret\r\n'
     if route=='/drip':
      c.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\n')
      for _ in range(100):
       if self.stop.wait(.01):break
       c.sendall(b'x')
     else:
      c.sendall(('HTTP/1.1 '+str(status)+' Test\r\nContent-Length: '+str(len(payload))+'\r\n').encode()+extra+b'Connection: close\r\n\r\n')
      if route=='/slowbody' and self.stop.wait(.4):continue
      c.sendall(payload)
   except (OSError,ValueError) as e:
    # Client timeout/closing socket is expected; malformed fixture itself is recorded separately.
    if isinstance(e,ValueError):self.errors.append(repr(e))
   finally:self.idle.set()
 def close(self):
  self.stop.set();self.sock.close();self.worker.join(2)
  if self.worker.is_alive():raise RuntimeError('loopback worker failed bounded shutdown')

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--expected-sha',required=True)
 ap.add_argument('--output',type=Path,required=True);ap.add_argument('--prepare-only',action='store_true');ap.add_argument('--red-only',action='store_true');ap.add_argument('--header-witness',type=Path)
 a=ap.parse_args();exe=a.binary.resolve();out=a.output.resolve()
 assert exe.is_relative_to(ROOT) and out.is_relative_to(ROOT) and not out.exists()
 w=ROOT/'Temp/T18-B-public';w.mkdir(parents=True,exist_ok=True);out.parent.mkdir(parents=True,exist_ok=True)
 server=None;rows=[];cases=[];q=lambda x:json.dumps(x,ensure_ascii=False)
 try:
  if a.prepare_only:base='http://127.0.0.1:1'
  else:server=Loopback();server.start();base=server.base
  def call(path='/',method='GET',headers='[]',body='',ms=1000,cap=10000):return 'httpRequest('+q(method)+','+q(base+path)+','+headers+','+q(body)+','+str(ms)+','+str(cap)+')'
  def add(name,src,cls=None,msg=None,allow=True,args=(),usage=None,count=None,where=None):cases.append(dict(name=name,src=src,cls=cls,msg=msg,allow=allow,args=list(args),usage=usage,count=count,where=where))
  add('feature_red','let r='+call()+'\nassert(r.status==200)',count=1)
  for status in (404,500):add('status_'+str(status),'let r='+call('/'+str(status))+'\nassert(r.status=='+str(status)+')',count=1)
  add('POST_UTF8','let r='+call('/echo','POST',body='雪')+'\nassert(r.body=="雪")',count=1)
  add('query_percent','let r='+call('/%E9%9B%AA?q=1')+'\nassert(r.body=="ok")',count=1)
  add('duplicate_headers','let r='+call('/duplicate')+'\nassert(filter((p)=>p[0]=="x-dup",r.headers)==[["x-dup","one"],["x-dup","two"]])\nassert(len(filter((p)=>p[0]=="set-cookie",r.headers))==2)',count=1)
  add('empty','assert('+call('/empty')+'.body=="")',count=1)
  add('HEAD','assert('+call('/',method='HEAD')+'.body=="")',count=1)
  add('BOM_CRLF','assert('+call('/utf8')+'.body=="\ufeffa\\r\\n")',count=1)
  add('invalid_UTF8',call('/invalid'),cls='ValueError',msg='HTTP 响应 body 不是合法 UTF-8',count=1)
  add('redirect_no_follow','assert('+call('/redirect')+'.status==302)',count=1)
  add('zero_cap',call(cap=0),cls='ValueError',msg='HTTP 响应超过字节上限',count=1)
  for route in ('slowheaders','slowbody','drip'):
   add('local_'+route,'let r=attempt(()=>'+call('/'+route,ms=50)+')\nassert(!r.ok)\nassert(r.error.message=="HTTP 请求时间上限已到")',count=1)
  add('global_deadline','attempt(()=>'+call('/slowbody',ms=1000)+')',cls='ExecutionLimitError',msg='执行时间上限已到',args=['--timeout-ms','50'],count=1)
  for name,src,cls,msg in [('arity','httpRequest()','TypeError',None),('outer_type','httpRequest(1,"",[],"",1,1)','TypeError',None),('pair_shape','httpRequest("GET",'+q(base)+',[["x"]],"",1,1)','ValueError','HTTP headers 非法或超出上限')]:add(name,src,cls,msg,count=0)
  for method in ('get','TRACE'):
   add('method_'+method,call(method=method),cls='ValueError',msg='HTTP method 不受支持',count=0)
  for url in ('ftp://127.0.0.1/','http://u:p@127.0.0.1/','http://127.0.0.1/#bad','http://127.0.0.1/%zz','http://127.0.0.1:0/'):
   add('invalid_url_'+url,'httpRequest("GET",'+q(url)+',[],"",100,100)',cls='ValueError',msg='HTTP URL 非法或超出长度上限',count=0)
  for header in ('Host','Content-Length','Accept-Encoding'):
   add('reserved_'+header,call(headers='[['+q(header)+',"x"]]'),cls='ValueError',msg='HTTP headers 非法或超出上限',count=0)
  add('GET_body',call(body='x'),cls='ValueError',msg='HTTP body 非法或超出上限',count=0)
  add('timeout_range',call(ms=0),cls='ValueError',msg='HTTP timeoutMs 必须在 1..600000 范围内',count=0)
  add('cap_range',call(cap=-1),cls='ValueError',msg='HTTP maxResponseBytes 必须在 0..16777216 范围内',count=0)
  add('default_disabled',call(),cls='ValueError',msg='HTTP 能力未启用',allow=False,count=0)
  add('process_does_not_grant_http',call(),cls='ValueError',msg='HTTP 能力未启用',allow=False,args=['--allow-process'],count=0)
  add('later_session_disabled',call(),cls='ValueError',msg='HTTP 能力未启用',allow=False,count=0)
  add('duplicate_flag','1',args=['--allow-http'],usage="选项 '--allow-http' 不能重复",count=0)
  for value in ('','1'):add('equals_'+value,'1',allow=False,args=['--allow-http='+value],usage="选项 '--allow-http' 不接受值",count=0)
  for cmd in ('check','test','--help','-h','--version','-V'):add('inapplicable_'+cmd,'1',args=[cmd],usage='HTTP 能力选项只适用于运行命令',count=0)
  add('CODE_flag','--allow-http',allow=False,cls='NameError',count=0)
  add('argv_flag','assert(argv()==["--allow-http","--allow-http=1"])',args=['--','--allow-http','--allow-http=1'],count=0)
  lib=w/'fault.lfz';lib.write_text('#42\n'+call('/slowbody',ms=50)+'\n',encoding='utf8');entry=w/'entry.lfz';entry.write_text('#42\nimport m from "./fault.lfz"\n',encoding='utf8')
  add('module_source','',args=[str(entry)],cls='ValueError',msg='HTTP 请求时间上限已到',count=1,where=lib.name)
  pending=['exactN/N+1 requires independent RAW_HEADERS_CRLF witness','TLS self-signed rejection fixture','host cancel/callback diagnostics require unit/independent tests']
  if a.header_witness:
   evidence=json.loads(a.header_witness.read_text(encoding='utf8'));assert evidence['route']=='/' and evidence['body_hex']=='6f6b'
   expected_wire=b'HTTP/1.1 200 Test\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok'
   assert evidence['fixture_wire_sha256']==hashlib.sha256(expected_wire).hexdigest()
   raw=evidence['raw_headers_crlf'];assert raw.endswith('\r\n\r\n') and '\x00' not in raw
   cap=len(raw.encode('utf8'))+2
   add('exact_N',call(cap=cap),count=1);add('over_N_minus_one',call(cap=cap-1),cls='ValueError',msg='HTTP 响应超过字节上限',count=1)
   pending.pop(0)
  if a.prepare_only:result={'schema':'LFZ-T18B-PREPARED-1','cases_total':len(cases),'cases':[c['name'] for c in cases],'executed':False,'pending':pending}
  else:
   digest=sha(exe);assert digest==a.expected_sha.lower()
   for c in (cases[:1] if a.red_only else cases):
    assert sha(exe)==digest
    assert server.idle.wait(2),'previous fixture connection did not finish'
    start=len(server.snapshot());args=['--json']+(['--allow-http'] if c['allow'] else [])
    args+=c['args'] if c['args'] and (c['args'][0] in ('check','test','--help','-h','--version','-V') or c['where']) else ['-e',c['src'],*c['args']]
    row={'name':c['name'],'args':args};checks={};p=subprocess.Popen([str(exe),*args],cwd=w,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    try:
     so,se=p.communicate(timeout=6);so,se=so.decode('utf8'),se.decode('utf8');row.update(exit=p.returncode,stdout=so,stderr=se)
     checks['exit']=p.returncode==(2 if c['cls'] or c['usage'] else 0)
     if c['usage']:checks['usage']=not so and se.splitlines()[0]=='lfz: '+c['usage']
     else:
      obj=json.loads(so);checks['protocol']=list(obj)==(['ok','error','message','file','line','col','traceback'] if c['cls'] else ['ok']) and not se
      checks['ok']=obj.get('ok')==(not c['cls'])
      if c['cls']:checks['class']=obj.get('error')==c['cls']
      if c['msg']:checks['message']=obj.get('message')==c['msg']
      if c['where']:checks['source']=Path(obj.get('file','')).name==c['where']
    except (subprocess.TimeoutExpired,ValueError,UnicodeError) as err:
     row['exception']=repr(err);checks['completed']=False
     if p.poll() is None:p.kill();p.communicate(timeout=2)
    # A small observation window catches immediate unintended redirects/replays; no exactly-once claim.
    time.sleep(.06);observed=server.snapshot()[start:];row['requests']=observed
    if c['count'] is not None:checks['request_count']=len(observed)==c['count']
    row.update(checks=checks,passed=all(checks.values()));rows.append(row)
   assert sha(exe)==digest
   result={'schema':'LFZ-T18B-CLI-1','binary_sha256':digest,'script_sha256':sha(Path(__file__)),'checks_total':len(rows),'checks_passed':sum(r['passed'] for r in rows),'rows':rows,'pending':pending}
 finally:
  if server:server.close()
 result['timestamp_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();result['server_errors']=server.errors if server else []
 result['header_witness_sha256']=sha(a.header_witness) if a.header_witness else None
 out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps({k:v for k,v in result.items() if k not in ('rows','cases')},ensure_ascii=False))
 return 0 if a.prepare_only or result['checks_total']==result['checks_passed'] and not result['server_errors'] else 1
if __name__=='__main__':sys.exit(main())
