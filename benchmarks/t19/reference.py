"""Same JSONL aggregation task, no input hashing in the timed path."""
import json,sys
s=dict(rows=0,valid=0,bad=0,sum_id=0,sum_value=0,active=0,weighted_mod=0,first_id=None,last_id=None)
with open(sys.argv[1],encoding='utf-8',newline='') as f:
    for line in f:
        s['rows']+=1
        try:v=json.loads(line)
        except json.JSONDecodeError:s['bad']+=1;continue
        s['valid']+=1;s['sum_id']+=v['id'];s['sum_value']+=v['value'];s['active']+=int(v['active'])
        s['weighted_mod']=(s['weighted_mod']+(v['id']+1)*(v['value']+1001))%1000000007
        if s['first_id'] is None:s['first_id']=v['id']
        s['last_id']=v['id']
print(json.dumps(s,separators=(',',':')),flush=True)
if sys.argv[2]=='hold':sys.stdin.readline()