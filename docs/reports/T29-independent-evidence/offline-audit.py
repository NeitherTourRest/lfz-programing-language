"""Offline T29 audit. Reads repository JSON/ZIP only, launches no interpreters."""
from pathlib import Path
from zipfile import ZipFile
import argparse,hashlib,json,statistics,math,collections
ROOT=Path(__file__).resolve().parents[3]
parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path);options=parser.parse_args()
read=lambda p:json.loads(p.read_bytes());digest=lambda b:hashlib.sha256(b).hexdigest();sha=lambda p:digest(p.read_bytes())
folder=ROOT/'benchmarks/t29/results';gate=read(folder/'gate.json');receipt=read(folder/'gate.receipt.json');screen=read(folder/'screen.json');bundle=folder/'benchmark-inputs.zip';assert sha(bundle)=='8800480693667d9c296412564fafea3d3ada488a6749dcac9ac698fa57d9383a'
with ZipFile(bundle) as z:inputs={n:z.read(n)for n in z.namelist()};assert len(inputs)==len(z.namelist())
protocol=json.loads(inputs['benchmarks/t13/protocol.json']);assert protocol==gate['protocol'];assert digest(inputs['benchmarks/t13/protocol.json'])==gate['protocol_sha256']=='74934a4ebc1f2eca5a6ea00e8d6e1c39ed5a4de5315a6cb8524d076d84c9f1bb'
for name,h in {**protocol['original_baseline_files'],**protocol['supporting_scripts']}.items():assert digest(inputs[name])==h
assert sha(folder/'gate.json')==receipt['raw_sha256']=='66aa0e666abcda306369c710bbd32b246b0645a35c2603db6f40452e6c3b916b'
assert gate['status']==receipt['status']=='completed' and receipt['exit']==0 and gate['timing_performed'];assert receipt['runner_sha256']==digest(inputs['benchmarks/t13/run.py']);assert receipt['head']=='66565e31bf677855b79502f15c47fc729466ebe3'
candidate='f0d483f7a0cbf2d9c45da5bd684327f8ac177ac4c2378e23230eba86d29a4ed9';assert receipt['candidate_sha256']==gate['hashes']['new']==candidate
products={};archives={}
for tag in ('t28','t29'):
 path=ROOT/f'benchmarks/{tag}/results/candidate-snapshot.zip';meta=read(path.with_suffix('.json'));assert sha(path)==meta['archive_sha256'] and path.stat().st_size==meta['archive_bytes']
 with ZipFile(path) as z:
  assert set(z.namelist())==set(meta['entries']) and len(z.namelist())==len(meta['entries']);files={n:z.read(n)for n in z.namelist()}
  for n,data in files.items():assert digest(data)==meta['entries'][n]['sha256'] and len(data)==meta['entries'][n]['bytes']
 products[tag]=digest(files['lfz.exe']);archives[tag]=meta['archive_sha256']
 if tag=='t29':
  assert products[tag]==candidate and len(files['lfz.exe'])==1137664
  assert receipt['sources_before']==receipt['sources_after'] and all(digest(files[n])==v for n,v in receipt['sources_before'].items())
assert gate['source_hashes']==gate['end_source_hashes'] and all(receipt['sources_before'][n]==v for n,v in gate['source_hashes'].items())
for label in ('old','t12'):assert gate['hashes'][label]==gate['binary_sources'][label]['sha256']==protocol['baselines'][label]['sha256']
assert gate['binary_sources']['new']['path']=='Temp/T29/candidate.exe' and gate['binary_sources']['new']['bytes']==gate['new_bytes']==1137664
assert (protocol['warmup_runs_per_label_case'],protocol['formal_runs_per_label_case'],protocol['settle_after_copy_seconds'])==(2,7,5)
counts=collections.Counter();group_count=0

def samples(group,label,expected):
 global group_count
 group_count+=1;assert len(group['warmup_runs'])==2 and len(group['runs'])==7
 for i,row in enumerate(group['warmup_runs']+group['runs']):
  phase='warmup'if i<2 else'formal';assert row['index']==i and row['phase']==phase and row['sha256']==gate['hashes'][label];assert row['exit']==0 and row['stdout']==expected and row['stderr']==''
  assert all(math.isfinite(row[k])and row[k]>=0 for k in ('elapsed_ms','user_ms','kernel_ms'));counts[phase]+=1
 for field,target in [('elapsed_ms','median_ms'),('user_ms','median_user_ms'),('kernel_ms','median_kernel_ms')]:assert statistics.median(row[field]for row in group['runs'])==group[target]
 assert min(x['elapsed_ms']for x in group['runs'])==group['min_ms'] and max(x['elapsed_ms']for x in group['runs'])==group['max_ms'];return group['median_ms']
