from pathlib import Path
s=Path('Temp/T20-tooling/directed.py').read_text(encoding='utf-8-sig').replace("('first-'+name", "('second-'+name")
Path('Temp/T20-tooling/directed-second.py').write_text(s,encoding='utf-8')
s=Path('Temp/T19-tooling/watch.py').read_text(encoding='utf-8-sig').replace("W=R/'Temp/T19-tooling'","W=R/'Temp/T20-tooling'").replace("'test_runner-a9d3da191eeb2958.exe'","'cli_t20-be52399e4be58bfb.exe','test_runner-a9d3da191eeb2958.exe'")
Path('Temp/T20-tooling/watch.py').write_text(s,encoding='utf-8')
