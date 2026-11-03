"""T19 fixed-window runner. --prepare-only launches no product; --run needs parent GO."""
import argparse,ctypes,hashlib,json,os,statistics,subprocess,sys,threading,time
from pathlib import Path
from prepare import ROOT,DEFAULT,generate,oracle
EXPECTED='5d2219f88db8b3196cac259941fc0a8a3b489ef03519bc7e20415f8edf6388e5'
KEYS=('rows','valid','bad','sum_id','sum_value','active','weighted_mod','first_id','last_id')
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def inside(path):path=path.resolve();path.relative_to(ROOT/'Temp');return path
def peak(pid):
    from ctypes import wintypes as w
    class Counters(ctypes.Structure):
        _fields_=[('cb',w.DWORD),('PageFaultCount',w.DWORD),('PeakWorkingSetSize',ctypes.c_size_t),('WorkingSetSize',ctypes.c_size_t),('QuotaPeakPagedPoolUsage',ctypes.c_size_t),('QuotaPagedPoolUsage',ctypes.c_size_t),('QuotaPeakNonPagedPoolUsage',ctypes.c_size_t),('QuotaNonPagedPoolUsage',ctypes.c_size_t),('PagefileUsage',ctypes.c_size_t),('PeakPagefileUsage',ctypes.c_size_t)]
    k=ctypes.WinDLL('kernel32',use_last_error=True);api=ctypes.WinDLL('psapi',use_last_error=True)
    k.OpenProcess.argtypes=[w.DWORD,w.BOOL,w.DWORD];k.OpenProcess.restype=w.HANDLE;k.CloseHandle.argtypes=[w.HANDLE]
    api.GetProcessMemoryInfo.argtypes=[w.HANDLE,ctypes.POINTER(Counters),w.DWORD];api.GetProcessMemoryInfo.restype=w.BOOL
    h=k.OpenProcess(0x410,False,pid)
    if not h:raise ctypes.WinError(ctypes.get_last_error())
    try:
        c=Counters();c.cb=ctypes.sizeof(c)
        if not api.GetProcessMemoryInfo(h,ctypes.byref(c),c.cb):raise ctypes.WinError(ctypes.get_last_error())
        return {'peak_working_set_bytes':c.PeakWorkingSetSize,'working_set_bytes_at_marker':c.WorkingSetSize,'pagefile_bytes_at_marker':c.PagefileUsage}
    finally:k.CloseHandle(h)
