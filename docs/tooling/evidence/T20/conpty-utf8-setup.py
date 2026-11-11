from pathlib import Path
D=Path('Temp/T20-tooling');s=(D/'conpty-errors-cursor-fixed.py').read_text(encoding='utf-8').replace('/d /s /c ""','/d /s /c "chcp 65001 >nul & "').replace('conpty-errors-cursor-fixed','conpty-errors-utf8');(D/'conpty-errors-utf8.py').write_text(s,encoding='utf-8')
