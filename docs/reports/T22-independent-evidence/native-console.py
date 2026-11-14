from pathlib import Path
import ctypes as c
from ctypes import wintypes as t
import subprocess,sys,os,hashlib,json
K=c.WinDLL('kernel32',use_last_error=True)
class Pos(c.Structure):_fields_=[('X',c.c_short),('Y',c.c_short)]
class Rect(c.Structure):_fields_=[('Left',c.c_short),('Top',c.c_short),('Right',c.c_short),('Bottom',c.c_short)]
class Screen(c.Structure):_fields_=[('size',Pos),('cursor',Pos),('attr',t.WORD),('window',Rect),('maxwindow',Pos)]
class EventChar(c.Union):_fields_=[('unicode',c.c_wchar),('ascii',c.c_char)]
class Key(c.Structure):_fields_=[('down',t.BOOL),('repeat',t.WORD),('vk',t.WORD),('scan',t.WORD),('char',EventChar),('state',t.DWORD)]
class Event(c.Structure):_fields_=[('typ',t.WORD),('key',Key)]
def api(name,args,ret=t.BOOL):
 f=getattr(K,name);f.argtypes=args;f.restype=ret;return f
std=api('GetStdHandle',[t.DWORD],t.HANDLE); mode=api('GetConsoleMode',[t.HANDLE,c.POINTER(t.DWORD)]); ft=api('GetFileType',[t.HANDLE],t.DWORD); info=api('GetConsoleScreenBufferInfo',[t.HANDLE,c.POINTER(Screen)]); read=api('ReadConsoleOutputCharacterW',[t.HANDLE,t.LPWSTR,t.DWORD,Pos,c.POINTER(t.DWORD)]); fill=api('FillConsoleOutputCharacterW',[t.HANDLE,t.WCHAR,t.DWORD,Pos,c.POINTER(t.DWORD)]); cursor=api('SetConsoleCursorPosition',[t.HANDLE,Pos]); setcp=api('SetConsoleOutputCP',[t.UINT]); getcp=api('GetConsoleOutputCP',[],t.UINT); create=api('CreateFileW',[t.LPCWSTR,t.DWORD,t.DWORD,t.LPVOID,t.DWORD,t.DWORD,t.HANDLE],t.HANDLE); events=api('WriteConsoleInputW',[t.HANDLE,c.POINTER(Event),t.DWORD,c.POINTER(t.DWORD)]); flush=api('FlushConsoleInputBuffer',[t.HANDLE]); close=api('CloseHandle',[t.HANDLE]); title=api('SetConsoleTitleW',[t.LPCWSTR]);
def ck(v):
 if not v:raise c.WinError(c.get_last_error())
 return v
def output_handle(access):
 h=create('CONOUT$',access,3,None,3,0,None)
 if h==c.c_void_p(-1).value:raise c.WinError(c.get_last_error())
 return h
def check_handle(h):
 m=t.DWORD();c.set_last_error(0); got=mode(h,c.byref(m));return dict(type=ft(h),console_mode=bool(got),mode=m.value if got else None,mode_error=c.get_last_error() if not got else None)
root=Path.cwd(); w=root/'Temp/T22-verifier'; x=root/'dist/lfz.exe'; sha=lambda b:hashlib.sha256(b).hexdigest(); env=dict(os.environ,TEMP=str(w),TMP=str(w))
def verify():
 import msvcrt
 rows=[]; details={}; report=w/'ordinary-console.json'; original_cp=getcp(); h=None
 try:
  ck(original_cp); ck(title('LFZ private verifier'))
  protocol=[check_handle(std(i)) for i in (-10,-11,-12)]; assert all(n['console_mode'] for n in protocol),protocol
  h=output_handle(0xc0000000); test=output_handle(0x40000000); details.update(original_cp=original_cp,ordinary=protocol,explicit_write_only=check_handle(test),read_write_console=check_handle(h));close(test)
  ok=w/'print.lfz';ok.write_bytes('#42\nprint("中文雪😀")\n'.encode());bad=w/'error.lfz';bad.write_bytes(b'#42\nmissing()\n')
  for cp in [437,936,65001]:
   if not setcp(cp):rows.append(dict(cp=cp,error=c.get_last_error()));continue
   for variant in ['ordinary-out','ordinary-error','ordinary-repl','write-only-out','write-only-error']:
    start=Pos(0,0);s=Screen();ck(info(h,c.byref(s)));n=t.DWORD();ck(fill(h,' ',s.size.X*min(s.size.Y,30),start,c.byref(n)));ck(cursor(h,start)); p=None
    if variant=='ordinary-repl':
     hi=std(-10);ck(flush(hi)); seq='argv()[0]\r:quit\r'; es=(Event*(len(seq)*2))();
     for i,ch in enumerate(seq):
      for j,val in enumerate([1,0]):
       ev=es[i*2+j];ev.typ=1;ev.key.down=val;ev.key.repeat=1;ev.key.vk=13 if ch=='\r' else 0;ev.key.char.unicode=ch
     ck(events(hi,es,len(es),c.byref(n)));assert n.value==len(es)
     p=subprocess.run([str(x),'repl','--','中文雪😀'],cwd=w,env=env,timeout=8)
    elif variant.startswith('write-only'):
     raw=output_handle(0x40000000); fd=msvcrt.open_osfhandle(raw,os.O_WRONLY); file=os.fdopen(fd,'wb',buffering=0)
     with file:
      kwargs={'stdout':file} if variant.endswith('out') else {'stderr':file}; p=subprocess.run([str(x),str(ok if variant.endswith('out') else bad)],cwd=w,env=env,timeout=8,**kwargs)
    else:p=subprocess.run([str(x),str(ok if variant.endswith('out') else bad)],cwd=w,env=env,timeout=8)
    cs=Screen();ck(info(h,c.byref(cs)));buf=c.create_unicode_buffer(cs.size.X*(max(cs.cursor.Y+1,4)));ck(read(h,buf,len(buf)-1,start,c.byref(n)));raw=bytes(c.string_at(c.addressof(buf),n.value*2)); name=f'{cp}-{variant}';(w/f'{name}.utf16le').write_bytes(raw)
    text=raw.decode('utf-16le',errors='replace'); line=[''.join(text[i:i+cs.size.X]).rstrip() for i in range(0,len(text),cs.size.X)]; rows.append(dict(cp=cp,actual_cp=getcp(),variant=variant,exit=p.returncode,text='\n'.join(line).rstrip(),raw_sha256=sha(raw)))
 except Exception as ex:details['harness_error']=repr(ex)
 finally:
  if original_cp:setcp(original_cp)
  if h:close(h)
  report.write_text(json.dumps(dict(binary_sha256=sha(x.read_bytes()),handles=details,results=rows),ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if sys.argv[-1]=='--child':verify()
else:
 s=subprocess.STARTUPINFO();s.dwFlags|=subprocess.STARTF_USESHOWWINDOW;s.wShowWindow=0
 p=subprocess.run([sys.executable,'-B',__file__,'--child'],cwd=root,env=env,startupinfo=s,creationflags=subprocess.CREATE_NEW_CONSOLE,timeout=55)
 print((w/'ordinary-console.json').read_text(encoding='utf8').encode('ascii','backslashreplace').decode('ascii'))
