from pathlib import Path
s=Path('Temp/T18-A-close/watch.py').read_text(encoding='utf-8').replace("W=R/'Temp/T18-A-close'","W=R/'Temp/T18-B-tooling'").replace("'test_runner-a9d3da191eeb2958.exe'","'cli_t18b-4fb747188dc55a23.exe','test_runner-a9d3da191eeb2958.exe'")
Path('Temp/T18-B-tooling/watch.py').write_text(s,encoding='utf-8')
