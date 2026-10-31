import argparse,ast,datetime,hashlib,http.server,json,os,re,socket,subprocess,threading,time
from pathlib import Path
R=Path.cwd();a=argparse.ArgumentParser();a.add_argument('--prepare-only',action='store_true');a.add_argument('--binary',type=Path);a.add_argument('--expected-sha256');a.add_argument('--output',type=Path);o=a.parse_args();W=R/'Temp/T18-B-ai';W.mkdir(parents=True,exist_ok=True)
paths=['.opencode/skills/lfz-programming/'+n for n in ['SKILL.md','README.md','VERIFICATION.md','prompt-template.md']]+['docs/guide/ai/README.md'];sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();docs={n:sha(R/n)for n in paths}
examples=[]
for i,n in enumerate([paths[0],paths[-1]]):
 t=(R/n).read_text(encoding='utf8').split('## 6.7 T18-B：')[1];code=re.search(r'```lfz\n(.*?)\n```',t,re.S)[1];p=W/('skill.lfz'if i==0 else'ai.lfz');p.write_text(code+'\n',encoding='utf8',newline='\n');examples.append(p)
if o.prepare_only:print('prepared two extracted LFZ examples; interpreter/network launches 0');raise SystemExit
assert o.binary and o.expected_sha256 and o.output
X=o.binary.resolve();O=o.output.resolve();H=o.expected_sha256.lower();assert X.is_relative_to(R) and O.is_relative_to(R) and not O.exists() and sha(X)==H
closed=False;rows=[];visits=[];lock=threading.Lock();env=os.environ.copy();env.update(TEMP=str(W),TMP=str(W),PYTHONDONTWRITEBYTECODE='1')
class Handler(http.server.BaseHTTPRequestHandler):
 protocol_version='HTTP/1.0'
 def setup(self):super().setup();self.connection.settimeout(2)
 def log_message(self,*args):pass
 def do_GET(self):self.handle_request()
 def do_POST(self):self.handle_request()
 def do_HEAD(self):self.handle_request()
 def handle_request(self):
  with lock:visits.append({'method':self.command,'path':self.path})
  body=self.rfile.read(min(int(self.headers.get('Content-Length','0')),1048576))
  status=200;extra=[]
  if self.path=='/slow':time.sleep(.2);data=b'slow'
  elif self.path=='/missing':status=404;data=b'absent'
  elif self.path=='/redirect':status=302;extra=[('Location','/target')];data=b''
  elif self.path=='/invalid':data=b'\xff'
  elif self.path=='/headers':data=b'ok';extra=[('X-Repeat','one'),('X-Repeat','two')]
  elif self.path=='/echo':data=body
  else:data=json.dumps({'message':'雪','path':self.path},ensure_ascii=False,separators=(',',':')).encode('utf8')
  try:
   self.send_response(status);self.send_header('Content-Length',str(len(data)))
   for k,v in extra:self.send_header(k,v)
   self.end_headers()
   if self.command!='HEAD':self.wfile.write(data);self.wfile.flush()
  except (BrokenPipeError,ConnectionResetError,OSError):pass
