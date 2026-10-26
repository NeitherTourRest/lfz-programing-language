"""Read-only system CPU probe plus old same-path noop readiness check."""
import ctypes,ctypes.wintypes,datetime,hashlib,json,pathlib,shutil,statistics,subprocess,time,argparse,os
ROOT=pathlib.Path(__file__).resolve().parents[2]
ap=argparse.ArgumentParser(); ap.add_argument('--output',type=pathlib.Path,required=True); ap.add_argument('--settle-seconds',type=float,default=0); args=ap.parse_args()
out=args.output.resolve(); assert out.is_relative_to(ROOT) and not out.exists()
old=ROOT/'Temp/t12-perf/lfz-before.exe'; subject=ROOT/'Temp/T12/perf-subject.exe'; shutil.copyfile(old,subject); time.sleep(args.settle_seconds)
noop=ROOT/'Temp/t12-perf/noop.lfz'; noop.write_text('#42\nprint("RESULT ok=1")\n',encoding='utf-8')
def cpu():
 vals=[ctypes.wintypes.FILETIME() for _ in range(3)]
 assert ctypes.windll.kernel32.GetSystemTimes(*[ctypes.byref(v) for v in vals])
 return [(v.dwHighDateTime<<32)+v.dwLowDateTime for v in vals]
a=cpu(); time.sleep(.5); b=cpu(); delta=[y-x for x,y in zip(a,b)]; used=100*(1-delta[0]/(delta[1]+delta[2]))
env=os.environ.copy(); env.update(TEMP=str(ROOT/'Temp/T12'),TMP=str(ROOT/'Temp/T12'))
vals=[]
for i in range(9):
 assert hashlib.sha256(subject.read_bytes()).hexdigest()=='149b184e442ffbf250590431fa0aeffab401eac526faf60b4b435f620725d5ab'
 st=time.perf_counter_ns(); p=subprocess.run([str(subject),'run',str(noop)],input=b'',capture_output=True,timeout=15,cwd=ROOT,env=env); ms=(time.perf_counter_ns()-st)/1e6
 assert p.returncode==0 and p.stdout.decode().replace('\r\n','\n')=='RESULT ok=1\n'
 if i>=2:vals.append(ms)
result={'timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'system_cpu_percent_over_500ms':used,'old_sha256':hashlib.sha256(subject.read_bytes()).hexdigest(),'old_noop_same_subject_median_ms':statistics.median(vals),'old_noop_ms':vals,'original_noop_limit_ms':16.3071,'settle_after_copy_seconds':args.settle_seconds,'ready':used<30 and statistics.median(vals)<=16.3071}
out.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8'); print(result)
raise SystemExit(0 if result['ready'] else 1)