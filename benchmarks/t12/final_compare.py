"""T12 final same-path comparison. Copy time is excluded; bytes verified each run."""
import argparse, datetime, hashlib, json, os, pathlib, platform, shutil, statistics, subprocess, sys, time
from affinity_control import run as run_with_affinity
from process_measure import capture
ROOT=pathlib.Path(__file__).resolve().parents[2]

def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def main(affinity):
    ap=argparse.ArgumentParser(); ap.add_argument('--output',type=pathlib.Path,required=True); ap.add_argument('--new-binary',type=pathlib.Path,required=True); ap.add_argument('--expected-new-sha',required=True); ap.add_argument('--cpu',type=int,default=None); ap.add_argument('--settle-seconds',type=float,default=0); args=ap.parse_args()
    output=args.output.resolve(); assert output.is_relative_to(ROOT) and not output.exists()
    temp=ROOT/'Temp/T12'; temp.mkdir(parents=True,exist_ok=True)
    subject=temp/'perf-subject.exe'
    binaries={'old':ROOT/'Temp/t12-perf/lfz-before.exe','new':args.new_binary.resolve()}
    assert binaries['new'].is_relative_to(ROOT) and binaries['new'] != subject
    assert len(args.expected_new_sha)==64 and all(c in '0123456789abcdef' for c in args.expected_new_sha.lower())
    hashes={k:digest(v) for k,v in binaries.items()}
    assert hashes=={'old':'149b184e442ffbf250590431fa0aeffab401eac526faf60b4b435f620725d5ab','new':args.expected_new_sha.lower()}
    env=os.environ.copy(); env.update(TEMP=str(temp),TMP=str(temp),PYTHONDONTWRITEBYTECODE='1')
    baseline=[]
    for file in ['before.json','before-small.json']:
        baseline+=json.loads((ROOT/'benchmarks/t12/results'/file).read_text(encoding='utf-8'))['cases']
    result={'timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'environment':{'os':platform.platform(),'cpu':platform.processor(),'logical_cpus':os.cpu_count(),'python':sys.version},'binary_sources':{k:{'path':str(v.relative_to(ROOT)),'sha256':hashes[k],'bytes':v.stat().st_size} for k,v in binaries.items()},'method':{'execution_path':str(subject.relative_to(ROOT)),'clock':'perf_counter_ns','whole_process':True,'settle_after_each_copy_seconds':args.settle_seconds,'cpu_accounting':'GetProcessTimes user/kernel milliseconds; per-process resolution affects small values','warmup_after_each_copy':2,'runs':7,'timeout_seconds':15,'copy_excluded_from_timing':True,'sha_verified_before_each_launch':True,'case_order':'baseline order; old/new group order alternates by case; new left in common path afterwards','thresholds':'unchanged original before.json/before-small.json: max(before*1.2,before+5ms); noop factor1.3'},'cases':[]}
    for idx,row in enumerate(baseline):
        path=ROOT/row['command'][-1]; path.write_text(row['source'],encoding='utf-8',newline='\n')
        groups={}
        for label in (['old','new'] if idx%2==0 else ['new','old']):
            shutil.copyfile(binaries[label],subject); assert digest(subject)==hashes[label]
            time.sleep(args.settle_seconds)
            runs=[]
            for i in range(9):
                assert digest(subject)==hashes[label]
                p,ms,cpu=capture([str(subject),'run',str(path)],cwd=ROOT,env=env,input=row['stdin'].encode(),timeout=15)
                actual=p.stdout.decode('utf-8').replace('\r\n','\n'); assert p.returncode==0 and actual==row['expected_stdout'],(label,row['name'],p.returncode,actual)
                if i>=2: runs.append({'elapsed_ms':ms,**cpu,'sha256':hashes[label],'exit':p.returncode,'stdout':actual,'stderr':p.stderr.decode('utf-8','replace')})
            vals=[r['elapsed_ms'] for r in runs]; groups[label]={'median_ms':statistics.median(vals),'min_ms':min(vals),'max_ms':max(vals),'runs':runs,'median_user_ms':statistics.median(r['user_ms'] for r in runs),'median_kernel_ms':statistics.median(r['kernel_ms'] for r in runs)}
        threshold=max(row['median_ms']*(1.3 if row['name']=='noop' else 1.2),row['median_ms']+5)
        item={'name':row['name'],'n':row['n'],'source':row['source'],'source_sha256':digest(path),'stdin':row['stdin'],'expected_stdout':row['expected_stdout'],'original_before_ms':row['median_ms'],'original_fixed_limit_ms':threshold,'groups':groups,'new_pass_fixed_limit':groups['new']['median_ms']<=threshold,'contemporary_old_within_original_limit':groups['old']['median_ms']<=threshold,'new_over_same_path_old':groups['new']['median_ms']/groups['old']['median_ms']}
        result['cases'].append(item); print(row['name'],row['n'],'same-old',round(groups['old']['median_ms'],3),'same-new',round(groups['new']['median_ms'],3),'original_limit',round(threshold,3),'PASS' if item['new_pass_fixed_limit'] else 'FAIL',flush=True)
    probe=next(r for r in baseline if r['name']=='numeric_loop' and r['n']==500000); path=ROOT/probe['command'][-1]
    runs=[]
    for i in range(9):
        assert digest(binaries['new'])==hashes['new']; p,ms,cpu=capture([str(binaries['new']),'run',str(path)],cwd=ROOT,env=env,input=probe['stdin'].encode(),timeout=15)
        actual=p.stdout.decode('utf-8').replace('\r\n','\n'); assert p.returncode==0 and actual==probe['expected_stdout']
        if i>=2: runs.append({'elapsed_ms':ms,**cpu,'exit':p.returncode,'stdout':actual,'stderr':p.stderr.decode('utf-8','replace')})
    result['dist_path_probe']={'command':[str(binaries['new'].relative_to(ROOT)),'run',probe['command'][-1]],'n':500000,'median_ms':statistics.median(r['elapsed_ms'] for r in runs),'runs':runs,'sha256':hashes['new']}
    result['all_19_fixed_limits_pass']=all(r['new_pass_fixed_limit'] for r in result['cases'])
    struct8=next(r for r in result['cases'] if r['name']=='struct_ops' and r['n']==8000)
    result['struct8_same_path_reduction_fraction']=1-struct8['groups']['new']['median_ms']/struct8['groups']['old']['median_ms']
    result['struct8_original_30pct_target_pass']=struct8['groups']['new']['median_ms']<=struct8['original_before_ms']*.7
    shutil.copyfile(binaries['new'],subject); assert digest(subject)==hashes['new']
    output.parent.mkdir(parents=True,exist_ok=True); output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
    print('dist500k',result['dist_path_probe']['median_ms'],'all19',result['all_19_fixed_limits_pass'])
    return output
if __name__=='__main__': run_with_affinity(main)