def sample(command,expected,output,timeout):
    begin=time.perf_counter_ns();state={'command':command,'watchdog':False};p=None
    with output.with_suffix('.stderr').open('wb') as err:
        p=subprocess.Popen(command,cwd=ROOT,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err)
        def stop():
            state['watchdog']=True
            try:p.kill()
            except OSError:pass
        timer=threading.Timer(timeout,stop);timer.start()
        try:
            marker=p.stdout.readline();observed=time.perf_counter_ns()
            state.update(pid=p.pid,stdout_marker=marker.decode('utf-8',errors='replace'),processing_wall_ns=observed-begin)
            if not marker:raise RuntimeError('missing result marker')
            actual=json.loads(marker)
            if actual!={k:expected[k] for k in KEYS}:raise AssertionError({'expected':expected,'actual':actual})
            if p.poll() is not None:raise RuntimeError('child exited before live OS peak observation')
            state.update(peak(p.pid));p.stdin.write(b'\n');p.stdin.flush();p.stdin.close()
            tail=p.stdout.read();state['stdout_tail']=tail.decode('utf-8',errors='replace');state['exit']=p.wait(timeout=timeout)
            state['total_exit_wall_ns']=time.perf_counter_ns()-begin
            if state['exit'] or tail or state['watchdog']:raise RuntimeError('unexpected exit/tail/watchdog')
            state['ok']=True
        except Exception as e:
            state.update(ok=False,error=repr(e))
            if p.poll() is None:p.kill()
            p.wait(timeout=10)
        finally:
            timer.cancel();p.stdout.close()
            if not p.stdin.closed:p.stdin.close()
            state['stderr']=output.with_suffix('.stderr').read_text(encoding='utf-8',errors='replace')
            output.write_text(json.dumps(state,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    if not state['ok']:raise RuntimeError('sample failed; retained '+str(output))
    return state

def main():
    p=argparse.ArgumentParser();p.add_argument('--prepare-only',action='store_true');p.add_argument('--run',action='store_true');p.add_argument('--binary',type=Path);p.add_argument('--expected-sha',default=EXPECTED);p.add_argument('--root',type=Path,default=DEFAULT/'window01');p.add_argument('--timeout',type=float,default=600);a=p.parse_args()
    if a.prepare_only==a.run:p.error('choose --prepare-only OR --run')
    work=inside(a.root);work.mkdir(parents=True,exist_ok=True)
    protocol={'expected_sha':a.expected_sha,'sizes_mib':[10,100,300],'warmups':1,'formal_rounds':5,'peak':'Windows GetProcessMemoryInfo live handshake; physical working set OS peak','wall':'startup through parent-observed JSON marker; observer wait excluded; total exit separate','baseline':'whole/readFile only10MiB','product_invocations':0,'measured':False,'drivers':{n:digest(Path(__file__).with_name(n)) for n in ('stream.lfz','whole.lfz','reference.py')}}
    (work/'protocol.json').write_text(json.dumps(protocol,indent=2)+'\n',encoding='utf-8')
    if a.prepare_only:print(json.dumps(protocol));return
    if os.name!='nt':p.error('Windows measurement only')
    if a.binary is None:p.error('--binary required')
    exe=a.binary.resolve()
    if digest(exe)!=a.expected_sha:raise RuntimeError('candidate hash mismatch')
    results=[]
    for mib in protocol['sizes_mib']:
        data=work/f'data-{mib}.jsonl';made=generate(data,mib*1048576);checked=oracle(data)
        assert all(made[k]==checked[k] for k in ('rows','bytes','sha256'))
        (work/f'data-{mib}-manifest.json').write_text(json.dumps({'generator':made,'independent_oracle':checked},indent=2)+'\n',encoding='utf-8')
        modes=['stream','python','whole'] if mib==10 else ['stream','python']
        for run in range(6):
            for mode in modes:
                if digest(exe)!=a.expected_sha:raise RuntimeError('candidate changed')
                cmd=[sys.executable,'-B',str(Path(__file__).with_name('reference.py')),str(data),'hold'] if mode=='python' else [str(exe),str(Path(__file__).with_name(mode+'.lfz')), '--',str(data),'hold']
                raw=sample(cmd,checked,work/f'{mode}-{mib}-{run}.json',a.timeout)
                raw.update(mode=mode,mib=mib,warmup=run==0,input_bytes=made['bytes'],rows=checked['rows'])
                results.append(raw);(work/'all-samples.json').write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    assert digest(exe)==a.expected_sha
    summary=[]
    for mib in protocol['sizes_mib']:
        for mode in (['stream','python','whole'] if mib==10 else ['stream','python']):
            rows=[r for r in results if r['mib']==mib and r['mode']==mode and not r['warmup']];times=[r['processing_wall_ns']/1e9 for r in rows]
            summary.append({'mode':mode,'mib':mib,'seconds_median':statistics.median(times),'seconds_min':min(times),'seconds_max':max(times),'peak_ws_bytes_all':[r['peak_working_set_bytes'] for r in rows],'mib_s_median':rows[0]['input_bytes']/1048576/statistics.median(times)})
    (work/'summary.json').write_text(json.dumps({'binary_sha':a.expected_sha,'measured':True,'raw_samples':len(results),'summary':summary,'r304':'OPEN; not this protocol'},indent=2)+'\n',encoding='utf-8');print(json.dumps(summary))
if __name__=='__main__':main()