original=[row for name in protocol['original_baseline_files']for row in json.loads(inputs[name])['cases']];assert len(original)==len(gate['cases'])==19
verdicts=collections.Counter();summaries=[]
for i,(base,locked,case)in enumerate(zip(original,protocol['cases'],gate['cases'])):
 assert all(base[k]==locked[k]==case[k]for k in ('name','n','source','stdin','expected_stdout'));assert all(case[k]==v for k,v in locked.items());assert digest(case['source'].encode())==case['source_sha256'];limit=max(base['median_ms']*(1.3 if base['name']=='noop' else 1.2),base['median_ms']+5);assert case['original_before_ms']==base['median_ms'] and limit==case['original_fixed_limit_ms']
 order=['old','t12','new'];order=order[i%3:]+order[:i%3];assert case['label_order']==list(case['groups'])==order
 med={k:samples(case['groups'][k],k,case['expected_stdout'])for k in order};status='PASS'if med['new']<=limit else'ENVIRONMENT_INDETERMINATE'if med['old']>limit else'FAIL';assert status==case['absolute_status'];assert med['new']/med['old']==case['new_over_old'] and med['new']/med['t12']==case['new_over_t12'];verdicts[status]+=1;summaries.append(dict(name=case['name'],n=case['n'],median_ms=med,limit_ms=limit,status=status))
caps={};expected_names=[(name,n)for n in (2000,4000,8000)for name in ('json_parse','json_stringify')]+[(n,0)for n in ('cli_eval','cli_stdin','cli_argv')];assert [(g['name'],g['n'])for g in gate['capabilities']]==expected_names
for i,g in enumerate(gate['capabilities']):
 order=['t12','new']if i%2==0 else['new','t12'];assert g['label_order']==list(g['groups'])==order;name=g['name'];n=g['n']
 if n:
  data=json.dumps({f'k{j}':j for j in range(n)},sort_keys=True,separators=(',',':')).encode();assert digest(data)==g['input_sha256'] and len(data)==g['input_bytes'] and g['iterations']==20;assert digest(g['source'].encode())==g['source_sha256'];expected=f"RESULT total={20*(n-1 if name=='json_parse'else len(data))}\n"
 else:expected=(json.dumps(['literal','中文','--json','-e',''],ensure_ascii=False,separators=(',',':'))+'\n')if name=='cli_argv'else'RESULT ok=1\n'
 assert expected==g['expected_stdout'];med={k:samples(g['groups'][k],k,expected)for k in order};assert med['new']/med['t12']==g['new_over_t12'];caps[name,n]=med
probe=gate['dist_path_probe'];assert probe['path']=='Temp/T29/candidate.exe' and probe['sha256']==candidate and probe['samples']==probe['warmup_runs']+probe['runs'];numeric=next(c for c in gate['cases']if(c['name'],c['n'])==('numeric_loop',500000));probe_ms=samples(probe,'new',numeric['expected_stdout']);assert group_count==76 and counts=={'warmup':152,'formal':532}
startup=gate['cases'][0]['groups']['new']['median_ms'];json_rule={}
for name in ('json_parse','json_stringify'):
 total=[caps[name,n]['new']for n in (2000,4000,8000)];net=[x-startup for x in total];shares=[startup/x for x in total];eligible=[x>5 and y<.5 for x,y in zip(net,shares)];ratios=[net[i+1]/net[i]if eligible[i]and eligible[i+1]else None for i in range(2)];json_rule[name]=dict(total_ms=total,approx_net_ms=net,startup_share=shares,eligible=eligible,doubling_ratios=ratios)
