import json, hashlib
from pathlib import Path
root=Path.cwd()
run=root/'Temp/agent-benchmark-v2/v2-20261024-r01'
out=root/'docs/reports/agent-benchmark-v2/v2-20261024-r01/tool-traces'
out.mkdir(parents=True,exist_ok=True)
parent='01a16c3f-dac1-75c3-a3cb-5b02d2230d04'
found={}
for path in Path('C:/Users/19170/.codex/sessions/2026/10/24').glob('rollout-2026-10-24T18-*.jsonl'):
    raw=path.read_bytes()
    rows=[]
    for line in raw.decode('utf-8').splitlines():
        try: rows.append(json.loads(line))
        except json.JSONDecodeError: continue
    meta=next((r.get('payload',{}) for r in rows if r.get('type')=='session_meta'),{})
    spawn=meta.get('source',{}).get('subagent',{}).get('thread_spawn',{}) if isinstance(meta.get('source'),dict) else {}
    agent=spawn.get('agent_path','')
    if spawn.get('parent_thread_id')!=parent or not agent.startswith('/root/v2_student_b'): continue
    task=agent.rsplit('_',1)[1].upper()
    if task not in ['B%02d'%i for i in range(1,9)]: continue
    calls=[]; results=[]; models=[]; completed=[]
    for row in rows:
        p=row.get('payload',{})
        if row.get('type')=='turn_context': models.append({k:p.get(k) for k in ('model','effort','cwd')})
        if row.get('type')=='response_item' and p.get('type') in ('function_call','custom_tool_call'):
            calls.append({'timestamp':row.get('timestamp'),'type':p.get('type'),'name':p.get('name'),'call_id':p.get('call_id'),'arguments':p.get('arguments'),'input':p.get('input')})
        if row.get('type')=='response_item' and p.get('type') in ('function_call_output','custom_tool_call_output'):
            results.append({'timestamp':row.get('timestamp'),'type':p.get('type'),'call_id':p.get('call_id'),'output':p.get('output')})
        if row.get('type')=='event_msg' and p.get('type')=='task_complete': completed.append(row.get('timestamp'))
    doc={'task_id':task,'agent_path':agent,'session_id':meta.get('id'),'parent_thread_id':parent,'session_start_utc':meta.get('timestamp'),'source_session_sha256':hashlib.sha256(raw).hexdigest(),'source_session_bytes':len(raw),'model_contexts':models,'tool_calls':calls,'tool_results':results,'task_completed_utc':completed,'coverage':'All recorded tool-call and tool-result response items in this student session. Excludes private reasoning, system/developer messages, user context and unrelated sessions. This is recorded tool access, not an OS file-access trace.'}
    if task in found: raise RuntimeError('duplicate student session '+task)
    (out/(task+'.json')).write_text(json.dumps(doc,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    found[task]={'session_id':meta.get('id'),'models':models,'calls':len(calls),'results':len(results),'completed':completed,'trace_sha256':hashlib.sha256((out/(task+'.json')).read_bytes()).hexdigest()}
assert len(found)==8,sorted(found)
(out/'index.json').write_text(json.dumps(found,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(found,ensure_ascii=False))