"""评分私有纯Python预期；不提供学生。"""
import json
def jsonl(raw):
 parts=raw.split(b'\n');parts=parts[:-1]if raw.endswith(b'\n')else parts
 if not raw:parts=[]
 rows=[];bad=0;total=0
 for n,b in enumerate(parts,1):
  if len(b)>4096:return {'error':'ValueError','committed':rows[:len(rows)//32*32]}
  if b.endswith(b'\r') and (n<len(parts)or raw.endswith(b'\n')):b=b[:-1]
  try:s=b.decode('utf8')
  except UnicodeError:return {'error':'ValueError','committed':rows[:len(rows)//32*32]}
  try:v=json.loads(s)
  except (ValueError,TypeError):bad+=1;continue
  if not isinstance(v,dict)or not isinstance(v.get('account'),str)or not v['account']or type(v.get('delta'))is not int:bad+=1;continue
  rows.append(dict(account=v['account'],delta=v['delta'],lineNo=n));total+=v['delta']
 return {'summary':dict(accepted=len(rows),rejected=bad,sum=total),'rows':rows}
def jobs(values):return dict(items=[dict(ok=type(v)is int,value=v if type(v)is int else None,error=None if type(v)is int else 'AssertionError')for v in values],count=len(values),nilOk=True)
def quote(q,args):return dict(a=q*7,b=14,c=33,snapshot=99,history=[q*7,14,33],args=args)
def http(enabled):
 if not enabled:return [{'error':'ValueError'}]*5
 return [dict(status=200,body='雪',repeated=['one','two']),dict(status=404,body='absent',repeated=[]),dict(status=302,body='',repeated=[]),dict(error='ValueError'),dict(status=200,body='雪',repeated=['one','two'])]
