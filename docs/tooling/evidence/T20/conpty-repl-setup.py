from pathlib import Path
D=Path('Temp/T20-tooling');s=(D/'conpty-diagnostic.py').read_text(encoding='utf-8')
s=s.replace("exe=str(R/'target/release/lfz.exe');cmd=C.create_unicode_buffer('\"'+exe+'\" repl')", "exe=r'C:\\Windows\\System32\\cmd.exe';lfz=str(R/'target/release/lfz.exe');cmd=C.create_unicode_buffer('\"'+exe+'\" /d /s /c \"\"'+lfz+'\" repl < CONIN$ > CONOUT$ 2>&1\"')")
s=s.replace('conpty-diagnostic','conpty-console-repl')
(D/'conpty-console-repl.py').write_text(s,encoding='utf-8')
