from pathlib import Path
import json,hashlib,time,subprocess,os,statistics,shutil,datetime,platform
root=Path.cwd();base=root/'Temp/runtime-t12/startup-candidates';method=json.loads((base/'method.json').read_text(encoding='utf8'))
inputs=[]
for f in ['before.json','before-small.json']:inputs+=json.loads((root/'benchmarks/t12/results'/f).read_text(encoding='utf8'))['cases']
lookup={(r['name'],r['n']):r for r in inputs}
limits={(r['name'],r['n']):r['limit_ms'] for r in json.loads((root/'benchmarks/t12/results/comparison.json').read_text(encoding='utf8'))['cases']}
variants={'old149b':root/'Temp/t12-perf/lfz-before.exe','current7641':root/'dist/lfz.exe','opt2':base/'opt2/target/release/lfz.exe','opts':base/'opts/target/release/lfz.exe'}
metadata={k:{'path':str(v.relative_to(root)),'sha256':hashlib.sha256(v.read_bytes()).hexdigest(),'bytes':v.stat().st_size} for k,v in variants.items()}
assert metadata['old149b']['sha256'].startswith('149b184e') and metadata['current7641']['sha256'].startswith('7641b1ba')
result={'method':method,'method_sha256':hashlib.sha256((base/'method.json').read_bytes()).hexdigest(),'binaries':metadata,'environment':{'platform':platform.platform(),'cpu':platform.processor(),'logical_cpus':os.cpu_count()},'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'cases':[]}
env=os.environ.copy();env['TEMP']=env['TMP']=str(base);dest=base/'subject.exe'
print('binaries',json.dumps(metadata),flush=True)
for case_index,(name,n) in enumerate(method['cases']):
 data=lookup[(name,n)];source=base/f'{name}-{n}.lfz';source.write_text(data['source'],encoding='utf8',newline='\n')
 expected=data['expected_stdout'];order=list(variants)
 if case_index%2:order.reverse()
 row={'name':name,'n':n,'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'source':data['source'],'stdin':data['stdin'],'expected_stdout':expected,'original_limit_ms':limits.get((name,n),max(data['median_ms']*1.2,data['median_ms']+5)),'variants':{}}
 for label in order:
  shutil.copyfile(variants[label],dest)
  assert hashlib.sha256(dest.read_bytes()).hexdigest()==metadata[label]['sha256']
  time.sleep(method['copy_settle_seconds'])
  all_runs=[]
  for i in range(method['warmup_per_variant_case']+method['formal_runs_per_variant_case']):
   start=time.perf_counter_ns();p=subprocess.run([str(dest),'run',str(source)],input=data['stdin'].encode(),stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=root,env=env,timeout=15);elapsed=(time.perf_counter_ns()-start)/1e6
   actual=p.stdout.decode().replace('\r\n','\n');assert p.returncode==0 and actual==expected,(name,n,label,p.returncode,actual,p.stderr)
   all_runs.append({'warmup':i<method['warmup_per_variant_case'],'elapsed_ms':elapsed,'exit':p.returncode,'stdout':actual,'stderr':p.stderr.decode()})
  samples=[r['elapsed_ms'] for r in all_runs if not r['warmup']];median=statistics.median(samples)
  row['variants'][label]={'median_ms':median,'min_ms':min(samples),'max_ms':max(samples),'passes_original_limit':median<=row['original_limit_ms'],'runs':all_runs}
  print(name,n,label,round(median,4),'ms',metadata[label]['bytes'],'bytes',flush=True)
 result['cases'].append(row)
 # 每个完整case保存checkpoint，不覆盖最终只写一次文件。
 (base/f'checkpoint-{case_index}.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf8')
result['ended_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();out=base/'results.json';assert not out.exists();out.write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf8')
print('complete',out,flush=True)
