from pathlib import Path
D=Path('Temp/T20-tooling');s=(D/'conpty-errors-cursor-fixed.py').read_text(encoding='utf-8').replace("expect('repl: 未知命令',mark)","expect('repl: ',mark)").replace('conpty-errors-cursor-fixed','conpty-errors-ascii');(D/'conpty-errors-ascii.py').write_text(s,encoding='utf-8')
