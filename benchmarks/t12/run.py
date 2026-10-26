"""T12 bounded whole-process benchmark; stdlib only, no source mutation."""
import argparse, datetime, hashlib, json, os, pathlib, platform, statistics, subprocess, sys, time
ROOT = pathlib.Path(__file__).resolve().parents[2]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--lfz', type=pathlib.Path, required=True)
    ap.add_argument('--label', required=True)
    ap.add_argument('--suite', choices=['standard','small'], default='standard')
    ap.add_argument('--output', type=pathlib.Path, required=True)
    ap.add_argument('--runs',type=int,default=7)
    ap.add_argument('--warmup',type=int,default=2)
    args=ap.parse_args()
    binary=args.lfz.resolve(); output=args.output.resolve()
    if not output.is_relative_to(ROOT) or output.exists():
        raise SystemExit('output must be a new project-local path')
    temp=ROOT/'Temp/t12-perf'; temp.mkdir(parents=True,exist_ok=True)
    env=os.environ.copy(); env.update(TEMP=str(temp),TMP=str(temp),PYTHONDONTWRITEBYTECODE='1')
    cases=[]
    def add(name,n,source,expected,stdin=b''):
        path=temp/(name+('_'+str(n) if n else '')+'.lfz')
        path.write_text(source,encoding='utf-8',newline='\n')
        cases.append((name,n,path,stdin,expected))
    add('noop',0,'#42\nprint("RESULT ok=1")\n','RESULT ok=1\n')
    for name,sizes in [('numeric_loop',[10000,100000,500000]),('array_builtins',[1000,10000,50000]),('struct_ops',[2000,4000,8000]),('string_ops',[2000,4000,8000])]:
        source=(ROOT/'benchmarks/lfz'/f'{name}.lfz').read_text(encoding='utf-8')
        for n in sizes:
            if name=='numeric_loop': expected=f'RESULT total={n*(n+1)//2}\n'
            elif name=='array_builtins': expected=f'RESULT total={sum((i*7%n) for i in range(n) if (i*7%n)%2==0)}\n'
            elif name=='struct_ops': expected=f'RESULT keys={n} total={n*(n-1)*(2*n-1)//6}\n'
            else: expected=f'RESULT chars={sum(len("item"+str(i)) for i in range(n))+n-1} words={n} tag=ababab\n'
            add(name,n,source,expected,f'{n}\n'.encode())
    for n in [2000,4000,8000]:
        source='#42\nlet n = int(input())\nlet parts = range(n) |> map((i) => "  item${i}  ")\nlet cleaned = parts |> map((s) => trim(s))\nlet joined = join(",", cleaned)\nlet words = split(",", replace("item", "ITEM", joined))\nlet tag = repeat(3, "ab")\nprint("RESULT chars=${len(joined)} words=${len(words)} tag=${tag}")\n'
        expected=f'RESULT chars={sum(len("item"+str(i)) for i in range(n))+n-1} words={n} tag=ababab\n'
        add('string_map',n,source,expected,f'{n}\n'.encode())
    if args.suite == 'small':
        cases=[]
        for n in [4,16,64]:
            source='#42\nlet n = int(input())\nvar total = 0\nvar j = 0\nwhile j < 1000 {\n var s = {}\n var i = 0\n while i < n { s["k${i}"] = i * i\n i += 1 }\n for k in keys(s) { total += s[k] }\n j += 1\n}\nprint("RESULT total=${total}")\n'
            add('struct_small',n,source,f'RESULT total={1000*n*(n-1)*(2*n-1)//6}\n',f'{n}\n'.encode())
    result={'label':args.label,'timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'environment':{'os':platform.platform(),'cpu':platform.processor(),'logical_cpus':os.cpu_count(),'python':sys.version,'python_exe':sys.executable},'binary':{'path':str(binary.relative_to(ROOT)),'sha256':sha(binary),'bytes':binary.stat().st_size},'method':{'clock':'perf_counter_ns','whole_process':True,'warmup':args.warmup,'runs':args.runs,'timeout_seconds':15,'background_load_controlled':False,'note':'sequential case order; total includes process startup, parsing, evaluation and output capture'},'cases':[]}
    for name,n,path,stdin,expected in cases:
        samples=[]; runs=[]
        for i in range(args.warmup+args.runs):
            start=time.perf_counter_ns()
            p=subprocess.run([str(binary),'run',str(path)],input=stdin,stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=ROOT,env=env,timeout=15)
            elapsed=(time.perf_counter_ns()-start)/1e6
            actual=p.stdout.decode('utf-8').replace('\r\n','\n')
            if p.returncode or actual!=expected:
                raise RuntimeError((name,n,p.returncode,actual,expected,p.stderr.decode('utf-8','replace')))
            if i>=args.warmup:
                samples.append(elapsed); runs.append({'elapsed_ms':elapsed,'exit':p.returncode,'stdout':actual,'stderr':p.stderr.decode('utf-8','replace')})
        row={'name':name,'n':n,'source_sha256':sha(path),'source':path.read_text(encoding='utf-8'),'stdin':stdin.decode(),'command':[str(binary.relative_to(ROOT)),'run',str(path.relative_to(ROOT))],'expected_stdout':expected,'median_ms':statistics.median(samples),'min_ms':min(samples),'max_ms':max(samples),'runs':runs}
        result['cases'].append(row)
        print(f'{name:16} {n:7} {row["median_ms"]:10.3f} ms')
    output.parent.mkdir(parents=True,exist_ok=True)
    output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
    print('all outputs matched; raw:',output.relative_to(ROOT))
if __name__=='__main__': main()