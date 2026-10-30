import ctypes,hashlib,json,os,subprocess
from pathlib import Path
R=Path.cwd();E=R/'docs/reports/T18-A-independent-evidence';W=R/'Temp/T18-A-verifier/own';W.mkdir(parents=True,exist_ok=True);X=R/'target/release/lfz.exe';F=R/'Temp/T18-A-tooling/fixture.exe';Q=lambda s:json.dumps(str(s),ensure_ascii=False);rows=[]
def call(mode,args=[],limit=1000,ms=1000):return 'exec(['+','.join(Q(v) for v in [F.as_posix(),mode,*args])+'],'+str(ms)+','+str(limit)+')'
def test(name,src,error=None,flags=[],allow=True):
 args=[str(X),'--json']+(['--allow-process'] if allow else [])+['-e',src,*flags];p=subprocess.run(args,capture_output=True,timeout=8,cwd=W);d=json.loads(p.stdout);row=dict(name=name,args=args,exit=p.returncode,stdout=p.stdout.decode('utf8'),stderr=p.stderr.decode('utf8'));rows.append(row);assert p.returncode==(2 if error else 0),row;assert not p.stderr,row
 if error:assert d['message']==error,row
 row['passed']=True;return d
for n in [0,13,14]:test('dual_'+str(n),call('dual',['7'],n),'子进程输出超过字节上限' if n<14 else None)
test('dual_large','let p='+call('dual',['100000'],200000)+'\nassert(len(p.stdout)==100000 && len(p.stderr)==100000)')
test('tail_limit',call('dual',['100000'],199999),'子进程输出超过字节上限')
test('utf8_reclaimed',call('invalid',[],1),'子进程输出不是合法 UTF-8')
test('nonzero','assert('+call('exit',['-1'],0)+'.exitCode==4294967295)')
test('local_catch','let r=attempt(()=>'+call('sleep',['500'],0,10)+')\nassert(!r.ok && r.error.message=="子进程执行时间上限已到")')
test('global_uncatch','attempt(()=>'+call('sleep',['1000'],0,2000)+')','执行时间上限已到',['--timeout-ms','30'])
test('argv_quoting','let p='+call('echo',['','x y','雪','a\\','a"b'],100)+'\nassert(p.stdout=='+Q('0:\n3:x y\n3:雪\n2:a\\\n3:a"b\n')+')')
marker=W/'forbidden.txt';test('subsequent_session_disabled',call('mark',[marker.as_posix()],0),'子进程能力未启用',allow=False);assert not marker.exists()
pidfile=W/'desc.txt';d=test('descendant','let p='+call('parent',[pidfile.as_posix()],100,3000)+'\nassert(p.exitCode==0)');pid=int(pidfile.read_text());k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.restype=ctypes.c_void_p;k.OpenProcess.argtypes=[ctypes.c_uint32,ctypes.c_int,ctypes.c_uint32];k.WaitForSingleObject.argtypes=[ctypes.c_void_p,ctypes.c_uint32];k.CloseHandle.argtypes=[ctypes.c_void_p];h=k.OpenProcess(0x100000,0,pid)
if h:assert k.WaitForSingleObject(h,0)==0;k.CloseHandle(h)
else:assert ctypes.get_last_error()==87
rows[-1]['descendant_pid']=pid;rows[-1]['descendant_reaped']=True
(E/'own.json').write_text(json.dumps(dict(total=len(rows),passed=len(rows),fixture_sha256=hashlib.sha256(F.read_bytes()).hexdigest(),rows=rows),ensure_ascii=False,indent=2),encoding='utf8');print(len(rows),'independent actual process witnesses PASS')
