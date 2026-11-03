from pathlib import Path
import json,hashlib,statistics
R=Path.cwd();E=R/'docs/reports/T19-independent-evidence';W=R/'docs/reports/T19-performance-evidence/window01';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();H='5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5';assert sha(R/'target/release/lfz.exe')==H
samples=json.loads((W/'all-samples.json').read_text());assert len(samples)==42;summary=json.loads((W/'summary.json').read_text());inputs=[]
for size in [10,100,300]:
 m=json.loads((W/f'data-{size}-manifest.json').read_text());p=R/m['generator']['path'];digest=hashlib.sha256();total=0
 with p.open('rb') as f:
  while b:=f.read(1024*1024):digest.update(b);total+=len(b)
 assert total==m['generator']['bytes'] and digest.hexdigest()==m['generator']['sha256'];inputs.append(dict(mib=size,bytes=total,sha256=digest.hexdigest()))
for g in summary['summary']:
 allrows=[s for s in samples if s['mode']==g['mode'] and s['mib']==g['mib']];assert len(allrows)==6 and sum(s['warmup'] for s in allrows)==1
 formal=[s for s in allrows if not s['warmup']];assert statistics.median(s['processing_wall_ns']/1e9 for s in formal)==g['seconds_median'];assert [s['peak_working_set_bytes'] for s in formal]==g['peak_ws_bytes_all']
for s in samples:
 assert s['ok'] and s['exit']==0 and not s['watchdog'] and not s['stdout_tail'];m=json.loads((W/f"data-{s['mib']}-manifest.json").read_text())['independent_oracle'];expected={k:v for k,v in m.items() if k not in ['max_raw_line_bytes','bytes','sha256']};assert json.loads(s['stdout_marker'])==expected
n=json.loads((W/'data-100-manifest.json').read_text());a=dict(rows=0,valid=0,bad=0,sum_id=0,sum_value=0,active=0,weighted_mod=0,first_id=None,last_id=None);maxraw=0
with (R/n['generator']['path']).open('rb') as f:
 for raw in f:
  a['rows']+=1;maxraw=max(maxraw,len(raw.removesuffix(b'\n')))
  try:v=json.loads(raw)
  except json.JSONDecodeError:a['bad']+=1;continue
  a['valid']+=1;a['sum_id']+=v['id'];a['sum_value']+=v['value'];a['active']+=bool(v['active']);a['weighted_mod']=(a['weighted_mod']+(v['id']+1)*(v['value']+1001))%1000000007
  if a['first_id'] is None:a['first_id']=v['id']
  a['last_id']=v['id']
assert all(n['independent_oracle'][k]==v for k,v in a.items());assert maxraw==n['independent_oracle']['max_raw_line_bytes']
c=json.loads((R/'docs/reports/T19-performance-evidence/pilot01/os-peak-control.json').read_text());assert c['peak_working_set_bytes']>c['working_set_bytes_at_marker']+60*1048576
protocol=json.loads((W/'protocol.json').read_text());assert all(sha(R/'benchmarks/t19'/k)==v for k,v in protocol['drivers'].items())
(E/'scale-audit.json').write_text(json.dumps(dict(scope='raw evidence audit plus independent Python streaming100MiB oracle; NO new performance window or product run',inputs=inputs,samples_checked=42,oracle100=a,max_raw_line_bytes=maxraw,summary=summary,negative_control=dict(peak=c['peak_working_set_bytes'],current=c['working_set_bytes_at_marker']),driver_hashes=protocol['drivers']),indent=2),encoding='utf8');print('42 samples and input hashes PASS; independent100MiB oracle PASS')
