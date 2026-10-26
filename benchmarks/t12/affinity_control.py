"""Optional Windows process-only affinity; child processes inherit at creation."""
import argparse, contextlib, ctypes, json, os

@contextlib.contextmanager
def pin(cpu):
    info={'enabled':cpu is not None,'logical_cpu':cpu,'pid':os.getpid(),'scope':'harness process and inherited child process mask; no system settings','restore_verified':None}
    if cpu is None:
        yield info
        return
    if os.name!='nt' or cpu<0: raise RuntimeError('CPU affinity control requires Windows and a nonnegative CPU index')
    api=ctypes.WinDLL('kernel32',use_last_error=True)
    api.GetCurrentProcess.argtypes=[]; api.GetCurrentProcess.restype=ctypes.c_void_p
    api.GetProcessAffinityMask.argtypes=[ctypes.c_void_p,ctypes.POINTER(ctypes.c_size_t),ctypes.POINTER(ctypes.c_size_t)]; api.GetProcessAffinityMask.restype=ctypes.c_int
    api.SetProcessAffinityMask.argtypes=[ctypes.c_void_p,ctypes.c_size_t]; api.SetProcessAffinityMask.restype=ctypes.c_int
    proc=api.GetCurrentProcess()
    def read():
        p=ctypes.c_size_t(); s=ctypes.c_size_t()
        if not api.GetProcessAffinityMask(proc,ctypes.byref(p),ctypes.byref(s)): raise ctypes.WinError(ctypes.get_last_error())
        return p.value,s.value
    original,system=read(); target=1<<cpu
    if target & system != target: raise RuntimeError('CPU unavailable in system mask')
    info.update(original_mask=original,system_mask=system,target_mask=target,inheritance_source='https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-setprocessaffinitymask')
    if not api.SetProcessAffinityMask(proc,target): raise ctypes.WinError(ctypes.get_last_error())
    try:
        active,_=read(); info['active_mask']=active
        assert active==target
        yield info
    finally:
        if not api.SetProcessAffinityMask(proc,original): raise ctypes.WinError(ctypes.get_last_error())
        restored,_=read(); info.update(restored_mask=restored,restore_verified=restored==original)
        assert info['restore_verified']

def run(main):
    ap=argparse.ArgumentParser(add_help=False); ap.add_argument('--cpu',type=int); args,_=ap.parse_known_args()
    with pin(args.cpu) as info:
        output=main(info)
    if output:
        data=json.loads(output.read_text(encoding='utf-8')); data['affinity']=info
        output.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')