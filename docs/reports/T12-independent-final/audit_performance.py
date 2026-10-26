import hashlib,json,math,statistics
from pathlib import Path
R=Path.cwd();P=R/'benchmarks/t12/results';O=R/'docs/reports/T12-independent-final/performance-audit.json'
read=lambda p:json.loads(p.read_text(encoding='utf8'))
d=read(P/'opt2-same-path.json');a=read(P/'opt2-assessment.json');f=read(P/'opt2-new-features.json')
new='47ed2bfe2a2694b11bd611172cfcf0000d0bfc58e3b9bd809ca95f066dd0261f';old='149b184e442ffbf250590431fa0aeffab401eac526faf60b4b435f620725d5ab'
assert d['binary_sources']['new']['sha256']==new and d['binary_sources']['old']['sha256']==old
assert d['binary_sources']['new']['bytes']==759808
assert f['binary_sha256']==new and f['binary_bytes']==759808
assert d['method']['warmup_after_each_copy']==2 and d['method']['runs']==7 and d['method']['settle_after_each_copy_seconds']==5
assert f['method']['warmup']==2 and f['method']['runs']==7
assert d['affinity']['enabled'] is False
base={}
for name in ['before.json','before-small.json']:
 for r in read(P/name)['cases']:base[(r['name'],r['n'])]=r
assert len(base)==len(d['cases'])==len(a['cases'])==19
summaries=[]
for r,ass in zip(d['cases'],a['cases']):
 b=base[(r['name'],r['n'])]
 assert (ass['name'],ass['n'])==(r['name'],r['n'])
 assert r['source']==b['source'] and r['stdin']==b['stdin'] and r['expected_stdout']==b['expected_stdout']
 limit=max(b['median_ms']*(1.3 if r['name']=='noop' else 1.2),b['median_ms']+5)
 assert math.isclose(r['original_fixed_limit_ms'],limit)
 for label,sha in [('old',old),('new',new)]:
  g=r['groups'][label];runs=g['runs'];assert len(runs)==7
  assert statistics.median(x['elapsed_ms'] for x in runs)==g['median_ms']
  assert statistics.median(x['user_ms'] for x in runs)==g['median_user_ms']
  assert statistics.median(x['kernel_ms'] for x in runs)==g['median_kernel_ms']
  for v in runs:assert v['sha256']==sha and v['exit']==0 and v['stdout']==r['expected_stdout'] and not v['stderr']
 n=r['groups']['new']['median_ms'];o=r['groups']['old']['median_ms']
 status='PASS' if n<=limit else ('ENVIRONMENT_INDETERMINATE' if o>limit else 'FAIL')
 assert ass['absolute_status']==status and math.isclose(ass['paired_new_over_old'],n/o)
 assert ass['new_wall_ms']==n and ass['same_path_old_wall_ms']==o
 assert ass['old_user_ms']==r['groups']['old']['median_user_ms'] and ass['new_user_ms']==r['groups']['new']['median_user_ms']
 summaries.append({'name':r['name'],'n':r['n'],'status':status,'old_ms':o,'new_ms':n,'fixed_limit_ms':limit,'paired_ratio':n/o,'old_user_ms':ass['old_user_ms'],'new_user_ms':ass['new_user_ms']})
for name,sha in a['source_artifact_sha256'].items():assert hashlib.sha256((P/name).read_bytes()).hexdigest()==sha
for r in f['cases']:
 assert len(r['runs'])==7 and statistics.median(x['elapsed_ms'] for x in r['runs'])==r['median_ms']
 for v in r['runs']:assert v['sha256']==new and v['exit']==0 and v['stdout']==r['expected_stdout'] and not v['stderr']
 if r['name'].startswith('json_'):
  n=r['n'];data=json.dumps({f'k{i}':i for i in range(n)},sort_keys=True,separators=(',',':')).encode()
  assert r['input_bytes']==len(data) and r['input_sha256']==hashlib.sha256(data).hexdigest()
  want=20*(n-1 if r['name']=='json_parse' else len(data))
  assert r['iterations']==20 and r['expected_stdout']==f'RESULT total={want}\n'
counts={s:sum(x['status']==s for x in summaries) for s in ['PASS','ENVIRONMENT_INDETERMINATE','FAIL']}
assert counts=={'PASS':15,'ENVIRONMENT_INDETERMINATE':3,'FAIL':1}
assert a['R304']=='OPEN' and a['overall_efficiency_pass'] is False and d['all_19_fixed_limits_pass'] is False
assert len(f['cases'])==9 and sum(len(r['runs']) for r in f['cases'])==63
result={'scope':'independent arithmetic/hash/method/output audit; no new timing window','new_sha256':new,'counts':counts,'old_new_formal_runs':266,'dist_formal_runs':len(d['dist_path_probe']['runs']),'new_features_formal_runs':63,'cases':summaries,'R304':'OPEN','peak_memory_measured':False,'notes':['profile opt2 and while change both present; benefits not attributable solely to one','whole process wall includes startup; user/kernel accounting may quantize short tasks','three reference failures with old also over threshold remain indeterminate, not passes','one string_map8k exceeds fixed absolute line with old within; no new window selected']}
O.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print(json.dumps({'audit':'PASS','counts':counts,'R304':'OPEN'}))