struct=next(c for c in gate['cases']if(c['name'],c['n'])==('struct_ops',8000));struct_pass=struct['groups']['new']['median_ms']<=struct['original_before_ms']*.7;assert gate['assessment']['absolute_counts']=={s:verdicts[s]for s in ('PASS','FAIL','ENVIRONMENT_INDETERMINATE')} and verdicts=={'PASS':19} and struct_pass and gate['assessment']['overall_original_efficiency_gate_pass'];assert all(x is None or x<=3 for v in json_rule.values()for x in v['doubling_ratios'])
# Prespecified screen audit uses archived fixtures, never host paths.
assert sha(folder/'screen.json')=='c88474ea44fa69c0831cd745f25a6ba3b9cbfd8d7d14ebdf6d9a4c21bfe532d1';criteria=json.loads(inputs['benchmarks/t29/results/screen-protocol.json'])['criteria'];assert criteria==screen['criteria'];assert all(digest(inputs[n])==v for n,v in screen['dependencies_sha256'].items());names=criteria['cases_in_initial_order'];assert list(screen['cases'])==names and screen['status']=='completed';assert screen['binary_sources']['candidate']['sha256']==products['t29'] and screen['binary_sources']['t28']['sha256']==products['t28']
seq=[]
for j in range(-1,6):
 labels=['candidate','t28']if j%2 else['t28','candidate'];off=(j+1)%6;seq.extend((j,l,n)for l in labels for n in names[off:]+names[:off])
assert len(seq)==len(screen['samples'])==84;screen_rows={}
for n,c in screen['cases'].items():assert digest(c['source'].encode())==c['sha256']
for (j,l,n),s in zip(seq,screen['samples']):
 assert (s['round'],s['label'],s['case'])==(j,l,n)and s['binary_sha256']==screen['binary_sources'][l]['sha256']and s['fixture_sha256']==screen['cases'][n]['sha256'];assert s['stdin_hex']==screen['cases'][n]['stdin'].encode().hex();assert s['exit']==0 and not s['timed_out'] and s['matched']and not bytes.fromhex(s['stderr_hex']);assert bytes.fromhex(s['stdout_hex']).replace(b'\r\n',b'\n')==screen['cases'][n]['stdout'].encode()
for n in names:
 paired={l:{s['round']:s['elapsed_ms']for s in screen['samples']if s['case']==n and s['label']==l and s['round']>=0}for l in ('t28','candidate')};assert all(set(v)==set(range(6))for v in paired.values());old=statistics.median(paired['t28'].values());new=statistics.median(paired['candidate'].values());wins=sum(paired['candidate'][j]<paired['t28'][j]for j in range(6));slower=sum(paired['candidate'][j]>paired['t28'][j]for j in range(6));row=dict(t28_median_ms=old,candidate_median_ms=new,candidate_over_t28_median=new/old,paired_wins=wins,paired_slower_rounds=slower,paired_ties=6-wins-slower,paired_elapsed_ms=[dict(round=j,t28=paired['t28'][j],candidate=paired['candidate'][j])for j in range(6)]);assert row==screen['assessment']['cases'][n];screen_rows[n]=row
improve={n:screen_rows[n]['candidate_median_ms']<screen_rows[n]['t28_median_ms']and screen_rows[n]['paired_wins']>=4 for n in ('array_full','string_full')};veto={n:screen_rows[n]['candidate_median_ms']>screen_rows[n]['t28_median_ms']*1.1 and screen_rows[n]['paired_slower_rounds']>=5 for n in ('control','push_self_8000')};assert improve==screen['assessment']['improvement_pass'] and veto==screen['assessment']['regression_veto'] and screen['assessment']['screen_pass']==(all(improve.values())and not any(veto.values()))
result=dict(status='INDEPENDENT_MEASUREMENT_GATE_PASS',interpreter_launches=0,raw_sha256=sha(folder/'gate.json'),receipt_sha256=sha(folder/'gate.receipt.json'),protocol_sha256=gate['protocol_sha256'],candidate_sha256=candidate,archives=archives,input_zip_sha256=sha(bundle),samples=dict(counts),groups=76,counts=dict(verdicts),cases=summaries,startup_ms=startup,struct8=dict(original_ms=struct['original_before_ms'],candidate_ms=struct['groups']['new']['median_ms'],reduction_percent=100*(1-struct['groups']['new']['median_ms']/struct['original_before_ms'])),json_rule=json_rule,candidate_path_probe_ms=probe_ms,screen=dict(samples=84,passes=True,cases=screen_rows))
text=json.dumps(result,ensure_ascii=False,indent=2)+'\n'
if options.output:
 target=options.output.resolve();assert target.is_relative_to(ROOT/'Temp')and not target.exists();target.parent.mkdir(parents=True,exist_ok=True);target.write_text(text,encoding='utf8')
else:print(text)

