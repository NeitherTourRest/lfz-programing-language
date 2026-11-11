"""学生自调包装器（不是无旁路的OS权限边界）；配置由调度者提供。"""
import argparse,datetime,hashlib,json,subprocess
from pathlib import Path
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 a=argparse.ArgumentParser();a.add_argument('--config',type=Path,required=True);a.add_argument('--argv-json',default='[]');a.add_argument('--allow-http',action='store_true');a.add_argument('--json',action='store_true');a.add_argument('--max-steps',default='200000');v=a.parse_args();c=json.loads(v.config.read_text(encoding='utf8'));w=Path(c['workspace']).resolve();x=Path(c['binary']).resolve();log=w/'selftest-audit';log.mkdir(exist_ok=True)
 assert not v.allow_http or c['task']=='C04','HTTP self-test permission is limited to C04'
 assert sha(x)==c['binary_sha256'];now=datetime.datetime.now(datetime.timezone.utc);deadline=datetime.datetime.fromisoformat(c['deadline_utc'].replace('Z','+00:00'));assert now<deadline
 prior=list(log.glob('cycle-*.json'));assert len(prior)<5
 n=len(prior)+1;out=log/f'cycle-{n}.json';assert not out.exists();argv=json.loads(v.argv_json);assert isinstance(argv,list)and all(isinstance(s,str)for s in argv)
 snap=log/f'cycle-{n}';snap.mkdir();sources={}
 for p in w.rglob('*.lfz'):
  if log in p.parents:continue
  rel=p.relative_to(w);dst=snap/rel;dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes(p.read_bytes());sources[str(rel)]=sha(p)
 row=dict(cycle=n,start_utc=now.isoformat(),source_hashes=sources,runs=[])
 def save():out.write_text(json.dumps(row,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 save()
 commands=[['check',str(w/'main.lfz')],['run',*(['--allow-http']if v.allow_http else[]),*(['--json']if v.json else[]),'--max-steps',v.max_steps,'--timeout-ms','5000',str(w/'main.lfz'),'--',*argv]]
 for args in commands:
  assert sha(x)==c['binary_sha256'];remaining=(deadline-datetime.datetime.now(datetime.timezone.utc)).total_seconds();assert remaining>0
  try:
   p=subprocess.run([str(x),*args],cwd=w,capture_output=True,timeout=min(15,remaining));r=dict(args=args,exit=p.returncode,stdout=p.stdout.decode('utf8'),stderr=p.stderr.decode('utf8'))
  except Exception as ex:r=dict(args=args,host_exception=repr(ex),exit=None)
  row['runs'].append(r);save();print(json.dumps(r,ensure_ascii=False))
  if r['exit']!=0:break
 row['end_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();save()
if __name__=='__main__':main()
