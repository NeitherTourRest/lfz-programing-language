"""审计预算纯逻辑。必须由独立审计者从真实trace填入，不能采用学生自报。"""
def verdict(v):
 if not v.get('complete_trace')or not v.get('source_hash_bound')or not v.get('process_reconciled'):return 'UNKNOWN'
 if v['tools']>40 or v['cycles']>5 or v['products']>10 or v['elapsed']>1200 or not v['check_before_run'] or not v['access_compliant']:return 'FAIL'
 return 'PASS'
if __name__=='__main__':
 import json
 base=dict(complete_trace=True,source_hash_bound=True,process_reconciled=True,tools=40,cycles=5,products=10,elapsed=1200,check_before_run=True,access_compliant=True)
 tests=[('boundary',base,'PASS')]+[(k,base|{k:v},e)for k,v,e in [('tools',41,'FAIL'),('cycles',6,'FAIL'),('products',11,'FAIL'),('elapsed',1200.001,'FAIL'),('check_before_run',False,'FAIL'),('access_compliant',False,'FAIL'),('complete_trace',False,'UNKNOWN'),('process_reconciled',False,'UNKNOWN')]]
 rows=[dict(name=n,expected=e,actual=verdict(v))for n,v,e in tests];print(json.dumps(rows));assert all(r['actual']==r['expected']for r in rows)
