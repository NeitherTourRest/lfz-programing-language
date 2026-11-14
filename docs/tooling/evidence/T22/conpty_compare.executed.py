# Local Windows ConPTY acceptance probe. No product build or external dependencies.
from pathlib import Path
import ctypes as C,ctypes.wintypes as W,threading,time,re,json,os,hashlib
R=Path.cwd();D=R/'Temp/T22-tooling';K=C.WinDLL('kernel32',use_last_error=True);H=W.HANDLE;P=C.c_void_p
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
import msvcrt, subprocess, sys

console_mode = bind('GetConsoleMode', [H, C.POINTER(W.DWORD)], W.BOOL)
set_input_cp = bind('SetConsoleCP', [W.UINT], W.BOOL)
set_output_cp = bind('SetConsoleOutputCP', [W.UINT], W.BOOL)
get_input_cp = bind('GetConsoleCP', [], W.UINT)
get_output_cp = bind('GetConsoleOutputCP', [], W.UINT)
create_file = bind('CreateFileW', [W.LPCWSTR, W.DWORD, W.DWORD, P, W.DWORD, W.DWORD, H], H)

cp = int(sys.argv[-2])
variant = sys.argv[-1]
prefix = D / f'{cp}-{variant}'
exe = R / 'dist/lfz.exe'

if sys.argv[1] == '--child':
    mode_status = []
    supported = bool(set_input_cp(cp)) and bool(set_output_cp(cp))
    meta = {'requested_cp': cp, 'input_cp': get_input_cp(), 'output_cp': get_output_cp(), 'supported': supported, 'handles': mode_status}
    if supported:
        descriptors = []
        for device, access in [('CONIN$', 0xc0000000), ('CONOUT$', 0x40000000 if variant == 'write' else 0xc0000000), ('CONOUT$', 0x40000000 if variant == 'write' else 0xc0000000)]:
            handle = create_file(device, access, 3, None, 3, 0, None)
            if handle == C.c_void_p(-1).value:
                raise C.WinError(C.get_last_error())
            value = W.DWORD()
            detected = bool(console_mode(handle, C.byref(value)))
            mode_status.append({'device': device, 'access': hex(access), 'mode_ok': detected, 'mode': value.value, 'error': 0 if detected else C.get_last_error()})
            descriptors.append(msvcrt.open_osfhandle(handle, os.O_BINARY | (os.O_WRONLY if access == 0x40000000 else os.O_RDWR)))
    prefix.with_suffix('.handles.json').write_text(json.dumps(meta, indent=2), encoding='utf8')
    if not supported:
        sys.exit(77)
    try:
        result = subprocess.run([str(exe), 'repl'], stdin=descriptors[0], stdout=descriptors[1], stderr=descriptors[2], timeout=9)
    finally:
        for fd in descriptors:
            os.close(fd)
    sys.exit(result.returncode)

ir, iw, out_r, ow, pc = H(), H(), H(), H(), H()
pi = PI()
owned = []
attrs = None
attrs_ready = False
reader_thread = None
captured = bytearray()
done = threading.Event()
report = {'binary_sha256': hashlib.sha256(exe.read_bytes()).hexdigest()}

def watchdog():
    if not done.is_set():
        prefix.with_suffix('.watchdog').write_text('15s timeout; own process only\n', encoding='utf8')
        if pi.process:
            kill(pi.process, 124)
        os._exit(124)

def drain():
    buf = C.create_string_buffer(4096)
    n = W.DWORD()
    while read(out_r, buf, 4096, C.byref(n), None) and n.value:
        captured.extend(buf.raw[:n.value])

timer = threading.Timer(15, watchdog)
timer.daemon = True
timer.start()
try:
    ok(pipe(C.byref(ir), C.byref(iw), None, 0)); owned.extend([ir, iw])
    ok(pipe(C.byref(out_r), C.byref(ow), None, 0)); owned.extend([out_r, ow])
    hr = createpty(Coord(120, 40), ir, ow, 0, C.byref(pc))
    if hr != 0:
        raise OSError(f'CreatePseudoConsole HRESULT {hr:#x}')
    reader_thread = threading.Thread(target=drain)
    reader_thread.start()
    size = C.c_size_t()
    init(None, 1, 0, C.byref(size))
    attrs = C.create_string_buffer(size.value)
    ok(init(attrs, 1, 0, C.byref(size)))
    attrs_ready = True
    ok(update(attrs, 0, 0x20016, pc, C.sizeof(H), None, None))
    sx = SX(); sx.si.cb = C.sizeof(SX); sx.attrs = C.cast(attrs, P)
    cmd = C.create_unicode_buffer(subprocess.list2cmdline([sys.executable, '-B', str(Path(__file__).resolve()), '--child', str(cp), variant]))
    ok(spawn(sys.executable, cmd, None, None, False, 0x80000, None, str(R), C.cast(C.byref(sx), C.POINTER(SI)), C.byref(pi)))
    for handle in [ir, ow]:
        ok(close(handle)); owned.remove(handle)
    payload = ':unknown\rprint("RESULT<"+input()+">")\r你好LFZ\rlet = "中文源码"\r:reset\r:quit\r'.encode('utf8')
    prefix.with_suffix('.input.bin').write_bytes(payload)
    n = W.DWORD(); ok(write(iw, payload, len(payload), C.byref(n), None)); assert n.value == len(payload)
    assert wait(pi.process, 11000) == 0, 'child did not exit'
    code = W.DWORD(); ok(exitcode(pi.process, C.byref(code))); report['exit'] = code.value
finally:
    if pi.process and wait(pi.process, 0) != 0:
        kill(pi.process, 125); wait(pi.process, 2000)
    if attrs_ready: delete(attrs)
    if pc: closepty(pc)
    if reader_thread:
        reader_thread.join(2)
        assert not reader_thread.is_alive()
    for handle in owned: close(handle)
    if pi.thread: close(pi.thread)
    done.set(); timer.cancel(); timer.join(1)
    if pi.process: close(pi.process)
    prefix.with_suffix('.raw.bin').write_bytes(captured)
    rendered = re.sub(r'\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07]*(?:\x07|\x1b\\)|[@-_])', '', captured.decode('utf8', errors='replace'))
    prefix.with_suffix('.text.txt').write_text(rendered, encoding='utf8')
    report.update(strict_utf8=bytes(captured).decode('utf8',errors='replace').encode('utf8') == captured,
                  unknown_message='repl: 未知命令' in rendered,
                  input_result='RESULT<你好LFZ>' in rendered,
                  diagnostic_source=bool(re.search(r'File "<repl:\d+>", line 1\s+let = "中文源码"', rendered)),
                  syntax_error='SyntaxError:' in rendered)
    prefix.with_suffix('.result.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n', encoding='utf8')
print(json.dumps(report, ensure_ascii=True))
