from pathlib import Path
import json, subprocess, time, statistics, os, hashlib, argparse
root=Path.cwd();ap=argparse.ArgumentParser();ap.add_argument('output');ap.add_argument('binaries',nargs='+');o=ap.parse_args()
env=os.environ.copy();env['TEMP']=env['TMP']=str(root/'Temp/runtime-t12')
rows=[]
for warm in range(2):
 for binary in o.binaries:subprocess.run([binary,'run','benchmarks/lfz/numeric_loop.lfz'],input=b'500000\n',stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=True,env=env)
for round in range(7):
 for binary in (o.binaries if round%2==0 else o.binaries[::-1]):
  for n in [100000,500000]:
   start=time.perf_counter_ns();r=subprocess.run([binary,'run','benchmarks/lfz/numeric_loop.lfz'],input=f'{n}\n'.encode(),stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=True,env=env);ms=(time.perf_counter_ns()-start)/1e6
   assert r.stdout.decode().strip()==f'RESULT total={n*(n+1)//2}'
   rows.append(dict(round=round,binary=binary,n=n,ms=ms,exit=r.returncode,stdout=r.stdout.decode(),stderr=r.stderr.decode()))
result={'runs':rows,'binaries':{b:hashlib.sha256(Path(b).read_bytes()).hexdigest() for b in o.binaries},'medians':[]}
for binary in o.binaries:
 for n in [100000,500000]:
  row={'binary':binary,'n':n,'median_ms':statistics.median(r['ms'] for r in rows if r['binary']==binary and r['n']==n)};result['medians'].append(row);print(row)
p=Path(o.output);assert not p.exists();p.write_text(json.dumps(result,indent=2),encoding='utf8')
