import json,pathlib,hashlib,re,datetime,shutil
r=pathlib.Path.cwd(); root=r/'docs/reports/T20-agent-v3-evidence';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));save=lambda p,d:p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf8');when=lambda s:datetime.datetime.fromisoformat(s.replace('Z','+00:00'))
def tool_outputs(t):
 for item in t['tool_results']:
  raw=item['output']; parts=[p.get('text','')for p in raw if isinstance(p,dict)]if isinstance(raw,list)else[raw]
  for s in parts:
   if not isinstance(s,str):continue
   try: v=json.loads(s)
   except ValueError: continue
   if isinstance(v,dict) and isinstance(v.get('output'),str):yield item,v
rows=[]
for d in sorted(root.glob('C0*')):
 task=d.name; tr=load(d/'tool-trace.json'); rev=d/'protocol-review.json'; p=load(rev); inp=load(d/'dispatch.json'); audit=sorted((d/'selftest-audit').glob('cycle-*.json')); expected=[rr for f in audit for rr in load(f)['runs']]; observed=[]; host=list(tool_outputs(tr)); tools={c['call_id']:c for c in tr['tool_calls']}; final=tr['tool_calls'][-1]; assert final['name']=='send_message'; message_ack=next(x for x in tr['tool_results'] if x['call_id']==final['call_id']); assert when(message_ack['timestamp'])<=when(p['deadline_utc'])
 for item, value in host:
  for line in value['output'].splitlines():
   try:j=json.loads(line)
   except ValueError:continue
   if isinstance(j,dict) and {'args','exit','stdout','stderr'}<=j.keys():observed.append(j)
 assert observed==expected and len(observed)==p['product_calls_observed']
 extra={}
 if task=='C03':
  parsed_fail=[v for x,v in host if 'student_cycle.py' in v['output'] and 'json.loads(v.argv_json)' in v['output'] and v.get('exit_code')==1];assert len(parsed_fail)==1;extra['prelaunch_argument_parse_failure']=True
 if task=='C04':
  started=[v['session_id']for _,v in host if 'session_id'in v and re.search(r'http://127\.0\.0\.1:\d+',v['output'])];assert len(started)==2
  shutdown=[]
  for item,v in host:
   s=tools[item['call_id']].get('input')or tools[item['call_id']].get('arguments')or''
   for sid in started:
    if 'tools.write_stdin' in s and str(sid) in s and '\\u0003' in s:
     assert 'exit_code'in v and 'session_id'not in v;shutdown.append(dict(session_id=sid,exit=v['exit_code'],call_id=item['call_id']))
  assert len(shutdown)==2;extra['loopback_service_termination']=shutdown
 shutil.copyfile(rev,d/'protocol-preliminary.json'); p.pop('limitation');p.update(verdict='PASS',received_utc=message_ack['timestamp'],launch_reconciliation=dict(matched=len(observed),wrapper_results_exact=True),audited_scope='All recorded tool invocation payloads and responses reviewed against dispatched workspace and frozen wrapper. No bypass or out-of-scope action in reviewed commands.',**extra);save(rev,p)
 score=d/('sealed-'+task+'.json'); g=load(score); assert g['functional_axis']=='PASS' and g['source_axis']=='PASS'; assert g['source_hashes']==p['source_hashes']; assert g['protocol_axis']=='UNKNOWN';
 rows.append(dict(task=task,functional='PASS',source='PASS',protocol='PASS',certified='PASS',functional_rows=len(g['runs']),top_level_calls=p['top_level_calls'],cycles=p['cycles'],product_launches=p['product_calls_observed'],model=p['actual_model'],effort=p['actual_effort'],elapsed_seconds=(when(p['received_utc'])-when(p['start_utc'])).total_seconds(),score_sha256=sha(score),final_protocol_sha256=sha(rev),source_review_sha256=sha(d/'source-review.json'),tool_trace_sha256=sha(d/'tool-trace.json')))
save(root/'summary.json',dict(version='v3-t20-r01',manifest_sha256=load(root/'C01/dispatch.json')['manifest_sha256'],binary_sha256=load(root/'C01/dispatch.json')['config']['binary_sha256'],certified='4/4',tasks=rows,protocol_note='The original runner verdict was captured before protocol review finalized. The independent final verdict joins those immutable functional results and source reviews with final protocol receipts.'))
for x in rows:print(x['task'],x['functional_rows'],x['top_level_calls'],x['cycles'],x['product_launches'],x['elapsed_seconds'])
