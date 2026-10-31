"""Loopback-only HTTP application oracle; no product compilation or public network."""
import argparse,hashlib,json,os,socket,subprocess,threading,time
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--lfz',type=Path);ap.add_argument('--expected-sha');ap.add_argument('--output',type=Path);ap.add_argument('--prepare-only',action='store_true');a=ap.parse_args()
    app=ROOT/'app/httpplan.lfz'
    if a.prepare_only:print(json.dumps({'prepared':True,'source_sha256':sha(app),'interpreter_processes':0,'server_started':False}));return
    assert a.lfz and a.expected_sha and a.output
    exe=a.lfz.resolve();out=a.output.resolve();expected=a.expected_sha.lower();assert exe.is_relative_to(ROOT) and out.is_relative_to(ROOT) and not out.exists() and sha(exe)==expected
    w=ROOT/'Temp/T18-B-app';w.mkdir(parents=True,exist_ok=True);out.parent.mkdir(parents=True,exist_ok=True)
    cfg=w/'plan.json';output=w/'report.json';env=dict(os.environ,TEMP=str(w),TMP=str(w),PYTHONDONTWRITEBYTECODE='1');calls=[];lock=threading.Lock();rows=[]
    class Handler(BaseHTTPRequestHandler):
        protocol_version='HTTP/1.1'
        def log_message(self,*args):pass
        def do_POST(self):self.handle_request()
        def do_GET(self):self.handle_request()
        def handle_request(self):
            self.connection.settimeout(2)
            data=self.rfile.read(int(self.headers.get('Content-Length','0')))
            with lock:calls.append({'method':self.command,'path':self.path,'body_hex':data.hex(),'tag':self.headers.get('X-Tag')})
            status=200;headers=[];body=b'ready'
            if self.path.startswith('/echo/'):
                body=json.dumps({'method':self.command,'path':self.path,'body':data.decode(),'tag':self.headers.get('X-Tag')},ensure_ascii=False,separators=(',',':')).encode();headers=[('X-Reply','one'),('X-Reply','two')]
            elif self.path=='/missing':status=404;body=b'missing'
            elif self.path=='/redirect':status=302;headers=[('Location','/target')];body=b'redirect'
            elif self.path=='/bad':body=b'\xff'
            elif self.path=='/slow':time.sleep(.4);body=b'slow'
            elif self.path=='/large':body=b'x'*200
            elif self.path=='/target':body=b'SHOULD_NOT_FOLLOW'
            try:
                self.send_response_only(status);self.send_header('Content-Length',str(len(body)))
                for k,v in headers:self.send_header(k,v)
                self.end_headers();self.wfile.write(body);self.wfile.flush()
            except (BrokenPipeError,ConnectionResetError,ConnectionAbortedError,OSError):pass
            self.close_connection=True
    server=ThreadingHTTPServer(('127.0.0.1',0),Handler);server.daemon_threads=False;server.block_on_close=True
    thread=threading.Thread(target=server.serve_forever,kwargs={'poll_interval':.02});thread.start();base='http://127.0.0.1:'+str(server.server_port)
    result={'date':'2026-10-31','binary_sha256':expected,'binary_bytes':exe.stat().st_size,'source_sha256':sha(app),'network_scope':base,'status':'running','rows':rows}
    def save():
        with lock:result['server_requests']=list(calls)
        out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
    def job(name,path,method='GET',headers=None,body='',timeout=5000,cap=4096):return dict(name=name,method=method,url=base+path,headers=headers or [],body=body,timeoutMs=timeout,maxResponseBytes=cap)
    def run(name,jobs,flags=(),code=0,machine=False):
        cfg.write_text(json.dumps(jobs,ensure_ascii=False),encoding='utf8');before=cfg.read_bytes();output.write_bytes(b'KEEP');assert sha(exe)==expected
        command=[str(exe),'run',*flags,*(['--json'] if machine else []),str(app),'--',str(cfg),str(output)]
        try:p=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,timeout=12,creationflags=0x08000000 if os.name=='nt' else 0)
        except subprocess.TimeoutExpired as error:
            rows.append({'name':name,'watchdog':True,'stdout_hex':(error.stdout or b'').hex(),'stderr_hex':(error.stderr or b'').hex(),'passed':False});save();raise
        r={'name':name,'arguments':command,'exit':p.returncode,'stdout':p.stdout.decode(),'stderr':p.stderr.decode(),'output_hex':output.read_bytes().hex(),'passed':False};rows.append(r);save();assert p.returncode==code and cfg.read_bytes()==before,r;return p,r
    def done(r):r['passed']=True;save()
    try:
        for name,flags in [('disabled',[]),('process_capability_does_not_grant_http',['--allow-process'])]:
            with lock:n=len(calls)
            p,r=run(name,[job('no_connect','/target')],flags,code=2,machine=True)
            assert output.read_bytes()==b'KEEP' and json.loads(p.stdout)['error']=='AssertionError' and '需要 --allow-http' in p.stdout.decode()
            with lock:assert len(calls)==n
            done(r)
        jobs=[job('ok','/ok'),job('post','/echo/%E4%B8%AD?q=one%20two',method='POST',headers=[['X-Tag',' demo ']],body='数据🙂'),job('404','/missing'),job('302','/redirect'),job('utf8_error','/bad'),job('timeout','/slow',timeout=20),job('limit','/large',cap=1),job('after','/after')]
        stable=None
        for suffix in ['first','repeat']:
            p,r=run('batch_'+suffix,jobs,['--allow-http']);assert not p.stderr
            doc=json.loads(output.read_bytes());assert (doc['total'],doc['succeeded'],doc['failed'])==(8,5,3) and len(doc['rows'])==8
            for i,status in [(0,200),(1,200),(2,404),(3,302),(7,200)]:
                item=doc['rows'][i];assert item['ok'] and item['error'] is None and item['value']['status']==status
            echo=json.loads(doc['rows'][1]['value']['body']);assert echo==dict(method='POST',path='/echo/%E4%B8%AD?q=one%20two',body='数据🙂',tag='demo')
            assert [pair for pair in doc['rows'][1]['value']['headers'] if pair[0]=='x-reply']==[['x-reply','one'],['x-reply','two']]
            for i,msg in [(4,'HTTP 响应 body 不是合法 UTF-8'),(5,'HTTP 请求时间上限已到'),(6,'HTTP 响应超过字节上限')]:
                item=doc['rows'][i];assert not item['ok'] and item['value'] is None and item['error']['class']=='ValueError' and item['error']['message']==msg
            assert doc['rows'][2]['value']['body']=='missing' and doc['rows'][3]['value']['body']=='redirect' and doc['rows'][7]['value']['body']=='ready'
            if stable is None:stable=output.read_bytes()
            else:assert stable==output.read_bytes(),'unexpected nondeterminism'
            with lock:assert not any(x['path']=='/target' for x in calls)
            assert json.loads(p.stdout)==dict(total=8,succeeded=5,failed=3,path=str(output));r['report']=doc;done(r)
        p,r=run('global_deadline_not_caught',[job('slow','/slow'),job('after','/target')],['--allow-http','--timeout-ms','40'],code=2,machine=True)
        o=json.loads(p.stdout);line=next(i for i,s in enumerate(app.read_text(encoding='utf8').splitlines(),1) if 'httpRequest(job.method' in s)
        assert not p.stderr and o['error']=='ExecutionLimitError' and o['message']=='执行时间上限已到' and o['line']==line and o['traceback'][-1]['func']=='<fn>'
        assert output.read_bytes()==b'KEEP'
        with lock:assert not any(x['path']=='/target' for x in calls)
        done(r)
        with lock:n=len(calls)
        p,r=run('prevalidate_before_network',[job('must_not_send','/target'),{'name':'bad'}],['--allow-http'],code=2,machine=True)
        assert json.loads(p.stdout)['error']=='AssertionError' and output.read_bytes()==b'KEEP'
        with lock:assert len(calls)==n
        done(r)
        p,r=run('empty_plan',[],['--allow-http']);assert json.loads(output.read_bytes())==dict(total=0,succeeded=0,failed=0,rows=[]);done(r)
        assert sha(exe)==expected and sha(app)==result['source_sha256'];result.update(status='completed',total=len(rows),passed=sum(r['passed'] for r in rows));save()
    except BaseException as error:result.update(status='failed',error=repr(error),total=len(rows),passed=sum(r['passed'] for r in rows));save();raise
    finally:
        server.shutdown();server.server_close();thread.join(timeout=3);assert not thread.is_alive(),'server did not stop'
        result['server_shutdown_complete']=True;save()
    print(json.dumps({k:v for k,v in result.items() if k not in ('rows','server_requests')}))
if __name__=='__main__':main()
