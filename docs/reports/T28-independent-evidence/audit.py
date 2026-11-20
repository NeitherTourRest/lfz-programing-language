from pathlib import Path
import hashlib,json,math,statistics,collections,datetime
import argparse
ap=argparse.ArgumentParser();ap.add_argument('--candidate-sha',required=True);ap.add_argument('--parent-head',required=True);ap.add_argument('--raw',type=Path,default=Path('Temp/T28/gate.json'));ap.add_argument('--receipt',type=Path,default=Path('Temp/T28/gate.receipt.json'));options=ap.parse_args()
r=Path.cwd(); w=r/'Temp/T28'; out=r/'Temp/T28-verifier'; raw=options.raw; receipt=options.receipt; p=r/'benchmarks/t13/protocol.json'; rd=lambda f:json.loads(f.read_text(encoding='utf-8-sig')); sha=lambda b:hashlib.sha256(b).hexdigest(); hh=lambda f:sha(f.read_bytes()); a=rd(raw); b=rd(receipt); protocol=rd(p); psha='74934a4ebc1f2eca5a6ea00e8d6e1c39ed5a4de5315a6cb8524d076d84c9f1bb'; candidate=options.candidate_sha
assert hh(p)==a['protocol_sha256']==psha and a['protocol']==protocol
assert hh(raw)==b['raw_sha256'] and b['exit']==0 and b['status']==a['status']=='completed' and a['timing_performed']
for path, expected in {**protocol['original_baseline_files'],**protocol['supporting_scripts']}.items():assert hh(r/path)==expected,(path,expected)
assert b['runner_sha256']==hh(r/'benchmarks/t13/run.py') and b['candidate_sha256']==a['hashes']['new']==candidate
assert a['binary_sources']['new']['path']=='Temp/T28/candidate.exe' and hh(r/'Temp/T28/candidate.exe')==candidate
for label, meta in a['binary_sources'].items():
 path=r/meta['path'];assert hh(path)==meta['sha256']==a['hashes'][label] and path.stat().st_size==meta['bytes'];assert label=='new' or {k:meta[k]for k in ('path','sha256')}==protocol['baselines'][label]
assert a['binary_sources']['new']['bytes']==a['new_bytes']
assert a['source_hashes']==a['end_source_hashes'] and b['sources_before']==b['sources_after'];assert all(hh(r/n)==v for n,v in b['sources_before'].items());assert all(b['sources_before'][n]==v for n,v in a['source_hashes'].items());assert set(b['sources_before'])=={p.relative_to(r).as_posix()for p in (r/'src').rglob('*.rs')}|{'Cargo.toml','Cargo.lock'}
assert a['affinity']['modified'] is False and a['affinity']['process_mask']==a['affinity']['system_mask']
assert protocol['whole_process'] and (protocol['warmup_runs_per_label_case'],protocol['formal_runs_per_label_case'],protocol['settle_after_copy_seconds'])==(2,7,5)
count=collections.Counter(); groups=0; cases=[]; caps=[]
def inspect(g,label,expected):
 global groups
 groups+=1
 assert len(g['warmup_runs'])==2 and len(g['runs'])==7
 for i,s in enumerate(g['warmup_runs']+g['runs']):
  phase='warmup' if i<2 else 'formal';assert s['index']==i and s['phase']==phase and s['sha256']==a['hashes'][label];assert s['exit']==0 and s['stdout']==expected and s['stderr']==''
  assert all(math.isfinite(s[k]) and s[k]>=0 for k in ('elapsed_ms','user_ms','kernel_ms')) and s['elapsed_ms']>0; count[phase]+=1
 def vals(k):return [s[k] for s in g['runs']]
 result=statistics.median(vals('elapsed_ms'));assert result==g['median_ms'] and min(vals('elapsed_ms'))==g['min_ms'] and max(vals('elapsed_ms'))==g['max_ms']; assert statistics.median(vals('user_ms'))==g['median_user_ms'] and statistics.median(vals('kernel_ms'))==g['median_kernel_ms'];return result
original=[]
for path in ('benchmarks/t12/results/before.json','benchmarks/t12/results/before-small.json'):original.extend(rd(r/path)['cases'])
assert len(original)==len(protocol['cases'])==len(a['cases'])==19
labels=['old','t12','new']; state=collections.Counter()
for i,(o,z,g)in enumerate(zip(original,protocol['cases'],a['cases'])):
 assert all(o[k]==z[k]==g[k] for k in ('name','n','source','stdin','expected_stdout'));assert all(g[k]==v for k,v in z.items());assert sha(g['source'].encode())==hh(r/g['input_script'])==z['source_sha256'];lim=max(o['median_ms']*(1.3 if o['name']=='noop' else 1.2),o['median_ms']+5);assert o['median_ms']==z['original_before_ms'] and lim==z['original_fixed_limit_ms'];order=labels[i%3:]+labels[:i%3];assert g['label_order']==list(g['groups'])==order
 med={k:inspect(g['groups'][k],k,z['expected_stdout'])for k in order}; x='PASS' if med['new']<=lim else ('ENVIRONMENT_INDETERMINATE' if med['old']>lim else 'FAIL');assert x==g['absolute_status'] and med['new']/med['old']==g['new_over_old'] and med['new']/med['t12']==g['new_over_t12'];state[x]+=1;cases.append(dict(name=g['name'],n=g['n'],original=o['median_ms'],limit=lim,median=med,status=x))
