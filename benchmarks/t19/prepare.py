"""T19 deterministic bounded fixture preparation; no LFZ invocation/measurement."""
import argparse,hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
DEFAULT=ROOT/'Temp'/'T19-perf'
def row(i):
    # Deliberate invalid JSON is deterministic; every physical line remains UTF-8.
    if i%97==0:return ('{"broken":'+str(i)+'\n').encode()
    obj={'id':i,'value':(i%2001)-1000,'active':i%3==0,'payload':'x'*(32+i%113),'tag':'数据😀' if i%7==0 else 'plain'}
    return (json.dumps(obj,ensure_ascii=False,separators=(',',':'))+'\n').encode()
def generate(path,minimum):
    path.parent.mkdir(parents=True,exist_ok=True);n=0;size=0;sha=hashlib.sha256()
    with path.open('wb') as f:
        while size<minimum:
            b=row(n);f.write(b);sha.update(b);size+=len(b);n+=1
    return {'path':str(path.relative_to(ROOT)),'bytes':size,'rows':n,'sha256':sha.hexdigest()}
def oracle(path):
    out={'rows':0,'valid':0,'bad':0,'sum_id':0,'sum_value':0,'active':0,'weighted_mod':0,'first_id':None,'last_id':None,'max_raw_line_bytes':0};sha=hashlib.sha256();size=0
    with path.open('rb') as f:
        for raw in f:
            sha.update(raw);size+=len(raw);out['rows']+=1;out['max_raw_line_bytes']=max(out['max_raw_line_bytes'],len(raw)-1)
            try:value=json.loads(raw.decode('utf-8'))
            except json.JSONDecodeError:out['bad']+=1;continue
            out['valid']+=1;out['sum_id']+=value['id'];out['sum_value']+=value['value'];out['active']+=int(value['active'])
            out['weighted_mod']=(out['weighted_mod']+(value['id']+1)*(value['value']+1001))%1000000007
            if out['first_id'] is None:out['first_id']=value['id']
            out['last_id']=value['id']
    return dict(out,bytes=size,sha256=sha.hexdigest())
def main():
    p=argparse.ArgumentParser();p.add_argument('--prepare-only',action='store_true');p.add_argument('--output',type=Path,default=DEFAULT/'prepared.json');a=p.parse_args()
    if not a.prepare_only:p.error('Only --prepare-only is authorized in this preparation phase')
    output=a.output.resolve();output.relative_to(ROOT/'Temp')
    smoke=DEFAULT/'smoke.jsonl';made=generate(smoke,4096);checked=oracle(smoke)
    assert all(made[k]==checked[k] for k in ('rows','bytes','sha256'))
    result={'status':'prepared_only','product_invocations':0,'network_requests':0,'measured':False,'schema':1,'planned_mib':[10,100,300],'warmups':1,'formal_rounds':5,'smoke_generator':made,'independent_reparse_oracle':checked,'limits':{'max_line_bytes':4096,'row_distribution':'i%97 invalid; valid id/value/active/payload/Unicode periodic','physical_separator':'LF'},'measurement':'Future fixed-hash Windows new-process PeakWorkingSetSize via OS API; no sampling labelled peak; no R304 claim'}
    output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({'prepared':True,'bytes':made['bytes'],'rows':made['rows'],'product_invocations':0}))
if __name__=='__main__':main()