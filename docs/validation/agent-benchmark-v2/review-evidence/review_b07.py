import importlib.util, json
from pathlib import Path
root=Path.cwd()
p=root/'docs/validation/agent-benchmark-v2/run.py'
s=importlib.util.spec_from_file_location('bench_review',p); m=importlib.util.module_from_spec(s); s.loader.exec_module(m)
a=['aKB',['a','b','akb']]
actual=m.reference('B07',a)
r={'command':'python Temp/verifier-benchmark-v2/review_b07.py','args':a,'expected':[['a',1,[0]],['b',1,[1]],['akb',0,[]]],'actual':actual}
r['pass']=r['actual']==r['expected']
(root/'Temp/verifier-benchmark-v2/b07-initial.json').write_text(json.dumps(r,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(r,ensure_ascii=False))