expected_caps=[(mode,n)for n in (2000,4000,8000) for mode in ('json_parse','json_stringify')]+[(mode,0)for mode in ('cli_eval','cli_stdin','cli_argv')];assert [(g['name'],g['n'])for g in a['capabilities']]==expected_caps
for i,g in enumerate(a['capabilities']):
 order=['t12','new'] if not i%2 else ['new','t12'];assert g['label_order']==list(g['groups'])==order;name=g['name'];n=g['n']; expected='RESULT ok=1\n'
 if n:
  data=json.dumps({f'k{i}':i for i in range(n)},sort_keys=True,separators=(',',':')).encode();assert g['iterations']==20 and len(data)==g['input_bytes'] and sha(data)==g['input_sha256']==hh(r/f'Temp/t13-perf/json-{n}.json');assert sha(g['source'].encode())==g['source_sha256']==hh(Path(g['arguments'][1]));assert g['arguments'][0]=='run' and g['stdin']==''; expected=f"RESULT total={20*(n-1 if name=='json_parse' else len(data))}\n"
 elif name=='cli_eval':assert g['arguments']==['-e',g['source']] and g['stdin']==''
 elif name=='cli_stdin':assert g['arguments']==['-'] and g['stdin']==g['source']
 else:
  args=['literal','\u4e2d\u6587','--json','-e',''];assert g['arguments']==['-e',g['source'],'--',*args] and g['stdin']=='';expected=json.dumps(args,ensure_ascii=False,separators=(',',':'))+'\n'
 assert expected==g['expected_stdout'];med={k:inspect(g['groups'][k],k,expected)for k in order};assert med['new']/med['t12']==g['new_over_t12'];caps.append(dict(name=name,n=n,median=med))
d=a['dist_path_probe'];assert d['sha256']==candidate and d['path']=='Temp/T28/candidate.exe' and d['name']=='dist_numeric_loop' and d['n']==500000;assert d['samples']==d['warmup_runs']+d['runs'];den=next(g for g in protocol['cases']if g['name']=='numeric_loop' and g['n']==500000); dm=inspect(d,'new',den['expected_stdout'])
assert groups==76 and count=={'warmup':152,'formal':532}; struct=next(g for g in cases if g['name']=='struct_ops' and g['n']==8000);target=struct['original']*.7; achieved=struct['median']['new']<=target;assert a['assessment']['absolute_counts']=={k:state[k]for k in ('PASS','FAIL','ENVIRONMENT_INDETERMINATE')} and a['assessment']['struct8_original_30pct_target_pass']==achieved;overall=state['PASS']==19 and achieved;assert a['assessment']['overall_original_efficiency_gate_pass']==overall and a['assessment']['R304']==('MEASUREMENT_GATE_PASS_REQUIRES_INDEPENDENT_REVIEW' if overall else 'OPEN')
assert b['head']==options.parent_head
startup=cases[0]['median']['new']
json_scaling={}
for name in ('json_parse','json_stringify'):
 measurements=[g for g in caps if g['name']==name]
 totals=[g['median']['new'] for g in measurements]; net=[v-startup for v in totals]; shares=[startup/v for v in totals]; eligible=[v>5 and share<.5 for v, share in zip(net,shares)]
 steps=[net[i+1]/net[i] if eligible[i] and eligible[i+1] else None for i in range(2)]
 json_scaling[name]=dict(totals_ms=totals,near_net_ms=net,startup_share=shares,eligible=eligible,eligible_doubling_ratios=steps)
info=dict(protocol_sha256=hh(p),raw_sha256=hh(raw),receipt_sha256=hh(receipt),candidate_sha256=candidate,source_files_verified=len(b['sources_before']),groups=groups,samples=dict(count),absolute=dict(state),cases=cases,capabilities=caps,struct8=dict(target_ms=target,original_ms=struct['original'],candidate_ms=struct['median']['new'],reduction_percent=100*(1-struct['median']['new']/struct['original'])),dist_probe_median_ms=dm,measurement_gate=('PASS' if overall and all(v is None or v<=3.0 for q in json_scaling.values() for v in q['eligible_doubling_ratios']) else 'FAIL'),json_scaling=json_scaling,startup_ms=startup);(out/'audit.json').write_text(json.dumps(info,indent=2)+'\n',encoding='utf8');print(json.dumps({k:v for k,v in info.items()if k not in ('cases','capabilities')},indent=2)); print([(g['name'],g['n'],g['status'])for g in cases if g['status']!='PASS'])



