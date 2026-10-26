"""T12 new JSON/CLI capabilities: whole-process timing, independent output oracle."""
import argparse, datetime, hashlib, json, os, pathlib, platform, statistics, subprocess, sys, time
from affinity_control import run as run_with_affinity
ROOT=pathlib.Path(__file__).resolve().parents[2]
def main(affinity):
    ap=argparse.ArgumentParser(); ap.add_argument('--lfz',type=pathlib.Path,required=True); ap.add_argument('--output',type=pathlib.Path,required=True); ap.add_argument('--cpu',type=int,default=None); args=ap.parse_args()
    binary=args.lfz.resolve(); out=args.output.resolve(); assert out.is_relative_to(ROOT) and not out.exists()
    expected_hash=hashlib.sha256(binary.read_bytes()).hexdigest()
    temp=ROOT/'Temp/t12-perf'; temp.mkdir(parents=True,exist_ok=True)
    env=os.environ.copy(); env.update(TEMP=str(temp),TMP=str(temp),PYTHONDONTWRITEBYTECODE='1')
    cases=[]
    def add(name,n,command,stdin,expected,**meta): cases.append((name,n,command,stdin,expected,meta))
    for n in [2000,4000,8000]:
        obj={f'k{i}':i for i in range(n)}; data=json.dumps(obj,ensure_ascii=False,separators=(',',':'),sort_keys=True).encode()
        input_path=temp/f'json-{n}.json'; input_path.write_bytes(data)
        path_literal=json.dumps(input_path.relative_to(ROOT).as_posix())
        for mode in ['parse','stringify']:
            prefix=f'#42\nlet text = readFile({path_literal})\n'
            if mode=='stringify': prefix+='let value = jsonParse(text)\n'
            operation=f'let value = jsonParse(text)\n total += value["k{n-1}"]' if mode=='parse' else 'total += len(jsonStringify(value))'
            source=prefix+'var total = 0\nvar j = 0\nwhile j < 20 {\n '+operation+'\n j += 1\n}\nprint("RESULT total=${total}")\n'
            path=temp/f'json-{mode}-{n}.lfz'; path.write_text(source,encoding='utf-8',newline='\n')
            expected=f'RESULT total={20*(n-1 if mode=="parse" else len(data))}\n'
            add('json_'+mode,n,[str(binary),'run',str(path)],b'',expected,input_bytes=len(data),input_sha256=hashlib.sha256(data).hexdigest(),canonical_output_bytes=len(data),iterations=20,source=source)
    noop='print("RESULT ok=1")\n'; expected='RESULT ok=1\n'
    add('cli_eval',0,[str(binary),'-e',noop],b'',expected,source=noop)
    add('cli_stdin',0,[str(binary),'-'],noop.encode(),expected,source=noop)
    source='print(jsonStringify(argv()))\n'; script_args=['literal','中文','--json','-e','']
    add('cli_argv',0,[str(binary),'-e',source,'--']+script_args,b'',json.dumps(script_args,ensure_ascii=False,separators=(',',':'))+'\n',source=source)
    result={'timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'binary_bytes':binary.stat().st_size,'method':{'clock':'perf_counter_ns','whole_process':True,'warmup':2,'runs':7,'timeout_seconds':15,'json_iterations':20,'peak_memory_measured':False,'sha_verified_before_each_launch':True,'oracle':'Python stdlib json canonical output byte length and expected field value; ASCII JSON keys and integer values'},'environment':{'os':platform.platform(),'cpu':platform.processor(),'python':sys.version},'cases':[]}
    for name,n,command,stdin,expected,meta in cases:
        runs=[]
        for i in range(9):
            assert hashlib.sha256(binary.read_bytes()).hexdigest()==expected_hash
            start=time.perf_counter_ns(); p=subprocess.run(command,input=stdin,cwd=ROOT,env=env,capture_output=True,timeout=15); elapsed=(time.perf_counter_ns()-start)/1e6
            actual=p.stdout.decode('utf-8').replace('\r\n','\n')
            assert p.returncode==0 and actual==expected,(name,n,p.returncode,actual,expected,p.stderr)
            if i>=2: runs.append({'sha256':expected_hash,'elapsed_ms':elapsed,'stdout':actual,'stderr':p.stderr.decode('utf-8','replace'),'exit':p.returncode})
        vals=[v['elapsed_ms'] for v in runs]
        row={'name':name,'n':n,'command':command,'stdin':stdin.decode(),'expected_stdout':expected,**meta,'median_ms':statistics.median(vals),'min_ms':min(vals),'max_ms':max(vals),'runs':runs}
        result['cases'].append(row); print(name,n,'input bytes',meta.get('input_bytes'),round(row['median_ms'],3),'ms')
    out.parent.mkdir(parents=True,exist_ok=True); out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
    return out
if __name__=='__main__': run_with_affinity(main)