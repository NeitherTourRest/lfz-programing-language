import json,hashlib,datetime
from pathlib import Path
root=Path.cwd();run=root/'Temp/agent-benchmark-v2/v2-20261024-r01';md=json.loads((run/'run-metadata.json').read_text(encoding='utf-8'));manifest=json.loads((root/'docs/validation/agent-benchmark-v2/manifest.json').read_text(encoding='utf-8'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def dt(s):return datetime.datetime.fromisoformat(s.replace('Z','+00:00'))
records=[]
for meta in md['tasks']:
 task=meta['task_id'];p=run/'sealed'/task
 if not (p/'solution.lfz').exists():continue
 seal=json.loads((p/'seal.json').read_text(encoding='utf-8'));source_sha=sha(p/'solution.lfz');rows=[json.loads(x) for x in (p/'attempts.jsonl').read_text(encoding='utf-8').splitlines() if x.strip()]
 rec={'task':task,'solution_sha256':source_sha,'matches_seal':source_sha==seal['solution_sha256'],'matches_metadata':source_sha==meta['solution_sha256'],'received_before_deadline':dt(meta['received_utc'])<=dt(meta['deadline_utc']),'elapsed_dispatch_budget_seconds':(dt(meta['received_utc'])-dt(meta['started_utc'])).total_seconds(),'attempt_count':len(rows),'within_five_recorded_calls':len(rows)<=5,'task_hash_matches':sha(p/'TASK.md').lower()==meta['task_sha256'].lower(),'prompt_hash_matches':sha(run/'dispatch-prompts'/(task+'.txt')).lower()==meta['prompt_sha256'].lower(),'interpreter_hash_matches':sha(run/task/'lfz.exe').lower()==meta['interpreter_sha256'].lower(),'skill_hash_matches':all(sha(run/task/'skill'/Path(name).name)==value for name,value in manifest['files'].items() if '.opencode/skills/' in name),'attempts':[]}
 for row in rows:
  snap=row.get('snapshot') or row.get('source_snapshot') or row.get('source');start=row.get('start_utc') or row.get('utc_start');end=row.get('end_utc') or row.get('utc_end');stdout=row.get('stdout') or row.get('stdout_file');stderr=row.get('stderr') or row.get('stderr_file');command=row['command'];test=row.get('executed_source') or row.get('test_file') or row.get('test') or command.split()[-1]
  sn=p/snap;t=p/test
  rec['attempts'].append({'record':row,'snapshot_sha256':sha(sn),'snapshot_matches_final':sha(sn)==source_sha,'snapshot_mtime_utc':datetime.datetime.fromtimestamp(sn.stat().st_mtime,datetime.timezone.utc).isoformat(),'snapshot_mtime_before_call':datetime.datetime.fromtimestamp(sn.stat().st_mtime,datetime.timezone.utc)<=dt(start),'test_file':test,'test_sha256':sha(t),'test_contains_snapshot_functions':sn.read_text(encoding='utf-8').strip() in t.read_text(encoding='utf-8'),'time_order_and_budget':dt(meta['started_utc'])<=dt(start)<=dt(end)<=dt(meta['received_utc'])<=dt(meta['deadline_utc']),'stdout_sha256':sha(p/stdout),'stderr_sha256':sha(p/stderr),'stdout':(p/stdout).read_text(encoding='utf-8'),'stderr':(p/stderr).read_text(encoding='utf-8'),'first_recorded_exit':row.get('exit_code')})
 rec['first_recorded_run_success']=rows[0].get('exit_code')==0 if rows else None
 records.append(rec)
result={'kind':'INDEPENDENT_RETAINED_MATERIAL_CHECK','run_id':md['run_id'],'run_metadata_sha256':sha(run/'run-metadata.json'),'records':records,'limit':'Recorded calls and provided artifacts only; complete external-access/tool invocation history unavailable. No claim of OS isolation.'}
(root/'Temp/verifier-formal-r01/material-check.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
for r in records:print(r['task'],r['attempt_count'],'hashes',all(r[k] for k in ['matches_seal','matches_metadata','task_hash_matches','prompt_hash_matches','interpreter_hash_matches','skill_hash_matches']),'deadline',r['received_before_deadline'],'snapshot/time',[(x['snapshot_matches_final'],x['test_contains_snapshot_functions'],x['snapshot_mtime_before_call'],x['time_order_and_budget']) for x in r['attempts']])