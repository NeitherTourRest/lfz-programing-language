"""Frozen T12 regression recheck: interleave old/new, preserve raw evidence."""
import json, os, pathlib, statistics, subprocess, time, hashlib, datetime
ROOT=pathlib.Path(__file__).resolve().parents[2]
old=ROOT/'Temp/t12-perf/lfz-before.exe'; new=ROOT/'dist/lfz.exe'
rows=json.loads((ROOT/'benchmarks/t12/results/before.json').read_text(encoding='utf-8'))['cases']
env=os.environ.copy(); env.update(TEMP=str(ROOT/'Temp/t12-perf'),TMP=str(ROOT/'Temp/t12-perf'))
result={'timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'old_sha256':hashlib.sha256(old.read_bytes()).hexdigest(),'new_sha256':hashlib.sha256(new.read_bytes()).hexdigest(),'method':'old/new interleaved, 2 warmups then 7 samples each; unchanged original thresholds','cases':[]}
for row in rows:
    if (row['name'],row['n']) not in [('numeric_loop',100000),('numeric_loop',500000),('string_ops',2000)]: continue
    path=ROOT/row['command'][-1]; path.write_text(row['source'],encoding='utf-8',newline='\n')
    runs={'old':[],'new':[]}
    for i in range(9):
        for label,binary in ([('old',old),('new',new)] if i%2==0 else [('new',new),('old',old)]):
            start=time.perf_counter_ns(); p=subprocess.run([str(binary),'run',str(path)],cwd=ROOT,env=env,input=row['stdin'].encode(),capture_output=True,timeout=15)
            ms=(time.perf_counter_ns()-start)/1e6
            actual=p.stdout.decode('utf-8').replace('\r\n','\n')
            assert p.returncode==0 and actual==row['expected_stdout'],(label,p.returncode,actual)
            if i>=2: runs[label].append({'elapsed_ms':ms,'exit':p.returncode,'stdout':actual,'stderr':p.stderr.decode('utf-8','replace')})
    med={label:statistics.median(v['elapsed_ms'] for v in vals) for label,vals in runs.items()}
    limit=max(row['median_ms']*1.2,row['median_ms']+5)
    item={'name':row['name'],'n':row['n'],'original_before_ms':row['median_ms'],'original_limit_ms':limit,'medians_ms':med,'new_pass':med['new']<=limit,'contemporary_old_in_range':med['old']<=limit,'runs':runs}
    result['cases'].append(item); print(item['name'],item['n'],med,'limit',limit,'pass',item['new_pass'])
output=ROOT/'benchmarks/t12/results/recheck.json'
assert not output.exists(); output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')