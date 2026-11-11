"""Replay real LFZ REPL sessions through pipes, never script run."""
import argparse,ast,hashlib,json,os,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--prepare-only',action='store_true');p.add_argument('--binary',type=Path);p.add_argument('--expected-sha');p.add_argument('--output',type=Path);a=p.parse_args()
 d=ROOT/'app/repl_jsondesk';cases=json.loads((d/'cases.json').read_text(encoding='utf8'))
 assert len(cases)==6
 if a.prepare_only:
  print(json.dumps(dict(prepared=True,cases=len(cases),interpreter_processes=0,fixture_sha256=sha(d/'session.repl'))));return
 assert a.binary and a.expected_sha and a.output
 exe=a.binary.resolve();out=a.output.resolve();assert exe.is_relative_to(ROOT) and out.is_relative_to(ROOT) and not out.exists() and sha(exe)==a.expected_sha
 w=ROOT/'Temp/T20-app';w.mkdir(parents=True,exist_ok=True);out.parent.mkdir(parents=True,exist_ok=True)
 rows=[];result=dict(binary_sha256=sha(exe),binary_bytes=exe.stat().st_size,cases_sha256=sha(d/'cases.json'),status='running',rows=rows)
 def save():out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 try:
  for c in cases:
   assert sha(exe)==a.expected_sha
   text=(d/c['fixture']).read_text(encoding='utf8') if 'fixture' in c else c['text']
   (w/(c['name']+'.repl')).write_text(text,encoding='utf8')
   cmd=[str(exe),'repl',*c.get('flags',[])]
   try:r=subprocess.run(cmd,input=text.encode(),capture_output=True,cwd=ROOT,timeout=15,env=dict(os.environ,TEMP=str(w),TMP=str(w)),creationflags=0x08000000 if os.name=='nt' else 0)
   except subprocess.TimeoutExpired as e:
    rows.append(dict(name=c['name'],watchdog=True,stdout_hex=(e.stdout or b'').hex(),stderr_hex=(e.stderr or b'').hex(),passed=False));save();raise
   row=dict(name=c['name'],argv=cmd,input=text,exit=r.returncode,stdout=r.stdout.decode(),stderr=r.stderr.decode(),passed=False);rows.append(row);save()
   assert r.returncode==c['code'] and row['stdout'].splitlines()==c['stdout'],row
   assert all(s in row['stderr'] for s in c['stderr']),row
   assert 'lfz> ' not in row['stderr'] and '...> ' not in row['stderr'],row
   if c['code']==0:assert not row['stderr'],row
   if c['name']=='timeout_survives_reset':assert row['stderr'].count('执行时间上限已到')==2
   row['passed']=True;save()
  result['status']='completed'
 except BaseException as e:result.update(status='failed',error=repr(e));raise
 finally:result.update(total=len(rows),passed=sum(x['passed'] for x in rows));save()
 print(json.dumps(dict(status=result['status'],total=result['total'],passed=result['passed'])))
if __name__=='__main__':main()
