"""T17 application budget checks; old app/oracles are read-only. No compilation."""
import argparse, datetime, hashlib, json, os, subprocess
from pathlib import Path
ROOT=Path.cwd()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--lfz',type=Path);ap.add_argument('--expected-sha');ap.add_argument('--output',type=Path);ap.add_argument('--prepare-only',action='store_true');a=ap.parse_args()
    app=ROOT/'app/jsonbatch_modules/main.lfz';sources={str(p.relative_to(ROOT)):sha(p) for p in sorted(app.parent.glob('*.lfz'))}
    if a.prepare_only:print(json.dumps({'prepared':True,'interpreter_processes':0,'source_hashes':sources}));return
    assert a.lfz and a.expected_sha and a.output
    exe=a.lfz.resolve();out=a.output.resolve();expected=a.expected_sha.lower()
    assert exe.is_relative_to(ROOT) and out.is_relative_to(ROOT) and not out.exists() and sha(exe)==expected
    work=ROOT/'Temp/T17-verifier/app40';work.mkdir(parents=True,exist_ok=True);out.parent.mkdir(parents=True,exist_ok=True)
    env=dict(os.environ,TEMP=str(work),TMP=str(work),PYTHONDONTWRITEBYTECODE='1')
    ip=work/'input.jsonl';op=work/'output.json';data=b'1\nbad\nnull\n';ip.write_bytes(data);rows=[]
    result={'date':'2026-10-30','binary_sha256':expected,'binary_bytes':exe.stat().st_size,'source_hashes':sources,'status':'running','rows':rows}
    def save():out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
    def call(name,args):
        assert sha(exe)==expected
        p=subprocess.run([str(exe),*map(str,args)],capture_output=True,cwd=ROOT,env=env,timeout=15)
        r={'name':name,'arguments':list(map(str,args)),'exit':p.returncode,'stdout':p.stdout.decode('utf8'),'stderr':p.stderr.decode('utf8'),'passed':False};rows.append(r);save();return p,r
    def passed(r):r['passed']=True;save()
    def check_limit(p,r,message):
        assert p.returncode==2 and not p.stderr,r
        o=json.loads(p.stdout);assert o['ok'] is False and o['error']=='ExecutionLimitError' and o['message']==message,r
        assert isinstance(o['line'],int) and o['line']>0 and isinstance(o['col'],int) and o['col']>0 and o['traceback'],r
        return o
    try:
        base=None
        for name,flags in [('unlimited',[]),('enough_steps',['--max-steps','100000']),('enough_time',['--timeout-ms','60000']),('both',['--max-steps','100000','--timeout-ms','60000'])]:
            op.write_bytes(b'KEEP');p,r=call(name,['run',*flags,app,'--',ip,op]);assert p.returncode==0 and not p.stderr,r
            doc=json.loads(op.read_bytes());assert (doc['total'],doc['passed'],doc['failed'],doc['moduleCalls'])==(3,2,1,3)
            assert [x['ok'] for x in doc['rows']]==[True,False,True] and [x['line'] for x in doc['rows']]==[1,2,3]
            assert doc['rows'][0]['value']==1 and doc['rows'][2]['value'] is None and doc['rows'][1]['error']['class']=='ValueError'
            if base is None:base=op.read_bytes()
            else:assert op.read_bytes()==base,'limits changed normal application result'
            assert json.loads(p.stdout)==dict(total=3,passed=2,failed=1,path=str(op)) and ip.read_bytes()==data;passed(r)
        for name,flags,msg in [('zero_steps',['--max-steps','0'],'执行步数预算已耗尽'),('one_step',['--max-steps','1'],'执行步数预算已耗尽'),('zero_timeout',['--timeout-ms','0'],'执行时间上限已到'),('both_zero',['--max-steps','0','--timeout-ms','0'],'执行时间上限已到')]:
            op.write_bytes(b'KEEP');p,r=call(name,['run','--json',*flags,app,'--',ip,op]);o=check_limit(p,r,msg)
            assert op.read_bytes()==b'KEEP' and ip.read_bytes()==data and o['file'].replace('\\','/').endswith('/common.lfz'),r;passed(r)
        side=work/'side-effect.txt';side.write_bytes(b'BEFORE')
        sideprobe=work/'side-effect-probe.lfz'
        sideprobe.write_text('#42\nimport common from "../../../app/jsonbatch_modules/common.lfz"\nwriteFile(argv()[0], "SIDE")\nwhile true {}\n',encoding='utf8',newline='\n')
        p,r=call('completed_io_not_rolled_back',['run','--json','--max-steps','100',sideprobe,'--',side]);check_limit(p,r,'执行步数预算已耗尽')
        assert side.read_bytes()==b'SIDE' and op.read_bytes()==b'KEEP';passed(r)
        # This probe imports the real parsing module and deliberately fails if its
        # business recovery returns. Find a deterministic step edge INSIDE the
        # actual zero-argument JSON callback; retain every attempted budget.
        probe=work/'attempt-budget-probe.lfz'
        probe.write_text('#42\nimport record from "../../../app/jsonbatch_modules/record.lfz"\nlet result = record.parseLine("bad", 1)\nfail("AFTER_ATTEMPT")\n',encoding='utf8',newline='\n')
        p,r=call('probe_unlimited',['run','--json',probe]);o=json.loads(p.stdout)
        assert p.returncode==2 and not p.stderr and o['error']=='AssertionError' and o['message']=='AFTER_ATTEMPT';passed(r)
        witness=None
        for n in range(129):
            p,r=call('probe_budget_'+str(n),['run','--json','--max-steps',str(n),probe]);o=check_limit(p,r,'执行步数预算已耗尽');passed(r)
            if o['file'].replace('\\','/').endswith('/record.lfz') and o['traceback'][-1]['func']=='<fn>':witness=n;break
        assert witness is not None,'no bounded callback budget witness found'
        result['callback_budget_witness']=witness
        # check rejects execution options before parsing or evaluating the app.
        p,r=call('check_budget_rejected',['check','--max-steps','10',app]);assert p.returncode==2 and not p.stdout and '执行限额选项只适用于运行命令' in r['stderr'];passed(r)
        assert ip.read_bytes()==data and op.read_bytes()==b'KEEP'
        assert sources=={str(p.relative_to(ROOT)):sha(p) for p in sorted(app.parent.glob('*.lfz'))} and sha(exe)==expected
        result.update(status='completed',total=len(rows),passed=sum(r['passed'] for r in rows));save();print(json.dumps({k:v for k,v in result.items() if k not in ('rows','source_hashes')}))
    except BaseException as e:result.update(status='failed',error=repr(e),total=len(rows),passed=sum(r['passed'] for r in rows));save();raise
if __name__=='__main__':main()
