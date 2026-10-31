import argparse,datetime,hashlib,http.server,json,os,re,subprocess,threading,time
from pathlib import Path
R=Path.cwd();ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path);ap.add_argument('--expected-sha256');ap.add_argument('--output',type=Path);ap.add_argument('--prepare-only',action='store_true');a=ap.parse_args()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
G=[R/'docs/guide'/n for n in ['README.md','reference.md','tutorial.md','testing.md','errors.md','http.md']];before={str(p.relative_to(R)):sha(p)for p in G}
W=R/'Temp/T18-B-docs';W.mkdir(parents=True,exist_ok=True);text=(R/'docs/guide/http.md').read_text(encoding='utf8');source=re.search(r'```lfz\n(.*?)\n```',text,re.S).group(1);P=W/'fetch.lfz';P.write_text(source+'\n',encoding='utf8',newline='\n')
if a.prepare_only:print('prepare-only: tutorial extracted; interpreter launches 0; server starts 0');raise SystemExit
assert a.binary and a.expected_sha256 and a.output
X=a.binary.resolve();O=a.output.resolve();assert X.is_relative_to(R)and O.is_relative_to(R)and not O.exists();assert sha(X)==a.expected_sha256
rows=[];hits=[];env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1')
class Handler(http.server.BaseHTTPRequestHandler):
 protocol_version='HTTP/1.1'
 def log_message(self,*args):pass
 def do_HEAD(self):self.respond(True)
 def do_GET(self):self.respond(False)
 def do_POST(self):self.respond(False)
 def respond(self,head):
  body=self.rfile.read(int(self.headers.get('Content-Length','0')));hits.append({'path':self.path,'method':self.command,'body_hex':body.hex(),'duplicates':self.headers.get_all('X-Test')})
  if self.path=='/slow':time.sleep(.3)
  status=404 if self.path=='/missing'else 302 if self.path=='/redirect'else 200
  content=b'\xff'if self.path=='/invalid'else body if self.path=='/echo'else json.dumps({'message':'雪'},ensure_ascii=False,separators=(',',':')).encode('utf8')
  try:
   self.send_response(status);self.send_header('Content-Length',str(len(content)));self.send_header('X-Repeat','one');self.send_header('X-Repeat','two')
   if self.path=='/redirect':self.send_header('Location','/target')
   self.end_headers()
   if not head:self.wfile.write(content)
  except (BrokenPipeError,ConnectionResetError,ConnectionAbortedError):pass
S=http.server.ThreadingHTTPServer(('127.0.0.1',0),Handler);S.daemon_threads=False;T=threading.Thread(target=S.serve_forever,kwargs={'poll_interval':.02});T.start();base='http://127.0.0.1:'+str(S.server_port);q=lambda s:json.dumps(s,ensure_ascii=False)
def call(path='/data',method='GET',headers='[]',body='""',timeout=1000,limit=16384):return 'httpRequest('+q(method)+','+q(base+path)+','+headers+','+body+','+str(timeout)+','+str(limit)+')'
def run(name,args,code=0,stdout=None,error=None,message=None):
 assert sha(X)==a.expected_sha256
 p=subprocess.run([str(X),*args],cwd=R,env=env,capture_output=True,timeout=15);r={'name':name,'args':args,'exit':p.returncode,'stdout':p.stdout.decode('utf8'),'stderr':p.stderr.decode('utf8')};rows.append(r);O.parent.mkdir(parents=True,exist_ok=True);O.with_suffix('.partial.json').write_text(json.dumps({'binary_sha256':a.expected_sha256,'rows':rows,'hits':hits},ensure_ascii=False,indent=2)+'\n',encoding='utf8');assert p.returncode==code,r
 if error:
  j=json.loads(p.stdout);assert j['error']==error and not p.stderr,r
  if message:assert j['message']==message,r
 elif message:assert not p.stdout and r['stderr'].splitlines()[0]=='lfz: '+message,r
 elif stdout is not None:assert r['stdout'].replace('\r\n','\n')==stdout and not p.stderr,r
 r['passed']=True

