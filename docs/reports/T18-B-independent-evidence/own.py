import runpy,subprocess,json,os,hashlib
from pathlib import Path
R=Path.cwd();E=R/'docs/reports/T18-B-independent-evidence';W=R/'Temp/T18-B-verifier/own';W.mkdir(parents=True,exist_ok=True);X=R/'target/release/lfz.exe';Loopback=runpy.run_path(str(E/'public49.py'))['Loopback'];rows=[];Q=lambda x:json.dumps(x,ensure_ascii=False)
def one(name,expression,expect=None,flags=[],allow=True,file=None):
 s=Loopback();s.start()
 try:
  src=expression(s.base);args=[str(X),'--json']+(['--allow-http'] if allow else [])+flags
  if file:
   lib=W/'lib.lfz';lib.write_text('#42\n'+src+'\n',encoding='utf8');entry=W/'main.lfz';entry.write_text('#42\nimport m from "./lib.lfz"\n',encoding='utf8');args+=[str(entry)]
  else:args+=['-e',src]
  env=dict(os.environ,HTTP_PROXY='http://127.0.0.1:1',HTTPS_PROXY='http://127.0.0.1:1',TEMP=str(W),TMP=str(W));p=subprocess.run(args,capture_output=True,timeout=8,env=env);d=json.loads(p.stdout);rec=s.snapshot();row=dict(name=name,args=args,exit=p.returncode,stdout=p.stdout.decode('utf8'),stderr=p.stderr.decode('utf8'),requests=rec);rows.append(row);assert p.returncode==(2 if expect else 0),row;assert not p.stderr,row
  if expect:assert d['message']==expect,row
  if file:assert Path(d['file']).name=='lib.lfz',row
  row['passed']=True;return rec
 finally:s.close();assert not s.worker.is_alive()
def req(base,path='/',method='GET',body='',ms=500,cap=4096):return 'httpRequest('+','.join([Q(method),Q(base+path),'[]',Q(body),str(ms),str(cap)])+')'
one('404_value',lambda b:'let r='+req(b,'/404')+'\nassert(r.status==404 && r.body=="ok")')
one('duplicates',lambda b:'let r='+req(b,'/duplicate')+'\nassert(len(filter((x)=>x[0]=="x-dup",r.headers))==2)')
r=one('redirect_not_followed',lambda b:'assert('+req(b,'/redirect')+'.status==302)');assert len(r)==1 and r[0]['path']=='/redirect'
r=one('cookie_no_session_pool',lambda b:req(b,'/cookie')+'\n'+req(b,'/'));assert len(r)==2 and all(k.lower()!='cookie' for k,v in r[1]['headers'])
r=one('POST_no_retry_direct_proxy',lambda b:'let r='+req(b,'/echo?x=%E9%9B%AA','POST','雪')+'\nassert(r.body=="雪")');assert len(r)==1 and r[0]['body_hex']=='e99baa'
one('exact_API_N61',lambda b:req(b,cap=61))
one('API_N60',lambda b:req(b,cap=60),'HTTP 响应超过字节上限')
one('local_attempt',lambda b:'let r=attempt(()=>'+req(b,'/slowbody',ms=30)+')\nassert(!r.ok && r.error.message=="HTTP 请求时间上限已到")')
one('drip_total_deadline',lambda b:'attempt(()=>'+req(b,'/drip',ms=2000)+')','执行时间上限已到',['--timeout-ms','50'])
one('module_fault_source',lambda b:req(b,'/slowheaders',ms=30),'HTTP 请求时间上限已到',file=True)
r=one('separate_process_disabled',lambda b:req(b),'HTTP 能力未启用',allow=False);assert not r
(E/'own.json').write_text(json.dumps(dict(total=len(rows),passed=len(rows),fixture_reuse='public49 bounded loopback; independent request/oracle script',rows=rows),ensure_ascii=False,indent=2),encoding='utf8');print(len(rows),'independent HTTP witnesses PASS')
