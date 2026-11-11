# Local Windows ConPTY acceptance probe. No product build or external dependencies.
from pathlib import Path
import ctypes as C,ctypes.wintypes as W,threading,time,re,json,os,hashlib
R=Path.cwd();D=R/'Temp/T20-tooling';K=C.WinDLL('kernel32',use_last_error=True);H=W.HANDLE;P=C.c_void_p
class Coord(C.Structure):_fields_=[('x',C.c_short),('y',C.c_short)]
class SI(C.Structure):_fields_=[('cb',W.DWORD),('reserved',W.LPWSTR),('desktop',W.LPWSTR),('title',W.LPWSTR),('x',W.DWORD),('y',W.DWORD),('xs',W.DWORD),('ys',W.DWORD),('xc',W.DWORD),('yc',W.DWORD),('fill',W.DWORD),('flags',W.DWORD),('show',W.WORD),('reserved_n',W.WORD),('reserved_p',P),('input',H),('output',H),('error',H)]
class SX(C.Structure):_fields_=[('si',SI),('attrs',P)]
class PI(C.Structure):_fields_=[('process',H),('thread',H),('pid',W.DWORD),('tid',W.DWORD)]
def bind(n,a,r):f=getattr(K,n);f.argtypes=a;f.restype=r;return f
pipe=bind('CreatePipe',[C.POINTER(H),C.POINTER(H),P,W.DWORD],W.BOOL);close=bind('CloseHandle',[H],W.BOOL)
createpty=bind('CreatePseudoConsole',[Coord,H,H,W.DWORD,C.POINTER(H)],C.c_long);closepty=bind('ClosePseudoConsole',[H],None)
init=bind('InitializeProcThreadAttributeList',[P,W.DWORD,W.DWORD,C.POINTER(C.c_size_t)],W.BOOL);update=bind('UpdateProcThreadAttribute',[P,W.DWORD,C.c_size_t,P,C.c_size_t,P,P],W.BOOL);delete=bind('DeleteProcThreadAttributeList',[P],None)
spawn=bind('CreateProcessW',[W.LPCWSTR,W.LPWSTR,P,P,W.BOOL,W.DWORD,P,W.LPCWSTR,C.POINTER(SI),C.POINTER(PI)],W.BOOL)
read=bind('ReadFile',[H,P,W.DWORD,C.POINTER(W.DWORD),P],W.BOOL);write=bind('WriteFile',[H,P,W.DWORD,C.POINTER(W.DWORD),P],W.BOOL)
wait=bind('WaitForSingleObject',[H,W.DWORD],W.DWORD);kill=bind('TerminateProcess',[H,W.UINT],W.BOOL);exitcode=bind('GetExitCodeProcess',[H,C.POINTER(W.DWORD)],W.BOOL)
def ok(v):
 if not v:raise C.WinError(C.get_last_error())
ir,iw,orr,ow,pc=H(),H(),H(),H(),H();pi=PI();data=bytearray();events=[];lock=threading.Lock();done=threading.Event();owned=[];attrs=None;worker=None
result={'status':'FAIL','binary_sha256':hashlib.sha256((R/'target/release/lfz.exe').read_bytes()).hexdigest()}
def watchdog():
 if done.is_set():return
 (D/'conpty-final-cursor-fixed-watchdog.txt').write_text('15s watchdog; terminate only own CreateProcess handle; abort probe\n',encoding='utf-8')
 if pi.process:kill(pi.process,124)
 os._exit(124)
timer=threading.Timer(15,watchdog);timer.daemon=True;timer.start()
def reader():
 buf=C.create_string_buffer(4096);n=W.DWORD()
 while read(orr,buf,4096,C.byref(n),None):
  if n.value==0:break
  with lock:data.extend(buf.raw[:n.value])
def text():
 with lock:raw=bytes(data)
 decoded=re.sub(r'\x1b\[(\d*)C',lambda m:' '*int(m[1] or '1'),raw.decode('utf-8','replace'))
 return re.sub(r'\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07]*(?:\x07|\x1b\\)|[@-_])','',decoded)
def expect(fragment,start=0):
 end=time.monotonic()+4
 while time.monotonic()<end:
  value=text()
  if fragment in value[start:]:events.append({'observed_before_next_input':fragment,'text_chars':len(value)});return value.index(fragment,start)+len(fragment)
  time.sleep(.005)
 raise AssertionError((fragment,text()))
def send(s):
 b=s.encode('utf-8');n=W.DWORD();ok(write(iw,b,len(b),C.byref(n),None));assert n.value==len(b)
try:
 ok(pipe(C.byref(ir),C.byref(iw),None,0));owned.extend([ir,iw]);ok(pipe(C.byref(orr),C.byref(ow),None,0));owned.extend([orr,ow])
 hr=createpty(Coord(100,30),ir,ow,0,C.byref(pc));assert hr==0,hex(hr)
 worker=threading.Thread(target=reader);worker.start()
 size=C.c_size_t();init(None,1,0,C.byref(size));attrs=C.create_string_buffer(size.value);ok(init(attrs,1,0,C.byref(size)));ok(update(attrs,0,0x20016,pc,C.sizeof(H),None,None))
 sx=SX();sx.si.cb=C.sizeof(SX);sx.attrs=C.cast(attrs,P);exe=r'C:\Windows\System32\cmd.exe';lfz=str(R/'target/release/lfz.exe');cmd=C.create_unicode_buffer('"'+exe+'" /d /s /c ""'+lfz+'" repl < CONIN$ > CONOUT$ 2>&1"')
 ok(spawn(exe,cmd,None,None,False,0x80000,None,str(R),C.cast(C.byref(sx),C.POINTER(SI)),C.byref(pi)))
 for h in [ir,ow]:ok(close(h));owned.remove(h)
 mark=expect('lfz> ');send('fn f() {\r');mark=expect('...> ',mark);send('return 17\r');mark=expect('...> ',mark);send('}\r');mark=expect('lfz> ',mark)
 send('f()\r');mark=expect('17',mark);mark=expect('lfz> ',mark)
 send('input("data> ")\r');mark=expect('data> ',mark);send('hello\r');mark=expect('hello',mark);mark=expect('lfz> ',mark)
 send(':quit\r');assert wait(pi.process,4000)==0;code=W.DWORD();ok(exitcode(pi.process,C.byref(code)));assert code.value==0,code.value
 result.update(status='PASS',exit=code.value,events=events,pid=pi.pid,main_and_continuation_prompts_observed_before_sending_next_line=True,input_prompt_observed_before_data=True)
except Exception as error:
 result['exception']=repr(error)
 if pi.process:
  code=W.DWORD();exitcode(pi.process,C.byref(code));result['exit_at_failure']=code.value
  if code.value==259:
   send('1\r:quit\r');result['diagnostic_wait']=wait(pi.process,2000);exitcode(pi.process,C.byref(code));result['exit_after_diagnostic_input']=code.value
 raise
finally:
 if pi.process and wait(pi.process,0)!=0:kill(pi.process,125);wait(pi.process,2000)
 if attrs is not None:delete(attrs)
 if pc:closepty(pc)
 if worker is not None:worker.join(2);assert not worker.is_alive(),'ConPTY reader remained live'
 for h in owned:close(h)
 if pi.thread:close(pi.thread)
 done.set();timer.cancel();timer.join(1)
 if pi.process:close(pi.process)
 (D/'conpty-final-cursor-fixed-raw.bin').write_bytes(data);(D/'conpty-final-cursor-fixed-text.txt').write_text(text(),encoding='utf-8');(D/'conpty-final-cursor-fixed-result.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
print(json.dumps(result))
