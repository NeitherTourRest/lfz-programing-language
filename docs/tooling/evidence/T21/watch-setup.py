from pathlib import Path
s=Path('Temp/T20-tooling/watch.py').read_text(encoding='utf-8-sig').replace("W=R/'Temp/T20-tooling'","W=R/'Temp/T21-tooling'").replace("'test_runner-a9d3da191eeb2958.exe'","'cli_t21-f65aa2b7afce5cb1.exe','test_runner-a9d3da191eeb2958.exe'")
Path('Temp/T21-tooling/watch.py').write_text(s,encoding='utf-8')
