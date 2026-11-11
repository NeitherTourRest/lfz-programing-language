from pathlib import Path
D=Path('Temp/T20-tooling');s=(D/'conpty-control.py').read_text(encoding='utf-8').replace('/d /c echo CONPTY_CONTROL','/d /c "echo CONPTY_CONTROL > CONOUT$"').replace('conpty-control','conpty-console-control');(D/'conpty-console-control.py').write_text(s,encoding='utf-8')