# Independent read-only focused and Python oracle audit.
from zipfile import ZipFile
z=r/'benchmarks/t28/results/candidate-snapshot.zip'; zm=rd(z.with_suffix('.json'));assert hh(z)==zm['archive_sha256'] and z.stat().st_size==zm['archive_bytes']
with ZipFile(z) as zz:
 assert set(zz.namelist())==set(zm['entries']) and len(zz.namelist())==len(zm['entries'])
 for name in zz.namelist():
  data=zz.read(name);assert sha(data)==zm['entries'][name]['sha256'] and len(data)==zm['entries'][name]['bytes'];assert sha(data)==(candidate if name=='lfz.exe' else b['sources_before'][name])
fpath=w/'focused/focused.json'; f=rd(fpath); ypath=w/'focused/python.json'; y=rd(ypath); assert f['status']==y['status']=='completed';assert (f['warmup_rounds'],f['formal_rounds'],f['settle_seconds'])==(1,6,1) and (y['warmup_rounds'],y['formal_rounds'])==(1,6)
assert f['binaries']=={'t27':hh(r/'Temp/T27/candidate.exe'),'candidate':candidate};assert f['runner_sha256']==y['runner_sha256']==hh(r/'benchmarks/t28/focused.py'); assert y['lfz_window_sha256']==hh(fpath) and y['script_sha256']==hh(r/y['script']) and y['python_sha256']==hh(Path(y['executable']))
names=['numeric_loop_10000','string_ops_2000','string_ops_4000','string_ops_8000','push_self_4000','push_alias_4000','push_self_8000','push_alias_8000']; assert list(f['cases'])==names
for name in names:
 v=f['cases'][name];n=int(v['stdin'].strip());assert v['sha256']==sha(v['source'].encode())==hh(w/f'focused/{name}.lfz')
 if name.startswith('numeric'): expected=f'RESULT total={n*(n+1)//2}\n'
 elif name.startswith('string'):
  joined=','.join('item'+str(i)for i in range(n));expected=f'RESULT chars={len(joined)} words={n} tag=ababab\n'
 else:expected=f"RESULT n={n} first=0 last={n-1} sum={n*(n-1)//2 if name.startswith('push_alias') else 0}\n"
 assert v['stdout']==expected
seq=[]
for round_i in range(-1,6):
 order=['candidate','t27']if round_i%2 else ['t27','candidate'];off=(round_i+1)%8;nc=names[off:]+names[:off]
 seq.extend((round_i,label,name)for label in order for name in nc)
assert len(seq)==len(f['samples'])==112
ft=collections.defaultdict(list)
for (rnd,label,name),x in zip(seq,f['samples']):
 assert (rnd,label,name)==(x['round'],x['label'],x['case']) and x['binary_sha256']==f['binaries'][label];assert x['exit']==0 and not x['timed_out'] and x['matched'] and not bytes.fromhex(x['stderr_hex']);assert bytes.fromhex(x['stdout_hex']).replace(b'\r\n',b'\n')==f['cases'][name]['stdout'].encode();assert math.isfinite(x['elapsed_ms'])and x['elapsed_ms']>0
 if rnd>=0:ft[name,label].append(x['elapsed_ms'])
for (name,label),values in ft.items():assert len(values)==6 and statistics.median(values)==f['median_ms'][name][label]
pnames=names[1:4]; seq=[];pt=collections.defaultdict(list)
for rnd in range(-1,6):
 off=(rnd+1)%3;seq.extend((rnd,n)for n in pnames[off:]+pnames[:off])
assert len(seq)==len(y['samples'])==21
for (rnd,name),x in zip(seq,y['samples']):
 assert (rnd,name)==(x['round'],x['case']) and x['exit']==0 and not x['timed_out'] and x['matched'] and not bytes.fromhex(x['stderr_hex']);assert bytes.fromhex(x['stdout_hex']).replace(b'\r\n',b'\n')==f['cases'][name]['stdout'].encode(); assert math.isfinite(x['elapsed_ms'])and x['elapsed_ms']>0
 if rnd>=0:pt[name].append(x['elapsed_ms'])
for name,values in pt.items():assert len(values)==6 and statistics.median(values)==y['median_ms'][name]
info['focused']=dict(raw_sha256=hh(fpath),samples=112,warmup=16,formal=96,median_ms=f['median_ms']);info['python']=dict(raw_sha256=hh(ypath),samples=21,warmup=3,formal=18,median_ms=y['median_ms'],binary_sha256=y['python_sha256']);info['zip_sha256']=hh(z)
(out/'audit.json').write_text(json.dumps(info,indent=2)+'\n',encoding='utf8');print('focused112/Python21 exact sample audit PASS')
