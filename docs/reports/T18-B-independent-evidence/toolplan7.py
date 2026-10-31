"""Local fixture only; argv arrays, no shell, no compilation. Preserve every run."""
import argparse,hashlib,json,os,shutil,subprocess
from pathlib import Path
ROOT=Path.cwd()
FIXTURE_SHA='51b83f054f128e381fa034adbe980d6398f24f76621703ba27da4d7e54d27bee'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--lfz',type=Path,required=True);ap.add_argument('--expected-sha',required=True);ap.add_argument('--output',type=Path,required=True);a=ap.parse_args()
    exe=a.lfz.resolve();out=a.output.resolve();expected=a.expected_sha.lower();app=ROOT/'app/toolplan.lfz'
    assert exe.is_relative_to(ROOT) and out.is_relative_to(ROOT) and not out.exists() and sha(exe)==expected
    w=ROOT/'Temp/T18-B-verifier/toolplan';w.mkdir(parents=True,exist_ok=True);out.parent.mkdir(parents=True,exist_ok=True)
    src=ROOT/'Temp/T18-A-tooling/fixture.exe';fixture=w/'fixture.exe';assert sha(src)==FIXTURE_SHA
    shutil.copyfile(src,fixture);assert sha(fixture)==FIXTURE_SHA
    cfg=w/'plan.json';output=w/'report.json';marker=w/'marker.txt';late=w/'late-marker.txt';rows=[]
    env=dict(os.environ,TEMP=str(w),TMP=str(w),PYTHONDONTWRITEBYTECODE='1')
    result={'date':'2026-10-30','binary_sha256':expected,'fixture_sha256':FIXTURE_SHA,'source_sha256':sha(app),'status':'running','rows':rows}
    def save():out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8',newline='\n')
    def job(name,mode,args=(),timeout=5000,cap=4096):return dict(name=name,argv=[str(fixture),mode,*map(str,args)],timeoutMs=timeout,maxOutputBytes=cap)
    def run(name,jobs,flags=(),expected_code=0,json_mode=False):
        cfg.write_text(json.dumps(jobs,ensure_ascii=False),encoding='utf8');before=cfg.read_bytes();output.write_bytes(b'KEEP')
        assert sha(exe)==expected and sha(fixture)==FIXTURE_SHA
        command=[str(exe),'run',*flags,*(['--json'] if json_mode else []),str(app),'--',str(cfg),str(output)]
        try:p=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,timeout=12,creationflags=0x08000000 if os.name=='nt' else 0)
        except subprocess.TimeoutExpired as error:
            rows.append({'name':name,'timeout':True,'stdout':(error.stdout or b'').hex(),'stderr':(error.stderr or b'').hex(),'passed':False});save();raise
        r={'name':name,'command':command,'exit':p.returncode,'stdout':p.stdout.decode('utf8'),'stderr':p.stderr.decode('utf8'),'output_hex':output.read_bytes().hex(),'passed':False};rows.append(r);save()
        assert p.returncode==expected_code and cfg.read_bytes()==before,r
        return p,r
    def done(r):r['passed']=True;save()
    try:
        if marker.exists():marker.unlink()
        p,r=run('disabled_no_spawn',[job('marker','mark',[marker],cap=0)],expected_code=2,json_mode=True)
        assert not marker.exists() and output.read_bytes()==b'KEEP';assert json.loads(p.stdout)['error']=='AssertionError' and '需要 --allow-process' in p.stdout.decode();done(r)
        echo_args=['','a b','中文🙂','"quote"','tail\\']
        jobs=[job('args','echo',echo_args),job('nonzero','exit',['7'],cap=0),job('dual','dual',['4'],cap=8),job('too_much','dual',['4'],cap=7),job('local_timeout','sleep',['1000'],timeout=20,cap=0),job('invalid_utf8','invalid',cap=1),job('eof','stdin',cap=1),job('marker','mark',[marker],cap=0),job('after_errors','echo',['after'])]
        expected_rows=[(True,{'stdout':''.join(f'{len(s.encode())}:{s}\n' for s in echo_args),'stderr':'','exitCode':0,'signal':None}),(True,{'stdout':'','stderr':'','exitCode':7,'signal':None}),(True,{'stdout':'xxxx','stderr':'xxxx','exitCode':0,'signal':None}),(False,'子进程输出超过字节上限'),(False,'子进程执行时间上限已到'),(False,'子进程输出不是合法 UTF-8'),(True,{'stdout':'0','stderr':'','exitCode':0,'signal':None}),(True,{'stdout':'','stderr':'','exitCode':0,'signal':None}),(True,{'stdout':'5:after\n','stderr':'','exitCode':0,'signal':None})]
        stable=None
        for suffix in ['first','repeat']:
            p,r=run('orchestration_'+suffix,jobs,['--allow-process']);assert not p.stderr,r
            doc=json.loads(output.read_bytes());assert (doc['total'],doc['succeeded'],doc['failed'])==(9,6,3) and len(doc['rows'])==9
            for jobdef,item,(ok,value) in zip(jobs,doc['rows'],expected_rows):
                assert item['name']==jobdef['name'] and item['ok'] is ok
                if ok:assert item['error'] is None and item['value']==value
                else:assert item['value'] is None and item['error']['class']=='ValueError' and item['error']['message']==value
            assert marker.read_bytes()==b'ran' and json.loads(p.stdout)==dict(total=9,succeeded=6,failed=3,path=str(output))
            if stable is None:stable=output.read_bytes()
            else:assert stable==output.read_bytes(),'nondeterministic task report'
            r['report']=doc;done(r)
        p,r=run('enabled_does_not_leak',[job('marker','mark',[late],cap=0)],expected_code=2,json_mode=True)
        assert not late.exists() and output.read_bytes()==b'KEEP' and json.loads(p.stdout)['error']=='AssertionError';done(r)
        p,r=run('global_deadline_not_caught',[job('wait','sleep',['1000'],timeout=5000,cap=0),job('must_not_run','mark',[late],cap=0)],['--allow-process','--timeout-ms','40'],expected_code=2,json_mode=True)
        o=json.loads(p.stdout);assert o['error']=='ExecutionLimitError' and o['message']=='执行时间上限已到' and not p.stderr
        exec_line=next(i for i,text in enumerate(app.read_text(encoding='utf8').splitlines(),1) if 'exec(job.argv' in text)
        assert o['line']==exec_line and o['traceback'][-1]['func']=='<fn>'
        assert output.read_bytes()==b'KEEP' and not late.exists();done(r)
        p,r=run('prevalidate_all_jobs',[job('must_not_run','mark',[late],cap=0),{'name':'bad'}],['--allow-process'],expected_code=2,json_mode=True)
        assert json.loads(p.stdout)['error']=='AssertionError' and output.read_bytes()==b'KEEP' and not late.exists();done(r)
        p,r=run('empty_plan',[],['--allow-process']);assert json.loads(output.read_bytes())==dict(total=0,succeeded=0,failed=0,rows=[]);done(r)
        assert sha(exe)==expected and sha(app)==result['source_sha256'] and sha(fixture)==FIXTURE_SHA
        result.update(status='completed',total=len(rows),passed=sum(r['passed'] for r in rows));save();print(json.dumps({k:v for k,v in result.items() if k!='rows'}))
    except BaseException as error:result.update(status='failed',error=repr(error),total=len(rows),passed=sum(r['passed'] for r in rows));save();raise
if __name__=='__main__':main()