def expr(name,src,allow=True,**kw):run(name,['--json']+(['--allow-http']if allow else[])+['-e',src],**kw)
try:
 run('tutorial_check',['check','--json',str(P)],stdout='{"ok":true,"mode":"check","executed":false}\n');assert not hits
 run('tutorial_run',['--allow-http',str(P),'--',base+'/data'],stdout='雪\n')
 count=len(hits);expr('default_disabled',call(),allow=False,code=2,error='ValueError',message='HTTP 能力未启用');assert len(hits)==count
 run('process_not_http',['--json','--allow-process','-e',call()],code=2,error='ValueError',message='HTTP 能力未启用')
 expr('headers_duplicate_response','let r='+call('/missing')+'\nassert(r.status==404)\nassert(len(filter((p)=>p[0]=="x-repeat",r.headers))==2)\nassert(keys(r)==["body","headers","status"])',stdout='{"ok":true}\n')
 expr('explicit_post','let r='+call('/echo',method='POST',body=q('中文'))+'\nassert(r.body=="中文")',stdout='{"ok":true}\n')
 expr('request_duplicate_pairs','let r='+call(headers='[["X-Test","a"],["X-Test","b"]]')+'\nassert(r.status==200)',stdout='{"ok":true}\n');assert hits[-1]['duplicates']==['a','b']
 expr('no_redirect','assert('+call('/redirect')+'.status==302)',stdout='{"ok":true}\n');assert not any(h['path']=='/target'for h in hits)
 expr('strict_utf8',call('/invalid'),code=2,error='ValueError',message='HTTP 响应 body 不是合法 UTF-8')
 expr('headers_count_even_head',call(method='HEAD',limit=0),code=2,error='ValueError',message='HTTP 响应超过字节上限')
 expr('local_timeout_attempt','let r=attempt(()=>'+call('/slow',timeout=10)+')\nassert(!r.ok && r.error.message=="HTTP 请求时间上限已到")',stdout='{"ok":true}\n')
 run('global_deadline',['--json','--allow-http','--timeout-ms','30','-e','attempt(()=>'+call('/slow')+')'],code=2,error='ExecutionLimitError',message='执行时间上限已到')
 count=len(hits);expr('bad_method_no_connection',call(method='get'),code=2,error='ValueError',message='HTTP method 不受支持');assert len(hits)==count
 expr('reserved_header',call(headers='[["Host","bad"]]'),code=2,error='ValueError',message='HTTP headers 非法或超出上限')
 expr('get_body',call(body=q('data')),code=2,error='ValueError',message='HTTP body 非法或超出上限')
 expr('next_session_disabled',call(),allow=False,code=2,error='ValueError',message='HTTP 能力未启用')
 run('argv_protection',['-e','assert(argv()==["--allow-http","--allow-http=1"])','--','--allow-http','--allow-http=1'],stdout='')
 for suffix in ['','1']:run('equals_'+suffix,['--allow-http='+suffix,'-e','1'],code=2,message="选项 '--allow-http' 不接受值")
 run('duplicate',['--allow-http','--allow-http','-e','1'],code=2,message="选项 '--allow-http' 不能重复")
 run('check_reject',['check','--allow-http',str(P)],code=2,message='HTTP 能力选项只适用于运行命令')
 assert sha(X)==a.expected_sha256 and before=={str(p.relative_to(R)):sha(p)for p in G}
finally:
 S.shutdown();S.server_close();T.join(timeout=2);O.parent.mkdir(parents=True,exist_ok=True);O.with_suffix('.server.json').write_text(json.dumps({'server_stopped':not T.is_alive(),'port':S.server_port,'hits':hits},ensure_ascii=False,indent=2)+'\n',encoding='utf8');assert not T.is_alive()
result={'date':'2026-10-31','timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'binary_sha256':a.expected_sha256,'binary_bytes':X.stat().st_size,'guides':before,'passed':len(rows),'total':len(rows),'rows':rows,'hits':hits,'server_stopped':not T.is_alive(),'scope':'stdlib loopback HTTP only; no exact-N wire assumption, trusted TLS success, host Rust API compilation or cleanup fault injection claimed'};O.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n');print(json.dumps({'passed':len(rows),'total':len(rows)}))
