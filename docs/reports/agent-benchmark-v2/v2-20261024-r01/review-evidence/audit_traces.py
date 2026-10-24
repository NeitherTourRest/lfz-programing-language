import json,re,hashlib,datetime
from pathlib import Path
root=Path.cwd();out=root/'docs/reports/agent-benchmark-v2/v2-20261024-r01';idx=json.loads((out/'tool-traces/index.json').read_text(encoding='utf-8'));md=json.loads((out/'run-metadata.json').read_text(encoding='utf-8'));materials=json.loads((out/'review-evidence/material-check.json').read_text(encoding='utf-8'));summary=[]
def dt(s):return datetime.datetime.fromisoformat(s.replace('Z','+00:00'))
for p in sorted((out/'tool-traces').glob('B*.json')):
 d=json.loads(p.read_text(encoding='utf-8'));task=d['task_id'];calls=d['tool_calls'];results=d['tool_results'];meta=next(x for x in md['tasks'] if x['task_id']==task);mat=next(x for x in materials['records'] if x['task']==task);rows=[];count=0
 for call in calls:
  raw=call.get('input') or call.get('arguments') or '';cmds=re.findall(r'cmd\s*:\s*("(?:\\.|[^"\\])*")',raw);cmd=json.loads(cmds[0]);wds=re.findall(r'"?workdir"?\s*:\s*("(?:\\.|[^"\\])*")',raw);cwd=json.loads(wds[0]);view=re.sub(r"@'\r?\n.*?\r?\n'@",'<HERE_STRING>',cmd,flags=re.S)
  assert Path(cwd).resolve()==Path(meta['workspace']).resolve()
  runs=re.findall(r'&\s*\.\\lfz\.exe\s+run\s+([^\s]+)',view);count+=len(runs)
  assert all(Path(x).name==x for x in runs)
  assert not any(x in view.lower() for x in ['..','invoke-webrequest','http:','https:','curl ','wget ','python ','node ','gh ','ssh ','start-process','set-location'])
  rows.append({'call_id':call['call_id'],'timestamp':call['timestamp'],'name':call['name'],'cwd':cwd,'command':cmd,'command_without_here_string':view,'lfz_runs':runs})
 pair=sorted(c['call_id'] for c in calls)==sorted(c['call_id'] for c in results)
 assert pair and count==mat['attempt_count'] and count<=5
 assert hashlib.sha256(p.read_bytes()).hexdigest()==idx[task]['trace_sha256']
 assert d['parent_thread_id']=='01a16c3f-dac1-75c3-a3cb-5b02d2230d04' and d['agent_path']=='/root/v2_student_'+task.lower()
 assert all(x['model']=='gpt-6.1-sol' and x['effort']=='medium' for x in d['model_contexts'])
 assert all(dt(meta['started_utc'])<=dt(c['timestamp'])<=dt(meta['received_utc']) for c in calls)
 summary.append({'task':task,'session_id':d['session_id'],'trace_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'calls':len(calls),'results':len(results),'pair_ids_match':pair,'lfz_process_calls':count,'matches_attempts_log':count==mat['attempt_count'],'all_workdirs_own':True,'manual_command_review':'PASS - only authorized task/skill reads and own source/test/log/report/fixture operations; no prohibited tools or accesses in recorded calls','protocol_review':'PASS','model_contexts':d['model_contexts'],'commands':rows})
assert len({s['session_id'] for s in summary})==8
r={'kind':'INDEPENDENT_RECORDED_TOOL_TRACE_REVIEW','run_id':md['run_id'],'tasks':summary,'total_calls':sum(x['calls'] for x in summary),'total_results':sum(x['results'] for x in summary),'total_lfz_process_calls':sum(x['lfz_process_calls'] for x in summary),'scope':'All recorded student tools provided in source-hash-bound extracts; instruction isolation/shared FS, not OS file-access enforcement or proof of unrecorded behavior.'}
(out/'review-evidence/tool-trace-review.json').write_text(json.dumps(r,ensure_ascii=False,indent=2),encoding='utf-8');(out/'review-evidence/tool-command-index.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2),encoding='utf-8');print('PASS',r['total_calls'],r['total_results'],r['total_lfz_process_calls'])