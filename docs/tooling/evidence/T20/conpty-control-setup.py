from pathlib import Path
import shutil
D=Path('Temp/T20-tooling')
for name in ['conpty.py','conpty-result.json','conpty-raw.bin','conpty-text.txt']:
 shutil.copy2(D/name,D/('first-'+name))
s=(D/'conpty-diagnostic.py').read_text(encoding='utf-8')
s=s.replace("exe=str(R/'target/release/lfz.exe');cmd=C.create_unicode_buffer('\"'+exe+'\" repl')", "exe=r'C:\\Windows\\System32\\cmd.exe';cmd=C.create_unicode_buffer('\"'+exe+'\" /d /c echo CONPTY_CONTROL')")
a=s.index(" mark=expect('lfz> ')");b=s.index('\nexcept Exception',a)
s=s[:a]+''' mark=expect('CONPTY_CONTROL');assert wait(pi.process,4000)==0;code=W.DWORD();ok(exitcode(pi.process,C.byref(code)));assert code.value==0
 result.update(status='PASS',exit=code.value,events=events,pid=pi.pid,probe='system cmd console stdout control')
'''+s[b:]
s=s.replace('conpty-diagnostic','conpty-control')
(D/'conpty-control.py').write_text(s,encoding='utf-8')
