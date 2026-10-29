"""Verifier's independent audit of existing T13 raw data: zero process launches."""
import collections,datetime,hashlib,json,math,statistics,zipfile
from pathlib import Path
R=Path(__file__).resolve().parents[3]
E=R/'docs/reports/T13-independent-evidence'
RAW=R/'benchmarks/t13/results/final-window.json'
PRO=R/'benchmarks/t13/protocol.json'
read=lambda p:json.loads(p.read_text(encoding='utf8'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
digest=lambda b:hashlib.sha256(b).hexdigest()
r=read(RAW);p=read(PRO)
assert r['status']=='completed' and r['timing_performed'] is True
assert r['protocol']==p and r['protocol_sha256']==sha(PRO)=='74934a4ebc1f2eca5a6ea00e8d6e1c39ed5a4de5315a6cb8524d076d84c9f1bb'
assert datetime.datetime.fromisoformat(r['timestamp_utc']) < datetime.datetime.fromisoformat(r['finished_utc'])
assert r['source_hashes']==r['end_source_hashes']
archive=R/'docs/reports/T13-release-evidence/candidate-snapshot.zip'
z=zipfile.ZipFile(archive)
assert set(z.namelist())==set(r['source_hashes'])|{'dist/lfz.exe'}
for name,h in r['source_hashes'].items():assert digest(z.read(name))==h,('frozen archive source mismatch',name)
current_sources_match=all((R/name).exists() and sha(R/name)==h for name,h in r['source_hashes'].items())
release=read(R/'docs/reports/T13-release-evidence/source-hashes.json')
assert {row['path']:row['sha256'] for row in release}==r['source_hashes']
for name,h in {**p['original_baseline_files'],**p['supporting_scripts']}.items():assert sha(R/name)==h,name
assert (p['warmup_runs_per_label_case'],p['formal_runs_per_label_case'],p['settle_after_copy_seconds'],p['timeout_seconds'])==(2,7,5,15)
assert r['affinity']['modified'] is False and r['affinity']['process_mask']==16777215
hashes={'old':'149b184e442ffbf250590431fa0aeffab401eac526faf60b4b435f620725d5ab','t12':'47ed2bfe2a2694b11bd611172cfcf0000d0bfc58e3b9bd809ca95f066dd0261f','new':'9cda0c6fd5d85178335e8b758a843fc5f66574826bab4a80f2418a318e171cf9'}
assert r['hashes']==hashes
for label,meta in r['binary_sources'].items():
 path=R/meta['path']
 data=z.read('dist/lfz.exe') if label=='new' else path.read_bytes()
 assert digest(data)==meta['sha256']==hashes[label]
 assert len(data)==meta['bytes']
assert r['new_bytes']==761344
# Rebuild limits and the standard case oracle from the untouched original baselines.
base=read(R/'benchmarks/t12/results/before.json')['cases']+read(R/'benchmarks/t12/results/before-small.json')['cases']
assert len(base)==len(p['cases'])==len(r['cases'])==19
labels=['old','t12','new'];formal=warm=0
checks=[];counts=collections.Counter()

def verify_group(g,label,expected):
 global formal,warm
 assert len(g['runs'])==7 and len(g['warmup_runs'])==2
 samples=g['warmup_runs']+g['runs']
 assert [s['index'] for s in samples]==list(range(9))
 assert [s['phase'] for s in samples]==['warmup']*2+['formal']*7
 for s in samples:
  assert s['sha256']==hashes[label] and s['exit']==0 and s['stdout']==expected and s['stderr']=='',s
  for k in ['elapsed_ms','user_ms','kernel_ms']:assert math.isfinite(s[k]) and s[k]>=0,(k,s)
 formal+=7;warm+=2
 values=[s['elapsed_ms'] for s in g['runs']]
 for key,value in [('median_ms',statistics.median(values)),('min_ms',min(values)),('max_ms',max(values)),('median_user_ms',statistics.median(s['user_ms'] for s in g['runs'])),('median_kernel_ms',statistics.median(s['kernel_ms'] for s in g['runs']))]:
  assert g[key]==value,(key,g[key],value)
 return statistics.median(values)

def standard_oracle(name,n):
 if name=='noop':return 'RESULT ok=1\n'
 if name=='numeric_loop':return f'RESULT total={n*(n+1)//2}\n'
 if name=='array_builtins':return f'RESULT total={sum((i*7%n) for i in range(n) if (i*7%n)%2==0)}\n'
 if name=='struct_ops':return f'RESULT keys={n} total={sum(i*i for i in range(n))}\n'
 if name in ('string_ops','string_map'):return f'RESULT chars={sum(len("item"+str(i)) for i in range(n))+n-1} words={n} tag=ababab\n'
 if name=='struct_small':return f'RESULT total={1000*sum(i*i for i in range(n))}\n'
 raise AssertionError(name)

for index,(original,frozen,row) in enumerate(zip(base,p['cases'],r['cases'])):
 name,n=original['name'],original['n'];expected=standard_oracle(name,n)
 assert (row['name'],row['n'])==(frozen['name'],frozen['n'])==(name,n)
 for key in ['source','source_sha256','stdin','expected_stdout']:assert row[key]==frozen[key]==original[key],(name,key)
 assert row['expected_stdout']==expected
 assert digest(row['source'].encode('utf8'))==row['source_sha256']
 assert row['original_before_ms']==frozen['original_before_ms']==original['median_ms']
 limit=max(original['median_ms']*(1.3 if name=='noop' else 1.2),original['median_ms']+5)
 assert row['original_fixed_limit_ms']==frozen['original_fixed_limit_ms']==limit,(name,n,limit)
 assert row['label_order']==labels[index%3:]+labels[:index%3]
 medians={label:verify_group(row['groups'][label],label,expected) for label in labels}
 verdict='PASS' if medians['new']<=limit else ('ENVIRONMENT_INDETERMINATE' if medians['old']>limit else 'FAIL')
 assert row['absolute_status']==verdict
 assert row['new_over_old']==medians['new']/medians['old']
 assert row['new_over_t12']==medians['new']/medians['t12']
 counts[verdict]+=1
 checks.append({'name':name,'n':n,'old_ms':medians['old'],'t12_ms':medians['t12'],'new_ms':medians['new'],'original_limit_ms':limit,'absolute_status':verdict,'new_over_old':row['new_over_old'],'new_over_t12':row['new_over_t12']})
assert len(r['capabilities'])==9
for index,row in enumerate(r['capabilities']):
 name,n=row['name'],row['n']
 if name.startswith('json_'):
  data=json.dumps({f'k{i}':i for i in range(n)},ensure_ascii=False,separators=(',',':'),sort_keys=True).encode('utf8')
  assert n in (2000,4000,8000) and row['iterations']==20
  assert row['input_bytes']==len(data) and row['input_sha256']==digest(data)
  assert digest(row['source'].encode('utf8'))==row['source_sha256']
  expect=f'RESULT total={20*((n-1) if name=="json_parse" else len(data))}\n'
 elif name in ('cli_eval','cli_stdin'):expect='RESULT ok=1\n'
 elif name=='cli_argv':expect=json.dumps(['literal','中文','--json','-e',''],ensure_ascii=False,separators=(',',':'))+'\n'
 else:raise AssertionError(name)
 assert row['expected_stdout']==expect
 assert row['label_order']==(['t12','new'] if index%2==0 else ['new','t12'])
 t=verify_group(row['groups']['t12'],'t12',expect);v=verify_group(row['groups']['new'],'new',expect)
 assert row['new_over_t12']==v/t
probe=r['dist_path_probe'];assert probe['sha256']==hashes['new'] and probe['n']==500000
assert probe['samples']==probe['warmup_runs']+probe['runs']
verify_group(probe,'new',standard_oracle('numeric_loop',500000))
assert (formal,warm)==(532,152),(formal,warm)
assert dict(counts)==r['assessment']['absolute_counts']
struct=next(x for x in r['cases'] if x['name']=='struct_ops' and x['n']==8000)
target=struct['groups']['new']['median_ms']<=struct['original_before_ms']*.7
assert r['assessment']['struct8_original_30pct_target_pass']==target
assert r['assessment']['all_19_absolute_pass']==(counts['PASS']==19)
assert r['assessment']['overall_original_efficiency_gate_pass']==(counts['PASS']==19 and target)
assert r['assessment']['R304']=='OPEN' and counts=={'PASS':15,'ENVIRONMENT_INDETERMINATE':2,'FAIL':2}
assessment=read(R/'benchmarks/t13/results/assessment.json')
assert assessment['raw_sha256']==sha(RAW) and assessment['protocol_sha256']==sha(PRO)
assert assessment['absolute_counts']==dict(counts) and assessment['formal_runs_checked']==formal and assessment['warmup_runs_checked']==warm
result={'date':'2026-10-27','status':'audit PASS; efficiency gate NOT PASSED','interpreter_launches':0,'raw_sha256':sha(RAW),'protocol_sha256':sha(PRO),'perf_assessment_sha256':sha(R/'benchmarks/t13/results/assessment.json'),'binary_hashes':hashes,'formal_samples_verified':formal,'warmup_samples_verified':warm,'all_outputs_and_stderr_and_exit_valid':True,'raw_start_end_release_and_archive_sources_equal':True,'current_worktree_still_matches_measured_sources':current_sources_match,'candidate_snapshot_sha256':sha(archive),'original_thresholds_unchanged':True,'medians_and_cpu_and_ratios_recomputed':True,'absolute_counts':dict(counts),'struct8_original_30pct_target_pass':target,'R304':'OPEN','cases':checks}
(E/'performance-audit.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
print(json.dumps({k:v for k,v in result.items() if k not in ('cases','binary_hashes')},ensure_ascii=False))
