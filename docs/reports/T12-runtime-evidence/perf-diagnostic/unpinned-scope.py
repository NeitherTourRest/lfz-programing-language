from pathlib import Path
import ctypes,ctypes.wintypes as w,subprocess,os,time,json,hashlib,statistics,shutil,struct
K=ctypes.WinDLL('kernel32',use_last_error=True)
K.GetCurrentProcess.restype=w.HANDLE
K.GetProcessAffinityMask.argtypes=[w.HANDLE,ctypes.POINTER(ctypes.c_size_t),ctypes.POINTER(ctypes.c_size_t)]
K.SetProcessAffinityMask.argtypes=[w.HANDLE,ctypes.c_size_t]
K.GetPriorityClass.argtypes=[w.HANDLE];K.GetPriorityClass.restype=w.DWORD
K.GetProcessTimes.argtypes=[w.HANDLE,ctypes.POINTER(w.FILETIME),ctypes.POINTER(w.FILETIME),ctypes.POINTER(w.FILETIME),ctypes.POINTER(w.FILETIME)]
K.GetLogicalProcessorInformationEx.argtypes=[ctypes.c_int,w.LPVOID,ctypes.POINTER(w.DWORD)]
mask=ctypes.c_size_t();system=ctypes.c_size_t();assert K.GetProcessAffinityMask(K.GetCurrentProcess(),ctypes.byref(mask),ctypes.byref(system))
size=w.DWORD();K.GetLogicalProcessorInformationEx(0,None,ctypes.byref(size));buf=ctypes.create_string_buffer(size.value);assert K.GetLogicalProcessorInformationEx(0,buf,ctypes.byref(size));data=buf.raw;pos=0;cores=[]
while pos<len(data):
 rel,n=struct.unpack_from('<II',data,pos)
 flags,eff=struct.unpack_from('<BB',data,pos+8);count=struct.unpack_from('<H',data,pos+30)[0]
 groups=[]
 for i in range(count):
  bits,group=struct.unpack_from('<QH',data,pos+32+i*16);groups.append({'mask':bits,'group':group})
 cores.append({'flags':flags,'efficiency_class':eff,'groups':groups});pos+=n
byclass={}
for core in cores:
 for g in core['groups']:
  allowed=g['mask']&mask.value
  if allowed and g['group']==0:byclass.setdefault(core['efficiency_class'],(allowed&-allowed).bit_length()-1)
cpus=[min(byclass.values())]
root=Path.cwd();dest=root/'Temp/runtime-t12/affinity-subject.exe';env=os.environ.copy();env['TEMP']=env['TMP']=str(root/'Temp/runtime-t12')
binaries={'old':'Temp/t12-perf/lfz-before.exe','new':'Temp/runtime-t12/perf-lab/root-bad.exe','scope-elision':'Temp/runtime-t12/perf-lab/scope-elision/target/release/lfz.exe'}

rows=[]
for n in [100000,500000]:
 for rnd in range(9):
  for label in (list(binaries) if rnd%2==0 else list(binaries)[::-1]):
   shutil.copyfile(binaries[label],dest)
   start=time.perf_counter_ns()
   p=subprocess.Popen([str(dest),'run','benchmarks/lfz/numeric_loop.lfz'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env)
   actual=ctypes.c_size_t();sysmask=ctypes.c_size_t();assert K.GetProcessAffinityMask(w.HANDLE(p._handle),ctypes.byref(actual),ctypes.byref(sysmask))
   priority=K.GetPriorityClass(w.HANDLE(p._handle));out,err=p.communicate(f'{n}\n'.encode(),timeout=10);elapsed=(time.perf_counter_ns()-start)/1e6
   times=[w.FILETIME() for _ in range(4)];assert K.GetProcessTimes(w.HANDLE(p._handle),*[ctypes.byref(t) for t in times])
   cpu_ms=sum((t.dwHighDateTime<<32)+t.dwLowDateTime for t in times[2:])/10000
   assert p.returncode==0 and out.decode().strip()==f'RESULT total={n*(n+1)//2}'
   rows.append({'n':n,'round':rnd,'label':label,'warmup':rnd<2,'wall_ms':elapsed,'cpu_ms':cpu_ms,'affinity':actual.value,'priority':priority,'exit':p.returncode,'stdout':out.decode(),'stderr':err.decode()})
result={'pinned':False,'allowed_mask':mask.value,'binaries':{k:{'path':v,'sha256':hashlib.sha256(Path(v).read_bytes()).hexdigest()} for k,v in binaries.items()},'subject':str(dest),'rows':rows,'medians':[]}
for n in [100000,500000]:
 for label in binaries:
  selected=[r for r in rows if r['n']==n and r['label']==label and not r['warmup']]
  row={'n':n,'label':label,'wall_ms':statistics.median(r['wall_ms'] for r in selected),'cpu_ms':statistics.median(r['cpu_ms'] for r in selected)};result['medians'].append(row);print(row)
p=root/'Temp/runtime-t12/unpinned-scope.json';assert not p.exists();p.write_text(json.dumps(result,indent=2),encoding='utf8')
