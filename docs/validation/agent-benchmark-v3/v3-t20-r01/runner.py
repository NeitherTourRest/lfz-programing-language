"""准备评分器：不派学生。正式评分须另由root GO/冻结；可供verifier控制验证。"""
import argparse,hashlib,http.server,json,os,shutil,subprocess,threading,datetime
from pathlib import Path
from oracles import jsonl,jobs,quote,http as expected_http
HERE=Path(__file__).resolve().parent
ROOT=next(p for p in HERE.parents if (p/'AGENTS.md').exists())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def inside(p):
 p=p.resolve()
 if not p.is_relative_to(ROOT):raise ValueError('project paths only')
 return p
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--task',choices=['C01','C02','C03','C04'],required=True);ap.add_argument('--submission',type=Path,required=True);ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--sha256',required=True);ap.add_argument('--output',type=Path,required=True);ap.add_argument('--source-review',type=Path);ap.add_argument('--protocol-review',type=Path);a=ap.parse_args()
 exe=inside(a.binary);source=inside(a.submission);out=inside(a.output);assert sha(exe)==a.sha256.lower()and not out.exists()
 work=ROOT/'Temp/agent-benchmark-v3/grading'/out.stem;assert not work.exists();work.mkdir(parents=True)
 for p in source.rglob('*.lfz'):
  dst=work/p.relative_to(source);dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dst)
 sources={str(p.relative_to(work)):sha(p)for p in work.rglob('*.lfz')};rows=[]
 evidence=dict(task=a.task,mode='CONTROL_OR_SEALED_SUBMISSION_VALIDATION',binary_sha256=sha(exe),binary_bytes=exe.stat().st_size,source_hashes=sources,runs=rows,source_axis='UNKNOWN',protocol_axis='UNKNOWN',server_closed=None)
 def save():out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(evidence,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 def run(name,args,expect=None,error=None,check=False):
  assert sha(exe)==a.sha256.lower();row=dict(name=name,args=args);rows.append(row);save()
  try:
   p=subprocess.run([str(exe),*args],cwd=work,capture_output=True,timeout=15)
   s=p.stdout.decode('utf8');e=p.stderr.decode('utf8');row.update(exit=p.returncode,stdout=s,stderr=e)
   if check:ok=p.returncode==0 and not s and not e
   elif error:ok=p.returncode==2 and json.loads(s)['error']==error
   else:ok=p.returncode==0 and not e and json.loads(s)==expect
   row['pass']=ok
  except Exception as ex:row.update(pass_=False,exception=repr(ex));row['pass']=False
  save();return row
 marker=work/'marker.txt';entry=work/'main.lfz'
 # C02 check must load complete graph but not execute marker write.
 run('check',['check',str(entry)],check=True);evidence['check_marker_absent']=not marker.exists()
 def execute(n,argv,expected=None,error=None,flags=()):return run(n,['run',*flags,*(['--json']if error else[]),'--max-steps','200000','--timeout-ms','5000',str(entry),'--',*argv],expected,error)
 if a.task=='C01':
  datasets=[b'',b'{"account":"\xe9\x9b\xaa","delta":2}\r\nbad\n\n{"account":"x","delta":-3}',b'null\ntrue\n{"account":"","delta":1}\n{"account":"a","delta":true}\n',b'\xff',b'a'*4097,b' '*4065+b'{"account":"a","delta":1}\r\n']
  datasets += [(''.join(json.dumps({'account':'a','delta':i-20})+'\n'for i in range(n))).encode()for n in [33,65]]
  exact=b'{"account":"cap","delta":1}'
  datasets += [b' '*(4096-len(exact)-1)+exact+b'\r\n',b' '*(4097-len(exact)-1)+exact+b'\r\n']
  prefix=b'{"account":"keep","delta":1}\n'*33
  datasets += [prefix+b'\xff',prefix+b'a'*4097]
  for i,data in enumerate(datasets):
   inp=work/'input.jsonl';dest=work/'out.jsonl';inp.write_bytes(data);dest.write_text('stale',encoding='utf8');v=jsonl(data)
   row=execute('data'+str(i),[str(inp),str(dest)],v.get('summary'),v.get('error'))
   if 'rows'in v:
    try:row['output_rows']=[json.loads(x)for x in dest.read_text(encoding='utf8').splitlines()];row['pass'] &= row['output_rows']==v['rows']
    except Exception as ex:row.update(output_error=repr(ex));row['pass']=False
   if 'committed'in v:
    try:row['committed_rows']=[json.loads(x)for x in dest.read_text(encoding='utf8').splitlines()];row['pass'] &= row['committed_rows']==v['committed']
    except Exception as ex:row.update(output_error=repr(ex));row['pass']=False
   save()
 elif a.task=='C02':
  for q in [0,-4,13]:
   if marker.exists():marker.unlink()
   args=[str(marker),str(q),'--allow-http=数据'];row=execute('quantity'+str(q),args,quote(q,args));row['marker']=marker.read_text()if marker.exists()else None;row['pass'] &= row['marker']=='I';save()
  if marker.exists():marker.unlink()
  row=execute('fault',[str(marker),'fault','中文'],error='AssertionError')
  try:row['pass'] &= Path(json.loads(row['stdout'])['file']).name=='rates.lfz'
  except Exception:row['pass']=False
 elif a.task=='C03':
  for i,v in enumerate([[],[1,None,True,'x',-4],['bad','bad',3],[-9223372036854775808,9223372036854775807]]):execute('jobs'+str(i),['normal',json.dumps(v)],jobs(v))
  run('budget',['run','--json','--max-steps','100',str(entry),'--','budget','[]'],error='ExecutionLimitError')
 else:
  visits=[]
  class Handler(http.server.BaseHTTPRequestHandler):
   def log_message(self,*args):pass
   def do_GET(self):
    visits.append(self.path);status={'/missing':404,'/redirect':302}.get(self.path,200);body={'/ok':'雪'.encode(),'/missing':b'absent','/redirect':b'','/bad':b'\xff'}.get(self.path,b'target')
    self.send_response(status);self.send_header('Content-Length',str(len(body)))
    if self.path=='/ok':self.send_header('X-Repeat','one');self.send_header('X-Repeat','two')
    if self.path=='/redirect':self.send_header('Location','/target')
    self.end_headers();self.wfile.write(body)
  server=http.server.ThreadingHTTPServer(('127.0.0.1',0),Handler);thread=threading.Thread(target=server.serve_forever);thread.start();base='http://127.0.0.1:'+str(server.server_port)
  try:
   row=execute('disabled',[base],expected_http(False));row['pass'] &= not visits
   row=execute('enabled',[base],expected_http(True),flags=['--allow-http']);row['visits']=visits[:];row['pass'] &= visits==['/ok','/missing','/redirect','/bad','/ok']
  finally:server.shutdown();server.server_close();thread.join(5);evidence['server_closed']=not thread.is_alive();save()
 evidence['functional_axis']='PASS'if rows and all(r['pass']for r in rows)and evidence['check_marker_absent']and (a.task!='C04'or evidence['server_closed']is True)else'FAIL'
 # Review records must bind exact source hash set and be signed by independent reviewer.
 for key,p in [('source_axis',a.source_review),('protocol_axis',a.protocol_review)]:
  if p:
   v=json.loads(inside(p).read_text(encoding='utf8'));assert v['source_hashes']==sources and v['reviewer'] and v['task']==a.task
   assert v['verdict']in ['PASS','FAIL','UNKNOWN'];evidence[key]=v['verdict'];evidence[key+'_record_sha256']=sha(p)
 evidence['certified']='FAIL'if 'FAIL'in [evidence[k]for k in ['functional_axis','source_axis','protocol_axis']]else ('PASS'if all(evidence[k]=='PASS'for k in ['functional_axis','source_axis','protocol_axis'])else'UNKNOWN');save();print(evidence['functional_axis'],evidence['certified'])
if __name__=='__main__':main()
