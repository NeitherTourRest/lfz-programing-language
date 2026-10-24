import importlib.util,json,itertools,csv,io
from pathlib import Path
root=Path.cwd(); out=root/'Temp/verifier-benchmark-v2'; package=root/'docs/validation/agent-benchmark-v2'
s=importlib.util.spec_from_file_location('r',package/'run.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m);d=m.load_cases()
checks=[]
def check(name,ok,detail=None):
 checks.append({'name':name,'pass':bool(ok),'detail':detail})
# Independent ASCII scan preserves all non-ASCII separators before normalization.
for c in d['tasks']['B07']:
 text,qs=c['args']; words=[]; token=''
 for ch in text+' ':
  if 'a'<=ch<='z' or 'A'<=ch<='Z':token+=ch.lower()
  elif token:words.append(token);token=''
 result=[[q.lower(),sum(w==q.lower() for w in words),[i for i,w in enumerate(words) if w==q.lower()]] for q in qs]
 check(c['id']+' independent ASCII oracle',result==c['expected'])
 check(c['id']+' bounds',len(text)<=4096 and len(qs)<=32 and all(1<=len(q)<=64 and q.isascii() and q.isalpha() for q in qs))
# Exhaustive subset optimum with minimal binary mask encodes higher-index tie exclusion.
for c in d['tasks']['B05']:
 items,cap=c['args']; e=c['expected']; ix=e['indices']
 check(c['id']+' witness weight/value',ix==sorted(set(ix)) and sum(items[i][0] for i in ix)<=cap and sum(items[i][1] for i in ix)==e['value'])
 if len(items)<=12:
  best=max(((sum(v for j,(w,v) in enumerate(items) if mask>>j&1),-mask) for mask in range(1<<len(items)) if sum(w for j,(w,v) in enumerate(items) if mask>>j&1)<=cap))
  check(c['id']+' exhaustive optimum and tie',e=={'value':best[0],'indices':[j for j in range(len(items)) if (-best[1])>>j&1]})
 check(c['id']+' bounds',len(items)<=24 and 0<=cap<=80 and all(w>0 and v>=0 for w,v in items) and sum(v for w,v in items)<=2**63-1)
# LF-only split oracle does not share runner splitlines implementation.
for c in d['tasks']['B08']:
 total=0;rows=['index,value,running']; nums=[int(x.strip()) for x in c['args'][0].split('\n') if x.strip()]
 for i,n in enumerate(nums,1):total+=n;rows.append(f'{i},{n},{total}')
 expected='\n'.join(rows)+'\n'
 check(c['id']+' independent output oracle',expected==c['expected_output'] and c['expected']=={'count':len(nums),'total':total})
 check(c['id']+' LF/noBOM bounds',not expected.startswith('\ufeff') and '\r' not in expected and len(c['args'][0].splitlines())<=128)
# CSV domain and blank-line handling plus stdlib output cross check.
for c in d['tasks']['B02']:
 text=c['args'][0];actual=[] if not text else [row or [''] for row in csv.reader(io.StringIO(text,newline=''),strict=True)]
 check(c['id']+' CSV grammar expected',len(text)<=2048 and actual==c['expected'])
# Independent broad domain checks on fixed finite cases.
for c in d['tasks']['B03']:
 ops=c['args'][0];keys=[op[1] for op in ops if op[0]!='size'];check(c['id']+' bounds',len(ops)<=128 and len(set(keys))<=32 and all(len(k)<=128 for k in keys))
for c in d['tasks']['B04']:
 g,a,b=c['args'];check(c['id']+' bounds',1<=len(g)<=20 and 1<=len(g[0])<=20 and all(len(x)==len(g[0]) and set(x)<=set('.#') for x in g) and all(0<=p[0]<len(g) and 0<=p[1]<len(g[0]) and g[p[0]][p[1]]=='.' for p in [a,b]))
for c in d['tasks']['B06']:
 ops=c['args'][0];keys=[k for op in ops for k in (op[1:2] if op[0]=='deposit' else op[1:3])];check(c['id']+' bounds',len(ops)<=128 and all(len(k)<=128 for k in keys))
check('B07 initial regression fixed',m.reference('B07',['aKB',['a','b','akb']])==[['a',1,[0]],['b',1,[1]],['akb',0,[]]])
r={'kind':'INDEPENDENT_CASE_REVIEW_NOT_AGENT_SCORE','checks':checks,'total':len(checks),'passed':sum(x['pass'] for x in checks),'failed':[x for x in checks if not x['pass']]}
(out/'independent-case-review.json').write_text(json.dumps(r,ensure_ascii=False,indent=2),encoding='utf-8');print(json.dumps({'checks':r['total'],'passed':r['passed'],'failed':r['failed']},ensure_ascii=True))