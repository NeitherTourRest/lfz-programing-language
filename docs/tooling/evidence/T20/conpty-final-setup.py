from pathlib import Path
D=Path('Temp/T20-tooling');s=(D/'conpty-console-repl.py').read_text(encoding='utf-8');s=s.replace("return re.sub(r'\\x1b", "decoded=re.sub(r'\\x1b\\[(\\d*)C',lambda m:' '*int(m[1] or '1'),raw.decode('utf-8','replace'))\n return re.sub(r'\\x1b").replace("raw.decode('utf-8','replace'))\ndef expect", "decoded)\ndef expect")
s=s.replace('conpty-console-repl','conpty-final');(D/'conpty-final.py').write_text(s,encoding='utf-8')
s=s.replace('conpty-final','conpty-errors')
a=s.index(" mark=expect('lfz> ')");b=s.index('\nexcept Exception',a)
s=s[:a]+''' mark=expect('lfz> ');send(':unknown\\r');mark=expect('repl: 未知命令',mark);mark=expect('lfz> ',mark);send(':reset\\r');mark=expect('lfz> ',mark);send(':quit\\r');assert wait(pi.process,4000)==0;code=W.DWORD();ok(exitcode(pi.process,C.byref(code)));assert code.value==2,code.value
 result.update(status='PASS',exit=code.value,events=events,pid=pi.pid,reset_kept_error_exit=True)
'''+s[b:]
(D/'conpty-errors.py').write_text(s,encoding='utf-8')
