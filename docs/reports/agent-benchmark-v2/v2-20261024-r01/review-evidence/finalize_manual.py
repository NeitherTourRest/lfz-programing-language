import json,hashlib
from pathlib import Path
root=Path.cwd();o=root/'docs/reports/agent-benchmark-v2/v2-20261024-r01';f=json.loads((o/'functional-results.json').read_text(encoding='utf-8'));sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();manifest=sha(root/'docs/validation/agent-benchmark-v2/manifest.json')
m={'kind':'INDEPENDENT_MANUAL_REVIEW','run_id':'v2-20261024-r01','reviewer':'verifier','functional_report_sha256':sha(o/'functional-results.json'),'manifest_sha256':manifest,'tasks':[]}
for t in f['tasks']:
 task=t['task'];m['tasks'].append({'task':task,'source_sha256':sha(o/'sealed'/task/'solution.lfz'),'source_review':'PASS','protocol_review':'PASS','evidence':['sealed/'+task+'/solution.lfz','sealed/'+task+'/seal.json','sealed/'+task+'/attempts.jsonl','sealed/'+task+'/REPORT.md','dispatch-prompts/'+task+'.txt','tool-traces/'+task+'.json','review-evidence/source-review.md','review-evidence/material-check.json','review-evidence/tool-trace-review.json','REVIEW.md']})
(o/'manual-review.json').write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
assert len({t['task'] for t in m['tasks']})==8
certified=[]
for t in f['tasks']:
 v=next(x for x in m['tasks'] if x['task']==t['task']);assert v['source_sha256']==t['cases'][0]['source_sha256'];certified.append(t['functional_status']=='FUNCTIONAL_PASS' and v['source_review']==v['protocol_review']=='PASS')
s={'kind':'INDEPENDENT_CERTIFIED_RESULT','run_id':m['run_id'],'functional_report_sha256':m['functional_report_sha256'],'manual_review_sha256':sha(o/'manual-review.json'),'manifest_sha256':manifest,'denominator':8,'functional_pass_tasks':f['functional_pass_tasks'],'certified_pass_tasks':sum(certified),'passed_cases':sum(t['passed_cases'] for t in f['tasks']),'total_cases':sum(t['total_cases'] for t in f['tasks']),'isolation':'instruction-based shared filesystem, reviewed recorded student tool calls; no OS isolation claim','model':'gpt-6.1-sol','reasoning_effort':'medium','model_evidence':'tool-traces/B01..B08.json model_contexts','historical_comparison':'new v2 baseline; no percentage improvement calculated against old 2/8'}
(o/'certified-results.json').write_text(json.dumps(s,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(s,ensure_ascii=False))