class Server(http.server.ThreadingHTTPServer):daemon_threads=False;block_on_close=True
server=Server(('127.0.0.1',0),Handler);thread=threading.Thread(target=server.serve_forever,kwargs={'poll_interval':.01});thread.start();base='http://127.0.0.1:'+str(server.server_port)
def save():O.parent.mkdir(parents=True,exist_ok=True);O.write_text(json.dumps(dict(date='2026-10-31',timestamp_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),binary=str(X.relative_to(R)),binary_sha256=H,document_hashes=docs,checks_total=len(rows),checks_passed=sum(r['passed']for r in rows),rows=rows,server_visits=visits,server_closed=closed),ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
def run(n,args,expected='',error=None,message=None,usage=None,oracle=None,network=False):
 assert sha(X)==H
 before=len(visits);p=subprocess.run([str(X),*args],cwd=W,env=env,capture_output=True,timeout=15);s=p.stdout.decode('utf8');e=p.stderr.decode('utf8');ok=p.returncode==(2 if error or usage else 0)
 if usage:ok=ok and not s and usage in e
 elif error:
  try:v=json.loads(s);ok=ok and v['error']==error and not e and (message is None or v['message']==message)
  except Exception:ok=False
 elif oracle:
  try:ok=ok and not e and oracle(s)
  except Exception:ok=False
 else:ok=ok and s==expected and not e
 if not network:ok=ok and len(visits)==before
 rows.append(dict(name=n,args=args,exit=p.returncode,stdout=s,stderr=e,new_connections=len(visits)-before,passed=ok));save()
def call(path,method='GET',body='',headers=None,ms=1000,limit=16384):return 'httpRequest('+','.join([json.dumps(method),json.dumps(base+path),json.dumps(headers or []),json.dumps(body,ensure_ascii=False),str(ms),str(limit)])+')'
try:
 for i,p in enumerate(examples):
  run('extracted_check_'+str(i),['check','--json',str(p)],'{"ok":true,"mode":"check","executed":false}\n')
  def batch(s):
   v=json.loads(s);return len(v)==2 and v[0]['status']==200 and json.loads(v[0]['body'])['message']=='雪' and v[1]['status']==404
  run('extracted_batch_'+str(i),['--allow-http','--max-steps','1000','--timeout-ms','5000','run',str(p),'--',base],oracle=batch,network=True)
 c=call('/data');run('default_disabled',['--json','-e',c],error='ValueError',message='HTTP 能力未启用');run('process_does_not_grant_http',['--allow-process','--json','-e',c],error='ValueError',message='HTTP 能力未启用')
 run('enabled_three_fields',['--allow-http','-e','let r='+c+'\nassert(len(keys(r))==3 && r.status==200 && type(r.headers)=="array")\nassert(jsonParse(r.body).message=="雪")'],network=True)
 run('new_session_disabled',['--json','-e',c],error='ValueError',message='HTTP 能力未启用')
 run('repeated_headers',['--allow-http','-e','let r='+call('/headers')+'\nlet hs=filter((p)=>p[0]=="x-repeat",r.headers)\nassert(hs==[["x-repeat","one"],["x-repeat","two"]])'],network=True)
 run('post_utf8_once',['--allow-http','-e','assert('+call('/echo','POST','中文')+'.body=="中文")'],network=True)
 run('no_auto_redirect',['--allow-http','-e','assert('+call('/redirect')+'.status==302)'],network=True)
 rows[-1]['passed'] &= not any(v['path']=='/target'for v in visits);save()
 run('head_still_counts_headers',['--allow-http','--json','-e',call('/data',method='HEAD',limit=0)],error='ValueError',message='HTTP 响应超过字节上限',network=True)
 for n,c,m in [('invalid_utf8',call('/invalid'),'HTTP 响应 body 不是合法 UTF-8'),('local_timeout',call('/slow',ms=10),'HTTP 请求时间上限已到')]:run(n,['--allow-http','-e','let r=attempt(()=>'+c+')\nassert(!r.ok && r.error["class"]=="ValueError" && r.error.message=='+json.dumps(m,ensure_ascii=False)+')'],network=True)
 run('global_deadline_not_caught',['--allow-http','--timeout-ms','10','--json','-e','attempt(()=>'+call('/slow',ms=1000)+')'],error='ExecutionLimitError',message='执行时间上限已到',network=True)
 for n,c,m in [('method_before_capability',call('/data',method='get'),'HTTP method 不受支持'),('headers_before_capability',call('/data',headers=[['Host','bad']]),'HTTP headers 非法或超出上限')]:run(n,['--json','-e',c],error='ValueError',message=m)
 run('equals_rejected',['--allow-http=1','--help'],usage="选项 '--allow-http' 不接受值")
 run('duplicate_rejected',['--allow-http','--allow-http','-e',''],usage="选项 '--allow-http' 不能重复")
 run('check_rejects_http',['check','--allow-http','-e','1'],usage='HTTP 能力选项只适用于运行命令')
 run('mixed_first_http',['check','--allow-http','--allow-process','-e','1'],usage='HTTP 能力选项只适用于运行命令')
 run('mixed_first_process',['check','--allow-process','--allow-http','-e','1'],usage='子进程能力选项只适用于运行命令')
 run('scan_before_applicability',['check','--allow-http','--max-steps=1','-e','1'],usage="选项 '--max-steps' 不支持等号写法，请使用 '--max-steps N'")
 run('code_protection',['check','--json','-e','--allow-http'],'{"ok":true,"mode":"check","executed":false}\n')
 run('argv_protection',['-e','print(argv()[0])','--','--allow-http'],'--allow-http\n')
 assert docs=={n:sha(R/n)for n in paths};save()
finally:server.shutdown();server.server_close();thread.join(5)
assert not thread.is_alive();closed=True;save();print(json.dumps({'checks_total':len(rows),'checks_passed':sum(r['passed']for r in rows),'server_closed':True}));raise SystemExit(0 if all(r['passed']for r in rows)else 1)