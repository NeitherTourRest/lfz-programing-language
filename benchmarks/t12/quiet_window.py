"""One prespecified quiet/cooldown window; stops on failed readiness."""
import argparse,datetime,hashlib,json,pathlib,subprocess,sys,time
ROOT=pathlib.Path(__file__).resolve().parents[2]
ap=argparse.ArgumentParser(); ap.add_argument('--prefix',type=pathlib.Path,required=True); args=ap.parse_args()
prefix=args.prefix.resolve(); assert prefix.is_relative_to(ROOT)
outputs={k:pathlib.Path(str(prefix)+suffix) for k,suffix in [('window','.window.json'),('ready','.readiness.json'),('same','.same-path.json'),('new','.new-features.json')]}
assert all(not p.exists() for p in outputs.values())
expected='7641b1ba98551d8726918b320d18501d62e30faa5c8c5c08fe2dc789a4748d81'
assert hashlib.sha256((ROOT/'dist/lfz.exe').read_bytes()).hexdigest()==expected
record={'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'initial_quiet_seconds':90,'settle_after_each_copy_seconds':5,'affinity':'unbound; inherited normal system mask','expected_new_sha256':expected,'status':'quiet_wait'}
def save(): outputs['window'].write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
save()
for i in range(3):
 print('fixed quiet interval:',90-i*30,'seconds remaining',flush=True); time.sleep(30)
record['readiness_started_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat(); save()
p=subprocess.run([sys.executable,'-B',str(ROOT/'benchmarks/t12/readiness.py'),'--settle-seconds','5','--output',str(outputs['ready'])],cwd=ROOT)
if p.returncode:
 record.update(status='not-ready; no formal benchmark run',finished_utc=datetime.datetime.now(datetime.timezone.utc).isoformat()); save(); raise SystemExit(1)
record['status']='formal same-path measurement'; save()
p=subprocess.run([sys.executable,'-B',str(ROOT/'benchmarks/t12/final_compare.py'),'--new-binary',str(ROOT/'dist/lfz.exe'),'--expected-new-sha',expected,'--settle-seconds','5','--output',str(outputs['same'])],cwd=ROOT)
if p.returncode:
 record.update(status='formal measurement error',exit=p.returncode); save(); raise SystemExit(p.returncode)
record['status']='new capability measurement'; save()
p=subprocess.run([sys.executable,'-B',str(ROOT/'benchmarks/t12/new_features.py'),'--lfz',str(ROOT/'Temp/T12/perf-subject.exe'),'--output',str(outputs['new'])],cwd=ROOT)
record.update(status='completed' if p.returncode==0 else 'new capability measurement error',exit=p.returncode,finished_utc=datetime.datetime.now(datetime.timezone.utc).isoformat()); save()
raise SystemExit(p.returncode)