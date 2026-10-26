"""Whole-process wall time and Windows process user/kernel accounting."""
import ctypes, ctypes.wintypes, os, subprocess, time

def capture(command, **kwargs):
    start=time.perf_counter_ns()
    p=subprocess.Popen(command,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,**{k:v for k,v in kwargs.items() if k not in ('input','timeout')})
    try:
        stdout,stderr=p.communicate(kwargs.get('input'),timeout=kwargs.get('timeout',15))
    except subprocess.TimeoutExpired:
        p.kill(); p.communicate(); raise
    elapsed_ms=(time.perf_counter_ns()-start)/1e6
    cpu={'user_ms':None,'kernel_ms':None,'accounting':'not available on this platform'}
    if os.name=='nt':
        api=ctypes.WinDLL('kernel32',use_last_error=True)
        api.GetProcessTimes.argtypes=[ctypes.c_void_p]+[ctypes.POINTER(ctypes.wintypes.FILETIME)]*4
        api.GetProcessTimes.restype=ctypes.c_int
        vals=[ctypes.wintypes.FILETIME() for _ in range(4)]
        if not api.GetProcessTimes(int(p._handle),*[ctypes.byref(v) for v in vals]): raise ctypes.WinError(ctypes.get_last_error())
        ticks=[(v.dwHighDateTime<<32)+v.dwLowDateTime for v in vals]
        cpu={'user_ms':ticks[3]/10000,'kernel_ms':ticks[2]/10000,'accounting':'GetProcessTimes; resolution may quantize short tasks to zero'}
    return subprocess.CompletedProcess(command,p.returncode,stdout,stderr),elapsed_ms,cpu